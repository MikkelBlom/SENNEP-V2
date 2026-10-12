# Prompt til ny chat: SENNEP V2 — skelet, stel og motorophæng

> Kopiér alt under stregen ind i en ny Claude Code-chat i mappen `SENNEP V2`.

---

Du skal hjælpe mig med at designe **skelettet** til min robot SENNEP V2: stel, bundplade og motor-/lejeophæng, der kan bære hele robotten. Hjulet bliver designet i en anden chat. **Du må ikke røre hjulet.**

## Læs først
- `ai-instructions/PROJECT_OVERVIEW.md` — hele projektet: komponenter, motor-datablad (§7d), Fusion-dokumentet (§9) og hjulets historik (§10–12). §12 "v3" har de seneste print-erfaringer.
- `ai-instructions/SESSIONS.md` — hvad der er sket.
- `print/testhjul_v3/LÆS MIG.md` — hvordan hjulet samles i dag.

## Hvad du IKKE må røre
- `fusion/SENNEP_WheelGenerator/` (scriptet) og komponenten **"11 Wheel Lab"** i Fusion. De genbygges af hjulchatten, og dine ændringer bliver slettet.
- Fusion-dokumentet **"Robot v59"** (gammel version, kun til reference).
- Brugerparametre der starter med `Wheel_`, `Axle_`, `Spacer_`, `Motor_`. Læs dem gerne, men ændr dem ikke.
- Arbejd kun i Fusion-dokumentet **"SENNEP V2"**. Tjek at det er det aktive dokument, før et script ændrer noget.

## Grænsefladen til hjulet (det du skal bygge op omkring)
Hent de præcise tal fra parametrene og fra generatorens log. Status nu:

**Hjulet**
- Ø106,2 mm (`Wheel_OD`), 64,1 mm bredt, 9 ruller.
- Rim A vender mod motoren, Rim B udad.
- Der skal bruges venstre- og højrehåndede hjul (`HAND` i generatoren). Det laver hjulchatten. Du skal bare have plads til et hjul pr. hjørne.

**Akslen** (printet PETG, 119,3 mm lang), indefra og ud mod motoren:
1. Hoved Ø42 (yderst på Rim B).
2. Sekskant 18 mm gennem hjulet.
3. Afstandsrør Ø26 × 12,7 mm (`Axle_ToBearing`) ind til lejet.
4. Rund lejesektion Ø16,9 × 19,5 mm i lejet. Den passer perfekt og må ikke ændres.
5. Kobling Ø16,9 × 19 mm (`Axle_CouplingL`) med D-hul til motorakslen (8 mm, D-flade) og en M3-pinolskrue i en indfanget møtrik.

**Lejet:** KFL003, en 17 mm flangeleje med 2 bolte (6 stk købt). **Mål flangen og hulafstanden** på et rigtigt leje, eller spørg mig. Gæt ikke.

**Motoren:** RS PRO 834-7641. Gearkasse Ø42, hus Ø44,6 × 111,9 mm, 624 g, aksel 8 mm × 20 mm. **Mål flangens skruehuller**, eller find dem i databladet (link i §7d).
- Motorens aksel tåler maks. 5 kgf radialt. **Motoren må ikke bære robotten.** Lejet bærer hjulet, og motoren skal kun levere drejningsmoment.
- Motor, leje og hjul skal derfor flugte præcist på én akse. Ellers knækker den printede kobling. Overvej om motoren skal kunne finde sin egen position langs aksen, så den ikke bliver spændt skævt.

## Krav til skelettet
- **Bæreevne:** robot plus last op til **50 kg** samlet. Regn efter og fortæl mig, hvor de svage punkter er.
- **Modulært:** samles af printede dele, der hver passer på pladen (Bambu X1C, 256 × 256 × 256 mm; brug maks. 250). Delene skal kunne udskiftes enkeltvis.
- **Motoren skal slottes direkte ind og låses** med en mekanisme, der er nem at printe og samle. Ingen lim og ingen ekstra indkøb.
- **Kun hardware jeg har:**
  - M3-, M4- og M5-bolte/skruer i forskellige længder, møtrikker, pinolskruer.
  - Messing-gevindindsatser M3, 100 stk. De smeltes i med loddekolben. Brug dem gerne til samlinger, der skal skrues fra hinanden mange gange.
  - Stål- eller aluminiumsprofiler er ikke købt. Foreslå dem kun, hvis print ikke kan klare det, og sig så hvorfor.
- **Plads til alt elektronikken:** batteri 3S6P (20× 18650), 4 motordrivere, Pi 5, Arduino Nano/Mega, LiDAR i midten af toppen med frit 360° udsyn, 7" skærm, kamera, 8 ToF-sensorer rundt om, 4 kantsensorer, blæsere, højttalere, nødstop og relæ. Hele listen står i §4 i PROJECT_OVERVIEW.
- **Mål:** fodaftryk og akselafstand ligger i Fusion-parametrene (`RobotLength`, `RobotWidth`, `Track`, `Wheelbase`, `BodyBottom` osv. — se §9). Størrelsen er ikke endeligt besluttet, så **byg parametrisk**.
- **Hjulkassen** skal give hjulet mindst 2 mm frigang hele vejen rundt.

## Det har vi lært om print (PETG, 0,16 mm Optimal, X1C, elephant foot-kompensation slået til)
**Pasninger:**
- 0,25 mm pr. side er **for løst**. Samlingen slingrer og føles ikke solid.
- 0,1 mm pr. side giver en god, stram pasning, hvor det kræver lidt kraft at samle.
- 0,35 mm radialt til dele, der skal kunne drejes, for eksempel en bajonet.
- En aksel på Ø16,9 i et 17 mm-leje er perfekt.

**Overhang:**
- Flade lofter i huller og lommer bliver til spaghetti, også små lofter.
- Alt der vender nedad skal være mindst 45°. Tjek altid **hvilken flade ligger på printpladen**, før du laver geometri, der skal være "printbar". En 45°-affasning ved selve printpladen hjælper ikke.
- Brug trekantede tapper og riller i stedet for firkantede.

**Styrke:**
- Lange, tynde pinde, der stikker ud, knækker (de lange rullepinde knækkede). Hold udragende tapper korte og med en kraftig rod.
- Printede dele flækker **langs lagene**. Gevind skåret i plast tværs af lagene flækkede akslen, da pinolskruen blev spændt. Brug en indfanget møtrik eller en gevindindsats i stedet.
- En møtriklomme kan være åben i én ende, så møtrikken skubbes ind efter printet. Så kræver den ingen printpause. Den er fanget, når skruen går igennem den.
- Tænk over lagretningen for hver del, der skal tage last.

**Design-stil:**
- Tegn hver samling som **én sammenhængende profil**. Læg ikke flere rettelser oven på hinanden; det blev afvist som "lappeløsninger".
- Simpel, symmetrisk, "selvbygget og engineered". Synlige M3-bolte er OK. Store affasninger er OK.

## Det har vi lært om Fusion MCP og scripting
- Fusion-forbindelsen er claude.ai-connectoren **Autodesk_Fusion** (`fusion_mcp_execute` med `featureType: script`). Scripts skal definere `def run(context)`.
- Store builds: kør **én build pr. MCP-kald**. To builds i ét kald gav timeout.
- Scripts kan ikke køre, mens jeg har en dialog åben i Fusion. Bed mig lukke den.
- **Byg med generator-scripts**, ligesom `fusion/SENNEP_WheelGenerator/`:
  - Parametre i et `PARAMS`-dict, som bliver til Fusion-brugerparametre.
  - Komponenten genbygges på stedet.
  - Geometri laves med TemporaryBRepManager og base features.
  - Fillets via en "try"-funktion, der fortæller om den lykkedes.
  - Automatiske tjek: kollision, frigange, flader med overhang.
- **Fusion-faldgruber:**
  - Indlejrede occurrence-transforms virker kun via `createForAssemblyContext(...).transform2`, og `moveToComponent` nulstiller transformationen.
  - Joints og motion links skal slettes før en genopbygning.
  - Body-navne sættes efter `finishEdit()`.
- **Verificér geometrien med tal**, før du præsenterer den: kollisionsvolumen, overhang-flader, frigange. Eksportér STL til `print/<navn>/` med en `LÆS MIG.md` (orientering, samling, hvad der skal testes).
- Bash-heredocs ødelægger `\n`. Skriv patch-scripts med Write-værktøjet.

## Arbejdsform
- Svar på dansk, kort og direkte. Sig fra, hvis noget er en dårlig idé, og kom med alternativer.
- Spørg, før du gætter på mål du ikke kender: motorflange, lejeflange, batteripakkens mål.
- **Start med et forslag til skelettets opbygning:** hvilke moduler, hvordan de samles, hvordan motor og leje låses, og hvordan lasten går fra hjul til stel. Vis det som tegning eller skitse, **før** du modellerer noget i Fusion.
- Opdatér `ai-instructions/PROJECT_OVERVIEW.md` (en ny sektion) og `SESSIONS.md`, når du har lavet noget.
