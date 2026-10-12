# Battery plan — 3S6P (18 × Samsung 35E)

> Status: **Plan v2 — APPROVED by Mikkel 2026-10-12 (consolidation session). Method: spot-welding at a workshop in Odense.** (§3a). The compression pack (method 2) is the fallback. The final step-by-step guide is written after the cells are welded (see §10).

> **Order of work: measure and match all cells NOW (§6), before anything is welded or connected → send the numbers to Claude → Claude assigns the 3 groups → only then take the sorted cells to the welder.**

## 1. Status: what we know

| Part | What we have | Source | Note |
|---|---|---|---|
| Cells | 20 × Samsung INR18650-35E, **new and never used** (bought May 2025). 18 are needed → 2 spare | inventory `18650-35e` + Mikkel | 3.6 V nom., 4.20 V charge voltage, 2.50 V cutoff, **max. 8 A discharge**, max. 1.5 A charge per cell |
| Holders | 20 × "1×2 holder with spacer" (click-together plastic) | inventory `18650-holder` | **No metal contacts.** They only hold the cells in place |
| Connections | **None.** The cells just sit in the holders. There is no strip, solder or weld | Mikkel | All 36 cell ends must be connected, not just 2 series links |
| Nickel strip | Pure nickel 0.15 × 27 mm "2H-19 mm" (bought May 2025) | inventory `nickel-strip` + Mikkel | Exists, but the pieces are **too small to cover a row** → must be cut and welded together. The workshop will probably use its own strip |
| BMS | 2 × "3S 40 A BMS with balancing" (Bot'n Roll, product photo says **HW-288**) | inventory `bms-3s` | Listing: 40 A continuous, < 80 A start, charge 12.6–13.6 V / max 10 A, auto recovery, 60 × 41 × 3.4 mm; seller recommends a 10–100 µF non-polar cap (≥ 25 V) across each brushed motor. Boards of this type: 40 A continuous (needs cooling), often ~80 A inrush, passive balancing ~40–100 mA, only at full charge |
| Charger | 3S Li-ion charger 12.6 V 2 A | inventory `charger-3s` | Suitable. 20 Ah / 2 A ≈ 10–11 h from empty |
| Cable | 10 AWG silicone (3 × 2 m), 16 AWG (10 m), 22 AWG (50 m) | inventory `cable-10awg`, `cable-16awg`, `cable-22awg` | 10 AWG for main current, 22 AWG for balance leads |
| Connectors | 2 × XT60 pairs | inventory `xt60` | XT60 is good for battery ↔ robot and charger |
| Fuses | Blade fuses (180-piece assortment) + **maybe blade holders** in the same box, 10 × 5×20 mm glass fuse holders | inventory `blade-fuses`, `fuse-holder` + Mikkel | The holders must be found and their rating checked (often printed on the holder/lead). 5×20 holders are typically max ~10 A → not for the main fuse |
| Other | E-stop 40 mm + 40 A relay, INA219, voltmeter display | inventory `estop-40mm`, `relay-40a`, `ina219`, `voltmeter-display` | |
| Tools | **Fluke 83** (excellent), 48 W soldering station (Puctec ZD-99), side cutters | Mikkel | No spot welder, no crimping tool |

**Unknown / to check:** Which BMS model is it (photo of the board with pads)? Which blade holders are in the box (mini/standard, wire gauge)? Long bolts: **no** (only affects method 2). Fish paper: unknown.

## 2. Electrical numbers

```
3S6P:  3 groups in series, 6 cells in parallel in each
       10.8 V nominal · 12.6 V full · ~9.0–9.9 V empty · ~20.4 Ah · ~220 Wh · ~0.9 kg cells
Max. discharge per cell 8 A  →  pack theoretically 48 A
BMS 40 A                     →  6.7 A per cell (OK)
Charging 2 A                 →  0.33 A per cell (very gentle)
```

**Motors (4 × RS 834-7641):** 41.3 W out at 60 % efficiency ≈ 69 W in ≈ **~5.7 A per motor at rated load** (a similar 42 mm 49:1 motor is rated at 5.5 A). **The stall current is not given** in the RS datasheet. Similar motors are at ~12–25 A, so that number is uncertain. Four motors starting or stalling at the same time can therefore draw 50–100 A for a short moment. That is more than the BMS can deliver.
→ **Measure it yourself:** measure the motor's winding resistance R with the Fluke 83 (Ω range, subtract the resistance of the test leads). Then stall ≈ 12.6 V / R. This is safe and needs no power.
→ **Consequence:** soft-start/ramp in the motor code is mandatory. Without it the BMS cuts out at every hard start. That is annoying but not dangerous. Normal driving (~10–20 A total) is fine for both cells and BMS.

**Motor (measured 2026-10-12):** winding resistance 1.7–1.8 Ω at rest (Fluke, incl. ~0.1–0.3 Ω probe leads). Stall current ≈ 12.6 V / ~1.6–1.8 Ω ≈ **7–8 A per motor**, ~30 A for all four: under the BMS's 40 A. Readings while turning are meaningless (brushes + the motor generating voltage).

## 3. Methods for connecting the cells — ranked by safety

| # | Method | Safety | Does he have it? | Assessment |
|---|---|---|---|---|
| **1** | **Spot-welding nickel strip** — either have a workshop do it (e-bike workshop/battery shop, 36 welds ≈ 15 min of work) or buy a spot welder | ★★★★★ | Strip: maybe. Welder: no | The standard method and the most robust against vibration. Almost no heat goes into the cell. **Recommended as the permanent solution** |
| **2** | **Compression pack** — nickel strip pressed against the cell ends by printed end plates, clamped together with bolts/threaded rod. Springing from TPU pads or disc springs | ★★★☆☆ | Printer, PETG, strip (?), M3 inserts. Probably missing long bolts + fish paper | Can be made from what he has (almost). Only holds if the pressure is kept: PETG creeps under heat and pressure, and vibration can loosen the contact → resistance → heat. **OK to get started, but only with voltage-drop measurement per contact and periodic inspection** |
| 3 | Soldering directly onto the cells | ★★☆☆☆ | 48 W is **too little** | Needs a powerful iron (≥ 80–100 W, wide tip, flux), so each joint takes under 3 s. With 48 W you heat for too long, and that damages the inside of the cell (separator and safety vent). **Not recommended, and not with the equipment he has** |
| ✗ | Magnets (neodymium) between cell and strip | Dangerous | — | High contact resistance. Neodymium loses magnetism at ~80 °C. They can slide → short circuit. **Rejected** |
| ✗ | Strip laid loosely in the click holders / alligator clips | Dangerous | — | The holders give no pressure. Alligator clips can only handle a few A and can fall off → short circuit. **Rejected**, also "just for a test" |
| ✗ | Spring battery boxes (1×N plastic boxes) | Poor | No | The springs can typically handle 2–3 A and get hot. Too weak for motors |

**Honest assessment:** The fastest and safest route is method 1 via a workshop. It probably costs less than a welder, and you never have to put heat on the cells yourself. Method 2 is "do it now with what you have". It can work, but it needs care and measurements. It belongs on a robot that mostly stands still and drives gently, not on one that drives hard over door sills.

### 3a. Welding service in Odense (Mikkel's choice)

Found by web search 2026-10-12. **I could not open the pages myself** (network block), so address and services come from search results and must be confirmed:

| Place | What we know | Chance |
|---|---|---|
| **Fyns Batteri Service ApS**, Knullen 8, Odense S | Refurbishes e-bike batteries (= they spot-weld 18650 packs). Prices "from 1,750 DKK" are for a full refurbishment incl. cells, not welding of your own cells | Best. Call and ask |
| E-bike workshops in Odense (e.g. via Handyhand) | Many send batteries on instead of welding themselves | Low–medium |
| Himex (e-bike webshop, has battery service) | Location not confirmed | Unknown |
| SDU MakerSpace (Videnbyen) | Soldering, 3D printing, laser. Spot welder not mentioned. SDU students only | Only if you are an SDU student |
| Odense Library Makerspace | The machine list could not be fetched | Ask |

**What to say when you call (suggestion, in Danish):**
> "Jeg har 18 nye Samsung 35E-celler (18650), som jeg vil have punktsvejset til en 3S6P-pakke (12 V) til en robot. Cellerne er målt og sorteret, og de sidder i klik-holdere i den rigtige retning. Kan I punktsvejse nikkelstrip på (jeres strip eller min 0,15 mm rene nikkel), og evt. sætte balanceledninger på? Hvad koster det, og hvor lang tid tager det?"

(English: "I have 18 new Samsung 35E cells (18650) that I want spot-welded into a 3S6P pack (12 V) for a robot. The cells are measured and sorted, and they sit in click holders in the right orientation. Can you spot-weld nickel strip on (your strip or my 0.15 mm pure nickel), and maybe add balance leads? What does it cost, and how long does it take?")

Also ask: **pure nickel or nickel-plated steel?** (Only pure nickel. Nickel-plated steel has ~5× higher resistance and gets hot.) Bring rings for + (fish paper) if they have them. **Feel free to bring the BMS** if they will fit it too.

They may not want to weld a customer's own cells (liability). If everyone says no → buy a spot welder (method 1b) or build the compression pack (method 2).

**Preparation before the workshop** (so they only have to weld):
1. Measure and sort the cells (§6) → 3 groups of 6 with equal averages. **This comes first. Nothing is welded or connected before the cells are measured and grouped.**
2. Click the pack together exactly according to the layout in §4 (G1 + up, G2 − up, G3 + up). **Check the orientation of every single cell twice** — one reversed cell in a parallel group = short circuit the moment the strip is welded on.
3. Write "+ PACK" / "− PACK" and "G1/G2/G3" with a marker on the holders.
4. Transport the pack in a box with no metal; tape over the ends (masking tape) so nothing can short them.

### Soldering away from the cells is OK
Once the nickel strip is on (welded or pressed), you may solder **on the strip tabs that stick out past the edge of the pack (≥ 10 mm from the nearest cell)**: balance leads (22 AWG) and main leads. The heat goes into the strip, not into the cell. Soldering on the BMS board is also completely fine with 48 W. 10 AWG can be heavy for 48 W. Use the widest tip, tin the wire first, or use an M4 bolt + ring terminal.

## 4. Pack layout (from the Fusion model "31 Battery Module")

**Orientation (fixed 2026-10-12):** the block lies in front of you with the **black mark at the front-left corner of the top face**. Front row = G1 (slots 1–6, left → right), middle = G2 (7–12), back = G3 (13–18). Bottom view = flipped front-over-back: left stays left, G1 ends up at the back. Interactive diagram: hub → Battery cells.

3 rows of 6 cells, alternating orientation. This way the strip itself makes all series connections. No extra series wires are needed between the groups:

```
          TOP SIDE                                BOTTOM SIDE
  ┌────────────────────────────┐          ┌────────────────────────────┐
  │ G1:  + + + + + +           │ strip A  │ G1:  − − − − − −           │ strip C = PACK −  (B−)
  │ G2:  − − − − − −           │ (G1+/G2−)│ G2:  + + + + + +           │ strip D
  │ G3:  + + + + + +  strip B  │ = PACK + │ G3:  − − − − − −           │ (G2+/G3−)
  └────────────────────────────┘          └────────────────────────────┘
  strip A: covers G1 and G2 (12 cells) → balance B1
  strip B: covers G3 (6 cells)         → PACK + and balance B+
  strip C: covers G1 (6 cells)         → PACK − and BMS B−
  strip D: covers G2 and G3 (12 cells) → balance B2
```
**Strip types (2026-10-12):** Mikkel has **2P** and **1P** strip for integrated holders, 19 mm spacing. A 2P strip is a ladder: two continuous rails joined by rungs. It is meant to lie **across two rows** (one rail on each row's cell ends, the rungs bridging between them), which makes it exactly strip A (rows 1+2, top) and strip D (rows 2+3, bottom). 1P covers one row: strips B (row 3, top) and C (row 1, bottom). Weld spots go on the rails over each cell end; the small slits in the rails sit between the two spots of one cell.

**Pitch (measured 2026-10-12, calipers):** a 1×2 holder is 20.2 × 40.4 mm, so the **cell pitch is 20.2 mm** in both directions (the holders tile edge to edge). The strip (AliExpress, made for one-piece "integrated" holders) is **19 mm**: 1.2 mm short per cell.
- Across the rows this barely matters: each 2P rail lands ~0.6 mm off the cell centre.
- Along a row it adds up: the rungs and slits drift 1.2 mm per cell, ~6 mm by cell 6. The rails are continuous, so every cell end still has rail on it; the welder puts the spots on the rail over each cell end and ignores where the slits fall.
- **Do not cut the strip between cells** to re-space it: the continuous rails are what parallel the cells in a group.
- Alternative if the welder dislikes it: plain nickel strip without holes (0.15 mm, ~8–10 mm wide), laid as rails + short bridges, or the welder's own strip.
- Block size from the pitch: 6 × 20.2 = 121.2 mm by 3 × 20.2 = 60.6 mm. The Fusion "31 Battery Module" (122.7 × 62.1) is ~1.5 mm larger, i.e. the battery mount has a little play.

**Current in strips B and C (estimate, not measured):** a 1P rail (~8 × 0.15 mm pure nickel) carries roughly as much as ~0.3 mm² copper, i.e. a few amps without getting warm. Strips B and C collect the **whole pack current** towards the main leads, so ask the welder for a **double layer** on B and C and a tab for each 10 AWG lead (lead in the middle of the row rather than at one end). The rungs between rows only carry one cell's share (~1/6) and are fine.

## 5. BMS wiring (3S)

```
 PACK −  (strip C)  ──10 AWG── B−   ┐
 strip A (G1+/G2−)  ──22 AWG── B1   │  BMS
 strip D (G2+/G3−)  ──22 AWG── B2   │
 PACK +  (strip B)  ──22 AWG── B+   ┘
                                P− ──10 AWG── XT60 − ──→ robot / charger
 PACK +  (strip B)  ──10 AWG── MAIN FUSE 30 A ── XT60 + ──→ robot / charger
```
- Measure before each lead is connected (Fluke, from B−): B1 ≈ 3.6 V, B2 ≈ 7.2 V, B+ ≈ 10.8 V. If a number is off by more than ~0.1 V from 1×, 2× or 3× the group voltage, STOP: a lead is wrong.
- Order: **B− first**, then B1 → B2 → B+ (low → high). Follow the BMS board's own markings if they say otherwise.
- Some boards must be "woken up" the first time: P− gives 0 V until the charger has been connected for a moment.
- The charger charges through P+/P− (same port as discharge). Balancing only happens at ~4.2 V and is weak → the groups must be matched from the start (§6).

## 6. Matching the cells — EXACTLY like this (done BEFORE anything is connected)

This is risk-free as long as you follow point 0. The cells are not connected to each other, so you measure each cell on its own. It can easily be done while they sit in the click holders.

**0. Safety first**
- Take off your watch, rings and bracelets. Work on a wooden table/paper, not on metal, and clear screws, tools and alligator clips off the table.
- **Fluke leads: black in COM, red in the V/Ω jack.** NEVER red in the "A" or "mA" jack: then the multimeter is a short circuit, and the cell pushes up to 10+ A through the meter (in the best case the fuse blows).
- Rotary switch on **V with a straight line (V⎓ / DC)**, not V~ (AC), and not Ω or diode.

**1. Know the poles**
```
   + end: small button in the middle, surrounded by an insulation ring
          (the metal "shoulder" area around the ring is − !)
   − end: completely flat bottom
```
The insulation ring at + is the only thing that separates + from − on that end. If it is missing or cracked, the cell is a "spare".

**2. Number** the cells 1–20 with a marker on the wrap or on the holder.

**3. Inspect each cell** (turn it in the light): cracks or tears in the plastic wrap (especially at the edges at both ends), dents, rust or discoloration, missing insulation ring. Write "OK" or what you see.

**4. Measure each cell:**
- Black probe on the flat − end, red probe in the middle of the + button. Hold them still, ~2 s.
- Read all decimals (e.g. `3.512`). The Fluke 83 shows 3 decimals in the 4 V range. If it shows "AUTO", it picks the range itself.
- If it shows a minus sign (`-3.512`), you have swapped the probes. That is harmless, just write the number without the minus.
- Write the number in the table:

| Cell | Voltage (V) | Wrap/ring OK? | Note |
|---|---|---|---|
| 1 | | | |
| … | | | |
| 20 | | | |

The measurement log can also be filled in at `hub/` (file `hub/data/battery_cells.csv`, columns `cell;voltage_v;wrap_ok;group;note`; the "Battery cells" tab shows range and spread live).

**5. Send the numbers to Claude.** Then I make the grouping. The limits I use:
- All 3.0–4.2 V and spread ≤ 0.05 V → fine.
- A cell < 2.5 V or > 0.3 V from the others → spare.
- < 2.0 V → must **not** be charged, discard it (battery collection point, not the bin).
- Each group: cells within ~0.05 V of each other. Group averages as equal as possible (ideally ±0.01 V), because the BMS balancing is weak.

New cells from the same batch are typically equal within a few mV. After 17 months of storage I expect ~3.4–3.6 V per cell. That is an estimate, not measured.

## 7. Fuses and main switch

- **Main fuse 30 A** (blade) as close to PACK + as possible. It protects against short circuits in the wiring if the BMS MOSFETs fail during a short. **Needs a blade fuse holder (inline, for 10–12 AWG)**, and there is none in the list → must be bought (~20–40 DKK) or found.
- Branches after the fuse: motor drivers 15 A each (or one 30 A for all four). Logic buck: 5 A (the 5×20 holders are OK here).
- E-stop → drives the 40 A relay (as planned), the relay breaks the main current after the fuse.
- INA219 (max 26 V, shunt typically 0.1 Ω = **max 3.2 A**!) → can NOT sit in the motor current with the standard shunt. It can measure pack voltage and logic current. Software cutoff at ~3.3 V/cell (9.9 V), before the BMS's ~2.5–2.8 V.

## 8. Charging and storage

- Charge on a non-flammable surface (tile/metal tray), only under supervision — not overnight for the first 5 times. Smoke alarm in the room.
- Measure the three group voltages after each of the first charges: all ~4.18–4.20 V, difference < 0.03 V.
- Storage > 1 week: 3.6–3.8 V/cell (~11 V pack), cool, in a metal box/LiPo bag.
- The pack must not get hot. Feel it after driving: lukewarm is OK. If you cannot keep your hand on it → stop and find the bad contact.

## 9. What is missing (to buy)

| Item | Why | Approx. price | Needed? |
|---|---|---|---|
| Inline blade fuse holder 10–12 AWG | Main fuse | 20–40 DKK | **Yes**, unless the car fuse box has a holder that can handle 30 A |
| Fish paper rings 18650 (self-adhesive) | Extra insulation at the + pole | 20–40 DKK | Yes for method 2, recommended for 1 |
| Nickel strip 0.15 mm pure nickel (if it cannot be found) | Connections | 40–80 DKK | If missing |
| Welding service **or** spot welder | Method 1 | Service: unknown (ask). Welder: ~200–600 DKK | Method 1 |
| 4–6 × threaded rod/bolts M4/M5 ~100 mm + nuts | Method 2 | 20–40 DKK | Method 2 |
| Ring terminals M4 for 10 AWG (+ crimper) | Main lead to strip | 20–60 DKK | Recommended |

## 10. Next steps

**Measure first, weld later. Do not take anything to the welder before step 2 is done.**

1. **Now, no risk:** number, inspect and measure all 20 cells (§6). Do not weld or connect anything yet.
2. Send the numbers → Claude assigns the cells to the 3 groups.
3. Click the sorted cells into the holders per §4 and §3a. Only then take them to the welder.
4. Call Fyns Batteri Service (§3a) — if they say yes: price, strip type, and whether they fit the BMS + balance leads.
5. Find the blade fuse holders in the car fuse box, and take a photo of the BMS board (both sides).
6. Measure one motor's winding resistance (§2) → stall current.
7. The final illustrated guide (assembly after welding, BMS, fuse, first charge) is written once steps 1–5 are settled.
