# Testhjul v2.7 — printguide

Genereret 2026-10-05 af `fusion/SENNEP_WheelGenerator` (Fusion-dokument "SENNEP V2 v13"). Ét hjul.

| Fil | Antal | Materiale | Orientering |
|---|---|---|---|
| 1x_RimA_motorside.stl | 1 | PETG | Liggende på den flade yderside — navrøret med de skrå riller peger op |
| 1x_RimB_outer.stl | 1 | PETG | Liggende på den flade yderside |
| 9x_Roller_core_PETG.stl | 9 | PETG | Stående på den flade ende |
| 9x_Roller_sleeve_TPU.stl | 9 | TPU | Stående på enden |
| 1x_Axle.stl | 1 | PETG | Stående på hovedet (Ø42) |
| 1x_Spacer.stl | 1 | PETG | Stående på enden med de 3 tapper nedad |

Tip: STL-filerne er eksporteret i hjulets monteringsposition — brug "Lay on face" i Bambu Studio.

## Samling
1. Pres TPU-sleeves på rullekernerne.
2. Sæt rullerne på Rim A's lange pinde (lige ind).
3. Skru Rim B ned over navrøret — de skrå riller fører den ~43° rundt, så de korte pinde går lige ind i rullerne.
4. Sæt afstandsrøret i Rim A (motorsiden): tapper i slidserne, drej 60° til det låser.
5. Stik akslen ind udefra gennem Rim B, Rim A og afstandsrøret. Sekskanten låser det hele.
6. M3-indsats i akslens tværhul + pinolskrue mod motorakslens flade side.

## Det skal testes
- Rullerne: drejer de frit på pindene (frigang 0,3 mm radialt på Rim A, 0,6 mm på Rim B's korte pinde)? Står de stabilt på enden (flad ende 40 mm²)?
- Riller/spor: glider Rim B jævnt på (frigang 0,3 mm pr. side)? → `Wheel_KeyClear`
- Bajonetten: går tapperne i og låser? → `Spacer_*`
- Sekskanten: aksel i fælge/afstandsrør (frigang 0,5 mm total) → `Axle_HexClear`
- Motoraksel i akslens D-hul (Ø8,4, flade 0,5 mm) → `Motor_ShaftClear`, `Motor_ShaftFlat`
- Lejesektionen Ø16,9 i pillow block-lejet (lejets pinolskruer spænder den) → `Axle_BearingDia`

Noter hvad der er for stramt/løst — det er alle parametre i Fusion (Change Parameters).

Tip: slå "Elephant foot compensation" til i Bambu Studio (0,15 mm), så huller ved printpladen ikke bliver for snævre.
