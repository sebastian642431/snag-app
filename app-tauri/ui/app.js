const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

const win = getCurrentWindow();
const urlEl = document.getElementById("url");
const hintEl = document.getElementById("url-hint");
const goEl = document.getElementById("go");
const listEl = document.getElementById("list");
const emptyEl = document.getElementById("empty");
const outEl = document.getElementById("out");
const fieldEl = document.getElementById("field");
const announceEl = document.getElementById("announce");

const cards = new Map();
let mode = "mp3";

document.getElementById("min").onclick = () => win.minimize();
document.getElementById("max").onclick = () => win.toggleMaximize();
document.getElementById("close").onclick = () => win.close();

// Screen readers only hear progress when it matters: when a job or an
// install ends. Progress lines change many times a second and would drown
// everything else out.
function announce(text) {
  announceEl.textContent = "";
  announceEl.textContent = text;
}

const modeButtons = [...document.querySelectorAll(".segb")];

function setMode(next) {
  mode = next;
  for (const button of modeButtons) {
    const on = button.dataset.mode === next;
    button.classList.toggle("on", on);
    button.setAttribute("aria-checked", on ? "true" : "false");
    // Only the chosen option sits in the tab order; the arrow keys move
    // between them, as they do in a group of radio buttons.
    button.tabIndex = on ? 0 : -1;
  }
}

for (const button of modeButtons) {
  button.onclick = () => setMode(button.dataset.mode);
  button.addEventListener("keydown", e => {
    if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)) return;
    e.preventDefault();
    const step = e.key === "ArrowLeft" || e.key === "ArrowUp" ? -1 : 1;
    const index = modeButtons.indexOf(button);
    const next = modeButtons[(index + step + modeButtons.length) % modeButtons.length];
    setMode(next.dataset.mode);
    next.focus();
  });
}

function card(id) {
  if (cards.has(id)) return cards.get(id);

  const el = document.createElement("div");
  el.className = "job running";

  const top = document.createElement("div");
  top.className = "jtop";
  const title = document.createElement("div");
  title.className = "jtitle";
  const pill = document.createElement("span");
  pill.className = "pill running";
  pill.textContent = "0%";
  top.append(title, pill);

  const track = document.createElement("div");
  track.className = "track";
  track.setAttribute("role", "progressbar");
  track.setAttribute("aria-valuemin", "0");
  track.setAttribute("aria-valuemax", "100");
  track.setAttribute("aria-valuenow", "0");
  const bar = document.createElement("div");
  bar.className = "bar";
  track.append(bar);

  const line = document.createElement("div");
  line.className = "jline";

  el.append(top, track, line);
  listEl.prepend(el);

  const refs = { el, title, pill, track, bar, line, state: "running" };
  cards.set(id, refs);
  // After the card is counted, or the empty state lingers until the next probe.
  paintEmpty();
  return refs;
}

listen("job", event => {
  const job = event.payload;
  const refs = card(job.id);
  const pct = Math.round(job.pct * 100);

  refs.title.textContent = job.title;
  refs.title.title = job.title;
  refs.line.textContent = job.line;
  // The line is clipped to one row while running; the full text stays
  // reachable on hover, which matters most for a long error.
  refs.line.title = job.line;
  refs.bar.style.width = pct + "%";
  refs.track.setAttribute("aria-label", job.title);

  refs.el.classList.toggle("running", job.state === "running");
  refs.el.classList.toggle("failed", job.state === "failed");
  refs.pill.className = "pill " + job.state;
  refs.bar.className = "bar" + (job.state === "running" ? "" : " " + job.state);

  if (job.state === "running") {
    refs.pill.textContent = pct + "%";
    refs.track.setAttribute("aria-valuenow", pct);
  } else {
    refs.pill.textContent = job.state === "done" ? "Done" : "Failed";
    refs.bar.style.width = "100%";
    refs.track.setAttribute("aria-valuenow", "100");
    if (refs.state === "running") {
      announce((job.state === "done" ? "Done: " : "Failed: ") + job.title + ". " + job.line);
    }
  }
  refs.state = job.state;
});

function showHint(text) {
  hintEl.textContent = text;
  fieldEl.classList.toggle("invalid", !!text);
  if (text) {
    urlEl.setAttribute("aria-invalid", "true");
  } else {
    urlEl.removeAttribute("aria-invalid");
  }
}

async function go() {
  const url = urlEl.value.trim();
  if (!url.startsWith("http")) {
    showHint(url ? "That does not look like a link. It should start with http." : "Paste a link first.");
    urlEl.focus();
    return;
  }
  urlEl.value = "";
  showHint("");
  try {
    await invoke("start", { url, mode });
  } catch (err) {
    urlEl.value = url;
    showHint("Could not start the download: " + err);
  }
}

goEl.onclick = go;
urlEl.addEventListener("keydown", e => {
  if (e.key === "Enter") go();
  if (e.key === "Escape") {
    urlEl.value = "";
    showHint("");
  }
});
urlEl.addEventListener("input", () => showHint(""));

document.getElementById("openf").onclick = () => invoke("open_folder");

const pickEl = document.getElementById("pickf");
pickEl.onclick = async () => {
  // The dialog is modal to the window but not to this handler: a second
  // click while it is open would stack a second dialog behind the first.
  pickEl.disabled = true;
  try {
    const picked = await invoke("pick_folder");
    if (picked) outEl.textContent = picked;
  } catch (err) {
    console.error(err);
  } finally {
    pickEl.disabled = false;
  }
};

function indicator(id, ok, text) {
  const el = document.getElementById(id);
  el.classList.toggle("on", ok);
  el.classList.toggle("off", !ok);
  el.querySelector("em").textContent = text;
}

const depsEl = document.getElementById("deps");
const toolsToggle = document.getElementById("tools-toggle");
let showTools = false;

toolsToggle.onclick = () => {
  showTools = !showTools;
  toolsToggle.classList.toggle("open", showTools);
  toolsToggle.setAttribute("aria-expanded", showTools ? "true" : "false");
  depsEl.hidden = !showTools;
  paintEmpty();
};
const depsRows = document.getElementById("deps-rows");
const depsNote = document.getElementById("deps-note");
const depsTitle = document.getElementById("deps-title");
const depsLead = document.getElementById("deps-lead");
const depsAll = document.getElementById("deps-all");

const DEPS = [
  { key: "ytdlp", name: "yt-dlp", what: "does the downloading" },
  { key: "ffmpeg", name: "ffmpeg", what: "extracts MP3, merges video" }
];

const rows = new Map();
const busy = new Set();
let hasWinget = true;
let toolsMissing = false;

function paintEmpty() {
  emptyEl.classList.toggle("hide", cards.size > 0 || toolsMissing);
}

depsAll.onclick = () => invoke("update_all");

function depRow(dep) {
  if (rows.has(dep.key)) return rows.get(dep.key);

  const el = document.createElement("div");
  el.className = "dep";

  const meta = document.createElement("div");
  meta.className = "dep-meta";
  const name = document.createElement("div");
  name.className = "dep-name";
  name.textContent = dep.name;
  const what = document.createElement("div");
  what.className = "dep-what";
  what.textContent = dep.what;
  meta.append(name, what);

  const action = document.createElement("div");
  action.className = "dep-action";
  const status = document.createElement("span");
  status.className = "dep-status";
  const button = document.createElement("button");
  button.className = "tool-btn";
  button.textContent = "Install";
  button.setAttribute("aria-label", "Install " + dep.name);
  action.append(status, button);

  const head = document.createElement("div");
  head.className = "dep-head";
  head.append(meta, action);

  const track = document.createElement("div");
  track.className = "dep-track";
  track.hidden = true;
  track.setAttribute("role", "progressbar");
  track.setAttribute("aria-label", "Installing " + dep.name);
  track.setAttribute("aria-valuemin", "0");
  track.setAttribute("aria-valuemax", "100");
  const bar = document.createElement("div");
  bar.className = "dep-bar";
  track.append(bar);

  el.append(head, track);

  const refs = { el, button, status, track, bar, what, name: dep.name, action: "install", state: "idle" };
  button.onclick = () => invoke("install_dep", { dep: dep.key, action: refs.action });
  rows.set(dep.key, refs);
  return refs;
}

function setButtonLabel(refs, text) {
  refs.button.textContent = text;
  refs.button.setAttribute("aria-label", text + " " + refs.name);
}

listen("install", event => {
  const update = event.payload;
  const refs = rows.get(update.dep);
  if (!refs) return;

  refs.state = update.state;

  if (update.state === "running") {
    busy.add(update.dep);
    depsAll.disabled = true;
    refs.button.hidden = true;
    refs.track.hidden = false;
    if (update.stage === "downloading") {
      const pct = Math.round(update.pct * 100);
      refs.status.textContent = "Downloading " + pct + "%";
      refs.bar.classList.remove("idle");
      refs.bar.style.width = pct + "%";
      refs.track.setAttribute("aria-valuenow", pct);
    } else {
      refs.status.textContent = update.stage === "installing" ? "Installing…" : "Starting…";
      refs.bar.classList.add("idle");
      refs.bar.style.width = "100%";
      refs.track.removeAttribute("aria-valuenow");
    }
    refs.status.className = "dep-status";
    refs.status.title = "";
    return;
  }

  busy.delete(update.dep);
  depsAll.disabled = !hasWinget || busy.size > 0;
  refs.track.hidden = true;
  refs.button.hidden = false;

  if (update.state === "done") {
    refs.status.textContent = update.message || "";
    refs.status.className = "dep-status good";
    refs.status.title = "";
    announce(refs.name + ": " + (update.message || "installed"));
    refresh();
  } else {
    setButtonLabel(refs, "Retry");
    refs.status.textContent = update.message || "Failed";
    refs.status.title = update.message || "";
    refs.status.className = "dep-status bad";
    announce(refs.name + " failed: " + (update.message || "winget could not install it"));
  }
});

function renderDeps(info) {
  const missing = !info.ytdlp || !info.ffmpeg;
  toolsMissing = missing;

  depsEl.hidden = !missing && !showTools;
  toolsToggle.hidden = missing;
  toolsToggle.classList.toggle("open", showTools);
  toolsToggle.setAttribute("aria-expanded", showTools ? "true" : "false");
  depsEl.classList.toggle("ok", !missing);
  depsTitle.textContent = missing ? "Missing tools" : "Tools";
  depsLead.hidden = !missing;
  depsAll.hidden = missing;
  depsAll.disabled = !hasWinget || busy.size > 0;

  for (const dep of DEPS) {
    const refs = depRow(dep);
    if (refs.el.parentNode !== depsRows) depsRows.append(refs.el);

    const installed = dep.key === "ytdlp" ? !!info.ytdlp : !!info.ffmpeg;
    const version = dep.key === "ytdlp" ? info.ytdlp_version : info.ffmpeg_version;

    refs.what.textContent = installed ? version || "installed" : dep.what;
    refs.what.classList.toggle("ver", installed && !!version);
    refs.action = installed ? "upgrade" : "install";
    if (refs.state !== "failed" && refs.state !== "running") {
      setButtonLabel(refs, installed ? "Update" : "Install");
    }
    refs.button.disabled = !hasWinget;
    refs.button.classList.toggle("primary", !installed);
  }

  depsNote.hidden = !missing && hasWinget;
  depsNote.textContent = hasWinget
    ? "Installed through winget. No admin rights needed."
    : "winget is not available here. Install them manually.";
  depsNote.classList.toggle("bad", !hasWinget);

  paintEmpty();
}

async function refresh() {
  let info;
  try {
    info = await invoke("probe");
  } catch (err) {
    // The next tick will try again; there is nothing to show for one miss.
    console.error(err);
    return;
  }
  hasWinget = info.winget;

  if (info.version) {
    document.getElementById("brand").textContent = "Snag · Tauri v" + info.version;
    document.getElementById("self-version").textContent = "v" + info.version;
  }
  outEl.textContent = info.out;
  outEl.title = info.out;

  const mark = (name, ok, version) => {
    if (!ok) return name + " missing";
    return version ? name + " " + version : name + " ready";
  };

  indicator("ind-yt", !!info.ytdlp, mark("yt-dlp", !!info.ytdlp, info.ytdlp_version));
  indicator("ind-ff", info.ffmpeg, mark("ffmpeg", !!info.ffmpeg, info.ffmpeg_version));

  const usable = !!info.ytdlp;
  goEl.disabled = !usable;
  urlEl.disabled = !usable;
  fieldEl.classList.toggle("off", !usable);
  urlEl.placeholder = usable ? "https://..." : "Install yt-dlp first";

  renderDeps(info);
}

const updEl = document.getElementById("upd");
const updTitle = document.getElementById("upd-title");
const updLead = document.getElementById("upd-lead");

let pendingUpdate = null;

const openRelease = () => {
  if (pendingUpdate) invoke("open_url", { url: pendingUpdate.page });
};
document.getElementById("upd-get").onclick = openRelease;
document.getElementById("self-get").onclick = openRelease;

// "Later" only hides the banner for this run. The Tools panel keeps showing
// the release, so it can never be lost for good.
document.getElementById("upd-later").onclick = () => {
  updEl.hidden = true;
  paintEmpty();
};

async function lookForUpdate() {
  let info;
  try {
    info = await invoke("check_update");
  } catch (err) {
    console.error(err);
    return;
  }
  if (!info) return;
  pendingUpdate = info;
  updTitle.textContent = "Snag " + info.tag + " is available";
  updLead.textContent = "You are on " + info.current;
  updEl.hidden = false;
  document.getElementById("self-status").textContent = info.tag + " available";
  document.getElementById("self-get").hidden = false;
  paintEmpty();
}

(async () => {
  await refresh();
  if (!urlEl.disabled) urlEl.focus();
  setInterval(refresh, 2500);
  lookForUpdate();
})();
