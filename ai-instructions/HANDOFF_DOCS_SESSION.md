# Handoff: docs + parts-list session (2026-10-12) → consolidation session

This is one of 6 cloud sessions that worked independently. This file records everything this session did, asked and learned, so the consolidation session can merge it with the others. Mikkel has **not read** this session's output.

## Where the work is
- Repo `MikkelBlom/SENNEP-V2`, branch **`claude/trusting-einstein-vlvcvm`** (pushed, **no PR, not merged into `main`**). Base: `547c8db` (main at the time).
- Commits: `0605b7f` docs restructure · `b3680e3` parts list translated + new columns · (third) session log + count deferred · (fourth) this handoff.
- To get it: `git fetch origin claude/trusting-einstein-vlvcvm` and merge or diff it against the other sessions' branches.

## Original task (from Mikkel)
A) Create the project docs required by Mikkel's global CLAUDE.md: project `CLAUDE.md` (~300–500 words: what the project is + what must never be violated), `INDEX.md`, `FEATURES.md` (Feature | Status | Description | Date | Depends on | Blocked by), `FUTURE_IDEAS.md`, `WORKING_NOTES.md` (concerns + decisions with reasoning) and polished docs in `docs/`. Sort existing knowledge without losing anything.
B) Parts list: interactive count with Mikkel per category, add leftovers from other projects with a source column, fill the known gaps (Pi 5 / LiDAR price + RAM, BTS7960 / IRFZ44N unconfirmed), rebuild with `build_inventory.py`.
Required order: stocktake → plan → AskUserQuestion with Mikkel → build.

## Mikkel's answers (verbatim where it matters)
| Question | Answer |
|---|---|
| Language of new files | **All English** |
| PROJECT_OVERVIEW.md | **Split + archive** (content into docs/ + AI notes, untouched copy archived) |
| Detail level of docs | **Current state + short history** |
| Docs format | **Markdown in docs/** |
| Global CLAUDE.md (missing in cloud containers) | "Follow the task text - but remember it is a public repo." |
| Location of project CLAUDE.md | **Repo root** |
| Inventory language | **Translate it to English** |
| Pi ↔ Nano vs Pi ↔ Mega | "Nevermind this. We will run a full consolidation later. For now you just update what is already there and relevant. But technically it is both - but main is pi to mega." |
| Boards / comms / leftovers count | "Something is different. Please note that with the consolidation session we will go through every part i have available to me right now." → **the physical count was moved to the consolidation session** |
| Pi 5 RAM | "I believe 4 or 8. Not sure - it should be on the parts list, no?" (It is not: there is no receipt for the Pi 5.) |

Interpretation used: contradictions are **flagged, not resolved**, and the content was moved faithfully.

## What was produced
- `CLAUDE.md` (root): project summary + 10 never-violate rules (public repo / no private files, ask before guessing dimensions, the motor must not carry load, printability rules, only owned hardware, generators own geometry, verify with numbers, CSV is the single source of parts, archive is frozen, log every session).
- `ai-instructions/INDEX.md`, `FEATURES.md`, `WORKING_NOTES.md` (open concerns C1–C18 + decision table with reasoning/rejected options), `FUTURE_IDEAS.md`.
- `docs/README.md`, `requirements.md`, `wheel.md`, `chassis.md`, `motor-and-power.md`, `printing-lessons.md`, `fusion-scripting.md`, `history.md`, `DOCS_PLAN.md` (stocktake + plan + outcome).
- Archived unchanged (with a frozen banner): `ai-instructions/archive/PROJECT_OVERVIEW_2026-10-12.md`, `ai-instructions/archive/HANDOFF_CHASSIS.md`.
- `Cut noter.txt` removed from the root. Its text is embedded verbatim in `docs/history.md`.
- `components/`: `inventory.csv`, `build_inventory.py`, `index.template.html` and `README.md` translated to English. New columns are `source` (`SENNEP V2` / `SENNEP V1` / `other project`) and `condition` (`ok` / `used` / `broken` / `unknown`), validated by the build script and filterable in `index.html`. The empty `order` column was dropped (privacy). New category `Hardware` for screws. All 85 rows kept, with Danish ids renamed to English (e.g. `rele-40a` → `relay-40a`, `unbrako-sortiment` → `hex-screw-assortment`; the full mapping is in `git diff 547c8db -- components/inventory.csv`). Every row has `condition = unknown`; the motors have `source = SENNEP V1`, everything else `SENNEP V2`.
- `.gitignore`: added `secrets.h` (ESP-01 Wi-Fi credentials were not ignored before).
- `fusion/SENNEP_ChassisGenerator/SENNEP_ChassisGenerator.py`: docstring path updated to the archived handoff + `docs/printing-lessons.md` (comment only).
- `SESSIONS.md`: entry "2026-10-12 (cont. 4)".

## Stocktake findings (full list in `docs/DOCS_PLAN.md` §1 and `WORKING_NOTES.md`)
- Contradictions: size (650 × 400 vs 800 × 650 decided), wheel Ø (110/130 vs 106.2 real), print limit (256/250 vs 220), fasteners ("only M3" vs M4 + printed M8; HANDOFF says M5 available, the overview says none), Pi ↔ Nano vs Mega, L298N vs 43 A H-bridges, GPS "unresolved" vs "include", waterproof vs not, resolved items listed as open, misplaced section in the overview, dead path `fusion/wheel_generator.py`, housing guide says PLA, pod vs housing naming, stale "not a git repo".
- Unconfirmed: wheelbase 550 (set by Claude), H-bridge chip, Bot'n Roll MOSFETs, wheel fits (no v3 quick-test result recorded), self-aligning KFL003 under load, no motor lock.
- Parts known from notes but **not in the inventory** (already owned): old M2–M4 screw kit (M3 × 12 / × 19 ×20 each, M4 × 12 ×20, M4 × 25 ×10, nuts/washers), tap & die set M3–M12, screwdrivers (≤ Ø16 × 150), Bambu X1C, TPU filament, soldering iron, nickel strip 0.15 × 27 mm, calipers, V1 leftovers. Also: TVS diodes are planned but not on the list.
- Open inventory gaps: Pi 5 RAM (4 or 8 GB per Mikkel) and price, LiDAR price, motors (approximate price), PETG remaining, fan voltage, hex-screw assortment sizes, buck-5a chip, whether the 2024 Ardustore order was V1 or V2 (all marked `SENNEP V2` for now).

## Not done / for the consolidation session
1. **Physical count of every part** (qty, source, condition, leftovers from other projects) → then `python components/build_inventory.py`.
2. Resolve WORKING_NOTES C1–C18 with Mikkel.
3. Pi 5 / LiDAR receipts: the Gmail connector needed re-authentication in this session, so they were not searched.
4. Merge conflicts to expect: other sessions may also have edited `ai-instructions/SESSIONS.md`, `PROJECT_OVERVIEW.md` (moved to `archive/` here: port their additions into `docs/` or the AI notes instead) and `components/inventory.csv` (renamed columns and ids here).
5. Decide whether `docs/DOCS_PLAN.md` and this handoff are archived after the merge.
