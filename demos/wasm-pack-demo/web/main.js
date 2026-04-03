import initWasm, { init, pack_files } from "../pkg/crushr_wasm_pack_demo.js";

const MAX_TOTAL_BYTES = 256 * 1024 * 1024;
const MAX_FILES = 1000;
const MAX_SINGLE_FILE_BYTES = 128 * 1024 * 1024;

const dropZone = document.getElementById("dropZone");
const fileInput = document.getElementById("fileInput");
const folderInput = document.getElementById("folderInput");
const selectionSummary = document.getElementById("selectionSummary");
const packBtn = document.getElementById("packBtn");
const statusEl = document.getElementById("status");
const errorEl = document.getElementById("error");
const downloadLink = document.getElementById("downloadLink");
const resetBtn = document.getElementById("resetBtn");
const packLog = document.getElementById("packLog");
const themeToggleBtn = document.getElementById("themeToggleBtn");

const THEME_STORAGE_KEY = "crushr_wasm_pack_demo_theme";
const MAX_LOG_LINES = 40;

let selectedFiles = [];
let downloadUrl = null;

function toArchiveBytes(value) {
  if (value instanceof Uint8Array) {
    return value;
  }
  if (ArrayBuffer.isView(value)) {
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  }
  if (value instanceof ArrayBuffer) {
    return new Uint8Array(value);
  }
  if (Array.isArray(value)) {
    return Uint8Array.from(value);
  }
  throw new Error("WASM pack output is not a byte array.");
}

function setStatus(stage) {
  statusEl.textContent = `Status: ${stage}`;
}

function appendLog(message) {
  const item = document.createElement("li");
  item.textContent = `${new Date().toLocaleTimeString()} — ${message}`;
  packLog.appendChild(item);
  while (packLog.children.length > MAX_LOG_LINES) {
    packLog.removeChild(packLog.firstChild);
  }
  packLog.scrollTop = packLog.scrollHeight;
}

function resetLog(seedMessage = "Log reset.") {
  packLog.textContent = "";
  appendLog(seedMessage);
}

function clearError() {
  errorEl.hidden = true;
  errorEl.textContent = "";
}

function showError(message) {
  errorEl.hidden = false;
  errorEl.textContent = message;
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

function setTheme(theme) {
  document.documentElement.dataset.theme = theme;
  themeToggleBtn.textContent = theme === "light" ? "Theme: Light" : "Theme: Dark";
}

function initTheme() {
  const stored = localStorage.getItem(THEME_STORAGE_KEY);
  if (stored === "light" || stored === "dark") {
    setTheme(stored);
    return;
  }
  const prefersDark = globalThis.matchMedia?.("(prefers-color-scheme: dark)")?.matches ?? true;
  setTheme(prefersDark ? "dark" : "light");
}

function normalizePath(file) {
  return file.webkitRelativePath && file.webkitRelativePath.length > 0
    ? file.webkitRelativePath
    : file.name;
}

function validate(files) {
  if (files.length > MAX_FILES) {
    return `Input exceeds the 1,000 file demo limit (received ${files.length}).`;
  }

  let total = 0;
  for (const file of files) {
    if (file.size > MAX_SINGLE_FILE_BYTES) {
      return `A file exceeds the 128 MiB per-file demo limit (${normalizePath(file)}).`;
    }
    total += file.size;
  }

  if (total > MAX_TOTAL_BYTES) {
    return `Total input exceeds 256 MiB demo limit (${(total / (1024 * 1024)).toFixed(1)} MiB).`;
  }

  return null;
}

function setSelection(files) {
  resetLog(files.length === 0 ? "Selection cleared." : "Input selection updated.");
  clearError();
  clearDownload();
  selectedFiles = files;

  if (files.length === 0) {
    selectionSummary.textContent = "No files selected.";
    packBtn.disabled = true;
    setStatus("idle");
    return;
  }

  const total = files.reduce((sum, file) => sum + file.size, 0);
  selectionSummary.textContent = `${files.length} files selected, ${(total / (1024 * 1024)).toFixed(2)} MiB total.`;

  const violation = validate(files);
  if (violation) {
    appendLog("Validating limits...");
    appendLog(violation);
    showError(violation);
    packBtn.disabled = true;
    setStatus("blocked by demo limits");
    return;
  }
  appendLog("Validating limits...");
  appendLog("Input accepted within demo limits.");

  packBtn.disabled = false;
  setStatus("ready to pack");
}

async function listDroppedFiles(dataTransferItems) {
  const files = [];
  for (const item of dataTransferItems) {
    if (item.kind === "file") {
      const file = item.getAsFile();
      if (file) files.push(file);
    }
  }
  return files;
}

async function onPack() {
  resetLog("Starting pack run.");
  clearError();
  clearDownload();

  if (selectedFiles.length === 0) {
    appendLog("No files selected.");
    showError("Select or drop files before packing.");
    return;
  }

  appendLog("Validating limits...");
  const violation = validate(selectedFiles);
  if (violation) {
    appendLog(violation);
    showError(violation);
    return;
  }
  appendLog("Limits validated.");

  try {
    packBtn.disabled = true;
    setStatus("reading input...");
    appendLog("Reading input...");
    const payload = [];
    for (const file of selectedFiles) {
      const bytes = new Uint8Array(await file.arrayBuffer());
      payload.push({
        path: normalizePath(file),
        mtime_unix_seconds: Math.floor(file.lastModified / 1000),
        bytes,
      });
    }

    setStatus("preparing archive...");
    appendLog("Preparing archive...");
    await Promise.resolve();
    setStatus("packing archive...");
    appendLog("Packing archive...");
    const packed = pack_files(payload);
    appendLog("Finalizing archive...");

    const archiveBytes = toArchiveBytes(packed.archive_bytes);
    const blob = new Blob([archiveBytes], { type: "application/octet-stream" });
    downloadUrl = URL.createObjectURL(blob);
    const stamp = new Date().toISOString().replace(/[:.]/g, "-");
    downloadLink.href = downloadUrl;
    downloadLink.download = `crushr_browser_demo_${stamp}.crs`;
    downloadLink.hidden = false;

    setStatus("ready for download");
    appendLog("Ready for download.");
  } catch (error) {
    showError(error instanceof Error ? error.message : String(error));
    setStatus("error");
    appendLog(`Error: ${error instanceof Error ? error.message : String(error)}`);
  } finally {
    packBtn.disabled = selectedFiles.length === 0 || Boolean(validate(selectedFiles));
  }
}

function resetDemoState() {
  fileInput.value = "";
  folderInput.value = "";
  selectedFiles = [];
  clearError();
  clearDownload();
  selectionSummary.textContent = "No files selected.";
  packBtn.disabled = true;
  setStatus("idle");
  resetLog("Demo state reset.");
}

fileInput.addEventListener("change", () => setSelection(Array.from(fileInput.files || [])));
folderInput.addEventListener("change", () => setSelection(Array.from(folderInput.files || [])));
packBtn.addEventListener("click", onPack);
resetBtn.addEventListener("click", resetDemoState);
themeToggleBtn.addEventListener("click", () => {
  const nextTheme = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  setTheme(nextTheme);
  localStorage.setItem(THEME_STORAGE_KEY, nextTheme);
});

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

dropZone.addEventListener("drop", async (event) => {
  const dropped = await listDroppedFiles(event.dataTransfer?.items || []);
  setSelection(dropped);
});

await initWasm();
init();
initTheme();
setStatus("idle");
resetLog("Ready.");
