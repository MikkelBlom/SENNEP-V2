# Pi 5 ↔ Arduino Mega — link and protocol

Status: **design APPROVED 2026-10-12** (round 2, §0b). Open question **Q1 (Arduino C++ vs Rust on the Mega)** is still waiting for Mikkel (§0c). Combined plan + next steps: `ai-instructions/PLAN_PI_MEGA_LINK.md`. Code: `firmware/`, `protocol/`, `tools/`, `tests/`.

## 0. Mikkel's answers, round 1 (2026-10-12)

- **Mega or Nano?** Unresolved — "depends on the design". He has (by his own account) **several Unos, one Mega, some Nanos and one Pi**.
  ⚠ `inventory.csv` says 2× Mega, 2× Nano, 0× Uno → the parts list must be corrected (the documentation task).
- **Link:** **UART now, if the materials exist**, otherwise USB now and UART later. → It can be done, see §2b.
- **Language on the Pi:** postponed. Leans towards a fast language (Rust/C), but it depends on the rest of the stack (the LiDAR code is not settled).
  *Superseded: the Pi side is Rust (decided 2026-10-12, §0c).*
- **"Instant":** the robot must not react slowly, even with a lot of inputs. Low latency has high priority. How the processing is split (several Nanos + Pi, or everything central) is open → §4b.
- **Sensor split:** "whatever is the best design", as long as latency is prioritised → the proposal in §5 stands.
Ownership: this document owns protocol and software. The physical wiring (connectors, level shifting, ground, E-stop circuit)
is owned by the circuit task in `docs/electronics/`. Where it says **[circuit]**, it must be aligned there.

## 0b. Mikkel's answers, round 2 (2026-10-12) — APPROVED

- **Architecture:** one Mega as real-time hub + a Nano/Uno for the LED strip. **Requirement: room for expansion** — V1 ran out of pins. → §4d.
- **Link:** GPIO-UART (Mega Serial1 ↔ Pi GPIO14/15 via BSS138). **ESP-01 is dropped.** Flash from the Pi via a **reset wire** (3rd channel on the converter).
- **Code now:** Mega firmware skeleton + language-neutral C99 protocol core + host tests with Python as the test counterpart. The Pi application's language is chosen later.
  *Superseded: the Pi side is Rust (decided 2026-10-12, §0c).*
- **The safety rules in §4 are approved as written** (explicit ARM, 200 ms watchdog → ramp to stop + new ARM, local blocking at edge/obstacle, hardware E-stop on the relay).

## 0c. Round 3 (2026-10-12): Rust

- **Mikkel: "full Rust for the system".** The Pi side becomes Rust. The Python tool (`tools/sennep_link.py`) is replaced by a Rust CLI.
- **Open (Q1):** should the Mega also run Rust (avr-hal), or keep Arduino C++? Mikkel has only used the Arduino IDE and asked for an explanation
  (given in the chat 2026-10-12). Recommendation: Rust on the Pi, Arduino C++ on the Mega for now; if you want Rust on a microcontroller,
  an RP2040 (Pico) is a better target than AVR. Protocol and safety rules are the same whatever the choice.
- Status now: the C core + the Mega sketch compile (17 KB flash, 1.6 KB RAM), and 12,071 host tests are green. Nothing is tested on hardware.

## 1. Starting point

### What exists (from `components/INVENTORY.md`)

| Part | Qty | Relevant for the link |
|---|---|---|
| Raspberry Pi 5 | 1 | 3.3 V GPIO, 4× USB-A, UART on GPIO14/15 + more via overlays |
| Arduino Mega 2560 R3 (clone) | 2 | 5 V logic, 8 KB RAM, 256 KB flash, **4 hardware UARTs**, 15 PWM, 16 ADC. USB chip unknown (16U2 or CH340) |
| Arduino Nano (CH340) | 2 | Spare. Was the original motor MCU |
| 4-channel level converter (BSS138) | **1** | Shared with ESP-01 (uses 2 channels) |
| BTS7960 drivers | 5 | 4 in use: each has RPWM, LPWM, R_EN, L_EN, R_IS, L_IS |
| MPU-6050, QMC5883L, NEO-6M, INA219, 8× VL53L0X + TCA9548A, 4× E18-D80NK, 4× HC-SR04, NRF24L01, ESP-01, WS2812B, E-stop + 40 A relay | | Sensors/IO to be distributed |

### What is already decided (PROJECT_OVERVIEW §3, §7b, §8)

- Pi = high level in **Python, no ROS2**. MCU = motors + secondary collision avoidance + RF override.
  *Superseded: the Pi side is Rust (decided 2026-10-12, §0c). No ROS2 still stands.*
- Pi ↔ MCU "via UART" — but the MCU was the **Nano**, the Mega was plan B. This task assumes the **Mega** (4 UARTs, 4× RAM).
  → Mikkel must confirm this (question 1).
- **No motor encoders** (deliberate choice, Apr 2025). The motors (RS 834-7641) have no encoder.
  Consequence: the Mega can only control PWM/duty, not speed. Odometry must come from LiDAR scan matching + IMU.
- The E-stop controls the 40 A relay (hardware, independent of software).
- GPS, IMU and magnetometer are to be included. Not waterproof.

### Voltages

Pi GPIO is **3.3 V and does not tolerate 5 V**. The Mega is 5 V logic and reads high from about 3.0 V (0.6·Vcc), so 3.3 V into
the Mega typically works, but with a small margin. Everything from Mega → Pi GPIO **must** be level shifted. Over USB the problem goes away.

## 2. Physical link — three options

```
 A) USB cable                       B) GPIO-UART                      C) I²C / SPI
 Pi USB-A ──USB──► Mega USB-B       Pi GPIO14 TX ─►LV│HV─► Mega RX2    Pi = master, Mega = slave
   (CDC serial via 16U2/CH340)      Pi GPIO15 RX ◄─LV│HV◄─ Mega TX2    needs level shifting + polling
   5 V + GND come with the cable    + GND, + GPIO → Mega RESET (flash)
```

| | **A: USB** | **B: GPIO-UART + level shifting** | **C: I²C/SPI** |
|---|---|---|---|
| Latency (30-byte frame) | ~1–3 ms (USB frame 1 ms + chip buffer) | ~0.6 ms at 500 kbaud | Pi must poll; I²C 400 kHz ≈ 1 ms + polling jitter |
| Bandwidth | plenty (500 kbaud–1 Mbaud) | plenty (250k/500k/1M are error-free at 16 MHz) | I²C low, SPI high |
| Level shifting | **none** | yes, 2 channels (shares the one converter with ESP-01) **[circuit]** | yes, 2–4 lines |
| Flash the Mega from the Pi | **free**: `arduino-cli upload` / avrdude, DTR reset is built in | needs extra GPIO → 100 nF → RESET, avrdude with `-c arduino` + reset script | no (USB/ISP separately) |
| Supply | The Mega can get 5 V from the Pi (careful: Pi 5 USB budget) — or its own 5 V **[circuit]** | Mega on its own 5 V | — |
| Robustness | USB can disconnect from motor noise/ESD → **auto-reconnect needed**. Connector can shake loose | No enumeration, cannot "disappear". Dupont wires can shake loose | The Mega as I²C slave is known to hang the bus |
| Auto-reset | The Mega **restarts every time the port is opened** (DTR). Handled in the protocol (HELLO) | none | none |
| Debugging | Can plug a laptop into the Mega instead of the Pi | needs a USB-TTL adapter | hard |
| The Mega's UARTs | Serial0 goes to the Pi → Serial1–3 free | Serial0 free for debug/flash → one of Serial1–3 used | all free |

**Conclusion:** C is rejected (polling, no benefit, I²C slave problems). Between A and B, **A (USB)** wins for a hobby robot:
zero extra hardware, flash from the Pi without tricks, and you can test the Mega with a laptop. The latency difference (~1–2 ms)
means nothing for a robot with 0.6 m/s top speed (1 ms = 0.6 mm). B is a good plan B if USB turns out unstable
with motor noise — the protocol is the same, so the switch is a cable and configuration change.

## 2b. UART now — do we have the materials? Yes.

| Need | Have | Note |
|---|---|---|
| Level shift Mega TX (5 V) → Pi RX (3.3 V) | 4-channel BSS138 converter | **Must** be level shifted (the Pi does not tolerate 5 V) |
| Pi TX (3.3 V) → Mega RX | same converter | Would work directly (the Mega reads high from ~3.0 V), but the converter gives margin |
| Pi GPIO → Mega RESET (flash from the Pi) | 3rd channel on the converter | Optional. Otherwise the Mega is flashed with the USB cable when needed |
| Wires, common GND | 22 AWG | **[circuit]**: short, twisted TX/RX/GND, away from the motor wires |

Conflict: the ESP-01 also uses 2 channels. With Pi link (2) + reset (1) only 1 is left → **ESP-01 is dropped** (the Pi has WiFi),
or it gets its own resistor divider when there are resistors (not on the parts list).

Baud rate over BSS138: the converter has 10 kΩ pull-ups, so the edges are slow (~0.5 µs). **Start at 250000** (4 µs per bit);
500000 (2 µs per bit) probably works too, but must be measured. 250000 is exact at 16 MHz and gives ~25 kB/s, i.e. 5× more than the message list uses.

Pi 5 side: GPIO14/15 = UART0. The sources disagree on the name (`/dev/ttyAMA0` vs `ttyAMA10`; `ttyAMA10` is the debug connector),
and on whether it needs `dtparam=uart0=on` or `dtoverlay=uart0-pi5` in `config.txt`. Check on the Pi with `ls -l /dev/serial* /dev/ttyAMA*`.
The serial console (login shell) must be turned off on the port (`raspi-config` → Interface → Serial: login no, hardware yes).

Gain of UART over USB: no auto-reset on port open, no USB enumeration that can disappear with motor noise, ~1–2 ms less latency, and
the Mega's USB (Serial0) is free for debug from a laptop. Price: 3 wires + the converter, and flashing needs either the reset wire or the USB cable.

## 3. Protocol — options

| | Assessment |
|---|---|
| **micro-ROS** | **Does not fit.** Needs "tens of kilobytes" of RAM and a 32-bit MCU; no 8-bit/AVR support. And you have rejected ROS. |
| **rosserial** | ROS1 (EOL 2025). Rejected. |
| **Firmata** | The Pi controls the pins directly → **the real-time logic and failsafe end up on the Pi**. Goes against the whole idea. Rejected. |
| **MAVLink v2 (own dialect)** | Proven to run on Mega 2560 (old ArduPilot APM). Header-only C, `pymavlink` on the Pi. Built-in framing, CRC, sequence no. Downside: XML dialect + code generation, ~280 bytes buffer per channel, a lot of machinery for ~15 messages. |
| **Own binary protocol: COBS + CRC-16** | Small (~200 bytes RAM, ~1 KB flash), fully understood, easy to test on host. Needs both sides kept in sync → one shared message table that generates a C header + Python module. **Recommended.** |
| Text/ASCII (`M 100 -50\n`) | Easy to read, but slow parsing on AVR, no CRC, unclear boundaries. Debug only. |

### Recommended framing

```
 raw frame (before COBS):  [ type u8 | seq u8 | payload 0–48 B | crc16 u16 LE ]
 on the wire:              COBS(raw frame) 0x00
```

- **COBS** removes all 0x00 from the data → 0x00 always means "end of frame". If a byte is lost, at most one frame is lost;
  the receiver resyncs at the next 0x00. Overhead: 1 byte per 254.
- **CRC-16/CCITT-FALSE** (poly 0x1021, init 0xFFFF) over type+seq+payload. Catches all 1–2-bit errors and all bursts ≤ 16 bits.
- **seq** counts per direction (0–255) → the receiver counts lost frames.
- All numbers are **little-endian, fixed integers** (no float on the wire except parameters): mm/s, mrad/s, mV, mA, µs.
- Max payload 60 bytes (room for a wrapped node frame) → the Mega has fixed buffers, no `malloc`, no `String`.

### Baud rate

USB-CDC ignores the baud rate between the Pi and the USB chip, but the chip forwards to the ATmega at the chosen rate.
16 MHz divides 115200 unevenly (≈ +2.1 % with U2X); **250000 / 500000 / 1000000 are exact**. Proposal: **500000**.
Load with the message list below: ~5 kB/s of 50 kB/s = ~10 %.

## 4. Real time, failsafe and reconnection

### What "instant" can mean (proposal — question for Mikkel)

| Chain | Budget |
|---|---|
| Pi sends command → PWM changed | typically 2–4 ms, worst case ~10 ms (Linux jitter) |
| Sensor event on the Mega → the Pi has it | IMU 10 ms (100 Hz), distances 40 ms (25 Hz) |
| Edge detector/ToF "too close" → motor stopped | **< 5 ms, without the Pi** (the Mega does it itself) |
| Pi dies → motors stopped | **200 ms** (command timeout) + ramp |
| E-stop pressed → motor power gone | immediately, pure hardware (relay) |

### 4b. Where does the latency actually come from? (answer to "many inputs, low latency")

What makes a robot slow is rarely the link or the programming language — it is **the sensors' own measurement rates**
and **where the decision is made**. The amount of data is small: all sensors together give ~5 kB/s, and a 16 MHz Mega
handles that with a good margin, as long as nothing in `loop()` blocks.

| Link in the chain | Typical time |
|---|---|
| VL53L0X ToF, one measurement | ~20–33 ms (all 8 measure in parallel in continuous mode) |
| HC-SR04 ultrasound | ~6–25 ms per measurement; the 4 must take turns (otherwise they hear each other's echo) → ~60–100 ms per round |
| LiDAR LD19 | 10 Hz → 100 ms per revolution |
| Camera | 33 ms (30 fps) + inference |
| E18 edge sensor | < 2 ms (digital) |
| MPU-6050 | 1 ms (can be read at 1 kHz) |
| **The Pi ↔ Mega link (UART, 30 B)** | **~1 ms** |
| Linux jitter on the Pi (Python *or* Rust) | typically < 1 ms, rarely 5–10 ms |
| Motor + gearbox + the robot's mass | 50–200 ms (physics) |

Conclusion: Rust/C on the Pi saves maybe 1 ms in the link. It **matters** for heavy computation (LiDAR scan matching, camera)
— but that is a decision for the LiDAR stack, not for the link. The big gain comes from a **reflex architecture**:

```
 Reflex   (< 5 ms, Mega alone):    edge sensor / ToF too close / E-stop / link lost  → stop or block direction
 Control  (10–20 ms, Mega):        kinematics, ramp, heading hold with IMU
 Tactics  (50–100 ms, Pi):         avoid obstacles from LiDAR + ToF, follow path
 Strategy (> 100 ms, Pi):          map, planning, UI, camera
```

Each layer works even if the layer above is slow or dead. That is why the robot feels fast.

### 4c. One central Mega or several Nanos?

Pin count if **all real time is on one Mega** (the Mega has 54 digital + 16 analog, 15 PWM, 4 UART, 6 external interrupts + PCINT):

| Function | Pins |
|---|---|
| 4× BTS7960: RPWM+LPWM | 8 PWM |
| BTS7960 enable (shared or per driver) | 1–4 |
| BTS7960 current sense IS (R_IS+L_IS combined per driver) | 4 analog |
| I²C bus: MPU-6050, QMC5883L, INA219, TCA9548A → 8× VL53L0X | 2 |
| 4× E18 edge | 4 |
| 4× HC-SR04 (shared trigger, 4 echo on PCINT) | 5 |
| NRF24L01 (SPI + CE + CSN + IRQ) | 6 |
| E-stop state | 1 |
| UART to the Pi (+ reset) | 2 (+1 on the Pi side) |
| GPS / DFPlayer (if on the Mega) | 2 + 2 |
| Dock IR receivers (e.g. 4) | 4 |
| Lights (head-lights/tail-lights via MOSFET) | 2–4 PWM |
| **Total** | **~45–50 of 70** |

That fits on one Mega with room for more. A Nano (20 I/O) cannot do it alone.

| | **A: One Mega (+ maybe 1 Nano for LED)** | **B: Mega + several Nano "sensor nodes"** | **C: Nanos directly to the Pi** |
|---|---|---|---|
| Latency sensor → motor | 1 hop, < 5 ms | 2 hops (Nano → Mega): +1–3 ms + poll | sensor → Pi → motor Nano: 2 hops + Linux |
| Reflex without the Pi | yes | yes | **no** (the decision is on the Pi) |
| Complexity | 1 firmware, 1 link | N firmwares, internal bus, more failure modes | N links on the Pi |
| When it makes sense | standard | if the Mega runs out of CPU time or pins | never for safety-critical things |

**Recommendation: A.** One Mega as real-time hub. A Nano/Uno only for tasks that ruin the timing, mainly the WS2812B strip
(turns interrupts off). Uno = same chip as Nano (ATmega328P), just a bigger board. If the Mega's `loop()` time later gets too high
(the STATUS message measures max loop time), sensors can be moved out to a Nano — the protocol and the Pi won't notice the difference.

### 4d. Room for expansion (so V1's pin shortage does not repeat)

Five expansion routes, in the order they should be used:

1. **The I²C bus is the main route for sensors.** 2 pins, up to ~100 addresses. Five TCA9548A give up to 40 extra channels for sensors with
   the same address. Rule: new slow sensors (temperature, light, extra ToF) go on I²C, not on their own pins.
2. **I²C port expanders: you already have them.** The 6 LCD adapters (`i2c-lcd-adapter`) are PCF8574 chips = 8 digital I/O each.
   They are good for lights, relays, buttons and slow inputs, but not for PWM or fast signals. They can be desoldered from the LCD, or
   you can use a spare one.
3. **Expansion ports: the Mega's Serial2 and Serial3** speak the *same* COBS protocol to a Nano/Uno "node" (5 V ↔ 5 V, 3 wires, no
   level shifting). The Mega forwards the node's frames unchanged to the Pi in a `NODE_FRAME` message, and frames from the Pi go the opposite way
   (`NODE_SEND`). So the Mega does not need new firmware to get a new node. The LED Nano is node #3 (Serial3), and Serial2 is free for node #2.
   The Unos and Nanos thereby become building blocks.
4. **The Pi's own ports** for what is not real time: GPS (own UART), LiDAR (USB/UART), sound, camera, USB devices.
5. **Pin reserve on the Mega:** the goal is at least 15 free pins incl. 2 PWM and 4 analog after the first build. `pins.h` keeps the accounts.

With this, pin shortage becomes a question of "which route?", not "new design".

### Failsafe layers

1. **Hardware E-stop** breaks the relay. The Mega reads the E-stop state on an input pin and reports it **[circuit]**.
2. **Mega state machine:** `BOOT → DISARMED → ARMED → (LINK_LOST | ESTOP | FAULT)`. Motors only run in `ARMED`.
   The Pi must send `ARM` explicitly after every (re)connection — a Mega restart can never make the robot drive on its own.
3. **Command watchdog:** no valid `CMD_DRIVE` or `HEARTBEAT` for 200 ms → `LINK_LOST`, motors are ramped to 0
   in ~150 ms, BTS7960 EN pins are pulled low. When the link comes back, a new `ARM` is required.
4. **Local safety on the Mega:** edge detectors and ToF below a limit block movement in that direction,
   whatever the Pi says (the Pi gets a flag bit about it).
5. **AVR hardware watchdog** (`wdt_enable(WDTO_250MS)`) restarts the Mega if the firmware hangs.
   ⚠ Older Mega 2560 bootloaders (found on clones) hang in a reset loop after a WDT reset. Must be tested on your board.
6. **Pi side:** no telemetry for 100 ms → the library reports "Mega gone", closes the port and retries every 0.5 s.

### Reconnection and auto-reset

- A **udev rule** gives the Mega a fixed name: `/dev/sennep-mega` (match on USB VID/PID + serial number).
- When the port is opened, the Mega restarts (DTR). The Pi waits for `HELLO` (≤ 2.5 s), checks the protocol version and then sends `ARM`.
- `HELLO` contains the boot reason (power-on / watchdog / brown-out / external reset) and a boot counter → unexpected restarts show up in the log.

### Time synchronisation

Every telemetry frame has the Mega's `micros()` (u32, wraps after 71 min — the Pi unwraps it).
The Pi sends `TIME_SYNC(t_pi)` every second; the Mega replies with `t_pi` + its own time. The Pi uses the 8 latest replies
with the shortest round trip to estimate offset + drift (the NTP principle). Expected precision: ~0.5 ms — plenty to
fuse the IMU with LiDAR scans (10 Hz).

### Pitfalls on the Mega that the design accounts for

- The Serial RX buffer is 64 bytes = 1.3 ms at 500 kbaud. `loop()` must never block longer than that:
  no `delay()`, no `pulseIn()` (HC-SR04 is done with a pin-change interrupt), VL53L0X in continuous mode and non-blocking reads.
  The buffer can be raised to 256 with `-DSERIAL_RX_BUFFER_SIZE=256` (costs 192 bytes RAM per UART).
- **WS2812B turns interrupts off** during an update: 150 LEDs ≈ 4.5 ms → ~225 lost bytes at 500 kbaud.
  Solution: the LED strip on the Nano, on the Pi (SPI), or only update in a gap after a received frame. Question for Mikkel.
- Fixed memory: no `String`, no `malloc`; `F()` strings in flash.

## 5. Who owns which sensor (proposal)

Principle: **everything safety-critical and everything with hard timing → Mega. Everything with heavy data processing, or that is not time-critical → Pi.**

| Device | Proposal | Why |
|---|---|---|
| 4× BTS7960 (8 PWM + EN + 8 IS current readings) | **Mega** | Real time, failsafe |
| Mecanum kinematics (vx, vy, ω → 4 wheels) + ramp | **Mega** | RF override and the Pi use the same code; the Pi only sends vx/vy/ω |
| E-stop state (input) | **Mega** | Reports to the Pi |
| 4× E18-D80NK edge + 8× VL53L0X (via TCA9548A) + 4× HC-SR04 | **Mega** | Local collision/fall stop without the Pi |
| MPU-6050 + QMC5883L | **Mega** (raw data 100 Hz to the Pi) | Heading hold can run locally; the Pi does sensor fusion |
| NRF24L01 RF joystick | **Mega** | Manual override must work even when the Pi is dead |
| INA219 battery | **Mega** | Low-voltage stop locally |
| GY-NEO6MV2 GPS | **Pi** (own UART, 3.3 V directly) | 1 Hz, not time-critical, `gpsd`/NMEA parsing is easier on the Pi. Alternatively Mega Serial2 + forward |
| LiDAR LD19 | **Pi** (USB adapter or UART) | 230400 baud, heavy data processing |
| Camera, 7" screen, sound, WiFi | **Pi** | — |
| ESP-01 | **Dropped** (round 2) | The Pi has WiFi; the converter channels are used for the Pi link + reset |
| WS2812B | **Nano/Uno node on Mega Serial3** (round 2) | Turns interrupts off; must not be on the Mega |
| DFPlayer | Pi (the Pi can do sound itself) or Mega Serial3 | — |

The Mega's UARTs (approved): Serial0 = USB (flash/debug), Serial1 = Pi, Serial2 = expansion node #2 (free), Serial3 = LED node (Nano/Uno). GPS and DFPlayer/sound on the Pi.

## 6. Message list — draft v0

> **The source is now `protocol/messages.json`** (generates `protocol/MESSAGES.md`, the C header and the Python module). The table here is the original draft; if they disagree, `MESSAGES.md` applies.

Pi → Mega (type 0x01–0x3F):

| Type | Name | Payload | Rate |
|---|---|---|---|
| 0x01 | `HEARTBEAT` | — | 10 Hz when not driving |
| 0x02 | `ARM` / 0x03 `DISARM` | — | as needed |
| 0x04 | `ESTOP_SW` | — | software E-stop (from the UI) |
| 0x10 | `CMD_DRIVE` | vx i16 mm/s, vy i16 mm/s, ω i16 mrad/s, flags u8 | 20–50 Hz (also works as heartbeat) |
| 0x11 | `CMD_WHEELS` | 4× i16 duty ‰ (−1000…1000) | test/calibration |
| 0x20 | `SET_PARAM` | id u8, value f32 | as needed → reply `ACK` |
| 0x21 | `GET_PARAM` | id u8 | as needed |
| 0x22 | `SET_OUTPUT` | output id u8, value u16 (lights, buzzer, LED pattern) | as needed |
| 0x30 | `TIME_SYNC` | t_pi u32 µs | 1 Hz |

Mega → Pi (type 0x80–0xBF):

| Type | Name | Payload | Rate |
|---|---|---|---|
| 0x80 | `HELLO` | protocol version u8, firmware hash u32, boot reason u8, boot counter u16 | at startup, repeated 2 Hz until the first ARM/HEARTBEAT |
| 0x81 | `STATUS` | t u32, state u8, fault bits u16, battery mV u16, current mA i16, max loop µs u16, rx errors u16 | 10 Hz |
| 0x82 | `IMU` | t u32, acc 3×i16, gyro 3×i16 (raw) | 100 Hz |
| 0x83 | `MAG` | t u32, 3×i16 | 50 Hz |
| 0x84 | `RANGES` | t u32, 8× ToF u16 mm, 4× sonar u16 mm, edge bits u8 | 25 Hz |
| 0x85 | `MOTORS` | t u32, 4× duty i16, 4× current u16 (from IS) | 50 Hz |
| 0x86 | `RC` | t u32, 2× axis i16, buttons u8, link-ok u8 | 50 Hz when active |
| 0x87 | `TIME_SYNC_REPLY` | t_pi u32, t_mega u32 | reply |
| 0x88 | `ACK` | reply-to type u8, seq u8, result u8 (+ value f32 for GET_PARAM) | reply |
| 0x89 | `LOG` | level u8, text ≤ 40 B | rarely |

The message table is written once (e.g. `protocol/messages.yaml`) and generates `sennep_msgs.h` (Mega) + `sennep_msgs.py` (Pi),
so the two sides can never get out of sync. The protocol version in `HELLO` is checked on connection.

## 7. Software sketch

*Superseded: the Pi side is Rust (decided 2026-10-12, §0c), not Python 3 as in the sketch below. The structure is the same.*

```
Pi 5 (Python 3)                                        Mega (Arduino C++)
┌───────────────────────────────┐                      ┌──────────────────────────────────┐
│ planner / UI / LiDAR          │                      │ loop() — never blocking          │
│        │  drive(vx,vy,w)      │                      │  ├ link.poll()  (COBS/CRC → msg) │
│ sennep_link (library)         │                      │  ├ safety.update() watchdog/edge │
│  ├ reader thread: COBS→CRC→msg│ USB CDC 500 kbaud    │  ├ drive.update() kinematics+ramp│
│  ├ latest value per message   │◄────────────────────►│  ├ sensors.update() (timed)      │
│  ├ callbacks / asyncio        │                      │  ├ rc.update()                   │
│  ├ heartbeat + time-sync      │                      │  └ telemetry.update() (rates)    │
│  └ reconnect + udev name      │                      │ wdt_reset()                      │
└───────────────────────────────┘                      └──────────────────────────────────┘
```

Flash from the Pi: `arduino-cli compile --fqbn arduino:avr:mega firmware/sennep_mega && arduino-cli upload -p /dev/sennep-mega …`
(the library closes the port first). The firmware hash in `HELLO` shows whether the Pi and the Mega run matching versions.

Testing without hardware (cloud): encode/decode round trip, CRC against known test vectors, lost/inserted/changed bytes in the middle of a stream,
fuzz with random bytes, and the same C code (frame layer) compiled for host and run against the Python implementation.

### Language-neutral core (because the Pi language was postponed)

The frame layer (COBS + CRC + message structures) is written as **portable C99 without Arduino dependencies**. The same file is used on the Mega,
compiled and tested on a PC, and can be called from C/C++ or Rust (FFI) on the Pi. Python is only used as an independent
test counterpart (two separate implementations that must give the same bytes = a good test). When the Pi language is chosen, the Pi library is written
on top of the spec — either as a binding to the C core or as its own implementation tested against the same test vectors.
*Superseded: the Pi language is chosen — Rust (decided 2026-10-12, §0c).*

## 8. Recommendation (short, updated after round 1)

**One Mega as real-time hub (reflex and control layers) ↔ Pi over GPIO-UART (Mega Serial1, BSS138, 250 kbaud, maybe 500k) with an own
binary protocol (COBS + CRC-16), 200 ms command watchdog with explicit ARM, and a language-neutral C core.** The USB cable is used
for flash/debug and is plan B for the link. ESP-01 is dropped. WS2812B on a Nano/Uno.
MAVLink is a reasonable alternative if you later want to use existing tools — but it is more machinery than the robot needs now.

## Sources

- micro-ROS supported hardware: https://micro.ros.org/docs/overview/hardware/ and maintainer answer about Uno/Mega: https://discourse.ros.org/t/micro-ros-on-arduino/17009
- Pi 5 UART (GPIO14/15, `ttyAMA0` vs `ttyAMA10`, overlays): https://github.com/raspberrypi/documentation/issues/3239 , https://pinout.xyz/pinout/pin10_gpio15 — **must be verified on the Pi** (`ls -l /dev/serial*`), the sources disagree.
- Baud rate error at 16 MHz: https://www.mikrocontroller.net/topic/361697 + own U2X calculation (should be checked against the ATmega2560 datasheet's UBRR table).
- COBS: https://satlab.aau.dk/stem/jensd/doc/packetprotocols/cobs.html ; PacketSerial (COBS/SLIP, no CRC): https://www.arduino.cc/reference/en/libraries/packetserial
