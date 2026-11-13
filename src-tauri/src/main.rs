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
use std::sync::{Mutex, OnceLock, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use zip::ZipArchive;
use tauri::Emitter;

const KEYRING_SERVICE: &str = "NOVA Client";
const MINECRAFT_ACCOUNT_USER: &str = "minecraft-account";
const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const MICROSOFT_CLIENT_ID: &str = "09e6a2a2-90ee-47a6-92c4-f8810958de91";

static AUTH_REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn refresh_saved_minecraft_account(saved_account: &Account) -> Result<Account, String> {
    let _refresh_guard = AUTH_REFRESH_LOCK.lock().await;
    let auth = Authenticator::new()
        .map_err(|e| format!("Could not prepare Microsoft account refresh: {e}"))?;
    let account = auth.refresh(saved_account).await
        .map_err(|e| format!("Microsoft session expired or could not be refreshed. Sign in again: {e}"))?;
    let refreshed_json = serde_json::to_string(&account)
        .map_err(|e| format!("Could not update the saved account: {e}"))?;
    credential_entry(MINECRAFT_ACCOUNT_USER)?
        .set_password(&refreshed_json)
        .map_err(|e| format!("Could not update the secure account store: {e}"))?;
    Ok(account)
}

const MAX_ARCHIVE_ENTRIES: usize = 20_000;
const MAX_ARCHIVE_FILE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_ARCHIVE_TOTAL_BYTES: u64 = 2 * 1024 * 1024 * 1024;
static ACTIVE_LAUNCHES: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(12))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Could not initialize secure network client: {e}"))
}

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
    let client = http_client()?;
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
    // Bind through the same hostname used by the registered OAuth redirect.
    let listener = TcpListener::bind(("localhost", 0))
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
        listener.set_nonblocking(true)
            .map_err(|e| format!("Could not configure the Microsoft sign-in callback: {e}"))?;
        let deadline = Instant::now() + Duration::from_secs(180);
        let (stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err("Microsoft sign-in timed out. Start sign-in again and finish it in your browser.".into());
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(error) => return Err(format!("Could not receive Microsoft's sign-in response: {error}")),
            }
        };
        let mut stream = stream;
        stream.set_read_timeout(Some(Duration::from_secs(8)))
            .map_err(|e| format!("Could not set Microsoft callback timeout: {e}"))?;
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

        let body = "<!doctype html><html><head><meta charset=\"utf-8\"><title>NOVA Client</title></head><body style=\"font-family:Segoe UI,sans-serif;background:#10131c;color:#f5f7fb;display:grid;place-items:center;min-height:90vh\"><main><h2>Sign-in response received</h2><p>Return to NOVA Client while your account is being verified.</p></main></body></html>";
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
    let response = http_client()?
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
async fn prepare_modpack_instance(app: tauri::AppHandle, instance_name: String) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let folder = instance_folder(&name)?;
    let metadata: Value = serde_json::from_slice(
        &std::fs::read(folder.join("nova-instance.json"))
            .map_err(|e| format!("Could not read modpack instance: {e}"))?
    ).map_err(|e| format!("Modpack instance metadata is invalid: {e}"))?;

    let version = metadata["minecraftVersion"].as_str()
        .ok_or("The modpack has no Minecraft version.")?.to_owned();
    let loader = metadata["loader"].as_str().unwrap_or("vanilla").to_ascii_lowercase();
    let loader_build = metadata["loaderVersion"].as_str().unwrap_or("latest").to_owned();

    emit_pack_progress(&app, "preparing", 0, 0, "Preparing Minecraft, Java and mod loader");
    // Preparing a modpack installs Minecraft, Java, libraries, assets and its loader
    // into the selected instance without starting the game.
    let account_json = credential_entry(MINECRAFT_ACCOUNT_USER)?
        .get_password()
        .map_err(|_| "Sign in with Microsoft before completing the modpack installation.".to_string())?;
    let saved_account: Account = serde_json::from_str(&account_json)
        .map_err(|e| format!("Saved Minecraft account is unreadable: {e}"))?;
    let account = refresh_saved_minecraft_account(&saved_account).await?;

    let local_app_data = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Windows LOCALAPPDATA folder could not be found.".to_string())?;
    let game_root = PathBuf::from(local_app_data)
        .join("NOVA Client").join("minecraft");
    std::fs::create_dir_all(&game_root)
        .map_err(|e| format!("Could not create the Minecraft game folder: {e}"))?;

    let mut options = LaunchOptions::new(game_root, version.clone());
    options.instance = Some(name.clone());
    match loader.as_str() {
        "vanilla" => {}
        "fabric" => options.loader.kind = Some(LoaderKind::Fabric),
        "forge" => options.loader.kind = Some(LoaderKind::Forge),
        "neoforge" => options.loader.kind = Some(LoaderKind::NeoForge),
        "quilt" => options.loader.kind = Some(LoaderKind::Quilt),
        "legacyfabric" => options.loader.kind = Some(LoaderKind::LegacyFabric),
        other => return Err(format!("Unsupported mod loader '{other}' in this modpack.")),
    }
    if loader != "vanilla" {
        options.loader.build = loader_build.clone();
        options.loader.enable = true;
    }
    // Modpack preparation installs files only; RAM allocation is applied when the
    // user launches the instance.
    options.download_concurrency = 8;

    let launch = Launch::new(options, account)
        .map_err(|e| format!("Could not prepare modpack installation: {e}"))?;
    launch.prepare().await
        .map_err(|e| format!("Minecraft or mod-loader installation failed: {e}"))?;
    emit_pack_progress(&app, "ready", 1, 1, "Minecraft and mod loader are ready");

    Ok(json!({
        "ready": true,
        "instanceName": name,
        "minecraftVersion": version,
        "loader": loader,
        "loaderVersion": loader_build
    }))
}

#[tauri::command]
async fn launch_minecraft(app: tauri::AppHandle, version: String, loader: Option<String>, loader_build: Option<String>, instance_name: Option<String>, memory_mb: Option<u32>) -> Result<(), String> {
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
    let manifest = http_client()?
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
    let account = refresh_saved_minecraft_account(&saved_account).await?;

    let local_app_data = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Windows LOCALAPPDATA folder could not be found.".to_string())?;
    let game_root = std::path::PathBuf::from(local_app_data)
        .join("NOVA Client")
        .join("minecraft");
    std::fs::create_dir_all(&game_root)
        .map_err(|e| format!("Could not create the Minecraft game folder: {e}"))?;

    let mut options = LaunchOptions::new(game_root, version.clone());
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
    let memory_mb = memory_mb.unwrap_or(4096);
    if !(1024..=16384).contains(&memory_mb) {
        return Err("Memory allocation must be between 1024 MB and 16384 MB.".into());
    }
    options.memory.max = format!("{memory_mb}M");
    options.download_concurrency = 8;

    let launch = Launch::new(options, account)
        .map_err(|e| format!("Could not prepare Minecraft: {e}"))?;
    let launch_key = instance_name.as_deref()
        .map(safe_instance_name).transpose()?
        .unwrap_or_else(|| format!("default-{version}"));
    let active = ACTIVE_LAUNCHES.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
    {
        let mut running = active.lock().map_err(|_| "Could not inspect active Minecraft launches.")?;
        if !running.insert(launch_key.clone()) {
            return Err("This Minecraft instance is already running.".into());
        }
    }
    let mut child = match launch.run().await {
        Ok(child) => child,
        Err(error) => {
            if let Ok(mut running) = active.lock() { running.remove(&launch_key); }
            return Err(format!("Minecraft could not be installed or started: {error}"));
        }
    };

    // Supervise the child and release the instance lock when the game exits.
    tokio::spawn(async move {
        let exit = child.wait().await;
        if let Ok(mut running) = active.lock() { running.remove(&launch_key); }
        let (success, code) = match exit {
            Ok(status) => (status.success(), status.code()),
            Err(_) => (false, None),
        };
        let _ = app.emit("nova-game-exit", json!({"success":success,"code":code}));
    });
    Ok(())
}


fn emit_pack_progress(app: &tauri::AppHandle, phase: &str, completed: u64, total: u64, detail: &str) {
    let percent = if total > 0 { Some(((completed as f64 / total as f64) * 90.0).round() as u32) } else { None };
    let _ = app.emit("nova-pack-progress", json!({"phase":phase,"completed":completed,"total":total,"percent":percent,"detail":detail}));
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
    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err("The modpack archive contains too many files and was blocked for safety.".into());
    }
    let mut total_uncompressed = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)
            .map_err(|e| format!("Could not read modpack archive entry: {e}"))?;
        let entry_size = entry.size();
        if entry_size > MAX_ARCHIVE_FILE_BYTES {
            return Err("The modpack archive contains a file larger than 512 MiB and was blocked.".into());
        }
        total_uncompressed = total_uncompressed.saturating_add(entry_size);
        if total_uncompressed > MAX_ARCHIVE_TOTAL_BYTES {
            return Err("The modpack archive expands beyond the 2 GiB safety limit.".into());
        }
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
            atomic_write(&output, &file_bytes)
                .map_err(|e| format!("Could not write pack override: {e}"))?;
        }
    }
    Ok(())
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    if !url.starts_with("https://") {
        return Err("The pack supplied a non-HTTPS download URL; it was blocked.".into());
    }
    let response = http_client()?.get(url).send().await
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status().map_err(|e| format!("Download server returned an error: {e}"))?;
    if response.content_length().is_some_and(|size| size > MAX_ARCHIVE_TOTAL_BYTES) {
        return Err("The downloaded archive exceeds NOVA's 2 GiB safety limit.".into());
    }
    let bytes = response.bytes().await.map_err(|e| format!("Could not read downloaded file: {e}"))?;
    if bytes.len() as u64 > MAX_ARCHIVE_TOTAL_BYTES {
        return Err("The downloaded archive exceeds NOVA's 2 GiB safety limit.".into());
    }
    Ok(bytes.to_vec())
}

fn atomic_write(destination: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = destination.parent().ok_or("Download destination has no parent folder.")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("Could not create download folder: {e}"))?;
    let file_name = destination.file_name().and_then(|n| n.to_str()).unwrap_or("download");
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".{file_name}.{}.{sequence}.partial", std::process::id()));
    std::fs::write(&temporary, bytes).map_err(|e| format!("Could not write temporary download: {e}"))?;
    if let Err(error) = std::fs::rename(&temporary, destination) {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("Could not finalize downloaded file: {error}"));
    }
    Ok(())
}

async fn download_pack_file(url: &str, destination: &Path, expected_size: Option<u64>, expected_sha1: Option<&str>) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("The pack supplied a non-HTTPS download URL; it was blocked.".into());
    }
    if expected_size.is_some_and(|size| size > MAX_ARCHIVE_FILE_BYTES) {
        return Err("A mod file exceeds NOVA's 512 MiB per-file safety limit.".into());
    }
    let response = http_client()?.get(url).send().await
        .map_err(|e| format!("Mod download failed: {e}"))?
        .error_for_status().map_err(|e| format!("Mod download server returned an error: {e}"))?;
    if response.content_length().is_some_and(|size| size > MAX_ARCHIVE_FILE_BYTES) {
        return Err("A mod file exceeds NOVA's 512 MiB per-file safety limit.".into());
    }
    let parent = destination.parent().ok_or("Download destination has no parent folder.")?;
    tokio::fs::create_dir_all(parent).await
        .map_err(|e| format!("Could not create download folder: {e}"))?;
    let file_name = destination.file_name().and_then(|n| n.to_str()).unwrap_or("download");
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".{file_name}.{}.{sequence}.partial", std::process::id()));
    let result: Result<(), String> = async {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(&temporary).await
            .map_err(|e| format!("Could not create temporary mod download: {e}"))?;
        let mut response = response;
        let mut hasher = Sha1::new();
        let mut downloaded = 0u64;
        while let Some(chunk) = response.chunk().await
            .map_err(|e| format!("Could not read mod download: {e}"))? {
            downloaded = downloaded.checked_add(chunk.len() as u64)
                .ok_or("Mod download size overflowed.")?;
            if downloaded > MAX_ARCHIVE_FILE_BYTES {
                return Err("A mod file exceeds NOVA's 512 MiB per-file safety limit.".into());
            }
            if expected_size.is_some_and(|size| downloaded > size) {
                return Err("A mod file exceeded the size declared by its manifest.".into());
            }
            hasher.update(&chunk);
            file.write_all(&chunk).await
                .map_err(|e| format!("Could not write temporary mod download: {e}"))?;
        }
        file.flush().await.map_err(|e| format!("Could not flush mod download: {e}"))?;
        drop(file);
        if let Some(size) = expected_size {
            if downloaded != size {
                return Err(format!("A mod file had an unexpected size (expected {size}, received {downloaded})."));
            }
        }
        if let Some(expected) = expected_sha1.filter(|s| !s.is_empty()) {
            let digest = hasher.finalize();
            let actual = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
            if !actual.eq_ignore_ascii_case(expected) {
                return Err("A mod file failed its SHA-1 integrity check.".into());
            }
        }
        std::fs::rename(&temporary, destination)
            .map_err(|e| format!("Could not finalize downloaded mod: {e}"))?;
        Ok(())
    }.await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
    }
    result
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
    let serialized = serde_json::to_vec_pretty(&metadata)
        .map_err(|e| format!("Could not prepare instance metadata: {e}"))?;
    atomic_write(&folder.join("nova-instance.json"), &serialized)
        .map_err(|e| format!("Could not save instance metadata: {e}"))?;
    Ok(metadata)
}

#[tauri::command]
async fn install_modrinth_modpack(app: tauri::AppHandle, project_id: String, instance_name: String, version_id: Option<String>) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let client = http_client()?;
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
    emit_pack_progress(&app, "archive", 0, 0, "Downloading Modrinth archive");
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
    let pack_files = index["files"].as_array().ok_or("Modrinth manifest has no files list.")?;
    let total_files = pack_files.iter().filter(|item| item["env"]["client"].as_str() != Some("unsupported")).count() as u64;
    let mut completed_files = 0u64;
    emit_pack_progress(&app, "files", 0, total_files, "Downloading Modrinth files");
    for (i, item) in pack_files.iter().enumerate() {
        if item["env"]["client"].as_str() == Some("unsupported") { continue; }
        let relative = safe_relative_path(item["path"].as_str().ok_or("A Modrinth file path is missing.")?)?;
        let urls = item["downloads"].as_array().ok_or("A Modrinth file has no download URLs.")?;
        if urls.is_empty() { return Err("A Modrinth file has no download URLs.".into()); }
        let size = item["fileSize"].as_u64();
        let sha1 = item["hashes"]["sha1"].as_str();
        let mut last_error = None;
        let mut downloaded = false;
        for candidate in urls.iter().filter_map(Value::as_str) {
            match download_pack_file(candidate, &folder.join(relative.clone()), size, sha1).await {
                Ok(()) => { downloaded = true; break; }
                Err(error) => last_error = Some(error),
            }
        }
        if !downloaded {
            return Err(format!("Mod file {} of {} failed from all available mirrors: {}", i + 1, pack_files.len(), last_error.unwrap_or_else(|| "No usable HTTPS download URL.".into())));
        }
        completed_files += 1;
        emit_pack_progress(&app, "files", completed_files, total_files, &format!("Downloaded file {completed_files} of {total_files}"));
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
    let client = http_client()?;
    let mut all_files = Vec::new();
    // CurseForge paginates file history. Fetch every page in bounded batches so
    // older releases are available in NOVA's version picker as well.
    for page in 0..20u32 {
        let response = client.get(format!("{CURSEFORGE_API}/mods/{mod_id}/files"))
            .headers(headers.clone())
            .query(&[("pageSize","50"),("index",&(page * 50).to_string())])
            .send().await.map_err(|e| format!("Could not reach CurseForge: {e}"))?
            .error_for_status().map_err(|e| format!("CurseForge could not list modpack versions: {e}"))?
            .json::<Value>().await.map_err(|e| format!("CurseForge returned an unreadable version list: {e}"))?;
        let files = response["data"].as_array().ok_or("CurseForge returned an invalid modpack version list.")?;
        let count = files.len();
        all_files.extend(files.iter().cloned());
        if count < 50 { break; }
    }
    Ok(Value::Array(all_files))
}

#[tauri::command]
async fn install_curseforge_modpack(app: tauri::AppHandle, mod_id: String, instance_name: String, file_id: Option<u64>) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let api_key = curseforge_api_key()?;
    let client = http_client()?;
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(&api_key).map_err(|_| "The saved CurseForge API key is invalid.")?);
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    // The version picker can show releases from up to 1,000 files. Search the
    // same paginated history here so selecting an older release actually works.
    let mut selected_file: Option<Value> = None;
    for page in 0..20u32 {
        let files: Value = client.get(format!("{CURSEFORGE_API}/mods/{mod_id}/files"))
            .headers(headers.clone())
            .query(&[("pageSize","50"),("index",&(page * 50).to_string())])
            .send().await.map_err(|e| format!("Could not reach CurseForge: {e}"))?
            .error_for_status().map_err(|e| format!("CurseForge could not list pack files: {e}"))?
            .json().await.map_err(|e| format!("CurseForge returned an unreadable file list: {e}"))?;
        let file_list = files["data"].as_array().ok_or("CurseForge returned an invalid file list.")?;
        if let Some(requested) = file_id {
            if let Some(found) = file_list.iter().find(|f| f["id"].as_u64() == Some(requested)) {
                selected_file = Some(found.clone());
                break;
            }
        } else if page == 0 {
            selected_file = file_list.first().cloned();
            break;
        }
        if file_list.len() < 50 { break; }
    }
    let selected_file = selected_file.ok_or_else(|| {
        if file_id.is_some() {
            "The selected CurseForge modpack version is no longer available in the version history.".to_string()
        } else {
            "CurseForge has no downloadable files for this modpack.".to_string()
        }
    })?;
    let file_id = selected_file["id"].as_u64().ok_or("CurseForge file ID is missing.")?;
    let manifest_url: String = client.get(format!("{CURSEFORGE_API}/mods/{mod_id}/files/{file_id}/download-url"))
        .headers(headers.clone()).send().await.map_err(|e| format!("Could not request the CurseForge pack download: {e}"))?
        .error_for_status().map_err(|e| format!("CurseForge did not provide a download URL: {e}"))?
        .json::<Value>().await.map_err(|e| format!("CurseForge download response was unreadable: {e}"))?["data"]
        .as_str().ok_or("CurseForge did not provide a downloadable archive URL for this file.")?.to_owned();
    emit_pack_progress(&app, "archive", 0, 0, "Downloading CurseForge archive");
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
    let total_files = pack_files.iter().filter(|item| item["required"].as_bool() != Some(false)).count() as u64;
    let mut completed_files = 0u64;
    emit_pack_progress(&app, "files", 0, total_files, "Downloading CurseForge files");
    for (i, item) in pack_files.iter().enumerate() {
        if item["required"].as_bool() == Some(false) { continue; }
        let project = item["projectID"].as_u64().ok_or("CurseForge project ID is missing.")?;
        let file = item["fileID"].as_u64().ok_or("CurseForge file ID is missing.")?;
        let url: String = client.get(format!("{CURSEFORGE_API}/mods/{project}/files/{file}/download-url"))
            .headers(headers.clone()).send().await.map_err(|e| format!("Could not resolve CurseForge mod {}: {e}", i+1))?
            .error_for_status().map_err(|e| format!("CurseForge could not provide mod {}: {e}", i+1))?
            .json::<Value>().await.map_err(|e| format!("CurseForge mod {} response was unreadable: {e}", i+1))?["data"]
            .as_str().ok_or_else(|| format!("CurseForge did not provide a download URL for required mod {project}/{file}. The author may have disabled third-party downloads."))?.to_owned();
        let filename = format!("{project}-{file}.jar");
        let mods_folder = folder.join("mods");
        std::fs::create_dir_all(&mods_folder).map_err(|e| format!("Could not create mods folder: {e}"))?;
        download_pack_file(&url, &mods_folder.join(filename), None, None).await
            .map_err(|e| format!("Could not save CurseForge mod {}: {e}", i+1))?;
        completed_files += 1;
        emit_pack_progress(&app, "files", completed_files, total_files, &format!("Downloaded file {completed_files} of {total_files}"));
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
    if let Some(parent) = folder.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Could not create instances folder: {e}"))?;
    }
    match std::fs::create_dir(&folder) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err("An instance with that name already exists. Choose a different name so existing game files are not overwritten.".into());
        }
        Err(error) => return Err(format!("Could not create instance folder: {error}")),
    }
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
fn list_instances() -> Result<Vec<Value>, String> {
    let local = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Windows LOCALAPPDATA folder could not be found.".to_string())?;
    let root = PathBuf::from(local).join("NOVA Client").join("minecraft").join("instances");
    if !root.exists() { return Ok(Vec::new()); }
    let entries = std::fs::read_dir(&root)
        .map_err(|e| format!("Could not scan NOVA instances: {e}"))?;
    let mut instances = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Could not read an instance folder: {e}"))?;
        if !entry.file_type().map_err(|e| format!("Could not inspect an instance folder: {e}"))?.is_dir() {
            continue;
        }
        let metadata_path = entry.path().join("nova-instance.json");
        if !metadata_path.is_file() { continue; }
        let bytes = match std::fs::read(&metadata_path) {
            Ok(bytes) => bytes,
            Err(error) => {
                eprintln!("Skipping unreadable NOVA instance metadata: {error}");
                continue;
            }
        };
        let metadata: Value = match serde_json::from_slice(&bytes) {
            Ok(metadata) => metadata,
            Err(error) => {
                eprintln!("Skipping invalid NOVA instance metadata in '{}': {error}", entry.file_name().to_string_lossy());
                continue;
            }
        };
        let Some(name) = metadata["instanceName"].as_str() else { continue; };
        let Some(version) = metadata["minecraftVersion"].as_str() else { continue; };
        let Ok(safe_name) = safe_instance_name(name) else { continue; };
        let folder_name = entry.file_name();
        if safe_name == name
            && folder_name.to_string_lossy().as_ref() == name
            && !version.trim().is_empty()
        {
            instances.push(metadata);
        }
    }
    Ok(instances)
}

#[tauri::command]
fn delete_instance(instance_name: String) -> Result<(), String> {
    let name = safe_instance_name(&instance_name)?;
    let active = ACTIVE_LAUNCHES.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
    if active.lock().map_err(|_| "Could not inspect running Minecraft processes.")?.contains(&name) {
        return Err("This instance is currently running. Close Minecraft before deleting it.".into());
    }
    let folder = instance_folder(&name)?;
    if folder.exists() {
        std::fs::remove_dir_all(&folder)
            .map_err(|e| format!("Could not delete instance files: {e}"))?;
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::{safe_instance_name, safe_relative_path};

    #[test]
    fn instance_names_strip_unsafe_characters() {
        assert_eq!(safe_instance_name(" My Pack! 01 ").unwrap(), "MyPack01");
    }

    #[test]
    fn instance_names_reject_empty_values() {
        assert!(safe_instance_name(" !!! ").is_err());
    }

    #[test]
    fn instance_names_are_limited_to_48_characters() {
        let input = "a".repeat(60);
        assert_eq!(safe_instance_name(&input).unwrap().len(), 48);
    }

    #[test]
    fn relative_paths_reject_parent_traversal() {
        assert!(safe_relative_path("mods/../../outside").is_err());
        assert!(safe_relative_path("../outside").is_err());
    }

    #[test]
    fn relative_paths_accept_normal_mod_paths() {
        assert!(safe_relative_path("mods/example.jar").is_ok());
    }
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
            prepare_modpack_instance,
            install_modrinth_modpack,
            curseforge_modpack_versions,
            install_curseforge_modpack,
            create_instance,
            list_instances,
            delete_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running NOVA Client");
}
