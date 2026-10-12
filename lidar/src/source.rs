//! Where bytes come from. The pipeline only sees `ByteSource`, so a recording or the simulator
//! can stand in for the sensor everywhere (tests, benchmarks, UI development on a PC).

use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::protocol::{Packet, RawPoint, POINTS_PER_PACKET};
use crate::sim::Simulator;

/// Process-wide monotonic clock in µs.
pub fn now_us() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_micros() as u64
}

pub trait ByteSource: Send {
    /// Replaces `buf` with the next chunk of bytes and returns the host time (µs) it arrived.
    /// `Ok(None)` means the stream ended. A live source may return an empty chunk on timeout.
    fn read_chunk(&mut self, buf: &mut Vec<u8>) -> io::Result<Option<u64>>;
    fn describe(&self) -> String;
}

// ---------------------------------------------------------------- serial

pub struct SerialSource {
    port: Box<dyn serialport::SerialPort>,
    path: String,
}

impl SerialSource {
    pub const BAUD: u32 = 230_400;

    pub fn open(path: &str) -> io::Result<Self> {
        let port = serialport::new(path, Self::BAUD)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| io::Error::other(format!("{path}: {e}")))?;
        Ok(SerialSource { port, path: path.to_string() })
    }
}

impl ByteSource for SerialSource {
    fn read_chunk(&mut self, buf: &mut Vec<u8>) -> io::Result<Option<u64>> {
        buf.resize(4096, 0);
        match self.port.read(buf) {
            Ok(n) => {
                buf.truncate(n);
                Ok(Some(now_us()))
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                buf.clear();
                Ok(Some(now_us()))
            }
            Err(e) => Err(e),
        }
    }
    fn describe(&self) -> String {
        format!("serial {}", self.path)
    }
}

// ---------------------------------------------------------------- recordings

/// Raw recording: magic, then `[host_us u64 LE][len u32 LE][bytes]` per chunk. Bit-exact replay.
pub const LDR_MAGIC: &[u8; 11] = b"SENNEPLDR1\n";

pub struct Recorder {
    out: BufWriter<File>,
}

impl Recorder {
    pub fn create(path: &Path) -> io::Result<Self> {
        let mut out = BufWriter::new(File::create(path)?);
        out.write_all(LDR_MAGIC)?;
        Ok(Recorder { out })
    }
    pub fn write(&mut self, host_us: u64, bytes: &[u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }
        self.out.write_all(&host_us.to_le_bytes())?;
        self.out.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.out.write_all(bytes)
    }
    pub fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

/// Replays recorded chunks, either as fast as possible (tests, benchmarks) or paced like the
/// original (live UI).
pub struct ReplaySource {
    chunks: Vec<(u64, Vec<u8>)>,
    next: usize,
    realtime: bool,
    looped: bool,
    started: Option<(Instant, u64)>,
    name: String,
    loop_shift_us: u64,
}

impl ReplaySource {
    pub fn open(path: &Path, realtime: bool, looped: bool) -> io::Result<Self> {
        let chunks = if path.extension().is_some_and(|e| e == "jsonl") {
            load_legacy_jsonl(path)?
        } else {
            load_ldr(path)?
        };
        Ok(Self::from_chunks(chunks, realtime, looped, path.display().to_string()))
    }

    pub fn from_chunks(chunks: Vec<(u64, Vec<u8>)>, realtime: bool, looped: bool, name: String) -> Self {
        ReplaySource { chunks, next: 0, realtime, looped, started: None, name, loop_shift_us: 0 }
    }

    pub fn total_bytes(&self) -> usize {
        self.chunks.iter().map(|c| c.1.len()).sum()
    }
}

impl ByteSource for ReplaySource {
    fn read_chunk(&mut self, buf: &mut Vec<u8>) -> io::Result<Option<u64>> {
        if self.next >= self.chunks.len() {
            if !self.looped || self.chunks.is_empty() {
                return Ok(None);
            }
            // Keep time monotonic across loops so the clock and odometry see one continuous run.
            let span = self.chunks.last().unwrap().0 - self.chunks[0].0;
            self.loop_shift_us += span + 2_400;
            self.next = 0;
        }
        let (t, bytes) = &self.chunks[self.next];
        self.next += 1;
        let t = t + self.loop_shift_us;
        if self.realtime {
            let (wall0, rec0) = *self.started.get_or_insert((Instant::now(), t));
            let due = Duration::from_micros(t - rec0);
            let el = wall0.elapsed();
            if due > el {
                std::thread::sleep(due - el);
            }
        }
        buf.clear();
        buf.extend_from_slice(bytes);
        Ok(Some(t))
    }
    fn describe(&self) -> String {
        format!("replay {}", self.name)
    }
}

pub fn load_ldr(path: &Path) -> io::Result<Vec<(u64, Vec<u8>)>> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    if !data.starts_with(LDR_MAGIC) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not a SENNEPLDR1 file"));
    }
    let mut chunks = Vec::new();
    let mut i = LDR_MAGIC.len();
    while i + 12 <= data.len() {
        let t = u64::from_le_bytes(data[i..i + 8].try_into().unwrap());
        let n = u32::from_le_bytes(data[i + 8..i + 12].try_into().unwrap()) as usize;
        i += 12;
        if i + n > data.len() {
            break; // truncated tail (recording was killed); keep what is complete
        }
        chunks.push((t, data[i..i + n].to_vec()));
        i += n;
    }
    Ok(chunks)
}

/// Converts the old Electron app's `.jsonl` (already-parsed points) back into LD19 packets, so
/// they go through the real parser. Each line is one "sweep" made of whole 12-point packets.
pub fn load_legacy_jsonl(path: &Path) -> io::Result<Vec<(u64, Vec<u8>)>> {
    let reader = BufReader::new(File::open(path)?);
    let mut chunks = Vec::new();
    let mut unwrapped: Option<(u16, u64)> = None;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let Some(sweep) = v["sweep"].as_array() else {
            continue;
        };
        for pkt in sweep.chunks(POINTS_PER_PACKET) {
            if pkt.len() < POINTS_PER_PACKET {
                continue;
            }
            let f = |p: &serde_json::Value, k: &str| p[k].as_f64().unwrap_or(0.0);
            let mut points = [RawPoint::default(); POINTS_PER_PACKET];
            for (i, p) in pkt.iter().enumerate() {
                points[i] =
                    RawPoint { dist_mm: f(p, "distance") as u16, intensity: f(p, "confidence") as u8 };
            }
            let cdeg = |a: f64| ((a * 100.0).round() as i64).rem_euclid(36_000) as u16;
            let ts = f(&pkt[0], "timestamp") as u16;
            // Host arrival time: sensor time + 2 ms transport, unwrapped across the 30 s wrap.
            let ms = match unwrapped {
                None => ts as u64,
                Some((last, acc)) => acc + ((ts as u64 + 30_000 - last as u64) % 30_000),
            };
            unwrapped = Some((ts, ms));
            let packet = Packet {
                speed_dps: f(&pkt[0], "speed") as u16,
                start_cdeg: cdeg(f(&pkt[0], "angle")),
                end_cdeg: cdeg(f(&pkt[POINTS_PER_PACKET - 1], "angle")),
                timestamp_ms: ts,
                points,
                host_us: 0,
            };
            chunks.push((ms * 1000 + 2_000, packet.encode().to_vec()));
        }
    }
    Ok(chunks)
}

// ---------------------------------------------------------------- simulator

/// Wraps the simulator as a byte stream. Bytes arrive in ~1 ms chunks with 1–3 ms of transport
/// delay, like a USB serial adapter.
pub struct SimSource {
    pub sim: Simulator,
    realtime: bool,
    started: Option<Instant>,
    pending: Vec<u8>,
    pending_t: f64,
    done: bool,
}

impl SimSource {
    pub fn new(sim: Simulator, realtime: bool) -> Self {
        SimSource { sim, realtime, started: None, pending: Vec::new(), pending_t: 0.0, done: false }
    }
}

/// Fixed host time at which the simulated run "starts", so host times are never near zero.
pub const SIM_HOST_EPOCH_US: u64 = 50_000_000;

impl ByteSource for SimSource {
    fn read_chunk(&mut self, buf: &mut Vec<u8>) -> io::Result<Option<u64>> {
        buf.clear();
        if self.done && self.pending.is_empty() {
            return Ok(None);
        }
        // Gather packets until a 1 ms USB frame boundary passes.
        while !self.done {
            match self.sim.next_packet() {
                Some((p, t)) => {
                    self.sim.wire_bytes(&p, &mut self.pending);
                    self.pending_t = t;
                    if self.pending.len() >= 47 * 2 {
                        break;
                    }
                }
                None => self.done = true,
            }
        }
        std::mem::swap(buf, &mut self.pending);
        let jitter = 0.001 + 0.002 * ((self.pending_t * 7_919.0).sin().abs());
        let t = self.pending_t + jitter;
        if self.realtime {
            let start = *self.started.get_or_insert_with(Instant::now);
            let due = Duration::from_secs_f64(t);
            let el = start.elapsed();
            if due > el {
                std::thread::sleep(due - el);
            }
        }
        Ok(Some(SIM_HOST_EPOCH_US + (t * 1e6) as u64))
    }
    fn describe(&self) -> String {
        "simulator".into()
    }
}
