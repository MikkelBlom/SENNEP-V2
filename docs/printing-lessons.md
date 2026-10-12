# Printing lessons

What test prints have taught so far. Setup: **PETG, 0.16 mm "Optimal", Bambu X1C, elephant-foot compensation on (0.15 mm recommended)**. Roller sleeves are TPU. The real build limit is **220 mm** on every axis (`MaxPrint`), not the printer's nominal 256 mm.

## Fits
| Clearance | Result |
|---|---|
| 0.25 mm per side | **Too loose.** The joint wobbles (the v2.7 wheel wobbled ~0.25 mm). |
| 0.1 mm per side | Good, tight fit that needs a little force. Default for keys, hex and tubes. |
| 0.35 mm radial | For parts that must turn, e.g. a bayonet. |
| Ø16.9 shaft in a 17 mm bearing | Perfect. Do not change. |
| Motor tube around the gearbox | 0.1 mm (`Chassis_MotorClear`, was 0.2). |

## Overhang
- Flat ceilings in holes and pockets turn into spaghetti, even small ones (the v2.7 bayonet pocket failed this way).
- Anything facing down must be at least 45°. A 45° chamfer **at the build plate** does not help.
- Always check **which face lies on the plate** before adding "printable" geometry.
- Use triangular lugs and grooves instead of square ones.
- Round holes across the print direction up to Ø18 bridge fine, and flat bridges up to ~20.6 mm are accepted by the overhang check. Mikkel wants **round holes, not teardrops**.

## Strength
- Long, thin pins that stick out break (the long Rim A roller pins broke). Keep lugs short with a thick root.
- Printed parts split **along the layers**. M5 thread cut in plastic across the layers split the axle when the set screw was tightened. Use a captured nut or a brass insert instead.
- A nut pocket can be open at one end so the nut slides in after printing (no print pause). It is captured once the screw goes through.
- Think about layer direction for every load-bearing part. Printed M8 bolts are printed **lying on a flat side** so the layers run along the bolt.
- Drilling from the wrong side ruined a v1 housing. Tool access is now designed in (tool channels from the back).

## Design style
- Draw every joint as **one continuous profile** from the start. Stacking fixes on top of each other was rejected as "patch-on-patch".
- Simple, symmetric, "self-built and engineered". Visible bolts and large chamfers are fine.
- Hollowing a part does not automatically save filament: the hollow housing variant used ~5 g **more** (241 g vs 236 g at 3 walls / 15 %), because the chamber walls print solid. Real savings need a smaller outer volume.

## Hardware notes
- M3 brass inserts are melted in with the soldering iron. Use them for joints that are taken apart often.
- Mikkel has taps and dies M3–M12, which makes printed M8 × 1.25 bolts possible: the bolt is a Ø7.85 rod with two flats, the thread is cut with a die, and the nut has a Ø7.0 hole that is tapped.
- Screwdrivers: max Ø16 and 150 mm long, so tool holes can be at most ~100 mm deep and still allow turning.
