# Plan: Pi 5 ↔ Arduino Mega link (SENNEP V2)

> Consolidated plan from the 2026-10-12 session (branch `claude/stoic-cray-tsomgg`). Written so a later session can merge it
> with the other workstreams (circuit/breadboard, LiDAR stack, PCB, documentation) without reading the chat.
> Detailed reasoning, trade-off tables and sources: `docs/electronics/PI_MEGA_LINK.md` (Danish). If the two disagree, this plan is newer.

## 1. Goal

The Pi does the high-level work: LiDAR, planning, UI, camera. The Mega does the real-time I/O: motors, safety stops, IMU, close-range
sensors and the RF joystick. Together they should feel instant. Low latency is Mikkel's top priority, and the robot must have room to grow
(V1 ran out of pins).

## 2. Decisions (agreed with Mikkel)

| # | Decision | Round |
|---|---|---|
| D1 | **One Arduino Mega as the real-time hub.** A Nano/Uno is used only for the WS2812B LED strip, because the strip disables interrupts for ~4.5 ms per update and would drop serial bytes on the Mega. | 2 |
| D2 | **Pi ↔ Mega over GPIO UART**: Mega `Serial1` (pins 18/19) ↔ Pi GPIO14/15 through the 4-channel BSS138 level converter. **250 000 baud** to start (exact on 16 MHz); try 500 000 once the edges have been measured. USB stays for debugging and as plan B. | 1–2 |
| D3 | **ESP-01 is dropped.** The Pi has WiFi, and its two converter channels are needed for the link + reset. | 2 |
| D4 | **Flash the Mega from the Pi** with a reset wire: Pi GPIO4 → converter channel 3 → Mega RESET, then avrdude over the same UART. | 2 |
| D5 | **Custom binary protocol**: COBS framing + CRC-16/CCITT-FALSE, little-endian, max payload 60 B. micro-ROS (doesn't fit 8 KB RAM / AVR), Firmata (puts safety on the Pi), rosserial (ROS1, EOL) and I²C/SPI (polling, slave problems) were all rejected. | 1 |
| D6 | **Safety rules approved as written** (§5). | 2 |
| D7 | **Reflex architecture**: anything that must react in < 5 ms (edge, too-close ToF, e-stop, link loss) is decided on the Mega without the Pi. | 1–2 |
| D8 | **Expansion plan** (§4): I²C first, then PCF8574 expanders, then Nano/Uno nodes on Mega `Serial2`/`Serial3` speaking the same protocol, then Pi ports. | 2 |
| D9 | **"Full Rust for the system."** The Pi side is Rust. The Python bench tool (`tools/sennep_link.py`) will be **replaced by a Rust CLI**. | 3 |
| D10 | Sensor ownership: "whatever is the best design, latency first" → the proposal in §3 stands. | 1 |

## 3. Who owns what

| Device | Owner | Why |
|---|---|---|
| 4× BTS7960 (8 PWM at 20 kHz, shared EN, 4× IS current sense) | Mega | Real time, failsafe |
| Mecanum kinematics + ramps | Mega | The RF joystick and the Pi use the same code; the Pi only sends vx/vy/ω |
| E-stop state, 4× E18 edge, 8× VL53L0X (TCA9548A), 4× HC-SR04 | Mega | Local stop without the Pi |
| MPU-6050 + QMC5883L | Mega | Raw data streamed at 100/50 Hz; heading hold possible locally |
| NRF24L01 RF joystick | Mega | Manual override must work with the Pi dead |
| INA219 (battery voltage) | Mega | Low-battery stop. ⚠ The module's 0.1 Ω shunt only measures ~3.2 A → motor current needs an external shunt |
| WS2812B | Nano/Uno node on Mega Serial3 | See D1 |
| GPS NEO-6M | Pi (own UART, 3.3 V) | 1 Hz, not time-critical |
| LiDAR LD19, camera, 7" screen, audio/DFPlayer, WiFi | Pi | Heavy processing / not real time |
| ESP-01 | dropped | D3 |

Mega UARTs: Serial0 = USB debug/flash, Serial1 = Pi, Serial2 = expansion node #2 (free), Serial3 = LED node.

## 4. Room to grow

1. **I²C bus first**: 2 pins, ~100 addresses; each TCA9548A adds 8 channels (5 owned = 40).
2. **PCF8574 I/O expanders**: the 6 LCD backpacks are 8 digital I/O each. Good for lights, relays and buttons; not for PWM or fast signals.
3. **Expansion nodes**: a Nano/Uno on Serial2/Serial3 speaks the same protocol (5 V ↔ 5 V, 3 wires). The Mega forwards its frames
   unchanged (`NODE_FRAME` / `NODE_SEND`), so a new node needs no Mega firmware change. Node payloads must be ≤ 55 bytes.
4. **Pi ports** for anything not real time.
5. **Pin reserve**: keep ≥ 15 free Mega pins (incl. 2 PWM, 4 analog) after the first build. The current draft uses ~45–50 of 70.

## 5. Safety rules (approved)

- Motors only run in `ARMED`, which needs an explicit `ARM` from the Pi. A Mega reboot can never make the robot drive by itself.
- No `CMD_DRIVE` / `CMD_WHEELS` / `HEARTBEAT` for 200 ms → `LINK_LOST`: ramp to 0 (~0.17 s from full), EN low. A new `ARM` is required.
- Edge sensor → movement towards that corner is blocked immediately (no ramp), and rotation is blocked.
  ToF < 120 mm → movement towards that side is blocked. 0 = no reading = no block.
- Hardware e-stop cuts the 40 A relay. The Mega reads it (fail-safe input: a broken wire = pressed) → `ESTOP`; released → `DISARMED`.
- Software `ESTOP_SW` latches until `DISARM`. Battery < 10.2 V for 2 s → `FAULT`.
- The AVR hardware watchdog (250 ms) resets a hung Mega (⚠ test the clone bootloader, §9).
- Pi side: no telemetry for 100 ms → report "Mega gone" and reconnect.

## 6. Protocol (summary)

- Wire: `COBS( type u8 | seq u8 | payload ≤60 | crc16 LE ) 0x00`. A lost byte costs at most one frame; the receiver re-syncs on the next 0x00.
- **Single source of truth: `protocol/messages.json`.** `tools/gen_msgs.py` generates the C header, the Python table and `protocol/MESSAGES.md`.
  `--check` fails if the generated files are stale. `HELLO` carries `proto_version` + `spec_hash` so the Pi detects a mismatched Mega.
- Pi → Mega: HEARTBEAT, ARM, DISARM, ESTOP_SW, CMD_DRIVE (20–50 Hz), CMD_WHEELS, SET/GET_PARAM, SET_OUTPUT, TIME_SYNC, NODE_SEND.
- Mega → Pi: HELLO, STATUS 10 Hz, IMU 100 Hz, MAG 50 Hz, RANGES 25 Hz, MOTORS 50 Hz, RC, TIME_SYNC_REPLY, ACK, LOG, NODE_FRAME.
- Load ≈ 5.7 kB/s ≈ 23 % of 250 kbaud. Expected Pi command → PWM ≈ 2–4 ms, worst case ~10 ms.
- Time sync: NTP-style `TIME_SYNC` at 1 Hz; keep the 8 shortest round trips → offset + drift (~0.5 ms).
- Runtime params (RAM only for now): LINK_TIMEOUT_MS 200, RAMP 2500 ‰/s, STOP_RAMP 6000 ‰/s, MAX_WHEEL_MM_S 800, ROT_ARM_MM 450,
  TOF_STOP_MM 120, EDGE_ENABLE 1, BATTERY_LOW_MV 10200, MOTOR_INVERT_MASK 0.

## 7. What exists now (all on branch `claude/stoic-cray-tsomgg`)

| Path | What | Status |
|---|---|---|
| `docs/electronics/PI_MEGA_LINK.md` | Design, options, trade-offs, rounds 1–3 | Done |
| `protocol/messages.json`, `protocol/MESSAGES.md` | Message spec + generated table | Done (v1) |
| `tools/gen_msgs.py` | Generator (C + Python + MD). **Needs a Rust output** | Works |
| `firmware/libraries/SennepProto/` | Portable C99: COBS, CRC, stream decoder (`sennep_proto`), safety/kinematics state machine (`sennep_logic`), generated `sennep_msgs.h` | Done, tested |
| `firmware/sennep_mega/` | Arduino sketch: links + node relay, 20 kHz PWM on timers 3/4/5, MPU-6050/QMC5883L/INA219, edge + e-stop inputs, telemetry scheduler, AVR watchdog, boot reason/count. **Pin map in `config.h` is a DRAFT** | Compiles: 17 262 B flash (6.6 %), 1 576 B RAM (19 %) |
| `tools/build_mega.sh` | arduino-cli build, or `--avr-gcc` fallback (Ubuntu `gcc-avr avr-libc arduino-core-avr`) | Works |
| `tests/test_core.c` | 12 071 checks (CRC vectors, COBS fuzz, dropped/flipped/inserted bytes, lost delimiter, overflow, every safety rule, kinematics, ramps), ASan + UBSan | **All green** |
| `tests/proto_tool.c` | C encode/decode helper for cross-language tests | Written, not used yet |
| `tools/sennep_link.py`, `tools/sennep_msgs.py` | Python reference + bench CLI | Works; **to be replaced by Rust (D9)** |

Not done in the Mega firmware: VL53L0X/TCA9548A driver, HC-SR04 (pin-change interrupts), NRF24 RC, saving params to EEPROM, `firmware/README.md`.
**Nothing has run on real hardware.**

## 8. Open questions (ask Mikkel)

| # | Question | Recommendation |
|---|---|---|
| Q1 | **Should the Mega run Rust too?** Explained to Mikkel: every Arduino sketch already *is* custom firmware; the language only changes the tools (Arduino IDE vs Rust's `cargo` + avrdude) and the libraries. AVR is Rust's weakest target (nightly compiler, fewer drivers). | **Rust on the Pi, Arduino C++ on the Mega for now.** If Rust on the MCU is wanted later, use an RP2040 (Pico, ~40 kr, 3.3 V, embassy) rather than AVR. Waiting for his answer. |
| Q2 | Inventory: Mikkel says 1 Mega + several Uno + some Nano; `components/inventory.csv` says 2 Mega, 2 Nano, 0 Uno. | Fix in the documentation task. |
| Q3 | Which Pi 5 device is GPIO14/15 (`/dev/ttyAMA0` vs `ttyAMA10`), and is it `dtparam=uart0=on` or `dtoverlay=uart0-pi5`? Sources disagree. | Check on the Pi: `ls -l /dev/serial* /dev/ttyAMA*`. |
| Q4 | 250 k or 500 k baud through the BSS138? | Start at 250 k; measure with the `latency` test. |
| Q5 | Exact pins: owned by the circuit task. | Use `firmware/sennep_mega/config.h` as the starting point. |

## 9. Next steps, in order

**Phase A — Rust on the Pi (do regardless of Q1)**
1. Cargo workspace `rust/`:
   - `sennep-proto`: `#![no_std]`, no alloc. COBS, CRC, streaming decoder, typed messages. Runs on a Pi, a PC and (if Q1 = Rust) the Mega.
   - `sennep-link`: std. `serialport` crate, reader thread, latest value per message, keep-alive, time-sync estimator, reconnect, HELLO/spec-hash check.
   - `sennep-cli`: `monitor`, `latency`, `wheels`, `drive`, `watchdog`, `param`, `flash` (same commands as the Python tool).
2. Extend `tools/gen_msgs.py` with a Rust output (`sennep-proto/src/msgs.rs`) so the spec stays the single source.
3. Tests: port the framing tests from `tests/test_core.c`, plus a **cross-test against the C core** via `tests/proto_tool.c`
   (encode both ways, feed a corrupted stream to both, and both must accept/reject the same frames).
4. When the Rust CLI matches the Python tool: delete `tools/sennep_link.py` + the Python output of the generator.
5. Cross-compile for the Pi 5 (`aarch64-unknown-linux-gnu`) or build on the Pi.

**Phase B — Mega firmware (depends on Q1)**
- *Arduino C++ (recommended):* add the VL53L0X driver via TCA9548A (continuous mode, non-blocking), HC-SR04 with pin-change interrupts on A8–A11
  and staggered triggering, NRF24 RC → `RC` message + local manual drive, params in EEPROM, `firmware/README.md` (Arduino IDE: copy
  `firmware/libraries/SennepProto` into `~/Arduino/libraries`; CLI: `--libraries firmware/libraries`).
- *Rust (avr-hal):* port `sennep_logic.c` + the sketch to `arduino-hal` on nightly, reuse `sennep-proto`, keep `tests/test_core.c`
  as the reference for the safety tests. Expect extra work on timers/PWM and sensor drivers.

**Phase C — LED node**: a Nano/Uno sketch for the WS2812B on Mega Serial3, speaking the protocol (payload ≤ 55 B).

**Phase D — Pi setup**: enable UART0 on GPIO14/15 and disable the serial login console (`raspi-config` → Interface → Serial: login *no*,
hardware *yes*). Flash script: `pinctrl set 4 op dl; sleep 0.05; pinctrl set 4 ip` then `avrdude -c stk500v2 -p atmega2560
-P /dev/ttyAMA0 -b 115200 -D -U flash:w:sennep_mega.ino.hex:i` (the Mega bootloader runs at 115200 regardless of the link baud;
the timing must be tested). The link program must close the port first.

**Phase E — hardware tests (§10)**, then tune params (MAX_WHEEL_MM_S, ROT_ARM_MM, ramps, TOF_STOP_MM, MOTOR_INVERT_MASK).

## 10. Hardware test checklist (in order, wheels off the ground first)

1. **Flash over USB** from a PC (Arduino IDE / `tools/build_mega.sh`). The Serial Monitor at 115200 should show `SENNEP Mega proto v1 spec 0x… boot N`.
2. **Watchdog/bootloader**: a Mega that resets itself via the watchdog must come back. If it hangs after a watchdog reset, set `USE_AVR_WDT 0` and note the board.
3. **Level shifter + UART**: the Pi can see `HELLO` (`monitor`). Check that the Mega TX → Pi RX line never exceeds 3.3 V (multimeter/scope).
4. **Latency**: run `latency` 200× and record min/median/p95/max. Expect ~1–3 ms round trip. Repeat at 500 k.
5. **Error counters**: run `monitor` for 10 min with the motors running. `rx_crc`/`rx_frame`/`gaps` should be ~0, and loop_max under 5000 µs.
6. **Each wheel** with `wheels 200 0 0 0 2` etc.: must turn forward. If not, set the bit in MOTOR_INVERT_MASK (no rewiring).
7. **Kinematics**: forward, sideways and rotation each move the correct way (`drive`).
8. **Watchdog test**: `watchdog` must report the motors at 0 within ~0.4 s of the Pi going silent. **If not: do not drive the robot.**
   Also unplug the TX wire while driving.
9. **E-stop**: press it while driving → relay off, STATUS = ESTOP. Release → DISARMED, no motion until ARM. Also pull the sense wire → ESTOP.
10. **Edge sensors**: lift one corner → motion towards it stops immediately; driving away still works.
11. **Reset wire**: flash from the Pi (Phase D) without touching the robot.
12. **Noise**: run the motors at full load near the link wiring; the counters from step 5 stay ~0.

## 11. What the circuit task must provide (`docs/electronics/`)

- BSS138 converter: ch1 Mega TX1 → Pi RX (GPIO15), ch2 Pi TX (GPIO14) → Mega RX1, ch3 Pi GPIO4 → Mega RESET. HV = 5 V, LV = 3.3 V, common GND.
  Short twisted TX/RX/GND, routed away from the motor wires.
- E-stop: a separate NC contact (or relay contact) to Mega pin 23 → GND, so a pressed button **or a broken wire** reads HIGH.
- Each BTS7960: R_EN + L_EN tied to Mega pin 22. R_IS + L_IS joined per driver to A0–A3 (check the IS resistor/voltage range against the datasheet).
- Edge sensors E18-D80NK on pins 24–27 (NPN output, pull-up).
- An external shunt (or another sensor) if motor current should be measured; the INA219 module itself is good to ~3.2 A.
- IRFZ44N MOSFETs for the lights are not logic level → they need a gate driver or a logic-level MOSFET.
- Power for the Mega that doesn't sag when the motors start (separate buck), plus a common ground point.

## 12. Risks and notes

- The Mega serial RX buffer is 64 B (≈ 2.6 ms at 250 k). `loop()` must never block (no `delay`, `pulseIn`, blocking ToF reads).
  STATUS reports `loop_max_us` and `RX_OVERFLOW`.
- The old Mega 2560 bootloader on clones can hang after a watchdog reset (test 2).
- Without encoders (a deliberate choice) duty ≠ speed. Odometry has to come from LiDAR scan matching + IMU, and MAX_WHEEL_MM_S is an estimate.
- The motor direction/kinematics sign convention (mecanum X layout) must be checked at the robot (tests 6–7).
- Cloud sessions cannot reach `downloads.arduino.cc`; arduino-cli builds happen on the Pi/PC. The Rust toolchain and crates.io index *are* reachable.

## 13. Conversation record (for the consolidating session)

Mikkel has **not read** the outputs of this session. Everything he said and everything he was told is recorded here.

**Original task:** work out how the Pi 5 and the Mega work together "instantly and seamlessly". Strict order: take stock →
research → write `docs/electronics/PI_MEGA_LINK.md` → AskUserQuestion rounds → code only after approval. This task owns the
protocol and software; the circuit task owns the physical wiring.

**Round 1: Mikkel's answers (close to verbatim)**
- Mega or Nano: "I'm not sure if mega replaces nano. It kinda depends on the design. I have multiple unos, one mega, a few nanos, and one
  raspberry pi." Link: "If we have the material I want to do UART now. Otherwise USB for now and UART later."
- Pi language: "depends on what works best … LiDAR code is still being debated. Maybe defer."
- "Instant": "I don't want the robot to be slow reacting. There are a LOT of inputs … we need to be able to process it all … low latency is
  a priority. Which is also why I am leaning towards a fast language like Rust or C … depends on the stack and design."
- Sensor placement: "Whatever is best design … as long as latency is priority."

**What he was told in reply:** latency comes from sensor rates (ToF 20–33 ms, sonar 60–100 ms per round, LiDAR 100 ms) and from *where
decisions are made*, not from the link (~1 ms) or the language. Hence the reflex architecture (D7). Everything real-time fits on one Mega.

**Round 2:** Mega hub + Nano chosen, *"but on SENNEP V1 we ran out of pins … plan for possible future expansion"* (→ §4).
Reset wire chosen. "Build firmware + C core + tests now" chosen. Safety rules approved as written.

**Mid-session:** "FYI — we decided to go with full rust for the system." (Sent twice; it is the same message.)

**Round 3:** replace the Python tool with a Rust CLI. On "Rust on the Mega?" he answered: *"I'm not sure. I don't really understand … I have
only ever worked with Arduino, where I just got it out of the package, flashed some code from the Arduino IDE … I have never made
custom firmware … I guess it is just a processing unit, so it could work. Please explain."*

**The explanation he was given (summary):**
- Every Arduino sketch already is custom firmware: the IDE compiles C++ to a .hex and avrdude flashes it. The chip only runs machine code.
- The language only changes the tools and libraries. Option 1, Arduino C++ on the Mega: he keeps the IDE + Upload, has proven libraries,
  and the code is already written and tested. The system is then ~95 % Rust (the Pi) and ~1000 lines of Arduino (the Mega), kept in sync
  by one spec + cross-tests. Option 2, Rust on the Mega: same chip and flashing, but no IDE, a nightly compiler, fewer drivers and more
  hand-written work. That is a lot of new things for a first non-IDE firmware.
- **Recommendation given: Rust on the Pi, Arduino C++ on the Mega for now; RP2040/Pico if Rust on an MCU is wanted later.**
  (In the question before that, "Rust on Mega" had been marked "recommended"; this was corrected in the explanation once it was clear he only knows the IDE.)
- He was asked to reply "Arduino on Mega" or "Rust on Mega". **He has not answered (Q1 is still open).**

**Last instruction:** write everything down for a separate consolidating session. Six cloud sessions ran independently (LiDAR stack,
this link, breadboard circuit, PCB, documentation/component list, 3S6P battery). The PCB session logged that it was blocked
because the breadboard plan was missing (branch `claude/youthful-planck-dsjzos`). When this session ran, `docs/electronics/` existed
only on this branch: no circuit work had been pushed yet.

**Branch:** everything is on `claude/stoic-cray-tsomgg` (not merged into `main`, no PR). The files are listed in §7.
