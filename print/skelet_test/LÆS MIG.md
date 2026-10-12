# Skelet-testprint: én bjælkesamling + printet M8

Genereret 2026-10-06 af `fusion/SENNEP_ChassisGenerator` (`build_test_print()`), Fusion-komponent "93 Test Print". PETG, 0,16 mm Optimal, X1C.

Formålet er at teste tre ting, før hele rammen printes:
1. Om V-nøglen (spids ryg i rille, 0,1 mm luft) gør samlingen helt fri for slør.
2. Om du kan skære M8×1,25-gevind på den printede bolt (snitbakke) og i møtrikken (tap).
3. Om bolten kan spændes, uden at noget flækker.

| Fil | Antal | Orientering |
|---|---|---|
| 1x_Test_rail_stykke.stl | 1 | Toppen (den lukkede side) ned på pladen |
| 1x_Test_tvaerbjaelke_ende.stl | 1 | Toppen ned på pladen. Ryggen på endepladen peger vandret. |
| 1x_P-M8x26_bolt.stl | 1 (gerne 2) | **Liggende på en af de to flade sider.** Så løber lagene på langs af bolten. |
| 1x_P-M8_moetrik.stl | 1 (gerne 2) | Fladt, hullet lodret |
| 2x_P-M8_skive.stl | 2 | Fladt |

STL-filerne ligger i monteringsposition, så brug "Lay on face" i Bambu Studio. Brug mindst 4 vægge og 40 % fyld på bolt og møtrik.

## Gevind
- **Bolt:** stangen er Ø7,85 med to flader (3,4 mm fra midten). Skær M8×1,25 med snitbakken. Brug lidt olie eller sæbevand, og kør en halv omgang tilbage for hver omgang frem, så spånerne brækker af. Fladerne giver huller i gevindet. Det er meningen.
- **Møtrik:** hullet er Ø7,0. Printede huller bliver ca. 0,2 mm mindre, så det ender omkring tapborets 6,8. Skær med M8×1,25-tappen.
- Skiverne har et Ø8,8-hul, ligesom alle frigangshuller i rammen.

## Samling
1. Skub tværbjælkens ryg ned i rillen i bjælkestykket. Rillen er åben i bunden, så den glider ind nedefra.
2. Skive, bolt gennem endepladen og bjælkens væg, skive og møtrik på indersiden. Spænd med en nøgle (13 mm), men ikke hårdt.

## Hvis noget er for stramt eller for løst
| Problem | Parameter / hvor |
|---|---|
| V-nøglen | `Frame_KeyClear` (0,1) |
| Bolten er for tyk til snitbakken | `PM8['blank']` i generatoren (7,85) |
| Møtrikhullet er for småt eller stort | `PM8['nut_hole']` (7,0) |
| Frigangshullet | `PM8['clear']` (8,8) |
