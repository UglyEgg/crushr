const dropZoneEl = document.getElementById("drop-zone");
const fileEl = document.getElementById("file");
const browseBtn = document.getElementById("browse");
const unloadBtn = document.getElementById("unload");
const themeToggleBtn = document.getElementById("theme-toggle");
const headerStatusEl = document.getElementById("header-status");
const summaryEl = document.getElementById("summary");
const queryEl = document.getElementById("query");
const resultsEl = document.getElementById("results");
const entryEl = document.getElementById("entry");
const extentEl = document.getElementById("extent-visualization");
const previewEl = document.getElementById("entry-preview");
const propagationToggleEl = document.getElementById("propagation-toggle");
const propagationSummaryEl = document.getElementById("propagation-summary");
const propagationDetailEl = document.getElementById("propagation-detail");
const errorEl = document.getElementById("error");
const searchBtn = document.getElementById("search");
const searchBusyEl = document.getElementById("search-busy");
const progressOverlayEl = document.getElementById("progress-overlay");
const progressStageEl = document.getElementById("progress-stage");
const THEME_STORAGE_KEY = "crushr_demo_theme";

const NO_FILE_MESSAGE = "No archive loaded. Choose or drop a .crs file to begin.";
const NO_RESULTS_MESSAGE = "No matching entries found for the current query.";
const DEFAULT_EXTENT_MESSAGE = "Select an entry from the results pane to view extent placement.";
const DEFAULT_PREVIEW_MESSAGE = "Select an entry from the results pane to preview entry content.";
const SEARCH_PROMPT_MESSAGE = "Archive loaded. Enter a query and click Find to browse entries.";
const FIND_LIMIT_MESSAGE_PREFIX = "Showing first";

const worker = new Worker(new URL("./wasm-worker.js", import.meta.url), { type: "module" });
let nextRequestId = 1;
const pendingRequests = new Map();

let bytes = null;
let selectedPath = null;
let latestMatches = [];
let uiState = "idle";
let activeWorkerStatusStage = null;
let currentArchiveName = null;
let propagationState = {
  enabled: false,
  impactedByPath: new Map(),
  noImpactMessage: "Propagation view is disabled.",
};

worker.onmessage = (event) => {
  const message = event.data ?? {};
  if (message.type === "status") {
    activeWorkerStatusStage = message.stage ?? null;
    updateSearchBusyState();
    if (message.message) {
      setUiState("working", message.message);
    }
    return;
  }
  if (message.type !== "response") {
    return;
  }
  const request = pendingRequests.get(message.id);
  if (!request) {
    return;
  }
  pendingRequests.delete(message.id);
  activeWorkerStatusStage = null;
  updateSearchBusyState();
  if (message.ok) {
    request.resolve(message.result ?? {});
    return;
  }
  request.reject(new Error(message.error || "Worker request failed."));
};

function requestWorker(type, data = {}) {
  const id = nextRequestId++;
  const result = new Promise((resolve, reject) => {
    pendingRequests.set(id, { resolve, reject });
  });
  worker.postMessage({ id, type, data });
  return result;
}

function clearError() {
  errorEl.textContent = "";
}

function setHeaderStatus(message = "") {
  headerStatusEl.textContent = message;
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
  if (state === "working") {
    setHeaderStatus("");
  } else if (state === "error") {
    setHeaderStatus(message || "Error");
  } else if (bytes) {
    setHeaderStatus(`Loaded: ${currentArchiveName || "archive"} • Ready`);
  } else {
    setHeaderStatus("Idle");
  }
  if (state === "working") {
    progressOverlayEl.hidden = false;
    progressOverlayEl.classList.add("is-active");
    progressStageEl.textContent = message;
    return;
  }
  progressOverlayEl.classList.remove("is-active");
  progressOverlayEl.hidden = true;
  progressStageEl.textContent = "";
}

function setControlsBusy(isBusy) {
  fileEl.disabled = isBusy;
  browseBtn.disabled = isBusy;
  unloadBtn.disabled = isBusy || !bytes;
  searchBtn.disabled = isBusy;
  queryEl.disabled = isBusy || !bytes;
  propagationToggleEl.disabled = isBusy || !bytes;
  updateSearchBusyState();
}

function updateSearchBusyState() {
  const isSearchBusy = activeWorkerStatusStage === "search_busy";
  searchBtn.textContent = isSearchBusy ? "Searching..." : "Find";
  searchBtn.setAttribute("aria-busy", isSearchBusy ? "true" : "false");
  searchBusyEl.textContent = isSearchBusy ? "Searching…" : "";
  searchBusyEl.hidden = !isSearchBusy;
}

async function runWorking(actionLabel, work) {
  setUiState("working", actionLabel);
  setControlsBusy(true);
  try {
    return await work();
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

async function resetDemoState(options = {}) {
  const { clearFileInput = false, resetWorker = true } = options;

  bytes = null;
  currentArchiveName = null;
  if (resetWorker) {
    try {
      await requestWorker("reset");
    } catch (_error) {
      // no-op: UI reset should still complete
    }
  }
  selectedPath = null;
  latestMatches = [];
  queryEl.value = "";
  propagationToggleEl.checked = false;

  summaryEl.textContent = NO_FILE_MESSAGE;
  renderResultsMessage(NO_FILE_MESSAGE);
  entryEl.textContent = "No entry selected.";
  renderEmptyExtentState(DEFAULT_EXTENT_MESSAGE);
  renderEmptyPreviewState(DEFAULT_PREVIEW_MESSAGE);
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
  activeWorkerStatusStage = null;
  setUiState("idle", "");
  setControlsBusy(false);
}

function renderEmptyExtentState(message) {
  extentEl.innerHTML = "";
  const p = document.createElement("p");
  p.className = "muted";
  p.textContent = message;
  extentEl.appendChild(p);
}

function renderEmptyPreviewState(message) {
  previewEl.innerHTML = "";
  const p = document.createElement("p");
  p.className = "muted";
  p.textContent = message;
  previewEl.appendChild(p);
}

function renderExtentVisualization(detail, previewMeta = null) {
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
  const extentMetaBytes = Number(previewMeta?.extent_metadata_bytes_derived ?? 0);
  const extentDataBytes = Number(
    previewMeta?.extent_data_bytes ??
      segments.reduce((acc, segment) => acc + Number(segment.size_bytes || 0), 0),
  );
  const perExtentMetaBytes = segments.length > 0 ? extentMetaBytes / segments.length : 0;
  for (const segment of segments) {
    const block = document.createElement("div");
    block.className = "extent-block";
    if (propagationState.enabled) {
      block.classList.add(isImpacted ? "extent-block-impacted" : "extent-block-normal");
      block.classList.add("extent-block-selected");
    }
    const ratio = Number(segment.size_bytes || 1) / maxBytes;
    block.style.flexGrow = String(Math.max(1, Math.round(ratio * 8)));

    const dataBytes = Number(segment.size_bytes || 0);
    const totalBytes = dataBytes + perExtentMetaBytes;
    const dataPct = totalBytes > 0 ? (dataBytes / totalBytes) * 100 : 0;
    const metaPct = totalBytes > 0 ? (perExtentMetaBytes / totalBytes) * 100 : 0;

    const dataSegment = document.createElement("div");
    dataSegment.className = "extent-part extent-part-data";
    dataSegment.style.width = `${dataPct}%`;

    const metadataSegment = document.createElement("div");
    metadataSegment.className = "extent-part extent-part-meta";
    metadataSegment.style.width = `${metaPct}%`;

    const label = document.createElement("span");
    label.className = "extent-block-label";
    label.textContent = `E${segment.extent_index}`;

    block.appendChild(dataSegment);
    block.appendChild(metadataSegment);
    block.appendChild(label);
    strip.appendChild(block);
  }
  extentEl.appendChild(strip);

  const legend = document.createElement("ul");
  legend.className = "extent-legend";
  const stateLegend = propagationState.enabled
    ? `
      <li><span class="legend-swatch legend-selected"></span>selected entry</li>
      <li><span class="legend-swatch legend-impacted"></span>impacted entry</li>
      <li><span class="legend-swatch legend-normal"></span>normal entry</li>
    `
    : "";
  legend.innerHTML = `
    ${stateLegend}
    <li><span class="legend-swatch legend-data"></span>payload data bytes</li>
    <li><span class="legend-swatch legend-metadata"></span>metadata bytes (derived from IDX extent records)</li>
  `;
  extentEl.appendChild(legend);

  const segmentationNote = document.createElement("p");
  segmentationNote.className = "muted extent-note";
  segmentationNote.textContent = `Metadata segment is derived as ${extentMetaBytes} bytes total (28 bytes per extent from IDX record encoding), data segment is ${extentDataBytes} payload bytes.`;
  extentEl.appendChild(segmentationNote);

  const meta = document.createElement("ul");
  meta.className = "extent-meta";
  for (const segment of segments) {
    const row = document.createElement("li");
    row.textContent = `E${segment.extent_index}: logical ${segment.logical_start}-${segment.logical_end}, size ${segment.size_bytes} bytes, block ${segment.block_id}`;
    meta.appendChild(row);
  }
  extentEl.appendChild(meta);
}

function renderPreview(preview) {
  previewEl.innerHTML = "";
  if (!preview) {
    renderEmptyPreviewState("Selected entry preview is unavailable.");
    return;
  }

  const header = document.createElement("p");
  header.className = "extent-header";
  const truncationLabel = preview.truncated ? " • preview truncated at 5 KiB" : "";
  header.textContent = `${preview.path} • read ${preview.bytes_read}/${preview.cap_bytes} bytes${truncationLabel}`;
  previewEl.appendChild(header);

  if (preview.preview_kind === "text") {
    const textPre = document.createElement("pre");
    textPre.className = "preview-text";
    textPre.textContent = preview.text_preview ?? "";
    previewEl.appendChild(textPre);
  } else if (preview.preview_kind === "binary") {
    const message = document.createElement("p");
    message.className = "preview-binary";
    message.textContent = preview.binary_message ?? "unknown";
    previewEl.appendChild(message);
  } else {
    renderEmptyPreviewState(preview.note ?? "No preview available.");
    return;
  }

  if (preview.note) {
    const note = document.createElement("p");
    note.className = "muted extent-note";
    note.textContent = preview.note;
    previewEl.appendChild(note);
  }
}

function renderSearchResults(resultPayload) {
  const matches = resultPayload?.matches ?? [];
  latestMatches = matches;
  resultsEl.innerHTML = "";

  if (resultPayload?.truncated) {
    const notice = document.createElement("li");
    notice.className = "empty-state";
    const shownCount = matches.length;
    const totalCount = resultPayload.total_matches ?? shownCount;
    notice.textContent = `${FIND_LIMIT_MESSAGE_PREFIX} ${shownCount} of ${totalCount} matches. Refine your search to narrow results.`;
    resultsEl.appendChild(notice);
  }

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
      if (!bytes) {
        return;
      }
      try {
        await runWorking("Loading entry detail...", async () => {
          const { detail } = await requestWorker("entry", { path: match.path });
          const { preview } = await requestWorker("entryPreview", { path: match.path });
          selectedPath = match.path;
          entryEl.textContent = render(detail);
          renderExtentVisualization(detail, preview);
          renderPreview(preview);
          renderPropagationDetail(selectedPath);
          renderSearchResults({ matches: latestMatches, truncated: resultPayload?.truncated, total_matches: resultPayload?.total_matches });
          setUiState("success", "Ready");
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

  await runWorking("Analyzing propagation...", async () => {
    const { report } = await requestWorker("propagation");
    propagationState.noImpactMessage = report.no_impact_message;
    propagationState.impactedByPath = new Map(report.impacted_entries.map((item) => [item.path, item]));
    renderPropagationSummary();
    renderPropagationDetail(selectedPath);
    setUiState("success", "Ready");
  });
}

async function loadArchive(file) {
  clearError();
  if (!file) {
    await resetDemoState({ resetWorker: false });
    setError("No file provided.");
    return;
  }

  await resetDemoState();

  try {
    await runWorking("Loading archive...", async () => {
      const nextBytes = new Uint8Array(await file.arrayBuffer());
      const { summary } = await requestWorker("loadArchive", { fileName: file.name, bytes: nextBytes });
      bytes = nextBytes;
      currentArchiveName = file.name;
      summaryEl.textContent = render(summary);
      renderResultsMessage(SEARCH_PROMPT_MESSAGE);
      renderPropagationSummary();
      renderPropagationDetail(selectedPath);
      setUiState("success", "Ready");
    });
  } catch (_error) {
    await resetDemoState({ resetWorker: false });
  }
}

async function performSearch() {
  clearError();
  if (!bytes) {
    setError("Load an archive before searching.");
    return;
  }

  try {
    await runWorking("Searching...", async () => {
      const { matches } = await requestWorker("search", { query: queryEl.value });
      setUiState("working", "Rendering results...");
      selectedPath = null;
      entryEl.textContent = "No entry selected.";
      renderEmptyExtentState(DEFAULT_EXTENT_MESSAGE);
      renderEmptyPreviewState(DEFAULT_PREVIEW_MESSAGE);
      renderPropagationDetail(selectedPath);
      renderSearchResults(matches);
      setUiState("success", "Ready");
    });
  } catch (_error) {
    // setError already handled in runWorking
  }
}

await resetDemoState();

function setTheme(theme) {
  document.body.setAttribute("data-theme", theme);
  themeToggleBtn.textContent = `Theme: ${theme === "dark" ? "Dark" : "Light"}`;
  themeToggleBtn.setAttribute("aria-pressed", theme === "dark" ? "true" : "false");
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch (_error) {
    // ignore storage write failures
  }
}

function initTheme() {
  let preferredTheme = "light";
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark") {
      preferredTheme = stored;
    } else if (window.matchMedia?.("(prefers-color-scheme: dark)").matches) {
      preferredTheme = "dark";
    }
  } catch (_error) {
    if (window.matchMedia?.("(prefers-color-scheme: dark)").matches) {
      preferredTheme = "dark";
    }
  }
  setTheme(preferredTheme);
}

initTheme();

fileEl.addEventListener("change", async () => {
  const file = fileEl.files?.[0];
  await loadArchive(file);
});

browseBtn.addEventListener("click", () => {
  fileEl.click();
});

unloadBtn.addEventListener("click", async () => {
  await resetDemoState({ clearFileInput: true });
});

themeToggleBtn.addEventListener("click", () => {
  const nextTheme = document.body.getAttribute("data-theme") === "dark" ? "light" : "dark";
  setTheme(nextTheme);
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
    if (!bytes) {
      return;
    }
    const { matches } = await requestWorker("search", { query: queryEl.value });
    renderSearchResults(matches);
    if (selectedPath) {
      const { detail } = await requestWorker("entry", { path: selectedPath });
      const { preview } = await requestWorker("entryPreview", { path: selectedPath });
      renderExtentVisualization(detail, preview);
      renderPreview(preview);
    }
    setUiState("success", "Ready");
  } catch (_error) {
    // setError already handled
  }
});
