// SENNEP hub — interactive 3S6P battery diagram (Battery cells tab).
//
// Orientation convention (fixed, 2026-10-12): the block lies in front of you with the black mark at the
// FRONT-LEFT corner of the top face. Columns 1–6 run left → right. Rows: front (marked) = G1, middle = G2,
// back = G3. Slots: front 1–6, middle 7–12, back 13–18; "spare" = loose cell.
// Bottom view: default is "see-through" (rows keep their places, as if looking through the block from above).
// "Flipped" = the block turned front-over-back (like turning a page upward): left stays left, row 1 ends up at the back.
"use strict";

const BATT = {
  rows: [ // index 0 = front
    { name: "front", group: "G1", top: "+", slots: [1, 2, 3, 4, 5, 6] },
    { name: "middle", group: "G2", top: "−", slots: [7, 8, 9, 10, 11, 12] },
    { name: "back", group: "G3", top: "+", slots: [13, 14, 15, 16, 17, 18] },
  ],
  // Nickel strips (BATTERY_PLAN §4/§5). rows = indices into BATT.rows.
  strips: {
    top: [
      { id: "A", rows: [0, 1], text: "Strip A · G1+ ↔ G2−", lead: "BMS B1", gauge: "22 AWG", mult: 1 },
      { id: "B", rows: [2], text: "Strip B · G3+ = PACK +", lead: "BMS B+ · 30 A fuse → XT60 +", gauge: "22 + 10 AWG", mult: 3 },
    ],
    bottom: [
      { id: "C", rows: [0], text: "Strip C · G1− = PACK −", lead: "BMS B− (connect first)", gauge: "10 AWG", mult: 0 },
      { id: "D", rows: [1, 2], text: "Strip D · G2+ ↔ G3−", lead: "BMS B2", gauge: "22 AWG", mult: 2 },
    ],
  },
};

function battNum(v) { const x = parseFloat(String(v ?? "").replace(",", ".")); return isNaN(x) ? null : x; }

// Make sure the CSV has a `slot` column (cell n starts in slot n; 19–20 are spares).
function battMigrate(data) {
  if (!data.fields.includes("slot")) {
    data.fields = data.fields.filter(f => f !== "group");
    data.fields.splice(data.fields.indexOf("cell") + 1, 0, "slot");
    for (const r of data.rows) { const n = parseInt(r.cell); r.slot = n >= 1 && n <= 18 ? String(n) : "spare"; delete r.group; }
    return true;
  }
  return false;
}

function batteryDiagram(data, onChange) {
  const migrated = battMigrate(data);
  let state = { side: "top", layer: false, flip: false, sel: null };
  try { Object.assign(state, JSON.parse(localStorage.hubBatt || "{}")); } catch {}
  state.sel = null;
  const persist = () => { try { localStorage.hubBatt = JSON.stringify({ side: state.side, layer: state.layer, flip: state.flip }); } catch {} };

  const el = document.createElement("div");
  el.className = "batt";
  el.innerHTML = `
    <div class="batt-bar">
      <div class="seg" data-k="side"><button data-v="top">Top view</button><button data-v="bottom">Bottom view</button></div>
      <label class="batt-toggle"><input type="checkbox" data-k="layer"> Show connections</label>
      <label class="batt-toggle batt-flip"><input type="checkbox" data-k="flip"> As seen flipped over</label>
      <span class="hint batt-cap"></span>
    </div>
    <div class="batt-main">
      <div class="batt-svg"></div>
      <div class="batt-side"></div>
    </div>
    <div class="batt-edit"></div>`;
  const svgBox = el.querySelector(".batt-svg"), side = el.querySelector(".batt-side"), edit = el.querySelector(".batt-edit");
  el.querySelectorAll(".seg button").forEach(b => b.onclick = () => { state.side = b.dataset.v; persist(); draw(); });
  const layerBox = el.querySelector('[data-k="layer"]');
  layerBox.onchange = () => { state.layer = layerBox.checked; persist(); draw(); };
  const flipBox = el.querySelector('[data-k="flip"]');
  flipBox.onchange = () => { state.flip = flipBox.checked; persist(); draw(); };

  const cellIn = slot => data.rows.find(r => String(r.slot) === String(slot));
  const spares = () => data.rows.filter(r => !(parseInt(r.slot) >= 1 && parseInt(r.slot) <= 18));
  const measured = () => data.rows.map(r => battNum(r.voltage_v)).filter(x => x !== null);
  const median = () => { const v = measured().sort((a, b) => a - b); return v.length ? v[Math.floor((v.length - 1) / 2)] : null; };
  function status(r) {
    const v = battNum(r?.voltage_v);
    if (v === null) return "none";
    if (v < 2.5 || v > 4.2 || r.wrap_ok === "no") return "bad";
    const m = median(), d = Math.abs(v - m);
    return d <= 0.02 ? "good" : d <= 0.05 ? "ok" : "off";
  }
  const rowMean = i => { const v = BATT.rows[i].slots.map(s => battNum(cellIn(s)?.voltage_v)).filter(x => x !== null); return v.length ? { mean: v.reduce((a, b) => a + b, 0) / v.length, n: v.length } : null; };

  function draw() {
    el.querySelectorAll(".seg button").forEach(b => b.classList.toggle("on", b.dataset.v === state.side));
    layerBox.checked = state.layer;
    flipBox.checked = state.flip;
    el.querySelector(".batt-flip").style.display = state.side === "bottom" ? "" : "none";
    el.querySelector(".batt-cap").textContent = state.side === "top"
      ? "Seen from above. Black mark front-left, nearest you."
      : state.flip ? "Block turned front-over-back (like a page): left stays left, row 1 is now at the back."
      : "Underside, see-through: every row stays where it is in the top view.";
    svgBox.innerHTML = svg();
    svgBox.querySelectorAll("[data-cell]").forEach(g => g.addEventListener("click", () => select(g.dataset.cell)));
    sidePanel();
    editor();
  }

  function svg() {
    const P = 84, R = 36, L = 180, T = 34, W = 6 * P + 24, H = 3 * P + 24;
    const bottom = state.side === "bottom", flipped = bottom && state.flip;
    // SVG row order top→bottom: back, middle, front — except the flipped bottom view: front, middle, back.
    const order = flipped ? [0, 1, 2] : [2, 1, 0];
    const yOf = ri => T + 12 + order.indexOf(ri) * P + P / 2;
    const xOf = c => L + 12 + c * P + P / 2;
    let s = `<svg viewBox="0 0 ${L + W + 250} ${T + H + 140}" role="img" aria-label="Battery ${state.side} view">
      <defs>
        <radialGradient id="metal" cx="40%" cy="35%" r="70%"><stop offset="0" stop-color="#f4f4f1"/><stop offset=".6" stop-color="#c9c9c3"/><stop offset="1" stop-color="#9d9d97"/></radialGradient>
        <radialGradient id="metal2" cx="40%" cy="35%" r="70%"><stop offset="0" stop-color="#ecebe6"/><stop offset="1" stop-color="#b8b6ac"/></radialGradient>
        <pattern id="nickel" width="8" height="8" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
          <rect width="8" height="8" fill="#cfd3d6"/><rect width="3" height="8" fill="#b7bcc0"/></pattern>
        <filter id="sh" x="-10%" y="-10%" width="120%" height="130%"><feDropShadow dx="0" dy="3" stdDeviation="4" flood-opacity=".25"/></filter>
      </defs>`;
    // holder block
    s += `<rect x="${L}" y="${T}" width="${W}" height="${H}" rx="14" fill="#1b1c1e" filter="url(#sh)"/>`;
    // row labels
    for (const ri of [0, 1, 2]) {
      const r = BATT.rows[ri], pol = bottom ? (r.top === "+" ? "−" : "+") : r.top, m = rowMean(ri);
      s += `<text x="${L - 14}" y="${yOf(ri) - 6}" text-anchor="end" class="b-row">Row ${ri + 1} · ${r.group}</text>
            <text x="${L - 14}" y="${yOf(ri) + 14}" text-anchor="end" class="b-sub">${r.name} · ${pol} ${bottom ? "end" : "up"}${m ? ` · ⌀ ${m.mean.toFixed(3)} V` : ""}</text>`;
    }
    // cells
    for (const ri of [0, 1, 2]) {
      const row = BATT.rows[ri], pol = bottom ? (row.top === "+" ? "−" : "+") : row.top;
      row.slots.forEach((slot, c) => {
        const cell = cellIn(slot), x = xOf(c), y = yOf(ri), st = status(cell), sel = cell && state.sel === cell.cell;
        s += `<g class="b-cell ${sel ? "sel" : ""}" data-cell="${cell ? cell.cell : ""}" transform="translate(${x},${y})">
          <circle r="${R + 5}" class="halo st-${st}"/>
          <circle r="${R}" fill="#e58bb8"/>`;
        s += pol === "+"
          ? `<circle r="${R - 6}" fill="#f7f5ef"/><circle r="${R - 13}" fill="url(#metal)"/><circle r="${R - 19}" fill="none" stroke="#8d8c86" stroke-width="1"/>`
          : `<circle r="${R - 4}" fill="url(#metal2)"/>`;
        const v = battNum(cell?.voltage_v);
        s += `<text y="-${R - 22}" class="b-num">${cell ? cell.cell : "–"}</text>
          <text y="${v !== null ? 8 : 6}" class="b-volt ${v === null ? "empty" : ""}">${v !== null ? v.toFixed(3) : pol}</text>
        </g>`;
      });
    }
    // mark
    if (!bottom) s += `<path d="M${L} ${T + H - 34} v24 a10 10 0 0 0 10 10 h24" fill="none" stroke="#000" stroke-width="10" stroke-linecap="round"/>
        <text x="${L + 4}" y="${T + H + 22}" class="b-sub">◤ black mark</text>`;
    else if (flipped) s += `<path d="M${L} ${T + 34} v-24 a10 10 0 0 1 10 -10 h24" fill="none" stroke="#000" stroke-width="6" stroke-dasharray="4 5"/>
        <text x="${L + 4}" y="${T - 10}" class="b-sub">mark is underneath here</text>`;
    else s += `<path d="M${L} ${T + H - 34} v24 a10 10 0 0 0 10 10 h24" fill="none" stroke="#000" stroke-width="6" stroke-dasharray="4 5"/>
        <text x="${L + 4}" y="${T + H + 22}" class="b-sub">◤ mark (on the top face)</text>`;
    // strips layer
    if (state.layer) {
      const strips = BATT.strips[state.side];
      s += `<g pointer-events="none">`;
      strips.forEach((st, i) => {
        const ys = st.rows.map(yOf), y0 = Math.min(...ys) - P / 2 + 8, y1 = Math.max(...ys) + P / 2 - 8;
        const means = [0, 1, 2].map(rowMean);
        const expect = st.mult === 0 ? 0 : means.slice(0, st.mult).every(Boolean) ? means.slice(0, st.mult).reduce((a, m) => a + m.mean, 0) : null;
        const col = { A: "#3b82c4", B: "#d64545", C: "#222", D: "#2e9d5b" }[st.id];
        s += `<rect x="${L + 10}" y="${y0}" width="${W - 20}" height="${y1 - y0}" rx="10" fill="url(#nickel)" opacity=".55" class="b-strip"/>
          <rect x="${L + 10}" y="${y0}" width="${W - 20}" height="${y1 - y0}" rx="10" fill="none" stroke="${col}" stroke-width="4"/>
          <text x="${L + 22}" y="${y0 + 18}" class="b-strip-id" fill="${col}">${st.id}</text>`;
        const ly = (y0 + y1) / 2, lx = L + W + 18;
        s += `<line x1="${L + W - 10}" y1="${ly}" x2="${lx + 4}" y2="${ly}" class="b-lead" />
          <circle cx="${L + W - 10}" cy="${ly}" r="5" fill="#d64545"/>
          <text x="${lx + 10}" y="${ly - 14}" class="b-strip-t">${st.text}</text>
          <text x="${lx + 10}" y="${ly + 4}" class="b-sub">→ ${st.lead}</text>
          <text x="${lx + 10}" y="${ly + 21}" class="b-sub">${st.gauge}${expect !== null ? ` · expect ≈ ${expect.toFixed(2)} V from B−` : ` · expect ${st.mult}× cell V from B−`}</text>`;
      });
      s += `</g>`;
    }
    if (state.layer) s += `<text x="${L + W / 2}" y="${T - 14}" text-anchor="middle" class="b-sub">Series path: PACK − → C → row 1 → A (top) → row 2 → D (bottom) → row 3 → B (top) → PACK +</text>`;
    // "you" marker
    s += `<text x="${L + W / 2}" y="${T + H + 30}" text-anchor="middle" class="b-you">▼ ${flipped ? "back edge of the block (originally)" : "front edge · you sit here"}</text>`;
    // spares
    const sp = spares();
    if (sp.length) {
      s += `<text x="${L}" y="${T + H + 70}" class="b-row">Spares</text>`;
      sp.forEach((cell, i) => {
        const x = L + 110 + i * 92, y = T + H + 92, st = status(cell), v = battNum(cell.voltage_v), sel = state.sel === cell.cell;
        s += `<g class="b-cell ${sel ? "sel" : ""}" data-cell="${cell.cell}" transform="translate(${x},${y})">
          <rect x="-44" y="-20" width="88" height="40" rx="20" class="halo st-${st}"/>
          <rect x="-40" y="-16" width="80" height="32" rx="16" fill="#e58bb8"/>
          <text x="-24" y="5" class="b-num" style="font-size:12px">${cell.cell}</text>
          <text x="10" y="5" class="b-volt ${v === null ? "empty" : ""}" style="font-size:12px">${v !== null ? v.toFixed(3) : "—"}</text></g>`;
      });
    }
    return s + "</svg>";
  }

  function sidePanel() {
    const v = measured(), n = data.rows.length;
    const means = [0, 1, 2].map(rowMean);
    const full = means.every(m => m && m.n === 6);
    const spreadG = full ? Math.max(...means.map(m => m.mean)) - Math.min(...means.map(m => m.mean)) : null;
    const lo = v.length ? Math.min(...v) : null, hi = v.length ? Math.max(...v) : null;
    side.innerHTML = `
      <div class="b-stat"><span>Measured</span><b>${v.length} / ${n}</b></div>
      <div class="b-stat"><span>Range</span><b>${v.length ? lo.toFixed(3) + "–" + hi.toFixed(3) + " V" : "—"}</b></div>
      <div class="b-stat"><span>Spread (incl. spares)</span><b class="${v.length > 1 ? (hi - lo <= 0.05 ? "okc" : "warn") : ""}">${v.length > 1 ? (hi - lo).toFixed(3) + " V" : "—"}</b></div>
      ${[0, 1, 2].map(i => `<div class="b-stat"><span>${BATT.rows[i].group} mean</span><b>${means[i] ? means[i].mean.toFixed(3) + " V" : "—"}</b></div>`).join("")}
      <div class="b-stat"><span>Group means differ</span><b class="${spreadG === null ? "" : spreadG <= 0.01 ? "okc" : "warn"}">${spreadG === null ? "after all 18" : spreadG.toFixed(3) + " V"}</b></div>
      <div class="b-legend">
        <div><i class="st-good"></i> within 0.02 V of median</div>
        <div><i class="st-ok"></i> within 0.05 V</div>
        <div><i class="st-off"></i> further off → maybe spare</div>
        <div><i class="st-bad"></i> &lt; 2.5 V, &gt; 4.2 V or damaged wrap</div>
      </div>
      <p class="hint">Targets: cells in a group within 0.05 V; group means within 0.01 V. Click a cell to enter its reading — Enter jumps to the next.</p>`;
  }

  function select(cellNo) { state.sel = cellNo || null; draw(); }

  function editor() {
    edit.innerHTML = "";
    const r = data.rows.find(x => x.cell === state.sel);
    if (!r) { edit.innerHTML = `<p class="hint">Click a cell in the diagram to log its voltage.</p>`; return; }
    const slot = parseInt(r.slot), ri = BATT.rows.findIndex(x => x.slots.includes(slot));
    const where = ri >= 0 ? `${BATT.rows[ri].group} · ${BATT.rows[ri].name} row · column ${BATT.rows[ri].slots.indexOf(slot) + 1} from the left` : "spare cell";
    edit.innerHTML = `
      <div class="b-form">
        <div class="b-title">Cell <b>${r.cell}</b> <span class="hint">${where}</span></div>
        <label>Voltage (V)<input class="b-v" inputmode="decimal" placeholder="e.g. 3.52" value="${r.voltage_v || ""}"></label>
        <label>Wrap + ring OK?<select class="b-w"><option value=""></option><option>yes</option><option>no</option></select></label>
        <label class="grow">Note<input class="b-n" value="${(r.note || "").replace(/"/g, "&quot;")}"></label>
        <div class="b-btns"><button class="ghost b-prev">← Prev</button><button class="btn b-next">Next →</button></div>
      </div>`;
    const vIn = edit.querySelector(".b-v"), wIn = edit.querySelector(".b-w"), nIn = edit.querySelector(".b-n");
    wIn.value = r.wrap_ok || "";
    const commit = () => {
      const nv = vIn.value.trim().replace(",", "."), changed = nv !== (r.voltage_v || "") || wIn.value !== (r.wrap_ok || "") || nIn.value !== (r.note || "");
      r.voltage_v = nv; r.wrap_ok = wIn.value; r.note = nIn.value;
      if (changed) onChange();
    };
    const order = data.rows.map(x => x.cell).sort((a, b) => a - b);
    const go = d => { commit(); const i = order.indexOf(r.cell); state.sel = order[(i + d + order.length) % order.length]; draw(); edit.querySelector(".b-v").focus(); };
    vIn.addEventListener("keydown", e => { if (e.key === "Enter") { e.preventDefault(); go(1); } });
    for (const inp of [vIn, wIn, nIn]) inp.addEventListener("change", () => { commit(); svgBox.innerHTML = svg(); svgBox.querySelectorAll("[data-cell]").forEach(g => g.addEventListener("click", () => select(g.dataset.cell))); sidePanel(); });
    edit.querySelector(".b-prev").onclick = () => go(-1);
    edit.querySelector(".b-next").onclick = () => go(1);
    vIn.focus(); vIn.select();
  }

  draw();
  if (migrated) onChange();
  return { el, refresh: draw };
}
