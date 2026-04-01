import init, { init as setup, archive_summary, find, entry } from "../pkg/crushr_wasm_readonly_demo.js";

const dropZoneEl = document.getElementById("drop-zone");
const fileEl = document.getElementById("file");
const summaryEl = document.getElementById("summary");
const queryEl = document.getElementById("query");
const resultsEl = document.getElementById("results");
const entryEl = document.getElementById("entry");
const extentEl = document.getElementById("extent-visualization");
const errorEl = document.getElementById("error");
const searchBtn = document.getElementById("search");

let bytes = null;
let selectedPath = null;

function clearError() {
  errorEl.textContent = "";
}

function setError(message) {
  errorEl.textContent = message;
}

function render(obj) {
  return JSON.stringify(obj, null, 2);
}

async function loadArchive(file) {
  clearError();
  if (!file) {
    setError("No file provided.");
    return;
  }

  try {
    const nextBytes = new Uint8Array(await file.arrayBuffer());
    const summary = archive_summary(file.name, nextBytes);
    bytes = nextBytes;
    selectedPath = null;
    summaryEl.textContent = render(summary);
    resultsEl.innerHTML = "";
    entryEl.textContent = "";
    renderEmptyExtentState("Select a search result to view extent placement.");
  } catch (error) {
    setError(String(error));
  }
}

function renderEmptyExtentState(message) {
  extentEl.innerHTML = "";
  const p = document.createElement("p");
  p.className = "muted";
  p.textContent = message;
  extentEl.appendChild(p);
}

function renderExtentVisualization(detail) {
  extentEl.innerHTML = "";
  if (!detail) {
    renderEmptyExtentState("Selected entry details are unavailable.");
    return;
  }
  const segments = detail.extent_segments ?? [];
  if (segments.length === 0) {
    renderEmptyExtentState("Extent segmentation is unavailable for this entry.");
    return;
  }
  const header = document.createElement("p");
  header.textContent = `${detail.path} • ${detail.extent_count} extent(s) • logical range ${detail.logical_range.start}-${detail.logical_range.end}`;
  extentEl.appendChild(header);

  const strip = document.createElement("div");
  strip.className = "extent-strip";
  const maxBytes = Math.max(...segments.map((segment) => Number(segment.size_bytes || 1)));
  for (const segment of segments) {
    const block = document.createElement("div");
    block.className = "extent-block";
    const ratio = Number(segment.size_bytes || 1) / maxBytes;
    block.style.flexGrow = String(Math.max(1, Math.round(ratio * 8)));
    block.textContent = `E${segment.extent_index}`;
    strip.appendChild(block);
  }
  extentEl.appendChild(strip);

  const meta = document.createElement("ul");
  meta.className = "extent-meta";
  for (const segment of segments) {
    const row = document.createElement("li");
    row.textContent = `E${segment.extent_index}: logical ${segment.logical_start}-${segment.logical_end}, size ${segment.size_bytes} bytes, block ${segment.block_id}`;
    meta.appendChild(row);
  }
  extentEl.appendChild(meta);
}

function renderSearchResults(matches) {
  resultsEl.innerHTML = "";
  for (const match of matches) {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.textContent = `${match.path} (${match.trust_class})`;
    if (selectedPath === match.path) {
      button.classList.add("is-selected");
    }
    button.addEventListener("click", () => {
      const detail = entry(bytes, match.path);
      selectedPath = match.path;
      entryEl.textContent = render(detail);
      renderExtentVisualization(detail);
      renderSearchResults(matches);
    });
    item.appendChild(button);
    resultsEl.appendChild(item);
  }
}

await init();
setup();

fileEl.addEventListener("change", async () => {
  const file = fileEl.files?.[0];
  await loadArchive(file);
});

dropZoneEl.addEventListener("dragenter", (event) => {
  event.preventDefault();
  dropZoneEl.classList.add("drag-over");
});

dropZoneEl.addEventListener("dragover", (event) => {
  event.preventDefault();
  dropZoneEl.classList.add("drag-over");
});

dropZoneEl.addEventListener("dragleave", (event) => {
  if (!dropZoneEl.contains(event.relatedTarget)) {
    dropZoneEl.classList.remove("drag-over");
  }
});

dropZoneEl.addEventListener("drop", async (event) => {
  event.preventDefault();
  dropZoneEl.classList.remove("drag-over");
  const file = event.dataTransfer?.files?.[0];
  await loadArchive(file);
});

searchBtn.addEventListener("click", () => {
  clearError();
  if (!bytes) return;
  try {
    const matches = find(bytes, queryEl.value);
    selectedPath = null;
    entryEl.textContent = "";
    renderEmptyExtentState("Select a search result to view extent placement.");
    renderSearchResults(matches);
  } catch (error) {
    setError(String(error));
  }
});
