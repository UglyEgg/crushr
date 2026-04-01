import init, { init as setup, archive_summary, find, entry, propagation } from "../pkg/crushr_wasm_readonly_demo.js";

const dropZoneEl = document.getElementById("drop-zone");
const fileEl = document.getElementById("file");
const summaryEl = document.getElementById("summary");
const queryEl = document.getElementById("query");
const resultsEl = document.getElementById("results");
const entryEl = document.getElementById("entry");
const extentEl = document.getElementById("extent-visualization");
const propagationToggleEl = document.getElementById("propagation-toggle");
const propagationSummaryEl = document.getElementById("propagation-summary");
const propagationDetailEl = document.getElementById("propagation-detail");
const errorEl = document.getElementById("error");
const searchBtn = document.getElementById("search");

let bytes = null;
let selectedPath = null;
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
    propagationSummaryEl.innerHTML = "";
    propagationDetailEl.innerHTML = "";
    propagationState = {
      enabled: propagationToggleEl.checked,
      impactedByPath: new Map(),
      noImpactMessage: "No impacted entries detected from current corruption inputs.",
    };
    renderEmptyExtentState("Select a search result to view extent placement.");
    refreshPropagationState();
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
  resultsEl.innerHTML = "";
  for (const match of matches) {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.textContent = `${match.path} (${match.trust_class})`;
    if (propagationState.enabled && propagationState.impactedByPath.has(match.path)) {
      button.classList.add("is-impacted");
      button.textContent = `${match.path} (${match.trust_class}) • impacted`;
    }
    if (selectedPath === match.path) {
      button.classList.add("is-selected");
    }
    button.addEventListener("click", () => {
      const detail = entry(bytes, match.path);
      selectedPath = match.path;
      entryEl.textContent = render(detail);
      renderExtentVisualization(detail);
      renderPropagationDetail(selectedPath);
      renderSearchResults(matches);
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
  if (!path) {
    propagationDetailEl.textContent = "Select a search result to view propagation details.";
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

function refreshPropagationState() {
  if (!bytes || !propagationState.enabled) {
    propagationState.impactedByPath = new Map();
    renderPropagationSummary();
    renderPropagationDetail(selectedPath);
    return;
  }
  try {
    const report = propagation(bytes);
    propagationState.noImpactMessage = report.no_impact_message;
    propagationState.impactedByPath = new Map(report.impacted_entries.map((item) => [item.path, item]));
    renderPropagationSummary();
    renderPropagationDetail(selectedPath);
  } catch (error) {
    setError(String(error));
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
    renderPropagationDetail(selectedPath);
    renderSearchResults(matches);
  } catch (error) {
    setError(String(error));
  }
});

propagationToggleEl.addEventListener("change", () => {
  propagationState.enabled = propagationToggleEl.checked;
  refreshPropagationState();
  if (bytes) {
    const matches = find(bytes, queryEl.value);
    renderSearchResults(matches);
    if (selectedPath) {
      const detail = entry(bytes, selectedPath);
      renderExtentVisualization(detail);
    }
  }
});
