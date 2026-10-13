import init, {
  edit_profile,
  get_catalog,
  open_profile,
  preview_changes,
} from "../wasm/pkg/ror2_profile_wasm.js";
import wasmUrl from "../wasm/pkg/ror2_profile_wasm_bg.wasm?url";
import type { ProfileBackend } from "./backend";
import type { Catalog, Preview, ProfileSnapshot } from "./types";

const MAX_PROFILE_BYTES = 32 * 1024 * 1024;
let initialization: Promise<unknown> | null = null;
let source: { bytes: Uint8Array<ArrayBuffer>; filename: string } | null = null;

async function ready(): Promise<void> {
  initialization ??= init({ module_or_path: wasmUrl }).catch(
    (error: unknown) => {
      initialization = null;
      throw error;
    },
  );
  await initialization;
}

function selectedFile(input: HTMLInputElement): Promise<File | null> {
  input.value = "";
  return new Promise((resolve, reject) => {
    const finish = (file: File | null) => {
      input.removeEventListener("change", changed);
      input.removeEventListener("cancel", canceled);
      resolve(file);
    };
    const changed = () => finish(input.files?.item(0) ?? null);
    const canceled = () => finish(null);
    input.addEventListener("change", changed);
    input.addEventListener("cancel", canceled);
    try {
      input.click();
    } catch (error) {
      input.removeEventListener("change", changed);
      input.removeEventListener("cancel", canceled);
      reject(error);
    }
  });
}

function currentSource() {
  if (!source) throw new Error("Open a profile XML first.");
  return source;
}

export const backend: ProfileBackend = {
  async getCatalog() {
    await ready();
    return get_catalog() as Catalog;
  },
  async pickProfile(input) {
    const file = await selectedFile(input);
    if (!file) return null;
    if (file.size > MAX_PROFILE_BYTES)
      throw new Error("Profile exceeds the 32 MiB size limit.");
    const bytes = new Uint8Array(await file.arrayBuffer());
    await ready();
    const snapshot = open_profile(bytes, file.name) as ProfileSnapshot;
    // Commit the new source only after Rust accepts it; failed opens preserve the draft.
    source = { bytes, filename: file.name };
    return snapshot;
  },
  async preview(args) {
    await ready();
    return preview_changes(currentSource().bytes, args) as Preview;
  },
  async save(args) {
    await ready();
    const original = currentSource();
    const result = edit_profile(original.bytes, args) as {
      profile: ProfileSnapshot;
      preview: Preview;
      xml: string;
    };
    return {
      ...result,
      kind: "downloads",
      original: original.bytes,
      filename: original.filename,
    };
  },
};
