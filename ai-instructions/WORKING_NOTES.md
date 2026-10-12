# Working notes

## Open concerns
Kept open until Mikkel decides. A full consolidation is planned; do not resolve these silently.

| # | Concern | Since | Notes |
|---|---|---|---|
| C1 | **Wheelbase 550** was set by Claude, never confirmed | 2026-10-05 | Same 125 mm overhang as before |
| C2 | **Pi ↔ Mega vs Nano split** | 2026-10-12 | Main link is Pi ↔ Mega (Mikkel), Nano also used. Older notes say Pi ↔ Nano UART with Mega as plan B. Who does what is not settled. |
| C3 | **KFL003 is self-aligning** | 2026-10-05 | If the seat rotates under load, the moment goes into the motor shaft (~450 N vs 50 N allowed). Load test planned. |
| C4 | **No motor lock** | 2026-10-06 | Only the coupling set screw holds the motor axially |
| C5 | **Wheel fits unconfirmed** | 2026-10-11 | No record of the v3 quick-test result before the full v3.1 wheel |
| C6 | **H-bridge chip** | 2025-12 | Assumed BTS7960, truncated item name. Read the chip marking. |
| C7 | **Bot'n Roll MOSFETs** | 2026-10-07 | Assumed IRFZ44N. IRFZ44N is not logic-level (barely opens at 3.3 V; OK from 5 V for the 3 W LEDs, which need current limiting). |
| C8 | **E-stop rating** | 2025 | The small 40 mm E-stop is probably ~10 A, so it drives the 40 A relay instead of breaking main power |
| C9 | **Battery series connections** not made | 2025-05 | Mikkel is wary of soldering/welding cells |
| C10 | **TVS diodes** not on the parts list | 2025 | Part of the V1-spike protection plan |
| C11 | Housing print guide says "≈160 g **PLA**"; everything else is PETG | 2026-10-12 | `print/housing_v2/LÆS MIG.md` |
| C12 | Housing material use | 2026-10-12 | Hollow variant uses more filament. Real savings need a smaller outer volume. |
| C13 | Short set screw | 2026-10-12 | M3 × 12 is the shortest in the kit. A short grub screw (M3 × 5–6) would be better. |
| C14 | Thin wall at the wheel pin root | 2026-10-05 | Pin root only 0.9 mm under the rim edge (generator warns). Accepted for now because the thrust ring surrounds it. |
| C15 | LiDAR test app build | 2026-10-04 | `next build` fails on missing `onCalibrationProgress` in the preload types |
| C16 | Naming "pod" vs "housing" | 2026-10-12 | Mikkel says housing; code and Fusion components still say pod |
| C17 | Unreviewed sources | 2026-10-04 | ChatGPT project (8 chats + 2 docx) not read; the ChatGPT zip with images not received |
| C18 | Screen mounting holes | 2025 | Wrong in the laser-cut test. Check the Waveshare drawing before the body panels. |

## Decisions
Newest first. Reasoning included so they are not re-litigated.

| Date | Decision | Why | Rejected |
|---|---|---|---|
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
| 2025-04 | No ROS 2, Python from scratch on the Pi | Learning | ROS 2 (invented by a ChatGPT summary) |
| 2025-04 | No wheel encoders | Deliberate | — |
| 2025 | 3S6P, not 6S3P | 12 V motors; BMS and charger are 3S | 6S3P (early Gemini chat) |
