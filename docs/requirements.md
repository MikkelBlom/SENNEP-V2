# Requirements and architecture

Current values. Where a newer decision replaced an older requirement, the old value is noted so the change is visible. Open questions live in `ai-instructions/WORKING_NOTES.md`.

## Physical requirements
| Requirement | Current value | Earlier value |
|---|---|---|
| Footprint | **800 × 650 mm** (2026-10-05) | 650 × 400 (2025), 500 × 816 (Robot v58) |
| Height | LiDAR on top, max 300 mm above the floor; shell below z ≈ 264 | |
| Wheels | Own printed mecanum, Ø106.2, inside the body | "110 mm" (2025), Ø130 (bbox misread) |
| Manufacture | 100 % 3D-printed PETG, Bambu X1C, **220 mm** part limit | 256 printer limit. The library's free laser cutter is an option but not preferred. |
| Fasteners | M3 at corners and bearings, M4 at the motor collars, printed M8 in frame joints, M3 brass inserts | "only M3, 3 mm walls" (2025) |
| Assembly | Modular and replaceable parts, no glue, no extra purchases | |
| Payload | ~50 kg total (robot + load) | |
| Environment | Indoor, **not waterproof**, ventilation kept | "rain + puddles" (Apr 2025) |
| Form | Simple, symmetric, "self-built and engineered"; not sci-fi, no hard organic curves; large chamfers OK; flat top with the LiDAR in the middle and a free 360° view | |
| Shell | Swappable later: everything needed to drive sits on the skeleton (incl. LiDAR, ToF and edge sensors) | |

## Sensors and outputs
- **Navigation:** LDROBOT STL-19P / LD19 360° LiDAR (UART 230400 baud), 8× VL53L0X ToF low around the body (N/S/E/W + 4 angled corners, via TCA9548A because they share address 0x29), 4× E18-D80NK edge detectors in the corners, MPU-6050 IMU, QMC5883L magnetometer, NEO-6M GPS (to be included).
- **Vision/UI:** Pi AI Camera (IMX500) and 7" HDMI touch screen in the sloped front, head-lights (3 W LEDs) and tail-lights, WS2812B strip.
- **Other:** speaker(s) + PAM8403 / DFPlayer, buzzer, microphones (level only, no direction), temperature/humidity, light sensor (TSL2561), 40 mm fans.
- **Docking (later):** 940 nm IR beacons on the dock, VS1838B receivers on the robot, charging via two metal plates and spring contacts.

## Architecture
- **Raspberry Pi 5:** high level: LiDAR, camera, screen, most sensors. Python from scratch, **no ROS 2**. OS installed May 2025 (user `sennep`).
- **Arduino Mega (main) and Nano:** low level: motor control, secondary collision avoidance, RF override (NRF24L01 + joystick). The main link is **Pi ↔ Mega**, and the Nano is also used. The exact split is not settled (a consolidation is planned).
- **No wheel encoders** (deliberate choice, Apr 2025).
- Power: see `motor-and-power.md`.
