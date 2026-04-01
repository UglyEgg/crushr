const WASM_MODULE_PATHS = ["./pkg/crushr_wasm_readonly_demo.js", "../pkg/crushr_wasm_readonly_demo.js"];

let wasmFns = null;
let wasmInitError = null;

async function initializeWasmRuntime() {
  for (const path of WASM_MODULE_PATHS) {
    try {
      const module = await import(path);
      await module.default();
      module.init();
      wasmFns = {
        archive_summary: module.archive_summary,
        find: module.find,
        entry: module.entry,
        propagation: module.propagation,
      };
      return;
    } catch (error) {
      wasmInitError = error;
    }
  }
  throw wasmInitError ?? new Error("Unknown WASM initialization failure.");
}

function requireWasmReady() {
  if (wasmFns) {
    return true;
  }
  const detail = wasmInitError ? ` ${String(wasmInitError)}` : "";
  setError(`Failed to initialize wasm runtime.${detail}`);
  setUiState("error", "WASM runtime initialization failed.");
  return false;
}

const dropZoneEl = document.getElementById("drop-zone");
const fileEl = document.getElementById("file");
const unloadBtn = document.getElementById("unload");
const summaryEl = document.getElementById("summary");
const queryEl = document.getElementById("query");
const resultsEl = document.getElementById("results");
const entryEl = document.getElementById("entry");
const extentEl = document.getElementById("extent-visualization");
const propagationToggleEl = document.getElementById("propagation-toggle");
const propagationSummaryEl = document.getElementById("propagation-summary");
const propagationDetailEl = document.getElementById("propagation-detail");
const statusEl = document.getElementById("status");
const errorEl = document.getElementById("error");
const searchBtn = document.getElementById("search");

const NO_FILE_MESSAGE = "No archive loaded. Choose or drop a .crs file to begin.";
const NO_RESULTS_MESSAGE = "No matching entries found for the current query.";
const DEFAULT_EXTENT_MESSAGE = "Select an entry from the results pane to view extent placement.";
const SEARCH_PROMPT_MESSAGE = "Archive loaded. Enter a query and click Find to browse entries.";

let bytes = null;
let selectedPath = null;
let latestMatches = [];
let uiState = "idle";
let propagationState = {
  enabled: false,
  impactedByPath: new Map(),
  noImpactMessage: "Propagation view is disabled.",
};

function clearError() {
  errorEl.textContent = "";
}

function setError(message) {
  errorEl.textContent = message;
  setUiState("error", message);
}

function formatActionError(actionLabel, error) {
  const detail = String(error);
  const action = actionLabel.replace(/\.\.\.$/, "");
  if (detail.includes("RuntimeError: unreachable")) {
    return `${action} failed due to an internal WASM runtime error. Reload the archive and retry. Details: ${detail}`;
  }
  return `${action} failed: ${detail}`;
}

function setUiState(state, message) {
  uiState = state;
  statusEl.className = `status-banner status-${state}`;
  statusEl.textContent = message;
}

function setControlsBusy(isBusy) {
  fileEl.disabled = isBusy;
  unloadBtn.disabled = isBusy || !bytes;
  searchBtn.disabled = isBusy;
  queryEl.disabled = isBusy || !bytes;
  propagationToggleEl.disabled = isBusy || !bytes;
}

async function runWorking(actionLabel, work) {
  setUiState("working", actionLabel);
  setControlsBusy(true);
  try {
    const result = await work();
    setUiState("success", `${actionLabel.replace(/\.\.\.$/, "")}: done.`);
    return result;
  } catch (error) {
    setError(formatActionError(actionLabel, error));
    throw error;
  } finally {
    setControlsBusy(false);
  }
}

function render(obj) {
  return JSON.stringify(obj, null, 2);
}

function renderResultsMessage(message) {
  resultsEl.innerHTML = "";
  const item = document.createElement("li");
  item.className = "empty-state";
  item.textContent = message;
  resultsEl.appendChild(item);
}

function resetDemoState(options = {}) {
  const { clearFileInput = false, statusMessage = "Idle. Load an archive to start." } = options;

  bytes = null;
  selectedPath = null;
  latestMatches = [];
  queryEl.value = "";
  propagationToggleEl.checked = false;

  summaryEl.textContent = NO_FILE_MESSAGE;
  renderResultsMessage(NO_FILE_MESSAGE);
  entryEl.textContent = "No entry selected.";
  renderEmptyExtentState(DEFAULT_EXTENT_MESSAGE);
  propagationSummaryEl.textContent = "Propagation view is disabled.";
  propagationDetailEl.textContent = "Enable propagation view to inspect impact details.";
  propagationState = {
    enabled: false,
    impactedByPath: new Map(),
    noImpactMessage: "No impacted entries detected from current corruption inputs.",
  };

  if (clearFileInput) {
    fileEl.value = "";
  }

  clearError();
  setUiState("idle", statusMessage);
  setControlsBusy(false);
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
  header.className = "extent-header";
  header.textContent = `${detail.path} • ${detail.extent_count} extent(s) • logical range ${detail.logical_range.start}-${detail.logical_range.end}`;
  extentEl.appendChild(header);

  const impact = propagationState.impactedByPath.get(detail.path);
  const isImpacted = propagationState.enabled && Boolean(impact);

  const strip = document.createElement("div");
  strip.className = "extent-strip";
  const maxBytes = Math.max(...segments.map((segment) => Number(segment.size_bytes || 1)));
  for (const segment of segments) {
    const block = document.createElement("div");
    block.className = "extent-block";
    if (propagationState.enabled) {
      block.classList.add(isImpacted ? "extent-block-impacted" : "extent-block-normal");
      block.classList.add("extent-block-selected");
    }
    const ratio = Number(segment.size_bytes || 1) / maxBytes;
    block.style.flexGrow = String(Math.max(1, Math.round(ratio * 8)));
    block.textContent = `E${segment.extent_index}`;
    strip.appendChild(block);
  }
  extentEl.appendChild(strip);

  if (propagationState.enabled) {
    const legend = document.createElement("ul");
    legend.className = "extent-legend";
    legend.innerHTML = `
      <li><span class="legend-swatch legend-selected"></span>selected entry</li>
      <li><span class="legend-swatch legend-impacted"></span>impacted entry</li>
      <li><span class="legend-swatch legend-normal"></span>normal entry</li>
    `;
    extentEl.appendChild(legend);
  }

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
  latestMatches = matches;
  resultsEl.innerHTML = "";
  if (matches.length === 0) {
    renderResultsMessage(NO_RESULTS_MESSAGE);
    return;
  }

  for (const match of matches) {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.classList.add("result-button");
    button.textContent = `${match.path} (${match.trust_class})`;
    if (propagationState.enabled && propagationState.impactedByPath.has(match.path)) {
      button.classList.add("is-impacted");
      button.textContent = `${match.path} (${match.trust_class}) • impacted`;
    }
    if (selectedPath === match.path) {
      button.classList.add("is-selected");
    }

    button.addEventListener("click", async () => {
      if (!bytes || !requireWasmReady()) {
        return;
      }
      try {
        await runWorking("Loading entry detail...", async () => {
          const detail = wasmFns.entry(bytes, match.path);
          selectedPath = match.path;
          entryEl.textContent = render(detail);
          renderExtentVisualization(detail);
          renderPropagationDetail(selectedPath);
          renderSearchResults(latestMatches);
        });
      } catch (_error) {
        // setError already handled in runWorking
      }
    });

    item.appendChild(button);
    resultsEl.appendChild(item);
  }
}

function renderPropagationSummary() {
  propagationSummaryEl.innerHTML = "";
  if (!propagationState.enabled) {
    propagationSummaryEl.textContent = "Propagation view is disabled.";
    return;
  }
  const impactedCount = propagationState.impactedByPath.size;
  if (impactedCount === 0) {
    propagationSummaryEl.textContent = propagationState.noImpactMessage;
    return;
  }
  propagationSummaryEl.textContent = `${impactedCount} impacted entr${impactedCount === 1 ? "y" : "ies"} detected.`;
}

function renderPropagationDetail(path) {
  propagationDetailEl.innerHTML = "";
  if (!propagationState.enabled) {
    propagationDetailEl.textContent = "Enable propagation view to inspect impact details.";
    return;
  }
  if (!bytes) {
    propagationDetailEl.textContent = "Load an archive to inspect propagation details.";
    return;
  }
  if (!path) {
    propagationDetailEl.textContent = "Select an entry from the results pane to view propagation details.";
    return;
  }

  const impact = propagationState.impactedByPath.get(path);
  if (!impact) {
    propagationDetailEl.textContent = "Selected entry has no active propagation impact.";
    return;
  }

  const lines = [
    `Impact status: ${impact.impact_status}`,
    `Consequence: ${impact.consequence}`,
    `Canonical blocked: ${impact.canonical_blocked ? "yes" : "no"}`,
    `Trust-class support: ${impact.trust_class_support.join(", ")}`,
    `Impact reasons: ${impact.impact_reasons.join(", ")}`,
    `Relevant structures: ${impact.relevant_structures.join(", ")}`,
  ];
  propagationDetailEl.textContent = lines.join("\n");
}

async function refreshPropagationState() {
  if (!bytes || !propagationState.enabled) {
    propagationState.impactedByPath = new Map();
    renderPropagationSummary();
    renderPropagationDetail(selectedPath);
    return;
  }
  if (!requireWasmReady()) {
    return;
  }

  await runWorking("Analyzing propagation...", async () => {
    const report = wasmFns.propagation(bytes);
    propagationState.noImpactMessage = report.no_impact_message;
    propagationState.impactedByPath = new Map(report.impacted_entries.map((item) => [item.path, item]));
    renderPropagationSummary();
    renderPropagationDetail(selectedPath);
  });
}

async function loadArchive(file) {
  clearError();
  if (!file) {
    resetDemoState({ statusMessage: "No archive loaded." });
    setError("No file provided.");
    return;
  }

  resetDemoState({ statusMessage: "Preparing archive load..." });

  try {
    await runWorking("Loading archive...", async () => {
      const nextBytes = new Uint8Array(await file.arrayBuffer());
      if (!requireWasmReady()) {
        return;
      }

      const summary = wasmFns.archive_summary(file.name, nextBytes);
      bytes = nextBytes;
      summaryEl.textContent = render(summary);
      renderResultsMessage(SEARCH_PROMPT_MESSAGE);
      await refreshPropagationState();
    });
  } catch (_error) {
    resetDemoState({ statusMessage: "Load failed; archive state reset." });
  }
}

async function performSearch() {
  clearError();
  if (!bytes) {
    setError("Load an archive before searching.");
    return;
  }
  if (!requireWasmReady()) {
    return;
  }

  try {
    await runWorking("Searching entries...", async () => {
      const matches = wasmFns.find(bytes, queryEl.value);
      selectedPath = null;
      entryEl.textContent = "No entry selected.";
      renderEmptyExtentState(DEFAULT_EXTENT_MESSAGE);
      renderPropagationDetail(selectedPath);
      renderSearchResults(matches);
    });
  } catch (_error) {
    // setError already handled in runWorking
  }
}

resetDemoState();

try {
  await initializeWasmRuntime();
} catch (_error) {
  requireWasmReady();
}

fileEl.addEventListener("change", async () => {
  const file = fileEl.files?.[0];
  await loadArchive(file);
});

unloadBtn.addEventListener("click", () => {
  resetDemoState({ clearFileInput: true, statusMessage: "Archive unloaded. Demo reset to empty state." });
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

searchBtn.addEventListener("click", async () => {
  await performSearch();
});

propagationToggleEl.addEventListener("change", async () => {
  propagationState.enabled = propagationToggleEl.checked;
  try {
    await refreshPropagationState();
    if (!bytes || !requireWasmReady()) {
      return;
    }
    const matches = wasmFns.find(bytes, queryEl.value);
    renderSearchResults(matches);
    if (selectedPath) {
      const detail = wasmFns.entry(bytes, selectedPath);
      renderExtentVisualization(detail);
    }
  } catch (_error) {
    // setError already handled
  }
});
