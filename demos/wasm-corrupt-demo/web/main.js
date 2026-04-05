import initWasm, { init, inspect_archive, corrupt_archive } from "../pkg/crushr_wasm_corrupt_demo.js";

const MAX_ARCHIVE_BYTES = 256 * 1024 * 1024;

const dropZone = document.getElementById("dropZone");
const fileInput = document.getElementById("fileInput");
const fileSummary = document.getElementById("fileSummary");
const modeSelect = document.getElementById("modeSelect");
const modeDescription = document.getElementById("modeDescription");
const applyBtn = document.getElementById("applyBtn");
const statusEl = document.getElementById("status");
const errorEl = document.getElementById("error");
const impactSummaryEl = document.getElementById("impactSummary");
const downloadLink = document.getElementById("downloadLink");

const presetDetails = document.getElementById("presetDetails");
const explanationSummary = document.getElementById("explanationSummary");
const explanationList = document.getElementById("explanationList");

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

function buildRanges(bytesLength) {
  const headerEnd = Math.min(bytesLength, 512);
  const tailStart = Math.max(0, bytesLength - 1024);
  const indexStart = Math.max(headerEnd, bytesLength - 768);
  const indexEnd = Math.max(indexStart + 1, bytesLength - 192);
  const payloadStart = Math.min(bytesLength - 1, Math.max(headerEnd, 512));
  const payloadEnd = Math.max(payloadStart + 1, tailStart);
  return {
    header: { start: 0, end: Math.max(1, headerEnd) },
    index: { start: indexStart, end: Math.max(indexStart + 1, indexEnd) },
    payload: { start: payloadStart, end: Math.max(payloadStart + 1, payloadEnd) },
    tail: { start: Math.max(0, tailStart), end: Math.max(tailStart + 1, bytesLength) },
  };
}

function boundedOffset(range, len) {
  const maxOffset = Math.max(range.start, range.end - len);
  return Math.min(maxOffset, range.start + Math.floor((range.end - range.start - len) / 2));
}

function magnitudeBytes(bytesLength, magnitude) {
  if (magnitude === "1B") return 1;
  if (magnitude === "256B") return Math.min(256, Math.max(16, Math.floor(bytesLength / 64)));
  return Math.min(4096, Math.max(512, Math.floor(bytesLength / 3)));
}

const PRESETS = {
  "p2-rep-payload-bitflip-1b": {
    tier: "representative",
    harnessType: "bit_flip",
    harnessTarget: "payload",
    harnessMagnitude: "1B",
    description: "Phase 2 mapping: payload bit flip at 1B magnitude.",
    explanation: [
      "Emulates a small number of random bit errors in stored or transferred data.",
      "Usually keeps container structure readable while making some payload verification fail.",
    ],
    configFor(bytes) {
      const windowStart = Math.min(bytes.length - 1, Math.max(128, Math.floor(bytes.length * 0.25)));
      const windowSpan = Math.max(64, Math.floor(bytes.length * 0.15));
      return {
        mode: "random_flip",
        seed: 1337,
        flip_count: 1,
        random_flip_offset: windowStart,
        random_flip_span: Math.min(windowSpan, Math.max(1, bytes.length - windowStart - 2048)),
      };
    },
    fileTag: "p2-bit_flip-payload-1b-representative",
  },
  "p2-rep-payload-overwrite-1b": {
    tier: "representative",
    harnessType: "byte_overwrite",
    harnessTarget: "payload",
    harnessMagnitude: "1B",
    description: "Phase 2 mapping: payload byte overwrite at 1B magnitude.",
    explanation: [
      "Emulates a tiny wrong-byte write inside payload data.",
      "Often keeps index/footer intact but can invalidate one entry's integrity.",
    ],
    configFor(bytes) {
      const range = buildRanges(bytes.length).payload;
      return {
        mode: "overwrite",
        overwrite_offset: boundedOffset(range, 1),
        overwrite_len: 1,
        overwrite_value: 0xa5,
      };
    },
    fileTag: "p2-byte_overwrite-payload-1b-representative",
  },
  "p2-rep-payload-zerofill-256b": {
    tier: "representative",
    harnessType: "zero_fill",
    harnessTarget: "payload",
    harnessMagnitude: "256B",
    description: "Phase 2 mapping: payload zero-fill at 256B magnitude.",
    explanation: [
      "Emulates a bounded damaged payload span (for example sector-level data loss).",
      "Intended to show degraded payload truth while preserving archive inspectability.",
    ],
    configFor(bytes) {
      const range = buildRanges(bytes.length).payload;
      const len = Math.min(magnitudeBytes(bytes.length, "256B"), Math.max(1, range.end - range.start));
      return {
        mode: "overwrite",
        overwrite_offset: boundedOffset(range, len),
        overwrite_len: len,
        overwrite_value: 0x00,
      };
    },
    fileTag: "p2-zero_fill-payload-256b-representative",
  },
  "p2-stress-index-overwrite-1b": {
    tier: "stress",
    harnessType: "byte_overwrite",
    harnessTarget: "index",
    harnessMagnitude: "1B",
    description: "Phase 2 mapping: index byte overwrite at 1B magnitude (structural stress).",
    explanation: [
      "Emulates a small metadata/index corruption event.",
      "Can still be informative, but structure-level failures become more likely than payload-only cases.",
    ],
    configFor(bytes) {
      const range = buildRanges(bytes.length).index;
      return { mode: "overwrite", overwrite_offset: range.start, overwrite_len: 1, overwrite_value: 0x5a };
    },
    fileTag: "p2-byte_overwrite-index-1b-stress",
  },
  "p2-stress-tail-bitflip-1b": {
    tier: "stress",
    harnessType: "bit_flip",
    harnessTarget: "tail",
    harnessMagnitude: "1B",
    description: "Phase 2 mapping: tail bit flip at 1B magnitude (structural stress).",
    explanation: [
      "Emulates subtle tail-region corruption where footer/index references live.",
      "Can produce either partial inspectability or immediate structure diagnostics depending on hit location.",
    ],
    configFor(bytes) {
      const range = buildRanges(bytes.length).tail;
      return {
        mode: "random_flip",
        seed: 2600,
        flip_count: 1,
        random_flip_offset: range.start,
        random_flip_span: Math.max(1, range.end - range.start),
      };
    },
    fileTag: "p2-bit_flip-tail-1b-stress",
  },
  "p2-stress-index-zerofill-256b": {
    tier: "stress",
    harnessType: "zero_fill",
    harnessTarget: "index",
    harnessMagnitude: "256B",
    description: "Phase 2 mapping: index zero-fill at 256B magnitude (structural stress).",
    explanation: [
      "Emulates heavier index metadata damage than single-byte stress.",
      "Often shifts from degraded behavior into hard structure invalidation.",
    ],
    configFor(bytes) {
      const range = buildRanges(bytes.length).index;
      const len = Math.min(magnitudeBytes(bytes.length, "256B"), Math.max(1, range.end - range.start));
      return { mode: "overwrite", overwrite_offset: range.start, overwrite_len: len, overwrite_value: 0x00 };
    },
    fileTag: "p2-zero_fill-index-256b-stress",
  },
  "p2-cat-truncation-tail-4kb": {
    tier: "catastrophic",
    harnessType: "truncation",
    harnessTarget: "tail",
    harnessMagnitude: "4KB",
    description: "Phase 2 mapping: truncation at 4KB magnitude (catastrophic).",
    explanation: [
      "Emulates severe trailing data loss (cut archive / incomplete write).",
      "This is expected to invalidate container structure in many cases.",
    ],
    configFor(bytes) {
      const cut = Math.max(0, bytes.length - magnitudeBytes(bytes.length, "4KB"));
      return { mode: "truncate", truncate_offset: cut };
    },
    fileTag: "p2-truncation-tail-4kb-catastrophic",
  },
  "p2-cat-tail-damage-4kb": {
    tier: "catastrophic",
    harnessType: "tail_damage",
    harnessTarget: "tail",
    harnessMagnitude: "4KB",
    description: "Phase 2 mapping: tail damage at 4KB magnitude (catastrophic).",
    explanation: [
      "Emulates aggressive damage to tail structures (footer/index/tail frame zone).",
      "Primarily a container-failure boundary demonstration, not a representative corruption case.",
    ],
    configFor(bytes) {
      const range = buildRanges(bytes.length).tail;
      const len = Math.min(magnitudeBytes(bytes.length, "4KB"), Math.max(1, range.end - range.start));
      const offset = Math.max(range.start, range.end - len);
      return { mode: "overwrite", overwrite_offset: offset, overwrite_len: len, overwrite_value: 0xff };
    },
    fileTag: "p2-tail_damage-tail-4kb-catastrophic",
  },
  "p2-cat-header-zerofill-4kb": {
    tier: "catastrophic",
    harnessType: "zero_fill",
    harnessTarget: "header",
    harnessMagnitude: "4KB",
    description: "Phase 2 mapping: header zero-fill at 4KB magnitude (catastrophic).",
    explanation: [
      "Emulates major corruption at archive start / header region.",
      "Likely to destroy structural openability and should be treated as a kill-shot case.",
    ],
    configFor(bytes) {
      const len = Math.min(4096, bytes.length);
      return { mode: "overwrite", overwrite_offset: 0, overwrite_len: len, overwrite_value: 0x00 };
    },
    fileTag: "p2-zero_fill-header-4kb-catastrophic",
  },
};

function getSelectedPreset() {
  return PRESETS[modeSelect.value] ?? PRESETS["p2-rep-payload-bitflip-1b"];
}

function updateModeControls() {
  const preset = getSelectedPreset();
  modeDescription.textContent = preset.description;
  const tierLabel =
    preset.tier === "representative"
      ? "Representative corruption preset"
      : preset.tier === "stress"
        ? "Structural stress preset"
        : "Catastrophic / kill-shot preset";
  presetDetails.textContent =
    `Tier: ${tierLabel} • Phase 2 mapping: type=${preset.harnessType}, target=${preset.harnessTarget}, magnitude=${preset.harnessMagnitude}`;
  explanationSummary.textContent = `${tierLabel}. This preset emulates ${preset.harnessType} on ${preset.harnessTarget} at ${preset.harnessMagnitude}.`;
  explanationList.innerHTML = "";
  for (const line of preset.explanation) {
    const li = document.createElement("li");
    li.textContent = line;
    explanationList.appendChild(li);
  }
}

function buildConfig(sourceBytesValue) {
  const preset = getSelectedPreset();
  return preset.configFor(sourceBytesValue);
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

function buildDownloadName(originalName, mode, config, seedApplied, presetKey) {
  const base = originalName.toLowerCase().endsWith(".crs")
    ? originalName.slice(0, -4)
    : originalName;
  const modePart = modeLabel(mode);
  const presetPart = PRESETS[presetKey]?.fileTag ?? presetKey;
  const seedPart = seedApplied == null ? "seedna" : `seed${seedApplied}`;

  const details = [];
  if (mode === "random_flip") {
    details.push(`count${config.flip_count}`);
    if (config.random_flip_offset != null) details.push(`off${config.random_flip_offset}`);
    if (config.random_flip_span != null) details.push(`span${config.random_flip_span}`);
  }
  if (mode === "overwrite") details.push(`off${config.overwrite_offset}`, `len${config.overwrite_len}`);
  if (mode === "truncate") details.push(`off${config.truncate_offset}`);
  if (mode === "remove") details.push(`off${config.remove_offset}`, `len${config.remove_len}`);

  return `${base}.corrupted-${presetPart}-${modePart}-${seedPart}-${details.join("-")}.crs`;
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

  const presetKey = modeSelect.value;
  const config = buildConfig(sourceBytes);
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
    downloadLink.download = buildDownloadName(sourceName, config.mode, config, result.seed_applied, presetKey);
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
