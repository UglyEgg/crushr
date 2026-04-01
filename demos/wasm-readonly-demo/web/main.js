import init, { init as setup, archive_summary, find, entry } from "../pkg/crushr_wasm_readonly_demo.js";

const dropZoneEl = document.getElementById("drop-zone");
const fileEl = document.getElementById("file");
const summaryEl = document.getElementById("summary");
const queryEl = document.getElementById("query");
const resultsEl = document.getElementById("results");
const entryEl = document.getElementById("entry");
const errorEl = document.getElementById("error");
const searchBtn = document.getElementById("search");

let bytes = null;

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
    summaryEl.textContent = render(summary);
    resultsEl.innerHTML = "";
    entryEl.textContent = "";
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
    resultsEl.innerHTML = "";
    for (const match of matches) {
      const item = document.createElement("li");
      const button = document.createElement("button");
      button.textContent = `${match.path} (${match.trust_class})`;
      button.addEventListener("click", () => {
        const detail = entry(bytes, match.path);
        entryEl.textContent = render(detail);
      });
      item.appendChild(button);
      resultsEl.appendChild(item);
    }
  } catch (error) {
    setError(String(error));
  }
});
