# Testhjul v3.1 — printguide

Eksporteret 2026-10-11 fra "11 Wheel Lab" i Fusion-dokumentet "SENNEP V2 v27" (samme geometri og parametre som v3; generatoren er uændret). Ét hjul. PETG, 0,16 mm Optimal @BBL X1C.

**Filerne er allerede vendt rigtigt:** printfladen ligger på Z = 0, og delen er centreret. Importér og slice — drej dem ikke, og brug ikke "Auto orient" (den vælger ofte en anden flade).

| Fil | Antal | Materiale | Ligger på pladen med | Højde |
|---|---|---|---|---|
| 1x_RimA_motorside.stl | 1 | PETG | Ydersiden (motorsiden med bajonet-hullet), 3065 mm² | 63,5 mm |
| 1x_RimB_outer.stl | 1 | PETG | Ydersiden (hjulets synlige side), 2915 mm² | 32,0 mm |
| 1x_Axle.stl | 1 | PETG | Hovedet (Ø42), koblingen peger op | 119,3 mm |
| 1x_Spacer.stl | 1 | PETG | Enden med de 3 tapper | 19,7 mm |
| 9x_Roller_core_PETG.stl | 9 | PETG | Enden (stående) | 45,5 mm |
| 9x_Roller_sleeve_TPU.stl | 9 | TPU | Enden (stående), valgfri | 45,5 mm |

## Bambu Studio
- Højreklik på en rulle → **Add instance** ×8 (eller "Fill bed"). Ruller og sleeves på hver sin plade, da de skal have forskelligt filament.
- **Support: fra.** Alle overhæng er ≤ 45° i denne orientering.
- **Brim** på akslen (119 mm høj på Ø42 er ok, men en 5 mm brim er billig forsikring) og på rullerne (kun 80 mm² kontaktflade, 45 mm høje). Rims og afstandsrør behøver ikke brim.
- Rullerne: sæt evt. 3–4 vægge, så pindeboringen bliver massiv.
- Sleeves i TPU: langsom hastighed og ingen brim nødvendig, hvis de printes alene.

## Samling
1. Sæt eventuelt sleeves på kernerne.
2. Sæt rullerne på Rim A's korte pinde.
3. Skru Rim B ned over navrøret. Rillerne drejer den cirka 43°, så pindene rammer rullerne.
4. Afstandsrøret: tapperne i slidserne i Rim A, tryk i bund, drej 60°.
5. Pres akslen ind udefra gennem Rim B, Rim A og afstandsrøret.
6. Skub en M3-møtrik ned i lommen fra akslens motor-ende, og skru pinolskruen et par omgange i møtrikken, så den holder møtrikken på plads uden at stikke ind i hullet. Skub akslen på motorakslen med D-fladen ind mod møtrikken, og spænd så **let**. D-fladen tager drejningsmomentet, så skruen skal kun holde akslen på plads.

## Hvis noget er for stramt eller for løst
Ret det i Fusion under Change Parameters og kør generatoren igen:

| Problem | Parameter |
|---|---|
| Riller | `Wheel_KeyClear`, `Wheel_TubeClear` |
| Sekskant | `Axle_HexClear` |
| Bajonet | `Spacer_Clear`, `Spacer_LockClear` |
| Ruller | `Wheel_PinClear`, `Wheel_EndGap` |

## hjulkasse/ (pod med motor og leje)
Fra "21 Pod Lab" (chassisgeneratoren, skelet v3.1). Samme vendt-rigtigt-princip som hjulet.

| Fil | Ligger på pladen med | Mål (B × D × H) |
|---|---|---|
| 1x_Pod_hjulkasse.stl | Motorenden (røret om gearkassen), 1124 mm² | 89 × 118 × 169 mm |
| 1x_Motor_collar.stl | Fladt (6 mm plade) | 42 × 54 × 6 mm |
| 1x_Bearing_keeper.stl | Fladt, stifterne opad | 52 × 79 × 11 mm |

- Ingen support. Poden er 169 mm høj på en lille fod → **brim 5–8 mm**.
- Poden er massiv i modellen (ikke udhulet endnu); slicerens infill (15 % gyroid) klarer det. 3–4 vægge omkring lejelommen og motorrøret.
- Hardware: 4× M4×12 + skiver (krave → motor), 2× M3×19 + 2 M3-messingindsatser (lejeholder → pod).

## Samling af hjul + hjulkasse (rækkefølgen betyder noget)
Se `hjulkasse/samling_*.png` og i Fusion: "22 Pod Lab Hardware" (alle skruer på plads) og "23 Pod Lab Samletrin" (skjult: hjulet drejet + unbrakonøgler).

Hardware: 2× M3×19 + 2 M3-skiver + 2 M3-indsatser (lejeholder), 4× M4×12 + 4 M4-skiver (motorkrave), 1× M3×12 + 1 M3-møtrik (kobling).

0. **Indsatser:** smelt de 2 M3-indsatser ned i lejelommens bund i poden (inde fra hjulkassen) — inden noget andet.
1. **Hjulet:** ruller i samleringen på Rim A → skru Rim B på → løft ud → afstandsrøret (bajonet) → akslen igennem.
2. **Lejet:** sæt lejeholderen på flangens forside (stifterne i flangehullerne). Skub lejet på akslen fra koblingsenden, flangens bagside mod motoren, til inderringen ligger an mod afstandsrøret. Spænd lejets egne pinolskruer.
3. **Koblingen:** M3-møtrikken ind i lommen fra koblingsenden. Skru M3×12 **helt i bund** (hovedet ligger på koblingen) — ellers rammer hovedet poden, når hjulet skubbes ind.
4. **Lejeholder-skruerne:** stik en M3×19 med skive gennem hvert hul i lejeholderen (gennem stiften og flangen). Spidserne stikker 6 mm ud bag flangen.
5. **Ind i poden:** skub hele hjulet ind fra den åbne yderside. Flangen går ned i lommen, og skruespidserne finder indsatserne.
6. **Spænd lejeholderen gennem hjulets vinduer:** drej hjulet, til du kan se begge skruehoveder gennem to vinduer (det sker i et bånd på ca. 28° hver 60°). Spænd med en 2,5 mm unbrakonøgle/-skruetrækker med mindst ~85 mm skaft (Ø ≤ 6 mm).
7. **Motoren:** skru kraven på motorens front med 4× M4×12 + skiver. Drej hjulet, så koblingsskruen står under det runde hul i toppen, og skru den ~14 omgange ud (til hovedet står ca. 7 mm over koblingen). Skub motoren ind bagfra med tapperne i kanalerne og D-fladen ind mod skruen. Spænd skruen **let** gennem hullet.
