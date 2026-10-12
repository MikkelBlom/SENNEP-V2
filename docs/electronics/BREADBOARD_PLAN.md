# SENNEP V2 — Elektronik, breadboard-version (plan v0.1)

Status: **UDKAST — afventer Mikkels svar** (se §8). Ingen ledningsdiagrammer før planen er godkendt.
Ejer: denne fil ejer *ledningerne* (strøm, stik, pins, niveauskift). Protokollen mellem Pi og Mega ejes af
`PI_MEGA_LINK.md` (findes ikke endnu, 2026-10-12). Batteriets serieforbindelser er en separat opgave.

Mål: en grov første version, der kan bygges med jumpere, skrueterminaler og Wago-klemmer og testes ét
delsystem ad gangen. Valgene skal kunne flyttes direkte over på et printkort senere (samme stik, samme
pinfordeling, samme strømtræ).

> Arkitekturændring i forhold til PROJECT_OVERVIEW §3: **Arduino Mega** (ikke Nano) styrer motorerne. Med 4
> BTS7960 (8 PWM + 4 strømmålinger), 8 ToF, 4 kantsensorer, NRF24 og 2–3 serielle porte er Nano'en for lille.

---

## 1. Status: alle elektriske dele

Kilde: `components/inventory.csv` + datablade. **Fed** = indgår i første version.

### Strøm og beskyttelse

| Del | Spænding | Strøm | Bemærk |
|---|---|---|---|
| **3S6P 18650 (Samsung 35E)** | 9,0–12,6 V (10,8 nom) | 8 A/celle maks → 48 A for pakken | ~20 Ah / ~220 Wh. **Serier ikke loddet endnu** |
| **3S 40 A BMS** (×2) | 3S | 40 A | Sidste forsvarslinje, ikke en sikring |
| **XT60** (×2 par) | — | 60 A | Batteristik, polaritetssikret |
| **Bilsikringer (blade), sortiment** | 32 V | 5–30 A (tjek hvilke værdier der er) | **Mangler holdere til blade-sikringer** |
| Sikringsholder 5×20 mm (×10) | 250 V | typisk maks 10 A | Kan bruges til logikgrenen (glassikring 5 A — haves ikke?) |
| **Bilrelæ 12 V 40 A, 5-ben** | spole 12 V | spole ~150 mA | Nødstoppet styrer spolen. Ingen flyback-diode indbygget (formentlig) |
| **Nødstop 40 mm** | ? | ? | Kontakttype (NC/NO) og rating ukendt → det skal kun bryde relæspolen |
| **Vippekontakt MTS-103** on-off-on 6 A | — | 6 A | Hovedafbryder for logik (2,6 A ved 10,8 V) |
| Nøgleafbryder S1203 | — | ukendt, lille | Kun til signal/spole, ikke til strøm |
| Trykknap PBS-110 rød | — | lille | Kan bruges som "genstart motorer" |
| **DC-DC 5 A buck** (×2, formentlig XL4015) | ind 8–36 V | 5 A (realistisk 3–4 A uden køling) | **Pi 5** |
| **LM2596-modul m. voltmeter** (×3) | ind 4,5–40 V | 3 A | **5 V logik** (Mega, sensorer, LiDAR) |
| **LD1117V33** (×3) | ind ≤ 15 V | 800 mA | **3,3 V** til ESP-01 + NRF24 (fra 5 V-logik) |
| L7805 (×10), LM2596T-12 (×3) | | | Ikke brug for i v1 (7805 bliver varm ved 12 V → buck i stedet) |
| Breadboard-PSU MB102 (×2) | 6,5–12 V ind | ~700 mA | Kun til bordtest af enkelte sensorer |
| **INA219** | ≤ 26 V bus | **maks 3,2 A med det påmonterede 0,1 Ω shunt** | Kan IKKE måle hele batteristrømmen → måler logikgrenen (se §3) |
| Elektrolyt-kit (120 stk), keramik-kit (≤ 0,1 µF), SMD-dioder (70 stk, typer ukendte) | | | Største elektrolytværdi og spænding ukendt |
| IRFZ44N (×5 + 3) | | | Ikke logic level. Ikke i v1 (lys senere) |

### Motorer og drivere

| Del | Spænding | Strøm | Pins |
|---|---|---|---|
| **4× RS PRO 834-7641** | 12 V | Databladet: 41,3 W, tomgang 5,5 mA (tvivlsomt tal). **Stallstrøm står ikke i databladet** — estimat 15–25 A (skal måles, se §2.1) | 2 terminaler |
| **4× BTS7960-modul (IBT-2)** (+1 reserve) | 5,5–27 V motor, 5 V logik | 43 A pr. chip (teoretisk) | VCC, GND, RPWM, LPWM, R_EN, L_EN, R_IS, L_IS + B+, B−, M+, M− |

BTS7960 (datablad): PWM op til 25 kHz, strømsense `kILIS = IL/IIS ≈ 8500` (5–12 spredning!) → 10 A giver 1,2 mA,
dvs. 1,2 V over 1 kΩ. Om IBT-2-klonen har en modstand på IS-benene, er ukendt → måles (§7, trin 3).
Modulets logik-GND og motor-GND er forbundet på printet.

### Compute og kommunikation

| Del | Forsyning | Logik | Interface |
|---|---|---|---|
| **Raspberry Pi 5** | 5,1 V / 5 A (USB-C) | **3,3 V, IKKE 5 V-tolerant** | UART ×5, I²C, USB ×4 |
| **Arduino Mega 2560** (×2) | 5 V-pin (eller VIN 7–12 V) | 5 V | Serial0–3, I²C, SPI, 15 PWM, 16 analoge |
| ESP-01 | **3,3 V**, ~300 mA spidser | 3,3 V, ikke 5 V-tolerant | UART (115200) |
| NRF24L01 (×3) | 3,3 V (12–115 mA m. PA) | 5 V-tolerante input | SPI + CE + IRQ |
| **BSS138 level converter, 4 kanaler** (×1) | HV 5 V / LV 3,3 V | | Kun 4 kanaler i alt → køb flere |
| TCA9548A (×5) | 1,65–5,5 V | | I²C 0x70 |

### Sensorer

| Del | Forsyning | Logik | Interface | Strøm |
|---|---|---|---|---|
| **LiDAR STL-19P (LD19)** | 5 V (4,5–5,5) | TX 3,3 V | UART 230400, kun TX. PWM-ben **skal i GND** | 290 mA (start: ukendt) |
| **MPU-6050 (GY-521)** | 3,3–5 V (LDO på modul) | 3,3 V I²C | 0x68 | ~4 mA |
| **QMC5883L (GY-271)** | 3,3–5 V | 3,3 V I²C | 0x0D | ~2 mA |
| **GY-NEO6MV2 GPS** | 3,3–5 V (LDO på modul) | TX 3,3 V | UART 9600 | ~50 mA |
| **8× VL53L0X** | 2,8–5 V (modulet har LDO + niveauskift) | I²C 0x29 (alle!) | via TCA9548A | ~20 mA hver |
| **4× E18-D80NK** | 5 V | NPN open collector (LAV = genstand set) | digital | ~25 mA hver |
| **INA219** | 3–5,5 V | I²C | 0x40 | |
| 4× HC-SR04 | 5 V | 5 V | trig/echo | 15 mA hver |
| TSL2561 | 3,3 V | I²C | 0x39 | |
| DS3231 | 3,3–5 V | I²C | **0x68 = samme som MPU-6050** | |
| Mikrofoner, IR-modtagere, DFPlayer, LCD, LED-strip, 3 W-LED | — | — | — | Ikke i v1 |
| Pi AI Camera, 7" skærm | via Pi (CSI / HDMI + USB) | | | ~0,2 A + ~0,5 A fra Pi |

### Mangler helt (til en sikker første version)

- **Holder(e) til blade-sikringer**, gerne en lille sikringsboks med minus-skinne (strømfordeling i ét).
- **Flyback-diode** til relæspolen (1N4007 / 1N5819) og **TVS-diode** på motorbussen (P6KE15A / SMBJ15A).
- **Modstandssæt** (ingen modstande på listen — bruges til spændingsdelere, pull-downs, LED'er).
- **Flere level convertere** (1 stk = 4 kanaler rækker ikke).
- **Stor elektrolyt** 1000–2200 µF / 25 V pr. driver, hvis kittet ikke har det.
- Strømfordeling: **Wago 221** (eller skrueklemmer/busbar) — strøm må ALDRIG gå gennem breadboard eller jumpere.
- **USB-C-pigtail** (han) fra 5 V-buck til Pi 5.
- Breadboards og Dupont-jumpere, multimeter og helst en **laboratorieforsyning med strømbegrænsning** står ikke på listen → spørgsmål i §8.

---

## 2. Research

### 2.1 Effektbudget

| Forbruger | Skinne | Typisk | Spids |
|---|---|---|---|
| Motor ×4 (kørsel) | Motorbus 9–12,6 V | 1–3 A hver | **stall 15–25 A hver (estimat)** |
| Pi 5 + kamera + skærm | 5 V-Pi | 1,5–2,5 A | 4–5 A |
| Mega | 5 V-logik | 80 mA | 100 mA |
| LiDAR | 5 V-logik | 290 mA | ~500 mA ved opstart (gæt) |
| 8× ToF + TCA | 5 V-logik | 160 mA | 200 mA |
| 4× E18-D80NK | 5 V-logik | 100 mA | 100 mA |
| GPS + IMU + mag | 5 V / 3,3 V-Pi | 60 mA | 80 mA |
| ESP-01 + NRF24 (via LD1117) | 3,3 V | 100 mA | 450 mA |
| BTS7960-logik ×4 | 5 V-logik | 20 mA | 40 mA |
| Relæspole | 12 V-logik | 150 mA | 150 mA |
| **Logik i alt (5 V-logik)** | | **~0,9 A** | **~1,5 A** → LM2596 (3 A) OK |
| **Logik i alt fra batteriet** (η ≈ 0,85) | | **~1,5–2 A** | **~3 A** → 5 A-sikring |

**Motorer er hele problemet.** Fire motorer i stall kan i teorien trække 60–100 A, men BMS'en slår fra ved 40 A,
og cellerne kan give 48 A. Derfor:
1. **Softstart og strømgrænse i koden** (rampe på PWM, IS-benet aflæses, fx 8 A grænse pr. motor).
2. **Sikring pr. motor** (15 A blade) og **hovedsikring** (30 A blade) — sikringen beskytter *ledningerne*.
3. **Stallstrømmen måles** før første kørsel med hjul på (trin 3 i §7: kort puls, blokeret aksel, laboratorieforsyning
   med strømgrænse eller multimeter på 10 A-området i højst 1 sekund).

Køretid (grovt): logik ~25 W → ~8 t i tomgang; kørsel med 4 × ~2 A × 11 V ≈ 90 W + logik → ~2 t. Usikkert, fordi
motorernes reelle strøm under last ikke kendes.

### 2.2 Ledningstværsnit (det, der allerede ligger i skuffen)

| Strækning | Strøm | Kabel | Sikring |
|---|---|---|---|
| Batteri → XT60 → hovedsikring → relæ → motorbus | ≤ 40 A | **10 AWG silikone** | **30 A** blade (hovedsikring, tæt på batteriet) |
| Motorbus → hver BTS7960 (B+/B−) | ≤ 15 A | **16 AWG** | **15 A** blade pr. driver |
| BTS7960 → motor | ≤ 15 A | **16 AWG**, snoet par | — |
| Batteri (efter hovedsikring) → afbryder → bucks | ≤ 3 A | 16 AWG (22 AWG er nok elektrisk, men 16 er mere robust) | **5 A** |
| 5 V-buck → Pi 5 | 5 A | 16 AWG eller USB-C-pigtail, < 20 cm | (buckens egen grænse) |
| Signaler | < 100 mA | 22 AWG / Dupont | — |

Tommelfingerregel brugt: silikone 10 AWG ~55 A, 16 AWG ~20 A, 22 AWG ~5 A i fri luft på korte strækninger. Vi ligger
godt under. Spændingsfald 5 V til Pi: 0,4 m 16 AWG (13 mΩ/m) × 5 A ≈ 26 mV — fint. Dupont-pins tåler ~1 A, så **Pi'en
må ikke forsynes via jumpere på GPIO-headeren**.

### 2.3 Stel og støj

- **Stjernestel:** ét samlingspunkt for minus (minus-skinnen i sikringsboksen/Wago-blokken tæt på batteriet). Motorbussens
  minus og logikkens minus løber hver for sig hertil. Ingen motorstrøm i signalledningernes GND.
- Fordi BTS7960-modulet forbinder logik-GND og motor-GND på printet, går der en sløjfe: Mega → driver → stjerne → buck →
  Mega. Den kan ikke undgås uden optokoblere; den gøres lille ved at lægge hver drivers signalledninger + GND **sammen og
  korte**, og ved at føre motorstrømmen tilbage ad sin egen tykke ledning.
- **Pr. motor:** 100 nF keramisk over terminalerne (+ evt. 2× 100 nF fra hver terminal til motorhuset), snoede motorledninger.
- **Motorbus:** 1000–2200 µF elektrolyt ved driverne + **TVS 15 V** (fanger spidser, når relæet åbner under kørsel — det
  var den slags spidser, der dræbte V1's Arduino).
- **PWM 20 kHz** (uhørligt, under BTS7960's 25 kHz-grænse) på Megaens Timer 3/4/5. Timer 0 (millis) røres ikke.
- Hver buck: 100 nF + 100 µF på udgangen ved forbrugeren. LD1117 skal have 10 µF ind og ud (ellers svinger den).
- LiDAR, GPS og IMU fysisk væk fra motorledningerne; magnetometeret mindst 10–15 cm fra motorer og 10 AWG-kabler
  (DC-strøm giver magnetfelt → kompasfejl).

### 2.4 Sikkert nødstop

Nødstoppet bryder **relæspolen**, ikke hovedstrømmen (dets rating er ukendt, og et 40 mm-nødstop er sjældent godt til
40 A DC). Kun motorbussen afbrydes; logikken kører videre, så Pi'en ikke mister SD-kortet og Megaen kan logge hvorfor.

```
 +12 V logik (efter hovedafbryder) ──[NØDSTOP NC]──┬── relæ 86
                                                   │      relæspole        ┌─ 1N4007 over spolen
 GND ──────────────────────────────────────────────┴── relæ 85 ────────────┘  (katode mod +)

 Hovedsikring 30 A ── relæ 30 ══ relæ 87 (NO) ══ motorbus ── 4× 15 A ── BTS7960 ×4
```

- **Fejlsikkert:** knækket ledning, slukket logik eller trykket nødstop → spole uden strøm → motorer døde. Det afhænger ikke af
  software.
- **Ingen automatisk genstart:** når nødstoppet drejes ud, kommer motorbussen tilbage, men Megaen ser bussen (spændingsdeler
  → A4) og holder alle EN-ben LAV, indtil den får en eksplicit "arm"-kommando (fra Pi'en eller den røde trykknap). En ren
  hardware-selvholdning kræver et ekstra relæ (bilrelæet har kun én kontakt) → kan komme på printet.
- **Lag 2:** Megaen trækker fælles EN LAV ved timeout på Pi-linket, strømgrænse eller kantsensor. 10 kΩ pull-down på EN
  holder driverne slukket, mens Megaen booter eller resetter.
- **Lag 3:** BMS (40 A) og hovedsikring.
- Kendt svaghed: at åbne relæet under fuld last giver lysbue på kontakterne. Megaen kan ikke nå at slukke driverne først,
  fordi den først ser det bagefter. Accepteret i v1; på printet kan nødstoppets anden kontakt (hvis den findes) gå til Megaen.

### 2.5 Hvem styrer hvad (forslag)

Princip: **3,3 V-dele til Pi'en, 5 V-dele til Megaen** → næsten intet niveauskift.

- **Pi 5 (3,3 V):** LiDAR (UART), GPS (UART), MPU-6050 + QMC5883L (I²C), kamera, skærm. Pi'en har selv RTC (kræver
  knapcelle) → DS3231 er overflødig.
- **Mega (5 V):** 4× BTS7960, 8× ToF via TCA9548A, 4× kantsensorer, INA219, nødstop-/bussensor, NRF24 (manuel styring),
  evt. HC-SR04 og ESP-01. Megaen er sikkerhedslaget: den kan stoppe ved kant/ToF uden Pi'en.

### 2.6 Niveauskift

| Forbindelse | Retning | Løsning |
|---|---|---|
| Pi ↔ Mega | begge | **USB-kabel** (intet niveauskift, Pi'en kan også flashe Megaen) — *eller* UART på GPIO via BSS138 (2 kanaler) |
| LiDAR TX → Pi RX | 3,3 → 3,3 | direkte |
| GPS ↔ Pi | 3,3 ↔ 3,3 | direkte (GPS-modulet forsynes med 5 V; dets LDO giver 3,3 V-logik) |
| IMU/mag ↔ Pi | 3,3 | direkte, forsynet fra Pi'ens 3,3 V-ben |
| ESP-01 ↔ Mega Serial1 | begge | BSS138 (2 kanaler), som i `components/esp-01/` |
| NRF24 ↔ Mega SPI | | direkte (input 5 V-tolerante, forsyning 3,3 V fra LD1117) |
| BTS7960 ↔ Mega | 5 V | direkte |
| ToF-moduler, TCA9548A, INA219 ↔ Mega I²C | 5 V | direkte (modulerne har egen LDO/niveauskift; tjek pull-ups ved måling) |

Hvis Pi ↔ Mega går via USB: Megaen forsynes på 5V-pinnen fra LM2596, og USB'ens 5 V-leder skal **afbrydes** (tape over
5 V-pinnen i USB-A-stikket eller et "data only"-kabel). Ellers kobles Pi'ens 5 V og buck'ens 5 V sammen gennem Megaens
USB-FET.

### 2.7 Pinfordeling

**Arduino Mega**

| Funktion | Pin | Note |
|---|---|---|
| Motor FV (forreste venstre) RPWM / LPWM | 2 / 3 | Timer 3 |
| Motor FH RPWM / LPWM | 5 / 6 | Timer 3 / 4 |
| Motor BV RPWM / LPWM | 7 / 8 | Timer 4 |
| Motor BH RPWM / LPWM | 44 / 45 | Timer 5 (46 i reserve) |
| Fælles EN (alle R_EN + L_EN) | 22 | 10 kΩ pull-down |
| Strømsense FV / FH / BV / BH (R_IS + L_IS samlet) | A0 / A1 / A2 / A3 | Kun én side leder ad gangen |
| Motorbus-spænding (nødstop-status) | A4 | Deler 47 k / 10 k → 12,6 V = 2,2 V |
| I²C SDA / SCL | 20 / 21 | TCA9548A 0x70 (8× ToF), INA219 0x40, LCD 0x27 senere |
| Serial0 (USB) | 0 / 1 | Pi-link (hvis USB) eller debug |
| Serial1 | 18 / 19 | ESP-01 (via BSS138) |
| Serial2 | 16 / 17 | Pi-link (hvis GPIO-UART, via BSS138) |
| Serial3 | 14 / 15 | Reserve (DFPlayer) |
| NRF24 MISO / MOSI / SCK / CSN / CE / IRQ | 50 / 51 / 52 / 53 / 48 / 49 | |
| Kantsensorer FV / FH / BV / BH | 30 / 31 / 32 / 33 | INPUT_PULLUP. HØJ = ingen gulv = stop. Knækket ledning = HØJ = også stop |
| HC-SR04 trig / echo (×4, valgfri) | 34–41 | Lige = trig, ulige = echo |
| "Arm"-knap (rød PBS-110) | 24 | INPUT_PULLUP |
| Status-LED | 13 | Indbygget |

**Raspberry Pi 5** (fysiske ben på 40-pin headeren)

| Funktion | GPIO (ben) | Note |
|---|---|---|
| I²C1 SDA / SCL → MPU-6050 + QMC5883L | 2 (3) / 3 (5) | 3,3 V fra ben 1 |
| UART0 TX / RX → Mega (hvis GPIO-link) | 14 (8) / 15 (10) | via BSS138 |
| uart2 RX ← LiDAR TX | 5 (29) | `dtoverlay=uart2-pi5`. LiDAR-PWM i GND |
| uart3 TX / RX ↔ GPS | 8 (24) / 9 (21) | `dtoverlay=uart3-pi5`. Optager SPI0 → ingen SPI på Pi'en |
| MPU-6050 INT (valgfri) | 17 (11) | |
| 5 V | — | **via USB-C-pigtail, ikke headeren.** `usb_max_current_enable=1` i config.txt (ingen PD-forhandling fra en buck) |

(UART-overlay-pinnene for Pi 5 er fundet i sekundære kilder — tjekkes med `pinctrl` på den rigtige Pi.)

---

## 3. Blokdiagram

```
                       ┌──────────────┐
  3S6P + BMS 40 A ─XT60┤ HOVEDSIKRING ├──┬───────────────────────────────────────────────┐
  (9–12,6 V)           │    30 A      │  │                                               │
                       └──────────────┘  │ 10 AWG                                        │ 16 AWG
                                         ▼                                               ▼
                             ┌─────────────────────┐                           ┌───────────────────┐
                             │ RELÆ 40 A (30→87)   │◄── spole ── NØDSTOP NC ◄──┤ HOVEDAFBRYDER     │
                             └─────────┬───────────┘                           │ MTS-103 + 5 A     │
                                       │ motorbus  + TVS 15 V + 2200 µF        └────────┬──────────┘
                                       │                                                │ 9–12,6 V logik
              ┌──────────┬─────────────┼───────────┐                       ┌───────────┼─────────────┐
            15 A       15 A          15 A        15 A                     INA219      (shunt 0,1 Ω, ≤3 A)
              ▼          ▼             ▼           ▼                         ▼           ▼             ▼
          BTS7960    BTS7960       BTS7960     BTS7960              5 V/5 A buck    LM2596 5 V     A4-deler
             FV         FH            BV          BH                 (Pi 5)          (logik)       (bus-status
              │          │             │           │                    │               │           fra relæ)
            Motor      Motor         Motor       Motor              USB-C │     ┌─────────┼──────────┐
                                                                      ▼   │     ▼         ▼          ▼
            PWM/EN/IS ◄────────────── ARDUINO MEGA ◄──── USB / UART ──── PI 5    LiDAR    LD1117 3,3 V
                                         │  │  │                       │  │  │   (→Pi)    │
                         I²C: TCA9548A ──┘  │  └── NRF24 (3,3 V) ◄─────┼──┼──┼───────────┤
                         → 8× ToF, INA219   │                          │  │  │           └─ ESP-01 (valgfri)
                         Kantsensorer ×4 ───┘          I²C: MPU-6050 ──┘  │  └── GPS (UART)
                                                            QMC5883L      └── kamera, skærm
  Alle minus → STJERNEPUNKT (minus-skinne) ved hovedsikringen
```

---

## 4. Byggestil (forslag, så det kan blive til et print)

- **Strøm:** sikringsboks/Wago + skrueterminaler på en træ- eller printet bundplade. Aldrig strøm gennem breadboard.
- **Signaler:** breadboard + Dupont til sensorer; Mega og Pi på afstandsstykker ved siden af.
- **Stik som på printet senere:** XT60 til batteri, 5,0 mm skrueterminaler til motorer og strøm, JST PH til sensorer.
- **Pinfordelingen i §2.7 fryses** og genbruges på printet; ændringer skrives her først.

---

## 5. Indkøbsliste (forslag, priser ca., AliExpress/Ardustore)

| Vare | Antal | ca. kr | Hvorfor |
|---|---|---|---|
| Sikringsboks 6-vejs til blade-sikringer m. minus-skinne | 1 | 60–100 | Fordeling + sikringer + stjernestel i én |
| Inline blade-sikringsholder 10 AWG (hovedsikring) | 1–2 | 20 | 30 A tæt på batteriet |
| 1N4007 / 1N5819 | 10 | 15 | Flyback på relæspolen |
| TVS P6KE15A (eller SMBJ15A) | 5 | 20 | Motorbus |
| Modstandssæt 1/4 W | 1 | 40 | Delere, pull-downs |
| Elektrolyt 2200 µF 25 V | 4 | 30 | Ved driverne (hvis kittet ikke har dem) |
| BSS138 level converter 4-kanal | 3 | 30 | Reserve + Pi-link + ESP |
| USB-C-pigtail (han, 2 ledere) | 2 | 20 | Buck → Pi 5 |
| Wago 221 sortiment | 1 | 80–120 | Strømfordeling uden lodning |
| Krympeflex + kabelsko/ferruler | 1 | 50 | |
| *Valgfrit:* INA226 + 50 A/75 mV shunt | 1 | 60 | Måle hele batteristrømmen (INA219 kan kun 3,2 A) |
| *Valgfrit:* laboratorieforsyning 30 V/5 A | 1 | 400–600 | Sikker test af alt, før batteriet er færdigt |
| **I alt (uden valgfrit)** | | **~370–450** | |

Breadboards, Dupont-jumpere og multimeter tilføjes, hvis Mikkel ikke har dem.

---

## 6. Grænseflade mod Pi↔Mega-opgaven

Denne plan leverer: fysisk forbindelse (USB *eller* Pi UART0 GPIO14/15 ↔ Mega Serial2 16/17 via BSS138), fælles GND,
og at Megaen nulstiller EN ved tavshed på linket. Protokol, baudrate og timeout bestemmes i `PI_MEGA_LINK.md`.
Hvis den opgave vælger noget andet fysisk, skal §2.6/§2.7 opdateres.

---

## 7. Bygge- og testrækkefølge (skitse — detaljeres efter godkendelse)

1. **Strømtræ uden last** (laboratorieforsyning eller færdigt batteri): sikringer, afbryder, relæ + nødstop. Mål: relæet
   klikker, nødstop bryder motorbussen, 0 V på bussen ved tryk.
2. **Bucks justeres uden last**: 5,15 V (Pi), 5,00 V (logik), 3,3 V. Derefter dummy-last (fx 3 A) og mål rippel/fald.
3. **Én BTS7960 + én motor** på bordet, Mega via USB: PWM-rampe, retning, IS-aflæsning, stallmåling (kort!).
4. Alle fire drivere + fælles EN + motorbus-sensor; nødstop under kørsel.
5. Pi 5 på sin buck (belastningstest, `vcgencmd get_throttled`), LiDAR, GPS, IMU/mag.
6. Pi ↔ Mega-link (efter PI_MEGA_LINK).
7. Megaens sensorer: TCA9548A + ToF, kantsensorer, INA219, NRF24.

---

## 8. Åbne spørgsmål til Mikkel

1. Har du laboratorieforsyning / 12 V-adapter / multimeter? (Batteriet er ikke i serie endnu.)
2. Hvad skal testes først?
3. Pi ↔ Mega: USB-kabel eller UART på GPIO?
4. Godkender du fordelingen 3,3 V-sensorer på Pi'en, 5 V-sensorer på Megaen?
5. Hvordan vil du bygge: breadboard + Wago, eller lodde på protoboard med det samme?
6. Hvad har du liggende, som ikke står på listen (breadboards, jumpere, modstande, krympetang, Wago, dioder)?
7. Budget til manglende dele?
8. Nødstoppet: ved du, om det har en NC-kontakt (og evt. en NO)?
9. ESP-01 og HC-SR04: med i v1? (Pi 5 har selv WiFi; ToF + LiDAR dækker afstand, men ultralyd ser glas.)
10. DS3231 droppes til fordel for Pi 5's egen RTC?
