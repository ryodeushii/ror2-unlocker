use ror2_profile_core::{Catalog, ChangeRequest, Preview, ProfileSnapshot, SaveResult};
use std::path::Path;

#[tauri::command]
fn get_catalog() -> &'static Catalog {
    ror2_profile_core::catalog()
}

#[tauri::command]
fn open_profile(path: String) -> Result<ProfileSnapshot, String> {
    ror2_profile_core::open_profile(Path::new(&path))
}

#[tauri::command]
fn preview_changes(args: ChangeRequest) -> Result<Preview, String> {
    ror2_profile_core::preview_changes(&args)
}

#[tauri::command]
fn save_changes(args: ChangeRequest) -> Result<SaveResult, String> {
    ror2_profile_core::save_changes(&args)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_catalog,
            open_profile,
            preview_changes,
            save_changes
        ])
        .run(tauri::generate_context!())
        .expect("failed to run ror2-unlocker");
}
