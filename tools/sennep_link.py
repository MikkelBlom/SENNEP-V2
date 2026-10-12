#!/usr/bin/env python3
"""SENNEP Pi <-> Mega link — Python reference implementation + bench tool.

This is the independent second implementation of the protocol (the C core is the first), used by the host tests,
and a command-line tool for testing at the robot. The final Pi application language is not decided yet
(docs/electronics/PI_MEGA_LINK.md §0); this module is fine to use from Python in the meantime.

Bench use on the Pi (needs `sudo apt install python3-serial`):
  python3 tools/sennep_link.py -p /dev/ttyAMA0 monitor           # print everything the Mega sends
  python3 tools/sennep_link.py -p /dev/ttyAMA0 latency           # round-trip time via TIME_SYNC
  python3 tools/sennep_link.py -p /dev/ttyAMA0 wheels 200 0 0 0 2   # FL at 20 % for 2 s (wheels in the air!)
  python3 tools/sennep_link.py -p /dev/ttyAMA0 drive 150 0 0 2      # 150 mm/s forward for 2 s
  python3 tools/sennep_link.py -p /dev/ttyAMA0 watchdog          # arm, go silent, verify the Mega stops by itself
  python3 tools/sennep_link.py -p /dev/ttyAMA0 param MOTOR_INVERT_MASK 5
"""
import argparse
import struct
import sys
import threading
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sennep_msgs as M  # noqa: E402  (generated)

HEADER, CRC_LEN = 2, 2
MAX_RAW = HEADER + M.MAX_PAYLOAD + CRC_LEN


# ---------------------------------------------------------------- primitives
def crc16(data: bytes) -> int:
    """CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF). crc16(b'123456789') == 0x29B1."""
    crc = 0xFFFF
    for b in data:
        crc ^= b << 8
        for _ in range(8):
            crc = ((crc << 1) ^ 0x1021) & 0xFFFF if crc & 0x8000 else (crc << 1) & 0xFFFF
    return crc


def cobs_encode(data: bytes) -> bytes:
    out = bytearray([0])
    code_pos, code = 0, 1
    for b in data:
        if b == 0:
            out[code_pos] = code
            code_pos, code = len(out), 1
            out.append(0)
        else:
            out.append(b)
            code += 1
            if code == 0xFF:
                out[code_pos] = code
                code_pos, code = len(out), 1
                out.append(0)
    out[code_pos] = code
    return bytes(out)


def cobs_decode(data: bytes) -> bytes:
    out = bytearray()
    i = 0
    while i < len(data):
        code = data[i]
        i += 1
        if code == 0 or i + code - 1 > len(data):
            raise ValueError("bad COBS")
        block = data[i:i + code - 1]
        if 0 in block:
            raise ValueError("zero in COBS block")
        out += block
        i += code - 1
        if code != 0xFF and i < len(data):
            out.append(0)
    return bytes(out)


def frame_encode(msg_type: int, seq: int, payload: bytes = b"") -> bytes:
    if len(payload) > M.MAX_PAYLOAD:
        raise ValueError("payload too long")
    raw = bytes([msg_type, seq & 0xFF]) + payload
    raw += struct.pack("<H", crc16(raw))
    return cobs_encode(raw) + b"\x00"


class StreamDecoder:
    """Feed arbitrary byte chunks; yields (type, seq, payload) for each valid frame. Recovers at the next 0x00."""

    def __init__(self):
        self.buf = bytearray()
        self.overflow = False
        self.frames_ok = self.crc_errors = self.frame_errors = 0

    def feed(self, data: bytes):
        for b in data:
            if b != 0:
                if not self.overflow:
                    self.buf.append(b)
                    if len(self.buf) > MAX_RAW + 1:  # longer than any COBS-encoded frame: drop until 0x00
                        self.overflow = True
                        self.buf.clear()
                continue
            chunk, self.buf = bytes(self.buf), bytearray()
            if self.overflow:
                self.overflow = False
                self.frame_errors += 1
                continue
            if not chunk:
                continue
            try:
                raw = cobs_decode(chunk)
            except ValueError:
                self.frame_errors += 1
                continue
            if len(raw) < HEADER + CRC_LEN or len(raw) > MAX_RAW:
                self.frame_errors += 1
                continue
            if crc16(raw[:-2]) != struct.unpack("<H", raw[-2:])[0]:
                self.crc_errors += 1
                continue
            self.frames_ok += 1
            yield raw[0], raw[1], raw[2:-2]


# ---------------------------------------------------------------- messages
def pack(name: str, **fields) -> tuple[int, bytes]:
    mid = M.BY_NAME[name]
    _, _, fmt, flist, tail = M.MESSAGES[mid]
    values = []
    for fname, count in flist:
        if count == -1:
            continue
        v = fields[fname]
        values.extend(v if count > 1 else [v])
    payload = struct.pack("<" + fmt, *values)
    if tail:
        payload += bytes(fields[flist[-1][0]])
    return mid, payload


def unpack(mid: int, payload: bytes) -> tuple[str, dict]:
    if mid not in M.MESSAGES:
        raise KeyError(f"unknown message 0x{mid:02X}")
    name, _, fmt, flist, tail = M.MESSAGES[mid]
    size = struct.calcsize("<" + fmt)
    if len(payload) < size or (not tail and len(payload) != size):
        raise ValueError(f"{name}: bad length {len(payload)}")
    vals = list(struct.unpack("<" + fmt, payload[:size]))
    out = {}
    for fname, count in flist:
        if count == -1:
            out[fname] = payload[size:]
        elif count > 1:
            out[fname], vals = vals[:count], vals[count:]
        else:
            out[fname] = vals.pop(0)
    return name, out


# ---------------------------------------------------------------- serial link
class Link:
    """Background reader + latest value per message + keep-alive. Thread-safe enough for a bench tool."""

    def __init__(self, port: str, baud: int = 250000):
        import serial  # pyserial
        self.ser = serial.Serial(port, baud, timeout=0.01)
        self.dec = StreamDecoder()
        self.seq = 0
        self.latest: dict[str, dict] = {}
        self.callbacks = []
        self.last_rx = 0.0
        self._lock = threading.Lock()
        self._drive = None  # (name, fields) re-sent by the keep-alive thread
        self._stop = threading.Event()
        threading.Thread(target=self._reader, daemon=True).start()
        threading.Thread(target=self._keepalive, daemon=True).start()

    def close(self):
        self._stop.set()
        time.sleep(0.05)
        self.ser.close()

    def send(self, name: str, **fields):
        mid, payload = pack(name, **fields)
        with self._lock:
            self.ser.write(frame_encode(mid, self.seq, payload))
            self.seq = (self.seq + 1) & 0xFF

    def hold(self, name=None, **fields):
        """Keep re-sending a drive command at 20 Hz (None: send HEARTBEAT instead)."""
        self._drive = (name, fields) if name else None

    def wait_for(self, name: str, timeout: float = 1.0, after: float = 0.0):
        end = time.time() + timeout
        while time.time() < end:
            m = self.latest.get(name)
            if m and m["_rx"] > after:
                return m
            time.sleep(0.002)
        return None

    def _reader(self):
        while not self._stop.is_set():
            try:
                data = self.ser.read(256)
            except Exception:
                return
            now = time.time()
            for mid, seq, payload in self.dec.feed(data):
                try:
                    name, fields = unpack(mid, payload)
                except (KeyError, ValueError) as e:
                    print("!", e)
                    continue
                fields["_rx"], fields["_seq"] = now, seq
                self.latest[name] = fields
                self.last_rx = now
                for cb in self.callbacks:
                    cb(name, fields)

    def _keepalive(self):
        while not self._stop.wait(0.05):
            d = self._drive
            if d:
                self.send(d[0], **d[1])
            else:
                self.send("HEARTBEAT")


# ---------------------------------------------------------------- CLI
def state_name(v):
    return {v_: k for k, v_ in vars(M.STATE).items() if not k.startswith("_")}.get(v, v)


def bits(enum_cls, v):
    return "|".join(k for k, b in vars(enum_cls).items() if not k.startswith("_") and v & b) or "-"


def cmd_monitor(link, args):
    counts = {}
    last_print = time.time()

    def cb(name, f):
        counts[name] = counts.get(name, 0) + 1
        if name in ("HELLO", "LOG", "ACK", "NODE_FRAME"):
            print(name, {k: v for k, v in f.items() if not k.startswith("_")})
    link.callbacks.append(cb)
    while True:
        time.sleep(0.2)
        if time.time() - last_print >= 1.0:
            s = link.latest.get("STATUS")
            line = " ".join(f"{k}:{v}/s" for k, v in sorted(counts.items()))
            if s:
                line += (f" | {state_name(s['state'])} faults={bits(M.FAULT, s['faults'])} blocked={bits(M.BLOCK, s['blocked'])}"
                         f" bat={s['battery_mv']}mV loop_max={s['loop_max_us']}us rx_crc={s['rx_crc_errors']}"
                         f" rx_frame={s['rx_frame_errors']} gaps={s['rx_seq_gaps']}")
            print(line, flush=True)
            counts.clear()
            last_print = time.time()


def cmd_latency(link, args):
    rtts, offsets = [], []
    for _ in range(args.n):
        t0 = time.monotonic_ns() // 1000
        sent = time.time()
        link.send("TIME_SYNC", t_pi_us=t0 & 0xFFFFFFFF)
        r = link.wait_for("TIME_SYNC_REPLY", 0.5, after=sent)
        t1 = time.monotonic_ns() // 1000
        if r and r["t_pi_us"] == t0 & 0xFFFFFFFF:
            rtts.append(t1 - t0)
            offsets.append(r["t_mega_us"] - (t0 + t1) // 2)
        time.sleep(0.02)
    if not rtts:
        print("no replies")
        return
    rtts.sort()
    print(f"{len(rtts)}/{args.n} replies. round trip us: min {rtts[0]}  median {rtts[len(rtts)//2]}  "
          f"p95 {rtts[int(len(rtts)*0.95)-1]}  max {rtts[-1]}")


def arm(link):
    t = time.time()
    link.send("ARM")
    a = link.wait_for("ACK", 0.5, after=t)
    if not a or a["result"] != M.RESULT.OK:
        raise SystemExit(f"ARM rejected: {a}")


def cmd_wheels(link, args):
    arm(link)
    link.hold("CMD_WHEELS", duty_permille=args.duty)
    time.sleep(args.seconds)
    link.hold(None)
    link.send("DISARM")
    time.sleep(0.3)


def cmd_drive(link, args):
    arm(link)
    link.hold("CMD_DRIVE", vx_mm_s=args.vx, vy_mm_s=args.vy, wz_mrad_s=args.wz, flags=0)
    time.sleep(args.seconds)
    link.hold(None)
    link.send("DISARM")
    time.sleep(0.3)


def cmd_watchdog(link, args):
    arm(link)
    link.hold("CMD_WHEELS", duty_permille=[args.duty] * 4)
    time.sleep(1.0)
    print("going silent ...")
    link._stop.set()  # stops keep-alive (and reader); restart reader manually below
    t_silent = time.time()
    dec = link.dec
    stopped_at = None
    while time.time() - t_silent < 2.0:
        for mid, _, payload in dec.feed(link.ser.read(256)):
            name, f = unpack(mid, payload)
            if name == "MOTORS" and stopped_at is None and all(d == 0 for d in f["duty_permille"]):
                stopped_at = time.time() - t_silent
            if name == "STATUS":
                print(f"  {time.time()-t_silent:5.3f}s state {state_name(f['state'])}")
    print(f"motors at 0 after {stopped_at:.3f} s" if stopped_at is not None else "MOTORS NEVER STOPPED — DO NOT USE")


def cmd_param(link, args):
    pid = getattr(M.PARAM, args.name)
    t = time.time()
    if args.value is None:
        link.send("GET_PARAM", param=pid)
    else:
        link.send("SET_PARAM", param=pid, value=float(args.value))
    print(link.wait_for("ACK", 0.5, after=t))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("-p", "--port", default="/dev/ttyAMA0")
    ap.add_argument("-b", "--baud", type=int, default=250000)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("monitor")
    p = sub.add_parser("latency"); p.add_argument("-n", type=int, default=200)
    p = sub.add_parser("wheels"); p.add_argument("duty", type=int, nargs=4); p.add_argument("seconds", type=float)
    p = sub.add_parser("drive")
    for a in ("vx", "vy", "wz"):
        p.add_argument(a, type=int)
    p.add_argument("seconds", type=float)
    p = sub.add_parser("watchdog"); p.add_argument("--duty", type=int, default=150)
    p = sub.add_parser("param"); p.add_argument("name"); p.add_argument("value", nargs="?")
    args = ap.parse_args()

    link = Link(args.port, args.baud)
    h = link.wait_for("HELLO", 3.0) or link.wait_for("STATUS", 1.0)
    if h is None:
        raise SystemExit("no HELLO/STATUS from the Mega (wiring, baud, port, or serial console still enabled?)")
    if "proto_version" in h:
        ok = h["proto_version"] == M.PROTOCOL_VERSION and h["spec_hash"] == M.SPEC_HASH
        print(f"HELLO proto v{h['proto_version']} spec 0x{h['spec_hash']:08X} boot#{h['boot_count']} "
              f"reason {h['boot_reason']} {'OK' if ok else 'MISMATCH — reflash the Mega or regenerate'}")
    try:
        globals()["cmd_" + args.cmd](link, args)
    except KeyboardInterrupt:
        pass
    finally:
        try:
            link.hold(None)
            link.send("DISARM")
        except Exception:
            pass
        link.close()


if __name__ == "__main__":
    main()
