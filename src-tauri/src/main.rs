#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use keyring::Entry;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde_json::Value;

const KEYRING_SERVICE: &str = "NOVA Client";
const KEYRING_USER: &str = "curseforge-api-key";
const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";

fn credential_entry() -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Windows Credential Manager is unavailable: {e}"))
}

#[tauri::command]
fn has_curseforge_key() -> bool {
    credential_entry().and_then(|entry| entry.get_password().map_err(|e| e.to_string())).is_ok()
}

#[tauri::command]
fn save_curseforge_key(api_key: String) -> Result<(), String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("Enter a CurseForge API key first.".into());
    }
    if key.len() > 512 || key.chars().any(char::is_control) {
        return Err("The API key format is invalid.".into());
    }
    credential_entry()?
        .set_password(key)
        .map_err(|e| format!("Could not save the key to Windows Credential Manager: {e}"))
}

#[tauri::command]
fn delete_curseforge_key() -> Result<(), String> {
    match credential_entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Could not remove the saved CurseForge key: {e}")),
    }
}

#[tauri::command]
async fn curseforge_search(
    query: String,
    packs: bool,
    index: u32,
    game_version: Option<String>,
) -> Result<Value, String> {
    let api_key = credential_entry()?
        .get_password()
        .map_err(|_| "Save your CurseForge API key in the catalog first.".to_string())?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(&api_key).map_err(|_| "The saved CurseForge API key is invalid.".to_string())?,
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    let class_id = if packs { "4471" } else { "6" };
    let client = reqwest::Client::new();
    let mut request = client
        .get(format!("{CURSEFORGE_API}/mods/search"))
        .headers(headers)
        .query(&[
            ("gameId", "432".to_string()),
            ("classId", class_id.to_string()),
            ("pageSize", "24".to_string()),
            ("index", index.to_string()),
            ("sortField", "2".to_string()),
            ("sortOrder", "desc".to_string()),
        ]);

    let trimmed = query.trim();
    if !trimmed.is_empty() {
        request = request.query(&[("searchFilter", trimmed)]);
    }
    if let Some(version) = game_version.as_deref().filter(|v| !v.trim().is_empty()) {
        request = request.query(&[("gameVersion", version)]);
    }

    let response = request.send().await.map_err(|e| format!("Could not reach CurseForge: {e}"))?;
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err("CurseForge rejected the saved API key. Check that the key is approved and active.".into());
    }
    if !status.is_success() {
        return Err(format!("CurseForge catalog request failed ({status})."));
    }
    response.json::<Value>().await.map_err(|e| format!("CurseForge returned an unreadable response: {e}"))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            has_curseforge_key,
            save_curseforge_key,
            delete_curseforge_key,
            curseforge_search
        ])
        .run(tauri::generate_context!())
        .expect("error while running NOVA Client");
}
