# Sessions

## 2026-10-04 — Claude Code (Opus 5.5)
**Summary:** Rekonstruerede projektideen. Læste Gemini-share-linket (hele, ~110k tegn), `samtale-2.pdf` (motor/driver, 11 sider, billed-PDF), tre kvitteringer, `Cut noter.txt` og skimmede `Lidar Test Enviornment/`. Resultat i `PROJECT_OVERVIEW.md`.
**Issues:** `Samtale.pdf` er samme samtale som share-linket (billed-PDF, ikke læst separat). ChatGPT-projektet findes ikke i mappen. Mappen er ikke et git-repo. LiDAR-testappens `next build` fejler på en typefejl (`onCalibrationProgress` mangler i preload-typerne).
**Next:** Afklar Pi 4/5 + hjulstatus; byg "rullende skelet" (bundramme + motorer + hjul + batteri + Nano + L298N + RF-joystick) før karosseri; derefter strøm-/sensor-hub-design.

## 2026-10-04 (forts.) — Claude Code (Opus 5.5)
**Summary:** Gennemgik Gmail for AliExpress: 18 ordrer fra 17.–22. dec 2025, ~883 kr (tabel i PROJECT_OVERVIEW.md §4). Ordren løser flere tidligere flag (motordrivere, multiplexer, relæ til nødstop, INA219). Pi 5 bekræftet, hjul printet/testet, 100 % print foretrukket. Fandt officiel Autodesk Fusion MCP (Fusion → Preferences → General → API → Fusion MCP Server).
**Issues:** ChatGPT-projektlinket kræver login (ikke læst). Fusion-modellen ikke delt endnu.
**Next:** Forbind Fusion MCP eller eksportér STEP; design rullende skelet (bundramme i printbare segmenter).

## 2026-10-05 — Claude Code (Opus 5.5)
**Summary:** Læste hele `gpttxt.txt` (1950 linjer, ChatGPT apr–maj 2025) → tidslinje i PROJECT_OVERVIEW.md §7b. Registrerede Fusion MCP lokalt (`claude mcp add --transport http fusion http://127.0.0.1:27182/mcp`, local scope i ~/.claude.json). Hjul/lejer afklaret (pillow blocks bærer vægten).
**Issues:** Mange brugerbeskeder i gpttxt refererer til billeder, der ikke er med (afventer zip). Vandsikringskrav fra apr 2025 konflikter med luftindtag fra maj 2025.
**Next:** Ny session → test Fusion MCP-forbindelse, læs modellen, planlæg krop + sensorplacering.

## 2026-10-05 (forts.) — Claude Code (Opus 5.5)
**Summary:** Fusion MCP virker via claude.ai-connectoren (Autodesk_Fusion). Læste "Robot v58" read-only: skal 500×816×200, Ø130 hjul, kun ét hjørnemodul modelleret. Resultat i PROJECT_OVERVIEW.md §7c. Afklaret: ikke vandtæt, GPS med, batteriserier ikke forbundet.
**Issues:** For at se ind i modellen blev Body422/433/434 skjult og vist igen; dokumentet står derfor som "modified" uden reelle ændringer (ingen grund til at gemme). Fusion-scripts skal definere `run(context)`.
**Next:** Afklar målstørrelse; spejl hjørnemodulet ×4; del skal + bundplade i printbare paneler; placér komponenter.

## 2026-10-05 (forts. 2) — Claude Code (Opus 5.5)
**Summary:** Satte nyt Fusion-dokument "SENNEP V2" op (parametre, komponentstruktur, layout-skitse, Drive Module ×4 importeret fra Robot v59). Gemt som v1/v2. Detaljer i PROJECT_OVERVIEW.md §9. Fandt motorens datablad (§7d).
**Issues:** Joints til layout-punkter fejlede → positioner sat med transformationer. Spejlvendte hjul til FR/RL mangler.
**Next:** Spejlet hjulvariant; stigeramme i 20 Chassis efter parametre; stålaksel-beslutning; elektronik-pladsholdere.

## 2026-10-05 (forts. 3) — Claude Code (Opus 5.5)
**Summary:** Hjulgenerator v2.0→v2.6.1 (100 % printet hjul): printede pinde, aksel, afstandsrør, finish, Mikkels egne features indarbejdet fra tidslinjen, helix-riller mellem fælgene, asymmetriske pinde (fundet via automatisk samletest), bajonet på afstandsrør, skrue-joint med motion link. STL til testprint eksporteret. Detaljer: PROJECT_OVERVIEW.md §11–12.
**Issues:** Nested occurrence-transforms upålidelige via API → ruller som bodies. Bash-heredoc mangler `\n`-escapes → patches skrevet som filer i scratchpad.
**Next:** Testprint → justér frigange; venstre/højre hjul i drivmodulet; hjulkasse.

## 2026-10-05 (forts. 4) — Claude Code (Opus 5.5)
**Summary:** Testprint af v2.7 evalueret (5 problemer) → hjulgenerator v3: printbar bajonet, korte pinde + trykring, strammere pasninger, Ø21,5-ruller der virker uden sleeve, hurtigtest-dele. Detaljer: PROJECT_OVERVIEW.md §12 "v3". STL i print/testhjul_v3. Fusion v14.
**Issues:** Fusion MCP timer ud ved to builds i ét kald → én build pr. kald. Større ruller end Ø22 er umulige ved 9 ruller/Ø106.
**Next:** Mikkel printer hurtigtest → justér pasninger → helt hjul. Derefter venstre/højre hjul i drivmodulet og hjulkasse.

## 2026-10-05 (forts. 5) — Claude Code (Opus 5.5)
**Summary:** Bajonetten er lavet om efter Mikkels kritik: én 45°-profil med trekantede tapper i stedet for lappeløsninger. Ingen overhang er verificeret med et automatisk tjek af face-normaler. STL re-eksporteret, Fusion v15.
**Lesson:** Design hele profilen som én linje fra start. Tjek altid printorienteringen (hvilken flade ligger på pladen), før der tilføjes "printbar" geometri.

## 2026-10-05 (forts. 6) — Claude Code (Opus 5.5)
**Summary:** Akslen: pinolskruen sidder nu i en indfanget M3-møtrik i en lomme, der er åben mod koblingsenden. Ingen printpause, 1,7 mm væg udenfor møtrikken. Gevind skåret i plast (M5) flækkede akslen langs laget, og en M4-møtrik er der ikke plads til. Fusion v16, STL re-eksporteret. Skrev `HANDOFF_CHASSIS.md`: prompt til en ny chat om skelettet (stel, motor- og lejeophæng) med grænsefladen til hjulet og alle print-erfaringer.
**Next:** Mikkel printer hurtigtest + aksel. Skelettet laves i en separat chat. Hjulchatten fortsætter med pasninger og venstre/højre hjul.
- Rettelse (v17): væggen på 0,4 mm mellem møtriklommen og D-hullet er fjernet. Møtrikken ligger nu i flugt med D-fladen; den er 5,5 mm bred mod en åbning på 4 mm, så den kan ikke falde ind i hullet. Væggen udenfor er nu 2,1 mm. Koblingen kan ikke gøres tykkere, fordi den skal gennem lejet (maks. Ø16,9).
- v18: møtriklommen har nu lige vægge helt ned til hullet (ingen knivtynde trekanter). Møtrikken kan derfor glide ind i hullet, før motorakslen sidder i: skru pinolskruen et par omgange i møtrikken først, så holder den møtrikken.

## 2026-10-05 (forts. 7) — Claude Code (Opus 5.5), skelet-chat
**Summary:** Læste HANDOFF_CHASSIS og lavede udkast 1 til et parametrisk skelet (`docs/skelet_udkast_v1.html`, artifact). Intet modelleret i Fusion; kun read-only-scripts (parametre, gamle KFL003/motor-mål). Motordatablad s. 4 aflæst (4× M4 på PCD 35, tap Ø25). Detaljer: PROJECT_OVERVIEW §13.
**Issues:** Ved RobotWidth 500 overlapper venstre/højre motor (−11,5 mm) → anbefalet 540. KFL003 er selvjusterende, så momentet fra hjulet kan ende i motorakslen; skal testes. Wheel_Width-parameter findes ikke (kun WheelWidth 55, forældet).
**Next:** Mikkel: bredde, KFL003-mål, terminallængde, batterimål, lejetest, ok til flydende motor. Derefter generator v0 med ét pod.

## 2026-10-05 (forts. 8) — Claude Code (Opus 5.5), skelet-chat
**Summary:** Mikkel valgte 800 × 650 og 220 mm printgrænse. Skrev `SENNEP_ChassisGenerator` v0 og byggede ét hjørne ("21 Pod Lab") om det genererede hjul med de rigtige motor- og lejemodeller. Alle tjek er grønne. Fusion gemt. Skærmbilleder i `docs/skelet/`. Detaljer: PROJECT_OVERVIEW §13 "Skelet v0".
**Issues:** Proxy-transforms blev nulstillet af en parameterændring sidst i scriptet (løst). Gearkassens diameter er uenig mellem model (41,5) og datablad (42,4).
**Next:** Mikkel ser hjørnet i 3D og måler gearkassen. Derefter: 4 spejlede pods, skinner/gulv, udhuling, STL + testprint af ét hjørne.

## 2026-10-05 (forts. 9) — Claude Code (Opus 5.5), skelet-chat
**Summary:** Skelet v1: rund hjulkasse, rundt skruetrækkerhul, gearkasse Ø42,4 (datablad), dråbehuller overalt på tværs. Ny `build_chassis()` laver hele det rullende skelet (4 pods + 12 bundbakker) i "20 Chassis". Alle tjek er grønne. Fusion gemt. Skærmbilleder i `docs/skelet/`. Detaljer: PROJECT_OVERVIEW §13 "Skelet v1".
**Issues:** `SurfaceEvaluator` har ingen `getParameterExtents` → brug `parametricRange()`. Kameraets `viewExtents` opfører sig mærkeligt efter et ortho-kamera; `isFitView = True` virker.
**Next:** Mikkel vurderer. Derefter udhuling af pods, lukning af gulvhullerne, STL + testprint af ét hjørne + én bakke.

## 2026-10-06 — Claude Code (Opus 5.5), skelet-chat
**Summary:** Rettede to fejl, som Mikkel fandt (poden lukkede sig under hjulet; forkerte dråbehuller). Tilføjede hardwaremodeller (sæt M2–M4), batterimodul (celler + holdere), batteriplade + klemmer i forreste midterbakke, bolteksempler og BOM. Fusion gemt. Billeder i `docs/skelet/v2_*`. Detaljer: PROJECT_OVERVIEW §13 "Skelet v2".
**Issues:** Udseender forsvandt ved genberegning (sæt dem inde i base feature-edit). Låseskruen er fjernet. BOM: 144 M3×12 mod 20 i sættet.
**Next:** Mikkel beslutter hardware (køb flere M3, eller færre bolte pr. samling). Bedre motorlås. Udhule pods. STL + testprint af hjørne + bakke.

## 2026-10-06 (forts.) — Claude Code (Opus 5.5), skelet-chat
**Summary:** Bakkerne er erstattet af en stigeramme efter Mikkels ønske: længdebjælker, 4 tværbjælker, kofangere og LiDAR-mast. Alle samlinger har V-nøgle + bolte. LiDAR-data hentet fra databladet. Batteriet er flyttet op på tværbjælkerne. Alle tjek grønne. Fusion gemt. Billeder: `docs/skelet/v3_*`. Detaljer: PROJECT_OVERVIEW "Skelet v3".
**Issues:** Det første MCP-kald timede ud uden at køre (gentaget OK). V-nøglernes ender pegede nedad i print → rygge løber til toppen, riller ud i bunden.
**Next:** Mikkel vurderer rammen. Derefter: beslag til skallen, sensorholdere i kofangerne, motorlås, udhuling, STL og testprint af én samling (hjørne + bjælkeende).

## 2026-10-06 (forts. 2) — Claude Code (Opus 5.5), skelet-chat
**Summary:** Alle huller er nu runde (ramme, pod, skruetrækkerhul, klemmer). Pod, ramme og batteri er genbygget, alle tjek grønne, Fusion gemt.
**Open:** Mikkel spørger til printede bolte/møtrikker (han har for få M3). Anbefaling: printede M10-bolte og -møtrikker til rammens bjælkesamlinger (1–2 pr. samling sammen med V-nøglen), M3 kun i hjørnerne og ved lejet. Først et gevind-testprint med 3 frigange. Afventer svar.
- 2026-10-06 (forts. 3): Centrerede hulmønstre, kvadrathul i kofangerne, LiDAR-huller rettet. Alle tjek grønne, Fusion gemt. Mikkel overvejer printede bolte: retningen er bekymringen (vandret printede gevind) og alternativet med at skære gevind selv. Afventer svar om, hvilke tap/snitbakke-størrelser han har.
- 2026-10-06 (forts. 4): Diamantkanal gennem kofangere og tværbjælker (x = ±60, verificeret fri). Printede M8-bolte (gevind skæres af Mikkel) i alle bjælkesamlinger, batteri og mast. M3-behovet er nu 40 = hans sæt. Testprint-STL i print/skelet_test. Alle tjek grønne, Fusion gemt.
  **Next:** Mikkel printer testen (V-nøgle + M8-gevind). Derefter justeres `Frame_KeyClear` og `PM8`, og hele rammen eksporteres.
- 2026-10-06 (forts. 5): Body v0 tegnet (50 Shell): facetteret skal med skærm, gitter, hjulkasser og alle sensor-/lyshuller. `BIG`-fejl i halvrums-snit fundet og rettet (ramme genbygget, uændret). Fusion gemt. Billeder: docs/skelet/body_v0_*.png. **Next:** dele skallen i printpaneler + beslag/hurtigkobling til rammen.

## 2026-10-07 — Claude Code (Opus 5.5)
**Summary:** Byggede `components/`: komponentliste med 85 linjer (`inventory.csv` → `INVENTORY.md` + søgbar `index.html` via `build_inventory.py`), 37 datablade hentet til `components/datasheets/` og ESP-01-guide (`components/esp-01/`: pinout, SVG-diagram til Mega, passthrough- og webserver-sketch, begge kompileret med avr-g++ / AVR-core 1.8.8). Pointer i PROJECT_OVERVIEW §4.
**Issues:** ST/TI blokerer curl for nogle PDF'er → hentet fra SparkFun/Adafruit/Pololu-spejle (7805-databladet er TI µA7800). AliExpress-mails har afkortede varenavne (BTS7960 og IRFZ44N ikke bekræftet). Påstanden "WiFi droppet pga. hardwareproblemer" stammer fra en ChatGPT-opsummering af V1, som Mikkel kaldte forkert. Ingen arduino-cli fundet; sketches er kun kompileret, ikke testet på hardware. Mappen er stadig ikke et git-repo.
**Next:** Udfyld priser/RAM for Pi 5 og LiDAR; tilføj V1-dele der stadig findes (resistorer?); evt. test ESP-01 med passthrough-sketchen.

## 2026-10-11 — Claude Code (Opus 5.5)
**Summary:** Helt testhjul eksporteret til `print/testhjul_v3.1/` fra "11 Wheel Lab" (Fusion "SENNEP V2 v27", parametre = generatorens defaults, generatoren uændret siden v18). Nyt: STL'erne er roteret i printorientering (printflade på Z = 0, centreret) med `orient.py` (scratchpad; roterer om Y så valgt ±X-flade vender ned), så "Lay on face" ikke behøves. Kontaktflader verificeret: Rim A 3065, Rim B 2915, aksel 1193, afstandsrør 165, rulle 80 mm².
**Issues:** Ingen notat om hurtigtest-resultatet — ukendt om pasningerne er bekræftet før det fulde hjul. Mappen er stadig ikke et git-repo (intet commit).
**Next:** Mikkel printer hjulet → justér frigange; venstre/højre hjul i drivmodulet.
- (forts.) Hjulkassen også eksporteret til `print/testhjul_v3.1/hjulkasse/` fra "21 Pod Lab": pod (stående på motorenden, +X ned, 169 mm), motorkrave og lejeholder (−X ned). Poden er stadig massiv (562 cm³).

## 2026-10-12 — Claude Code (Opus 5.5)
**Summary:** Mikkel kunne ikke samle hjulet: rullerne falder af Rim A's korte pinde. Lavede en genbrugelig samlering (`fusion/SENNEP_WheelGenerator/assembly_ring.py` → "11 Wheel Lab Assembly Ring (samlering)", to varianter: med/uden TPU-sleeves). Ringen ligger kun ved r ≥ 50 (fælge maks. 49,6) og x ≥ 0,5 (Rim A-siden af samlingen); vuggerne er rullerne (+0,3 mm radialt) swept langs −X, så hjulet kan løftes ud. Tjek: 0 mm³ mod Rim A/B og under løft 0–20 mm; rulle skubbet 0,4 mm ud rammer vuggen. STL + guide i `print/testhjul_v3.1/samlering/`.
**Issues:** Fusion-MCP-connectoren i sessionen genforbandt ikke, selv om Fusion kørte → kaldt direkte via HTTP (127.0.0.1:27182/mcp) med en lille klient (scratchpad `fx.py`). Swept med 75 enkelt-unions timede ud (>10 min); fordobling (7 unions) + én cutter roteret 9× tog ~1 min. Sleeven er et hult rør → skal carves sammen med kernen. Ren radial forskydning efterlod rullens inderflanke → cutteren = rulle i hvile ∪ forskudt rulle. Body-navne sat inde i base-feature-edit går tabt → navngiv efter finishEdit.
**Next:** Mikkel printer ringen og samler hjulet → pasninger (riller, bajonet, sekskant, ruller).
- (forts.) Samleringen har fået materiale PET Plastic + blå mat appearance (71 / 65 g massiv); STL re-eksporteret, Fusion gemt. Resten af hjulets bodies står stadig som Steel (kun appearance er sat).

## 2026-10-12 (forts.) — Claude Code (Opus 5.5)
**Summary:** Mikkel spurgte, hvordan hjul + leje + pod samles, når lejeholderens skruer skal i fra hjulsiden. Tjek i Fusion: en Ø6-korridor ved r = 28 går fri gennem Rim A, Rim B, ruller, afstandsrør og akselhoved for hjulvinkler 16–44° (gentager hver 60°) → skruerne kan spændes gennem vinduerne; **poden skal ikke laves om/genprintes**. Nyt script `fusion/SENNEP_ChassisGenerator/pod_hardware.py` → "22 Pod Lab Hardware" (2× M3×19+skive+indsats, 4× M4×12+skive, M3×12+møtrik i koblingen) og "23 Pod Lab Samletrin" (skjult: hjul drejet −30° + 2 nøgler). Alle tjek fri (hardware mod plast, pinolskrue drejet 360°, nøgler gennem vinduer). Samlevejledning i `print/testhjul_v3.1/LÆS MIG.md`, billeder `hjulkasse/samling_*.png`.
**Findings:** Koblingsskruen M3×12 skal skrues helt i bund før hjulet skubbes ind (hoved r 11,4 går fri; kun i møtrikken r 20,1 rammer poden), og skrues ~7 mm ud gennem tophullet, før motoren sættes i. M3×12 er kortest i sættet; en kort pinolskrue (M3×5–6) ville være bedre.
**Rejected:** Mikkels idé om lange skruer bagfra: med BoltAngle 0 ligger boltlinjen (y = ±28, z = 0) oven i krave-tappernes kanaler; kræver BoltAngle ≈ 19,5° + styrerør og et nyt print. Unødvendigt, når vinduerne virker.

## 2026-10-12 (forts. 2) — Claude Code (Opus 5.5)
**Summary:** Mikkel ødelagde v1-housingen (boring bagfra) og fandt vindues-metoden for besværlig. "Pod" hedder nu **housing** over for Mikkel. Housing v2 i chassisgeneratoren: lejeholder-skruer bagfra (M3×19+skive, hoved i forsænkning Ø7,6 i lommebunden, 6 mm bund; indsatser i holderens forside; Ø10 værktøjskanal ud gennem motorenden, fri af motoren), pinolskrue nås nedefra (Ø12, `Chassis_ToolDia`), `Chassis_MotorClear` 0,2→0,1, krave-tapper roteret til 90° (`Chassis_LugAngle`) fordi tapkanalerne lå på skruelinjen. Hul variant (`make_hollow`, "21 Pod Lab (hul variant)") — estimeret ~5 g MERE filament (241 vs 236 g ved 3 vægge/15 %), da kammerets vægge printes massivt. `pod_hardware.py` opdateret ("22 Pod Lab Hardware", "23 Pod Lab Skruetraekkere bagfra"). STL i `print/housing_v2/`.
**Issues:** MCP-kald over HTTP timede ud på klientsiden, selv om Fusion gennemførte kaldet (hardware-scriptet) → verificér tilstanden bagefter i stedet for at stole på timeouts. Mikkel så "fejl i Fusion" — sandsynligvis de afbrudte kald; geometrien er verificeret (cylinderradier 3,8/5,0/6,0/21,3 i housing-bodyen). Samleringen talte som kollision i `display_bodies` → filtreret fra.
**Not done:** "20 Chassis" er ikke genbygget med housing v2 (4 hjørner + hardware i "91 Hardware (placed)" bruger stadig gammel lejeholder-placering i build_chassis linje ~1066). Virkelig filamentbesparelse kræver mindre ydre volumen (fx 45°-kilen under motorrøret).

## 2026-10-12 (forts. 3) — Claude Code (Opus 5.5)
**Summary:** Projektet er nu et git-repo (lokalt, `main`, ikke pushet endnu; `.gitignore` udelader node_modules/.next/.env). Seks opgave-chips oprettet til separate sessioner, alle med "status → research → plan → AskUserQuestion med Mikkel → først derefter byg": ny LiDAR-stack fra bunden (den gamle Gemini-app kun som inspiration), Pi↔Mega-link, breadboard-kredsløb, PCB (først efter breadboard), dokumentation + komplet komponentliste (inkl. dele fra andre projekter), 3S6P-batteriet med eksisterende materialer.
**Note:** Fusion kan ikke nås fra cloud-sessioner (MCP kører på 127.0.0.1:27182); en tunnel frarådes (fjernkørsel af vilkårlig Python uden auth).
- (forts.) Offentligt repo: https://github.com/MikkelBlom/SENNEP-V2 (HTTPS). Før push: fakturaer, Gemini-PDF'er, `gpttxt.txt` og `botnroll.txt` holdes lokalt (`.gitignore`: `/*.pdf` i roden), hjemadresse og ordrenumre fjernet fra noter og `inventory.csv` (kolonnen `ordre` er tømt), historikken squashet til ét rent commit. Git-identitet bruger GitHub-noreply-mail. Datablade (90 MB, producenternes PDF'er) ligger i repoet.
- (forts.) Chips genskabt efter GitHub-remote blev sat op (de gamle sagde 'no github remote'). Køen holder maks. 5 chips: LiDAR, Pi↔Mega, breadboard, docs+komponentliste, batteri. **PCB-chippen skal genskabes**, når breadboard-planen (`docs/electronics/BREADBOARD_PLAN.md`) er aftalt.

## 2026-10-12 — Claude Code (Opus 5.5), LiDAR-stack v2 (cloud-session, branch `claude/fervent-tesla-tbrtpz`)
**Summary:** Status → research → plan (`docs/lidar/PLAN.md`) → 2 runder spørgsmål → bygget. Mikkels valg: **Rust** (ikke Python; Pi'en skal også køre kamera/AI), forhindringer først, zoner sendes til Megaen, UI i browser + 7" skærm, LiDAR via USB-adapteren. Bygget `lidar/`: LD19-parser (testet mod manualens eksempelpakke), ur-synk, scan-samling på punkt-niveau, filtre, sikkerhedszoner pr. pakke, LiDAR-odometri (de-skew + punkt-til-linje ICP mod keyframes nær robotten), log-odds-kort, HTTP/WebSocket-UI, simulator der laver rå LD19-bytes, `bench`. 17 tests grønne. Tal: ~3 % af én kerne, ~6 MB RAM, statisk jitter < 1 mm, 0,02 % drift på en 8,7 m tur, tilbage ved start med 2,6 mm fejl efter 173 m. Statisk ARM64-binær bygges med `--target aarch64-unknown-linux-musl` (verificeret under qemu: identiske resultater).
**Findings:** Den gamle app hakkede pga. software, ikke sensoren (1,5 mm støj): sweeps skåret på pakkegrænser (492/504 punkter, overlap ved sømmen), ingen tidsstempler, kortet ignorerede positionen, ICP frame-til-frame uden startgæt, EMA-lag oven på hinanden, sandsynlig spejlvending (LD19 tæller med uret). Optagelserne er alle statiske (LiDAR på et bord) → simulatoren er nødvendig for at teste kørsel.
**Issues:** Intet kørt på rigtig LiDAR/Pi (cloud). Pi-tal er estimater; Mikkel skal køre `bench` på Pi'en. Kiosk-autostart på 7" skærm ikke sat op. `pkill -f <mønster>` i samme kommando dræber egen shell — brug `[s]ennep`-tricket.
**Handoff:** fuld kontekst (Mikkels svar ordret, beslutninger, antagelser, næste skridt) i `ai-instructions/HANDOFF_LIDAR.md`.
**Next:** Kør på Pi'en med rigtig LiDAR + `--record` nogle ture; tjek LiDAR'ens 0°-retning og robotomrids; integrér `ZoneReport` i Pi↔Mega-linket; gyro i odometrien; gem/indlæs kort.
