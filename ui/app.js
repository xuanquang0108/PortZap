const TAURI = window.__TAURI__;
const invoke = TAURI?.core?.invoke ?? mockInvoke;
const listen = TAURI?.event?.listen ?? (() => {});

const NS = "http://www.w3.org/2000/svg";
const CX = 230;
const CY = 230;
const R_IN = 96;
const R_OUT = 188;
const R_LABEL = 142;
const R_HIT = 142;
const ALL_SLOT = 4;
const RESULT_SLOTS = [0, 1, 2, 3, 5, 6, 7];
const MAX_ROWS = RESULT_SLOTS.length;
const SPAN = 41;
const CORNER = 12;

const DEFAULTS = {
  lang: "vi",
  core_label: "ĐÍCH",
  hint_input: "3000 · node",
  hint_idle: "gõ cổng hoặc tên app",
  hint_keys: "Enter để tắt · Esc đóng",
  killing: "Đang tắt…",
  all_label: "TẤT CẢ",
  all_sub: "diệt tất cả",
  query_failed: "Không truy vấn được",
  kill_failed: "Không tắt được",
  mode_port: "PORT",
  mode_proc: "TIẾN TRÌNH",
  mode_proc_sub: "tên app …",
  mode_hung: "TREO",
  mode_hung_sub: "không phản hồi",
};

const app = document.getElementById("app");
const input = document.getElementById("cmd");
const ring = document.getElementById("ring");
const labels = document.getElementById("labels");
const hits = document.getElementById("hits");
const coreLabel = document.getElementById("core-label");
const contextEl = document.getElementById("context");
const ctxMain = document.getElementById("ctx-main");
const ctxSub = document.getElementById("ctx-sub");

const state = {
  cfg: { ...DEFAULTS },
  rows: [],
  mode: null,
  summary: "",
  busy: false,
  error: "",
  view: "idle",
  hover: null,
  queryGen: 0,
  slotInfo: new Map(),
};

let qtimer = null;

const rad = (deg) => (deg * Math.PI) / 180;

function pt(r, deg) {
  return [CX + r * Math.cos(rad(deg)), CY + r * Math.sin(rad(deg))];
}

function slotAngle(slot) {
  return -90 + slot * 45;
}

function petalPath(slot) {
  const angle = slotAngle(slot);
  const a0 = angle - SPAN / 2;
  const a1 = angle + SPAN / 2;
  const r1 = R_IN;
  const r2 = R_OUT;
  const cr = CORNER;
  const dOut = (cr / r2) * (180 / Math.PI);
  const dIn = (cr / r1) * (180 / Math.PI);
  const p1 = pt(r1, a0);
  const p2 = pt(r2, a0);
  const p3 = pt(r2, a1);
  const p4 = pt(r1, a1);
  const s1 = pt(r2, a0 + dOut);
  const s2 = pt(r2, a1 - dOut);
  const s3 = pt(r2 - cr, a1);
  const s4 = pt(r1 + cr, a1);
  const s5 = pt(r1, a1 - dIn);
  const s6 = pt(r1, a0 + dIn);
  const s7 = pt(r1 + cr, a0);
  const s8 = pt(r2 - cr, a0);
  const n = (v) => v.toFixed(2);
  return [
    `M ${n(s1[0])} ${n(s1[1])}`,
    `A ${r2} ${r2} 0 0 1 ${n(s2[0])} ${n(s2[1])}`,
    `Q ${n(p3[0])} ${n(p3[1])} ${n(s3[0])} ${n(s3[1])}`,
    `L ${n(s4[0])} ${n(s4[1])}`,
    `Q ${n(p4[0])} ${n(p4[1])} ${n(s5[0])} ${n(s5[1])}`,
    `A ${r1} ${r1} 0 0 0 ${n(s6[0])} ${n(s6[1])}`,
    `Q ${n(p1[0])} ${n(p1[1])} ${n(s7[0])} ${n(s7[1])}`,
    `L ${n(s8[0])} ${n(s8[1])}`,
    `Q ${n(p2[0])} ${n(p2[1])} ${n(s1[0])} ${n(s1[1])}`,
    "Z",
  ].join(" ");
}

function modeMeta(key) {
  const c = state.cfg;
  if (key === "port") {
    return { main: c.mode_port, sub: ":3000" };
  }
  if (key === "proc") {
    return { main: c.mode_proc, sub: c.mode_proc_sub };
  }
  return { main: c.mode_hung, sub: c.mode_hung_sub };
}

function viewOf() {
  if (state.rows.length) return "results";
  if (state.mode && input.value.trim()) return "empty";
  return "idle";
}

function slotList(view) {
  if (view === "results") {
    const rows = state.rows.slice(0, MAX_ROWS);
    const out = RESULT_SLOTS.map((slot, i) =>
      i < rows.length ? { slot, kind: "row", row: rows[i] } : { slot, kind: "ghost" }
    );
    out.push({ slot: ALL_SLOT, kind: "all" });
    return out;
  }
  if (view === "empty") {
    return Array.from({ length: 8 }, (_, slot) => ({ slot, kind: "ghost" }));
  }
  return [
    { slot: 0, kind: "mode", key: "port" },
    { slot: 2, kind: "mode", key: "proc" },
    { slot: 6, kind: "mode", key: "hung" },
  ];
}

function mb(bytes) {
  return Math.round((bytes || 0) / (1024 * 1024));
}

function render() {
  const view = viewOf();
  const animate = view !== state.view;
  state.view = view;
  state.hover = null;
  state.slotInfo = new Map();

  ring.innerHTML = "";
  labels.innerHTML = "";
  hits.innerHTML = "";

  const slots = slotList(view).sort((a, b) => a.slot - b.slot);

  slots.forEach((s, index) => {
    const path = document.createElementNS(NS, "path");
    path.setAttribute("d", petalPath(s.slot));
    const kindClass =
      s.kind === "ghost" ? "ghost" : s.kind === "all" ? "all" : s.kind === "mode" ? "mode" : "row";
    path.setAttribute("class", `petal ${kindClass}${animate ? " in" : ""}`);
    path.dataset.slot = String(s.slot);
    if (animate) {
      path.style.animationDelay = `${Math.min(index, 7) * 18}ms`;
    }
    ring.appendChild(path);

    if (s.kind === "ghost") return;

    const label = document.createElement("div");
    label.className = `label ${kindClass}`;
    label.dataset.slot = String(s.slot);
    const [lx, ly] = pt(R_LABEL, slotAngle(s.slot));
    label.style.left = `${lx}px`;
    label.style.top = `${ly}px`;

    const main = document.createElement("span");
    main.className = "label-main";
    const sub = document.createElement("span");
    sub.className = "label-sub";

    let aria = "";
    let infoMain = "";
    let infoSub = "";

    if (s.kind === "row") {
      main.textContent = String(s.row.pid);
      sub.textContent = s.row.name;
      aria = `Kill ${s.row.name} (PID ${s.row.pid})`;
      infoMain = s.row.name;
      infoSub = `PID ${s.row.pid} · ${mb(s.row.ram_bytes)} MB`;
    } else if (s.kind === "all") {
      main.textContent = state.cfg.all_label;
      sub.textContent = state.cfg.all_sub;
      aria = `${state.cfg.all_label} (${state.rows.length})`;
      infoMain = state.cfg.all_label;
      infoSub = state.cfg.all_sub;
    } else {
      const meta = modeMeta(s.key);
      main.textContent = meta.main;
      sub.textContent = meta.sub;
      aria = meta.main;
      infoMain = meta.main;
      infoSub = meta.sub;
    }

    label.append(main, sub);
    labels.appendChild(label);
    state.slotInfo.set(s.slot, { main: infoMain, sub: infoSub });

    const hit = document.createElement("button");
    hit.type = "button";
    hit.className = "hit";
    hit.dataset.slot = String(s.slot);
    const [hx, hy] = pt(R_HIT, slotAngle(s.slot));
    hit.style.left = `${hx}px`;
    hit.style.top = `${hy}px`;
    hit.style.transform = `translate(-50%, -50%) rotate(${slotAngle(s.slot) + 90}deg)`;
    hit.setAttribute("aria-label", aria);
    hit.addEventListener("click", () => activate(s));
    hit.addEventListener("mouseenter", () => setHover(s.slot, true));
    hit.addEventListener("mouseleave", () => setHover(s.slot, false));
    hit.addEventListener("focus", () => setFocus(s.slot, true));
    hit.addEventListener("blur", () => setFocus(s.slot, false));
    hits.appendChild(hit);
  });

  updateContext();
}

function pathAt(slot) {
  return ring.querySelector(`path[data-slot="${slot}"]`);
}

function labelAt(slot) {
  return labels.querySelector(`.label[data-slot="${slot}"]`);
}

function setHover(slot, on) {
  const path = pathAt(slot);
  const label = labelAt(slot);
  if (path) path.dataset.lit = on ? "1" : "0";
  if (label) label.dataset.lit = on ? "1" : "0";
  if (on) {
    state.hover = slot;
  } else if (state.hover === slot) {
    state.hover = null;
  }
  updateContext();
}

function setFocus(slot, on) {
  const path = pathAt(slot);
  const label = labelAt(slot);
  if (path) path.dataset.focus = on ? "1" : "0";
  if (label) label.dataset.focus = on ? "1" : "0";
  if (on) {
    state.hover = slot;
  } else if (state.hover === slot) {
    state.hover = null;
  }
  updateContext();
}

function updateContext() {
  const c = state.cfg;
  contextEl.dataset.tone = "";
  let main = "";
  let sub = "";

  if (state.busy) {
    main = c.killing;
  } else if (state.error) {
    main = state.error;
    sub = c.hint_keys;
    contextEl.dataset.tone = "err";
  } else if (state.hover != null && state.slotInfo.has(state.hover)) {
    const info = state.slotInfo.get(state.hover);
    main = info.main;
    sub = info.sub;
  } else if (state.view === "results") {
    const extra = state.rows.length > MAX_ROWS ? `+${state.rows.length - MAX_ROWS} · ` : "";
    main = state.summary;
    sub = extra + c.hint_keys;
  } else if (state.view === "empty") {
    main = state.summary;
    sub = c.hint_idle;
  } else {
    main = c.hint_idle;
  }

  ctxMain.textContent = main;
  ctxSub.textContent = sub;
}

function activate(s) {
  if (state.busy) return;
  if (s.kind === "row") {
    kill([s.row.pid]);
  } else if (s.kind === "all") {
    killAll();
  } else if (s.kind === "mode") {
    setMode(s.key);
  }
}

function setMode(key) {
  state.error = "";
  if (key === "port") input.value = ":";
  else if (key === "proc") input.value = "";
  else input.value = "hang";
  input.focus();
  handleInput();
}

function scheduleQuery() {
  clearTimeout(qtimer);
  qtimer = setTimeout(runQuery, 140);
}

async function runQuery() {
  if (state.busy) return;
  const gen = state.queryGen;
  const value = input.value;
  try {
    const out = await invoke("query", { input: value });
    if (gen !== state.queryGen || state.busy) return;
    if (!out) return;
    state.rows = out.rows || [];
    state.mode = out.mode || null;
    state.summary = out.summary || "";
    state.error = "";
    render();
  } catch (e) {
    if (gen !== state.queryGen) return;
    state.rows = [];
    state.mode = null;
    state.error = state.cfg.query_failed;
    render();
  }
}

function handleInput() {
  state.queryGen++;
  state.error = "";
  const value = input.value.trim();
  if (!value) {
    clearTimeout(qtimer);
    state.rows = [];
    state.mode = null;
    state.summary = "";
    render();
    return;
  }
  scheduleQuery();
}

function setBusy(on) {
  state.busy = on;
  app.classList.toggle("busy", on);
  app.setAttribute("aria-busy", on ? "true" : "false");
  input.readOnly = on;
  if (on) state.error = "";
  updateContext();
}

async function kill(pids) {
  if (state.busy || !pids.length) return;
  setBusy(true);
  try {
    const report = await invoke("kill", { input: input.value, pids });
    if (report && Array.isArray(report.killed) && report.killed.length) {
      input.value = "";
      state.rows = [];
      state.mode = null;
      state.summary = "";
      state.error = "";
    } else {
      state.error = state.cfg.kill_failed;
    }
  } catch (e) {
    state.error = state.cfg.kill_failed;
  } finally {
    setBusy(false);
    render();
  }
}

function killAll() {
  kill(state.rows.map((r) => r.pid));
}

function applyCfg() {
  const c = state.cfg;
  coreLabel.textContent = c.core_label;
  input.placeholder = c.hint_input;
  input.setAttribute("aria-label", c.hint_input);
  document.documentElement.lang = c.lang || "vi";
  render();
}

input.addEventListener("input", handleInput);

hits.addEventListener("keydown", (e) => {
  if (!["ArrowRight", "ArrowLeft", "ArrowDown", "ArrowUp"].includes(e.key)) return;
  const buttons = [...hits.querySelectorAll("button")];
  if (!buttons.length) return;
  const index = buttons.indexOf(document.activeElement);
  if (index < 0) return;
  e.preventDefault();
  const step = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : -1;
  buttons[(index + step + buttons.length) % buttons.length].focus();
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    e.preventDefault();
    invoke("hide_overlay");
    return;
  }
  if (e.key !== "Enter") return;
  if (state.busy) return;
  const active = document.activeElement;
  if (active && active.closest && active.closest(".hit")) return;
  e.preventDefault();
  if (state.rows.length) killAll();
});

listen("overlay-open", () => {
  state.busy = false;
  state.error = "";
  state.hover = null;
  state.rows = [];
  state.mode = null;
  state.summary = "";
  state.queryGen++;
  input.value = "";
  app.classList.remove("busy");
  app.setAttribute("aria-busy", "false");
  input.readOnly = false;
  render();
  input.focus();
});

listen("lang-changed", (e) => {
  state.cfg = { ...DEFAULTS, ...(e.payload || {}) };
  applyCfg();
  if (input.value.trim()) runQuery();
});

// Browser preview (no Tauri backend): `?lang=en` renders the mock in English,
// mirroring src/i18n.rs — used for README screenshots.
const PREVIEW_EN = new URLSearchParams(location.search).get("lang") === "en";
const PREVIEW_CFG = {
  lang: "en",
  core_label: "TARGET",
  hint_idle: "type a port or app name",
  hint_keys: "Enter kill · Esc close",
  killing: "Killing…",
  all_label: "ALL",
  all_sub: "kill all",
  query_failed: "Query failed",
  kill_failed: "Couldn't kill",
  mode_port: "PORT",
  mode_proc: "PROCESS",
  mode_proc_sub: "node …",
  mode_hung: "HUNG",
  mode_hung_sub: "not responding",
};

(async () => {
  try {
    const cfg = await invoke("ui_config");
    if (cfg) state.cfg = { ...DEFAULTS, ...cfg };
  } catch (e) {
    state.error = state.cfg.query_failed;
  }
  applyCfg();
  input.focus();
  const preview = decodeURIComponent(location.hash.slice(1));
  if (preview) {
    input.value = preview;
    handleInput();
  }
})();

function mockInvoke(cmd, args) {
  if (cmd === "ui_config") {
    return Promise.resolve(PREVIEW_EN ? { ...PREVIEW_CFG } : { ...DEFAULTS });
  }
  if (cmd === "hide_overlay") return Promise.resolve();
  if (cmd === "kill") {
    const pids = args?.pids || [];
    return Promise.resolve({
      killed: pids.map((pid) => [pid, "node"]),
      denied: [],
      failed: [],
    });
  }
  if (cmd === "query") {
    const raw = String(args?.input || "").trim();
    const portMatch = raw.match(/^:?(\d{1,5})$/);
    if (portMatch) {
      const port = Number(portMatch[1]);
      if (port === 5555) {
        return Promise.resolve({
          mode: "port",
          summary: PREVIEW_EN ? "0 on port 5555" : "0 tiến trình · port 5555",
          rows: [],
        });
      }
      const count = port === 8080 ? 7 : port === 3000 ? 3 : 1;
      const names = ["node", "esbuild", "chrome", "Code", "explorer", "nginx", "python"];
      const rows = Array.from({ length: count }, (_, i) => ({
        pid: 20000 + i * 137,
        name: names[i % names.length],
        ram_bytes: (80 + i * 53) * 1024 * 1024,
      }));
      return Promise.resolve({
        mode: "port",
        summary: PREVIEW_EN ? `${count} on port ${port}` : `${count} tiến trình · port ${port}`,
        rows,
      });
    }
    if (raw.toLowerCase() === "hang") {
      return Promise.resolve({
        mode: "hang",
        summary: PREVIEW_EN ? "2 app(s) not responding" : "2 app đang treo",
        rows: [
          { pid: 4412, name: "explorer", ram_bytes: 210 * 1024 * 1024 },
          { pid: 9901, name: "Teams", ram_bytes: 380 * 1024 * 1024 },
        ],
      });
    }
    if (raw) {
      return Promise.resolve({
        mode: "name",
        summary: PREVIEW_EN ? "0 processes · 0 MB" : "0 tiến trình · 0 MB",
        rows: [],
      });
    }
    return Promise.resolve({ mode: null, summary: null, rows: [] });
  }
  return Promise.resolve(null);
}
