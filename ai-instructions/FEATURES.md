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
| Parts list (`components/`) | 🔄 | Inventory + datasheets; count incl. parts from other projects | 2026-10-12 | — | — |
| 3S6P battery series connections | 📋 | Connect the built groups using materials already owned (own workstream) | 2026-10-12 | — | Mikkel wary of soldering/welding cells |
| Breadboard circuits | 📋 | Power tree, drivers and sensors on breadboard (own workstream) | 2026-10-12 | Parts count | — |
| PCB | 📋 | Only after the breadboard works | 2026-10-12 | Breadboard circuits | — |
| Motor drivers | 📋 | Replace L298N with the 43 A H-bridges | 2025-12 | Chip confirmed | — |
| ESP-01 guide | ✅ | Pinout, wiring SVG, passthrough + web-server sketches (compiled, not tested on hardware) | 2026-10-07 | — | — |

## Software
| Feature | Status | Description | Date | Depends on | Blocked by |
|---|---|---|---|---|---|
| LiDAR stack (new) | 📋 | Built from scratch; the old Gemini app is inspiration only (own workstream) | 2026-10-12 | Pi 5 back from parents | — |
| Pi ↔ Mega link | 📋 | Main Pi–MCU link (Nano also used); protocol to be designed (own workstream) | 2026-10-12 | — | — |
| LiDAR test app | ✅ | Electron + Next.js viewer and pipeline. Known: `next build` fails on missing `onCalibrationProgress` preload type | 2026-04-10 | — | — |
| Motor control firmware | 📋 | Mecanum kinematics, soft-start, collision backup, RF override | — | Drivers, Pi ↔ Mega link | — |

## Docs
| Feature | Status | Description | Date | Depends on | Blocked by |
|---|---|---|---|---|---|
| AI notes + polished docs | ✅ | This structure (CLAUDE.md, INDEX, FEATURES, WORKING_NOTES, FUTURE_IDEAS, docs/) | 2026-10-12 | — | — |
| Full consolidation | 📋 | Resolve the remaining contradictions (see WORKING_NOTES) | 2026-10-12 | — | Mikkel |
