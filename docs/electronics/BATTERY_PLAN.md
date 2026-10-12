# Batteriplan — 3S6P (18 × Samsung 35E)

> Status: **UDKAST v2 (2026-10-12)**. Mikkel går efter **punktsvejsning hos et værksted i Odense** (§3a). Kompressionspakken (metode 2) er reserveløsning. Først når planen er godkendt, skrives den endelige trin-for-trin-guide.

## 1. Status: hvad vi ved

| Del | Hvad vi har | Kilde | Bemærkning |
|---|---|---|---|
| Celler | 20 × Samsung INR18650-35E, **nye og aldrig brugt** (købt maj 2025). Der skal bruges 18 → 2 i reserve | inventory + Mikkel | 3,6 V nom., 4,20 V ladespænding, 2,50 V cutoff, **maks. 8 A afladning**, maks. 1,5 A ladning pr. celle |
| Holdere | 20 × "1×2-holder m. afstandsstykke" (kliks-plastik) | inventory | **Ingen metalkontakter.** De holder kun cellerne på plads |
| Forbindelser | **Ingen.** Cellerne sidder bare i holderne, og der er hverken strip, lod eller svejs | Mikkel | Alle 36 celleender skal forbindes, ikke kun 2 serieled |
| Nikkelstrip | Ren nikkel 0,15 × 27 mm "2H-19 mm" (købt maj 2025) | Mikkel | Findes, men stykkerne er **for små til at dække en række** → skal klippes og svejses sammen. Værkstedet bruger nok sin egen strip |
| BMS | 2 × "3S 40 A BMS m. balance" (Bot'n Roll) | inventory | Præcis model ukendt. Kort af den type: 40 A kontinuert (kræver køling), ofte ~80 A startstrøm, passiv balance ~40–100 mA, kun ved fuld ladning |
| Lader | 3S Li-ion-lader 12,6 V 2 A | inventory | Passende. 20 Ah / 2 A ≈ 10–11 t fra tom |
| Kabel | 10 AWG silikone (3 × 2 m), 16 AWG (10 m), 22 AWG (50 m) | inventory | 10 AWG til hovedstrøm, 22 AWG til balanceledninger |
| Stik | 2 × XT60-par | inventory | XT60 er godt til batteri ↔ robot og lader |
| Sikringer | Blade-sikringer (180 stk sortiment) + **måske blade-holdere** i samme kasse, 10 × 5×20 mm glassikringsholder | inventory + Mikkel | Holderne skal findes, og deres rating tjekkes (står ofte på holderen/ledningen). 5×20-holdere er typisk max ~10 A → ikke til hovedsikringen |
| Andet | Nødstop 40 mm + 40 A-relæ, INA219, voltmeter-display | inventory | |
| Værktøj | **Fluke 83** (fremragende), loddestation 48 W (Puctec ZD-99), afbidertang | Mikkel | Ingen punktsvejser, ingen crimptang |

**Ukendt / skal tjekkes:** Hvilken BMS-model er det (foto af printet med pads)? Hvilke blade-holdere er der i kassen (mini/standard, ledningstykkelse)? Lange bolte: **nej** (påvirker kun metode 2). Fiskepapir: ukendt.

## 2. Elektriske tal

```
3S6P:  3 grupper i serie, 6 celler parallelt i hver
       10,8 V nominelt · 12,6 V fuld · ~9,0–9,9 V tom · ~20,4 Ah · ~220 Wh · ~0,9 kg celler
Maks. afladning pr. celle 8 A  →  pakken teoretisk 48 A
BMS 40 A                       →  6,7 A pr. celle (OK)
Ladning 2 A                    →  0,33 A pr. celle (meget skånsomt)
```

**Motorer (4 × RS 834-7641):** 41,3 W ud ved 60 % virkningsgrad ≈ 69 W ind ≈ **~5,7 A pr. motor ved mærkelast** (en lignende 42 mm 49:1-motor er opgivet til 5,5 A). **Stall-strømmen er ikke opgivet** i RS-databladet. Lignende motorer ligger på ~12–25 A, så det tal er usikkert. Fire motorer, der starter eller blokerer samtidig, kan altså trække 50–100 A i et kort øjeblik. Det er mere end BMS'en kan levere.
→ **Mål det selv:** mål motorens viklingsmodstand R med Fluke 83 (Ω-område, træk prøveledningernes modstand fra). Så er stall ≈ 12,6 V / R. Det er sikkert og kræver ingen strøm.
→ **Konsekvens:** soft-start/rampe i motorkoden er obligatorisk. Uden den slår BMS'en fra ved hver hård start, og det er generende men ikke farligt. Normal kørsel (~10–20 A i alt) er fint for både celler og BMS.

## 3. Metoder til at forbinde cellerne — rangeret efter sikkerhed

| # | Metode | Sikkerhed | Har han det? | Vurdering |
|---|---|---|---|---|
| **1** | **Punktsvejsning af nikkelstrip** — enten få et værksted til det (elcykelværksted/batteributik, 36 svejsninger ≈ 15 min arbejde) eller købe en punktsvejser | ★★★★★ | Strip: måske. Svejser: nej | Standardmetoden og den mest robuste mod vibrationer. Næsten ingen varme går ind i cellen. **Anbefales som den permanente løsning** |
| **2** | **Kompressionspakke** — nikkelstrip presses mod celleenderne af printede endeplader, der spændes sammen med bolte/gevindstang. Fjedring fra TPU-puder eller tallerkenfjedre | ★★★☆☆ | Printer, PETG, strip (?), M3-indsatser. Mangler sandsynligvis lange bolte + fiskepapir | Kan laves af det, han har (næsten). Holder kun, hvis trykket holdes: PETG kryber under varme og tryk, og vibrationer kan løsne kontakten → modstand → varme. **OK til at komme i gang, men kun med måling af spændingsfald pr. kontakt og periodisk eftersyn** |
| 3 | Lodde direkte på cellerne | ★★☆☆☆ | 48 W er **for lidt** | Kræver en kraftig kolbe (≥ 80–100 W, bred spids, flux), så hver lodning tager under 3 s. Med 48 W varmer man for længe, og det skader cellens indre (separator og sikkerhedsventil). **Ikke anbefalet, og ikke med det udstyr, han har** |
| ✗ | Magneter (neodym) mellem celle og strip | Farligt | — | Høj kontaktmodstand. Neodym mister magnetisme ved ~80 °C. De kan glide → kortslutning. **Afvist** |
| ✗ | Strip lagt løst i klikholderne / krokodillenæb | Farligt | — | Holderne giver intet tryk. Krokodillenæb kan kun klare få A og kan falde af → kortslutning. **Afvist**, også "bare til en test" |
| ✗ | Fjeder-batterikasser (1×N plastkasser) | Dårligt | Nej | Fjedrene kan typisk klare 2–3 A og bliver varme. For svage til motorer |

**Ærlig vurdering:** Den hurtigste og sikreste vej er metode 1 via et værksted. Det koster sandsynligvis mindre end en svejser, og du slipper for selv at røre cellerne med varme. Metode 2 er "lav det nu med det, du har". Den kan fungere, men den kræver omhu og målinger. Den hører hjemme på en robot, der står meget og kører roligt, ikke på én, der kører hårdt over dørtrin.

### 3a. Svejseservice i Odense (Mikkels valg)

Fundet ved websøgning 2026-10-12. **Jeg kunne ikke åbne siderne selv** (netværksblokering), så adresse og ydelser er fra søgeresultater og skal bekræftes:

| Sted | Hvad vi ved | Chance |
|---|---|---|
| **Fyns Batteri Service ApS**, Knullen 8, Odense S | Renoverer elcykelbatterier (= de punktsvejser 18650-pakker). Priser "fra 1.750 kr" gælder hel renovering inkl. celler, ikke svejsning af egne celler | Bedst. Ring og spørg |
| Elcykelværksteder i Odense (fx via Handyhand) | Mange sender batterier videre i stedet for at svejse selv | Lav–middel |
| Himex (elcykel-webshop, har batteriservice) | Placering ikke bekræftet | Ukendt |
| SDU MakerSpace (Videnbyen) | Lodning, 3D-print, laser. Punktsvejser ikke nævnt. Kun for SDU-studerende | Kun hvis du er SDU-studerende |
| Odense Bibliotek Makerspace | Maskinlisten kunne ikke hentes | Spørg |

**Hvad du siger, når du ringer (forslag):**
> "Jeg har 18 nye Samsung 35E-celler (18650), som jeg vil have punktsvejset til en 3S6P-pakke (12 V) til en robot. Cellerne er målt og sorteret, og de sidder i klik-holdere i den rigtige retning. Kan I punktsvejse nikkelstrip på (jeres strip eller min 0,15 mm rene nikkel), og evt. sætte balanceledninger på? Hvad koster det, og hvor lang tid tager det?"

Spørg også: **ren nikkel eller forniklet stål?** (Kun ren nikkel. Forniklet stål har ~5× højere modstand og bliver varmt.) Tag ringe på + (fiskepapir) med, hvis de har det. **Tag gerne BMS'en med**, hvis de også vil montere den.

Det er ikke sikkert, at de vil svejse kundens egne celler (ansvar). Siger alle nej → køb en punktsvejser (metode 1b) eller lav kompressionspakken (metode 2).

**Forberedelse før værkstedet** (så de kun skal svejse):
1. Mål og sortér cellerne (§6) → 3 grupper á 6 med ens gennemsnit.
2. Klik pakken sammen præcis efter layoutet i §4 (G1 + op, G2 − op, G3 + op). **Tjek retningen af hver eneste celle to gange** — én vendt celle i en parallelgruppe = kortslutning i det øjeblik, strippen svejses på.
3. Skriv "+ PAKKE" / "− PAKKE" og "G1/G2/G3" med tusch på holderne.
4. Transportér pakken i en kasse uden metal; tape over enderne (malertape) så intet kan kortslutte dem.

### Lodning væk fra cellerne er OK
Når nikkelstrippen først sidder på (svejset eller presset), må der gerne loddes **på stripens flapper, der stikker ud over pakkens kant (≥ 10 mm fra nærmeste celle)**: balanceledninger (22 AWG) og hovedledninger. Varmen går ind i strippen, ikke ind i cellen. Lodning på BMS-printet er også helt fint med 48 W. 10 AWG kan være tungt for 48 W. Brug den bredeste spids, fortin ledningen først, eller brug en M4-bolt + ringterminal.

## 4. Pakke-layout (fra Fusion-modellen "31 Battery Module")

3 rækker á 6 celler, vekslende retning. Så klares alle serieforbindelser af selve strippen. Der skal ikke være ekstra serieledninger mellem grupperne:

```
          TOP-SIDE                                BUND-SIDE
  ┌────────────────────────────┐          ┌────────────────────────────┐
  │ G1:  + + + + + +           │ strip A  │ G1:  − − − − − −           │ strip C = PAKKE −  (B−)
  │ G2:  − − − − − −           │ (G1+/G2−)│ G2:  + + + + + +           │ strip D
  │ G3:  + + + + + +  strip B  │ = PAKKE+ │ G3:  − − − − − −           │ (G2+/G3−)
  └────────────────────────────┘          └────────────────────────────┘
  strip A: dækker G1 og G2 (12 celler) → balance B1
  strip B: dækker G3 (6 celler)        → PAKKE + og balance B+
  strip C: dækker G1 (6 celler)        → PAKKE − og BMS B−
  strip D: dækker G2 og G3 (12 celler) → balance B2
```
**Skal tjekkes:** at stripens hulafstand passer til holdernes celleafstand (modellen siger ~20,5–20,7 mm, "2H-19 mm" tyder på 19 mm). Passer de ikke, skal strippen klippes og lægges som rækker.

## 5. BMS-ledninger (3S)

```
 PAKKE −  (strip C) ──10 AWG── B−   ┐
 strip A (G1+/G2−)  ──22 AWG── B1   │  BMS
 strip D (G2+/G3−)  ──22 AWG── B2   │
 PAKKE +  (strip B) ──22 AWG── B+   ┘
                                P− ──10 AWG── XT60 − ──→ robot / lader
 PAKKE +  (strip B) ──10 AWG── HOVEDSIKRING 30 A ── XT60 + ──→ robot / lader
```
- Mål før hver ledning sættes på (Fluke, fra B−): B1 ≈ 3,6 V, B2 ≈ 7,2 V, B+ ≈ 10,8 V. Afviger et tal mere end ~0,1 V fra 1×, 2× eller 3× gruppespændingen, så STOP: så er en ledning forkert.
- Rækkefølge: **B− først**, derefter B1 → B2 → B+ (lav → høj). Følg BMS-kortets egen påskrift, hvis den siger noget andet.
- Nogle kort skal "vækkes" første gang: P− giver 0 V, indtil laderen har været sat til et øjeblik.
- Laderen lader gennem P+/P− (samme port som afladning). Balanceringen sker kun ved ~4,2 V og er svag → grupperne skal være matchet fra start (§6).

## 6. Matching af celler — PRÆCIS sådan (gøres FØR noget forbindes)

Det her er risikofrit, så længe du følger punkt 0. Cellerne er ikke forbundet med hinanden, så du måler hver celle for sig. Det kan sagtens gøres, mens de sidder i klikholderne.

**0. Sikkerhed først**
- Tag ur, ringe og armbånd af. Arbejd på et træbord/papir, ikke på metal, og fjern skruer, værktøj og krokodillenæb fra bordet.
- **Fluke'ens ledninger: sort i COM, rød i V/Ω-bøsningen.** ALDRIG rød i "A"- eller "mA"-bøsningen: så er multimeteret en kortslutning, og cellen sender op til 10+ A gennem måleren (sikringen springer i bedste fald).
- Drejeknappen på **V med lige streg (V⎓ / DC)**, ikke V~ (AC), og ikke Ω eller dioden.

**1. Kend polerne**
```
   + ende: lille knap i midten, omgivet af en isolationsring
           (det metalliske "skulder"-område rundt om ringen er − !)
   − ende: helt flad bund
```
Isolationsringen ved + er det eneste, der adskiller + fra − på den ende. Er den væk eller revnet, er cellen "reserve".

**2. Nummerér** cellerne 1–20 med tusch på folien eller på holderen.

**3. Efterse hver celle** (vend den i lyset): revner eller flænger i plastfolien (især ved kanterne i begge ender), buler, rust eller misfarvning, mangler isolationsring. Notér "OK" eller hvad du ser.

**4. Mål hver celle:**
- Sort spids på den flade − ende, rød spids midt på + knappen. Hold dem stille, ~2 sek.
- Aflæs med alle decimaler (fx `3.512`). Fluke 83 viser 3 decimaler i 4 V-området. Står der "AUTO", vælger den selv.
- Viser den et minustegn (`-3.512`), har du byttet spidserne om. Det er ufarligt, skriv bare tallet uden minus.
- Skriv tallet i skemaet:

| Celle | Spænding (V) | Folie/ring OK? | Note |
|---|---|---|---|
| 1 | | | |
| … | | | |
| 20 | | | |

**5. Send tallene til Claude.** Så laver jeg fordelingen. Grænserne jeg bruger:
- Alle 3,0–4,2 V og spredning ≤ 0,05 V → fint.
- En celle < 2,5 V eller > 0,3 V fra de andre → reserve.
- < 2,0 V → må **ikke** lades, kasseres (batteri-indsamling, ikke skraldespanden).
- Hver gruppe: celler inden for ~0,05 V af hinanden. Gruppernes gennemsnit så ens som muligt (helst ±0,01 V), fordi BMS'ens balancering er svag.

Nye celler fra samme batch er typisk ens inden for få mV. Efter 17 måneders opbevaring regner jeg med ~3,4–3,6 V pr. celle. Det er et estimat, ikke målt.

## 7. Sikringer og hovedafbryder

- **Hovedsikring 30 A** (blade) så tæt på PAKKE + som muligt. Den beskytter mod kortslutning i ledninger, hvis BMS-MOSFET'erne svigter ved en kortslutning. **Kræver en blade-sikringsholder (inline, til 10–12 AWG)**, og den er der ikke i listen → skal købes (~20–40 kr) eller findes.
- Grene efter sikringen: motordrivere 15 A hver (eller én 30 A for alle fire). Logik-buck: 5 A (5×20-holderne er OK her).
- Nødstop → styrer 40 A-relæet (som planlagt), relæet bryder hovedstrømmen efter sikringen.
- INA219 (maks 26 V, shunt typisk 0,1 Ω = **max 3,2 A**!) → kan IKKE sidde i motorstrømmen med standardshunten. Den kan måle pakkespænding og logikstrøm. Software-cutoff ved ~3,3 V/celle (9,9 V), før BMS'ens ~2,5–2,8 V.

## 8. Ladning og opbevaring

- Lad på et ikke-brændbart underlag (flise/metalbakke), kun under opsyn — ikke om natten de første 5 gange. Røgalarm i rummet.
- Mål de tre gruppespændinger efter hver af de første ladninger: alle ~4,18–4,20 V, forskel < 0,03 V.
- Opbevaring > 1 uge: 3,6–3,8 V/celle (~11 V pakke), køligt, i metalkasse/LiPo-pose.
- Pakken må ikke blive varm. Mærk efter kørsel: lunken er OK. Kan du ikke holde hånden på den → stop og find kontakten.

## 9. Hvad mangler (indkøb)

| Ting | Hvorfor | Ca. pris | Nødvendig? |
|---|---|---|---|
| Inline blade-sikringsholder 10–12 AWG | Hovedsikring | 20–40 kr | **Ja**, medmindre kassen med bilsikringer har en holder, der kan klare 30 A |
| Fiskepapir-ringe 18650 (selvklæbende) | Ekstra isolering ved +polen | 20–40 kr | Ja ved metode 2, anbefalet ved 1 |
| Nikkelstrip 0,15 mm ren nikkel (hvis den ikke findes) | Forbindelser | 40–80 kr | Hvis den mangler |
| Svejseservice **eller** punktsvejser | Metode 1 | Service: ukendt (spørg). Svejser: ~200–600 kr | Metode 1 |
| 4–6 × gevindstang/bolte M4/M5 ~100 mm + møtrikker | Metode 2 | 20–40 kr | Metode 2 |
| Ringterminaler M4 til 10 AWG (+ tang) | Hovedledning til strip | 20–60 kr | Anbefalet |

## 10. Næste skridt

1. **Nu, uden risiko:** nummerér, efterse og mål alle 20 celler (§6). Send tallene → Claude fordeler dem i grupper.
2. Ring til Fyns Batteri Service (§3a) — svarer de ja: pris, strip-type, og om de monterer BMS + balanceledninger.
3. Find blade-sikringsholderne i bilsikringskassen, og tag et foto af BMS-printet (begge sider).
4. Mål én motors viklingsmodstand (§2) → stall-strøm.
5. Endelig illustreret guide (samling efter svejsning, BMS, sikring, første ladning) skrives, når 1–3 er afklaret.
