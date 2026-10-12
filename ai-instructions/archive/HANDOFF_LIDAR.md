# Handoff — LiDAR-stack v2 (cloud-session 2026-10-12, branch `claude/fervent-tesla-tbrtpz`)

Til den session der konsoliderer de 6 parallelle cloud-sessioner. Alt herfra er committet på branchen
ovenfor (main er merget ind, ingen konflikter). Mikkel har IKKE læst outputtet fra denne session —
gennemgå beslutningerne med ham.

## Opgaven (Mikkels prompt, kort)
Ny LiDAR-behandling + visualisering fra bunden: lynhurtig, ressourcebesparende, kører på Pi 5 (og PC til
udvikling). Den gamle Electron/Next.js-app (`Lidar Test Enviornment/`, lavet med Gemini) kun som inspiration
og testdata. Arbejdsgang: status → research → plan → AskUserQuestion → byg først efter godkendelse, med tests
der afspiller optagelser og måler latens/CPU. Seriel input bag et interface, så optagelser kan erstatte sensoren.

## Mikkels svar (ordret/næsten ordret) — VIGTIGT for andre sessioner
**Runde 1**
- Sprog: *"I told the arduino mega to raspberry pi link research session that the language would depend on
  latency and performance. Because its not only the lidar we need to process on the pi. Its also the facial
  recognition/ai camera and other stuff. So speed and performance is best. I suggested languages such as C or
  Rust - because they are fast and good for performance. Though being harder to develop in. It started doing
  the core loop in C - but with the possibility to rewrite later. So I will leave it up to you - but i prefer
  not using python at all."*
- Første funktion: forhindringer/sikkerhedszoner.
- UI: browser via WiFi + robottens 7" skærm. Vidste ikke hvad Foxglove er → forklaring i `docs/lidar/PLAN.md` §6
  (dev-værktøj på PC, valgfrit, ikke bygget).
- "Det virker" = stabilt billede når robotten står stille + kort der holder når den kører + robotten kan styres
  ud fra det (data til motorstyringen).

**Runde 2 (= plan godkendt)**
- **Rust** for hele Pi-applikationen (min anbefaling, valgt). → Påvirker Pi↔Mega-link-sessionen
  (`docs/electronics/PI_MEGA_LINK.md` på branch `claude/stoic-cray-tsomgg`), som udskød Pi-sproget og har en
  sprogneutral C99-protokolkerne (COBS + CRC-16) + Python-testmodpart. Den C-kerne kan bruges fra Rust via FFI
  eller porteres (~100 linjer).
- Hindringer: **Pi sender "nærmeste frirum pr. sektor" til Megaen** (~20–50 Hz); Megaen begrænser fart/stopper
  sammen med ToF (refleks-arkitekturen i PI_MEGA_LINK).
- Rækkefølge godkendt: (1) parser+scan-samling+simulator+tests → (2) web-UI → (3) zoner → (4) odometri+kort.
- LiDAR tilsluttes via **USB-adapteren fra kittet (CP2102)**, `/dev/serial/by-id/...` (Pi'ens GPIO14/15-UART
  er lovet til Megaen).

## Hvad der er bygget (alt i `lidar/`, Rust, ingen Python)
Én binær `sennep-lidar` (+ `bench`). Brug: `lidar/README.md`. Plan/begrundelser: `docs/lidar/PLAN.md`
(§1 status på gammel app, §2 sensorfakta, §3 research+budgetter, §4 arkitekturvalg, §6 Foxglove, §7 beslutninger,
§8 Rust vs C, §9 moduler, §10 resultater + næste skridt). Målinger: `docs/lidar/BENCHMARK.md`.
Skærmbillede: `docs/lidar/ui-sim-7inch.png`.

- `protocol.rs` LD19-parser (resync, CRC8, testet mod manualens eksempel) · `clock.rs` sensor-ms (wrap 30 s) → værtsur ·
  `scan.rs` omgange splittet på punkt-niveau ved 360→0, robotramme (x frem, y venstre, CCW; LD19 tæller CW!) ·
  `filter.rs` · `zones.rs` 16 sektorer, frirum fra robottens kant, pr. pakke (2,4 ms), 0,25°-bins, strålebredde ±0,4°,
  stale-flag · `odom.rs` konstant-hastighed → de-skew → punkt-til-linje ICP (Huber + 1/σ²-vægte) mod keyframes nær
  robotten (alle keyframes gemmes; genbruges ved genbesøg) · `grid.rs` log-odds 5 cm, dirty tiles ·
  `server.rs` + `web/index.html` HTTP/WebSocket, binære frames, nyeste-frame-semantik · `sim.rs`/`eval.rs` simuleret
  lejlighed der laver rå LD19-bytes (støj fra datablad, bitfejl, gående person) + scoring · `source.rs` serial/
  replay (`.ldr` rå optagelse + gamle `.jsonl`)/sim · `bin/bench.rs`.
- Kør uden hardware: `cargo run --release -- --sim --person` → http://localhost:8080.
- Pi: `cargo build --release --target aarch64-unknown-linux-musl` (statisk, rust-lld, `.cargo/config.toml`);
  verificeret under qemu (identiske resultater). systemd: `lidar/deploy/sennep-lidar.service` (by-id-navnet er et gæt).
- 17 tests grønne (`cargo test --release`).

## Resultater (x86 sky-CPU; Pi 5 forventes ~1,5–2,5× langsommere)
~3 % af én kerne, ~6 MB RAM, 2,9/7,2 ms median/p99 pr. omgang, zoner < 1 µs pr. pakke. Statisk jitter på de
gamle optagelser 0,25–0,7 mm / 0,08°. Sim: 8,7 m tur 0,02 % drift, 99 % kortpræcision; 173 m (11 min): tilbage
ved start med 2,6 mm / 0,01°. 0 "optimistiske" zonefejl i 4800 sektorværdier.

## Fund om den gamle app (hvorfor den hakkede)
Sensoren støjer kun ~1,5 mm. Fejlene var software: sweeps skåret på pakkegrænser (492/504 punkter, overlap ved
sømmen), tidsstempler ignoreret, kortet ignorerede pose, ICP frame-til-frame uden startgæt (og slået fra),
EMA-lag oven på hinanden, kun hver 2. sweep, sandsynlig spejlvending (LD19 er venstrehåndet/CW), Electron for
tungt til Pi. Optagelserne (3 stk, 122 sweeps, ~12 s) er alle statiske på et bord, max ~2 m.

## Antagelser andre sessioner skal tjekke
- Robotomrids i koden: **800 × 650 mm** (Body v0), LiDAR i centrum, LiDAR 0° = front (`Mount` i `scan.rs`,
  `Footprint` i `filter.rs`). Skal matche chassis-/skelet-sessionens mål.
- Zone-payload til Megaen: `ZoneReport::encode()` = 41 bytes (`t_ms u32 | n u8 | stale-maske u32 | 16 × u16 mm`),
  sektor 0 lige frem, mod uret. **Stale skal behandles som blokeret.** Passer i linkets 60-byte payload.
  Ikke integreret i linket endnu (`ObstacleSink`-trait klar).
- Ingen hjul-encodere (bevidst) → LiDAR-odometri er eneste bevægelseskilde; MPU-6050-gyro bør flettes ind (via Megaen).

## Ikke gjort / næste skridt
1. Intet kørt på rigtig LiDAR/Pi. Mikkel: kør `sennep-lidar --serial …`, `bench --long --recordings …` på Pi'en,
   optag ture med `--record`.
2. Tjek LiDAR'ens 0°-retning og omrids. 3. ZoneReport over Pi↔Mega-linket. 4. Gyro i prædiktionen.
5. Gem/indlæs kort + lokalisering; bevægelige objekter; loop closure ved store kort.
6. Kiosk-autostart på 7" skærm ikke sat op (afhænger af Pi OS/Wayland).

## Kendte begrænsninger
Zoner op til 100 ms gamle (én omgang); tynde ting > ~2 m rammes ikke hver omgang; kun ét scanplan (~285 mm);
ingen loop closure; kort gemmes ikke.

## Praktiske gotchas fra sessionen
- `serialport` skal have `default-features = false` (ellers kræver den libudev).
- `pkill -f <mønster>` i samme Bash-kommando dræber egen shell → brug `kill $(pgrep -x sennep-lidar)`.
- Playwright: brug `/opt/pw-browsers/chromium-1194/chrome-linux/chrome` som executablePath.
- `rustfmt.toml` (max_width 110, small heuristics Max) holder struct-literals kompakte.
