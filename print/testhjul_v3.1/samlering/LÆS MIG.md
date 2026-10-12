# Samlering (assembly ring) til hjulet

Holder rullerne på Rim A's korte pinde, mens Rim B skrues på. Genbrugelig: hjulet løftes lige op af ringen bagefter.

| Fil | Brug når |
|---|---|
| Samlering_uden_sleeves.stl | Rullerne er bare PETG-kerner |
| Samlering_med_TPU_sleeves.stl | Der sidder TPU-sleeves på kernerne |

Print: PETG (materiale i Fusion: PET Plastic, ca. 71 g massiv) eller PLA, bunden ned (filen er vendt rigtigt), **ingen support**, 112 × 112 × 31,5 mm. Alle vugger vender opad.

## Sådan
1. Læg ringen på bordet og Rim A i den med ydersiden nedad (motorsiden, den med bajonet-hullet). Rim A centrerer sig i ringen (0,4 mm luft).
2. Skub rullerne ned på pindene én ad gangen. Hver rulle lægger sig i sin vugge (0,3 mm luft) og kan ikke vælte af.
3. Skru Rim B ned over navrøret som normalt. Ringen når kun op til 0,5 mm under samlingen og ligger uden for fælgenes radius, så Rim B går fri hele vejen.
4. Løft det lukkede hjul lige op af ringen.

Verificeret i Fusion: ingen berøring med Rim A eller Rim B, ingen berøring mens hjulet løftes 0–20 mm op, og en rulle, der skubbes 0,4 mm udad, rammer vuggen.
Kilde: `fusion/SENNEP_WheelGenerator/assembly_ring.py` (`build_ring()`), komponent "11 Wheel Lab Assembly Ring (samlering)".
