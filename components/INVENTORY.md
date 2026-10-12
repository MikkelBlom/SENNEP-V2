# Komponentliste — SENNEP V2

> Genereret af `build_inventory.py` fra `inventory.csv`. Redigér CSV'en, ikke denne fil.

85 linjer · [Compute](#compute) · [Drivlinje](#drivlinje) · [Strøm](#strøm) · [Sensorer](#sensorer) · [Kommunikation](#kommunikation) · [UI](#ui) · [Lyd](#lyd) · [Lys](#lys) · [Styring](#styring) · [Kontakter](#kontakter) · [Kabler](#kabler) · [Passive](#passive) · [Mekanik](#mekanik) · [Forbrug](#forbrug) · [Værktøj](#værktøj)

## Pr. butik (ekskl. fragt)

| Butik | ≈ DKK |
|---|---:|
| Ardustore | 2.003 kr |
| Hobby Components | 824 kr |
| Bot'n Roll | 2.693 kr |
| AliExpress | 883 kr |
| RS / skolen (V1) | 2.400 kr |

## Compute

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `arduino-mega` | Arduino Mega 2560 R3 (klon) | 2 | 490 kr | Ardustore | 2024-05-14 | [arduino-mega-2560.pdf](datasheets/arduino-mega-2560.pdf) | Plan B hvis Nano løber tør for pins |
| `arduino-nano` | Arduino Nano-klon, CH340, USB-C | 2 | 65 kr | Hobby Components | 2025-04-18 | [arduino-nano.pdf](datasheets/arduino-nano.pdf) · [ch340.pdf](datasheets/ch340.pdf) | Planlagt motorstyring (Pi <-> Nano via UART). Kræver CH340-driver på Windows |
| `nano-terminal` | Terminaladapter til Arduino Nano | 2 | 27 kr | Hobby Components | 2025-04-18 |  |  |
| `raspberry-pi-5` | Raspberry Pi 5 | 1 | ? | ukendt | 2025-05 | [raspberry-pi-5.pdf](datasheets/raspberry-pi-5.pdf) | Ingen kvittering. RAM-størrelse ukendt. Ligger hos forældre (okt 2026) |
| `pi5-kabinet` | Aluminiumskabinet til Pi 5 m. 2 blæsere (Waveshare 26086) | 1 | 70 kr | Bot'n Roll | 2025-05-01 |  |  |
| `sd-32gb` | microSDHC 32 GB Adata Class 10 | 1 | 34 kr | Bot'n Roll | 2025-05-01 |  | Pi OS installeret maj 2025 (bruger sennep) |
| `sd-16gb` | SD-kort 16 GB Class 10 + adapter | 1 | 50 kr | Ardustore | 2024-05-14 |  |  |

## Drivlinje

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `l298n` | L298N dual H-bridge-modul | 2 | 80 kr | Ardustore | 2024-05-14 | [l298n.pdf](datasheets/l298n.pdf) | Maks ~2 A/kanal og ~2 V spændingsfald. Erstattes af h-bridge-43a |
| `h-bridge-43a` | H-bridge high-power motordriver (formentlig BTS7960, 43 A) | 5 | 182 kr | AliExpress | 2025-12 | [bts7960.pdf](datasheets/bts7960.pdf) | Varenavnet er afkortet i mailen, så chippen er ikke bekræftet. 4 i brug + 1 reserve |
| `motor-rs834` | RS PRO 834-7641 gearmotor 12 V, 49:1 (975D-serien) | 4 | 2.400 kr | RS / skolen (V1) | 2020-2023 | [rs-pro-834-7641-motor.pdf](datasheets/rs-pro-834-7641-motor.pdf) | Ca.-pris for alle 4. 8 mm aksel, 143 rpm, 41 W. Radial last < 5 kgf -> pillow blocks bærer vægten |

## Strøm

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `18650-35e` | Samsung INR18650-35E Li-ion 3400 mAh | 20 | 880 kr | Bot'n Roll | 2025-05-01 | [samsung-inr18650-35e.pdf](datasheets/samsung-inr18650-35e.pdf) | Samlet som 3S6P (10,8 V nom / 12,6 V fuld, ~20 Ah). Serieforbindelser ikke lavet endnu |
| `18650-holder` | 18650-holder 1×2 m. afstandsstykke | 20 | 45 kr | Bot'n Roll | 2025-05-01 |  |  |
| `bms-3s` | 3S 40 A BMS m. balance | 2 | 73 kr | Bot'n Roll | 2025-05-01 |  |  |
| `lader-3s` | 3S Li-ion-lader 12,6 V 2 A (EU-stik) | 1 | 89 kr | Bot'n Roll | 2025-05-01 |  |  |
| `buck-5a` | DC-DC step-down 5 A | 2 | 61 kr | Hobby Components | 2025-04-18 |  | Tænkt til Pi 5 (5 V / 5 A) |
| `lm2596-modul` | LM2596 step-down-modul m. voltmeter | 3 | 58 kr | Hobby Components | 2025-04-18 | [lm2596.pdf](datasheets/lm2596.pdf) | Maks 3 A |
| `lm2596t-12` | LM2596T-12 regulator (TO-220, fast 12 V) | 3 | 24 kr | Ardustore | 2024-05-14 | [lm2596.pdf](datasheets/lm2596.pdf) |  |
| `l7805` | L7805 5 V lineær regulator (TO-220) | 10 | 20 kr | Ardustore | 2024-05-14 | [l7805.pdf](datasheets/l7805.pdf) | Databladet er TI's µA7800 (samme pinout/funktion som ST L7805) |
| `ld1117-33` | LD1117V33 3,3 V lineær regulator (TO-220) | 3 | 24 kr | Ardustore | 2024-05-14 | [ld1117-3v3.pdf](datasheets/ld1117-3v3.pdf) | Maks 800 mA. Kan forsyne esp-01 |
| `breadboard-psu` | Breadboard-strømforsyning (MB102-type) | 2 | 40 kr | Ardustore | 2024-05-14 | [ams1117-breadboard-psu.pdf](datasheets/ams1117-breadboard-psu.pdf) | 5 V + 3,3 V til breadboard |
| `irfz44n` | IRFZ44N N-MOSFET (TO-220) | 5 | 30 kr | Ardustore | 2024-05-14 | [irfz44n.pdf](datasheets/irfz44n.pdf) | Ikke logic level: kræver ~10 V gate for fuld ledning |
| `mosfet-55v` | N-MOSFET 55 V 41 A (formentlig IRFZ44N) | 3 | 22 kr | Bot'n Roll | 2025-05-01 | [irfz44n.pdf](datasheets/irfz44n.pdf) |  |
| `rele-40a` | Bilrelæ 12 V 40 A, 5-ben | 1 | 8 kr | AliExpress | 2025-12 |  | Nødstop styrer relæet i stedet for at bryde hovedstrømmen |
| `sikringsholder` | Sikringsholder 5×20 mm m. låg | 10 | 22 kr | Bot'n Roll | 2025-05-01 |  |  |
| `bilsikringer` | Bilsikringer (blade), 180 stk sortiment | 1 | 28 kr | AliExpress | 2025-12 |  |  |

## Sensorer

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `vl53l0x` | VL53L0X ToF-afstandssensor (laser) | 8 | 219 kr | Hobby Components | 2025-04-18 | [vl53l0x.pdf](datasheets/vl53l0x.pdf) | Samme I²C-adresse (0x29) -> bruges via tca9548a. 8 stk rundt om robotten |
| `e18-d80nk` | E18-D80NK IR-nærhedssensor (justerbar) | 4 | 127 kr | Hobby Components | 2025-04-18 |  | Kantdetektorer i hjørnerne. Der findes ikke noget officielt datablad |
| `hc-sr04` | HC-SR04 ultralyd-afstandsmåler | 4 | 80 kr | Ardustore | 2024-05-14 | [hc-sr04.pdf](datasheets/hc-sr04.pdf) | 5 V-logik |
| `mpu-6050` | MPU-6050 accelerometer + gyro | 1 | 37 kr | Bot'n Roll | 2025-05-01 | [mpu-6050.pdf](datasheets/mpu-6050.pdf) · [mpu-6050-register-map.pdf](datasheets/mpu-6050-register-map.pdf) | I²C 0x68 |
| `qmc5883l` | GY-271 magnetometer (QMC5883L) | 1 | 19 kr | AliExpress | 2025-12 | [qmc5883l.pdf](datasheets/qmc5883l.pdf) | I²C 0x0D (ikke HMC5883L's 0x1E) |
| `neo-6m` | GY-NEO6MV2 GPS-modul | 1 | 17 kr | AliExpress | 2025-12 | [neo-6m.pdf](datasheets/neo-6m.pdf) | UART 9600 baud. Skal med (afklaret) |
| `ina219` | INA219 strøm-/spændingsmåler | 1 | 15 kr | AliExpress | 2025-12 | [ina219.pdf](datasheets/ina219.pdf) | Batteriovervågning. Maks 26 V |
| `tsl2561` | TSL2561 lyssensor | 1 | 36 kr | Hobby Components | 2025-04-18 | [tsl2561.pdf](datasheets/tsl2561.pdf) |  |
| `ds3231` | DS3231 RTC-modul (ZS-042) | 1 | 45 kr | Ardustore | 2024-05-14 | [ds3231.pdf](datasheets/ds3231.pdf) | ZS-042 har en ladekreds til CR2032. Fjern dioden/modstanden ved ikke-genopladeligt batteri |
| `mikrofon-modul` | Mikrofonmodul m. justering | 5 | 75 kr | Ardustore | 2024-05-14 | [lm393-microphone-module.pdf](datasheets/lm393-microphone-module.pdf) | Digital udgang via komparator + analog udgang |
| `vs1838b` | VS1838B IR-modtager 38 kHz | 10 | 30 kr | Bot'n Roll | 2025-05-01 | [vs1838b-ir-receiver.pdf](datasheets/vs1838b-ir-receiver.pdf) | Dock-beacon (modtager) |
| `magnetkontakt` | Dørmagnetkontakt MC-38 (NC) | 1 | 25 kr | Ardustore | 2024-05-14 |  |  |
| `lidar-ld19` | LDROBOT LD19 / STL-19P LiDAR (solgt som youyeetoo FHL-LD19) | 1 | ? | ukendt | 2025-05 | [ld19-stl-19p-datasheet.pdf](datasheets/ld19-stl-19p-datasheet.pdf) · [ld19-development-manual.pdf](datasheets/ld19-development-manual.pdf) | Ingen kvittering. UART 230400 baud, 3,3 V-logik. Testmiljø i Lidar Test Enviornment/ |
| `pi-ai-camera` | Raspberry Pi AI Camera (IMX500, 12 MP) | 1 | 633 kr | Bot'n Roll | 2025-05-01 | [pi-ai-camera.pdf](datasheets/pi-ai-camera.pdf) · [web](https://www.raspberrypi.com/documentation/accessories/ai-camera.html) |  |

## Kommunikation

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `nrf24l01` | NRF24L01 2,4 GHz radiomodul | 3 | 60 kr | Ardustore | 2024-05-14 | [nrf24l01.pdf](datasheets/nrf24l01.pdf) | 3,3 V forsyning (5 V-tolerante pins). Manuel styring / RF-joystick |
| `esp-01` | ESP8266 ESP-01 WiFi-modul | 1 | 25 kr | Ardustore | 2024-05-14 | [esp8266ex.pdf](datasheets/esp8266ex.pdf) · [esp8266-at-commands.pdf](datasheets/esp8266-at-commands.pdf) · [noter](esp-01/README.md) | Formentlig aldrig brugt. KUN 3,3 V: brug ikke Megaens 3,3V-pin. Pinout, diagram og kode i esp-01/ |
| `tca9548a` | TCA9548A I²C-multiplexer (8 kanaler) | 5 | 35 kr | AliExpress | 2025-12 | [tca9548a.pdf](datasheets/tca9548a.pdf) | Løser at alle 8 VL53L0X har samme adresse. I²C 0x70-0x77 |
| `level-converter` | 4-kanal logic level converter 3,3 V <-> 5 V | 1 | 11 kr | AliExpress | 2025-12 | [bss138-level-converter.pdf](datasheets/bss138-level-converter.pdf) | Bidirektional (BSS138). Fin til I²C og UART |

## UI

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `skaerm-7` | 7" HDMI LCD (C) 1024×600 IPS, touch (Waveshare) | 1 | 484 kr | Bot'n Roll | 2025-05-01 | [web](https://www.waveshare.com/wiki/7inch_HDMI_LCD_(C)) | Monteringshullerne passede ikke i laserskæringstesten |
| `lcd-20x4` | 20×4 LCD, hvid | 2 | 160 kr | Ardustore | 2024-05-14 | [hd44780-lcd.pdf](datasheets/hd44780-lcd.pdf) |  |
| `lcd-16x2` | 16×2 LCD, hvid | 4 | 240 kr | Ardustore | 2024-05-14 | [hd44780-lcd.pdf](datasheets/hd44780-lcd.pdf) |  |
| `i2c-lcd-adapter` | I²C-adapter til LCD (backpack) | 6 | 120 kr | Ardustore | 2024-05-14 | [pcf8574-i2c-lcd-adapter.pdf](datasheets/pcf8574-i2c-lcd-adapter.pdf) | I²C 0x27 (PCF8574) eller 0x3F (PCF8574A) |
| `keypad-3x4` | Membran-keypad 3×4 | 1 | 15 kr | Ardustore | 2024-05-14 |  |  |
| `voltmeter-display` | 0,28" voltmeter-display 0–30 V | 1 | 9 kr | AliExpress | 2025-12 |  |  |

## Lyd

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `dfplayer` | DFPlayer Mini MP3-afspiller | 2 | 60 kr | Ardustore | 2024-05-14 | [dfplayer-mini.pdf](datasheets/dfplayer-mini.pdf) | UART 9600 baud. Brug 1 kΩ i serie på RX ved 5 V-logik |
| `dfplayer-2` | DFPlayer Mini MP3-afspiller | 1 | 74 kr | Bot'n Roll | 2025-05-01 | [dfplayer-mini.pdf](datasheets/dfplayer-mini.pdf) | Samlet 3 stk med dfplayer |
| `hojttaler-1w` | Højttaler 1 W 8 Ω m. selvklæbende pakning | 8 | 48 kr | Hobby Components | 2025-04-18 |  |  |
| `pam8403` | PAM8403 mini-forstærker 2×3 W (5 stk) | 5 | 12 kr | AliExpress | 2025-12 | [pam8403.pdf](datasheets/pam8403.pdf) | 5 V forsyning |

## Lys

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `ws2812b` | WS2812B LED-strip 5 m, 30 LED/m | 1 | 52 kr | AliExpress | 2025-12 | [ws2812b.pdf](datasheets/ws2812b.pdf) | 150 LED -> op til ~9 A ved fuld hvid. 5 V |
| `led-3w` | 3 W LED-modul | 4 | 88 kr | Bot'n Roll | 2025-05-01 |  | Forlygter |
| `ir-led-940` | IR-LED 940 nm | 10 | 45 kr | Bot'n Roll | 2025-05-01 |  | Dock-beacon (sender) |

## Styring

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `joystick-ky023` | KY-023 analog joystick (XY + knap) | 4 | 60 kr | Ardustore | 2024-05-14 |  |  |
| `potentiometer` | Potentiometer, liggende | 5 | 10 kr | Ardustore | 2024-05-14 |  |  |

## Kontakter

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `xt60` | XT60 han/hun-par | 2 | 22 kr | Bot'n Roll | 2025-05-01 |  |  |
| `nodstop-40mm` | Nødstop 40 mm | 1 | 13 kr | Hobby Components | 2025-04-18 |  |  |
| `noegleafbryder` | Nøgleafbryder S1203 12 mm on/off | 1 | 18 kr | Ardustore | 2024-05-14 |  |  |
| `toggle-mts103` | Vippekontakt MTS-103 on-off-on 6 A | 1 | 7 kr | Ardustore | 2024-05-14 |  |  |
| `trykknap-rod` | Trykknap PBS-110 rød | 1 | 5 kr | Ardustore | 2024-05-14 |  |  |
| `skrueterminal-3p` | Skrueterminal 3-pol 5,0 mm, blå (10 stk/pk) | 3 | 28 kr | Hobby Components | 2025-04-18 |  | 3 pakker = 30 stk |
| `jst-ph-2` | JST PH 2,0 mm stik, 2-pol | 5 | 7 kr | Bot'n Roll | 2025-05-01 |  |  |
| `jst-ph-3` | JST PH 2,0 mm stik, 3-pol | 5 | 7 kr | Bot'n Roll | 2025-05-01 |  |  |
| `9v-clip` | DC-stik til 9 V-batteriklips | 2 | 6 kr | Hobby Components | 2025-04-18 |  |  |

## Kabler

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `kabel-10awg` | Silikonekabel 10 AWG, 2 m | 3 | 115 kr | AliExpress | 2025-12 |  |  |
| `kabel-16awg` | Kabel 16 AWG, 10 m | 1 | 60 kr | AliExpress | 2025-12 |  |  |
| `kabel-22awg` | Kabel 22 AWG, 50 m (5×10 m) | 1 | 74 kr | AliExpress | 2025-12 |  |  |
| `krokodillenaebe` | Krokodilleledninger, 10 stk | 1 | 29 kr | Bot'n Roll | 2025-05-01 |  |  |

## Passive

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `keramik-kit` | Keramiske kondensatorer, 300 stk (2 pF–0,1 µF) | 1 | 17 kr | Hobby Components | 2025-04-18 |  |  |
| `elektrolyt-kit` | Elektrolytkondensatorer, 120 stk | 1 | 27 kr | Hobby Components | 2025-04-18 |  |  |
| `smd-dioder` | SMD-dioder, 70 stk | 1 | 13 kr | Hobby Components | 2025-04-18 |  |  |

## Mekanik

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `kfl003` | KFL003 pillow block-leje, 17 mm (2 stk/pk) | 6 | 131 kr | AliExpress | 2025-12 |  | 3 pakker = 6 lejer. Selvjusterende |
| `koeleplade-to220` | Køleplade aluminium TO-220 | 5 | 25 kr | Ardustore | 2024-05-14 |  |  |
| `blaeser-40mm` | Blæser 40×40 mm brushless DC | 4 | 23 kr | Hobby Components | 2025-04-18 |  | Spænding står ikke på fakturaen |
| `protoboard-12x18` | Protoboard dobbeltsidet 12×18 cm | 4 | 56 kr | Hobby Components | 2025-04-18 |  |  |
| `loddebart-breadboard` | Loddbart breadboard, 400 huller | 1 | 10 kr | AliExpress | 2025-12 |  |  |
| `unbrako-sortiment` | Rustfri unbrakoskruer, sortiment 290 stk | 1 | 64 kr | AliExpress | 2025-12 |  |  |
| `gevindindsats-m3` | Messing-gevindindsatser M3, 100 stk | 1 | 42 kr | AliExpress | 2025-12 |  |  |

## Forbrug

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `petg` | PETG-filament (kg) | 15 | ? | ukendt |  |  | 15 kg på lager |
| `loddetin` | Loddetin 1,0 mm, 100 g | 1 | 95 kr | Ardustore | 2024-05-14 |  |  |
| `koelepasta` | Silikone-kølepasta, 30 g | 1 | 75 kr | Ardustore | 2024-05-14 |  |  |

## Værktøj

| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |
|---|---|---:|---:|---|---|---|---|
| `afbidertang` | Afbidertang 125 mm | 1 | 45 kr | Ardustore | 2024-05-14 |  |  |
