# components/ — alt hardware, der er købt til SENNEP V2

| Fil | Til hvad |
|---|---|
| **`index.html`** | Åbn i en browser: søgning, filtrering pr. kategori, sortering og klik direkte ind i databladene |
| **`INVENTORY.md`** | Samme liste som tabeller pr. kategori (læses i VS Code eller på GitHub) |
| **`inventory.csv`** | **Kilden.** Én linje pr. komponent, `;`-separeret (åbner direkte i dansk Excel) |
| `datasheets/` | PDF-datablade, navngivet efter komponent (fx `vl53l0x.pdf`) |
| `<id>/` | Noter, diagrammer og kode for en enkelt komponent (fx [`esp-01/`](esp-01/README.md)) |
| `build_inventory.py` | Genererer `INVENTORY.md`, `index.html` og kolonnen `dkk` ud fra CSV'en |
| `index.template.html` | Skabelon til `index.html` |

## Opdatering

1. Tilføj eller ret en linje i `inventory.csv`. **Brug aldrig `;` i en note**, for det er feltadskilleren.
2. Læg eventuelle datablade i `datasheets/` og skriv filnavnene (adskilt med mellemrum) i `datablade`.
3. Kør `python build_inventory.py`. Scriptet melder fejl, hvis et datablad mangler eller ikke bruges.

Kolonner: `id` (kort, unik, bruges som mappenavn) · `navn` · `kategori` · `antal` · `linjepris` (samlet pris
for linjen i original valuta) · `valuta` · `dkk` (beregnet, ca.: GBP 8,67 / EUR 7,46) · `butik` · `dato` ·
`ordre` · `chip` (hovedchippen på modulet) · `datablade` · `links` (web eller lokale noter) · `noter`.

## Til AI-agenter

Spørgsmål af typen "har jeg en X?", "hvor mange Y?" eller "hvilken spænding kører Z på?": søg i
`inventory.csv` (grep på navn, chip eller id) og læs den relevante PDF i `datasheets/`. Undersøg også
`<id>/`, hvis mappen findes. Lad være med at gennemgå kvitteringer eller Gmail igen. Hvis noget mangler på
listen, så tilføj det til CSV'en og kør scriptet.

## Kilder og huller

- Kvitteringer (kun lokalt, ikke i det offentlige repo): Ardustore 14-05-2024, Hobby Components 18-04-2025,
  Bot'n Roll 01-05-2025 og AliExpress dec 2025 (18 ordrer i Gmail, varenavnene er afkortede).
- Priserne er ekskl. fragt (39 kr, £6,42, €46,40). Hobby Components-fakturaen er uden moms. Eventuel importmoms
  er ikke med.
- **Uden kvittering:** Raspberry Pi 5, LD19-LiDAR, de 4 motorer (pris ca.) og PETG.
- **Ikke bekræftet:** at AliExpress-H-broerne er BTS7960, og at Bot'n Roll-MOSFET'erne er IRFZ44N.
- Komponenter uden datablad er generiske ting (kabler, stik, kontakter, kits), eller også findes der ikke et
  officielt datablad (E18-D80NK, 3S-BMS, 5 A-buck, 7"-skærm: se `links`).
