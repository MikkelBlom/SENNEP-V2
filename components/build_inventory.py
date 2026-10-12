"""Generate INVENTORY.md and index.html from inventory.csv.

inventory.csv is the single source of truth. Edit it, then run:
    python build_inventory.py
The script also fills the `dkk` column (approximate, fixed rates below) and
warns about datasheet files that are referenced but missing, or present but unused.
"""
import csv
import json
from collections import OrderedDict
from pathlib import Path

ROOT = Path(__file__).parent
CSV_PATH = ROOT / "inventory.csv"
DATASHEET_DIR = ROOT / "datasheets"

# Rates around the purchase dates; the Hobby Components rate matches the invoice total (~880 kr).
RATES_TO_DKK = {"DKK": 1.0, "GBP": 8.67, "EUR": 7.46}

CATEGORY_ORDER = [
    "Compute", "Drivlinje", "Strøm", "Sensorer", "Kommunikation", "UI", "Lyd", "Lys",
    "Styring", "Kontakter", "Kabler", "Passive", "Mekanik", "Forbrug", "Værktøj",
]


def readRows():
    with CSV_PATH.open(encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f, delimiter=";")
        rows = list(reader)
    bad = [r["id"] for r in rows if None in r or None in r.values()]
    if bad:
        raise SystemExit(f"Forkert antal felter (semikolon i en note?): {', '.join(bad)}")
    return reader.fieldnames, rows


def fillDkk(rows):
    for row in rows:
        price, currency = row["linjepris"].strip(), row["valuta"].strip()
        if not price or currency not in RATES_TO_DKK:
            row["dkk"] = ""
            continue
        row["dkk"] = f"{float(price) * RATES_TO_DKK[currency]:.2f}"


def writeRows(fields, rows):
    # Write to a temp file first so a bad row can never truncate the source of truth.
    tmp = CSV_PATH.with_suffix(".tmp")
    with tmp.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields, delimiter=";", lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    tmp.replace(CSV_PATH)


def checkDatasheets(rows):
    referenced = {name for row in rows for name in row["datablade"].split()}
    present = {p.name for p in DATASHEET_DIR.glob("*.pdf")}
    for name in sorted(referenced - present):
        print(f"MANGLER: datasheets/{name}")
    for name in sorted(present - referenced):
        print(f"UBRUGT:  datasheets/{name}")


def groupByCategory(rows):
    groups = OrderedDict((c, []) for c in CATEGORY_ORDER)
    for row in rows:
        groups.setdefault(row["kategori"], []).append(row)
    return OrderedDict((c, r) for c, r in groups.items() if r)


def formatKr(value):
    return f"{value:,.0f}".replace(",", ".") + " kr"


def storeTotals(rows):
    totals = OrderedDict()
    for row in rows:
        if row["dkk"]:
            totals[row["butik"]] = totals.get(row["butik"], 0.0) + float(row["dkk"])
    return totals


def mdLinks(row):
    parts = [f"[{n}](datasheets/{n})" for n in row["datablade"].split()]
    for link in row["links"].split():
        label = "noter" if not link.startswith("http") else "web"
        parts.append(f"[{label}]({link})")
    return " · ".join(parts)


def writeMarkdown(rows):
    groups = groupByCategory(rows)
    lines = [
        "# Komponentliste — SENNEP V2",
        "",
        "> Genereret af `build_inventory.py` fra `inventory.csv`. Redigér CSV'en, ikke denne fil.",
        "",
        f"{len(rows)} linjer · " + " · ".join(f"[{c}](#{c.lower()})" for c in groups),
        "",
        "## Pr. butik (ekskl. fragt)",
        "",
        "| Butik | ≈ DKK |",
        "|---|---:|",
    ]
    lines += [f"| {s} | {formatKr(t)} |" for s, t in storeTotals(rows).items()]
    for category, items in groups.items():
        lines += ["", f"## {category}", "", "| id | Komponent | Antal | ≈ DKK | Butik | Dato | Datablad / links | Noter |", "|---|---|---:|---:|---|---|---|---|"]
        for r in items:
            dkk = formatKr(float(r["dkk"])) if r["dkk"] else "?"
            cells = [f"`{r['id']}`", r["navn"], r["antal"], dkk, r["butik"], r["dato"], mdLinks(r), r["noter"]]
            lines.append("| " + " | ".join(c.replace("|", "/") for c in cells) + " |")
    (ROOT / "INVENTORY.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def writeHtml(rows):
    data = json.dumps(rows, ensure_ascii=False)
    categories = json.dumps(list(groupByCategory(rows).keys()), ensure_ascii=False)
    template = (ROOT / "index.template.html").read_text(encoding="utf-8")
    page = template.replace("/*DATA*/[]", data).replace("/*CATEGORIES*/[]", categories)
    (ROOT / "index.html").write_text(page, encoding="utf-8")


def main():
    fields, rows = readRows()
    fillDkk(rows)
    writeRows(fields, rows)
    checkDatasheets(rows)
    writeMarkdown(rows)
    writeHtml(rows)
    total = sum(float(r["dkk"]) for r in rows if r["dkk"])
    print(f"{len(rows)} komponenter, ≈ {formatKr(total)} (kendte priser, ekskl. fragt)")


if __name__ == "__main__":
    main()
