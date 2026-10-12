# sennep-lidar — LiDAR-stack v2 (Rust)

Ny LiDAR-behandling til SENNEP V2's LDROBOT STL-19P (LD19-protokol). Erstatter den gamle
Electron-app i `Lidar Test Enviornment/` (som kun bruges som testdata). Plan og begrundelser:
[`docs/lidar/PLAN.md`](../docs/lidar/PLAN.md). Målte tal: [`docs/lidar/BENCHMARK.md`](../docs/lidar/BENCHMARK.md).

**Hvad den gør:** læser LiDAR'en (USB-adapteren), udregner sikkerhedszoner hver 2,4 ms, samler hele
omdrejninger, finder robottens bevægelse (LiDAR-odometri), bygger et kort og viser det hele på en
webside, som du åbner fra PC, mobil eller robottens 7" skærm. Ét program, én fil, ~6 MB RAM.

## Kør det på din PC (uden hardware)

```bash
cd lidar
cargo run --release -- --sim --person                 # simuleret lejlighed med en gående person
cargo run --release -- --replay "../Lidar Test Enviornment/recordings/<fil>.jsonl" --loop --no-body
```
Åbn http://localhost:8080. `--no-body` bruges når LiDAR'en ikke sidder på robotten (de gamle optagelser
er lavet på et skrivebord), ellers ignoreres alt inden for robottens omrids (800 × 650 mm).

## Kør det på Pi'en

1. Byg på PC'en (ingen krydsværktøjer nødvendige; giver én statisk fil):
   ```bash
   rustup target add aarch64-unknown-linux-musl
   cargo build --release --target aarch64-unknown-linux-musl
   scp target/aarch64-unknown-linux-musl/release/{sennep-lidar,bench} sennep@<pi>:~
   ```
   (Eller byg direkte på Pi'en med `cargo build --release`; det tager et par minutter.)
2. Find adapterens faste navn: `ls /dev/serial/by-id/` (navnet ændrer sig ikke, som `/dev/ttyUSB0` kan).
   Brugeren skal være i `dialout`: `sudo usermod -aG dialout sennep` (log ud og ind).
3. Prøv: `./sennep-lidar --serial /dev/serial/by-id/usb-Silicon_Labs_…`, og åbn `http://<pi>:8080`.
4. Optag til senere test: tilføj `--record tur1.ldr` (rå bytes + tid; afspilles bit for bit med `--replay tur1.ldr`).
5. Start automatisk: `deploy/sennep-lidar.service` (vejledning øverst i filen).
6. **Mål Pi'en:** `./bench --long --recordings "<repo>/Lidar Test Enviornment/recordings" --out BENCHMARK-pi.md`
   og læg resultatet i `docs/lidar/`.

**7" skærm:** åbn siden i Chromium i kiosk-tilstand, fx `chromium-browser --kiosk http://localhost:8080`.
Hvordan det autostartes afhænger af Pi OS-versionen (Wayland/labwc i nyere versioner), så det er ikke
sat op endnu. Siden tegner kun når der kommer nye data, så den belaster ikke Pi'en meget.

## Test

```bash
cargo test --release          # 17 tests: parser, ur, filtre, zoner, optagelser, simulator
cargo run --release --bin bench -- --long
```
Testene kører de gamle optagelser gennem den rigtige parser og simulatoren (kendt sandhed) igennem
hele kæden. Simulatoren laver **rå LD19-bytes**, inkl. bitfejl på ledningen og en person der går rundt.

## Opbygning

| Fil | Hvad |
|---|---|
| `protocol.rs` | LD19-pakker: resync, CRC8, vinkler (testet mod manualens eksempel) |
| `source.rs` | `ByteSource`: seriel, afspilning (`.ldr` og gamle `.jsonl`), simulator |
| `clock.rs` | LiDAR'ens ms-ur (wrap 30 s) → Pi'ens ur |
| `scan.rs` | Hele omdrejninger, splittet præcist ved 360°→0°, robotramme, tid pr. punkt |
| `filter.rs` | Afstande, robottens egen krop, LDROBOT-lignende støjfilter (kun til odometri/kort) |
| `zones.rs` | 16 sektorer: frirum fra robottens kant, opdateret pr. pakke, "for gammel"-flag |
| `odom.rs` | Prædiktion → de-skew → punkt-til-linje ICP mod nærliggende keyframes |
| `grid.rs` | Log-odds-kort (5 cm), fri plads ryddes af stråler, ændrede fliser sendes |
| `server.rs`, `web/index.html` | HTTP + WebSocket, én statisk side (ingen build-trin) |
| `sim.rs`, `eval.rs` | Simuleret lejlighed, ture, sandhed og scoring |

Koordinater: x frem, y venstre, vinkler mod uret (ROS-konvention). LiDAR'ens 0° = robottens front
(`Mount` i `scan.rs`, hvis den monteres anderledes).

## Til Megaen

`ZoneReport::encode()` giver 41 bytes (tid, stale-maske, 16 × frirum i mm) — inden for linkets
60-byte payload (`docs/electronics/PI_MEGA_LINK.md`). Sektor 0 er lige frem, sektorerne går mod uret.
**En sektor markeret "stale" skal Megaen behandle som blokeret** (LiDAR'en er gået i stå).
Selve link-afsendelsen laves, når linket er bygget (`ObstacleSink`-trait'et er klar til det).

## Kendte begrænsninger (ærligt)

- **Ikke testet på den rigtige LiDAR eller Pi'en endnu** — kun optagelser, simulator og ARM-emulering.
- Zonerne viser den seneste passage, dvs. op til **100 ms gamle** (én omdrejning). Ved 0,4 m/s er det 4 cm.
- **Tynde ting langt væk** (et 4 cm stoleben på 2–4 m) rammes ikke af hver omdrejning; sampleafstanden er
  0,72°. ToF-sensorerne på Megaen dækker nærområdet.
- LiDAR'en ser kun i én højde (~285 mm over gulvet). Lave ting (sokler, kabler) ses ikke.
- Ingen loop closure endnu: keyframe-kortet holder positionen på steder robotten har været, men en
  meget stor runde (andre rum, lange gange) kan stadig give drift. Gyroen (MPU-6050) er ikke flettet ind endnu.
- Kortet gemmes ikke mellem genstarter endnu.
