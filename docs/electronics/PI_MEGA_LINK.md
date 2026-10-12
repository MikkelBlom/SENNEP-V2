# Pi 5 ↔ Arduino Mega — forbindelse og protokol

Status: **GODKENDT 2026-10-12** (runde 2, §0b). Samlet plan + næste skridt: `ai-instructions/PLAN_PI_MEGA_LINK.md`. Kode: `firmware/`, `protocol/`, `tools/`, `tests/`.

## 0. Mikkels svar, runde 1 (2026-10-12)

- **Mega eller Nano?** Uafklaret — "afhænger af designet". Han har (efter eget udsagn) **flere Uno, én Mega, nogle Nano og én Pi**.
  ⚠ `inventory.csv` siger 2× Mega, 2× Nano, 0× Uno → komponentlisten skal rettes (dokumentations-opgaven).
- **Forbindelse:** **UART nu, hvis materialerne findes**, ellers USB nu og UART senere. → Det kan lade sig gøre, se §2b.
- **Sprog på Pi:** udskudt. Hælder til et hurtigt sprog (Rust/C), men det afhænger af resten af stakken (LiDAR-koden er ikke afklaret).
- **"Instant":** robotten må ikke reagere langsomt, selvom der er rigtig mange input. Lav latens har høj prioritet. Hvordan behandlingen deles (flere Nano'er + Pi eller alt centralt) er åbent → §4b.
- **Sensorfordeling:** "hvad der er bedst design", så længe latensen prioriteres → forslaget i §5 står.
Ejerskab: dette dokument ejer protokol og software. Den fysiske ledningsføring (stik, niveauskift, stel, nødstop-kredsløb)
ejes af kredsløbsopgaven i `docs/electronics/`. Hvor der står **[kredsløb]**, skal det afstemmes dér.

## 0b. Mikkels svar, runde 2 (2026-10-12) — GODKENDT

- **Arkitektur:** én Mega som real-tids-hub + en Nano/Uno til LED-strippen. **Krav: plads til udvidelse** — V1 løb tør for pins. → §4d.
- **Forbindelse:** GPIO-UART (Mega Serial1 ↔ Pi GPIO14/15 via BSS138). **ESP-01 droppes.** Flash fra Pi'en via **reset-ledning** (3. kanal på converteren).
- **Kode nu:** Mega-firmware-skelet + sprog-neutral C99-protokolkerne + host-tests med Python som testmodpart. Pi-applikationens sprog vælges senere.
- **Sikkerhedsreglerne i §4 er godkendt som skrevet** (eksplicit ARM, 200 ms watchdog → rampe til stop + nyt ARM, lokal blokering ved kant/forhindring, hardware-nødstop på relæet).

## 0c. Runde 3 (2026-10-12): Rust

- **Mikkel: "fuld Rust for systemet".** Pi-siden bliver Rust. Python-værktøjet (`tools/sennep_link.py`) erstattes af en Rust-CLI.
- **Åbent:** skal Megaen også køre Rust (avr-hal), eller beholde Arduino-C++? Mikkel har kun brugt Arduino IDE og bad om en forklaring
  (givet i chatten 2026-10-12). Anbefaling: Rust på Pi, Arduino-C++ på Megaen indtil videre; vil man have Rust på en mikrocontroller,
  er en RP2040 (Pico) et bedre mål end AVR. Protokol og sikkerhedsregler er de samme uanset valget.
- Status nu: C-kernen + Mega-skitsen kompilerer (17 KB flash, 1,6 KB RAM), og 12.071 host-tests er grønne. Intet er testet på hardware.

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

## 2b. UART nu — har vi materialerne? Ja.

| Behov | Har | Note |
|---|---|---|
| Niveauskift Mega TX (5 V) → Pi RX (3,3 V) | 4-kanal BSS138-converter | **Skal** niveauskiftes (Pi tåler ikke 5 V) |
| Pi TX (3,3 V) → Mega RX | samme converter | Ville virke direkte (Mega læser høj fra ~3,0 V), men converteren giver margen |
| Pi GPIO → Mega RESET (flash fra Pi) | 3. kanal på converteren | Valgfri. Ellers flashes Megaen med USB-kablet, når det er nødvendigt |
| Ledninger, fælles GND | 22 AWG | **[kredsløb]**: kort, snoet par TX/RX/GND, væk fra motorledningerne |

Konflikt: ESP-01 bruger også 2 kanaler. Med Pi-link (2) + reset (1) er der kun 1 tilbage → **ESP-01 droppes** (Pi'en har WiFi),
eller den får sin egen modstandsdeler, når der er modstande (står ikke i komponentlisten).

Baudrate over BSS138: converteren har 10 kΩ pull-ups, så kanterne er langsomme (~0,5 µs). **Start på 250000** (4 µs pr. bit);
500000 (2 µs pr. bit) virker sandsynligvis også, men skal måles. 250000 er eksakt på 16 MHz og giver ~25 kB/s, dvs. 5× mere end beskedlisten bruger.

Pi 5-siden: GPIO14/15 = UART0. Kilderne er uenige om navnet (`/dev/ttyAMA0` vs `ttyAMA10`; `ttyAMA10` er debug-stikket),
og om det kræver `dtparam=uart0=on` eller `dtoverlay=uart0-pi5` i `config.txt`. Tjekkes på Pi'en med `ls -l /dev/serial* /dev/ttyAMA*`.
Den serielle konsol (login-shell) skal være slået fra på porten (`raspi-config` → Interface → Serial: login nej, hardware ja).

Gevinst ved UART frem for USB: ingen auto-reset ved port-åbning, ingen USB-enumerering der kan forsvinde ved motorstøj, ~1–2 ms mindre latens, og
Megaens USB (Serial0) er fri til debug fra en laptop. Pris: 3 ledninger + converteren, og flash kræver enten reset-ledningen eller USB-kablet.

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
- Maks payload 60 bytes (plads til en indpakket node-frame) → Megaen har faste buffere, ingen `malloc`, ingen `String`.

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

### 4b. Hvor kommer latensen egentlig fra? (svar på "mange input, lav latens")

Det, der gør en robot langsom, er sjældent linket eller programmeringssproget — det er **sensorernes egne målehastigheder**
og **hvor beslutningen træffes**. Datamængden er lille: alle sensorer tilsammen giver ~5 kB/s, og en 16 MHz Mega
klarer det med god margen, når intet i `loop()` blokerer.

| Led i kæden | Typisk tid |
|---|---|
| VL53L0X ToF, én måling | ~20–33 ms (alle 8 måler parallelt i continuous mode) |
| HC-SR04 ultralyd | ~6–25 ms pr. måling; 4 stk skal skiftes om (ellers hører de hinandens ekko) → ~60–100 ms pr. runde |
| LiDAR LD19 | 10 Hz → 100 ms pr. omgang |
| Kamera | 33 ms (30 fps) + inferens |
| E18 kant-sensor | < 2 ms (digital) |
| MPU-6050 | 1 ms (kan læses ved 1 kHz) |
| **Linket Pi ↔ Mega (UART, 30 B)** | **~1 ms** |
| Linux-jitter på Pi'en (Python *eller* Rust) | typisk < 1 ms, sjældent 5–10 ms |
| Motor + gearkasse + robottens masse | 50–200 ms (fysik) |

Konklusion: Rust/C på Pi'en sparer måske 1 ms i linket. Det **betyder noget** for tunge beregninger (LiDAR-scanmatching, kamera)
— men det er en beslutning for LiDAR-stakken, ikke for linket. Den store gevinst kommer af en **refleks-arkitektur**:

```
 Refleks  (< 5 ms, Mega alene):    kant-sensor / ToF for tæt / nødstop / link tabt  → stop eller bloker retning
 Kontrol  (10–20 ms, Mega):        kinematik, rampe, kursholdning med IMU
 Taktik   (50–100 ms, Pi):         undvig forhindring ud fra LiDAR + ToF, følg sti
 Strategi (> 100 ms, Pi):          kort, planlægning, UI, kamera
```

Hvert lag virker, selvom laget over er langsomt eller dødt. Derfor føles robotten hurtig.

### 4c. Én Mega centralt eller flere Nano'er?

Pin-optælling hvis **alt real-tid ligger på én Mega** (Megaen har 54 digitale + 16 analoge, 15 PWM, 4 UART, 6 eksterne interrupts + PCINT):

| Funktion | Pins |
|---|---|
| 4× BTS7960: RPWM+LPWM | 8 PWM |
| BTS7960 enable (fælles eller pr. driver) | 1–4 |
| BTS7960 strømmåling IS (R_IS+L_IS samlet pr. driver) | 4 analog |
| I²C-bus: MPU-6050, QMC5883L, INA219, TCA9548A → 8× VL53L0X | 2 |
| 4× E18 kant | 4 |
| 4× HC-SR04 (fælles trigger, 4 echo på PCINT) | 5 |
| NRF24L01 (SPI + CE + CSN + IRQ) | 6 |
| Nødstop-tilstand | 1 |
| UART til Pi (+ reset) | 2 (+1 på Pi-siden) |
| GPS / DFPlayer (hvis på Mega) | 2 + 2 |
| Dock IR-modtagere (fx 4) | 4 |
| Lys (forlygter/baglygter via MOSFET) | 2–4 PWM |
| **I alt** | **~45–50 af 70** |

Det passer på én Mega med plads til mere. En Nano (20 I/O) kan ikke klare det alene.

| | **A: Én Mega (+ evt. 1 Nano til LED)** | **B: Mega + flere Nano-"sensornoder"** | **C: Nano'er direkte til Pi** |
|---|---|---|---|
| Latens sensor → motor | 1 hop, < 5 ms | 2 hop (Nano → Mega): +1–3 ms + poll | sensor → Pi → motor-Nano: 2 hop + Linux |
| Refleks uden Pi | ja | ja | **nej** (beslutningen ligger på Pi'en) |
| Kompleksitet | 1 firmware, 1 link | N firmwares, intern bus, flere fejlformer | N links på Pi'en |
| Hvornår giver det mening | standard | hvis Megaen løber tør for CPU-tid eller pins | aldrig til sikkerhedskritiske ting |

**Anbefaling: A.** Én Mega som real-tids-hub. En Nano/Uno kun til opgaver, der ødelægger timingen, primært WS2812B-strippen
(slår interrupts fra). Uno = samme chip som Nano (ATmega328P), bare større print. Hvis Megaens `loop()`-tid senere bliver for høj
(STATUS-beskeden måler maks. loop-tid), kan sensorer flyttes ud på en Nano — protokollen og Pi'en mærker ikke forskel.

### 4d. Plads til udvidelse (så V1's pin-mangel ikke gentager sig)

Fem udvidelsesveje, i den rækkefølge de bør bruges:

1. **I²C-bussen er hovedvejen for sensorer.** 2 pins, op til ~100 adresser. Fem TCA9548A giver op til 40 ekstra kanaler til sensorer med
   samme adresse. Regel: nye langsomme sensorer (temperatur, lys, ekstra ToF) går på I²C, ikke på egne pins.
2. **I²C-portudvidere: du har dem allerede.** De 6 LCD-adaptere (`i2c-lcd-adapter`) er PCF8574-chips = 8 digitale I/O hver.
   De er gode til lys, relæer, knapper og langsomme inputs, men ikke til PWM eller hurtige signaler. Kan loddes af LCD'en, eller
   man kan bruge en ekstra.
3. **Udvidelsesporte: Megaens Serial2 og Serial3** taler *samme* COBS-protokol til en Nano/Uno-"node" (5 V ↔ 5 V, 3 ledninger, intet
   niveauskift). Megaen sender nodens frames uændret videre til Pi'en i en `NODE_FRAME`-besked, og frames fra Pi'en går den modsatte vej
   (`NODE_SEND`). Megaen behøver altså ikke ny firmware for at få en ny node. LED-Nano'en er node #3 (Serial3), og Serial2 er fri til node #2.
   Uno'erne og Nano'erne bliver dermed byggeklodser.
4. **Pi'ens egne porte** til det, der ikke er real-tid: GPS (egen UART), LiDAR (USB/UART), lyd, kamera, USB-enheder.
5. **Pin-reserve på Megaen:** målet er mindst 15 frie pins inkl. 2 PWM og 4 analoge efter første opbygning. `pins.h` fører regnskab.

Med det her bliver pin-mangel et spørgsmål om "hvilken vej?", ikke "nyt design".

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
| ESP-01 | **Droppet** (runde 2) | Pi har WiFi; converter-kanalerne bruges til Pi-link + reset |
| WS2812B | **Nano/Uno-node på Mega Serial3** (runde 2) | Slår interrupts fra; må ikke ligge på Megaen |
| DFPlayer | Pi (lyd kan Pi'en selv) eller Mega Serial3 | — |

Megaens UART'er (godkendt): Serial0 = USB (flash/debug), Serial1 = Pi, Serial2 = udvidelsesnode #2 (fri), Serial3 = LED-node (Nano/Uno). GPS og DFPlayer/lyd på Pi'en.

## 6. Beskedliste — udkast v0

> **Kilden er nu `protocol/messages.json`** (genererer `protocol/MESSAGES.md`, C-header og Python-modul). Tabellen her er det oprindelige udkast; ved uenighed gælder `MESSAGES.md`.

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

### Sprog-neutral kerne (fordi Pi-sproget er udskudt)

Frame-laget (COBS + CRC + beskedstrukturer) skrives som **portabel C99 uden Arduino-afhængigheder**. Samme fil bruges på Megaen,
kompileres og testes på en PC, og kan kaldes fra C/C++ eller Rust (FFI) på Pi'en. Python bruges kun som uafhængig
testmodpart (to separate implementationer, der skal give samme bytes = god test). Når Pi-sproget er valgt, skrives Pi-biblioteket
oven på spec'en — enten som binding til C-kernen eller som egen implementation testet mod de samme testvektorer.

## 8. Anbefaling (kort, opdateret efter runde 1)

**Én Mega som real-tids-hub (refleks- og kontrollag) ↔ Pi over GPIO-UART (Mega Serial1, BSS138, 250 kbaud, evt. 500k) med egen
binær protokol (COBS + CRC-16), 200 ms kommando-watchdog med eksplicit ARM, og en sprog-neutral C-kerne.** USB-kablet bruges
til flash/debug og er plan B for linket. ESP-01 droppes. WS2812B på en Nano/Uno.
MAVLink er et rimeligt alternativ, hvis I senere vil bruge eksisterende værktøjer — men det er mere maskineri, end robotten har brug for nu.

## Kilder

- micro-ROS supported hardware: https://micro.ros.org/docs/overview/hardware/ og maintainer-svar om Uno/Mega: https://discourse.ros.org/t/micro-ros-on-arduino/17009
- Pi 5 UART (GPIO14/15, `ttyAMA0` vs `ttyAMA10`, overlays): https://github.com/raspberrypi/documentation/issues/3239 , https://pinout.xyz/pinout/pin10_gpio15 — **skal verificeres på Pi'en** (`ls -l /dev/serial*`), kilderne er uenige.
- Baudrate-fejl på 16 MHz: https://www.mikrocontroller.net/topic/361697 + egen U2X-beregning (bør tjekkes mod ATmega2560-databladets UBRR-tabel).
- COBS: https://satlab.aau.dk/stem/jensd/doc/packetprotocols/cobs.html ; PacketSerial (COBS/SLIP, ingen CRC): https://www.arduino.cc/reference/en/libraries/packetserial
