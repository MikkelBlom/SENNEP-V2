# Docs + inventory plan (draft, 2026-10-12)

Status: **agreed 2026-10-12, part A carried out.** Mikkel's choices: all English · split PROJECT_OVERVIEW + archive · current state + short history · Markdown in docs/ · root CLAUDE.md · inventory translated to English · follow the task text (global CLAUDE.md not available in the cloud; remember the repo is public) · contradictions are flagged in WORKING_NOTES, not resolved (full consolidation later; main MCU link is Pi ↔ Mega). `Cut noter.txt` ended up embedded in `docs/history.md`. The ❓ marks below are the original open questions.

## 1. Stocktake

### What exists
| File | Words | What it actually is |
|---|---:|---|
| `ai-instructions/PROJECT_OVERVIEW.md` | 6.5k | Requirements, architecture, receipts, contradictions, history (Gemini/ChatGPT/Robot v58), motor datasheet, wheel generator v1→v3.1, skeleton v0→v3.3, body v0. Mixes current truth, history and stale statements. |
| `ai-instructions/SESSIONS.md` | 2.4k | Session log 2026-10-04 → 10-12. Holds decisions, lessons and open items that exist nowhere else. |
| `ai-instructions/HANDOFF_CHASSIS.md` | 1.1k | One-off prompt for the skeleton chat. Has the best write-up of **print lessons** and **Fusion pitfalls**, plus hard "do not touch" rules. Partly stale. |
| `components/` | — | `inventory.csv` (85 lines) → `INVENTORY.md` + `index.html`, 37 datasheets, ESP-01 guide. In good shape. |
| `print/*/LÆS MIG.md` | 2.4k | Per-print guides. Fine where they are. |
| `docs/skelet_udkast_v1.html`, `docs/skelet/*.png` | — | Skeleton sketch + screenshots. |
| `Cut noter.txt` (repo root) | 68 | Laser-cut test notes from the V1/2025 front panel. Already summarised in PROJECT_OVERVIEW §6. |

### Missing (required by the global rules)
- Project `CLAUDE.md`, `INDEX.md`, `FEATURES.md`, `FUTURE_IDEAS.md`, `WORKING_NOTES.md`, polished `docs/`.
- ⚠️ **The global CLAUDE.md is not in this cloud container** (`~/.claude/CLAUDE.md` does not exist). I only know its rules from the task description. ❓ Paste it, or accept that I follow the file list and formats given in the task.

### Contradictions and stale statements (to be resolved in the new files, not silently)
1. **Size:** §2 says 650 × 400 and §8 asks "500 × 816 or 650 × 400?". Decided 2026-10-05: **800 × 650**. Wheelbase 550 was set by Claude and **never confirmed by Mikkel**.
2. **Wheel:** §2 says "110 mm mecanum"; §7c says Ø130; real value **Ø106.2**, 9 rollers (generator v3.1).
3. **Print limit:** §2 Bambu 256, HANDOFF 250, decided **220** (`MaxPrint`).
4. **Fasteners:** §2 "only M3"; now M4 in motor collars + **printed M8** in frame joints. HANDOFF lists M5 as available; §13 says **no M5**.
5. **Motor controller:** §3 says Pi ↔ **Nano**; the 10-12 task chips say Pi ↔ **Mega** link. Undecided?
6. **Motor drivers:** §4 "Drivlinje: L298N ×2"; superseded by 5× high-power H-bridges (Dec 2025), chip not confirmed.
7. **GPS:** §6 "uafklaret"; §8 "skal med". → must go in.
8. **Waterproofing:** §7b requires it; §8 resolved **not waterproof** (indoor, ventilation kept).
9. **Resolved items still listed as open:** Pi 4/5 (→ Pi 5), motor shaft 8 mm "check", E-stop rating (→ relay), XSHUT vs mux (→ TCA9548A).
10. **Section mix-up:** "Løst af AliExpress-ordren" sits under §6 Cut noter; belongs to §5.
11. **Dead path:** §11 refers to `fusion/wheel_generator.py` (does not exist; replaced by `fusion/SENNEP_WheelGenerator/`).
12. **Material:** `print/housing_v2/LÆS MIG.md` says "≈160 g **PLA**"; everything else is PETG.
13. **Naming:** "pod" is now **"housing"** with Mikkel (10-12), but the generator, Fusion components and older notes still say pod.
14. **Stale facts in SESSIONS:** "not a git repo" (it is now), Fusion MCP setup notes that only apply locally.
15. **Not yet integrated:** "20 Chassis" still uses the housing-v1 bearing keeper (build_chassis ~l.1066); FR/RL mirrored wheels missing; no motor lock; pods/housings solid; shell not split into panels.
16. **LiDAR test app** `next build` fails (`onCalibrationProgress` missing in preload types) — logged once, never tracked.
17. **ESP-01:** "WiFi dropped due to hardware problems" is from a ChatGPT summary Mikkel called wrong.

### Duplicated
Requirements and architecture (PROJECT_OVERVIEW §2–3 vs HANDOFF), print lessons (HANDOFF vs §12 v3 vs print guides), Fusion pitfalls (HANDOFF vs §9/§13 vs SESSIONS), the receipt table (§4 vs `components/`).

### Inventory gaps
- No price / RAM: `raspberry-pi-5`, `lidar-ld19`. Approximate: `motor-rs834`. No count: `petg` ("15 kg" is a May 2025 figure).
- Unconfirmed chips: `h-bridge-43a` (BTS7960?), `mosfet-55v` (IRFZ44N?). Unknown: fan voltage, unbrako assortment sizes, buck-5a chip.
- **Not on the list at all (already owned / mentioned in notes):** old M2–M4 screw kit (M3×12/19 ×20 each, M4×12 ×20, M4×25 ×10, nuts/washers), tap & die set M3–M12, screwdrivers (≤ Ø16 × 150), Bambu X1C, TPU (roller sleeves), soldering iron (inserts), nickel strip 0.15 × 27 mm, calipers, plus whatever is left from V1 and other projects.
- Need vs stock: frame BOM needs M3×12 ×44, M3×19 ×88, M3 nuts ×84, washers ~220, inserts ×48 — far more than the old kit. Worth showing next to the count.

## 2. Target structure

```
CLAUDE.md                      ❓ root (auto-loaded by Claude Code) or ai-instructions/
ai-instructions/
  INDEX.md                     map of every file + "read this first" order
  FEATURES.md                  Feature | Status | Description | Date | Depends on | Blocked by
  FUTURE_IDEAS.md              not-MVP ideas (dock, swappable shell, GPS use, AI camera, …)
  WORKING_NOTES.md             open concerns + decisions with reasoning (incl. rejected options)
  SESSIONS.md                  unchanged log, new entry appended
  archive/
    PROJECT_OVERVIEW_2026-10-12.md   frozen copy, untouched (nothing lost)
    HANDOFF_CHASSIS.md               historical prompt
docs/
  README.md                    polished overview: what SENNEP V2 is, status, photos
  requirements.md              current requirements + architecture (resolved values only)
  wheel.md                     wheel design now + condensed version history
  chassis.md                   housing, ladder frame, body v0, BOM
  motor-and-power.md           motor datasheet, load estimate, battery 3S6P, power tree
  printing-lessons.md          fits, overhang, strength, style (from HANDOFF + §12)
  fusion-scripting.md          generator recipe, MCP usage, pitfalls
  history.md                   V1, Gemini/ChatGPT timeline, Robot v58, laser-cut notes
  DOCS_PLAN.md                 this file (deleted or archived when done)
```
`Cut noter.txt` moves into `docs/history.md` (content kept verbatim) and the file is removed. ❓

**PROJECT_OVERVIEW recommendation:** split. Its content moves into `docs/` and the AI files with the contradictions above resolved, and an untouched copy goes to `archive/` so nothing is lost and a diff is always possible. Keeping it as a living "deep reference" means two places claim to be the truth, which is how the current contradictions happened.

### What goes where
| Source | Destination |
|---|---|
| §1, §7, §7b, §7c, Cut noter | `docs/history.md` |
| §2, §3, §8 (resolved) | `docs/requirements.md`; unresolved → `WORKING_NOTES.md` |
| §4 receipts | already in `components/`; summary in `docs/README.md` |
| §5 contradictions | resolved → `WORKING_NOTES.md` decisions; open → concerns |
| §7d, §10 load calc, battery | `docs/motor-and-power.md` |
| §9, §11–12, Samlering | `docs/wheel.md`; Fusion API notes → `fusion-scripting.md` |
| §13 skeleton/body + SESSIONS 10-12 housing v2 | `docs/chassis.md` |
| HANDOFF print + Fusion sections | `printing-lessons.md`, `fusion-scripting.md`; "do not touch" rules → `CLAUDE.md` |
| SESSIONS lessons / rejected ideas | `WORKING_NOTES.md` (with date + session link) |
| Open "Next:" items | `FEATURES.md` |
| Swappable shell, dock, AI camera emotion, etc. | `FUTURE_IDEAS.md` |

## 3. Inventory count

> **Outcome 2026-10-12:** the CSV was translated to English and got `source` + `condition` columns (validated by the build script, filterable in `index.html`; the empty `order` column was dropped). Mikkel moved the physical count to the upcoming full consolidation session, where every part he has will be gone through. All rows stay `condition = unknown` until then. Gmail needed re-authentication, so the Pi 5 / LiDAR receipts were not searched again.

**CSV changes:** two new columns after `antal`: `kilde` (`SENNEP` = bought for this project / `andet projekt` = already owned) and `stand` (`ok` / `brugt` / `defekt` / `ukendt`). `build_inventory.py` + template get a source filter and show both. Rows without a receipt get `butik` = `eget lager`. ❓ Column name: `kilde` (matches the Danish CSV) or `source` (as written in the task).

**Count method:** one round per category group, in this order: boards → sensors → motors/drivers → power → connectors/switches → cables → hardware/screws → tools → consumables (+ UI/sound/light/passive along the way). Each round:
1. I show what the list says.
2. AskUserQuestion: "all as listed?" / "some differ (write which)" + "leftovers from other projects in this category?" (free text).
3. I update the CSV right away, so a stopped session loses nothing.

I also generate `components/TAELLELISTE.md`, a printable checklist with blank boxes, so you can count at the bench and answer from it.

Gaps (Pi 5 RAM/price, LiDAR price, H-bridge and MOSFET markings) are asked in the round for their category. For chips: read the marking on the chip itself.

At the end: `python build_inventory.py`, check that no datasheet warnings appear, commit, push.
