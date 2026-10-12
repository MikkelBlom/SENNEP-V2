# LiDAR-stack v2 — benchmark

Maskine: Intel(R) Xeon(R) Processor @ 2.80GHz (x86_64), 4 cores. Build: release. Genereret af `cargo run --release --bin bench`.

> **Ikke målt på Pi'en.** Pi 5 (Cortex-A76) forventes ca. 1,5–2,5× langsommere pr. kerne end denne maskine. Kør `bench` på Pi'en for de rigtige tal (se `lidar/README.md`).

Alle kilder afspilles så hurtigt som muligt; kun pipelinen (parser → zoner → scan-samling → filter → odometri → kort) tidsmåles.
*CPU* = CPU-tid / datavarighed, dvs. hvor stor en del af én kerne stacken bruger i realtid.
*Pr. pakke* = parser + zoneopdatering for 12 punkter (det sikkerhedszonerne venter på). *Pr. omgang* = filter + odometri + kort, efter omgangens sidste punkt.

## Ydelse

| Scenarie | Data | Omg. | Pakker | CPU (1 kerne) | Pr. pakke µs (median/p99/max) | Pr. omgang ms (median/p99/max) | ΔRAM |
|---|---:|---:|---:|---:|---:|---:|---:|
| Optagelse 2026-04-10T02-26-33 | 4.7 s | 46 | 1955 | 2.1 % | 0 / 1 / 13 | 2.15 / 4.74 / 4.74 | 0.5 MB |
| Optagelse 2026-04-10T12-28-47 | 2.6 s | 25 | 1081 | 2.3 % | 0 / 1 / 20 | 2.13 / 2.28 / 2.28 | 0.7 MB |
| Optagelse 2026-04-10T13-22-13 | 4.9 s | 48 | 2038 | 2.2 % | 0 / 1 / 5 | 2.16 / 4.77 / 4.77 | 6.6 MB |
| Sim: står stille 30 s | 30.0 s | 299 | 12500 | 1.5 % | 0 / 1 / 75 | 1.33 / 2.77 / 3.40 | 0.3 MB |
| Sim: tur 8,7 m | 32.0 s | 319 | 13333 | 3.4 % | 0 / 2 / 131 | 2.76 / 11.89 / 12.88 | 4.0 MB |
| Sim: tur + person + støj på ledningen | 32.0 s | 319 | 13266 (66 CRC-fejl) | 3.2 % | 0 / 2 / 254 | 2.75 / 7.26 / 8.09 | 0.0 MB |
| Sim: tur frem og tilbage | 64.0 s | 639 | 26666 | 3.3 % | 0 / 2 / 184 | 2.91 / 6.97 / 8.77 | 0.0 MB |
| Sim: 10 ture frem og tilbage | 640.0 s | 6399 | 266666 | 3.4 % | 0 / 2 / 572 | 2.87 / 7.19 / 12.63 | 0.0 MB |

## Stabilitet når robotten står stille

| Scenarie | STD x | STD y | STD kurs |
|---|---:|---:|---:|
| Optagelse 2026-04-10T02-26-33 | 0.25 mm | 0.36 mm | 0.080° |
| Optagelse 2026-04-10T12-28-47 | 0.29 mm | 0.68 mm | 0.090° |
| Optagelse 2026-04-10T13-22-13 | 0.43 mm | 0.41 mm | 0.071° |
| Sim: står stille 30 s | 1.39 mm | 1.05 mm | 0.039° |

## Odometri og kort mod simulatorens sandhed

| Scenarie | Kørt | Matches godkendt | ATE (RMS) | Slutfejl | Slutfejl kurs | Kort-præcision | Kort-dækning |
|---|---:|---:|---:|---:|---:|---:|---:|
| Sim: står stille 30 s | 0.0 m | 100.0 % | 2.9 mm | 3.2 mm | 0.00° | 100.0 % | 100.0 % |
| Sim: tur 8,7 m | 8.7 m | 100.0 % | 8.5 mm | 1.7 mm (0.02 %) | 0.18° | 99.2 % | 98.4 % |
| Sim: tur + person + støj på ledningen | 8.7 m | 100.0 % | 9.8 mm | 4.2 mm (0.05 %) | 0.04° | 99.0 % | 98.1 % |
| Sim: tur frem og tilbage | 17.3 m | 100.0 % | 11.9 mm | 6.8 mm (0.04 %) | 0.10° | 97.2 % | 98.5 % |
| Sim: 10 ture frem og tilbage | 173.5 m | 100.0 % | 11.0 mm | 2.6 mm (0.00 %) | 0.01° | 92.0 % | 98.4 % |

Kort-præcision = andel af optagne celler under 10 cm fra en rigtig væg. Kort-dækning = andel af de sete vægge, der står i kortet.
