#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crust_core::authenticator::{Account, Authenticator};
use crust_core::launcher::{Launch, LaunchOptions, LoaderKind};
use keyring::Entry;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use std::io::{BufRead, BufReader, Cursor, Read, Write};
use std::net::TcpListener;
use std::path::{Component, Path, PathBuf};
use zip::ZipArchive;
use tauri::Emitter;

const KEYRING_SERVICE: &str = "NOVA Client";
const MINECRAFT_ACCOUNT_USER: &str = "minecraft-account";
const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const MICROSOFT_CLIENT_ID: &str = "09e6a2a2-90ee-47a6-92c4-f8810958de91";

fn credential_entry(user: &str) -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, user)
        .map_err(|e| format!("Windows Credential Manager is unavailable: {e}"))
}

// The approved NOVA key is injected by GitHub Actions at compile time.
// This embeds it in the private NOVA build; do not distribute that binary publicly.
fn curseforge_api_key() -> Result<String, String> {
    option_env!("CURSEFORGE_API_KEY")
        .filter(|key| !key.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "This NOVA build was created without its CurseForge API credential. Rebuild with the CURSEFORGE_API_KEY GitHub Actions secret.".to_string())
}

#[tauri::command]
async fn curseforge_search(
    query: String,
    packs: bool,
    index: u32,
    game_version: Option<String>,
) -> Result<Value, String> {
    let api_key = curseforge_api_key()?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(&api_key)
            .map_err(|_| "The saved CurseForge API key is invalid.".to_string())?,
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

#[tauri::command]
async fn begin_microsoft_login(app: tauri::AppHandle) -> Result<String, String> {
    // Use NOVA's approved public-client app and the authorization-code + PKCE flow.
    // Microsoft opens the normal account sign-in page; the loopback callback returns
    // the authorization response directly to this desktop app (no device code).
    let auth = Authenticator::with_client_id(MICROSOFT_CLIENT_ID)
        .map_err(|e| format!("Could not start Microsoft sign-in: {e}"))?;
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| format!("Could not start the local Microsoft sign-in callback: {e}"))?;
    let port = listener.local_addr()
        .map_err(|e| format!("Could not prepare the Microsoft sign-in callback: {e}"))?
        .port();
    let redirect_uri = format!("http://localhost:{port}");
    let request = auth.authorize_request_with_redirect(redirect_uri.clone());
    let login_url = request.url.clone();

    app.emit("nova-login-link", login_url.clone())
        .map_err(|e| format!("Could not show the Microsoft sign-in link: {e}"))?;
    webbrowser::open(&login_url)
        .map_err(|e| format!("Could not open the Microsoft sign-in page: {e}"))?;

    let redirect_url = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let (stream, _) = listener.accept()
            .map_err(|e| format!("Could not receive Microsoft's sign-in response: {e}"))?;
        let mut reader = BufReader::new(stream);
        let mut first_line = String::new();
        reader.read_line(&mut first_line)
            .map_err(|e| format!("Could not read Microsoft's sign-in response: {e}"))?;
        let mut stream = reader.into_inner();

        let target = first_line.split_whitespace().nth(1)
            .ok_or_else(|| "Microsoft returned an invalid sign-in response.".to_string())?;
        if !target.starts_with('/') || target.starts_with("//") {
            return Err("Microsoft returned an unsafe sign-in callback URL.".into());
        }

        let body = "<!doctype html><html><head><meta charset=\"utf-8\"><title>NOVA Client</title></head><body style=\"font-family:Segoe UI,sans-serif;background:#10131c;color:#f5f7fb;display:grid;place-items:center;min-height:90vh\"><main><h2>Microsoft sign-in complete</h2><p>You can close this tab and return to NOVA Client.</p></main></body></html>";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
            body.len(), body
        );
        stream.write_all(response.as_bytes())
            .map_err(|e| format!("Could not finish the Microsoft sign-in callback: {e}"))?;
        Ok(format!("{redirect_uri}{target}"))
    }).await.map_err(|e| format!("Microsoft sign-in callback stopped unexpectedly: {e}"))??;

    // AuthorizeRequest validates the OAuth state and PKCE verifier before exchanging the code.
    let account = request.login(&auth, &redirect_url).await
        .map_err(|e| format!("Microsoft sign-in failed: {e}"))?;
    let account_json = serde_json::to_string(&account)
        .map_err(|e| format!("Could not prepare the signed-in account: {e}"))?;
    credential_entry(MINECRAFT_ACCOUNT_USER)?
        .set_password(&account_json)
        .map_err(|e| format!("Could not securely save the Minecraft account: {e}"))?;

    app.emit("nova-login-complete", account.name.clone())
        .map_err(|e| format!("Signed in, but could not update NOVA: {e}"))?;
    Ok(account.name)
}

#[tauri::command]
fn minecraft_account_name() -> Result<Option<String>, String> {
    let entry = credential_entry(MINECRAFT_ACCOUNT_USER)?;
    match entry.get_password() {
        Ok(value) => {
            let account: Account = serde_json::from_str(&value)
                .map_err(|e| format!("Saved Minecraft account is unreadable: {e}"))?;
            Ok(Some(account.name))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Could not read the saved Minecraft account: {e}")),
    }
}

#[tauri::command]
fn sign_out_minecraft() -> Result<(), String> {
    match credential_entry(MINECRAFT_ACCOUNT_USER)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("Could not remove the saved Minecraft account: {e}")),
    }
}

#[tauri::command]
async fn minecraft_versions() -> Result<Value, String> {
    let response = reqwest::Client::new()
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await
        .map_err(|e| format!("Could not fetch Minecraft version list: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Minecraft version service returned an error: {e}"))?;
    response.json::<Value>().await
        .map_err(|e| format!("Minecraft version list was unreadable: {e}"))
}

#[tauri::command]
async fn launch_minecraft(version: String, loader: Option<String>, loader_build: Option<String>, instance_name: Option<String>) -> Result<(), String> {
    let mut version = version.trim().to_owned();
    let mut loader = loader.unwrap_or_else(|| "vanilla".to_owned());
    let mut loader_build = loader_build.unwrap_or_else(|| "latest".to_owned());
    if let Some(instance) = instance_name.as_deref() {
        let folder = instance_folder(instance)?;
        let metadata: Value = serde_json::from_slice(&std::fs::read(folder.join("nova-instance.json"))
            .map_err(|e| format!("Could not read installed instance: {e}"))?)
            .map_err(|e| format!("Installed instance metadata is invalid: {e}"))?;
        version = metadata["minecraftVersion"].as_str().ok_or("Installed instance has no Minecraft version.")?.to_owned();
        loader = metadata["loader"].as_str().unwrap_or("vanilla").to_owned();
        loader_build = metadata["loaderVersion"].as_str().unwrap_or("latest").to_owned();
    }
    if version.is_empty() || version.eq_ignore_ascii_case("null") || version.eq_ignore_ascii_case("undefined") {
        return Err("No Minecraft version was selected. Choose a version and try again.".into());
    }
    let manifest = reqwest::Client::new()
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send().await
        .map_err(|e| format!("Could not verify Minecraft version {version}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Could not verify Minecraft version {version}: {e}"))?
        .json::<Value>().await
        .map_err(|e| format!("Could not read Minecraft version manifest: {e}"))?;
    let known_version = manifest["versions"].as_array()
        .map(|items| items.iter().any(|item| item["id"].as_str() == Some(version.as_str())))
        .unwrap_or(false);
    if !known_version {
        return Err(format!("Minecraft version '{version}' was not found in Mojang's official version list. Refresh the version list and try again."));
    }

    let account_json = credential_entry(MINECRAFT_ACCOUNT_USER)?
        .get_password()
        .map_err(|_| "Sign in with your Microsoft account before launching Minecraft.".to_string())?;
    let saved_account: Account = serde_json::from_str(&account_json)
        .map_err(|e| format!("Saved Minecraft account is unreadable: {e}"))?;
    let auth = Authenticator::new()
        .map_err(|e| format!("Could not prepare Microsoft account refresh: {e}"))?;
    let account = auth.refresh(&saved_account).await
        .map_err(|e| format!("Microsoft session expired or could not be refreshed. Sign in again: {e}"))?;
    let refreshed_json = serde_json::to_string(&account)
        .map_err(|e| format!("Could not update the saved account: {e}"))?;
    credential_entry(MINECRAFT_ACCOUNT_USER)?
        .set_password(&refreshed_json)
        .map_err(|e| format!("Could not update the secure account store: {e}"))?;

    let local_app_data = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Windows LOCALAPPDATA folder could not be found.".to_string())?;
    let game_root = std::path::PathBuf::from(local_app_data)
        .join("NOVA Client")
        .join("minecraft");
    std::fs::create_dir_all(&game_root)
        .map_err(|e| format!("Could not create the Minecraft game folder: {e}"))?;

    let mut options = LaunchOptions::new(game_root, version);
    if let Some(name) = instance_name.as_deref() {
        options.instance = Some(safe_instance_name(name)?);
    }
    match loader.as_str() {
        "vanilla" => {}
        "fabric" => options.loader.kind = Some(LoaderKind::Fabric),
        "forge" => options.loader.kind = Some(LoaderKind::Forge),
        "neoforge" => options.loader.kind = Some(LoaderKind::NeoForge),
        "quilt" => options.loader.kind = Some(LoaderKind::Quilt),
        "legacyfabric" => options.loader.kind = Some(LoaderKind::LegacyFabric),
        other => return Err(format!("Unsupported mod loader '{other}'.")),
    }
    if loader != "vanilla" {
        options.loader.build = loader_build;
        options.loader.enable = true;
    }
    options.memory.max = "4G".to_owned();
    options.download_concurrency = 8;

    let launch = Launch::new(options, account)
        .map_err(|e| format!("Could not prepare Minecraft: {e}"))?;
    let mut child = launch.run().await
        .map_err(|e| format!("Minecraft could not be installed or started: {e}"))?;

    // Keep the child process supervised without blocking the launcher window.
    tokio::spawn(async move {
        let _ = child.wait().await;
    });
    Ok(())
}


fn safe_instance_name(input: &str) -> Result<String, String> {
    let value: String = input.trim().chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(48).collect();
    if value.is_empty() {
        return Err("Give this modpack a name containing letters or numbers.".into());
    }
    Ok(value)
}

fn safe_relative_path(input: &str) -> Result<PathBuf, String> {
    let normalized = input.replace('\\', "/");
    let path = Path::new(&normalized);
    if path.is_absolute() || path.components().any(|c| !matches!(c, Component::Normal(_) | Component::CurDir)) {
        return Err(format!("The pack contains an unsafe file path: {input}"));
    }
    Ok(path.to_path_buf())
}

fn instance_folder(instance_name: &str) -> Result<PathBuf, String> {
    let local = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Windows LOCALAPPDATA folder could not be found.".to_string())?;
    Ok(PathBuf::from(local).join("NOVA Client").join("minecraft").join("instances").join(safe_instance_name(instance_name)?))
}

fn extract_pack_overrides(bytes: &[u8], prefix: &str, destination: &Path) -> Result<(), String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("The modpack archive is invalid: {e}"))?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)
            .map_err(|e| format!("Could not read modpack archive entry: {e}"))?;
        let entry_name = entry.name().replace('\\', "/");
        let Some(relative) = entry_name.strip_prefix(prefix) else { continue };
        if relative.is_empty() { continue; }
        let relative_path = safe_relative_path(relative)?;
        let output = destination.join(relative_path);
        if entry.is_dir() {
            std::fs::create_dir_all(&output)
                .map_err(|e| format!("Could not create pack folder: {e}"))?;
        } else {
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Could not create pack folder: {e}"))?;
            }
            let mut file_bytes = Vec::new();
            entry.read_to_end(&mut file_bytes)
                .map_err(|e| format!("Could not extract pack override: {e}"))?;
            std::fs::write(&output, file_bytes)
                .map_err(|e| format!("Could not write pack override: {e}"))?;
        }
    }
    Ok(())
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    if !url.starts_with("https://") {
        return Err("The pack supplied a non-HTTPS download URL; it was blocked.".into());
    }
    reqwest::Client::new().get(url).send().await
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status().map_err(|e| format!("Download server returned an error: {e}"))?
        .bytes().await.map(|b| b.to_vec())
        .map_err(|e| format!("Could not read downloaded file: {e}"))
}

async fn download_pack_file(url: &str, destination: &Path, expected_size: Option<u64>, expected_sha1: Option<&str>) -> Result<(), String> {
    let bytes = fetch_bytes(url).await?;
    if let Some(size) = expected_size {
        if bytes.len() as u64 != size {
            return Err(format!("A mod file had an unexpected size (expected {size}, received {}).", bytes.len()));
        }
    }
    if let Some(expected) = expected_sha1.filter(|s| !s.is_empty()) {
        let digest = Sha1::digest(&bytes);
        let actual = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
        if !actual.eq_ignore_ascii_case(expected) {
            return Err("A mod file failed its SHA-1 integrity check.".into());
        }
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Could not create mod folder: {e}"))?;
    }
    std::fs::write(destination, bytes).map_err(|e| format!("Could not save a downloaded mod: {e}"))
}

fn loader_from_id(id: &str) -> Result<(String, String), String> {
    let (kind, build) = id.split_once('-')
        .ok_or_else(|| format!("The modpack has an unsupported loader identifier: {id}"))?;
    let loader = match kind.to_ascii_lowercase().as_str() {
        "fabric" => "fabric",
        "forge" => "forge",
        "neoforge" => "neoforge",
        "quilt" => "quilt",
        "legacyfabric" => "legacyfabric",
        other => return Err(format!("NOVA does not support the '{other}' mod loader in this pack.")),
    };
    if build.trim().is_empty() { return Err(format!("The modpack has no loader build in '{id}'.")); }
    Ok((loader.to_owned(), build.to_owned()))
}

fn save_instance_metadata(folder: &Path, metadata: Value) -> Result<Value, String> {
    std::fs::create_dir_all(folder).map_err(|e| format!("Could not create instance folder: {e}"))?;
    std::fs::write(folder.join("nova-instance.json"), serde_json::to_vec_pretty(&metadata)
        .map_err(|e| format!("Could not prepare instance metadata: {e}"))?)
        .map_err(|e| format!("Could not save instance metadata: {e}"))?;
    Ok(metadata)
}

#[tauri::command]
async fn install_modrinth_modpack(project_id: String, instance_name: String, version_id: Option<String>) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let client = reqwest::Client::new();
    let versions: Value = client.get(format!("https://api.modrinth.com/v2/project/{project_id}/version"))
        .send().await.map_err(|e| format!("Could not reach Modrinth: {e}"))?
        .error_for_status().map_err(|e| format!("Modrinth returned an error: {e}"))?
        .json().await.map_err(|e| format!("Modrinth returned an unreadable version list: {e}"))?;
    let version_list = versions.as_array().ok_or("Modrinth returned an invalid version list.")?;
    let latest = if let Some(requested) = version_id.as_deref() { version_list.iter().find(|v| v["id"].as_str() == Some(requested)).ok_or("The selected Modrinth modpack version is no longer available.")? } else { version_list.first().ok_or_else(|| "Modrinth has no downloadable versions for this modpack.".to_string())? };
    let version_id = latest["id"].as_str().ok_or("Modrinth version ID is missing.")?;
    let file = latest["files"].as_array().and_then(|a| a.iter().find(|f| f["primary"].as_bool() == Some(true)).or_else(|| a.first()))
        .ok_or("Modrinth did not provide a modpack archive.")?;
    let download_url = file["url"].as_str().ok_or("Modrinth download URL is missing.")?;
    let archive_bytes = fetch_bytes(download_url).await?;
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes.clone()))
        .map_err(|e| format!("The Modrinth pack is not a valid ZIP archive: {e}"))?;
    let mut index_bytes = Vec::new();
    archive.by_name("modrinth.index.json")
        .map_err(|_| "This Modrinth archive is missing modrinth.index.json.".to_string())?
        .read_to_end(&mut index_bytes).map_err(|e| format!("Could not read Modrinth manifest: {e}"))?;
    let index: Value = serde_json::from_slice(&index_bytes)
        .map_err(|e| format!("Modrinth manifest is invalid: {e}"))?;
    if index["formatVersion"].as_u64() != Some(1) {
        return Err("This Modrinth modpack format is not supported.".into());
    }
    let mc_version = index["dependencies"]["minecraft"]["version"].as_str()
        .ok_or("Modrinth pack does not specify a Minecraft version.")?.to_owned();
    let loader_entry = ["fabric-loader", "forge", "neoforge", "quilt-loader", "legacy-fabric"]
        .iter().find_map(|key| index["dependencies"][*key].as_str().map(|v| (*key, v)));
    let (loader, loader_build) = if let Some((key, build)) = loader_entry {
        (match key { "fabric-loader"=>"fabric", "quilt-loader"=>"quilt", "legacy-fabric"=>"legacyfabric", other=>other }.to_owned(), build.to_owned())
    } else { ("vanilla".to_owned(), String::new()) };
    let folder = instance_folder(&name)?;
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create instance folder: {e}"))?;
    for (i, item) in index["files"].as_array().ok_or("Modrinth manifest has no files list.")?.iter().enumerate() {
        if item["env"]["client"].as_str() == Some("unsupported") { continue; }
        let relative = safe_relative_path(item["path"].as_str().ok_or("A Modrinth file path is missing.")?)?;
        let urls = item["downloads"].as_array().ok_or("A Modrinth file has no download URLs.")?;
        let url = urls.first().and_then(Value::as_str).ok_or("A Modrinth file has no usable download URL.")?;
        let size = item["fileSize"].as_u64();
        let sha1 = item["hashes"]["sha1"].as_str();
        download_pack_file(url, &folder.join(relative), size, sha1).await
            .map_err(|e| format!("Mod file {} of {}: {e}", i + 1, index["files"].as_array().map(|a|a.len()).unwrap_or(0)))?;
    }
    extract_pack_overrides(&archive_bytes, "overrides/", &folder)?;
    extract_pack_overrides(&archive_bytes, "client-overrides/", &folder)?;
    let title = index["name"].as_str().filter(|s| !s.trim().is_empty()).unwrap_or(&name).to_owned();
    save_instance_metadata(&folder, json!({
        "instanceName":name,"name":title,"minecraftVersion":mc_version,"loader":loader,
        "loaderVersion":loader_build,"source":"modrinth","projectId":project_id,"packVersionId":version_id
    }))
}


#[tauri::command]
async fn curseforge_modpack_versions(mod_id: String) -> Result<Value, String> {
    let api_key = curseforge_api_key()?;
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&api_key).map_err(|_| "The saved CurseForge API key is invalid.")?);
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    let response = reqwest::Client::new()
        .get(format!("{CURSEFORGE_API}/mods/{mod_id}/files"))
        .headers(headers).query(&[("pageSize","50"),("index","0")])
        .send().await.map_err(|e| format!("Could not reach CurseForge: {e}"))?
        .error_for_status().map_err(|e| format!("CurseForge could not list modpack versions: {e}"))?
        .json::<Value>().await.map_err(|e| format!("CurseForge returned an unreadable version list: {e}"))?;
    Ok(response["data"].as_array().cloned().unwrap_or_default().into())
}

#[tauri::command]
async fn install_curseforge_modpack(mod_id: String, instance_name: String, file_id: Option<u64>) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let api_key = curseforge_api_key()?;
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&api_key).map_err(|_| "The saved CurseForge API key is invalid.")?);
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    let files: Value = client.get(format!("{CURSEFORGE_API}/mods/{mod_id}/files"))
        .headers(headers.clone()).query(&[("pageSize","50"),("index","0")])
        .send().await.map_err(|e| format!("Could not reach CurseForge: {e}"))?
        .error_for_status().map_err(|e| format!("CurseForge could not list pack files: {e}"))?
        .json().await.map_err(|e| format!("CurseForge returned an unreadable file list: {e}"))?;
    let file_list = files["data"].as_array().ok_or("CurseForge returned an invalid file list.")?;
    let latest = if let Some(requested) = file_id { file_list.iter().find(|f| f["id"].as_u64() == Some(requested)).ok_or("The selected CurseForge modpack version is no longer available.")? } else { file_list.first().ok_or("CurseForge has no downloadable files for this modpack.")? };
    let file_id = latest["id"].as_u64().ok_or("CurseForge file ID is missing.")?;
    let manifest_url: String = client.get(format!("{CURSEFORGE_API}/mods/{mod_id}/files/{file_id}/download-url"))
        .headers(headers.clone()).send().await.map_err(|e| format!("Could not request the CurseForge pack download: {e}"))?
        .error_for_status().map_err(|e| format!("CurseForge did not provide a download URL: {e}"))?
        .json::<Value>().await.map_err(|e| format!("CurseForge download response was unreadable: {e}"))?["data"]
        .as_str().ok_or("CurseForge did not provide a downloadable archive URL for this file.")?.to_owned();
    let archive_bytes = fetch_bytes(&manifest_url).await?;
    let mut archive = ZipArchive::new(Cursor::new(archive_bytes.clone()))
        .map_err(|e| format!("The CurseForge pack is not a valid ZIP archive: {e}"))?;
    let mut manifest_bytes = Vec::new();
    archive.by_name("manifest.json").map_err(|_| "This CurseForge archive is missing manifest.json.".to_string())?
        .read_to_end(&mut manifest_bytes).map_err(|e| format!("Could not read CurseForge manifest: {e}"))?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("CurseForge manifest is invalid: {e}"))?;
    if manifest["manifestType"].as_str() != Some("minecraftModpack") {
        return Err("The downloaded CurseForge file is not a Minecraft modpack.".into());
    }
    let mc_version = manifest["minecraft"]["version"].as_str()
        .ok_or("CurseForge pack does not specify a Minecraft version.")?.to_owned();
    let loader_entry = manifest["minecraft"]["modLoaders"].as_array()
        .and_then(|a| a.iter().find(|v| v["primary"].as_bool() == Some(true)).or_else(|| a.first()))
        .and_then(|v| v["id"].as_str());
    let (loader, loader_build) = if let Some(id) = loader_entry { loader_from_id(id)? } else { ("vanilla".into(), String::new()) };
    let folder = instance_folder(&name)?;
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create instance folder: {e}"))?;
    let pack_files = manifest["files"].as_array().ok_or("CurseForge manifest has no files list.")?;
    for (i, item) in pack_files.iter().enumerate() {
        if item["required"].as_bool() == Some(false) { continue; }
        let project = item["projectID"].as_u64().ok_or("CurseForge project ID is missing.")?;
        let file = item["fileID"].as_u64().ok_or("CurseForge file ID is missing.")?;
        let url: String = client.get(format!("{CURSEFORGE_API}/mods/{project}/files/{file}/download-url"))
            .headers(headers.clone()).send().await.map_err(|e| format!("Could not resolve CurseForge mod {}: {e}", i+1))?
            .error_for_status().map_err(|e| format!("CurseForge could not provide mod {}: {e}", i+1))?
            .json::<Value>().await.map_err(|e| format!("CurseForge mod {} response was unreadable: {e}", i+1))?["data"]
            .as_str().ok_or_else(|| format!("CurseForge did not provide a download URL for required mod {project}/{file}. The author may have disabled third-party downloads."))?.to_owned();
        let bytes = fetch_bytes(&url).await?;
        let filename = format!("{project}-{file}.jar");
        std::fs::create_dir_all(folder.join("mods")).map_err(|e| format!("Could not create mods folder: {e}"))?;
        std::fs::write(folder.join("mods").join(filename), bytes)
            .map_err(|e| format!("Could not save CurseForge mod {}: {e}", i+1))?;
    }
    extract_pack_overrides(&archive_bytes, "overrides/", &folder)?;
    let title = manifest["name"].as_str().filter(|s| !s.trim().is_empty()).unwrap_or(&name).to_owned();
    save_instance_metadata(&folder, json!({
        "instanceName":name,"name":title,"minecraftVersion":mc_version,"loader":loader,
        "loaderVersion":loader_build,"source":"curseforge","projectId":mod_id,"packFileId":file_id
    }))
}


#[tauri::command]
fn create_instance(instance_name: String, display_name: String, version: String, loader: String) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let version = version.trim();
    if version.is_empty() || version.eq_ignore_ascii_case("null") || version.eq_ignore_ascii_case("undefined") {
        return Err("Choose a valid Minecraft version.".into());
    }
    let loader = loader.to_ascii_lowercase();
    if !matches!(loader.as_str(), "vanilla" | "fabric" | "forge" | "neoforge" | "quilt" | "legacyfabric") {
        return Err(format!("Unsupported mod loader '{loader}'."));
    }
    let folder = instance_folder(&name)?;
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create instance folder: {e}"))?;
    let loader_version = if loader == "vanilla" { String::new() } else { "latest".to_owned() };
    let metadata = json!({
        "instanceName": name,
        "name": display_name.trim(),
        "minecraftVersion": version,
        "loader": loader,
        "loaderVersion": loader_version,
        "source": "custom"
    });
    save_instance_metadata(&folder, metadata)
}

#[tauri::command]
fn delete_instance(instance_name: String) -> Result<(), String> {
    let folder = instance_folder(&instance_name)?;
    if folder.exists() {
        std::fs::remove_dir_all(&folder)
            .map_err(|e| format!("Could not delete instance files: {e}"))?;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            curseforge_search,
            begin_microsoft_login,
            minecraft_account_name,
            sign_out_minecraft,
            minecraft_versions,
            launch_minecraft,
            install_modrinth_modpack,
            curseforge_modpack_versions,
            install_curseforge_modpack,
            create_instance,
            delete_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running NOVA Client");
}
