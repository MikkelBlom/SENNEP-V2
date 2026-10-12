# components/ — every part for SENNEP V2

| File | Purpose |
|---|---|
| **`index.html`** | Open in a browser: search, filter by category and source, sort, click straight into datasheets |
| **`INVENTORY.md`** | The same list as tables per category (VS Code or GitHub) |
| **`inventory.csv`** | **The source.** One line per part, `;`-separated (opens directly in Excel with a `;` list separator) |
| `datasheets/` | PDF datasheets, named after the part (e.g. `vl53l0x.pdf`) |
| `<id>/` | Notes, diagrams and code for a single part (e.g. [`esp-01/`](esp-01/README.md)) |
| `build_inventory.py` | Generates `INVENTORY.md`, `index.html` and the `dkk` column from the CSV |
| `index.template.html` | Template for `index.html` |

## Updating

1. Add or edit a line in `inventory.csv`. **Never use `;` in a note**, because it is the field separator.
2. Put any datasheets in `datasheets/` and list the file names (space-separated) in `datasheets`.
3. Run `python build_inventory.py`. It fails on an unknown `source`/`condition` and warns about missing or unused datasheets.

### Columns
| Column | Meaning |
|---|---|
| `id` | Short, unique, also used as a folder name |
| `name`, `category`, `qty` | What it is, and how many Mikkel has **now** (counted, not bought) |
| `source` | `SENNEP V2` (bought for this robot) · `SENNEP V1` (from the school robot) · `other project` (already owned) |
| `condition` | `ok` · `used` (works, but used/soldered) · `broken` · `unknown` (not checked yet) |
| `line_price`, `currency` | Total price for the line in the original currency |
| `dkk` | Computed, approximate (GBP 8.67 / EUR 7.46) |
| `store`, `date` | Where and when it was bought (`own stock` for leftovers) |
| `chip` | The main chip on the module |
| `datasheets`, `links` | PDF names in `datasheets/`, web links or local notes |
| `notes` | Anything else |

Order numbers are deliberately not stored (public repo).

## For AI agents

For questions like "do I have an X?", "how many Y?" or "what voltage does Z run on?": search `inventory.csv` (grep on name, chip or id) and read the relevant PDF in `datasheets/`. Also look in `<id>/` if it exists. Do not go through receipts or Gmail again. If something is missing from the list, add it to the CSV and run the script.

## Sources and gaps

- Receipts (local only, not in the public repo): Ardustore 2024-05-14, Hobby Components 2025-04-18, Bot'n Roll 2025-05-01 and AliExpress Dec 2025 (18 orders in Gmail; item names are truncated).
- Prices exclude shipping (39 kr, £6.42, €46.40). The Hobby Components invoice is without VAT. Any import VAT is not included.
- **No receipt:** Raspberry Pi 5, LD19 LiDAR, the 4 motors (approximate price) and PETG.
- **Not confirmed:** that the AliExpress H-bridges are BTS7960, and that the Bot'n Roll MOSFETs are IRFZ44N.
- Parts without a datasheet are generic (cables, connectors, switches, kits), or no official datasheet exists (E18-D80NK, 3S BMS, 5 A buck, 7" screen: see `links`).
