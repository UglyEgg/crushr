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

const PRESETS = {
  scattered_random_damage: {
    label: "Scattered random damage",
    category: "kill",
    description: "Scattered deterministic bit flips (can still invalidate index/footer structures).",
    configFor(bytes) {
      const windowStart = Math.min(Math.max(96, Math.floor(bytes.length * 0.12)), Math.max(0, bytes.length - 2));
      const tailGuard = 768;
      const maxWindowSpan = Math.max(1, bytes.length - windowStart - tailGuard);
      const windowSpan = Math.min(maxWindowSpan, Math.max(256, Math.floor(bytes.length * 0.45)));
      const count = Math.min(12, Math.max(3, Math.floor(bytes.length / 131072)));
      return {
        mode: "random_flip",
        seed: 42,
        flip_count: count,
        random_flip_offset: windowStart,
        random_flip_span: windowSpan,
      };
    },
    fileTag: "killshot-scattered-random",
  },
  bounded_middle_overwrite: {
    label: "Bounded middle overwrite",
    category: "demo",
    description: "Overwrite a small bounded payload region in the middle of the archive.",
    configFor(bytes) {
      const len = Math.min(64, Math.max(24, Math.floor(bytes.length / 12288)));
      const maxOffset = Math.max(0, bytes.length - len);
      const offset = Math.min(maxOffset, Math.max(64, Math.floor(bytes.length * 0.32)));
      return { mode: "overwrite", overwrite_offset: offset, overwrite_len: len, overwrite_value: 0xa5 };
    },
    fileTag: "demo-middle-overwrite",
  },
  bounded_payload_overwrite: {
    label: "Bounded payload overwrite",
    category: "demo",
    description: "Overwrite a bounded payload window away from header and tail structures.",
    configFor(bytes) {
      const len = Math.min(64, Math.max(20, Math.floor(bytes.length / 12288)));
      const maxOffset = Math.max(0, bytes.length - len - 512);
      const offset = Math.min(maxOffset, Math.max(96, Math.floor(bytes.length * 0.25)));
      return { mode: "overwrite", overwrite_offset: offset, overwrite_len: len, overwrite_value: 0x5a };
    },
    fileTag: "demo-payload-overwrite",
  },
  bounded_header_overwrite: {
    label: "Bounded header damage",
    category: "demo",
    description: "Damage an early header-adjacent range while avoiding offset 0.",
    configFor(bytes) {
      const len = Math.min(40, Math.max(12, Math.floor(bytes.length / 24576)));
      const maxOffset = Math.max(0, bytes.length - len);
      const offset = Math.min(maxOffset, 32);
      return { mode: "overwrite", overwrite_offset: offset, overwrite_len: len, overwrite_value: 0x00 };
    },
    fileTag: "demo-header-damage",
  },
  killshot_tail_overwrite: {
    label: "Kill-shot: tail structure damage",
    category: "kill",
    description: "Overwrite near archive tail (likely footer/index invalidation).",
    configFor(bytes) {
      const len = Math.min(160, Math.max(48, Math.floor(bytes.length / 3072)));
      const offset = Math.max(0, bytes.length - len - 64);
      return { mode: "overwrite", overwrite_offset: offset, overwrite_len: len, overwrite_value: 0xff };
    },
    fileTag: "killshot-tail-overwrite",
  },
  killshot_truncate_start: {
    label: "Kill-shot: truncate at offset 0",
    category: "kill",
    description: "Truncate to 0 bytes (structural destruction; likely invalid container).",
    configFor() {
      return { mode: "truncate", truncate_offset: 0 };
    },
    fileTag: "killshot-truncate-off0",
  },
  killshot_remove_start: {
    label: "Kill-shot: remove from offset 0",
    category: "kill",
    description: "Remove bytes from the archive start (structural destruction likely).",
    configFor(bytes) {
      const len = Math.min(128, Math.max(32, Math.floor(bytes.length / 3072)));
      return { mode: "remove", remove_offset: 0, remove_len: len };
    },
    fileTag: "killshot-remove-off0",
  },
  killshot_middle_remove_window: {
    label: "Kill-shot: remove middle window",
    category: "kill",
    description: "Remove a middle byte window (often breaks index/tail offsets).",
    configFor(bytes) {
      const len = Math.min(64, Math.max(24, Math.floor(bytes.length / 6144)));
      const maxOffset = Math.max(0, bytes.length - len);
      const offset = Math.min(maxOffset, Math.floor(bytes.length * 0.4));
      return { mode: "remove", remove_offset: offset, remove_len: len };
    },
    fileTag: "killshot-middle-remove",
  },
};

function getSelectedPreset() {
  return PRESETS[modeSelect.value] ?? PRESETS.scattered_random_damage;
}

function updateModeControls() {
  const preset = getSelectedPreset();
  modeDescription.textContent = preset.description;
  presetDetails.textContent = `Preset category: ${
    preset.category === "kill" ? "Structural destruction / kill-shot" : "Demo corruption preset"
  }`;
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
