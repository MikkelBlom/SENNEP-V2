# Pi 5 ↔ Arduino Mega — forbindelse og protokol

Status: **UDKAST, ikke godkendt af Mikkel endnu.** Oprettet 2026-10-12.
Ejerskab: dette dokument ejer protokol og software. Den fysiske ledningsføring (stik, niveauskift, stel, nødstop-kredsløb)
ejes af kredsløbsopgaven i `docs/electronics/`. Hvor der står **[kredsløb]**, skal det afstemmes dér.

## 1. Udgangspunkt

### Hvad der findes (fra `components/INVENTORY.md`)

| Del | Antal | Relevant for linket |
|---|---|---|
| Raspberry Pi 5 | 1 | 3,3 V-GPIO, 4× USB-A, UART på GPIO14/15 + flere via overlays |
| Arduino Mega 2560 R3 (klon) | 2 | 5 V-logik, 8 KB RAM, 256 KB flash, **4 hardware-UART**, 15 PWM, 16 ADC. USB-chip ukendt (16U2 eller CH340) |
| Arduino Nano (CH340) | 2 | Reserve. Var den oprindelige motor-MCU |
| 4-kanal level converter (BSS138) | **1** | Deles med ESP-01 (bruger 2 kanaler) |
| BTS7960-drivere | 5 | 4 i brug: hver har RPWM, LPWM, R_EN, L_EN, R_IS, L_IS |
| MPU-6050, QMC5883L, NEO-6M, INA219, 8× VL53L0X + TCA9548A, 4× E18-D80NK, 4× HC-SR04, NRF24L01, ESP-01, WS2812B, nødstop + 40 A-relæ | | Sensorer/IO der skal fordeles |

### Hvad der allerede er besluttet (PROJECT_OVERVIEW §3, §7b, §8)

- Pi = højniveau i **Python, ingen ROS2**. MCU = motorer + sekundær kollisionsundgåelse + RF-override.
- Pi ↔ MCU "via UART" — men MCU'en var **Nano**, Mega var plan B. Denne opgave antager **Mega** (4 UART'er, 4× RAM).
  → Det skal Mikkel bekræfte (spørgsmål 1).
- **Ingen motor-encodere** (bevidst valg, apr 2025). Motorerne (RS 834-7641) har ingen encoder.
  Konsekvens: Megaen kan kun styre PWM/duty, ikke hastighed. Odometri må komme fra LiDAR-scanmatching + IMU.
- Nødstop styrer 40 A-relæet (hardware, uafhængigt af software).
- GPS, IMU og magnetometer skal med. Ikke vandtæt.

### Spændinger

Pi-GPIO er **3,3 V og tåler ikke 5 V**. Mega er 5 V-logik og læser høj fra ca. 3,0 V (0,6·Vcc), så 3,3 V ind i
Megaen virker typisk, men med lille margen. Alt fra Mega → Pi-GPIO **skal** niveauskiftes. Over USB er problemet væk.

## 2. Fysisk forbindelse — tre muligheder

```
 A) USB-kabel                       B) GPIO-UART                      C) I²C / SPI
 Pi USB-A ──USB──► Mega USB-B       Pi GPIO14 TX ─►LV│HV─► Mega RX2    Pi = master, Mega = slave
   (CDC-seriel via 16U2/CH340)      Pi GPIO15 RX ◄─LV│HV◄─ Mega TX2    kræver niveauskift + polling
   5 V + GND følger med kablet      + GND, + GPIO → Mega RESET (flash)
```

| | **A: USB** | **B: GPIO-UART + niveauskift** | **C: I²C/SPI** |
|---|---|---|---|
| Latens (30-byte frame) | ~1–3 ms (USB-frame 1 ms + chip-buffer) | ~0,6 ms ved 500 kbaud | Pi skal polle; I²C 400 kHz ≈ 1 ms + polling-jitter |
| Båndbredde | rigeligt (500 kbaud–1 Mbaud) | rigeligt (250k/500k/1M er fejlfri på 16 MHz) | I²C lav, SPI høj |
| Niveauskift | **ingen** | ja, 2 kanaler (deler den ene converter med ESP-01) **[kredsløb]** | ja, 2–4 linjer |
| Flash Megaen fra Pi | **gratis**: `arduino-cli upload` / avrdude, DTR-reset er indbygget | kræver ekstra GPIO → 100 nF → RESET, avrdude med `-c arduino` + reset-script | nej (USB/ISP separat) |
| Forsyning | Mega kan få 5 V fra Pi'en (pas på: Pi 5 USB-budget) — eller egen 5 V **[kredsløb]** | Mega på egen 5 V | — |
| Robusthed | USB kan disconnecte ved motorstøj/ESD → **auto-reconnect nødvendig**. Stik kan ryste løs | Ingen enumerering, kan ikke "forsvinde". Dupont-ledninger kan ryste løs | Mega som I²C-slave er kendt for at hænge bussen |
| Auto-reset | Megaen **genstarter hver gang porten åbnes** (DTR). Håndteres i protokollen (HELLO) | ingen | ingen |
| Fejlsøgning | Kan sætte en laptop i Megaen i stedet for Pi'en | kræver USB-TTL-adapter | svær |
| Megaens UART'er | Serial0 går til Pi → Serial1–3 fri | Serial0 fri til debug/flash → én af Serial1–3 brugt | alle fri |

**Konklusion:** C fravælges (polling, ingen fordel, I²C-slave-problemer). Mellem A og B vinder **A (USB)** for en hobby-robot:
nul ekstra hardware, flash fra Pi'en uden tricks, og man kan teste Megaen med en laptop. Latensforskellen (~1–2 ms)
betyder intet for en robot med 0,6 m/s topfart (1 ms = 0,6 mm). B er en god plan B, hvis USB viser sig ustabil
ved motorstøj — protokollen er den samme, så skiftet er en kabel- og konfigurationsændring.

## 3. Protokol — muligheder

| | Bedømmelse |
|---|---|
| **micro-ROS** | **Passer ikke.** Kræver "tens of kilobytes" RAM og 32-bit MCU; ingen 8-bit/AVR-support. Og I har valgt ROS fra. |
| **rosserial** | ROS1 (EOL 2025). Fravælges. |
| **Firmata** | Pi'en styrer pins direkte → **real-tidslogikken og failsafe havner på Pi'en**. Strider mod hele ideen. Fravælges. |
| **MAVLink v2 (egen dialekt)** | Kører beviseligt på Mega 2560 (gamle ArduPilot APM). Header-only C, `pymavlink` på Pi. Indbygget framing, CRC, sekvensnr. Ulempe: XML-dialekt + kodegenerering, ~280 bytes buffer pr. kanal, meget maskineri til ~15 beskeder. |
| **Egen binær protokol: COBS + CRC-16** | Lille (~200 bytes RAM, ~1 KB flash), fuldt forstået, let at teste på host. Kræver at begge sider holdes i sync → én fælles beskedtabel, der genererer C-header + Python-modul. **Anbefalet.** |
| Tekst/ASCII (`M 100 -50\n`) | Let at læse, men langsom parsing på AVR, ingen CRC, uklare grænser. Kun til debug. |

### Anbefalet framing

```
 rå frame (før COBS):  [ type u8 | seq u8 | payload 0–48 B | crc16 u16 LE ]
 på ledningen:         COBS(rå frame) 0x00
```

- **COBS** fjerner alle 0x00 fra data → 0x00 betyder altid "frame slut". Mister man en byte, er højst én frame tabt;
  modtageren synkroniserer ved næste 0x00. Overhead: 1 byte pr. 254.
- **CRC-16/CCITT-FALSE** (poly 0x1021, init 0xFFFF) over type+seq+payload. Fanger alle 1–2-bit-fejl og alle burst ≤ 16 bit.
- **seq** tæller pr. retning (0–255) → modtageren tæller tabte frames.
- Alle tal er **little-endian, faste heltal** (ingen float på ledningen undtagen parametre): mm/s, mrad/s, mV, mA, µs.
- Maks frame 52 bytes rå → Megaen har en fast buffer, ingen `malloc`, ingen `String`.

### Baudrate

USB-CDC ignorerer baudraten mellem Pi og USB-chippen, men chippen sender videre til ATmega'en med den valgte rate.
16 MHz deler 115200 skævt (≈ +2,1 % med U2X); **250000 / 500000 / 1000000 er eksakte**. Forslag: **500000**.
Belastning med beskedlisten nedenfor: ~5 kB/s af 50 kB/s = ~10 %.

## 4. Real-tid, failsafe og genforbindelse

### Hvad "instant" kan betyde (forslag — spørgsmål til Mikkel)

| Kæde | Budget |
|---|---|
| Pi sender kommando → PWM ændret | typisk 2–4 ms, worst case ~10 ms (Linux-jitter) |
| Sensorhændelse på Mega → Pi har den | IMU 10 ms (100 Hz), afstande 40 ms (25 Hz) |
| Kantdetektor/ToF "for tæt" → motor stoppet | **< 5 ms, uden Pi'en** (Megaen gør det selv) |
| Pi går død → motorer stoppet | **200 ms** (kommando-timeout) + rampe |
| Nødstop trykket → motorstrøm væk | øjeblikkeligt, ren hardware (relæ) |

### Failsafe-lag

1. **Hardware-nødstop** bryder relæet. Megaen læser nødstoppets tilstand på en input-pin og melder det **[kredsløb]**.
2. **Mega-tilstandsmaskine:** `BOOT → DISARMED → ARMED → (LINK_LOST | ESTOP | FAULT)`. Motorer kører kun i `ARMED`.
   Pi'en skal sende `ARM` eksplicit efter hver (gen)forbindelse — en Mega-genstart kan aldrig få robotten til at køre af sig selv.
3. **Kommando-watchdog:** ingen gyldig `CMD_DRIVE` eller `HEARTBEAT` i 200 ms → `LINK_LOST`, motorer rampes til 0
   på ~150 ms, BTS7960 EN-pins trækkes lave. Kommer linket tilbage, kræves nyt `ARM`.
4. **Lokal sikkerhed på Megaen:** kantdetektorer og ToF under en grænse blokerer bevægelse i den retning,
   uanset hvad Pi'en siger (Pi'en får en flag-bit om det).
5. **AVR hardware-watchdog** (`wdt_enable(WDTO_250MS)`) genstarter Megaen, hvis firmwaren hænger.
   ⚠ Ældre Mega 2560-bootloadere (findes på kloner) hænger i en reset-løkke efter et WDT-reset. Skal testes på jeres board.
6. **Pi-siden:** ingen telemetri i 100 ms → biblioteket melder "Mega væk", lukker porten og prøver igen hvert 0,5 s.

### Genforbindelse og auto-reset

- **udev-regel** giver Megaen et fast navn: `/dev/sennep-mega` (match på USB VID/PID + serienummer).
- Når porten åbnes, genstarter Megaen (DTR). Pi'en venter på `HELLO` (≤ 2,5 s), tjekker protokolversion og sender så `ARM`.
- `HELLO` indeholder boot-årsag (power-on / watchdog / brown-out / ekstern reset) og boot-tæller → uventede genstarter kan ses i loggen.

### Tidssynkronisering

Hver telemetri-frame har Megaens `micros()` (u32, wrapper efter 71 min — Pi'en folder den ud).
Pi'en sender `TIME_SYNC(t_pi)` hvert sekund; Megaen svarer med `t_pi` + sin egen tid. Pi'en bruger de 8 seneste svar
med kortest rundtur til at estimere offset + drift (NTP-princippet). Forventet præcision: ~0,5 ms — rigeligt til
at fusionere IMU med LiDAR-scans (10 Hz).

### Faldgruber på Megaen, som designet tager højde for

- Serial RX-bufferen er 64 bytes = 1,3 ms ved 500 kbaud. `loop()` må aldrig blokere længere end det:
  ingen `delay()`, ingen `pulseIn()` (HC-SR04 laves med pin-change-interrupt), VL53L0X i continuous mode og ikke-blokerende læsning.
  Bufferen kan hæves til 256 med `-DSERIAL_RX_BUFFER_SIZE=256` (koster 192 bytes RAM pr. UART).
- **WS2812B slår interrupts fra** under opdatering: 150 LED ≈ 4,5 ms → ~225 tabte bytes ved 500 kbaud.
  Løsning: LED-strip på Nano'en, på Pi'en (SPI), eller kun opdatere i et hul efter en modtaget frame. Spørgsmål til Mikkel.
- Fast hukommelse: ingen `String`, ingen `malloc`; `F()`-strenge i flash.

## 5. Hvem ejer hvilken sensor (forslag)

Princip: **alt sikkerhedskritisk og alt med hård timing → Mega. Alt med tung databehandling, eller som ikke er tidskritisk → Pi.**

| Enhed | Forslag | Hvorfor |
|---|---|---|
| 4× BTS7960 (8 PWM + EN + 8 IS-strømmålinger) | **Mega** | Real-tid, failsafe |
| Mecanum-kinematik (vx, vy, ω → 4 hjul) + rampe | **Mega** | RF-override og Pi bruger samme kode; Pi sender kun vx/vy/ω |
| Nødstop-tilstand (input) | **Mega** | Melder til Pi |
| 4× E18-D80NK kant + 8× VL53L0X (via TCA9548A) + 4× HC-SR04 | **Mega** | Lokal kollisions-/fald-stop uden Pi |
| MPU-6050 + QMC5883L | **Mega** (rådata 100 Hz til Pi) | Kursholdning (heading hold) kan køre lokalt; Pi laver sensorfusion |
| NRF24L01 RF-joystick | **Mega** | Manuel override skal virke, selv når Pi'en er død |
| INA219 batteri | **Mega** | Lavspændings-stop lokalt |
| GY-NEO6MV2 GPS | **Pi** (egen UART, 3,3 V direkte) | 1 Hz, ikke tidskritisk, `gpsd`/NMEA-parsing er lettere på Pi. Alternativt Mega Serial2 + videresend |
| LiDAR LD19 | **Pi** (USB-adapter eller UART) | 230400 baud, tung databehandling |
| Kamera, 7" skærm, lyd, WiFi | **Pi** | — |
| ESP-01 | **Spørgsmål** | Pi har WiFi. Mulig rolle: nød-fjernbetjening via WiFi uden Pi. Ellers droppes den |
| WS2812B | **Spørgsmål** | Se faldgrube ovenfor |
| DFPlayer | Pi (lyd kan Pi'en selv) eller Mega Serial3 | — |

Megaens UART'er med USB-forslaget: Serial0 = Pi, Serial1 = ESP-01 (hvis med), Serial2 = GPS (hvis den alligevel skal på Mega), Serial3 = DFPlayer/fri.

## 6. Beskedliste — udkast v0

Pi → Mega (type 0x01–0x3F):

| Type | Navn | Payload | Rate |
|---|---|---|---|
| 0x01 | `HEARTBEAT` | — | 10 Hz når den ikke kører |
| 0x02 | `ARM` / 0x03 `DISARM` | — | ved behov |
| 0x04 | `ESTOP_SW` | — | software-nødstop (fra UI) |
| 0x10 | `CMD_DRIVE` | vx i16 mm/s, vy i16 mm/s, ω i16 mrad/s, flags u8 | 20–50 Hz (fungerer også som heartbeat) |
| 0x11 | `CMD_WHEELS` | 4× i16 duty ‰ (−1000…1000) | test/kalibrering |
| 0x20 | `SET_PARAM` | id u8, værdi f32 | ved behov → svar `ACK` |
| 0x21 | `GET_PARAM` | id u8 | ved behov |
| 0x22 | `SET_OUTPUT` | output-id u8, værdi u16 (lys, buzzer, LED-mønster) | ved behov |
| 0x30 | `TIME_SYNC` | t_pi u32 µs | 1 Hz |

Mega → Pi (type 0x80–0xBF):

| Type | Navn | Payload | Rate |
|---|---|---|---|
| 0x80 | `HELLO` | protokolversion u8, firmware-hash u32, boot-årsag u8, boot-tæller u16 | ved opstart, gentaget 2 Hz til første ARM/HEARTBEAT |
| 0x81 | `STATUS` | t u32, tilstand u8, fejlbits u16, batteri mV u16, strøm mA i16, maks loop µs u16, rx-fejl u16 | 10 Hz |
| 0x82 | `IMU` | t u32, acc 3×i16, gyro 3×i16 (rå) | 100 Hz |
| 0x83 | `MAG` | t u32, 3×i16 | 50 Hz |
| 0x84 | `RANGES` | t u32, 8× ToF u16 mm, 4× sonar u16 mm, kant-bits u8 | 25 Hz |
| 0x85 | `MOTORS` | t u32, 4× duty i16, 4× strøm u16 (fra IS) | 50 Hz |
| 0x86 | `RC` | t u32, 2× akse i16, knapper u8, link-ok u8 | 50 Hz når aktiv |
| 0x87 | `TIME_SYNC_REPLY` | t_pi u32, t_mega u32 | svar |
| 0x88 | `ACK` | svar-på type u8, seq u8, resultat u8 (+ værdi f32 ved GET_PARAM) | svar |
| 0x89 | `LOG` | niveau u8, tekst ≤ 40 B | sjældent |

Beskedtabellen skrives én gang (fx `protocol/messages.yaml`) og genererer `sennep_msgs.h` (Mega) + `sennep_msgs.py` (Pi),
så de to sider aldrig kan komme ud af sync. Protokolversionen i `HELLO` tjekkes ved forbindelse.

## 7. Software-skitse

```
Pi 5 (Python 3)                                        Mega (Arduino C++)
┌───────────────────────────────┐                      ┌──────────────────────────────────┐
│ planner / UI / LiDAR          │                      │ loop() — aldrig blokerende       │
│        │  drive(vx,vy,w)      │                      │  ├ link.poll()  (COBS/CRC → msg) │
│ sennep_link (bibliotek)       │                      │  ├ safety.update() watchdog/kant │
│  ├ reader-tråd: COBS→CRC→msg  │ USB CDC 500 kbaud    │  ├ drive.update() kinematik+rampe│
│  ├ seneste værdi pr. besked   │◄────────────────────►│  ├ sensors.update() (tidsstyret) │
│  ├ callbacks / asyncio        │                      │  ├ rc.update()                   │
│  ├ heartbeat + time-sync      │                      │  └ telemetry.update() (rater)    │
│  └ reconnect + udev-navn      │                      │ wdt_reset()                      │
└───────────────────────────────┘                      └──────────────────────────────────┘
```

Flash fra Pi'en: `arduino-cli compile --fqbn arduino:avr:mega firmware/sennep_mega && arduino-cli upload -p /dev/sennep-mega …`
(biblioteket lukker porten først). Firmware-hash i `HELLO` viser, om Pi'en og Megaen kører matchende version.

Test uden hardware (cloud): encode/decode-rundtur, CRC mod kendte testvektorer, tabte/indsatte/ændrede bytes midt i en strøm,
fuzz med tilfældige bytes, og den samme C-kode (frame-laget) kompileret for host og kørt mod Python-implementationen.

## 8. Anbefaling (kort)

**USB-kabel + egen binær protokol (COBS + CRC-16) med 500 kbaud, Python-bibliotek på Pi, kinematik og alle
sikkerhedsfunktioner på Megaen, 200 ms kommando-watchdog med eksplicit ARM.** GPIO-UART er plan B med samme protokol.
MAVLink er et rimeligt alternativ, hvis I senere vil bruge eksisterende værktøjer — men det er mere maskineri, end robotten har brug for nu.

## Kilder

- micro-ROS supported hardware: https://micro.ros.org/docs/overview/hardware/ og maintainer-svar om Uno/Mega: https://discourse.ros.org/t/micro-ros-on-arduino/17009
- Pi 5 UART (GPIO14/15, `ttyAMA0` vs `ttyAMA10`, overlays): https://github.com/raspberrypi/documentation/issues/3239 , https://pinout.xyz/pinout/pin10_gpio15 — **skal verificeres på Pi'en** (`ls -l /dev/serial*`), kilderne er uenige.
- Baudrate-fejl på 16 MHz: https://www.mikrocontroller.net/topic/361697 + egen U2X-beregning (bør tjekkes mod ATmega2560-databladets UBRR-tabel).
- COBS: https://satlab.aau.dk/stem/jensd/doc/packetprotocols/cobs.html ; PacketSerial (COBS/SLIP, ingen CRC): https://www.arduino.cc/reference/en/libraries/packetserial
