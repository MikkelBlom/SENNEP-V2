# Motors and power

## Motor: RS PRO 834-7641 (975D series)
Datasheet: `components/datasheets/rs-pro-834-7641-motor.pdf`. Four motors, carried over from V1 (bought 2020–23, probably through school).

| | |
|---|---|
| Type | 12 V brushed DC, planetary gear 49:1 |
| Size | Gearbox Ø42.4 × 45.9, motor Ø44.6 × 66, total 111.9 mm (front to terminal end ~120.5), 624 g |
| Shaft | Ø8 × 20 mm with D-flat (7 mm) |
| Front flange | 4× M4 × 6 deep on PCD 35, spigot Ø25 × 2 |
| Speed | 143 rpm no-load at 12 V (107 at 8 V, 72 at 6 V) |
| Power / efficiency | 41.3 W / 60 % |
| Torque | **rated 18 kgf·cm (1.77 Nm)**, short-term max 54 kgf·cm (5.3 Nm) |
| Shaft load | **radial < 5 kgf** (10 mm from flange), axial < 13 kgf → the motor must not carry the robot |
| Current | 3–4 A normal, > 10 A stall |

Mikkel's caliper model of the gearbox said Ø41.5. **The datasheet value (Ø42.4) is used.**

### Load estimate
Ø106 wheel (r ≈ 0.053 m) gives 33 N per wheel at rated torque. With 60 % utilisation, mecanum efficiency 0.7 forward / 0.5 sideways and 0.2 m/s² acceleration: **~90 kg total on hard floor and ~48 kg on carpet** going forward. Top speed ~0.8 m/s at no load (realistically lower). **Design target: 50 kg total** (robot + load).

Skeleton loads at 50 kg × 2 for bumps: 245 N per wheel. The weakest point is the **self-aligning KFL003**: if its spherical seat rotates under load, the moment goes into the motor shaft (≈ 450 N against 50 N allowed). Suggested test: hang 10–15 kg 55 mm out on the axle. Plan B is 2 bearings per wheel or an outer support. Mikkel chose to build first and re-evaluate later (no budget for more bearings).

## Motor drivers
- **2× L298N** (dual channel, enough for 4 motors): max ~2 A/channel and ~2 V drop. OK for a light test robot with soft-start.
- **5× high-power H-bridges** (AliExpress, Dec 2025, "43 A"): meant to replace the L298Ns (4 in use + 1 spare). The chip is **assumed to be BTS7960 but not confirmed** (the item name in the order mail was truncated).

## Battery: 3S6P
- 18 of 20 Samsung INR18650-35E (3400 mAh) cells → **10.8 V nominal / 12.6 V full, ~20.4 Ah, ~220 Wh**, estimated runtime 3.5–5 h.
- The parallel groups were built in May 2025 with a 3S BMS and pure nickel strip (0.15 × 27 mm). **The series connections are not made yet** (Mikkel is wary of soldering/welding cells). A separate workstream plans this with materials already owned.
- Pack size (without BMS and wiring): **122.7 × 62.1 × 67.8 mm**.
- Charger: 3S 12.6 V 2 A. BMS: 3S 40 A ×2.
- Note: an early Gemini chat assumed 6S3P (21.6 V). That would destroy the 12 V motors, and BMS, charger and motor chat all confirm 3S.

## Power tree (planned)
3S pack → fuse (blade fuses) → main switch: the **E-stop drives a 12 V 40 A relay** (the small 40 mm E-stop is probably only rated ~10 A) → 12 V directly to the motor drivers. Buck converters supply logic: 5 A buck for the Pi 5 (5 V / 5 A), LM2596 modules for other 5 V rails. TVS diodes, Schottky diodes and capacitors guard against the voltage spikes that killed V1's Arduino. An INA219 monitors the battery, and a 0.28" voltmeter gives a visible readout.

Open: TVS diodes are not on the parts list, and the power distribution / PCB is not designed yet (breadboard first, PCB after).
