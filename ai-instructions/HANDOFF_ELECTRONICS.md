# Handoff — elektronik/breadboard-session (2026-10-12)

Til konsolideringssessionen. Branch: `claude/zen-bardeen-klhwv5` (MikkelBlom/SENNEP-V2). Én af seks parallelle cloud-sessioner.
Hovedresultat: **`docs/electronics/BREADBOARD_PLAN.md` (plan v0.2)**. Session-log: `SESSIONS.md` (sidste indlæg).
Pointer tilføjet i `PROJECT_OVERVIEW.md` §3 (Mega i stedet for Nano; test-LiPo-advarsel).

## Opgaven (som givet)
Første grove elektronikversion (breadboard / jumpere / skrueterminaler) med dele Mikkel ejer; PCB senere (separat opgave), så
valgene skal kunne flyttes til et print. Omfang: 3S6P → hovedsikring, nødstop, afbryder → bucks (Pi 5 5 V/5 A + logik) →
4 motorer via BTS7960 → Mega, Pi 5, LiDAR STL-19P, MPU-6050 + magnetometer, GPS, ESP-01 (3,3 V), øvrige sensorer, niveauskift.
Arbejdsgang: status → research → plan → AskUserQuestion → **først efter godkendelse**: ledningsdiagrammer (SVG/KiCad) + trinvis
bygge-/testvejledning. Koordinér med Pi↔Mega-link-opgaven (`docs/electronics/PI_MEGA_LINK.md` ejer protokollen, denne opgave
ejer ledningerne). **Trin 5 er IKKE udført** — Mikkel vil godkende i konsolideringen først.

## Mikkels svar (ordret indhold)

Runde 1:
- *Testforsyning:* "I have a 12v battery with what looks like an XT60 connector — and a connector that exposes the two ends.
  I have been using that to power the motors — I can power one motor with the one (possibly more, if we set up the system).
  This one battery was able to run all of SENNEP V1's 4 motors at once. So it will be fine for testing." + **12 V-adapter** +
  **multimeter**.
- *Første test:* "I am currently working on the wheels and the chassis — so the first test is setting up the Pi, the Mega,
  the motors and 4 wheels, and the LiDAR. The Mega controlling the motors, and the Pi connected to it. And the Pi controlling
  the LiDAR. Other sessions are working on the other systems. Write down your stuff — we will have a common session that
  consolidates all the work of the six sessions."
- *Pi↔Mega fysisk:* "Another session is doing this."
- *Sensorfordeling (3,3 V på Pi / 5 V på Mega):* "Another session is doing this. So wait until we consolidate. But include
  your proposal."

Runde 2:
- *Testbatteriet:* "this is the same battery that fried the Mega and whole of SENNEP V1. It is a HobbyKing B3 AC compact
  charger. To a **Zippy Compact 35C Series high discharge LiPo battery 1000 mAh 3 cell 11.1 V**."
- *Byggestil:* **Wago/skrue til strøm + breadboard til signaler** (anbefalet valg).
- *Har liggende:* breadboards + Dupont-jumpere, **modstandssæt** (mange værdier). Ingen dioder ud over indkøbslisterne. "Might
  have the fuse stuff. I will make a full component list later."
- *Budget:* "Never mind this. Write down min requirements and best requirements. We will figure out later."

## Beslutninger
- **M1** = Pi 5 + Mega + 4 motorer med hjul + LiDAR. Mega styrer motorer, Pi forbundet til Mega, Pi læser LiDAR.
- **Arduino Mega** (ikke Nano) til motorer — Nano har for få PWM/analog/serielle.
- Strøm via Wago/skrueterminaler; aldrig strøm gennem breadboard/Dupont.
- Budget: minimum ~190 kr / anbefalet ~900–1200 kr (plan §5) — afgøres ved konsolidering.

## Kernepunkter i planen (detaljer i BREADBOARD_PLAN.md)
- **Nødstop** bryder relæspolen (bilrelæ 40 A, NO-kontakt 30→87 forsyner motorbussen); flyback-diode over spolen; logik kører
  videre. Mega måler motorbussen (deler 47k/10k → A4) og holder fælles EN lav indtil "arm". Fejlsikkert ved kabelbrud.
- **Sikringer:** hovedsikring ved XT60 (30 A med 3S6P, **20 A med test-LiPo**), 15 A pr. BTS7960, 5 A logikgren.
  Kabler: 10 AWG hoved, 16 AWG drivere/motorer/logik, 22 AWG signaler (alt haves).
- **Støj:** stjernestel, TVS P6KE15A + 1000–2200 µF på motorbussen, 100 nF over hver motor, PWM 20 kHz (Timer 3/4/5).
- **Pins Mega:** PWM 2/3, 5/6, 7/8, 44/45; EN 22 (10k pull-down); IS A0–A3; bus A4; I²C 20/21; Serial1 ESP-01; Serial2
  evt. Pi-link; NRF24 SPI 50–53 + CE 48/IRQ 49; kant 30–33; HC-SR04 34–41; arm-knap 24.
- **Pins Pi 5:** I²C1 GPIO2/3; UART0 GPIO14/15 (evt. Mega); uart2 RX GPIO5 ← LiDAR; uart3 GPIO8/9 ↔ GPS; 5 V via
  USB-C-pigtail + `usb_max_current_enable=1` (ingen PD fra buck). Overlay-pins fra sekundære kilder → verificér med `pinctrl`.
- **Forslag (til andre sessioners beslutning):** Pi↔Mega via USB (afbryd USB-5 V-lederen, Mega forsynes på 5V-pin);
  3,3 V-sensorer (LiDAR, GPS, IMU, mag) på Pi, 5 V-sensorer (ToF via TCA9548A, kant, INA219, NRF24) på Mega.
- **Test-LiPo (Zippy 1000 mAh 3S 35C, ingen BMS):** sikring < 10 cm fra XT60, LiPo-alarm, aldrig < 10,5 V under last,
  ~5–10 min kørsel med 4 motorer. Bænk-tilstand: logik på 12 V-adapter (≥ 3 A?), motorer på LiPo, kun fælles GND.
- **Testrækkefølge M1** (10 trin med måling pr. trin) i plan §7.

## Fund / usikkerheder
- Motorens stallstrøm står ikke i databladet (estimat 15–25 A, skal måles kort). 4 motorer i stall > BMS 40 A → softstart +
  strømgrænse i koden.
- INA219 med 0,1 Ω shunt kan kun ~3,2 A → kun logikgrenen; INA226 + 50 A/75 mV shunt foreslået.
- Ukendt om IBT-2-klonerne har modstande på IS-benene (web-søgning gav intet) → måles. kILIS ≈ 8500 (spredning 5–12k).
- 5 A-buck formodet XL4015 — skal belastningstestes for Pi 5 (≥ 5,0 V).
- DS3231 og MPU-6050 har begge I²C 0x68. Pi 5 har egen RTC + WiFi → DS3231/ESP-01 delvist overflødige.
- LiDAR: 5 V, 290 mA, TX 3,3 V, 230400 baud; PWM-ben skal i GND.
- `PI_MEGA_LINK.md` fandtes ikke ved sessionens start (heller ikke på remote-branches).

## Næste skridt (efter konsolidering)
1. Afstem med link-sessionen (fysisk forbindelse) og sensorsessionen (fordeling); opdatér plan §2.5–2.7.
2. Mikkel godkender planen → ledningsdiagrammer (SVG eller KiCad) for M1 + detaljeret byggevejledning pr. trin.
3. Målinger på hardware: nødstoppets kontakter, IS-modstande, stallstrøm, 12 V-adapterens rating, buck under last,
   elektrolyt-kittets største værdi.

Kilder brugt: Raspberry Pi USB-PD-whitepaper (RP-009856), bret.dk Pi 5-strømguide, Waveshare PoE HAT-wiki
(`usb_max_current_enable`), github.com/Felipegalind0/RPI5.pinout + pinout.xyz (Pi 5 UART-overlays), datablade i
`components/datasheets/` (bts7960, ld19-stl-19p, samsung 35E, rs-pro motor, neo-6m, ina219).
