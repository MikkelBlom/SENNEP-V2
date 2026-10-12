# Features

Status: ✅ done · 🔄 in progress · ⏳ waiting (on a print/test/Mikkel) · 📋 planned · ⛔ blocked. Date = last status change.

## Mechanics
| Feature | Status | Description | Date | Depends on | Blocked by |
|---|---|---|---|---|---|
| Wheel generator v3.1 | ⏳ | Printed mecanum wheel, STLs pre-oriented (`print/testhjul_v3.1/`) | 2026-10-11 | — | Mikkel printing and assembling |
| Wheel assembly ring | ⏳ | Cradle that holds the rollers during assembly | 2026-10-12 | Wheel v3.1 | Test print |
| Wheel fit tuning | 📋 | Adjust grooves, bayonet, hex and roller fits after the test assembly | 2026-10-12 | Wheel v3.1 + ring printed | Test result |
| Left/right-handed wheels | 📋 | FR/RL need mirrored rollers (`HAND`) | 2026-10-05 | Wheel fits | — |
| Corner housing v2 | ⏳ | Keeper screws from the back, tool channels (`print/housing_v2/`) | 2026-10-12 | Wheel v3.1 | Test print |
| Motor lock | 📋 | Axial lock for the floating motor (only the set screw today) | 2026-10-06 | Housing v2 | — |
| Housing material reduction | 📋 | Smaller outer volume (e.g. 45° wedge under the motor tube); hollowing alone did not help | 2026-10-12 | Housing v2 tested | — |
| Rebuild "20 Chassis" with housing v2 | 📋 | Frame still uses the old keeper placement (`build_chassis` ~l.1066) | 2026-10-12 | Housing v2 | Fusion (local session only) |
| Ladder frame v3.3 | ✅ | Beams, bumpers, LiDAR mast, V-keys, printed M8, diamond channel | 2026-10-06 | — | — |
| Frame joint test print | ⏳ | V-key + printed M8 bolt/nut (`print/skelet_test/`), then tune `Frame_KeyClear` and `PM8` | 2026-10-06 | — | Test print |
| Full frame STL export | 📋 | All frame parts to `print/` | 2026-10-06 | Joint test | — |
| Battery mount | ✅ | Plate + clamps on the cross beams (modelled) | 2026-10-06 | Battery pack dimensions | — |
| Body v0 | ✅ | Visual shell with all cut-outs | 2026-10-06 | Frame | — |
| Body print panels | 📋 | Split the shell into ≤ 220 mm panels with joints | 2026-10-06 | Body v0 | — |
| Shell brackets / quick release | 📋 | Mount the shell on the frame, swappable | 2026-10-06 | Body panels | — |
| Sensor holders in bumpers | 📋 | ToF + edge sensors on the skeleton | 2026-10-06 | Frame | — |
| Bearing load test | 📋 | 10–15 kg hung 55 mm out to check the self-aligning KFL003 | 2026-10-05 | Housing printed | — |

## Electronics and power
| Feature | Status | Description | Date | Depends on | Blocked by |
|---|---|---|---|---|---|
| Parts list (`components/`) | ✅ | English inventory with `source` + `condition` columns, datasheets | 2026-10-12 | — | — |
| Physical parts count | 🔄 | Tick `counted` per part in the hub (Parts/Tools tabs); set qty, source, condition; add missing parts. Pi 5 RAM/price + LiDAR price still open | 2026-10-12 | Parts list | Mikkel counting |
| 3S6P battery pack | ⏳ | Plan v2 approved: measure + match cells now, then spot-weld at a workshop in Odense (`docs/electronics/BATTERY_PLAN.md`) | 2026-10-12 | — | Mikkel measuring cells + calling the workshop |
| Breadboard circuits (M1) | 🔄 | Plan v0.2 approved (`docs/electronics/BREADBOARD_PLAN.md`). M1 = Pi 5 + Mega + 4 motors + LiDAR. Next: wiring diagrams + build guide | 2026-10-12 | — | — |
| PCB | ⛔ | Carrier/shield board in KiCad. Session stopped: needs a tested breadboard first | 2026-10-12 | Breadboard circuits | Breadboard not built |
| Motor drivers | 📋 | Replace L298N with the 43 A H-bridges | 2025-12 | Chip confirmed | — |
| ESP-01 guide | ✅ | Pinout, wiring SVG, sketches. **ESP-01 dropped from the design** (Pi has WiFi) | 2026-10-12 | — | — |

## Software
| Feature | Status | Description | Date | Depends on | Blocked by |
|---|---|---|---|---|---|
| LiDAR stack v2 (`lidar/`) | ⏳ | Rust: LD19 parser, safety zones, LiDAR odometry, occupancy grid, web UI, simulator, bench. 17 tests green; not run on real hardware | 2026-10-12 | — | Run on the Pi 5 + real LiDAR |
| Pi ↔ Mega link | 🔄 | Approved design: GPIO UART 250 k, COBS + CRC-16, spec in `protocol/messages.json`. C core + Mega sketch done (12 071 checks green). Next: Rust Pi side | 2026-10-12 | — | Q1: Arduino C++ or Rust on the Mega |
| LiDAR test app | ✅ | Electron + Next.js viewer and pipeline. Known: `next build` fails on missing `onCalibrationProgress` preload type | 2026-04-10 | — | — |
| Motor control firmware | 📋 | Mecanum kinematics, soft-start, collision backup, RF override | — | Drivers, Pi ↔ Mega link | — |

## Docs
| Feature | Status | Description | Date | Depends on | Blocked by |
|---|---|---|---|---|---|
| AI notes + polished docs | ✅ | This structure (CLAUDE.md, INDEX, FEATURES, WORKING_NOTES, FUTURE_IDEAS, docs/) | 2026-10-12 | — | — |
| Consolidation of the 6 cloud sessions | ✅ | Branches merged, docs in English, plans reconciled | 2026-10-12 | — | — |
| Project hub (`hub/`) | ✅ | Local HTML hub over the repo files: tasks, parts, tools, shopping, battery cells, ideas, links, docs/notes | 2026-10-12 | — | — |
