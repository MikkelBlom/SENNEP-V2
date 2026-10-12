# Testhjul v3 — printguide

Genereret 2026-10-05 af `fusion/SENNEP_WheelGenerator` (Fusion-dokument "SENNEP V2 v18"). Ét hjul. PETG, 0,16 mm Optimal @BBL X1C.

**Print `hurtigtest/` først** (~1 time): den tester alle pasninger, inden du bruger tid på et helt hjul.

| Fil | Antal | Materiale | Orientering |
|---|---|---|---|
| 1x_RimA_motorside.stl | 1 | PETG | Yderside (med afstandsrør-hullet) ned på pladen |
| 1x_RimB_outer.stl | 1 | PETG | Yderside ned på pladen |
| 9x_Roller_core_PETG.stl | 9 | PETG | Stående på enden |
| 9x_Roller_sleeve_TPU.stl | 9 | TPU | Stående på enden (valgfri, hjulet kører også uden) |
| 1x_Axle.stl | 1 | PETG | Stående på hovedet (Ø42) |
| 1x_Spacer.stl | 1 | PETG | Stående med de 3 tapper nedad |

STL-filerne ligger i monteringsposition, så brug "Lay on face" i Bambu Studio.

## hurtigtest/
| Fil | Tester |
|---|---|
| Test_RimA_spacer_socket + Test_Spacer | Bajonet: går tapperne i, kan den dreje 60° og låse? Sidder den stramt? |
| Test_RimA_tube_end + Test_RimB_hub | De skrå riller: skal kræve lidt kraft, men ingen slør |
| Test_Axle_hex | Sekskanten i alle tre testdele: skal presses i, ikke falde i |

## Ændret siden v2.7
- **Ruller Ø21,5 i stedet for Ø20.** Fælgkanten er sænket, så kernen alene stikker 2,0 mm ud. Med sleeve er det 3,2 mm. Før var det cirka 0,2 mm.
- **Korte pinde i begge ender** (Rim A 8 mm, Rim B 6 mm og let tilspidset). De lange pinde er væk.
- **Trykring:** rulleenden løber mod en flad ring, ikke mod pindens rod. Den kegleformede rod kilede sig fast i boringens kant.
- **Bajonet:** trekantede tapper. Én 45°-linje går fra rillen helt ned til sekskanten, og afstandsrørets tapper og skrå ende ligger på samme linje med 0,2 mm frigang. Der er ingen flade lofter, så delene kan printes uden support.
- **Strammere pasninger:**

  | Pasning | Før | Nu |
  |---|---|---|
  | Riller | 0,3 pr. side | 0,1 pr. side |
  | Navrør | 0,4 | 0,15 |
  | Sekskant (total) | 0,5 | 0,2 |

  Lejesektionen på Ø16,9 er uændret.
- Pinolskruen sidder nu i en **indfanget M3-møtrik** i en lomme, der er åben mod akslens ende (ingen printpause). Gevind i plast flækkede akslen langs laget. Der er ikke plads til en M4-møtrik i væggen.

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
