# SENNEP V2 — Projektoverblik

Rekonstrueret 2026-10-04 ud fra: Gemini-samtalen "Robot Redesign: From Box to Beauty" (17. juli 2025, `Samtale.pdf` / share-link),
Gemini-samtalen om motoren (`samtale-2.pdf`), tre kvitteringer, `Cut noter.txt` og `Lidar Test Enviornment/`.
ChatGPT-projektet (8 samtaler + filer, bl.a. `Fordybelsesprojekt - Rapport.docx` og `Hjul design noter og teori.docx`) er IKKE gennemgået — det ligger ikke i mappen.

## 1. Ideen

- **V1 = S.E.N.N.E.P.** — HTX-fordybelsesprojekt. Ca. 700×450 mm, ~90 % laserskåret akryl, kasseformet. Havde capacitive touch-sensorer (skolekrav). En Arduino døde/komponenter røg pga. spændingsspikes.
- **V2 = genopbygning for sjov og læring.** Holonomisk (mecanum) mobil robot, autonom navigation med 360° LiDAR, manuel override via RF-joystick, senere dockingstation.
- Status (jul 2025 + nu): "en bunke komponenter i en kasse". Ingen V2-robotkode. Eneste software: LiDAR-testmiljøet (Electron + Next.js) med optagelser fra 10. april 2026.

## 2. Faste designkrav (fra Mikkel)

| Krav | Værdi |
|---|---|
| Ydre mål | 650 × 400 mm (kan ændres lidt) |
| Hjul | Egne 110 mm mecanum, **flush med sidepanelerne**, inde i karosseriet |
| Fremstilling | 100 % 3D-print, PETG (15 kg på lager), Bambu X1C (256×256×256 mm) |
| Fastgørelse | Kun M3, godstykkelse 3 mm, printede slidser/tapper |
| Top | Flad, LiDAR i midten med frit 360° udsyn |
| Form | Simpel, symmetrisk, "selvbygget og engineered" — ikke sci-fi, ikke svære organiske kurver. Store fasninger/fillets OK |
| Sensorer i karosseri | 8 ToF lavt rundt om, 4 edge-detectors i hjørnerne |
| Front | 7" skærm + kamera, forlygter. Bag: baglygter |
| Plads til | Højtaler, buzzer, mikrofoner, temp/fugt, blæsere — plads er ikke et problem |

Gemini-forløbet om formen (Vector → Exo-Frame → Aura → Evo → Modular Plate → Praxis) endte uden brugbart resultat.
Sidste prompt-retning: aflang rektangulær krop, lodrette flade sider flush med hjul, fasede hjørner, synlige M3-bolte, flad top.

## 3. Arkitektur (besluttet)

- **Raspberry Pi** (højniveau: LiDAR, kamera, skærm, de fleste sensorer, Python fra bunden — INGEN ROS2).
- **Arduino Nano** (motorstyring + sekundær kollisionsundgåelse, RF-override). Mega som plan B hvis pins slipper op.
- Pi ↔ Nano via UART.
- **Opdateret 2026-10-12:** Megaen (ikke Nano) styrer motorerne (4× BTS7960 = 8 PWM + 4 strømsense). Elektronikplanen ligger i `docs/electronics/BREADBOARD_PLAN.md`.
- **Testbatteri:** Zippy Compact 1000 mAh 3S 35C LiPo uden BMS — det ødelagde V1's elektronik. Kun med sikring ved XT60 og LiPo-alarm.
- Strøm: 3S Li-ion → sikring → nødstop (hovedafbryder) → 12 V direkte til motordrivere; buck-convertere → 5 V til logik. TVS/Schottky/kondensatorer pga. V1-skaden.

## 4. Krydstjek: kvitteringer

> **Komponentlisten ligger nu i `components/`** (`inventory.csv` = kilden, `index.html` til at browse, datablade i `components/datasheets/`). Slå op dér i stedet for i tabellerne nedenfor eller i Gmail. Oprettet 2026-10-07.

| Dato | Butik | Beløb | ≈ DKK |
|---|---|---|---|
| 14-05-2024 | Ardustore (Odense) | 2.042 kr | 2.042 |
| 18-04-2025 | Hobby Components (UK) | £101,51 | ~880 |
| 01-05-2025 | Bot'n Roll (PT) — kun ordreliste, ingen faktura-PDF | €407,35 | ~3.040 |
| | | **I alt** | **~5.960 kr** |

Det passer med "ca. 6.000 kr". Men følgende **mangler i kvitteringerne**:
- Raspberry Pi (der er købt et **Pi 5**-alukabinet — oversigten siger Pi 4)
- LiDAR (LDROBOT STL-19P — den findes, testmiljøet har kørt på den)
- Motorer (4× RS PRO 834-7641, ~2.400 kr i alt, købt 2020–23, sandsynligvis via skolen/V1)
- TVS-dioder, I²C-multiplexer (begge noteret som "mangler"), ledninger, PETG

### AliExpress, december 2025 (fundet i Gmail — 18 ordrer)

| Vare | Antal | DKK |
|---|---|---|
| H-Bridge High-Power Motor Driver (formentlig BTS7960, 43 A) | 5 | 182,25 |
| KFL003 17 mm pillow block-lejer (2 stk/pk) | 3 pk = 6 | 130,91 |
| Silikonekabel 10 AWG, 2 m | 3 | 114,75 |
| Kabel 22 AWG, 50 m (5×10 m) | 1 | 74,01 |
| 290 stk rustfri unbrakoskruer (sortiment) | 1 | 63,62 |
| Kabel 16 AWG, 10 m | 1 | 59,96 |
| WS2812B LED-strip, 5 m, 30 LED/m | 1 | 52,30 |
| Messing-gevindindsatser M3, 100 stk | 1 | 41,52 |
| TCA9548A I²C-multiplexer | 5 | 34,83 |
| Bilsikringer (blade), 180 stk | 1 | 27,79 |
| GY-271 QMC5883L magnetometer | 1 | 18,99 |
| GY-NEO6MV2 GPS | 1 | 16,66 |
| INA219 strøm/spændingsmåler | 1 | 14,52 |
| PAM8403 mini-forstærker, 5 stk | 1 | 11,87 |
| 4-kanal logic level converter | 1 | 11,35 |
| Loddbart breadboard 400 huller | 1 | 10,16 |
| 0,28" voltmeter-display 0–30 V | 1 | 9,46 |
| 12 V 40 A relæ, 5-ben | 1 | 7,58 |
| **I alt** | | **~883 kr** |

Med AliExpress er samlet dokumenteret forbrug ~6.840 kr. Ingen andre AliExpress-ordrer i Gmail siden 2024.

### Hvad der er købt, pr. system

- **Compute:** Arduino Mega ×2, Nano ×2 (+2 terminal-adaptere), Pi 5-kabinet, 32 GB + 16 GB SD, ESP-01 ×1
- **Drivlinje:** L298N ×2 (= 4 kanaler, nok til 4 motorer), motorer fra V1
- **Strøm:** 20× Samsung 35E 18650 (3400 mAh) + holdere, 3S-lader 12,6 V 2 A, 3S 40 A BMS ×2, XT60 ×2, sikringsholdere ×10, nødstop 40 mm, 5 A buck ×2, LM2596-modul ×3, L7805 ×10, LM2596T-12 ×3, LD1117-3.3 ×3, breadboard-PSU ×2, elektrolyt- + keramikkit
- **Sensorer:** VL53L0X ToF ×8, E18-D80NK ×4 (edge), MPU-6050, TSL2561 lys, IR-modtager ×10 + IR-LED ×10 (dock-beacon), HC-SR04 ×4, mikrofonmoduler ×5, DS3231 RTC
- **UI:** 7" HDMI (C) 1024×600 touch, Pi AI Camera (IMX500), 20×4 LCD ×2, 16×2 ×4, I²C-adaptere ×6, DFPlayer ×3, højttalere 1 W ×8, 3 W LED ×4, MOSFETs (IRFZ44N ×5 + 3)
- **Manuel styring:** NRF24L01 ×3, joystick ×4
- **Mekanik/diverse:** 40 mm blæsere ×4, protoboards 12×18 ×4, skrueterminaler, JST PH-stik, tang, loddetin

## 5. Modsigelser og problemer fundet

1. **Batteri: 3S6P, ikke 6S3P.** Første Gemini-chat regnede med 6S3P (21,6 V) — det ville ødelægge 12 V-motorerne. Motor-chatten, BMS (3S) og lader (3S) bekræfter 3S6P: 10,8 V nom / 12,6 V fuld, ~20,4 Ah, ~220 Wh. Driftstidsestimatet (3,5–5 t) holder stadig.
2. **LiDAR er LDROBOT STL-19P** (LD19-protokol, UART 230400 baud), ikke RPLIDAR A1 som Gemini skrev.
3. **Pi 4 eller Pi 5?** Kabinettet er til Pi 5. Pi 5 vil have 5 V/5 A — 5 A-buck'en passer.
4. **L298N er flaskehalsen.** Max ~2 A/kanal + ~2 V spændingsfald; motoren er 41 W (3–4 A normal, >10 A stall). Fint til at teste med en let robot + soft-start i koden; opgradering = 4× BTS7960 senere. Gemini sagde "4× L298N" — der er kun 2, men de er dobbeltkanals, så det er nok.
5. **Motoraksel 8 mm** (RS 834-7641) — ChatGPT-oversigten nævnte "5 mm aksel". Skal tjekkes mod hjul-hub'en.
6. **8× VL53L0X har samme I²C-adresse (0x29).** Løsning uden køb: XSHUT-ben på hver sensor + adressetildeling ved opstart (koster 8 GPIO). Ellers TCA9548A-multiplexer (~40 kr).
7. **MOSFETs er ikke logic-level** (IRFZ44N). Fra Pi's 3,3 V åbner de knap nok; fra Nano 5 V går det til 3 W-LED'er. 3 W-LED'erne kræver strømbegrænsning.
8. **Nødstoppets strømrating** skal tjekkes — små 40 mm-nødstop er typisk ~10 A. Hvis det skal bryde hele batteriet, skal ratingen passe (eller lade det styre et relæ).
9. **Mikrofonmodulerne** (justerbar detektion) kan kun give lydniveau/tærskel — ikke reel lydretning.
10. ChatGPT-oversigten opfandt ROS2, motorkode-strukturen er fra V1, og "touch/tryksensorer" er droppet. Behandl den oversigt som upålidelig.

## 6. Cut noter (testudskæring af frontpanel)

Kamerahul +0,5 mm i højden; LED-huller ~1 mm tættere; blæserhuller ~0,5 mm tættere; protoboard ~1 mm tættere på den korte led;
skærmudskæring passer, men skærmens monteringshuller er helt forkerte. Lyssensor, ToF og kameraets monteringshuller passer.

### Løst af AliExpress-ordren (dec 2025)
- Punkt 4: 5× high-power H-bridge købt → L298N kan erstattes (4 i brug + 1 reserve).
- Punkt 6: 5× TCA9548A købt → multiplexer-løsningen er valgt (XSHUT ikke nødvendig).
- Punkt 8: 40 A-relæ købt → nødstoppet kan styre relæet i stedet for at bryde hovedstrømmen selv.
- Batteriovervågning: INA219. Kabler: 10/16/22 AWG. Sikringer: blade-sikringer.
- GPS (NEO-6M) er købt, selvom GPS blev droppet i ChatGPT-planen — uafklaret om den skal med.

## 7. Afklaret 2026-10-04

- Pi 5 (ligger hos forældre, hentes i efterårsferien).
- Ét hjul printet og testet; samles uden ekstra dele. Virker.
- Biblioteket har gratis laserskærer, men 100 % 3D-print foretrækkes (modulært, udskifteligt).
- Designretning i Fusion (screenshot): facetteret krop med fasede flader, skærm på skrå frontflade, flad top med trekants-gitter og LiDAR i midten. Komponenter i modellen: "Block Bearing", "Mount Plate", "... Mount V2".

## 7b. Fra ChatGPT-teksten (`gpttxt.txt`, kopieret 2026-10-05)

Tidslinje (2025):
- **7. apr** — V2 startes. Mål: køre selv og "se" (V1's ultralyd var for dårlig) → LiDAR + kamera.
- **9. apr** — Undersidemodel: skal sensorer vinkles; forslag om sensorer på hjørneflader pegende lidt nedad.
- **18. apr** — Strømarkitektur (3S, buck pr. gren, Schottky/TVS/kondensatorer/sikringer). Krav nævnt: **robotten skal være vandsikret (regn + vandpytter)**. Ingen motor-encodere (bevidst valg).
- **18. apr** — Hobby Components-ordren; 8 ToF begrundet: N/S/Ø/V + de 4 skrå paneler (NØ/NV/SØ/SV).
- **Apr** — Docking: konstant sendende 940 nm IR-beacons på docken, IR-modtagere på robotten; rotér til bageste/forreste modtagere balancerer, centrér sidelæns, kør frem. Ladning via to metalplader + fjederkontakter.
- **19. apr** — Stel stadig i laserskåret sort akryl; spørgsmål om fugemasse i samlinger.
- **4. maj** — Luftindtag: 4 riste á 13 slidser (~6865 mm²), exhaust-blæsere, støvfilter; placering af lyssensor. (Konflikter med vandsikring — uafklaret.)
- **10.–11. maj** — Hjulets rotationslås: M3-hul printes Ø2,4, bores 2,5, tappes M3.
- **9.–29. maj** — Printervalg → Bambu X1C.
- **16.–17. maj** — Pi 5 + AI Camera modtaget; Pi OS installeret (bruger `sennep`); AI-kamera-model (emotion recognition) forsøgt. **3S6P-batteripakken samlet** med 3S BMS og ren nikkelstrip 0,15 × 27 mm (2H-19 mm).
- **25. maj** — LiDAR: **youyeetoo FHL-LD19** (LD19-protokol, 230400 baud); Tkinter-testprogram `LidarDisplayTest.py` på Pi.
- Farver: lyst/gult hjul med sorte sleeves; ønske om mindre "helt sort" kabine.

ChatGPT's egen "overlevering" (øverst i filen) er upålidelig: opfundet ROS2, V1-kodestruktur, "5 mm aksel/direct drive" (faktisk: hjulakslen bæres af 17 mm pillow block-lejer, så motoren ikke bærer vægten).

## 7c. Fusion-modellen "Robot v58" (læst via Fusion MCP 2026-10-05)

Koordinater i mm (Fusion-origo). Robotcentrum x = −154,5.
- **Skal (Body422):** én hul solid, 500 (b) × 816,5 (l) × 200 (h), z −7,2…192,8. ~2 mm vægge. Facetteret, skærm på skrå front. Toplåg = 2 gitterplader (Body433/434, 140 × 621,5 × 3 mm). LiDAR-holder Body428/429/432 på toppen.
- **Hjul:** Ø130 mm (ikke 110), bredde ~98 mm, flush med siden. Centrum (−356,9, −370, 0). Frihøjde under skal ~58 mm. Samlet højde gulv→top ~258 mm + LiDAR.
- **Sporvidde** ~405 mm (hjulcentre), **akselafstand** 470 mm (hvis bageste hjul spejles om y = −135).
- **Kun ét hjørnemodul er modelleret (forreste venstre):** Base Plate (214,5 × 318,3 × 2 mm), Wheel Well (grøn bue over hjulet), Base Mount Plate (hænger ned og bærer pillow block-lejet), Motor Mount V2, Motor (140,5 mm lang, Ø44), Pillow Block Bearing, Lock Bolt.
- Skjulte pladsholdere: 4× 40 mm blæser, 2× højttaler + cover, protoboard 120×180, Body402 (200×200×10).
- 838 timeline-elementer, ingen brugerparametre.
- Aktuel størrelse (500 × 816) afviger fra 2025-kravet 650 × 400.

## 7d. Motoren — RS PRO 834-7641 (975D-serien), officielt datablad

Kilde: docs.rs-online.com/5f68/A700000007670028.pdf (hentet 2026-10-05).
12 V børstet DC, planetgear 49:1, 42 mm gear / Ø44,6 × 111,9 mm, 8 mm aksel (20 mm), 624 g, sintrede bronzelejer, levetid 3000 t.
- Tomgang: 143 rpm @12 V (107 @8 V, 72 @6 V). Effekt 41,3 W.
- **Nominelt maks. moment 18 kgf·cm (1,77 Nm)**, kortvarigt maks. 54 kgf·cm (5,3 Nm). Virkningsgrad 60 %.
- **Radial last på akslen < 5 kgf (10 mm fra flangen)**, aksial < 13 kgf → motoren må IKKE bære robotten; pillow block-lejerne er nødvendige.

Bæreevne-estimat (Ø130 hjul, r = 0,065 m): 27 N/hjul ved nominelt moment, 109 N i alt. Med 60 % udnyttelse og mecanum-virkningsgrad 0,7 frem / 0,5 sidelæns, acceleration 0,2 m/s²:
- Hårdt gulv (Crr ≈ 0,04): ~75 kg samlet frem, ~55 kg sidelæns.
- Tæppe (Crr ≈ 0,10): ~40 kg frem, ~28 kg sidelæns.
- Topfart ~0,95 m/s tomgang, realistisk ~0,6–0,7 m/s.
Konklusion: ~50 kg samlet (robot + last) er realistisk designmål; 100 kg kun på glat gulv og på grænsen. Næste begrænsninger: printede ruller/rullepinde, ramme, kobling motor→hjulaksel.

## 8. Åbne spørgsmål

- Fusion MCP registreret i Claude Code (`fusion` → http://127.0.0.1:27182/mcp); kræver ny session for at værktøjerne indlæses.
- ChatGPT-zip (med billeder) kommer senere.
- AFKLARET: ikke vandtæt (indendørs) → ventilation beholdes.
- GPS (GY-NEO6MV2, købt dec 2025) skal med. IMU (MPU-6050) + magnetometer skal med.
- Batteri: 3S6P-grupper samlet, men serieforbindelserne er ikke lavet endnu (Mikkel er utryg ved at lodde/svejse).
- Størrelse: 500 × 816 (Fusion nu) eller 650 × 400 (2025-krav)?
- Hjul + lejemekanisme er stort set færdigdesignet; resten af kroppen og sensorplacering mangler.

## 9. Nyt Fusion-dokument "SENNEP V2" (oprettet 2026-10-05)

Konvention: **front = −Y** (Fusions "Front"-visning), venstre = −X, op = +Z, origo = midt i fodaftrykket på gulvet.
- **Parametre** (Modify → Change Parameters): RobotLength 700, RobotWidth 500, WheelDia 106,2 (målt), WheelWidth 55 (rullende bredde), WheelOuterOffset 47,6, Track = RobotWidth − 2·WheelOuterOffset, Wheelbase 450, BodyBottom 58, BodyTop 258, Wall 3, Screw 3, InsertHole 4, MaxPrint 250 (mm).
- **Komponenter:** 00 Layout · 10 Drive · 20 Chassis · 30 Electronics · 40 Sensors · 50 Shell.
- **00 Layout / "Footprint (gulv)":** fuldt bestemt skitse; ydre rektangel (RobotWidth × RobotLength) + konstruktionsrektangel for hjulcentre (Track × Wheelbase). Testet: følger parametrene.
- **"10 Drive Module"** (direkte under roden, 4 instanser — wrapper-komponenten "10 Drive" blev fjernet fordi indlejrede transformationer gav dobbelt forskydning): Wheel_1, Motor_1, Pillow_Block_Bearing, Base_Mount_Plate, Motor_Mount_V2 importeret fra Robot v59. Origo = hjulcentrum, hjul ud mod −X. Venstre uroteret, højre roteret 180°.
- Modul-positionerne er sat med transformationer (ikke låst til parametrene endnu — joints til layout-punkter fejlede). Ved ændring af Track/Wheelbase skal modulerne flyttes (script).
- **TODO:** FR og RL skal have spejlvendte ruller (mecanum X-konfiguration: FL = RR, FR = RL). Lige nu er alle fire ens.
- **Svaghed:** hjulakslen gennem lejet er 3D-printet og bærer hele vægten → bør erstattes af stålaksel (Ø17 i lejet) eller printet bøsning på stålkerne.
- Fusion API-noter: scripts kræver `def run(context)`; nested occurrences flyttes via `createForAssemblyContext(...).transform2`; `moveToComponent` nulstiller transformationen.

## 10. Hjulanalyse (målt i Fusion 2026-10-05)

- Ydre diameter **106,2 mm** (ikke 130 — Fusions occurrence-bbox inkluderer skitser). Gammel placering svævede 11,9 mm.
- **10 ruller**, rulleakse på radius 43 mm, rullevinkel **≈ 40°** (standard mecanum = 45°).
- Rulle Ø18,2 (tøndeform), sleeve Ø20,2, længde ~47 mm.
- Frigang sleeve–sleeve 0,95 mm, sleeve–fælg 0,79 mm (stramt for FDM + TPU).
- Dækning: rullens tangentiale længde ≈ 47·sin40° ≈ 30 mm mod rulleafstand 2π·53,1/10 ≈ 33,4 mm → små huller mellem ruller (let "hakken"). Ved 45° ≈ 33,4 mm = præcis dækning.
- Bæreevne opdateret for Ø106: 33 N/hjul nominelt → ~90 kg samlet på hårdt gulv, ~48 kg på tæppe (frem). Topfart ~0,8 m/s tomgang.

## 11. Hjulgenerator v1 (`fusion/wheel_generator.py` (erstattet af v2, se §12), 2026-10-05)

- Køres i Fusion via MCP (`exec` af filen) — genopbygger komponenten `P['TARGET']` in place (default "11 Wheel Lab", står ved x = 450 mm ved siden af robotten).
- Beregner ideel rulleprofil ud fra OD, vinkel og rmax; rapporterer dækning, frigange, pinlængde.
- Struktur: "Roller" (én master-rulle: kerne + TPU-sleeve + skjult frigangsværktøj — eksportér denne til print), "Rollers (assembled)" (N kopier som bodies), "Rim A (pin side)", "Rim B (twist-on side)". Fælge er base-features bygget med TemporaryBRepManager.
- Montering i v1: stålpinde Ø4 presses i Rim A, ruller på, Rim B drejes på (pindene ligger langs en helix, så det kræver samme drejebevægelse som i Mikkels design), 3× M3 gennem begge halvdele låser rotationen (erstatter den gule lås). Hex-hul 17 mm AF til separat aksel.
- Testede varianter @ OD 106,2 / 45°: N=10 rmax=10 → rulle-rulle 0,2 mm (for tæt); **N=9 rmax=10 → 2,6 mm (valgt)**; N=10 rmax=9 → 2,7 mm; N=8 rmax=11 → 3,0 mm.
- Kendte mangler v1: fælgfingrene er tynde (beskæres af nabo-ruller) — skal forstærkes; ingen aksel/kobling endnu; ingen bajonet-klik; Drive Module bruger stadig det gamle hjul.

Styrkeoverslag (50 kg samlet → ~123 N pr. rulle statisk, ~250 N ved bump; fordelt last på pinden):
- Printet PETG-pind Ø7 (nuværende design): ~21 MPa statisk / ~43 MPa dynamisk, printet på tværs af lag (~20–30 MPa) → OK til robotten alene, ikke til børn.
- Stålpind Ø4: ~230 MPa dynamisk → kræver hærdet dowel pin (≥ 600 MPa); alm. blødt stål er marginalt → evt. Ø5.
- SLS PA12 (~48 MPa, isotropt) er markant bedre til fælge/nav end FDM-PETG; pinde bør stadig være stål.

## 12. Hjulgenerator v2 — 100 % printet (2026-10-05)

Erstatter v1 (stålpinde droppet: Mikkels krav = alt printes på én plade og samles uden værktøj/indkøb; barn ovenpå er kun en party-gag).
- Fil: `fusion/SENNEP_WheelGenerator/SENNEP_WheelGenerator.py` (+ `.manifest`) — en rigtig Fusion-script-mappe.
- Input = brugerparametre `Wheel_*`, `Axle_*`, `Motor_*` (oprettes automatisk ved første kørsel; redigeres i Change Parameters).
- Ruller kører på **printede pindestumper Ø6,5** fra begge fælghalvdele (boring Ø7,1). Rim B drejes på (helix).
- **Sekskantakslen (18 mm AF) går gennem begge fælge og låser drejningen** → ingen separat lås.
- Aksel: hoved Ø30 × 4 udvendigt, sekskant gennem hjul + afstandsrør, Ø17 gennem lejet, kobling med D-boring til Ø8 motoraksel + radialt Ø4-hul til M3-indsats/pinolskrue. Printes stående på hovedet. Rim A har afstandsrør Ø24 mod lejets inderring.
- Sideskive (Wheel_SidePlate 3 mm) + 6 navvinduer + afrundinger (try-fillet, springes over hvis Fusion fejler) + farver (SENNEP Rim/Roller Core/Sleeve/Axle).
- Resultat @ default: 9 ruller, længde 48,3, Ø20/Ø14, rullevæg ved enden 2,24 mm, frigang rulle-rulle 2,58 / rulle-fælg 1,46, hjulbredde 46,3, aksel 101,5 mm.
- MCP-kørsel: `exec` filen og kald `build()` (run() viser messageBox og må ikke bruges via MCP).
- Mangler: polish på sidefladens slidser, bajonet-klik (ikke nødvendig pga. hex-lås), spejlvendt hjul i drivmodulet, wheel well-generator.

### v2.1 polish (2026-10-05)
- Sideskiven placeres lige uden for rullernes frigangsvolumen (beregnet) → ingen slidser i skiven; pindeholderne går direkte fra rulleenden ind i skiven (ingen eger), trimmes flugtende med ydersiden.
- Pindestumper: affaset spids (nem samling) + konisk flare ved roden (`Wheel_PinRoot` 1 mm) for styrke; rullens boring har matchende affasning.
- Hjulbredde nu 54,5 mm (tidl. 46,3), aksel 109,7 mm. Frigang rulle-fælg præcis 1,50 mm.

### v2.2 — Mikkels egne ændringer indarbejdet (2026-10-05)
Workflow testet: Mikkel lavede features på Rim A efter generering; de blev læst fra tidslinjen (rul markøren tilbage før featuren for at læse edges) og oversat til regler:
- Extrude +5 mm på yderfladen → `Wheel_FaceDepth`; Chamfer 5 mm på yderkanten → `Wheel_FaceChamfer` (bygget som kegle i TBM); Fillet 8 mm flade/kegle → `Wheel_FaceBlend`; Fillet 4 mm ved afstandsrørets rod → `Axle_SpacerFillet`; 0,5 mm på toppen. Anvendt på begge fælge.
- Nyrevinduer (cirkelsektorer med runde ender) som ikke overlapper akselhovedet (`Wheel_WindowWeb`), indre afrundinger 2–3 mm (`Wheel_InnerFillet`), flare-rest på pindene fjernet (nabo-frigang trækkes fra stubben).
- Hjulbredde nu 64,5 mm, aksel 119,7 mm. Alle afrundinger lykkes (rapporteres pr. trin).

### v2.3 (2026-10-05)
- Lukning: 3× M3×20 skrues ind **fra robot-siden** gennem Rim A (forsænket 17,8 mm) i gevindindsatser Ø4×6 i Rim B — usynligt udefra (`Wheel_Bolts`, `Wheel_BoltLen`, `Wheel_InsertDia/Depth`). Vinduer drejet så skruerne sidder i webs (`Wheel_WindowWeb` 9).
- Afstandsrøret er nu en **separat del** ("Spacer", hex-boring, printes stående) → begge fælge har flad yderside og kan printes liggende.
- Artefakt fjernet: pindens flare stak 0,15 mm ud gennem fælgkanten → stubben klippes til fælgradius (præcis samme flade). 0 små flader tilbage.
- Rulleender: kun 0,4 mm afrunding (`Wheel_RollerEndFillet`) → flad ende ~20 mm² at printe på.

### v2.4 — skruesamling af fælghalvdelene (2026-10-05)
- Rim A har et **gevindrør M30×2** (modelleret Fusion-gevind, flanker trukket 0,15 mm tilbage for FDM) som går gennem et **rundt hul** i Rim B — Rim B kan dreje frit når den skrues ned langs de skrå pinde (~23° drejning over ~17 mm aksial vandring; pindenes "lead" ≈ 271 mm/omdr., for stejl til at være selve gevindet).
- **Printet navmøtrik** (Ø38 × 7, to slidser til mønt/skruetrækker) i forsænkning i Rim B's yderside spænder halvdelene sammen; akselhovedet ligger oven på og holder møtrikken.
- Sekskantakslen sættes i til sidst og låser derefter alt (så den hindrer ikke drejningen ved samling). M3-skruerne er slået fra (`Wheel_Bolts` = 0) men kan genaktiveres.
- Målt: frigang gevind 0,14 mm, aksel–Rim A 0,15 mm.
- Samling: ruller på Rim A → drej Rim B ned over pindene → skru møtrikken på → aksel igennem udefra → afstandsrør → leje → motoraksel + pinolskrue.

### v2.5 — skrå riller i stedet for møtrik (2026-10-05)
Møtrik/M30-gevind droppet (Mikkel: drejningen skal følge pindenes 45°-bane, ellers brækker pindene ved samling).
- Rim A: rundt navrør Ø30 med **3 helix-riller** (5 × 2,5 mm), Rim B: rundt hul + matchende spor (0,3 mm frigang). Helix: dθ/dx = HAND·tan(vinkel)/Ra ⇒ samme skruebevægelse som pindene; ~42° drejning over rørets længde. Rillerne griber før pindene når rullerne.
- Bygget som sweep med guide rail (Fusion-features oven på base-feature). Interferens Rim A/B: 0,01 mm³ (kontakt).
- Lås: sekskantakslen (ingen drejning ⇒ ingen adskillelse). Parametre: `Wheel_HubTube`, `Wheel_Keys`, `Wheel_KeyWidth`, `Wheel_KeyHeight`, `Wheel_KeyClear`.

### v2.6 — joints, samletest, bajonet (2026-10-05)
- **Skrue-joint i Fusion:** "Screw Carrier"-hjælpedel + slider (Rim B slide) + revolute (Rim B turn) koblet med Motion Link "Rim B screw path" (32,3 mm ↔ 43°). Alle andre dele grounded → træk i Rim B for at skrue af/på.
- **Automatisk samletest** i generatoren: Rim B flyttes langs skruebanen i 12 trin og skæres mod Rim A + ruller (TBM). Fandt reel konflikt: lige pinde på en skruebane driver ud af rullens boring (op til 90 mm³/rulle med halvlange pinde).
- **Fix: asymmetriske pinde** — Rim A lange (næsten gennem rullen; ruller sættes på i lige linje), Rim B korte (`Wheel_StubB` 6 mm, ~4,5 mm indgreb). Samletest: ingen kollision.
- **Bajonet på afstandsrøret** (`Spacer_*`): Ø26-tap med 3 tapper går 5 mm ind i Rim A, 60° drej låser under kant (60° ⇒ sekskanterne flugter igen; akslen låser drejet). Testet: fri ved −60/−30/0°, låst ved træk (74 mm³ kollision = holder).

### v2.6.1 — klar til testprint (2026-10-05)
- Akselhoved Ø42 (dækker Rim B's hul Ø30,8 + riller til Ø35,6; generatoren advarer hvis ikke). Navvinduer flyttet ud: r 23,5–35 (`Wheel_WindowOuter`).
- Generatoren sletter nu gamle joints/motion links før genopbygning (ellers fejlede base feature).
- Fillet-regler: bosses/plate ignorerer vindues-vægge; hub-kanten accepterer buer.
- **STL eksporteret** til `print/testhjul_v2.6/` (Rim A, Rim B, rullekerne ×9, sleeve ×9, aksel, afstandsrør) + `LÆS MIG.md` med orientering, samling og testpunkter.

### v2.7 — FDM-tolerancer + Mikkels vindues-fillet (2026-10-05)
- Tolerancer gennemgået: rulle/pind 0,3 radialt, riller 0,3/side, navrør 0,4, bajonet 0,2–0,3 (OK). Rettet: `Axle_HexClear` 0,3→0,5 (total), motorhul nu Ø8,4 (`Motor_ShaftClear` 0,2), lejesektion Ø16,9 (`Axle_BearingDia`). Anbefal elephant-foot-kompensation 0,15.
- Mikkels Fillet6 (3 mm på Rim B, vinduernes kanter inde i hjulet: cap-cylindre r=5,75 og ydre bue r=35 mod skivens inderside + aksiale kanter mod navet) → regel "windows inside" på begge fælge (`Wheel_WindowInnerFillet`), udelader kanter på yderfladen og samlefladen.
- STL: `print/testhjul_v2.7/` (v2.6 slettet).
- v2.7.1: rullens borings-affasning 1,3 → 0,6 mm (`Wheel_BoreChamfer`) → flad ende 40 mm² (før ~20). Pindens rod-flare ender før rullen, så dyb affasning var unødvendig. Den skjulte dog en skævhed: Rim B's korte pinde fik +0,3 mm radial frigang (`Wheel_StubBClear`). Samletest nu 44 trin (0,25 mm i de første 8 mm): kun let berøring 0,09 mm³/rulle (~0,05 mm) = OK. STL re-eksporteret (Fusion v13).

### v3 (2026-10-05) — efter testprint af v2.7 (PETG, 0,16 Optimal, X1C)
Feedback fra print og løsning:
1. **Bajonet-lommen** i Rim A havde et fladt tag. Det er 100 % overhang, når fælgen printes med ydersiden ned, og det blev spaghetti, så låsen kunne ikke drejes.
   → Første forsøg (et skrå sæde ved mundingen og en krave) blev afvist af Mikkel: sædet ligger ved printpladen og hjalp ikke.
   → Nu (Mikkels idé): trekantede tapper. Én 45°-kegle går fra rillen helt ned til sekskanten.
   → Afstandsrørets tapper og skrå ende ligger på samme linje med 0,2 mm frigang (`Spacer_Clear` − `Spacer_LockClear`).
   → `Spacer_LockDepth` = dybden af rørets flade ende. Ingen overhang-flader verificeret i både Rim A og afstandsrøret. Indgang, drej og lås er verificeret. Fusion v15.
2. **De lange Rim A-pinde knækkede.**
   → Korte pinde i begge ender: `Wheel_StubA` 8, `Wheel_StubB` 6. Rim B's pind er tilspidset (`StubBClear` = radial tilspidsning ×2).
3. **Rullerne satte sig fast ved enderne.** Pinderodens flare (1,3 mm kegle) kilede sig ind i boringens affasning.
   → Flad trykring (`Wheel_ThrustRing` 2,5) og `Wheel_EndGap` 0,4. Flaren er nu kun 0,5 mm og ligger inde i affasningen.
4. **Pasningerne var for løse** (hjulet slingrede ~0,25 mm).
   → `Wheel_KeyClear` 0,3→0,1/side, `Wheel_TubeClear` 0,4→0,15 og `Axle_HexClear` 0,5→0,2.
   → Lejesektionen Ø16,9 er perfekt ifølge Mikkel og er uændret.
   → `Axle_SetScrewHole` 4,0→4,3 (Mikkels ønske).
5. **Rullerne nåede ikke gulvet.** Kernen stak 0,8 mm over fælgen minus op til 0,6 mm frigang.
   → Sleeves skal stadig være der, men hjulet skal virke uden dem.
   → Ruller Ø21,5, `Wheel_Overlap` 4 % og `Wheel_RimMargin` 3,5. Kernen stikker nu 2,0 mm ud uden sleeve og 3,2 mm med.
   → Ø22/Ø23/Ø24 blev afvist: sleeverne rører/overlapper deres nabo. Den smalleste afstand ligger midt på rullerne, så overlap-procenten hjælper ikke; Ø22 er grænsen ved 9 ruller.
   → Bemærk: pinderoden sidder kun 0,9 mm under fælgkanten på ydersiden (generatoren advarer). Trykringen omslutter den, så det er accepteret indtil videre.
6. **Støj i samletesten.** Rillen og sporet blev swept langs forskellige midterradier, hvilket gav ~0,003 mm falsk overlap mellem Rim A og B.
   → Sporet bruger nu `tube_r - 0.5 - KC`, så midterradierne er ens.

Andet:
- Ny komponent "11 Wheel Lab Testprint (hurtigtest)" (skjult) med udsnit til hurtig test af pasningerne.
- STL ligger i `print/testhjul_v3/` (+ `hurtigtest/`). Fusion v14.

### Samlering (2026-10-12)
Rullerne faldt af Rim A's korte pinde ved samling. `fusion/SENNEP_WheelGenerator/assembly_ring.py` → `build_ring()` laver en genbrugelig ring (r 50–56, x 0,5 → bænk), som Rim A ligger i, med vugger = rullerne swept langs løfteretningen. Varianter med/uden sleeves. STL i `print/testhjul_v3.1/samlering/`.

## 13. Skelet — udkast 1 (2026-10-05, intet modelleret endnu)

Udkast: `docs/skelet_udkast_v1.html` (artifact https://claude.ai/artifact/5FDjRdiQJ4nmFvDRG4YigC). Planen er en `SENNEP_ChassisGenerator` efter samme opskrift som hjulgeneratoren.

**Fund: motorerne kolliderer ved RobotWidth 500.** Den aksiale stak pr. side fra hjulets yderside er 4 + 64,1 + 12,7 + 19,5 + 19 + 4 (tap + fri aksel) + 111,9 + 9 (terminaler, gæt) ≈ 244 mm. Luft mellem terminal-enderne: 500/offset 47,6 → −11,5; 500 flush → 7,6; 520 flush → 27,6; **540 flush → 47,6 (anbefalet)**. Flush = akselhovedet 2 mm inden for sidelinjen. Tegningerne er lavet ved 540 × 700.

**Koncept:**
- **Pod** pr. hjørne (én del: hjulkasse + lejevæg 10 mm + motorrør + 45 mm skinnestump), ca. 240 × 120 × 115. Printes **stående på motorenden** → lag ⟂ akslen, alle huller lodrette, lejevæggens bøjning ligger i lagplanet. Spejles L/R.
- Lejet centreres af en ring i væggen (lejehusets bagside), 2× M5 gennembolte med møtrik (væggen i tryk). Ingen indsatser i lastvejen.
- **Flydende motor:** printet krave på motorfronten (4× M4 i motorens egne M4×6-gevind, PCD 35 iflg. datablad) med 2 trekantede lugs oppe/nede i kanaler i røret (0,1 mm tangentialt, luft radialt og aksialt). Kun drejningsmoment til poddet; akslen bærer kun motorens vægt. Endedæksel 2× M3 med 2 mm aksial luft. Vindue til pinolskruen i siden.
- Samleklods i midten mellem L/R pods (forreste og bageste akse), sideskinner (I-profil 60 × 40, 240 lange, ét stykke), 4 gulvfliser 155 × 165 med 20 mm M3-indsatsgitter, 6 kofangere 180 × 63.
- Frihøjde under motorer = hjulradius − motorradius − væg ≈ 27 mm (afledt).

**Motordatablad (s. 4):** 4× M4×6 dp på Ø35, tap Ø25 × 2, aksel 20 lang med D-flade (7 mm), gear Ø42,4 × 45,9 (49:1), motor Ø44,6 × 66.
**KFL003 i den gamle v59-model:** flange 71 × 46, boltafstand 56, huller Ø7, bagside Ø46, lejecentrum 7 mm fra flangen. Ikke bekræftet på et rigtigt leje.

**Styrke (50 kg, ×2 for bump):** 245 N/hjul; moment i lejet 13,4 Nm (54,5 mm arm); M5 ≈ 240 N/bolt; skinne 5,3 MPa / 1,8 mm; skinnesamling 20 Nm → 4× M5 gennem; lugs ≈ 100 N.
**Svageste punkt:** KFL003 er selvjusterende (kugleformet sæde). Hvis sædet drejer under last, går momentet til motorakslen (≈ 450 N mod 50 N tilladt). Foreslået test: 10–15 kg hængt 55 mm ude på akslen. Plan B: 2 lejer pr. hjul (2 ekstra KFL003, længere aksel = hjulchatten) eller udvendig støtte.

**Åbent (afventer Mikkel):** bredde 540/520; KFL003-mål; terminalernes længde + M4-hullernes placering; batteripakkens mål; lejetest; accept af flydende motor. Hjulchatten bør eksponere hjulets totale bredde som parameter (f.eks. `Wheel_TotalW`); indtil da kan generatoren måle "11 Wheel Lab".


**Udskiftelig skal (Mikkels ønske, ikke MVP):** alt der skal til for at køre sidder på skelettet (inkl. LiDAR-mast, ToF og kantsensorer i kofangerne, så kalibreringen ikke ændrer sig). Skallen har kun aftagelige moduler (skærm, kamera, blæsere, højttalere, lys) med stik. Grænseflade der lægges i generatoren nu: 6 skalbeslag (pod-toppe + skinnemidter), én stikplade på skelettet, maks. skalhøjde givet af LiDAR-masten.

### Skelet v0 — første hjørne i Fusion (2026-10-05)
Mikkels beslutninger: **800 × 650** (RobotLength × RobotWidth), reel printgrænse **220 mm** på alle led (`MaxPrint` = 220), lejebekymringen udskydes ("lav det, revurdér senere" — ingen penge til flere lejer), flydende motor OK. Wheelbase sat til **550** af Claude (samme 125 mm udhæng som før) — ikke bekræftet af Mikkel.
- Batteripakke (uden BMS/forbindelser): **122,7 × 62,1 × 67,8 mm**.
- KFL003: AliExpress-ordre ("2pcs Zinc Alloy Pillow Block KFL003 17mm" ×3). Standarddata (simplybearings/kugellager-express): 71 lang, 56 mellem hullerne, M6-hul (Ø7). Resten fra Mikkels model (`Pillow_Block_Bearing (1)`): flange 7 tyk, hus Ø46, inderring 17,5 lang.
- Motor: Mikkels model `Motor_1 (1)` bruges som sandhed: gearkasse Ø41,5 × 46 (databladet siger Ø42,4 — skal måles, motorrøret har kun 0,2 mm luft), maks. Ø44, M4-huller diagonalt på PCD 35,07, front → terminalende 120,5.
- Generator: `fusion/SENNEP_ChassisGenerator/` → komponent **"21 Pod Lab"** placeret om "11 Wheel Lab"-hjulet (x = 450). Indeholder pod, motorkrave, lejeholder + rigtige motor- og lejeinstanser. Læser hjulets mål fra de genererede bodies (Rim A, aksel, ruller) og `Axle_*`.
- Pod-koncept (ændret fra udkastet): printes stående på motorenden (169 mm høj, 118 × 89). Leje i en tæt lomme med bagsiden mod motoren, holdes nede af en printet **lejeholder** (6 mm, stifter i flangehullerne, 2× M3 i gevindindsatser). Motoren bæres af røret om gearkassen (0,2 mm/side) og flyder kun aksialt (−1/+6 mm); kraven (4× M4 i motoren) har to tapper i kanaler (0,1 mm). Låseskrue M3 + møtrik bag tappen. 45° kegle fra koblingen ud til kraven, diamantvindue i toppen til pinolskruen (x = 74,24), 45° overgang fra hjulkasse til rør.
- Tjek ved kørsel: ingen kollisioner (hjul, aksel, afstandsrør, motor, leje), hjulets frigangsvolumen fri, ingen overhang-flader i printretningen (kun 4 små vandrette huller = låseskruen), frihøjde 23,6 mm, motorluft 162 mm, hjulcentre x = ±287 (Track 573,9). `WheelOuterOffset` sættes af generatoren (38,04).
- Fusion-faldgruber fundet: (1) at ændre en brugerparameter eller tilføje en occurrence **nulstiller transform-overrides** på proxy-occurrences → sæt parametre først, tilføj alle occurrences, placér til sidst. (2) Slet base features før bodies ved genopbygning.
- Mangler: spejlede pods ×4, montering til skinner/gulv (ingen beslag endnu), udhuling (641 cm³ massiv), STL-eksport, de gamle "10 Drive Module" står stadig på de gamle positioner (ikke flyttet til 800 × 650).

### Skelet v1 — rullende skelet (2026-10-05)
Mikkels input: hjulkassen skal være rund og følge hjulet; hullet til pinolskruen rundt (hans skruetrækkere er maks. Ø16 og 150 lange, maks. ~100 mm dybt hul for at kunne dreje); motormålene i Fusion er skydelære-mål → **databladet gælder** (`Mot_GearDia` = 42,4); tætte pasninger må ikke kræve support; udkastet skal også dække bundpladen og samlingen.
- **Generator:** `build_pod()` → "21 Pod Lab" (detalje om "11 Wheel Lab") og `build_chassis()` → "20 Chassis" (robotkoordinater). Kør dem i hvert sit MCP-kald. Motor, leje og hjul vises som **kopier** ("(visning) …"-bodies), ikke occurrences — transform-overrides på indlejrede occurrences nulstilles ved enhver parameterændring.
- **Pod v1:** udvendigt en "tunnel" (lodrette flade sider under akslen, halvcirkel over) med rund hjulkasse indvendigt (R = hjul + 2 mm); 45° kegle ned til motorrøret. Skruetrækkerhul Ø17 (`Chassis_ToolDia`) som dråbe, ca. 51 mm dybt. 6 M3-indsatser (dråbehuller) i hver flad side, hvor bakkerne boltes på. 169 × 118 × 118 mm, 737 cm³ (massiv). Ingen overhang i printretningen (normaler samples på alle flader, også cylindre).
- **Bundbakker:** 12 åbne bakker, 40 mm høje (`Chassis_TrayH`), 3 mm vægge/bund, M3-indsatsgitter 40 mm i bunden, printes fladt. Midte 3 × 2 (216,7 × 215,9), for/bag: midterbakke (216,7 × 184) + 2 L-formede næser (nosestrimmel foran poden + strimmel under motoren). Nabobakker boltes væg-mod-væg med M3 + møtrik (dråbehuller, 50 mm deling, 2 rækker); bakker og næser boltes til podernes flade sider i M3-indsatser. 0,1 mm luft pr. side mellem dele.
- **Tjek:** ingen kollisioner (alle dele + motorer + hjul), alle bakker ≤ 220, ingen overhang. Bund z = 23,6 mm (frihøjde). Plast: bakker 2,4 L, pods 4 × 0,74 L (før infill).
- Kendte huller/mangler: åbne trekanter i gulvet mellem bakke og podens kegle (ca. 50 × 40 mm pr. hjørne); alle 4 hjul er ens (venstre/højre-håndede mangler — hjulchatten); pods er massive (skal udhules); ingen STL endnu; låg/øverste dæk og skal mangler; "10 Drive Module" ×4 er skjult (ikke slettet).

### Skelet v2 — fejlrettelser, hardware og batteri (2026-10-06)
**Fejl fundet af Mikkel (og rettet):**
- Podens ydre bue var en hel cylinder, der gik under gulvet, så hjulet ikke kunne røre jorden. Nu skæres alt under podens bund væk, og et nyt tjek melder fejl, hvis en printet del når under bakkernes bund.
- Dråbehullerne var tegnet forkert: diamanten blev ikke klippet ved tangentpunkterne, så "spidsen" var bredere end hullet. Rettet i `Geo.teardrop`.

**Mikkels hardware (gammelt sæt, købt før projektet; antal ca.):** M2×12/19, M3×12/19 (20 af hver), M4×12 (20), M4×25 (10), møtrikker og skiver M2/M3/M4. Ingen M5. Han har også AliExpress-sættet "290 stk rustfri unbrakoskruer" (størrelser ukendte) og 100 M3-messingindsatser. Modelleret som ISO 4762 (unbrako) — hovedtypen er ikke bekræftet.

**Ændringer:**
- Motorkrave 6 mm (`Chassis_CollarT`), så M4×12 + skive giver 5,2 mm i motorens 6 mm dybe gevind. Keglen til kraven er gjort bredere til M4-skiverne.
- Låseskruen bag kraven er fjernet: den sad i massivt materiale. Motoren holdes aksialt af pinolskruen i koblingen. **En bedre motorlås mangler.**
- Bakkerne har en 3 mm plade (dråbeform med spidsen nedad) ved hver bolt ind i en pod, så M3×12 når indsatsen. Podens indsatshuller ligger lavere eller højere, så skiven går fri af bakkebunden.
- Gitterhullerne i bakkerne går nu gennem bunden (Ø3,4), så enhver skruelængde kan bruges.
- Nye builds: `build_hardware_library()` → "90 Hardware" (alle størrelser i sættet + indsats), `build_battery()` → "31 Battery Module" (18 celler 3S6P, 3 grupper á 6 vekslende op/ned, 36 holder-klips) + "32 Battery Mount" (plade med kant + 2 klemmebøjler, printes på siden, 8× M3×12 i bakkens indsatser). Hardware-eksempler i "91 Hardware (placed)" (én bakkesamling + alle bolte i hjørne FL).
- **BOM for skelettet:** M3×12 ×144 (+8 til batteriet), M3×19 ×8, M4×12 ×16, M3-møtrik ×96, M3-skive ×248, M4-skive ×16, M3-indsatser ×56 (+8). → **Sættet rækker langt fra** (20 M3×12, 40 møtrikker, 40 skiver).
- Fusion-faldgrube: udseender sat på bodies *efter* `finishEdit()` forsvinder, når en anden base feature genberegnes. Sæt dem inde i `startEdit/finishEdit`.

### Skelet v3 — stigeramme (2026-10-06)
Mikkel kunne ikke lide bakkerne og ville have en "rigtig" bilramme (stigeramme). LiDAR'en skal sidde på toppen af skallen, højst 300 mm over gulvet.
- **LiDAR STL-19P (datablad):** 54 × 46,29 × 35 mm, 3× Ø2,5-huller (to med 46,8 mm imellem, det tredje 31,92 mm lavere; positionerne er aflæst fra tegningen og er omtrentlige), scanningsplan 27,2 mm over foden, pitch-fejl op til 2°. Foden ligger på `BodyTop` (258) → scanningsplan 285, top 293. Ved 2° falder planet 18 mm ved hjørnerne → **skallen skal holde sig under z ≈ 264.**
- **Ramme** (`build_chassis()` → "20 Chassis"), alle bjælker er omvendte U-profiler (toppen på printpladen, åbne nedad, så bolte nås nedefra), vægge 6 mm:
  - Længdebjælker |x| 191–251, 50 høje, 2 halvdele pr. side samlet ved y = 0. De forreste halvdele er 222 lange og skal ligge diagonalt på pladen.
  - 4 tværbjælker ved y = ±60 og ±170 (34 × 40), hver i 2 halvdele samlet ved x = 0. Hul i toppen for hver 30 mm til plader ovenpå.
  - Kofangere for og bag, 3 segmenter, boltet i podernes yderflader.
  - LiDAR-mast (printes stående) på tværbjælkerne ved y = ±60, med LiDAR-model.
- **Samlingsprincip:** hver samling har en V-nøgle (90°-ryg i den ene del, rille med 0,1 mm luft i den anden) plus 4 bolte. Rygge løber helt op til printpladen, og riller løber ud i bunden, så intet vender nedad i printet. Poderne har V-riller i begge flade sider (lodret + vandret).
- **Batteri:** pladen ligger på tværbjælkerne y = −170/−60 og har et udsnit til mastens fod. 8× M3×19 går ned i tværbjælkernes huller med møtrik indeni.
- **Tjek:** ingen kollisioner, intet overhang, intet under bunden, alle dele passer på pladen, bolte i forreste venstre kvart går fri.
- **BOM ramme + hjørner + batteri:** M3×12 ×44, M3×19 ×88, M4×12 ×16, M3-møtrikker ×84, M3-skiver ~220, M3-indsatser ×48.
- **Mangler:** beslag til skallen (outriggers), sensorholdere i kofangerne, M2-møtrikker under LiDAR-pladen (huller med kegle er lavet), motorlås, udhuling af pods, STL. Bakke-koden er erstattet; `build_battery` og `build_chassis` er de aktuelle.
- v3.1 (2026-10-06): **alle huller er runde** (`Geo.ROUND = True`; Mikkel vil ikke have dråbeform). Overhang-tjekket springer nu runde huller op til Ø18 på tværs af printretningen over (de printer fint med bro). Spørgsmål fra Mikkel om printede bolte, fordi han mangler M3: se SESSIONS.
- v3.2 (2026-10-06):
  - Bjælkerne flyttet til |x| 196–256, og hjørnernes indsatser ligger som et 2×2-mønster centreret om bjælkens midte (±15 mm). Alle boltrækker er centreret i pladehøjden (`rows()`).
  - **Kvadrathul 20×20** (`Frame_Receiver`) gennem begge vægge i midten af for- og bagkofanger + Ø5,2 lodret låsehul (som et anhængertræk, til pinde/tilbehør).
  - LiDAR-møtrikhullerne går kun op i tårnets top.
  - Overhang-tjekket godtager flade broer op til 20,6 mm.
- v3.3 (2026-10-06):
  - Kvadrathul og låsehul fjernet. I stedet har kofangerne diamanthuller ved x = ±60 på linje med tværbjælkernes, så **en pind/stang kan stikkes hele vejen fra for til bag**. Generatoren tjekker, at kanalen (diamant, halv diagonal 9 mm, z ≈ 51) er fri for alt.
  - **Printede M8×1,25-bolte** i alle bjælkesamlinger, i mastens fod og i batteripladen. Mikkels gevindsæt har tappe og snitbakker M3–M12. Bolten er en Ø7,85-stang med D-D-flader (printes liggende, han skærer gevindet med snitbakke). Møtrikken har et Ø7,0-hul (tappes). Printede skiver Ø16 × 2, frigangshul Ø8,8. Én længde til alt: P-M8×26. Konstanter i `PM8` i generatoren.
  - Behov: 60 printede bolte + 60 møtrikker + 120 skiver. Købt hardware: kun 40 M3 (32 ind i hjørnerne, 12 eller 19 mm, insert-hullerne er 13,5 mm dybe; 8 M3×19 til lejeholderne) = præcis hans sæt (20+20) + 16 M4×12 til motorkraverne.
  - Hjørnernes indsatser i siderne er reduceret til 2×2 pr. side (bjælke- og kofangersamling).
  - Testprint: `build_test_print()` → "93 Test Print" + STL i `print/skelet_test/` (et bjælkestykke + en tværbjælkeende skåret ud af de rigtige dele, bolt, møtrik, skiver) med LÆS MIG.

### Body v0 (2026-10-06) — "for sjov skyld"
`build_body()` → "50 Shell" (én visuel krop, ikke delt i printpaneler endnu).
- Ottekantet set oppefra (`Shell_Corner` 70: ToF på de skrå hjørner), sider og bag affaset mod toppen (`Shell_Chamfer` 35), skrå front fra z 140 til toppen over 150 mm (`Shell_FrontZ`, `Shell_SlopeY`), flad top z = 250 (`Shell_Top`), væg 3 mm. Sidder på rammen (z 73,6). Kofangerne ses under den, som på en rigtig bil, hvor karrosseriet sidder oven på chassiset.
- Udskæringer: hjulkasser R62, LiDAR-hul i toppen (mastens top stikker op), 7" skærm (156 × 88 vindue) + kamera Ø14 i den skrå front, forlygter/baglygter Ø22, 8 ToF-vinduer Ø10 lavt (N/S/Ø/V + 4 skrå hjørner), riste til 2 indsugningsblæsere + 2 højttalere i siderne og 2 udsugningsblæsere bagpå, nødstop Ø22 i toppen bagest til venstre, trekantsgitter (3 ribbefamilier 0/60/120°) i toppen på begge sider af LiDAR'en.
- Tjek: ingen kollision med ramme, hjul, motorer, mast og batteri. Skal 650 × 800 × 176, ca. 2,8 kg PETG massivt. LiDAR-margen 17 mm ved 2° hældning.
- **Fejl rettet undervejs:** `BIG` (størrelsen på hjælpekasserne til halvrums-snit) var 1000 mm, og det var for lidt til skrå planer på hele robotten. Nu er den 4000. Rammen er genbygget efter rettelsen, og tjekkene er uændrede.
- Mangler: opdeling i printbare paneler (≤ 220) med samlinger, beslag til rammen (og hurtig af/på, så skallen kan skiftes), holdere indvendigt til skærm/blæsere/højttalere/ToF/LED, kantsensorer (sidder i rammens kofangerhjørner), farver/lygteglas.
