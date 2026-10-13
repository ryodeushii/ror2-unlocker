import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ProfileBackend } from "./backend";
import type { Catalog, Preview, ProfileSnapshot, SaveResult } from "./types";

export const backend: ProfileBackend = {
  getCatalog: () => invoke<Catalog>("get_catalog"),
  async pickProfile() {
    const path = await open({
      multiple: false,
      directory: false,
      title: "Open Risk of Rain 2 profile",
      filters: [{ name: "UserProfile XML", extensions: ["xml"] }],
    });
    return typeof path === "string"
      ? invoke<ProfileSnapshot>("open_profile", { path })
      : null;
  },
  preview: (args) => invoke<Preview>("preview_changes", { args }),
  async save(args) {
    const result = await invoke<SaveResult>("save_changes", { args });
    return { ...result, kind: "saved" };
  },
};
