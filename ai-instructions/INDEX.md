# Index

Read in this order: `/CLAUDE.md` → this file → `FEATURES.md` → `WORKING_NOTES.md` → the doc for your area.

## AI notes (`ai-instructions/`)
| File | Purpose | Update when |
|---|---|---|
| `INDEX.md` | This map | A file is added, moved or removed |
| `FEATURES.md` | Every feature/workstream with status and dependencies | Status changes |
| `WORKING_NOTES.md` | Open concerns + decisions with reasoning (incl. rejected options) | A decision is made or a concern found |
| `FUTURE_IDEAS.md` | Ideas outside the current scope | An idea comes up |
| `SESSIONS.md` | Log of every session (Danish before 2026-10-12) | End of every session |
| `archive/PROJECT_OVERVIEW_2026-10-12.md` | Frozen deep reference (Danish): every detail up to 2026-10-12 | Never |
| `archive/HANDOFF_CHASSIS.md` | Frozen prompt that started the skeleton chat | Never |

## Polished docs (`docs/`)
`README.md` (overview and status) · `requirements.md` · `wheel.md` · `chassis.md` · `motor-and-power.md` · `printing-lessons.md` · `fusion-scripting.md` · `history.md`. Also: `skelet_udkast_v1.html` (first skeleton sketch), `skelet/*.png` (Fusion screenshots), `DOCS_PLAN.md` (plan for this restructure).

## Parts (`components/`)
`inventory.csv` is the source of truth → `python components/build_inventory.py` → `INVENTORY.md` + `index.html`. Datasheets are in `datasheets/`, and per-part notes in `<id>/` (e.g. `esp-01/`). See `components/README.md`.

## Code and models
| Path | What |
|---|---|
| `fusion/SENNEP_WheelGenerator/` | Wheel generator + assembly ring |
| `fusion/SENNEP_ChassisGenerator/` | Housing, frame, battery, body, hardware, test prints |
| `print/<name>/` | STL sets, each with `LÆS MIG.md` (Danish print guide). Current: `testhjul_v3.1/`, `housing_v2/`, `skelet_test/`. Older: `testhjul_v2.7/`, `testhjul_v3/`. |
| `Lidar Test Enviornment/` | Electron + Next.js LiDAR viewer and processing pipeline (old Gemini-era app; inspiration for the new stack only) with recordings from 2026-04-10 |

## Local only (not in the public repo)
Invoices (`/*.pdf`), Gemini chat PDFs, `gpttxt.txt`, `botnroll.txt`, `secrets.h`.
