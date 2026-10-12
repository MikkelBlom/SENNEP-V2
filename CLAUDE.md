# SENNEP V2 — project rules for AI agents

## What this is
SENNEP V2 is Mikkel's hobby robot: a holonomic (mecanum) mobile robot, about 800 × 650 mm, that drives itself with a 360° LiDAR and can be driven by hand over an RF joystick. It is a rebuild "for fun and learning" of S.E.N.N.E.P. (V1), a high-school (HTX) project in laser-cut acrylic.

Almost everything is 3D-printed in PETG on a Bambu X1C. Wheels, axles, corner housings and frame come from parametric Python generators that run inside Autodesk Fusion (`fusion/`). Compute: Raspberry Pi 5 (high level: LiDAR, camera, screen, most sensors; Python written from scratch, no ROS 2) talks to an Arduino Mega (main link) and a Nano (low level: motors, collision backup, RF override). Power comes from a 3S6P 18650 pack.

Status (Oct 2026): the wheel and corner housing are at the test-print stage, the frame and body are modelled, and there is no robot code yet. The only software is the LiDAR test app (`Lidar Test Enviornment/`).

## Read first
`ai-instructions/INDEX.md` maps every file. Then read `FEATURES.md` (what is in progress) and `WORKING_NOTES.md` (open concerns and decisions). Parts: `components/inventory.csv`. Tasks, shopping, ideas, links and notes: `hub/` (plain CSV/Markdown that Mikkel edits through `hub/index.html`; re-read before editing, and keep the `;` CSV format).

## Never violate
1. **Public repo.** Never commit invoices, order numbers, home address, e-mail addresses, chat exports (`gpttxt.txt`, `botnroll.txt`, root `*.pdf`) or secrets. `components/esp-01/.../secrets.h` stays local; only the `.example` is committed.
2. **Ask before guessing a physical dimension** (bearing flange, motor holes, battery pack, screw lengths). Measured or datasheet values only, and say which.
3. **The motor must not carry the robot.** The motor shaft takes < 5 kgf radial load; the KFL003 pillow blocks carry the wheel. Motor, bearing and wheel must stay on one axis.
4. **Printability first.** Nothing facing down at less than 45°, no flat ceilings in holes, round holes (no teardrops), each joint drawn as one continuous profile (no "patch on patch"). Check which face lies on the plate before calling anything printable. Details: `docs/printing-lessons.md`.
5. **Only hardware Mikkel has.** Check `components/inventory.csv` before specifying screws, inserts or parts. No glue and no new purchases without asking.
6. **Generators own their geometry.** Change the Python generator, not the Fusion model by hand. Never edit `Wheel_*`, `Axle_*`, `Spacer_*` or `Motor_*` parameters from chassis work. Work only in the Fusion document "SENNEP V2"; "Robot v58/v59" are read-only references.
7. **Verify with numbers** (collision volume, clearances, overhang faces) before presenting geometry. One build per Fusion MCP call.
8. **`components/inventory.csv` is the single source for parts.** Edit it and run `python components/build_inventory.py`. Never hand-edit `INVENTORY.md` or `index.html`.
9. **Archived files are frozen.** Do not edit `ai-instructions/archive/`.
10. **Log every session** in `ai-instructions/SESSIONS.md` and keep `FEATURES.md` / `WORKING_NOTES.md` current.

## Working style
Mikkel wants direct answers, honest pushback with alternatives, and a plan agreed via questions before big builds ("status → research → plan → ask → build"). He calls the corner module "housing" (the code still says "pod"). New docs are in English; older notes and print guides are in Danish.
