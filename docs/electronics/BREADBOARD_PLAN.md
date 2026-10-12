# SENNEP V2 — Electronics, breadboard version (plan v0.2)

Status: **Plan v0.2 — APPROVED by Mikkel 2026-10-12 (consolidation). Next: wiring diagrams + step-by-step build guide for M1.**
No diagrams yet.
Owner: this file owns the *wiring* (power, connectors, pins, level shifting). The protocol between the Pi and the Mega is
owned by `docs/electronics/PI_MEGA_LINK.md` and `ai-instructions/PLAN_PI_MEGA_LINK.md` (the plan is newer if they disagree).
The Mega pin map lives in `firmware/sennep_mega/config.h`; §2.7 here mirrors it. The battery series connections are a
separate task (`docs/electronics/BATTERY_PLAN.md`).

Goal: a rough first version that can be built with jumpers, screw terminals and Wago clamps, and tested one subsystem
at a time. The choices must move straight over to a PCB later (same connectors, same pin map, same power tree).

> Architecture change from PROJECT_OVERVIEW §3: an **Arduino Mega** (not a Nano) drives the motors. With 4
> BTS7960 (8 PWM + 4 current senses), 8 ToF, 4 edge sensors, NRF24 and 2–3 serial ports, the Nano is too small.

---

## 0. Decisions and scope (Mikkel, 2026-10-12)

- **First milestone (M1):** Pi 5 + Mega + 4 motors with wheels + LiDAR. The Mega drives the motors, the Pi is connected
  to the Mega, and the Pi reads the LiDAR. Everything else (ToF, edge, GPS, IMU, NRF24 …) comes after M1.
- **Build style:** power through Wago/screw terminals, signals on breadboard + Dupont. Mikkel has breadboards, jumpers and
  a resistor kit. No diodes beyond those on the shopping lists. The full component list comes later (separate task).
- **Decided in other sessions (consolidated 2026-10-12):** the physical Pi↔Mega link and protocol (link session) and
  which sensors sit on the Pi vs. the Mega. This plan's earlier proposals in §2.5–2.6 were replaced by those decisions.
- **Budget:** not fixed → §5 has a *minimum* and a *recommended* level.
- **Power for testing:** 12 V adapter (current unknown), multimeter and a **Zippy Compact 1000 mAh 3S 35C LiPo** (XT60,
  charger HobbyKing B3AC). **This is the battery that destroyed V1's Mega and electronics** → §2.8.

---

## 1. Status: all electrical parts

Source: `components/inventory.csv` + datasheets. **Bold** = part of the first version.

### Power and protection

| Part | Voltage | Current | Note |
|---|---|---|---|
| **3S6P 18650 (Samsung 35E)** | 9.0–12.6 V (10.8 nom) | 8 A/cell max → 48 A for the pack | ~20 Ah / ~220 Wh. **Series not soldered yet** |
| **Test: Zippy Compact 1000 mAh 3S 35C LiPo** (not in inventory) | 9.0–12.6 V | 35 A cont., **no BMS** | Destroyed V1. Test only, see §2.8 |
| **3S 40 A BMS** (×2) | 3S | 40 A | Last line of defence, not a fuse |
| **XT60** (×2 pairs) | — | 60 A | Battery connector, polarised |
| **Car blade fuses, assortment** | 32 V | 5–30 A (check which values are there) | **No holders for blade fuses** |
| Fuse holder 5×20 mm (×10) | 250 V | typically max 10 A | Can be used for the logic branch (glass fuse 5 A — not owned?) |
| **Car relay 12 V 40 A, 5-pin** | coil 12 V | coil ~150 mA | The e-stop controls the coil. No built-in flyback diode (probably) |
| **E-stop 40 mm** | ? | ? | Contact type (NC/NO) and rating unknown → it must only break the relay coil |
| **Toggle switch MTS-103** on-off-on 6 A | — | 6 A | Main switch for logic (2.6 A at 10.8 V) |
| Key switch S1203 | — | unknown, small | Signal/coil only, not for power |
| Push button PBS-110 red | — | small | Can be used as "restart motors" (arm button, see §2.4) |
| **DC-DC 5 A buck** (×2, probably XL4015) | in 8–36 V | 5 A (realistically 3–4 A without cooling) | **Pi 5** |
| **LM2596 module with voltmeter** (×3) | in 4.5–40 V | 3 A | **5 V logic** (Mega, sensors, LiDAR) |
| **LD1117V33** (×3) | in ≤ 15 V | 800 mA | **3.3 V** for NRF24 (from 5 V logic). ESP-01 was dropped (§2.5) |
| L7805 (×10), LM2596T-12 (×3) | | | Not needed in v1 (the 7805 gets hot at 12 V → buck instead) |
| Breadboard PSU MB102 (×2) | 6.5–12 V in | ~700 mA | Bench test of single sensors only |
| **INA219** | ≤ 26 V bus | **max 3.2 A with the fitted 0.1 Ω shunt** | Can NOT measure the whole battery current → measures the logic branch (see §3) |
| Electrolytic kit (120 pcs), ceramic kit (≤ 0.1 µF), SMD diodes (70 pcs, types unknown) | | | Largest electrolytic value and voltage unknown |
| IRFZ44N (×5 + 3) | | | Not logic level. Not in v1 (lights later) |

### Motors and drivers

| Part | Voltage | Current | Pins |
|---|---|---|---|
| **4× RS PRO 834-7641** | 12 V | Datasheet: 41.3 W, no-load 5.5 mA (doubtful figure). **Stall current is not in the datasheet** — estimate 15–25 A (must be measured, see §2.1) | 2 terminals |
| **4× BTS7960 module (IBT-2)** (+1 spare) | 5.5–27 V motor, 5 V logic | 43 A per chip (theoretical) | VCC, GND, RPWM, LPWM, R_EN, L_EN, R_IS, L_IS + B+, B−, M+, M− |

BTS7960 (datasheet): PWM up to 25 kHz, current sense `kILIS = IL/IIS ≈ 8500` (5–12 spread!) → 10 A gives 1.2 mA,
i.e. 1.2 V across 1 kΩ. Whether the IBT-2 clone has a resistor on the IS pins is unknown → measure (§7, step 5).
The module's logic GND and motor GND are connected on the board.

### Compute and communication

| Part | Supply | Logic | Interface |
|---|---|---|---|
| **Raspberry Pi 5** | 5.1 V / 5 A (USB-C) | **3.3 V, NOT 5 V-tolerant** | UART ×5, I²C, USB ×4, WiFi |
| **Arduino Mega 2560** (×2) | 5V pin (or VIN 7–12 V) | 5 V | Serial0–3, I²C, SPI, 15 PWM, 16 analog |
| ESP-01 — **dropped** | **3.3 V**, ~300 mA peaks | 3.3 V, not 5 V-tolerant | UART (115200). Not used: the Pi has WiFi |
| NRF24L01 (×3) | 3.3 V (12–115 mA with PA) | 5 V-tolerant inputs | SPI + CE + IRQ |
| **BSS138 level converter, 4 channels** (×1) | HV 5 V / LV 3.3 V | | Only 4 channels in total; 3 are used by the Pi↔Mega link → buy more |
| CP2102 USB-UART adapter (from the LiDAR kit, not in inventory) | USB | 3.3 V | **LiDAR → Pi USB** |
| TCA9548A (×5) | 1.65–5.5 V | | I²C 0x70 |

### Sensors

| Part | Supply | Logic | Interface | Current |
|---|---|---|---|---|
| **LiDAR STL-19P (LD19)** | 5 V (4.5–5.5) | TX 3.3 V | UART 230400, TX only. PWM pin **must go to GND** | 290 mA (start: unknown) |
| **MPU-6050 (GY-521)** | 3.3–5 V (LDO on module) | 3.3 V I²C | 0x68 | ~4 mA |
| **QMC5883L (GY-271)** | 3.3–5 V | 3.3 V I²C | 0x0D | ~2 mA |
| **GY-NEO6MV2 GPS** | 3.3–5 V (LDO on module) | TX 3.3 V | UART 9600 | ~50 mA |
| **8× VL53L0X** | 2.8–5 V (module has LDO + level shifting) | I²C 0x29 (all of them!) | via TCA9548A | ~20 mA each |
| **4× E18-D80NK** | 5 V | NPN open collector (LOW = object seen) | digital | ~25 mA each |
| **INA219** | 3–5.5 V | I²C | 0x40 | |
| 4× HC-SR04 | 5 V | 5 V | trig/echo | 15 mA each |
| TSL2561 | 3.3 V | I²C | 0x39 | |
| DS3231 | 3.3–5 V | I²C | **0x68 = same as MPU-6050** | |
| Microphones, IR receivers, DFPlayer, LCD, LED strip, 3 W LED | — | — | — | Not in v1 |
| Pi AI Camera, 7" screen | via Pi (CSI / HDMI + USB) | | | ~0.2 A + ~0.5 A from the Pi |

### Missing completely (for a safe first version)

- **Holder(s) for blade fuses**, ideally a small fuse box with a minus bar (power distribution in one part).
- **Flyback diode** for the relay coil (1N4007 / 1N5819) and a **TVS diode** on the motor bus (P6KE15A / SMBJ15A).
- **Resistor kit** (no resistors on the list — used for voltage dividers, pull-downs, LEDs).
- **More level converters** (1 pc = 4 channels; the link uses 3 of them).
- **Large electrolytic** 1000–2200 µF / 25 V per driver, if the kit doesn't have one.
- Power distribution: **Wago 221** (or screw clamps/busbar) — power must NEVER go through a breadboard or jumpers.
- **USB-C pigtail** (male) from the 5 V buck to the Pi 5.
- Breadboards and Dupont jumpers, a multimeter and ideally a **bench supply with current limit** are not on the list → questions in §8.

---

## 2. Research

### 2.1 Power budget

| Load | Rail | Typical | Peak |
|---|---|---|---|
| Motor ×4 (driving) | Motor bus 9–12.6 V | 1–3 A each | **stall 15–25 A each (estimate)** |
| Pi 5 + camera + screen | 5 V Pi | 1.5–2.5 A | 4–5 A |
| Mega | 5 V logic | 80 mA | 100 mA |
| LiDAR | 5 V logic | 290 mA | ~500 mA at start-up (guess) |
| 8× ToF + TCA | 5 V logic | 160 mA | 200 mA |
| 4× E18-D80NK | 5 V logic | 100 mA | 100 mA |
| IMU + mag (Mega I²C) | 5 V logic | 6 mA | 10 mA |
| GPS (Pi UART) | 5 V logic | 50 mA | 70 mA |
| NRF24 (via LD1117) | 3.3 V | 15 mA | 115 mA |
| BTS7960 logic ×4 | 5 V logic | 20 mA | 40 mA |
| Relay coil | 12 V logic | 150 mA | 150 mA |
| **Logic total (5 V logic)** | | **~0.9 A** | **~1.5 A** → LM2596 (3 A) OK |
| **Logic total from the battery** (η ≈ 0.85) | | **~1.5–2 A** | **~3 A** → 5 A fuse |

The totals still include the dropped ESP-01 (was "ESP-01 + NRF24: 100 mA / 450 mA"), so they now carry ~0.3 A of extra
margin at peak. The old row "GPS + IMU + mag: 60 mA / 80 mA" is split above because the IMU and mag moved to the Mega.

**The motors are the whole problem.** Four stalled motors can in theory draw 60–100 A, but the BMS cuts out at 40 A,
and the cells can give 48 A. Therefore:
1. **Soft start and current limit in code** (PWM ramp, read the IS pin, e.g. 8 A limit per motor).
2. **Fuse per motor** (15 A blade) and a **main fuse** (30 A blade) — the fuse protects the *wires*.
3. **Measure the stall current** before the first run with the wheels on (step 5 in §7: short pulse, blocked shaft,
   bench supply with current limit, or a multimeter on the 10 A range for at most 1 second).

Run time (rough): logic ~25 W → ~8 h idle; driving at 4 × ~2 A × 11 V ≈ 90 W + logic → ~2 h. Uncertain, because the
motors' real current under load is not known.

### 2.2 Wire gauges (what is already in the drawer)

| Run | Current | Cable | Fuse |
|---|---|---|---|
| Battery → XT60 → main fuse → relay → motor bus | ≤ 40 A | **10 AWG silicone** | **30 A** blade with 3S6P, **20 A** with the test LiPo (§2.8) — right at the XT60 |
| Motor bus → each BTS7960 (B+/B−) | ≤ 15 A | **16 AWG** | **15 A** blade per driver |
| BTS7960 → motor | ≤ 15 A | **16 AWG**, twisted pair | — |
| Battery (after main fuse) → switch → bucks | ≤ 3 A | 16 AWG (22 AWG is enough electrically, but 16 is more robust) | **5 A** |
| 5 V buck → Pi 5 | 5 A | 16 AWG or USB-C pigtail, < 20 cm | (the buck's own limit) |
| Signals | < 100 mA | 22 AWG / Dupont | — |

Rule of thumb used: silicone 10 AWG ~55 A, 16 AWG ~20 A, 22 AWG ~5 A in free air on short runs. We are well
below. Voltage drop on the 5 V to the Pi: 0.4 m of 16 AWG (13 mΩ/m) × 5 A ≈ 26 mV — fine. Dupont pins handle ~1 A, so
**the Pi must not be powered through jumpers on the GPIO header**.

### 2.3 Ground and noise

- **Star ground:** one common point for minus (the minus bar in the fuse box/Wago block close to the battery). The motor
  bus minus and the logic minus each run there separately. No motor current in the signal wires' GND.
- Because the BTS7960 module connects logic GND and motor GND on the board, there is a loop: Mega → driver → star → buck →
  Mega. It can't be avoided without optocouplers. Keep it small by running each driver's signal wires + GND **together and
  short**, and by returning the motor current on its own thick wire.
- **Per motor:** 100 nF ceramic across the terminals (+ optionally 2× 100 nF from each terminal to the motor case),
  twisted motor wires.
- **Motor bus:** 1000–2200 µF electrolytic at the drivers + **TVS 15 V** (catches spikes when the relay opens while
  driving — the kind of spike that killed V1's Arduino).
- **PWM 20 kHz** (inaudible, below the BTS7960's 25 kHz limit) on the Mega's Timers 3/4/5. Timer 0 (millis) is not touched.
- Each buck: 100 nF + 100 µF on the output at the load. The LD1117 needs 10 µF in and out (otherwise it oscillates).
- LiDAR, GPS and IMU physically away from the motor wires; the magnetometer at least 10–15 cm from the motors and the
  10 AWG cables (DC current makes a magnetic field → compass error).
- Pi↔Mega link wires (TX/RX/GND, and the reset wire) short and twisted, routed away from the motor wires.

### 2.4 Safe e-stop

The e-stop breaks the **relay coil**, not the main power (its rating is unknown, and a 40 mm e-stop is rarely good for
40 A DC). Only the motor bus is cut; the logic keeps running, so the Pi doesn't lose its SD card and the Mega can log why.

```
 +12 V logic (after main switch) ──[E-STOP NC]──┬── relay 86
                                                │      relay coil        ┌─ 1N4007 across the coil
 GND ───────────────────────────────────────────┴── relay 85 ────────────┘  (cathode towards +)

 Main fuse 30 A ── relay 30 ══ relay 87 (NO) ══ motor bus ── 4× 15 A ── BTS7960 ×4
```

- **Fail-safe:** broken wire, logic off or e-stop pressed → coil without power → motors dead. It does not depend on
  software.
- **E-stop sense (Mega pin 23):** a separate NC contact on the e-stop to pin 23 → GND (INPUT_PULLUP). Pressed **or a
  broken wire** reads HIGH → `ESTOP`. This needs a second contact on the e-stop (see §8, item 4). The motor bus divider
  on A4 also shows whether the bus is live.
- **No automatic restart:** when the e-stop is twisted out, the motor bus comes back, but the Mega sees the bus (voltage
  divider → A4) and keeps all EN pins LOW until it gets an explicit `ARM`. A pure hardware latch needs an extra relay
  (the car relay has only one contact) → can come on the PCB.
- **Arm button (pin 29, red PBS-110) — OPEN QUESTION.** The link's safety rules (approved) say `ARM` must come from the
  Pi, and a Mega reboot must never make the robot drive by itself. The breadboard plan also had a physical arm button.
  It is now either (a) an extra *required* local confirmation on top of the Pi's `ARM`, or (b) a local override that
  arms without the Pi. Not decided; the pin is reserved in `config.h`, and the firmware does not read it yet.
- **Layer 2:** the Mega pulls the shared EN LOW on Pi-link timeout (200 ms), current limit or edge sensor. A 10 kΩ
  pull-down on EN keeps the drivers off while the Mega boots or resets.
- **Layer 3:** BMS (40 A) and main fuse.
- Known weakness: opening the relay under full load arcs the contacts. The Mega cannot turn the drivers off first,
  because it only sees it afterwards. Accepted in v1; on the PCB, the e-stop's second contact can also be wired so the
  Mega sees it a little earlier.

### 2.5 Who controls what (DECIDED — from the link plan)

Ownership follows `ai-instructions/PLAN_PI_MEGA_LINK.md` §3 (reflex architecture, D7/D10: "latency first"). Anything that
must react in < 5 ms is decided on the Mega without the Pi.

| Device | Owner | Why |
|---|---|---|
| 4× BTS7960 (8 PWM at 20 kHz, shared EN, 4× IS) | Mega | Real time, fail-safe |
| E-stop state, 4× E18 edge, 8× VL53L0X (TCA9548A), 4× HC-SR04 | Mega | Local stop without the Pi |
| MPU-6050 + QMC5883L | Mega (I²C, 5 V side) | Raw data at 100/50 Hz; heading hold possible locally |
| NRF24L01 RF joystick | Mega | Manual override must work with the Pi dead |
| INA219 (battery voltage) | Mega | Low-battery stop |
| WS2812B | Nano/Uno node on Mega Serial3 | The strip blocks interrupts ~4.5 ms per update |
| GPS NEO-6M | Pi (own UART, 3.3 V) | 1 Hz, not time-critical |
| LiDAR LD19 (via CP2102 USB), camera, 7" screen, audio/DFPlayer, WiFi | Pi | Heavy processing / not real time |
| ESP-01 | dropped | The Pi has WiFi, and its converter channels are needed for the link + reset |

Changed from this plan's earlier proposal: the IMU/mag were on the Pi ("3.3 V parts to the Pi, 5 V parts to the
Mega"). They move to the Mega, because heading and tilt feed the local reflexes and drive control; latency comes first,
and level shifting is only a small cost. The Pi still has its own RTC (needs a coin cell) → the DS3231 is redundant.

### 2.6 Level shifting

| Link | Direction | Solution |
|---|---|---|
| Pi ↔ Mega (link) | both | **GPIO UART through the BSS138**: ch1 Mega TX1 (18) → Pi RX (GPIO15), ch2 Pi TX (GPIO14) → Mega RX1 (19). 250 000 baud |
| Pi GPIO4 → Mega RESET | 3.3 → 5 | BSS138 ch3 (flash the Mega from the Pi) |
| LiDAR TX → Pi | 3.3 → USB | CP2102 USB adapter, `/dev/serial/by-id/...` (the GPIO14/15 UART is taken by the Mega) |
| GPS ↔ Pi | 3.3 ↔ 3.3 | direct (the GPS module is powered at 5 V; its LDO gives 3.3 V logic) |
| IMU/mag ↔ Mega I²C | 5 V side | direct, modules powered at 5 V (onboard LDO); check pull-ups/levels when measuring |
| NRF24 ↔ Mega SPI | | direct (inputs 5 V-tolerant, supply 3.3 V from LD1117) |
| BTS7960 ↔ Mega | 5 V | direct |
| ToF modules, TCA9548A, INA219 ↔ Mega I²C | 5 V | direct (the modules have their own LDO/level shifting; check pull-ups when measuring) |

BSS138: HV = 5 V, LV = 3.3 V, common GND. Ch4 is free. Check that the Mega TX → Pi RX line never exceeds 3.3 V.

USB between the Mega and a PC/Pi is only for debugging and plan B. If USB is plugged in while the Mega is powered on the
5V pin from the LM2596, the USB 5 V wire must be **cut** (tape over the 5 V pin in the USB-A plug, or a "data only"
cable). Otherwise the Pi's (or PC's) 5 V and the buck's 5 V are joined through the Mega's USB FET.

### 2.7 Pin map

**Arduino Mega** — the single map is `firmware/sennep_mega/config.h`. Change pins there first, then here.

| Function | Pin | Note |
|---|---|---|
| Motor FL (front left) RPWM / LPWM | 2 / 3 | Timer 3 |
| Motor FR RPWM / LPWM | 5 / 6 | Timer 3 / 4 |
| Motor RL RPWM / LPWM | 7 / 8 | Timer 4 |
| Motor RR RPWM / LPWM | 44 / 45 | Timer 5 (46 in reserve) |
| Shared EN (all R_EN + L_EN) | 22 | 10 kΩ pull-down. LOW = drivers off |
| Current sense FL / FR / RL / RR (R_IS + L_IS joined) | A0 / A1 / A2 / A3 | Only one side conducts at a time |
| E-stop sense | 23 | INPUT_PULLUP, NC contact to GND. HIGH = pressed or broken wire (fail-safe) |
| Edge sensors E18 FL / FR / RL / RR | 24 / 25 / 26 / 27 | INPUT_PULLUP. HIGH = no floor = stop. Broken wire = HIGH = also stop |
| HC-SR04 shared trigger | 28 | Reserved (driver not written) |
| **Arm button (red PBS-110)** | **29** | INPUT_PULLUP. Reserved, `ARM_BUTTON_PIN`. Role open (§2.4) |
| Buzzer (active) | 30 | |
| Dock IR receivers (4× VS1838B) | 31–34 | Reserved |
| Headlight / taillight PWM | 9 / 10 | Timer 2 → MOSFET gate driver (IRFZ44N is not logic level) |
| Status LED | 13 | Built in |
| **Motor bus voltage (e-stop status)** | **A4** | Divider 47 k / 10 k → 12.6 V = 2.2 V. Reserved, `MOTOR_BUS_SENSE_PIN` |
| HC-SR04 echo ×4 | A8–A11 | PCINT16–19 |
| I²C SDA / SCL | 20 / 21 | MPU-6050 0x68, QMC5883L 0x0D, INA219 0x40, TCA9548A 0x70 (8× ToF), LCD 0x27 later |
| Serial0 (USB) | 0 / 1 | Debug / flash only (115200) |
| Serial1 | 18 / 19 | **Pi link** via BSS138, 250 000 baud |
| Serial2 | 16 / 17 | Expansion node #2 (free) |
| Serial3 | 14 / 15 | LED node (Nano/Uno with WS2812B) |
| NRF24 MISO / MOSI / SCK / CSN / CE | 50 / 51 / 52 / 53 / 48 | IRQ not reserved yet (49 is free if needed) |
| RESET | RESET | ← Pi GPIO4 via BSS138 ch3 |

Changed from this plan's earlier map: edge sensors 30–33 → 24–27, arm button 24 → 29, HC-SR04 34–41 → 28 + A8–A11,
Pi link Serial2 → Serial1, ESP-01 on Serial1 removed, DFPlayer reserve on Serial3 → LED node (DFPlayer is on the Pi).

**Raspberry Pi 5** (physical pins on the 40-pin header)

| Function | GPIO (pin) | Note |
|---|---|---|
| UART0 TX / RX ↔ Mega Serial1 | 14 (8) / 15 (10) | via BSS138 ch2 / ch1. Serial login console off. Device name to check (`ttyAMA0` vs `ttyAMA10`) |
| Mega RESET | 4 (7) | via BSS138 ch3 |
| uart3 TX / RX ↔ GPS | 8 (24) / 9 (21) | `dtoverlay=uart3-pi5`. Takes SPI0 → no SPI on the Pi |
| LiDAR | USB | CP2102 adapter, `/dev/serial/by-id/...`. LiDAR PWM pin to GND |
| 5 V | — | **via USB-C pigtail, not the header.** `usb_max_current_enable=1` in config.txt (no PD negotiation from a buck) |

(The UART overlay pins for the Pi 5 come from secondary sources — check with `pinctrl` on the real Pi.)
Removed from the Pi: I²C1 to the IMU/mag (GPIO2/3), MPU-6050 INT (GPIO17), uart2 RX for the LiDAR (GPIO5).

### 2.8 The test battery: Zippy Compact 1000 mAh 3S 35C (no BMS)

Facts: 11.1 V nom / 12.6 V full, 1 Ah ≈ 11 Wh (the 3S6P pack has 20× as much). 35C = **35 A continuous**, and a
short circuit gives far more — there is no BMS to cut out. Together with the missing protection, this is probably what
caused the V1 damage (exact cause unknown; typical culprits are spikes from motors without TVS/capacitors, 12 V on the
Mega's 5V pin, or a short circuit without a fuse).

Rules while the LiPo is in use:
1. **Fuse right at the XT60 connector** (< 10 cm of wire before the fuse): **20 A** in the LiPo period (30 A with 3S6P).
2. **Never below 3.5 V/cell under load (10.5 V), never below 3.0 V/cell at rest.** The LiPo has no protection.
   Minimum: a "LiPo alarm" (buzzer on the balance plug). The Mega also measures the bus (A4) and stops the motors below 10.5 V.
3. Capacity: Pi 5 + LiDAR + Mega alone ≈ 2–2.5 A → ~20–25 min. With 4 motors running only **~5–10 min**. Therefore:
4. **Bench mode with two sources** (recommended for M1): the logic branch (Pi + Mega + LiDAR) on the 12 V adapter (must
   handle ≥ 3 A — check the label) and the motor bus on the LiPo through fuse + relay. The two sources are joined
   **only at GND** (the star point), never on plus. When the 3S6P is ready, it powers both branches and the adapter is removed.
5. Charge and store in a LiPo bag; never leave the charger unattended. Storage voltage 3.8 V/cell.
6. First connection always with the motors disconnected and a multimeter on the bus. First motor start without the wheels on the floor.

---

## 3. Block diagram

```
                       ┌──────────────┐
  3S6P + BMS 40 A ─XT60┤  MAIN FUSE   ├──┬───────────────────────────────────────────────┐
  (9–12.6 V)           │    30 A      │  │                                               │
                       └──────────────┘  │ 10 AWG                                        │ 16 AWG
                                         ▼                                               ▼
                             ┌─────────────────────┐                           ┌───────────────────┐
                             │ RELAY 40 A (30→87)  │◄── coil ── E-STOP NC ◄────┤ MAIN SWITCH       │
                             └─────────┬───────────┘    (2nd NC → Mega 23)     │ MTS-103 + 5 A     │
                                       │ motor bus  + TVS 15 V + 2200 µF       └────────┬──────────┘
                                       │                                                │ 9–12.6 V logic
              ┌──────────┬─────────────┼───────────┐                       ┌───────────┼─────────────┐
            15 A       15 A          15 A        15 A                     INA219      (shunt 0.1 Ω, ≤3 A)
              ▼          ▼             ▼           ▼                         ▼           ▼             ▼
          BTS7960    BTS7960       BTS7960     BTS7960              5 V/5 A buck    LM2596 5 V     A4 divider
             FL         FR            RL          RR                 (Pi 5)          (logic)       (bus status
              │          │             │           │                    │               │          from relay)
            Motor      Motor         Motor       Motor              USB-C │     ┌─────────┼──────────┐
                                                                      ▼   │     ▼         ▼          ▼
         PWM/EN/IS ◄──── ARDUINO MEGA ◄── Serial1 ── BSS138 ── UART0 ── PI 5   LiDAR   LD1117 3.3 V
                          │  │  │  │         (+ GPIO4 → RESET)          │  │    │          │
       I²C: TCA9548A ─────┘  │  │  └── NRF24 (3.3 V) ◄─────────────────┼──┼────┼──────────┘
       → 8× ToF, INA219,     │  │                                      │  │    └─ CP2102 USB ─► Pi
         MPU-6050, QMC5883L  │  └── Serial3 → LED node               │  └── GPS (uart3)
       Edge sensors ×4 ──────┘                                         └── camera, screen
  All minus → STAR POINT (minus bar) at the main fuse
```

---

## 4. Build style (so it can become a PCB)

- **Power:** fuse box/Wago + screw terminals on a wooden or printed base plate. Never power through a breadboard.
- **Signals:** breadboard + Dupont for sensors; Mega and Pi on standoffs next to it.
- **Connectors as on the later PCB:** XT60 for the battery, 5.0 mm screw terminals for motors and power, JST PH for sensors.
- **The pin map in §2.7 / `config.h` is frozen** and reused on the PCB; changes go into `config.h` and here first.

---

## 5. Shopping: minimum and recommended (for M1)

Mikkel has breadboards, Dupont and resistors. He may have fuse holders. Prices are rough (AliExpress/Ardustore).

**Minimum — M1 can't be built without these:**

| Item | Qty | approx. DKK | Why |
|---|---|---|---|
| Blade fuse holder (inline, 10–12 AWG) | 1 | 20 | Main fuse at the XT60 (20 A LiPo / 30 A 3S6P) |
| Blade fuse holder (inline, 16 AWG) *or* small fuse box | 4 + 1 | 40 | 15 A per driver + 5 A logic |
| 1N4007 or 1N5819 | 5 | 10 | Flyback across the relay coil |
| TVS P6KE15A (through-hole, fits a screw terminal) | 2 | 15 | Motor bus — against the V1 damage |
| Electrolytic 1000–2200 µF / 25 V | 2–4 | 20 | Motor bus (if the kit doesn't have ≥ 1000 µF/25 V) |
| LiPo alarm (buzzer on balance plug) | 1 | 20 | The test battery has no BMS |
| Wago 221 (or screw clamps) | 10–20 | 50 | Power distribution + star ground |
| USB-C pigtail, male | 1 | 15 | Buck → Pi 5 |
| **Total** | | **~190** | |

**Recommended — makes it safer and easier, and moves straight over to the PCB:**

| Item | approx. DKK | Why |
|---|---|---|
| 6-way blade fuse box with minus bar (instead of inline holders) | 80 | Distribution, fuses and star ground in one part |
| 3× extra BSS138 level converter | 30 | Spares (the one owned is used by the Pi link: 3 of 4 channels) |
| Dedicated 5 V/5 A+ buck for the Pi 5 (if the XL4015 module drops below 5.0 V in the load test) | 60–100 | The Pi 5 throttles below ~4.8 V |
| INA226 + 50 A/75 mV shunt | 60 | Measure the whole battery current (the INA219 can only do 3.2 A) |
| Ferrules + crimp tool, cable lugs, heat shrink | 100–150 | Reliable screw terminals on a robot that shakes |
| Bench supply 30 V/5 A with current limit | 400–600 | Safe first connection of everything (instead of the LiPo) |
| **Total recommended (incl. minimum)** | **~900–1200** | |

---

## 6. Interface to the Pi↔Mega task

The link is decided (`ai-instructions/PLAN_PI_MEGA_LINK.md` D2–D4, §11). This plan provides:
- Pi UART0 GPIO14/15 ↔ Mega Serial1 18/19 through the BSS138 (ch1 Mega TX1 → Pi RX GPIO15, ch2 Pi TX GPIO14 → Mega RX1),
  250 000 baud to start (500 000 after measuring the edges).
- Pi GPIO4 → BSS138 ch3 → Mega RESET (flash the Mega from the Pi with avrdude).
- Common GND, short twisted TX/RX/GND away from the motor wires.
- The e-stop sense contact on pin 23, and EN on pin 22 with a pull-down so the Mega can drop EN on link silence (200 ms).

USB is only for debugging and plan B (cut the USB 5 V, §2.6). Protocol, timeouts and safety rules are owned by the link plan.

---

## 7. Build and test order for M1 (detailed with diagrams after approval)

One subsystem at a time. Don't move on until the check in the right column is OK.

| # | Build | Measure / check before the next step |
|---|---|---|
| 1 | XT60 → 20 A fuse → Wago distribution, **no load**. Star point on minus | Bus voltage = battery voltage; no heat; polarity right on all outputs |
| 2 | Relay + flyback diode + e-stop (NC) in the coil circuit, TVS + electrolytic on the motor bus | Relay clicks at power on; 0 V on the motor bus with the e-stop pressed; correct again when twisted out |
| 3 | Logic branch: switch + 5 A fuse → LM2596 (5.00 V) and 5 A buck (5.15 V), **no load** | Set the voltages *before* anything is connected. Then a dummy load (e.g. a car bulb) → the voltage must not drop below 4.9 V |
| 4 | Mega on the LM2596's 5 V (5V pin, not VIN). No USB from the Pi yet | Mega boots, blink sketch, 5V pin reads 4.95–5.05 V |
| 5 | **One BTS7960 + one motor**, wheel in the air: VCC/GND/RPWM/LPWM/EN/IS to the Mega, 15 A fuse, 100 nF across the motor | Ramp 0→30 % both directions; IS voltage at a known load; measure the stall current in **< 1 s** (otherwise an estimate) |
| 6 | The other three drivers + motors, shared EN with 10 kΩ pull-down, A4 bus divider, e-stop sense on pin 23 | All 4 turn the right way; e-stop while driving stops everything; the Mega keeps EN low after a restart until it is armed |
| 7 | Pi 5 on the 5 A buck via the USB-C pigtail (`usb_max_current_enable=1`) | `vcgencmd get_throttled` = `0x0` under a stress test; buck voltage at the Pi ≥ 5.0 V |
| 8 | LiDAR: 5 V from the LM2596, TX → CP2102 USB adapter → Pi USB (`/dev/serial/by-id/...`), PWM pin to GND | Data at 230400 baud; error-free packets while the motors run (noise test) |
| 9 | Pi ↔ Mega: BSS138 ch1–3 (Serial1 ↔ GPIO14/15, GPIO4 → RESET), as in the link plan §10 | The Mega stops the motors on link timeout; Mega TX → Pi RX never above 3.3 V |
| 10 | Wheels on the floor, short drive | Bus voltage ≥ 10.5 V while driving (LiPo), no heat in wires/fuses |

---

## 8. For the consolidation / still open

Settled: test power (LiPo + 12 V adapter + multimeter), build style, M1 scope, breadboards/jumpers/resistors owned,
Pi↔Mega physical link (GPIO UART, §2.6), sensor split (§2.5), ESP-01 dropped, LiDAR via CP2102 USB.

Open:
1. **Arm button role** (§2.4): extra required confirmation on top of the Pi's `ARM`, or a local override?
2. ~~Sensor split~~ — decided (§2.5).
3. Current rating of the 12 V adapter — can it carry the logic branch (≥ 3 A)?
4. The e-stop's contacts (NC? an extra NO or second NC?) — measure with a multimeter. Pin 23 needs a second NC contact;
   if there is only one, the fail-safe sense must come from somewhere else (e.g. the A4 bus divider).
5. Do the IBT-2 modules have resistors on the IS pins? — measure (resistance IS → GND).
6. Motor stall current — measured in step 5.
7. What is on the 5 A buck (XL4015?) — and does it hold 5.0 V at 4 A?
8. HC-SR04 and DS3231 in later versions? (The Pi 5 has an RTC; ToF + LiDAR cover distance.) ESP-01 is dropped.
9. Does the electrolytic kit have ≥ 1000 µF / 25 V?

---

## 9. Changes in consolidation (2026-10-12)

- Status: plan v0.2 approved by Mikkel; next step is wiring diagrams + a step-by-step build guide for M1.
- Translated from Danish to English. Fixed two step references (IS and stall measurement is §7 step 5, not step 3).
- Pi ↔ Mega is GPIO UART: Mega Serial1 (18/19) ↔ Pi GPIO14/15 via BSS138 at 250 000 baud, plus Pi GPIO4 → ch3 → Mega RESET; USB is only debug/plan B (was "proposal: USB").
- ESP-01 dropped (the Pi has WiFi; the converter channels are needed for the link + reset): removed from Serial1 and the 3.3 V supply.
- LiDAR connects to the Pi through the CP2102 USB adapter (`/dev/serial/by-id/...`), not Pi uart2/GPIO5. GPS stays on Pi uart3.
- Mega pin map now follows `firmware/sennep_mega/config.h` (edge 24–27, sonar 28 + A8–A11, buzzer 30, dock IR 31–34, e-stop sense 23, Serial2 = expansion node, Serial3 = LED node).
- Motor bus divider (47k/10k) → A4 and arm button → pin 29 kept, and added to `config.h` as reserved defines.
- Arm button role flagged as open: the link's safety rules require `ARM` from the Pi.
- Sensor split follows the link plan: IMU + mag move from the Pi to the Mega I²C (reflex architecture, latency first); Pi owns GPS, LiDAR, camera.
