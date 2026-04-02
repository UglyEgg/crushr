const WASM_MODULE_PATHS = ["./pkg/crushr_wasm_readonly_demo.js", "../pkg/crushr_wasm_readonly_demo.js"];
const EMPTY_ARCHIVE_ARG = new Uint8Array();

let wasmFns = null;
let wasmInitError = null;
let loadedStatePrepared = false;

async function initializeWasmRuntime() {
  if (wasmFns) {
    return wasmFns;
  }
  for (const path of WASM_MODULE_PATHS) {
    try {
      const module = await import(path);
      await module.default();
      module.init();
      wasmFns = {
        archive_summary: module.archive_summary,
        prepare_loaded_archive_state: module.prepare_loaded_archive_state,
        reset_loaded_archive: module.reset_loaded_archive,
        find: module.find,
        entry: module.entry,
        entry_preview: module.entry_preview,
        propagation: module.propagation,
      };
      return wasmFns;
    } catch (error) {
      wasmInitError = error;
    }
  }
  throw wasmInitError ?? new Error("Unknown WASM initialization failure.");
}

function formatWorkerError(error, actionLabel) {
  const detail = String(error);
  const action = actionLabel.replace(/\.\.\.$/, "");
  if (detail.includes("RuntimeError: unreachable")) {
    return `${action} failed due to an internal WASM runtime error. Reload the archive and retry. Details: ${detail}`;
  }
  return `${action} failed: ${detail}`;
}

function postResponse(id, ok, payload) {
  self.postMessage({ type: "response", id, ok, ...payload });
}

function postStatus(id, stage, message) {
  self.postMessage({ type: "status", id, stage, message });
}

async function handleLoadArchive(id, data) {
  const runtime = await initializeWasmRuntime();
  runtime.reset_loaded_archive();
  loadedStatePrepared = false;

  postStatus(id, "load_reading_archive", "Reading archive...");
  const summary = runtime.archive_summary(data.fileName, data.bytes);

  postStatus(id, "load_inspecting_summary", "Inspecting archive summary...");

  postStatus(id, "load_ready", "Ready");
  postResponse(id, true, { result: { summary } });
}

async function handleSearch(id, data) {
  const runtime = await initializeWasmRuntime();
  if (!loadedStatePrepared) {
    postStatus(id, "search_preparing_state", "Preparing search state...");
    runtime.prepare_loaded_archive_state();
    loadedStatePrepared = true;
  }
  postStatus(id, "search_busy", "Searching...");
  const matches = runtime.find(EMPTY_ARCHIVE_ARG, data.query);
  postResponse(id, true, { result: { matches } });
}

async function handleEntry(id, data) {
  const runtime = await initializeWasmRuntime();
  if (!loadedStatePrepared) {
    postStatus(id, "entry_preparing_state", "Preparing entry state...");
    runtime.prepare_loaded_archive_state();
    loadedStatePrepared = true;
  }
  const detail = runtime.entry(EMPTY_ARCHIVE_ARG, data.path);
  postResponse(id, true, { result: { detail } });
}

async function handleEntryPreview(id, data) {
  const runtime = await initializeWasmRuntime();
  if (!loadedStatePrepared) {
    postStatus(id, "entry_preparing_state", "Preparing entry state...");
    runtime.prepare_loaded_archive_state();
    loadedStatePrepared = true;
  }
  const preview = runtime.entry_preview(data.path);
  postResponse(id, true, { result: { preview } });
}

async function handlePropagation(id) {
  const runtime = await initializeWasmRuntime();
  const report = runtime.propagation(EMPTY_ARCHIVE_ARG);
  postResponse(id, true, { result: { report } });
}

async function handleReset(id) {
  const runtime = await initializeWasmRuntime();
  runtime.reset_loaded_archive();
  loadedStatePrepared = false;
  postResponse(id, true, { result: {} });
}

self.onmessage = async (event) => {
  const { id, type, data } = event.data ?? {};
  if (!id || !type) {
    return;
  }

  try {
    if (type === "loadArchive") {
      await handleLoadArchive(id, data ?? {});
      return;
    }
    if (type === "search") {
      await handleSearch(id, data ?? {});
      return;
    }
    if (type === "entry") {
      await handleEntry(id, data ?? {});
      return;
    }
    if (type === "propagation") {
      await handlePropagation(id);
      return;
    }
    if (type === "entryPreview") {
      await handleEntryPreview(id, data ?? {});
      return;
    }
    if (type === "reset") {
      await handleReset(id);
      return;
    }
    throw new Error(`Unknown worker request type: ${type}`);
  } catch (error) {
    postResponse(id, false, { error: formatWorkerError(error, `${type}...`) });
  }
};
