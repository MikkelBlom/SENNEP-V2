# History

## V1: S.E.N.N.E.P.
A high-school (HTX) project ("fordybelsesprojekt"). About 700 × 450 mm, ~90 % laser-cut acrylic, box-shaped. It had capacitive touch sensors (a school requirement) and ultrasonic sensors that were too poor for navigation. An Arduino and other parts died from voltage spikes, which is why V2's power design has TVS/Schottky diodes, capacitors and fuses. The four RS PRO motors come from V1.

## V2 timeline
| When | What |
|---|---|
| 2024-05-14 | Ardustore order (boards, LCDs, regulators, basics) |
| 2025-04-07 | V2 starts. Goal: drive by itself and "see" → LiDAR + camera |
| 2025-04-09 | Underside model; idea of sensors on corner faces angled slightly down |
| 2025-04-18 | Power architecture (3S, buck per branch, Schottky/TVS/caps/fuses). Waterproofing required (later dropped). No encoders. Hobby Components order; 8 ToF = N/S/E/W + 4 angled panels. |
| 2025-04 | Docking concept with IR beacons and spring contacts |
| 2025-04-19 | Frame still planned in black laser-cut acrylic |
| 2025-05-01 | Bot'n Roll order (Pi 5 case, battery cells, screen, camera, …) |
| 2025-05-04 | Air intake: 4 grilles of 13 slots (~6865 mm²), exhaust fans, dust filter |
| 2025-05-10/11 | Wheel rotation lock: M3 hole printed Ø2.4, drilled 2.5, tapped M3 |
| 2025-05-09…29 | Printer choice → Bambu X1C |
| 2025-05-16/17 | Pi 5 + AI Camera received; Pi OS installed; emotion-recognition model tried. 3S6P groups built with BMS and nickel strip. |
| 2025-05-25 | LiDAR: youyeetoo FHL-LD19; Tkinter test program `LidarDisplayTest.py` on the Pi |
| 2025-07-17 | Gemini chat "Robot Redesign: From Box to Beauty": body shapes Vector → Exo-Frame → Aura → Evo → Modular Plate → Praxis, no usable result. Last direction: long rectangular body, flat vertical sides flush with the wheels, chamfered corners, visible M3 bolts, flat top. |
| 2025-12 | AliExpress orders (H-bridges, KFL003 bearings, cables, mux, GPS, …) |
| 2026-04-10 | LiDAR test environment (Electron + Next.js) recordings |
| 2026-10-04 → | Claude Code sessions: project reconstructed, wheel generator, skeleton, parts list. See `ai-instructions/SESSIONS.md`. |

Colours wished for: light/yellow wheel with black sleeves, a cabin that is not all black.

## Robot v58 (old Fusion model, read 2026-10-05)
A single hollow shell 500 × 816.5 × 200 mm, ~2 mm walls, faceted with the screen on the sloped front, top made of 2 grid plates, LiDAR holder on top. Only one corner module was modelled (base plate, wheel well, base mount plate carrying the pillow block, motor mount V2, motor, bearing, lock bolt). Hidden placeholders: 4× 40 mm fan, 2 speakers, 120 × 180 protoboard. 838 timeline features, no user parameters. Superseded by the "SENNEP V2" document.

## Sources and their reliability
- Gemini chats (`Samtale.pdf`, `samtale-2.pdf`) and the ChatGPT export (`gpttxt.txt`, Apr–May 2025) are kept **locally only** (private).
- ChatGPT's own "handover" summary is **unreliable**: it invented ROS 2, reused V1's code structure, claimed a "5 mm shaft / direct drive" and claimed "WiFi dropped due to hardware problems". The facts above are taken from the dated messages, not the summary.
- The ChatGPT project (8 chats, incl. `Fordybelsesprojekt - Rapport.docx` and `Hjul design noter og teori.docx`) has **not been reviewed**.

## Laser-cut test of the front panel (2025)
Mikkel's notes from cutting a test front panel, kept verbatim (Danish):

```
Kamera hul højde skal være 0,5mm større
Kamera monteringshuller ser korrekte ud

LED huller er korrekt størrelse - men skal være ca. 1mm tætter på hinanden

Lum sensor ser korrekt ud

ToF ser korrekt ud

Fan huller skal nok være 0,5mm tættere på hinanden

Proto-board passer på den lange længde, men skal ca. 1mm tætter på den korte led.

Skærm hul passer perfekt, men huller er helt forkerte
```

Summary: camera hole +0.5 mm in height; LED holes ~1 mm closer together; fan holes ~0.5 mm closer; protoboard ~1 mm closer on the short side; the screen cut-out fits, but the screen's mounting holes were completely wrong. Light sensor, ToF and camera mounting holes fit.
