# Handoff — batteri-sessionen (2026-10-12)

Cloud-session på branch `claude/nifty-faraday-doit7x` (én af 6 parallelle sessioner). Hovedresultat: `docs/electronics/BATTERY_PLAN.md` (udkast v2, **ikke godkendt endnu**). Mikkel har **ikke læst** den sidste chatbesked, så alt nedenfor er nyt for ham.

## Opgaven (fra Mikkel)
Få 3S6P-pakken (18 × Samsung 35E) til at virke NU med det, han har, sikkert. Han er utryg ved at lodde eller svejse. Arbejdsrækkefølge: status → research → plan i `docs/electronics/BATTERY_PLAN.md` → AskUserQuestion-runder → **først efter godkendelse** skrives en endelig illustreret trin-for-trin-guide (eksplicit om farer: kortslutning, vendte celler, beskadiget folie; ingen usikre genveje).

## Mikkels svar (AskUserQuestion, 2 runder)
**Runde 1**
- P-grupper: **"Kun klikset i holdere"**. Der er ingen strip, lod eller svejs. Alle 36 celleender mangler forbindelse. (Tidligere noter påstod, at P-grupperne var samlet. Det er nu rettet i PROJECT_OVERVIEW §8 og inventory.)
- Celler: **nye, aldrig brugt** (købt maj 2025).
- Værktøj: "Min loddekolbe er kun 48 W – Puctec soldering station model ZD-99 … Mit multimeter er ret høj klasse: **Fluke 83**." Ingen punktsvejser og ingen crimptang.
- Materialer uden for listen: "Ved ikke."

**Runde 2**
- Metode: **"Undersøg hvor i Odense jeg kan få det svejset."** → valgt retning: punktsvejsning hos et værksted.
- Materialer: "De nikkelstrips jeg har købt er ikke store nok til at kunne ligge på batteriet – så skal de klippes op og svejses/loddes sammen separat. Bolte har jeg ikke nogen lange. Og blade-sikringer, måske?? Jeg har en kasse med car fuses assortment kit. Og nogle holdere dertil, tror jeg."
- Indkøb (~50–80 kr til sikringsholder + fiskepapir): **"Tag højde for begge"** (altså både med og uden indkøb).
- Måling af celler: "Altså, sure… men der er jo ikke noget forbundet. **Du skal forklare præcis hvad jeg skal gøre. Har aldrig gjort det før.**" → En detaljeret målevejledning står i BATTERY_PLAN §6 og blev også sendt i chatten.

## Beslutninger og konklusioner
- Rangering: 1) punktsvejsning (værksted, ellers egen svejser 200–600 kr), 2) printet kompressionspakke (reserve, kræver lange bolte, som han ikke har), 3) direkte lodning på celler — **frarådet, og 48 W er for lidt**. Afvist: magneter, løs strip i klikholdere, krokodillenæb, fjeder-batterikasser.
- Det er OK at lodde på nikkelflapper ≥ 10 mm fra cellerne og på BMS-printet med 48 W. 10 AWG kan være tungt for 48 W.
- Layout (fra Fusion "31 Battery Module", 122,7 × 62,1 × 67,8 mm): 3 rækker á 6, vekslende retning (G1 + op, G2 − op, G3 + op). Fire strips: top A (G1+/G2−, B1), top B (G3+, PAKKE +/B+), bund C (G1−, PAKKE −/B−), bund D (G2+/G3−, B2). Det er ikke bekræftet, at stripens hulafstand (19 mm) passer til holderne (~20,5–20,7 mm).
- BMS-ledninger: B− først, så B1 → B2 → B+. Forventet fra B−: 3,6 / 7,2 / 10,8 V. Nogle kort skal "vækkes" med laderen.
- Hovedsikring 30 A blade tæt på PAKKE +. Nødstop → 40 A-relæ. Software-cutoff ~9,9 V (3,3 V/celle).
- Motorer: ~5,7 A pr. motor ved mærkelast (estimat). **Stall-strømmen er ukendt** (RS-databladets "5,5 mA" er en fejl; lignende motorer har ~12–25 A) → mål viklingsmodstanden med Fluke, stall ≈ 12,6 V / R. Soft-start i motorkoden er obligatorisk (fire samtidige starter > BMS 40 A).
- INA219-modulet (0,1 Ω shunt) kan max ~3,2 A → ikke i motorstrømmen.
- BMS-kort af typen "3S 40 A balance": ~40 A kontinuert med køling, ~80 A start (enhanced), passiv balance ~40–100 mA. Præcis model ukendt.

## Svejseservice i Odense (websøgning; siderne var blokeret fra containeren → IKKE verificeret)
- **Fyns Batteri Service ApS, Knullen 8, Odense S** — renoverer elcykelbatterier. "Fra 1.750 kr" gælder hel renovering med celler. Bedste kandidat. Ring og spørg.
- Elcykelværksteder via Handyhand (Odense V). Himex (har batteriservice, placering ukendt). SDU MakerSpace (lodning, ingen svejser nævnt, kun SDU-studerende). Odense Bibliotek Makerspace (maskinliste ikke hentet).
- Telefon-script og forberedelse (sortér, klik sammen efter layout, dobbelttjek retning, tape enderne) står i BATTERY_PLAN §3a. Spørg om **ren nikkel, ikke forniklet stål**.

## Åbne punkter / næste skridt
1. Mikkel måler alle 20 celler (§6) → Claude fordeler dem i 3 grupper (gennemsnit ±0,01 V, celler inden for 0,05 V).
2. Ring til Fyns Batteri Service: ja/nej, pris, strip-type, vil de montere BMS og balanceledninger?
3. Find blade-holderne i bilsikringskassen (størrelse og rating). Foto af BMS-printets to sider.
4. Mål én motors viklingsmodstand.
5. Opdatér planen → Mikkel godkender → skriv den endelige illustrerede guide (samling efter svejsning, BMS, sikring, første ladning under opsyn, opbevaring 3,6–3,8 V/celle).

## Filer ændret i sessionen
`docs/electronics/BATTERY_PLAN.md` (ny), `ai-instructions/HANDOFF_BATTERY.md` (denne), `ai-instructions/SESSIONS.md` (log), `ai-instructions/PROJECT_OVERVIEW.md` §8 (batterilinjen rettet), `components/inventory.csv` + genereret `INVENTORY.md`/`index.html` (nikkelstrip tilføjet, noter for 18650-35e og holdere rettet).

## Kilder
fynsbatteriservice.dk · handyhand.dk/cykel/elektrisk-cykel-reparation/fyn/odense-v · himex.dk · mitsdu.dk/maker · odensebib.dk/makerspace · sigmanortec.ro (3S 40A BMS) · circuitmall.ng (3S 40A enhanced) · za.rs-online.com/web/p/dc-motors/8347641 · robotshop.com (42 mm 49:1-motor) · components/datasheets/samsung-inr18650-35e.pdf, rs-pro-834-7641-motor.pdf
