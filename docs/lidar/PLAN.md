# LiDAR-stack v2 — plan

Status (2026-10-12): **godkendt og v1 bygget** i Rust (`lidar/`, se `lidar/README.md`). Alle fire
skridt fra runde 2 er lavet og testet mod optagelser + simulator; resultater i §10 og `BENCHMARK.md`.
**Ikke** testet på den rigtige LiDAR eller Pi'en endnu.
Gammel app (`Lidar Test Enviornment/`) bruges kun som inspiration og testdata.

---

## 1. Status: hvad den gamle app gjorde, og hvorfor den hakkede

Den gamle Electron + Next.js-app (Gemini, ~3300 linjer JS) kørte: serielparser → sweep-buffer →
PreFilter → Downsampler → OutlierRemoval → ICP → "SensorFusion" (stub) → WallExtractor/WallTracker →
OccupancyGrid → MovingObjectDetector → IPC til React-canvas.

**Måling på optagelserne** (3 filer, 122 sweeps ≈ 12 s, alle statiske, LiDAR'en stod stille på et bord;
median-afstand ~30 cm, maks ~2 m):

| Måling | Resultat | Betydning |
|---|---|---|
| Støj pr. grad over tid | median **1,5 mm** STD, p90 5 mm | Sensoren er rolig. Hakkeriet kom fra softwaren |
| Punkter pr. sweep | skifter **492 / 504** | Sweeps er skåret på pakkegrænser → sømmen hopper |
| Sweep-start | 7,85° → 1,52°, så 2,25° → 5,19° | Hver anden sweep dækker >360° (dobbelte punkter ved sømmen) |
| Rate | 10,0 Hz, 3600 °/s, ~5040 pkt/s | Som databladet |
| Fil 3 | p90-STD 409 mm i nogle vinkler | Noget bevægede sig (god test af "bevægelige objekter") |

**Rodårsager til jitter og bugs (fundet i koden):**

1. **Ingen ordentlig scan-samling.** Ny sweep startes, når en *pakkes* startvinkel falder >50°, så
   sweeps overlapper/mangler et par grader skiftevis. LDROBOTs eget SDK splitter på *punkt*-niveau ved 360→0.
2. **Tidsstempler ignoreres.** Pakkens ms-timestamp (wrap ved 30000) bruges ikke; ingen modtagetid.
   Uden tid kan man ikke de-skew'e (rette for robottens bevægelse under de 100 ms en omgang tager)
   eller fusionere med IMU.
3. **Kortet ignorerer positionen.** `OccupancyGrid.update()` får en pose, men bruger den ikke
   (kommentar: "Pipeline will pre-transform" — det gør den aldrig). Så snart robotten flytter sig, bliver kortet forkert.
4. **ICP er frame-til-frame, punkt-til-punkt, uden startgæt** (og slået fra som standard). Fejl summeres
   op (drift), og der er ikke noget kort at matche mod.
5. **Vægge udledes fra en polær profil i sensorens egen ramme** med EMA-udglatning → virker kun når
   robotten står stille, og EMA giver forsinkelse/"svømmen". Derefter *endnu* et lag EMA i WallTracker.
6. **Behandler kun hver 2. sweep** (`updateRate: 2`) og sender hele kortet som JS-objekter over IPC
   hver gang → tungt og hakkende i UI.
7. **Spejlvending sandsynlig:** LD19-vinkler stiger *med uret* (venstrehåndet system, manual §4), men
   koden bruger `cos/sin(angle − 90°)` som om de steg mod uret.
8. Mange tuning-lag (AutoTuner, CalibrationRunner) der justerer parametre mens den kører → ikke deterministisk.
9. Electron + Next.js dev-server: ~300–500 MB RAM bare for UI'en. For tungt til Pi'en.

**Lektioner til v2:** samle scans korrekt, stemple alt med tid, ét fælles koordinatsystem (robotramme →
verdensramme via pose), match mod et kort (ikke kun forrige scan), ingen magiske EMA-lag, deterministisk
og testbar, UI'en adskilt fra behandlingen.

## 2. Fakta om sensoren (datablad + manual i `components/datasheets/`)

- UART 230400 8N1, kun TX (envejs). 3,3 V logik. ~420 pakker/s × 47 B ≈ **20 kB/s** (trivielt).
- Pakke: `0x54 0x2C`, speed (°/s), startvinkel (0,01°), 12 × (afstand mm u16, intensitet u8),
  slutvinkel, timestamp (ms, wrap 30000), CRC8 (poly 0x4D, tabel i manualen).
- Vinkel pr. punkt = lineær interpolation start→slut, med wrap (`(end + 36000 − start) % 36000`).
- 10 Hz (6–13 Hz via PWM), 5000 målinger/s (STL-19P) → ~500 punkter/omgang, 0,72° opløsning.
- Rækkevidde 0,03–12 m (hvid), 8 m (sort). Nøjagtighed: STD 2 mm < 0,5 m, 4 mm < 2 m, 15 mm > 2 m.
- Pitch-fejl op til 2° → ved 3 m rammer strålen ±10 cm i højden (relevant for lave forhindringer).
- Venstrehåndet: 0° = sensorens front, vinkel stiger med uret. Vi konverterer til ROS-konvention
  (x frem, y venstre, vinkel mod uret) én gang i parseren.
- LDROBOTs SDK har et "near filter" til LD19 (zero'er små grupper < 5 m med lav intensitet) — god idé at efterligne.

## 3. Research — kort

**Scan matching / odometri.** Robotten har bevidst **ingen hjul-encodere** (mecanum glider alligevel),
så LiDAR (+ MPU-6050-gyro) er eneste kilde til odometri. Det gør scan matching til kernen.
- *Punkt-til-punkt ICP* (gammel app): simpel, men konvergerer langsomt og fejler ved store rotationer.
- *Punkt-til-linje ICP (PL-ICP, Censi 2008)*: konvergerer kvadratisk på indendørs vægge, standarden i CSM.
- *Korrelativ scan matching (Olson 2009, bruges i Cartographer/Karto)*: brute-force søgning i et
  sandsynlighedskort på grov+fin opløsning → robust mod dårligt startgæt. God til at "fange" før ICP finpudser.
- *Scan-til-kort i stedet for scan-til-scan* (Hector SLAM-princippet): matcher mod et lokalt kort → meget mindre drift.
- *Fuld SLAM med loop closure* (slam_toolbox, Cartographer): nødvendigt for store kort; overkill til v1 af et hjem/værksted.
- KISS-ICP er bygget til 3D og ikke oplagt til 2D.

**Kort:** log-odds occupancy grid (int8/float32 NumPy-array, fx 5 cm celler, 20×20 m = 160 k celler = 160 kB).
Ray-casting vektoriseret i NumPy eller via Bresenham i Numba.

**ROS 2 eller ej:** ROS 2 Jazzy kører på Pi 5 med slam_toolbox (LDROBOT har en officiel ROS 2-driver),
men en IEEE-sammenligning (2026, Pi 5) og forumindlæg viser at det kan køre, ikke at det er let:
DDS, colcon, launch-filer, TF-træ, Ubuntu i stedet for Raspberry Pi OS. Projektet har allerede besluttet
"Python fra bunden — INGEN ROS2" (PROJECT_OVERVIEW §3).

**Sprog:** Datamængden er lille (500 punkter × 10 Hz). Python + NumPy er rigeligt, hvis man arbejder
på hele arrays og undgår Python-løkker pr. punkt. Måling i denne session: KD-træ (SciPy) bygget +
opslag for 500 punkter ≈ 0,5 ms på en x86-sky-CPU; Pi 5 (Cortex-A76, ~1,5–2× Pi 4) forventes ~1–1,5 ms.
Rust/C++ giver 10–50× på løkker, men koster byggetid og læringskurve. Begge kan kombineres: Python
overalt, og kun en hot loop flyttes ud, hvis målinger viser behov.

**Visualisering:**
- *Egen web-UI serveret fra Pi'en* (én HTML-fil + Canvas/WebGL, WebSocket med binære frames):
  ~0 % CPU på Pi'en udover at sende ~10 kB/frame; virker fra PC, mobil og den 7" skærm (Chromium kiosk).
- *Foxglove* (`foxglove-sdk` 0.29, har aarch64-wheels, Python ≥ 3.10): fremragende dev-værktøj med
  plot, 3D, optagelse til MCAP, uden ROS. Kræver Foxglove-appen på PC'en.
- *Rerun*: lækkert til udvikling, men tungere viewer og mindre egnet til at køre på robottens skærm.

**Budget på Pi 5** (4× A76 @ 2,4 GHz; RAM ukendt, 4 eller 8 GB). LiDAR-stacken skal dele maskinen
med kamera/AI, motorkontrol-link og UI, så mål:

| Mål | Krav |
|---|---|
| Behandling pr. omgang (median / p99) | ≤ 10 ms / ≤ 25 ms (dvs. ≤ 10–25 % af én kerne) |
| Latens: sidste punkt i omgang → resultat sendt | ≤ 30 ms p99 |
| Hindringsdetektion pr. pakke (ikke pr. omgang) | ≤ 2 ms |
| RAM (RSS) | ≤ 150 MB |
| Stabilitet, statisk robot | pose-jitter ≤ 5 mm / 0,2°; vægge flytter sig ≤ 1 cm frame til frame |

## 4. Arkitekturforslag

Fælles for alle: sensoren sidder bag et `ScanSource`-interface med tre implementationer —
`SerialSource` (rigtig LiDAR), `ReplaySource` (optagelser, inkl. de gamle JSONL) og `SimSource`
(syntetisk rum + kendt kørselsbane, genererer *rå LD19-bytes*, så parseren også testes). Ny optager
gemmer **rå bytes + modtagetid** i et kompakt binært format, så fremtidige optagelser kan afspilles bit for bit.

```
 ScanSource ──bytes──▶ Parser (CRC, vinkler, tid) ──pakker──▶ ┬─▶ Sikkerhedszoner (pr. pakke, hurtigt)
 (Serial/Replay/Sim)                                          └─▶ Scan-samler (360°, punkt-niveau, deskew)
                                                                        │ Scan (NumPy, robotramme)
                                                                        ▼
                                                       Filter (near-filter, intensitet, egen krop)
                                                                        ▼
                                                 Odometri: korrelativ → PL-ICP mod lokalt kort (+ gyro)
                                                                        ▼
                                                   Occupancy grid ─▶ (senere) loop closure / lokalisering
                                                                        ▼
                                              Publisher ──WebSocket (binær)──▶ Web-UI / Foxglove / optager
```

### Mulighed A — Python + NumPy, én proces, egen web-UI (**anbefalet**)
- `asyncio`-proces: læser seriel i en tråd, behandling i hovedløkken, `aiohttp`/`websockets` til UI.
- Afhængigheder: `numpy`, `scipy` (KD-træ), `pyserial`, `websockets`. Valgfrit `numba` til ray-casting.
- UI: én statisk HTML/JS-fil (ingen build-step), Canvas 2D: punkter, kort, pose, bane, zoner, diagnostik.
- Foxglove-bro som valgfrit modul til udvikling (samme data, andre kanaler).
- **+** Matcher projektbeslutningen (Python, ingen ROS), let at læse/ændre for Mikkel, samme kode på PC og Pi,
  tests i `pytest`. **−** Python-løkker er langsomme → kræver disciplin (vektorisering). GIL: én kerne.

### Mulighed B — Rust-kerne + web-UI
- Én statisk binær (`serialport`, `nalgebra`, `tokio-tungstenite`), krydskompileret til aarch64.
- **+** Hurtigst og mindst (≈ 5–10 MB RAM, < 2 % CPU), ingen GIL, robust.
  **−** Ny sprogverden, længere iterationer, sværere at eksperimentere med algoritmer; resten af robotten er Python.

### Mulighed C — ROS 2 Jazzy + ldlidar_stl_ros2 + slam_toolbox + Foxglove
- **+** Gennemprøvet SLAM med loop closure "gratis", RViz/Foxglove, Nav2 senere.
  **−** Strider mod den eksisterende beslutning, tungt (Ubuntu, DDS, ~1 GB+ RAM i alt), meget "magi",
  sværere at forstå hvorfor ting går galt — og læringen ("bygget fra bunden") forsvinder.

### Anbefaling
**A**, med klare interfaces så en hot loop kan flyttes til Numba/Rust, *hvis* benchmarks på Pi'en
viser behov. Det er det hurtigste sted at nå "stabilt og målbart", og det passer til resten af robotten.

## 5. Byggerækkefølge (milepæle, hver med tests)

| # | Milepæl | "Færdig" betyder |
|---|---|---|
| M0 | Repo-skelet, `ScanSource`, parser, CRC, binært optagelsesformat, JSONL-import, simulator | Parser-tests mod manualens eksempelpakke; sim → parser → samme punkter |
| M1 | Scan-samler + filtre + ramme-konvertering | Altid ét scan pr. omgang, ingen overlap/huller ved sømmen; statisk jitter målt |
| M2 | Web-UI (live punkter + diagnostik) | Kører på PC mod replay; < 5 % CPU for server-delen |
| M3 | Sikkerhedszoner / hindringer | Latens pr. pakke målt; korrekte zoner i sim |
| M4 | Odometri (scan-til-lokalt-kort, korrelativ + PL-ICP) | Sim: drift < 2 % af kørt distance, < 1°/omgang rundt; statisk: jitter < 5 mm |
| M5 | Occupancy grid + kort i UI | Kort i sim matcher ground truth (IoU), stabilt i statisk replay |
| M6 | Bevægelige objekter, gem/indlæs kort, lokalisering, loop closure | Efter Mikkels prioritering |

Benchmark-script (`bench`) kører alle optagelser + sim og rapporterer median/p99-latens, CPU-tid og RSS
— så "hurtigt og stabilt" er målt, ikke påstået. Kan køres på Pi'en uændret.

## 6. Hvad er Foxglove, og hvor passer det ind?

Foxglove er et gratis desktop-/browserprogram til at *kigge på robotdata*: man forbinder til en
WebSocket på robotten og får paneler med 2D/3D-visning af punktskyer og kort, grafer over tid
(fx latens, gyro), tabeller med rå beskeder og afspilning af optagelser (MCAP-filer) med tidslinje.
Det er lavet til ROS, men virker uden ROS: programmet på Pi'en sender bare data i Foxgloves format.

Her ville det være et **udviklerværktøj på PC'en**, ikke robottens UI: når scan matching opfører sig
mærkeligt, kan man spole frem og tilbage og plotte fejl over tid uden at vi selv skal bygge de værktøjer.
Robottens egen visning (browser + 7" skærm) bygger vi selv, let og specialiseret. Foxglove-broen er
valgfri og kan komme senere (M2+); der findes en Rust-version af `foxglove-sdk`.

## 7. Beslutninger

### Runde 1 (2026-10-12)
- **Sprog:** helst **ikke Python**. Hastighed/CPU er vigtigst, fordi Pi'en også skal køre kamera/AI og andet.
  C eller Rust foretrækkes; valget er overladt til Claude. (Mega-link-sessionen udskød Pi-sproget til
  LiDAR-beslutningen og lavede en sprogneutral C99-protokolkerne.)
  → Mulighed A (Python) er **fravalgt**. Ny anbefaling: **Mulighed B, Rust** (begrundelse i §8).
- **Første funktion:** forhindringer/sikkerhedszoner.
- **UI:** browser via WiFi + robottens 7" skærm (samme webside). Foxglove forklaret i §6.
- **"Det virker" =** stabilt billede når robotten står stille, kort der holder når den kører,
  og robotten kan styres ud fra det (data videre til motorstyringen).
  Bemærk: "kort der holder" kræver odometri + occupancy grid (M4–M5), så de hører med i v1 —
  bare efter sikkerhedszonerne.

## 8. Rust frem for C — begrundelse

| | Rust | C |
|---|---|---|
| Hastighed | Samme som C (LLVM, ingen GC) | Samme |
| Hukommelsesfejl (buffer overflow, use-after-free, data races) | Fanges af compileren | Fanges først når robotten crasher |
| Tråde (seriel-læser, behandling, web-server) | Sikkert via typesystemet | Manuelt med mutex; klassisk fejlkilde |
| Web-server + WebSocket til UI | Færdige biblioteker (`tungstenite`, `tiny_http`) | Skal bindes til fx mongoose/libwebsockets |
| Tests + benchmarks | Indbygget (`cargo test`, `cargo bench`) | Selvbygget / eksternt framework |
| Krydskompilering til Pi 5 (aarch64) | `cargo build --target aarch64-unknown-linux-gnu` eller byg direkte på Pi'en | gcc-cross, Makefile |
| Genbrug af Mega-linkets C99-kerne | Via FFI (`cc`-crate) eller ~100 linjer Rust-port af COBS+CRC-16 | Direkte |
| Læringskurve | Stejl (ejerskab/borrow checker) | Lav at starte, høj at gøre robust |

Konklusion: Rust giver C's fart uden C's crash-klasser, og har det økosystem en web-UI og tests kræver.
Ulempen er læringskurven. C-kernen fra Mega-linket kan bruges som den er via FFI.

**Ærlig note om kamera-AI:** Raspberry Pi AI Camera (IMX500) kører det neurale netværk *på selve
kamerasensoren*; Pi'en modtager kun resultaterne. Ansigtsgenkendelse koster derfor langt mindre CPU
på Pi'en end man skulle tro — men det ændrer ikke valget, fordi Rust også er det bedste til resten.

### Runde 2 (2026-10-12) — plan godkendt
- **Sprog: Rust** for hele Pi-applikationen (Mega-linkets C99-kerne kan bruges via FFI eller porteres).
- **Hindringer:** Pi'en sender "nærmeste afstand pr. sektor" til Megaen (~20–50 Hz); Megaen begrænser
  fart/stopper sammen med ToF (refleks-arkitekturen i `docs/electronics/PI_MEGA_LINK.md`, branch
  `claude/stoic-cray-tsomgg`). I v1 ligger zonerne bag et interface; selve link-indkodningen laves, når linket er bygget.
- **Rækkefølge godkendt:** (1) parser + scan-samling + simulator + tests → (2) web-UI med stabilt
  live-billede → (3) sikkerhedszoner → (4) LiDAR-odometri + kort der holder når robotten kører.
- **Tilslutning:** USB-adapteren fra kittet (CP2102), fundet via `/dev/serial/by-id/…`; porten er en indstilling.

## 9. Endelig arkitektur (v1)

Én Rust-binær `sennep-lidar` (mappe `lidar/`), ingen runtime-afhængigheder:

| Modul | Ansvar |
|---|---|
| `protocol` | Streaming-parser for LD19-pakker: resync, CRC8, vinkel-wrap, ingen allokering pr. pakke |
| `source` | `ByteSource`-trait: `SerialSource`, `ReplaySource` (rå `.ldr` + gamle `.jsonl`, genindkodet til pakker), `SimSource` |
| `clock` | Afbilder LiDAR'ens ms-ur (wrap 30000) til værtens monotone tid |
| `scan` | Scan-samler på punkt-niveau (split ved 360→0), robotramme (x frem, y venstre, mod uret), tid pr. punkt |
| `filter` | Rækkevidde, LDROBOT-lignende near-filter, maske for robottens egen krop |
| `zones` | Sektorafstande fra robottens kant (rektangel), opdateret **pr. pakke** |
| `odom` | Scan-til-lokalt-kort: konstant-hastigheds-prædiktion → de-skew → punkt-til-linje ICP (Gauss-Newton, Huber) mod keyframes |
| `grid` | Log-odds occupancy grid (5 cm), ray-casting, beskidte fliser sendes inkrementelt |
| `sim` | Syntetisk rum + bane + støj fra databladet → **rå LD19-bytes** (tester hele kæden) |
| `server` | HTTP (én statisk side) + WebSocket med binære frames; klienter får altid nyeste frame (ingen kø) |
| `bench` | Afspiller optagelser + sim, måler latens, CPU, RSS, jitter, drift, kort-IoU → `docs/lidar/BENCHMARK.md` |

## 10. Status efter v1 (2026-10-12)

Målt i skyen (x86, ~1,5–2,5× hurtigere pr. kerne end Pi 5) — fuld tabel i `BENCHMARK.md`:

| Mål (§3) | Krav | Målt |
|---|---|---|
| Behandling pr. omgang, median / p99 | ≤ 10 / ≤ 25 ms | 2,9 / 7,2 ms (Pi-estimat ~5–7 / ~15 ms) |
| Zoner pr. pakke | ≤ 2 ms | < 1 µs median, p99 2 µs |
| CPU | ≤ 10–25 % af én kerne | 2–3,4 % (inkl. web-server: 5–6 % med sim) |
| RAM | ≤ 150 MB | ~6 MB for hele programmet |
| Statisk jitter | ≤ 5 mm / 0,2° | 0,25–0,7 mm / 0,07–0,09° (optagelser), 1,4 mm / 0,04° (sim, 15 mm støj) |
| Kort der holder | — | 8,7 m tur: 0,02 % drift, 99 % præcision; 173 m (11 min): 2,6 mm / 0,01° tilbage ved start, 92 % præcision |
| Zoner korrekte | — | 0 optimistiske fejl i 4800 sektorværdier med gående person |

Vigtige fund undervejs: sensoren overlapper af og til to pakker med 0,2° (også ved sømmen) — ufarligt;
tynde ting langt væk rammes ikke hver omgang (fysik, ikke bug); en zone er op til én omdrejning (100 ms)
gammel, hvilket ved rotation betyder at en ting kan "skifte sektor" mellem to passager; tykke vægge kræver
at ICP afviser bagsiden (normaler orienteret mod sensoren); keyframes skal genbruges når robotten kommer
tilbage, ellers driver kursen ~4° på 11 minutter.

**Næste skridt (forslag):**
1. Mikkel: kør `sennep-lidar --serial …` + `bench` på Pi'en med den rigtige LiDAR; optag et par ture med `--record`.
2. Tjek monteringen: at LiDAR'ens 0° peger fremad (ellers `Mount.yaw`) og at robottens omrids passer.
3. Send `ZoneReport` over Pi↔Mega-linket, når det er bygget (`ObstacleSink`).
4. Flet gyroen (MPU-6050 via Megaen) ind i prædiktionen → robust ved hurtige drej.
5. Gem/indlæs kort + lokalisering i et gemt kort; derefter bevægelige objekter.
