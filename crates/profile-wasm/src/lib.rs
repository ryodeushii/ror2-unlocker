use ror2_profile_core::{
    catalog, edit_profile_bytes, inspect_profile_bytes, preview_profile_bytes, ChangeRequest,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_catalog() -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(catalog())
        .map_err(|error| JsValue::from_str(&format!("Cannot serialize catalog: {error}")))
}

#[wasm_bindgen]
pub fn open_profile(bytes: Vec<u8>, name: &str) -> Result<JsValue, JsValue> {
    let profile = inspect_profile_bytes(name, bytes).map_err(|error| JsValue::from_str(&error))?;
    serde_wasm_bindgen::to_value(&profile)
        .map_err(|error| JsValue::from_str(&format!("Cannot serialize profile: {error}")))
}

#[wasm_bindgen]
pub fn preview_changes(bytes: Vec<u8>, args: JsValue) -> Result<JsValue, JsValue> {
    let args: ChangeRequest = serde_wasm_bindgen::from_value(args)
        .map_err(|error| JsValue::from_str(&format!("Invalid change request: {error}")))?;
    let preview = preview_profile_bytes(bytes, &args).map_err(|error| JsValue::from_str(&error))?;
    serde_wasm_bindgen::to_value(&preview)
        .map_err(|error| JsValue::from_str(&format!("Cannot serialize preview: {error}")))
}

#[wasm_bindgen]
pub fn edit_profile(bytes: Vec<u8>, args: JsValue) -> Result<JsValue, JsValue> {
    let args: ChangeRequest = serde_wasm_bindgen::from_value(args)
        .map_err(|error| JsValue::from_str(&format!("Invalid change request: {error}")))?;
    let edited = edit_profile_bytes(bytes, &args).map_err(|error| JsValue::from_str(&error))?;
    serde_wasm_bindgen::to_value(&edited)
        .map_err(|error| JsValue::from_str(&format!("Cannot serialize edited profile: {error}")))
}
