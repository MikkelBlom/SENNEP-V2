# Working notes

## Open concerns
Kept open until Mikkel decides. A full consolidation is planned; do not resolve these silently.

| # | Concern | Since | Notes |
|---|---|---|---|
| C1 | **Wheelbase 550** was set by Claude, never confirmed | 2026-10-05 | Same 125 mm overhang as before |
| C2 | ~~Pi ↔ Mega vs Nano split~~ **Resolved 2026-10-12**: one Mega is the real-time hub; a Nano/Uno only drives the WS2812B strip | 2026-10-12 | See decisions |
| C3 | **KFL003 is self-aligning** | 2026-10-05 | If the seat rotates under load, the moment goes into the motor shaft (~450 N vs 50 N allowed). Load test planned. |
| C4 | **No motor lock** | 2026-10-06 | Only the coupling set screw holds the motor axially |
| C5 | **Wheel fits unconfirmed** | 2026-10-11 | No record of the v3 quick-test result before the full v3.1 wheel |
| C6 | **H-bridge chip** | 2025-12 | Assumed BTS7960, truncated item name. Read the chip marking. |
| C7 | **Bot'n Roll MOSFETs** | 2026-10-07 | Assumed IRFZ44N. IRFZ44N is not logic-level (barely opens at 3.3 V; OK from 5 V for the 3 W LEDs, which need current limiting). |
| C8 | **E-stop rating** | 2025 | The small 40 mm E-stop is probably ~10 A, so it drives the 40 A relay instead of breaking main power |
| C9 | **Battery: no cells connected at all** (only clicked into holders; older notes wrongly said the P-groups were built) | 2026-10-12 | Plan approved: spot-welding service. `docs/electronics/BATTERY_PLAN.md` |
| C10 | **TVS diodes** not on the parts list | 2025 | Part of the V1-spike protection plan |
| C11 | Housing print guide says "≈160 g **PLA**"; everything else is PETG | 2026-10-12 | `print/housing_v2/LÆS MIG.md` |
| C12 | Housing material use | 2026-10-12 | Hollow variant uses more filament. Real savings need a smaller outer volume. |
| C13 | Short set screw | 2026-10-12 | M3 × 12 is the shortest in the kit. A short grub screw (M3 × 5–6) would be better. |
| C14 | Thin wall at the wheel pin root | 2026-10-05 | Pin root only 0.9 mm under the rim edge (generator warns). Accepted for now because the thrust ring surrounds it. |
| C15 | LiDAR test app build | 2026-10-04 | `next build` fails on missing `onCalibrationProgress` in the preload types |
| C16 | Naming "pod" vs "housing" | 2026-10-12 | Mikkel says housing; code and Fusion components still say pod |
| C17 | Unreviewed sources | 2026-10-04 | ChatGPT project (8 chats + 2 docx) not read; the ChatGPT zip with images not received |
| C19 | **Motor stall current unknown** | 2026-10-12 | Datasheet's "5.5 mA" is an error; similar motors 12–25 A. 4 stalled motors > BMS 40 A → soft-start + current limit are mandatory. Measure winding resistance. |
| C20 | **E-stop sense needs a second NC contact** | 2026-10-12 | Mega pin 23 reads the e-stop fail-safe; contact type of the 40 mm e-stop unknown. Fallback: A4 bus-voltage divider |
| C21 | **Arm button vs Pi-only ARM** | 2026-10-12 | Breadboard plan has a physical arm button (pin 29); link safety rules say ARM comes from the Pi. Role undecided |
| C22 | **Test LiPo (Zippy 1000 mAh, no BMS) fried V1** | 2026-10-12 | Only with a fuse < 10 cm from the XT60, LiPo alarm, never < 10.5 V under load |
| C23 | Inventory says 2 Mega / 2 Nano / 0 Uno; Mikkel says 1 Mega, several Uno, a few Nano | 2026-10-12 | Settle in the parts count |
| C18 | Screen mounting holes | 2025 | Wrong in the laser-cut test. Check the Waveshare drawing before the body panels. |

## Decisions
Newest first. Reasoning included so they are not re-litigated.

| Date | Decision | Why | Rejected |
|---|---|---|---|
| 2026-10-12 | Mega stays Arduino C++ with the Arduino IDE; Rust only on the Pi | Code already written and tested; IDE + Upload keeps working; AVR is Rust's weakest target | Rust on the Mega (avr-hal); later Rust MCU → RP2040/Pico |
| 2026-10-12 | Project hub `hub/`: local HTML over plain CSV/Markdown files in the repo, edits saved through Chrome folder access | Mikkel and Claude edit the same files; no server, works offline | Tiny local server; online claude.ai page |
| 2026-10-12 | All docs in English, including the electronics and LiDAR plans | One language (docs session choice) | — |
| 2026-10-12 | Battery: measure and match cells first, then spot-weld at a workshop (Fyns Batteri Service first) | Mikkel is wary of soldering/welding cells; 48 W iron too weak | Printed compression pack (backup), soldering on cells, magnets, loose strip |
| 2026-10-12 | Breadboard M1 = Pi 5 + Mega + 4 motors + LiDAR; power on Wago/screw terminals, signals on breadboard; e-stop cuts the 40 A relay coil | First test matches what Mikkel is building now | Power through breadboard/Dupont |
| 2026-10-12 | Mega pin map = `firmware/sennep_mega/config.h` (the breadboard plan follows it) | One map, already compiled and tested | Separate map in the plan |
| 2026-10-12 | Pi ↔ Mega: GPIO UART (Serial1 ↔ GPIO14/15) via BSS138, 250 kbaud, reset wire on GPIO4; USB only for debug | Lowest latency, frees the USB, flash from the Pi | USB (breadboard proposal), I²C/SPI |
| 2026-10-12 | Custom binary protocol: COBS + CRC-16, spec in `protocol/messages.json` | Fits the Mega, fully testable | micro-ROS, Firmata, rosserial |
| 2026-10-12 | Reflex architecture: anything < 5 ms (edge, ToF, e-stop, link loss) is decided on the Mega | Latency is the top priority | Safety on the Pi |
| 2026-10-12 | ESP-01 dropped | The Pi has WiFi; converter channels needed for the link | — |
| 2026-10-12 | Rust for all Pi software ("no Python") | Speed; the Pi also runs camera/AI | Python, C |
| 2026-10-12 | LiDAR on the CP2102 USB adapter; Pi sends "nearest free space per sector" (16 sectors) to the Mega | Pi GPIO UART is used by the Mega | LiDAR on a Pi UART |
| 2026-10-12 | Docs restructured: root `CLAUDE.md`, English, `PROJECT_OVERVIEW` split into `docs/` and archived | One source of truth; public repo | Keeping the overview as a living reference (two truths drift) |
| 2026-10-12 | Public repo; private files stay local, order numbers and address scrubbed | Sharing the project | — |
| 2026-10-12 | Every workstream starts "status → research → plan → ask Mikkel → build" | Avoid building the wrong thing | — |
| 2026-10-12 | Housing v2: keeper screws from the back with tool channels | Mikkel damaged v1 drilling from behind and found the through-window method too fiddly | Long screws from behind at BoltAngle 0 (bolt line hits the collar-lug channels; needs BoltAngle ≈ 19.5° + guide tubes) |
| 2026-10-12 | Through-wheel window method valid for v1 (Ø6 corridor at r = 28, wheel angle 16–44°) | No reprint needed | — (later superseded by v2) |
| 2026-10-12 | Wheel assembly ring | Rollers fell off Rim A's short pins | — |
| 2026-10-12 | No tunnel from cloud sessions to Fusion | Unauthenticated remote execution of arbitrary Python | — |
| 2026-10-06 | Printed M8 × 1.25 bolts in frame joints, thread cut with tap/die | Mikkel has too few M3 (BOM wanted 144 M3 × 12). M3 need is now 40 = the kit. | More bought M3; teardrop/printed threads printed horizontally (orientation concern) |
| 2026-10-06 | All holes round | Mikkel does not want teardrops; round holes up to Ø18 bridge fine | Teardrops |
| 2026-10-06 | Ladder frame instead of floor trays | Mikkel wanted a "real" car frame | 12 bolted trays (v1–v2) |
| 2026-10-06 | Diamond channel replaces the tow-hitch receiver | A rod can pass the whole length | 20 × 20 receiver + lock hole |
| 2026-10-06 | LiDAR on top of the shell, ≤ 300 mm above the floor; shell below z ≈ 264 | 2° pitch error drops the scan plane 18 mm at the corners | — |
| 2026-10-05 | 800 × 650, 220 mm print limit | Motors collide at 500 width (−11.5 mm). 220 is the real usable plate. | 540 × 700 (sketch), 650 × 400 (2025) |
| 2026-10-05 | Floating motor in a tube, collar lugs take torque only | Motor shaft may not carry load | Rigid motor mount |
| 2026-10-05 | Datasheet values beat caliper model values (gearbox Ø42.4) | Model was hand-measured | Ø41.5 |
| 2026-10-05 | Build first, re-evaluate the bearing later | No budget for more bearings | 2 bearings per wheel |
| 2026-10-05 | 100 % printed wheel, no steel pins | Everything prints on one plate, no tools or purchases | Steel Ø4 pins (v1) |
| 2026-10-05 | Rim B twists on along helical grooves | The turn must follow the pins' path or the pins break | M30 thread + printed nut (v2.4) |
| 2026-10-05 | 9 rollers, Ø21.5 max | Ø22 is the limit before sleeves touch | Ø22–24, 10 rollers |
| 2026-10-05 | Not waterproof, ventilation kept | Indoor robot | Waterproofing (Apr 2025) |
| 2026-10-05 | GPS, IMU and magnetometer included | Mikkel | — |
| 2026-10-04 | Pi 5 (not Pi 4) | The case and the OS install are Pi 5 | — |
| 2025-12 | TCA9548A mux for the 8 VL53L0X | All share 0x29; the mux saves 8 GPIO | XSHUT address assignment |
| 2025-12 | E-stop drives a 40 A relay | Small E-stop rating | E-stop breaking main power |
| 2025-04 | No ROS 2, from scratch on the Pi (language later changed to Rust) | Learning | ROS 2 (invented by a ChatGPT summary) |
| 2025-04 | No wheel encoders | Deliberate | — |
| 2025 | 3S6P, not 6S3P | 12 V motors; BMS and charger are 3S | 6S3P (early Gemini chat) |
