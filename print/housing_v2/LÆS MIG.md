# Housing v2 (hjulhjørne) — printguide

Fra "21 Pod Lab" (chassisgeneratoren) 2026-10-12. Filerne er vendt rigtigt (printfladen på Z = 0). Ingen support.

| Fil | Ligger på pladen med | Mål |
|---|---|---|
| 1x_Housing.stl | Motorenden | 89 × 118 × 169 mm — brim 5–8 mm |
| 1x_Housing_hul_variant.stl | Motorenden | Samme, men lukket kammer over lejet. **Bruger ~5 g MERE filament** (kammerets vægge printes massivt) — kun til sammenligning |
| 1x_Bearing_keeper.stl | Forsiden (hjulsiden), stifterne opad | 52 × 79 × 11 mm |
| 1x_Motor_collar.stl | Fladt | 54 × 42 × 6 mm |

Anbefalet: solid housing, 2 vægge, 10 % gyroid ≈ 160 g PLA.

## Ændret fra v1
- **Lejeholderens skruer kommer bagfra:** hovederne sidder i en forsænkning i lommebunden (6 mm plast under), indsatserne sidder nu i **lejeholderen** (smeltes ind fra hjulsiden). En Ø10-kanal går lige ud gennem bagenden til hver skrue → spænd med en lang 2,5 mm unbrako-skruetrækker (skaft ≤ Ø7,5 de sidste 4 mm, ellers ≤ Ø9) bagfra. Kanalerne går uden om motoren, så det kan gøres med motoren monteret.
- **Koblingens pinolskrue nås nedefra** gennem et Ø12-hul i bunden (hullet i toppen er væk). Drej hjulet, så skruen peger nedad.
- **Motoren sidder strammere:** 0,1 mm frigang pr. side (før 0,2).
- **Kravens tapper vender op/ned** (før til siderne, hvor de lå i vejen for skruekanalerne) → kraven skal printes igen.

## Samling
Hardware: 2× M3×19 + 2 M3-skiver + 2 M3-indsatser, 4× M4×12 + 4 M4-skiver, 1× M3×12 + 1 M3-møtrik.
1. Smelt de 2 indsatser ned i lejeholderens forside (den flade side uden stifter).
2. Hjulet samles (samleringen) → afstandsrør → aksel.
3. Lejeholderen på flangens forside (stifterne i hullerne), lejet på akslen, spænd lejets pinolskruer.
4. Koblingen: M3-møtrikken i lommen, M3×12 skrues **helt i bund** (ellers rammer hovedet housingen på vej ind).
5. Skub hjulet ind i housingen fra den åbne yderside, flangen ned i lommen.
6. Bagfra: stik en M3×19 med skive ned i hver kanal og spænd den i lejeholderens indsats.
7. Kraven på motoren (4× M4×12 + skiver). Drej hjulet, så koblingsskruen peger nedad, skru den ~14 omgange ud gennem hullet i bunden, skub motoren ind bagfra (tapperne i kanalerne, D-fladen mod skruen) og spænd skruen let.

Billeder: `1_bund_og_bagende.png`, `2_bagende_vaerktoejskanaler.png`, `3_gennemsigtig_med_skruer.png`. I Fusion: "22 Pod Lab Hardware" og "23 Pod Lab Skruetraekkere bagfra" (skjult).
