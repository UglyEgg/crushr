import initWasm, { init, inspect_archive, corrupt_archive } from "../pkg/crushr_wasm_corrupt_demo.js";

const MAX_ARCHIVE_BYTES = 256 * 1024 * 1024;

const dropZone = document.getElementById("dropZone");
const fileInput = document.getElementById("fileInput");
const fileSummary = document.getElementById("fileSummary");
const modeSelect = document.getElementById("modeSelect");
const applyBtn = document.getElementById("applyBtn");
const statusEl = document.getElementById("status");
const errorEl = document.getElementById("error");
const impactSummaryEl = document.getElementById("impactSummary");
const downloadLink = document.getElementById("downloadLink");

const randomControls = document.getElementById("randomControls");
const overwriteControls = document.getElementById("overwriteControls");
const truncateControls = document.getElementById("truncateControls");
const removeControls = document.getElementById("removeControls");

let sourceName = "";
let sourceBytes = null;
let sourceInspection = null;
let downloadUrl = null;

function setStatus(value) {
  statusEl.textContent = `Status: ${value}`;
}

function showError(message) {
  errorEl.hidden = false;
  errorEl.textContent = message;
}

function clearError() {
  errorEl.hidden = true;
  errorEl.textContent = "";
}

function clearDownload() {
  if (downloadUrl) {
    URL.revokeObjectURL(downloadUrl);
    downloadUrl = null;
  }
  downloadLink.hidden = true;
  downloadLink.removeAttribute("href");
  downloadLink.removeAttribute("download");
}

function parseIntSafe(value, fallback = 0) {
  const parsed = Number.parseInt(String(value), 10);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function updateModeControls() {
  const mode = modeSelect.value;
  randomControls.hidden = mode !== "random_flip";
  overwriteControls.hidden = mode !== "overwrite";
  truncateControls.hidden = mode !== "truncate";
  removeControls.hidden = mode !== "remove";
}

function buildConfig() {
  const mode = modeSelect.value;
  const config = { mode };

  if (mode === "random_flip") {
    config.seed = parseIntSafe(document.getElementById("seedInput").value, 12345);
    config.flip_count = parseIntSafe(document.getElementById("flipCountInput").value, 64);
  } else if (mode === "overwrite") {
    config.overwrite_offset = parseIntSafe(document.getElementById("overwriteOffsetInput").value, 0);
    config.overwrite_len = parseIntSafe(document.getElementById("overwriteLenInput").value, 1);
    config.overwrite_value = parseIntSafe(document.getElementById("overwriteValueInput").value, 0);
  } else if (mode === "truncate") {
    config.truncate_offset = parseIntSafe(document.getElementById("truncateOffsetInput").value, 0);
  } else if (mode === "remove") {
    config.remove_offset = parseIntSafe(document.getElementById("removeOffsetInput").value, 0);
    config.remove_len = parseIntSafe(document.getElementById("removeLenInput").value, 1);
  }

  return config;
}

function modeLabel(mode) {
  if (mode === "random_flip") return "randflip";
  if (mode === "overwrite") return "overwrite";
  if (mode === "truncate") return "truncate";
  return "remove";
}

function toDisplay(summary) {
  return [
    `bytes=${summary.total_bytes}`,
    `entries=${summary.total_entries}`,
    `blocks=${summary.total_blocks}`,
    `extents_valid=${summary.extents_valid}`,
    `strict_extraction_supported=${summary.strict_extraction_supported}`,
    `blake3=${summary.archive_hash_blake3}`,
  ].join("\n");
}

function buildDownloadName(originalName, mode, config, seedApplied) {
  const base = originalName.toLowerCase().endsWith(".crs")
    ? originalName.slice(0, -4)
    : originalName;
  const modePart = modeLabel(mode);
  const seedPart = seedApplied == null ? "seedna" : `seed${seedApplied}`;

  const details = [];
  if (mode === "random_flip") details.push(`count${config.flip_count}`);
  if (mode === "overwrite") details.push(`off${config.overwrite_offset}`, `len${config.overwrite_len}`);
  if (mode === "truncate") details.push(`off${config.truncate_offset}`);
  if (mode === "remove") details.push(`off${config.remove_offset}`, `len${config.remove_len}`);

  return `${base}.corrupted-${modePart}-${seedPart}-${details.join("-")}.crs`;
}

function toByteArray(value) {
  if (value instanceof Uint8Array) return value;
  if (ArrayBuffer.isView(value)) return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  if (Array.isArray(value)) return Uint8Array.from(value);
  throw new Error("WASM corruption output is not bytes.");
}

async function loadArchive(file) {
  if (!file) {
    return;
  }
  clearError();
  clearDownload();
  applyBtn.disabled = true;
  setStatus("loading archive...");

  if (!file.name.toLowerCase().endsWith(".crs")) {
    showError("Only .crs files are supported in this corruption demo.");
    setStatus("idle");
    return;
  }
  if (file.size > MAX_ARCHIVE_BYTES) {
    showError("Archive exceeds 256 MiB demo limit.");
    setStatus("idle");
    return;
  }

  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const summary = inspect_archive(bytes);

    sourceName = file.name;
    sourceBytes = bytes;
    sourceInspection = summary;
    fileSummary.textContent = `Loaded ${file.name} (${(file.size / (1024 * 1024)).toFixed(2)} MiB)`;
    impactSummaryEl.textContent = [
      "Before corruption",
      "-----------------",
      toDisplay(summary),
      "",
      "After corruption",
      "----------------",
      "Not generated yet.",
    ].join("\n");

    applyBtn.disabled = false;
    setStatus("ready");
  } catch (error) {
    showError(error instanceof Error ? error.message : String(error));
    setStatus("error");
  }
}

async function applyCorruption() {
  if (!sourceBytes) {
    showError("Load an archive first.");
    return;
  }

  clearError();
  clearDownload();
  applyBtn.disabled = true;
  setStatus("applying corruption...");

  const config = buildConfig();
  try {
    const result = corrupt_archive(sourceBytes, config);
    const corruptedBytes = toByteArray(result.archive_bytes);

    setStatus("inspecting corrupted archive...");
    let afterSummary = null;
    try {
      afterSummary = inspect_archive(corruptedBytes);
    } catch (inspectionError) {
      afterSummary = {
        total_bytes: corruptedBytes.length,
        total_entries: "n/a",
        total_blocks: "n/a",
        extents_valid: false,
        strict_extraction_supported: false,
        archive_hash_blake3: `inspection failed: ${inspectionError instanceof Error ? inspectionError.message : String(inspectionError)}`,
      };
    }

    impactSummaryEl.textContent = [
      "Before corruption",
      "-----------------",
      toDisplay(sourceInspection),
      "",
      "Operation",
      "---------",
      result.operation_summary,
      "",
      "After corruption",
      "----------------",
      toDisplay(afterSummary),
    ].join("\n");

    const blob = new Blob([corruptedBytes], { type: "application/octet-stream" });
    downloadUrl = URL.createObjectURL(blob);
    downloadLink.href = downloadUrl;
    downloadLink.download = buildDownloadName(sourceName, config.mode, config, result.seed_applied);
    downloadLink.hidden = false;

    setStatus("ready for download");
  } catch (error) {
    showError(error instanceof Error ? error.message : String(error));
    setStatus("error");
  } finally {
    applyBtn.disabled = false;
  }
}

fileInput.addEventListener("change", () => loadArchive(fileInput.files?.[0] ?? null));
modeSelect.addEventListener("change", updateModeControls);
applyBtn.addEventListener("click", applyCorruption);

["dragenter", "dragover"].forEach((eventName) => {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.add("dragging");
  });
});
["dragleave", "drop"].forEach((eventName) => {
  dropZone.addEventListener(eventName, (event) => {
    event.preventDefault();
    dropZone.classList.remove("dragging");
  });
});

dropZone.addEventListener("drop", (event) => {
  const file = event.dataTransfer?.files?.[0] ?? null;
  loadArchive(file);
});

await initWasm();
init();
updateModeControls();
setStatus("idle");
