# Handoff → next session (written 2026-10-12, end of the consolidation cloud session)

Read `/CLAUDE.md` → `INDEX.md` → this file. Everything below is on `main` (pushed).

## How Mikkel works (learned the hard way this session)
- Windows PC, PowerShell, repo at `C:\Users\mikke\Desktop\Projects\SENNEP V2`. Cloud sessions cannot reach his USB.
- He forgets to pull → `.\sync "message"` (repo root, `sync.cmd`) = pull + add all + commit + push; stops on a conflict.
- **Never assume what he did — look at the data first** (he was rightly angry when I blamed an "old exe").
- **Show, don't describe:** coordinates and "far corner" mean nothing to him. Use screenshots or point him to a
  time in the LiDAR playback ("open room_3, jump to 1:21"). Ask questions **in plain text**, numbered with a
  recommendation each: never AskUserQuestion or wrap-up question boxes (they hide the text in the desktop app).
- Replies in Danish. He saves money (exchange trip to Korea): price things before suggesting a purchase.

## Update 2026-10-18 (status grilling)
- Wheels v3.1 + housings v2: all 4 printed and assembled with bearings and motors. Next mechanically: a frame they
  bolt onto. Mikkel dislikes the printed rails and M8 bolts → explore 2040/2020 aluminium profiles (t27, C25). Out of
  black Bambu filament; 10 rolls of black PLA Basic in his cart (~110 €).
- Battery: he mailed Fyns Batteri Service for a price; otherwise a spot welder (~345 kr, hub Shopping s19).
- Breadboard plan was never reviewed by him (status corrected). No build guide: step by step from the Circuits drawing.
- Pi software = Faber. LiDAR view and AI camera (IMX500, already in inventory) become separate apps (t30, t31).
  Faber already has `examples/apps/sennep` (LiDAR top view + safe driving); it should move to this repo.
- Motor stall: ~8 A, computed from the measured 1.7–1.8 Ω winding (Circuits corrected; the 30 A main fuse still
  blows with all four held still).
- Short, direct answers; he answers questions, he does not read long output.

## State per workstream
| Area | State | Where |
|---|---|---|
| Docs / hub | All docs English. Local hub `hub/index.html` (Chrome folder access) edits CSV/MD in the repo: tasks, parts (`counted`), tools, shopping, battery cells (interactive top/bottom diagram + strips), ideas, links, notes | `hub/README.md` |
| Battery 3S6P | Plan v2 **approved**. All 20 cells measured 3.534–3.536 V, wraps OK, **no moves needed**. Orientation fixed: black mark front-left; rows front→back = G1/G2/G3 (slots 1–6/7–12/13–18), 19–20 spares. Holder pitch **20.2 mm** (measured) vs strip **19 mm** (2P ladder = strips A/D across two rows, 1P = B/C; weld on the rails, don't cut). BMS = HW-288 (listing). Motor winding 1.7–1.8 Ω → stall ~7–8 A/motor. **Next: Mikkel calls Fyns Batteri Service** (ask: pure nickel, 19 vs 20.2 mm pitch, double layer on B and C, tabs for 10 AWG). Then Claude writes the illustrated assembly guide. Buy an inline blade fuse holder (30 A main) | `docs/electronics/BATTERY_PLAN.md`, hub Battery cells |
| Breadboard M1 | Plan v0.2 **approved** (Pi 5 + Mega + 4 motors + LiDAR; Wago power, breadboard signals). Pin map = `firmware/sennep_mega/config.h` (+ A4 bus sense, 29 arm button reserved). **Next: wiring diagrams + step-by-step build guide** | `docs/electronics/BREADBOARD_PLAN.md` |
| Pi ↔ Mega link | Design approved: GPIO UART 250 k via BSS138, COBS + CRC-16, `protocol/messages.json`. **Q1 decided: Arduino C++ + IDE on the Mega**, Rust on the Pi. C core + sketch done (12 071 checks). **Next: Rust Pi side** (replace `tools/sennep_link.py`), Mega drivers (VL53L0X, HC-SR04, NRF24) | `ai-instructions/PLAN_PI_MEGA_LINK.md` |
| PCB | Blocked until the breadboard is built and tested | — |
| LiDAR (`lidar/`, Rust) | **Runs on the real LiDAR on his PC.** Walls as stable lines, objects as outlines, moving-thing tracking (speed, trail, 1 s prediction, lost/re-acquire), auto-recording + Keep/Mark, **playback in the page** (Recordings list, jump to mm:ss, speed, marks, back to live). 36 tests incl. his 3 recordings. **Runs on the Pi 5 since 2026-10-17** (service `sennep-lidar`, http://sennep.local:8080 or http://192.168.0.106:8080). **Next:** kiosk on the 7" screen, zone report over the link, gyro, more recordings | `lidar/README.md`, `docs/lidar/PLAN.md` §11 |

## LiDAR specifics for whoever continues
- Windows build (no Rust on his PC): `rustup target add x86_64-pc-windows-gnu`, `apt-get install gcc-mingw-w64-x86-64`,
  `CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc cargo build --release --target x86_64-pc-windows-gnu --bin sennep-lidar`,
  send the `.exe` with SendUserFile. **Never commit the exe** (`*.exe` is git-ignored).
- He runs from the repo root: `.\sennep-lidar.exe --serial COM5 --no-body --http 127.0.0.1:8090` (COM5 on his PC).
  It records into `lidar/recordings/auto/` (ignored); "Keep recording…" moves it to `lidar/recordings/<name>.ldr` + `.txt`.
- Recordings (each with a `.txt` of what happens when): `room_walkthrough` (walk in/out), `room_2` (stands still,
  3 sprints ≈ 3.5 m/s / 12–13 km/h, bends down), `room_3` (against the wall at 1:21, hands over the LiDAR 2:12–2:17,
  his arm at 2:43, glass kettle). LiDAR at chest height on a table in his living room; he was alone.
- Lessons: glass (his kettle) flickers → clutter learning; standing against a wall = invisible (10 cm solid radius);
  bending down = below the scan plane; arms split off → satellite rule; jumps in playback need `Pipeline::reset_all`.
- Ideas Mikkel wants later (hub Ideas): collision prediction vs the robot's path, AI-camera + LiDAR person following
  (360° re-find), turning towards movement. "Movement is enough" — no person classification for now.

## Open questions / loose ends
- **Delete the 6 old `claude/*` branches** on GitHub (the cloud proxy refused deletes; all merged into `main`).
- Physical parts count (hub Parts/Tools, `counted` column); Pi 5 RAM: 4 GB (seen on the Pi); Pi 5 / LiDAR prices (Gmail).
- WORKING_NOTES C1–C23 (C2 and C9 resolved). Arm button vs Pi-only ARM (C21). E-stop second NC contact (C20).
- Mechanics (local Fusion sessions): wheel v3.1, assembly ring, housing v2, frame joint test still to print/test.
