#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crust_core::authenticator::{Account, Authenticator};
use crust_core::foundation::events::Event;
use crust_core::launcher::{Launch, LaunchOptions, LoaderKind};
use keyring::Entry;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde_json::{json, Value};
use sha1::{Digest, Sha1};
use std::io::{BufRead, BufReader, Read, Write};
use std::fs::OpenOptions;
use std::net::TcpListener;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zip::ZipArchive;
use tauri::Emitter;

const KEYRING_SERVICE: &str = "NOVA Client";
const MINECRAFT_ACCOUNT_USER: &str = "minecraft-account";
const CURSEFORGE_API: &str = "https://api.curseforge.com/v1";
const MICROSOFT_CLIENT_ID: &str = "09e6a2a2-90ee-47a6-92c4-f8810958de91";

const NOVA_RELEASES_API: &str = "https://api.github.com/repos/painopro123456-lang/nova-client./releases/latest";
const NOVA_INSTALLER_ASSET: &str = "NOVA-Client-Setup.exe";
const MAX_UPDATER_DOWNLOAD_BYTES: u64 = 200 * 1024 * 1024;

fn logs_directory() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("NOVA Client").join("logs")
}

fn updater_log_path() -> PathBuf {
    logs_directory().join("updater.log")
}

fn game_log_path() -> PathBuf {
    logs_directory().join("game-launch.log")
}

fn game_log(message: &str) {
    let path = game_log_path();
    if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let safe_message = message.replace('\r', " ").replace('\n', " ");
        let _ = writeln!(file, "[unix:{timestamp}] {safe_message}");
    }
}

fn tail_file(path: &Path, max_lines: usize) -> String {
    match std::fs::read_to_string(path) {
        Ok(contents) => {
            let lines: Vec<&str> = contents.lines().collect();
            let start = lines.len().saturating_sub(max_lines);
            lines[start..].join("\n")
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => "No Minecraft log has been created yet. Launch the game once to generate logs/latest.log.".into(),
        Err(error) => format!("Could not read log file {}: {error}", path.display()),
    }
}

#[tauri::command]
fn get_game_logs(instance_name: Option<String>) -> Result<Value, String> {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Windows LOCALAPPDATA folder could not be found.".to_string())?;
    let root = PathBuf::from(local_app_data).join("NOVA Client").join("minecraft");
    let mut minecraft_sections = Vec::new();
    let mut minecraft_paths = Vec::new();
    if let Some(name) = instance_name.as_deref() {
        let game_dir = root.join("instances").join(safe_instance_name(name)?);
        let path = game_dir.join("logs").join("latest.log");
        minecraft_sections.push(format!("=== {} ===\\n{}", name, tail_file(&path, 500)));
        minecraft_paths.push(path.display().to_string());
    } else {
        let default_log = root.join("logs").join("latest.log");
        if default_log.exists() {
            minecraft_sections.push(format!("=== Default Minecraft ===\\n{}", tail_file(&default_log, 350)));
            minecraft_paths.push(default_log.display().to_string());
        }
        let instances_dir = root.join("instances");
        if let Ok(entries) = std::fs::read_dir(&instances_dir) {
            for entry in entries.flatten().take(30) {
                if !entry.file_type().is_ok_and(|kind| kind.is_dir()) { continue; }
                let path = entry.path().join("logs").join("latest.log");
                if path.is_file() {
                    minecraft_sections.push(format!("=== Instance {} ===\\n{}", entry.file_name().to_string_lossy(), tail_file(&path, 350)));
                    minecraft_paths.push(path.display().to_string());
                }
            }
        }
        if minecraft_sections.is_empty() {
            minecraft_sections.push("No Minecraft latest.log files have been created yet. Launch a profile once to generate its game log.".into());
        }
    }
    Ok(json!({
        "launcherLog": tail_file(&game_log_path(), 300),
        "minecraftLog": minecraft_sections.join("\\n\\n"),
        "launcherLogPath": game_log_path().display().to_string(),
        "minecraftLogPath": if minecraft_paths.is_empty() { "Minecraft logs\\\\latest.log".to_owned() } else { minecraft_paths.join(" | ") }
    }))
}

fn game_event_logger() -> Arc<dyn Fn(Event) + Send + Sync> {
    let last_progress = Mutex::new(Instant::now() - Duration::from_secs(2));
    Arc::new(move |event| match event {
        Event::Progress { downloaded, total, element } => {
            if let Ok(mut last) = last_progress.lock() {
                if last.elapsed() >= Duration::from_secs(2) || (total > 0 && downloaded >= total) {
                    game_log(&format!("Download: {element} ({downloaded}/{total} bytes)"));
                    *last = Instant::now();
                }
            }
        }
        Event::Check { checked, total, element } => {
            if checked == total || checked % 500 == 0 {
                game_log(&format!("Check: {element} ({checked}/{total})"));
            }
        }
        Event::Extract(message) => game_log(&format!("Extract: {message}")),
        Event::Patch(message) => game_log(&format!("Patch: {message}")),
        Event::Error(message) => game_log(&format!("Launcher error: {message}")),
        Event::Speed(_) | Event::Estimated(_) => {}
    })
}

fn updater_log(message: &str) {
    let path = updater_log_path();
    if let Some(parent) = path.parent() { let _ = std::fs::create_dir_all(parent); }
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let safe_message = message.replace('\r', " ").replace('\n', " ");
        let _ = writeln!(file, "[unix:{timestamp}] {safe_message}");
    }
}

#[tauri::command]
fn get_updater_logs() -> Result<String, String> {
    match std::fs::read_to_string(updater_log_path()) {
        Ok(contents) => {
            let lines: Vec<&str> = contents.lines().collect();
            let start = lines.len().saturating_sub(250);
            Ok(lines[start..].join("\n"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok("No updater activity has been logged yet.".into()),
        Err(error) => Err(format!("Could not read updater log: {error}")),
    }
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    let parse = |value: &str| -> Option<Vec<u64>> {
        value.trim_start_matches('v').split('.').map(|part| part.parse::<u64>().ok()).collect()
    };
    match (parse(latest), parse(current)) {
        (Some(latest), Some(current)) => {
            let length = latest.len().max(current.len());
            (0..length).map(|i| *latest.get(i).unwrap_or(&0))
                .cmp((0..length).map(|i| *current.get(i).unwrap_or(&0))).is_gt()
        }
        _ => false,
    }
}

async fn latest_nova_release() -> Result<Value, String> {
    let client = http_client()?;
    let response = client.get(NOVA_RELEASES_API)
        .header("User-Agent", "NOVA-Client-Updater")
        .header(ACCEPT, "application/vnd.github+json")
        .send().await.map_err(|e| format!("Could not check NOVA updates: {e}"))?
        .error_for_status().map_err(|e| format!("GitHub could not provide the latest NOVA release: {e}"))?;
    response.json::<Value>().await.map_err(|e| format!("GitHub returned invalid update information: {e}"))
}

fn release_installer_url(release: &Value) -> Result<String, String> {
    let assets = release["assets"].as_array().ok_or("The latest NOVA release has no installer assets.")?;
    let asset = assets.iter().find(|asset| asset["name"].as_str() == Some(NOVA_INSTALLER_ASSET))
        .ok_or("The latest NOVA release is missing NOVA-Client-Setup.exe.")?;
    let size = asset["size"].as_u64().ok_or("The NOVA installer size is missing.")?;
    if size == 0 || size > MAX_UPDATER_DOWNLOAD_BYTES {
        return Err("The NOVA installer size is invalid or exceeds the 200 MiB safety limit.".into());
    }
    let url = asset["browser_download_url"].as_str().ok_or("The NOVA installer download link is missing.")?;
    let parsed = reqwest::Url::parse(url).map_err(|_| "The NOVA installer download link is invalid.")?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("github.com") {
        return Err("The NOVA installer link is not a trusted GitHub HTTPS release URL.".into());
    }
    Ok(url.to_owned())
}

#[tauri::command]
async fn check_for_updates() -> Result<Value, String> {
    updater_log("Update check started.");
    let result = check_for_updates_inner().await;
    match &result {
        Ok(info) => updater_log(&format!("Update check completed: current={}, latest={}, available={}.",
            info["currentVersion"].as_str().unwrap_or("unknown"),
            info["latestVersion"].as_str().unwrap_or("unknown"),
            info["available"].as_bool().unwrap_or(false))),
        Err(error) => updater_log(&format!("Update check failed: {error}")),
    }
    result
}

async fn check_for_updates_inner() -> Result<Value, String> {
    let current = env!("CARGO_PKG_VERSION");
    let release = latest_nova_release().await?;
    let latest = release["tag_name"].as_str().ok_or("The latest NOVA release has no version tag.")?
        .trim_start_matches('v').to_owned();
    let available = version_is_newer(&latest, current);
    let download_url = if available { Some(release_installer_url(&release)?) } else { None };
    Ok(json!({
        "currentVersion": current,
        "latestVersion": latest,
        "available": available,
        "releaseNotes": release["body"].as_str().unwrap_or("No release notes were provided."),
        "publishedAt": release["published_at"].as_str().unwrap_or(""),
        "downloadUrl": download_url
    }))
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    updater_log("Update installation started.");
    let result = install_update_inner(app).await;
    if let Err(error) = &result { updater_log(&format!("Update installation failed: {error}")); }
    result
}

async fn install_update_inner(app: tauri::AppHandle) -> Result<(), String> {
    let release = latest_nova_release().await?;
    let latest = release["tag_name"].as_str().ok_or("The latest NOVA release has no version tag.")?
        .trim_start_matches('v');
    if !version_is_newer(latest, env!("CARGO_PKG_VERSION")) {
        return Err("NOVA is already up to date.".into());
    }
    updater_log(&format!("Downloading NOVA version {latest}."));
    let url = release_installer_url(&release)?;
    let client = http_client()?;
    let response = client.get(&url).header("User-Agent", "NOVA-Client-Updater")
        .send().await.map_err(|e| format!("Could not download the NOVA update: {e}"))?
        .error_for_status().map_err(|e| format!("The NOVA installer download failed: {e}"))?;
    if response.content_length().is_some_and(|size| size == 0 || size > MAX_UPDATER_DOWNLOAD_BYTES) {
        return Err("The downloaded NOVA installer has an invalid size.".into());
    }
    let bytes = response.bytes().await.map_err(|e| format!("Could not read the NOVA installer: {e}"))?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_UPDATER_DOWNLOAD_BYTES {
        return Err("The downloaded NOVA installer is empty or exceeds the 200 MiB safety limit.".into());
    }
    updater_log(&format!("Installer download complete: {} bytes.", bytes.len()));
    let installer = std::env::temp_dir().join(format!("NOVA-Client-Update-{}.exe", std::process::id()));
    std::fs::write(&installer, &bytes).map_err(|e| format!("Could not save the NOVA installer: {e}"))?;
    std::process::Command::new(&installer).spawn()
        .map_err(|e| format!("Could not start the NOVA installer: {e}"))?;
    updater_log("Installer process launched; closing NOVA for setup.");
    tokio::time::sleep(Duration::from_millis(900)).await;
    app.exit(0);
    Ok(())
}

static AUTH_REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn refresh_saved_minecraft_account(saved_account: &Account) -> Result<Account, String> {
    let _refresh_guard = AUTH_REFRESH_LOCK.lock().await;
    // Refresh with the same public client ID used for the original OAuth/PKCE
    // sign-in. Using Authenticator::new() here switches client IDs and can make
    // a valid saved Microsoft refresh token look like the user is signed out.
    let auth = Authenticator::with_client_id(MICROSOFT_CLIENT_ID)
        .map_err(|e| format!("Could not prepare Microsoft account refresh: {e}"))?;
    game_log("Refreshing saved Microsoft/Minecraft session.");
    let account = auth.refresh(saved_account).await
        .map_err(|e| {
            game_log(&format!("Microsoft session refresh failed: {e}"));
            format!("Microsoft session could not be refreshed. Sign in again: {e}")
        })?;
    game_log("Saved Microsoft/Minecraft session refreshed successfully.");
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
const MAX_PACK_MANIFEST_BYTES: u64 = 16 * 1024 * 1024;
static ACTIVE_LAUNCHES: OnceLock<Mutex<std::collections::HashSet<String>>> = OnceLock::new();
static TEMP_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

struct InstanceReservation {
    name: String,
}

impl InstanceReservation {
    fn acquire(name: String) -> Result<Self, String> {
        let active = ACTIVE_LAUNCHES.get_or_init(|| Mutex::new(std::collections::HashSet::new()));
        let mut operations = active.lock().map_err(|_| "Could not inspect active Minecraft operations.")?;
        if !operations.insert(name.clone()) {
            return Err("This instance is already running or another operation is using it.".into());
        }
        Ok(Self { name })
    }
}

impl Drop for InstanceReservation {
    fn drop(&mut self) {
        if let Some(active) = ACTIVE_LAUNCHES.get() {
            if let Ok(mut operations) = active.lock() {
                operations.remove(&self.name);
            }
        }
    }
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(12))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Could not initialize secure network client: {e}"))
}

fn download_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        // Large modpacks can take longer than five minutes on slower connections.
        .timeout(Duration::from_secs(1800))
        .build()
        .map_err(|e| format!("Could not initialize the download client: {e}"))
}

fn is_curseforge_cdn_host(host: &str) -> bool {
    matches!(host, "edge.forgecdn.net" | "mediafilez.forgecdn.net")
}

fn is_curseforge_cdn_url(url: &str) -> bool {
    reqwest::Url::parse(url).ok()
        .is_some_and(|parsed| parsed.scheme() == "https"
            && parsed.host_str().is_some_and(is_curseforge_cdn_host))
}

fn curseforge_download_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(1800))
        // CurseForge can move a file from edge.forgecdn.net to its mediafilez
        // CDN. Follow only redirects between these exact trusted hosts so the
        // API key is never forwarded to an unrelated domain.
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            let next_host = attempt.url().host_str().unwrap_or("");
            let previous_host = attempt.previous().last()
                .and_then(|previous| previous.host_str()).unwrap_or("");
            if next_host == previous_host
                || (is_curseforge_cdn_host(next_host) && is_curseforge_cdn_host(previous_host)) {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
        .map_err(|e| format!("Could not initialize the authenticated CurseForge download client: {e}"))
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
    // Browser launch can be blocked by Windows policy or a custom browser handler.
    // The frontend already received the URL, so keep the callback listener alive and
    // let the user open the displayed link manually instead of aborting sign-in.
    let _ = webbrowser::open(&login_url);

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
    let _reservation = InstanceReservation::acquire(name.clone())?;
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
        .map_err(|error| {
            game_log(&format!("Modpack preparation could not read the saved Microsoft account: {error}"));
            "NOVA could not read the saved Microsoft account from Windows Credential Manager. Sign out and sign in again, then retry.".to_string()
        })?;
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

    game_log(&format!("Preparing modpack instance '{name}' for Minecraft {version} ({loader})."));
    let launch = Launch::new(options, account)
        .map_err(|e| format!("Could not prepare modpack installation: {e}"))?
        .with_events(game_event_logger());
    launch.prepare().await
        .map_err(|e| {
            game_log(&format!("Modpack Minecraft preparation failed: {e}"));
            format!("Minecraft or mod-loader installation failed: {e}")
        })?;
    game_log(&format!("Modpack instance '{name}' is prepared."));
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
    let launch_key = if let Some(instance) = instance_name.as_deref() {
        safe_instance_name(instance)?
    } else {
        format!("default-{version}")
    };
    // Reserve before reading metadata so delete/install cannot race the launch setup.
    let reservation = InstanceReservation::acquire(launch_key)?;
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
        .map_err(|error| {
            game_log(&format!("Launch could not read the saved Microsoft account: {error}"));
            "NOVA could not read the saved Microsoft account from Windows Credential Manager. Sign out and sign in again, then retry.".to_string()
        })?;
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

    game_log(&format!("Starting Minecraft {version} ({loader}) in profile '{}'.", instance_name.as_deref().unwrap_or("default")));
    let launch = Launch::new(options, account)
        .map_err(|e| format!("Could not prepare Minecraft: {e}"))?
        .with_events(game_event_logger());
    let mut child = launch.run().await
        .map_err(|error| {
            game_log(&format!("Minecraft failed to install or start: {error}"));
            format!("Minecraft could not be installed or started: {error}")
        })?;
    game_log(&format!("Minecraft process started successfully (pid {:?}).", child.id()));

    // Keep the reservation alive until the game exits; dropping it releases the lock.
    tokio::spawn(async move {
        let _reservation = reservation;
        let exit = child.wait().await;
        let (success, code) = match exit {
            Ok(status) => (status.success(), status.code()),
            Err(error) => {
                game_log(&format!("Could not wait for Minecraft process: {error}"));
                (false, None)
            }
        };
        game_log(&format!("Minecraft process exited: success={success}, exit_code={code:?}."));
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
    if path.is_absolute() || path.components().any(|component| !matches!(component, Component::Normal(_) | Component::CurDir)) {
        return Err(format!("The pack contains an unsafe file path: {input}"));
    }
    for component in path.components() {
        let Component::Normal(part) = component else { continue };
        let Some(part) = part.to_str() else {
            return Err(format!("The pack contains a non-Unicode Windows file path: {input}"));
        };
        if part.chars().any(|c| c.is_control() || "<>:\"|?*".contains(c))
            || part.ends_with('.') || part.ends_with(' ')
        {
            return Err(format!("The pack contains a Windows-invalid file path: {input}"));
        }
        let stem = part.split('.').next().unwrap_or("").trim_end_matches(|c| c == ' ' || c == '.').to_ascii_uppercase();
        if matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9" | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9") {
            return Err(format!("The pack contains a reserved Windows device filename: {input}"));
        }
    }
    Ok(path.to_path_buf())
}

fn instance_folder(instance_name: &str) -> Result<PathBuf, String> {
    let local = std::env::var("LOCALAPPDATA")
        .map_err(|_| "Windows LOCALAPPDATA folder could not be found.".to_string())?;
    Ok(PathBuf::from(local).join("NOVA Client").join("minecraft").join("instances").join(safe_instance_name(instance_name)?))
}

fn extract_pack_overrides(archive_path: &Path, prefix: &str, destination: &Path) -> Result<(), String> {
    let archive_file = std::fs::File::open(archive_path)
        .map_err(|e| format!("Could not open downloaded modpack archive: {e}"))?;
    let mut archive = ZipArchive::new(archive_file)
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
            let output_name = output.file_name().and_then(|n| n.to_str()).unwrap_or("override");
            let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
            let temporary = output.with_file_name(format!(".{output_name}.{}.{sequence}.partial", std::process::id()));
            let extraction_result = (|| -> Result<(), String> {
                let mut file = std::fs::File::create(&temporary)
                    .map_err(|e| format!("Could not create temporary pack override: {e}"))?;
                std::io::copy(&mut entry, &mut file)
                    .map_err(|e| format!("Could not extract pack override: {e}"))?;
                file.flush().map_err(|e| format!("Could not flush pack override: {e}"))?;
                replace_file(&temporary, &output)
                    .map_err(|e| format!("Could not finalize pack override: {e}"))?;
                Ok(())
            })();
            if extraction_result.is_err() { let _ = std::fs::remove_file(&temporary); }
            extraction_result?;
        }
    }
    Ok(())
}

async fn download_pack_archive(app: &tauri::AppHandle, url: &str, destination: &Path) -> Result<(), String> {
    download_pack_archive_with_curseforge_key(app, url, destination, None).await
}

async fn download_pack_archive_with_curseforge_key(
    app: &tauri::AppHandle,
    url: &str,
    destination: &Path,
    api_key: Option<&str>,
) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("The pack supplied a non-HTTPS download URL; it was blocked.".into());
    }
    let client = if api_key.is_some() { curseforge_download_client()? } else { download_client()? };
    let mut request = client.get(url);
    if let Some(key) = api_key.filter(|_| is_curseforge_cdn_url(url)) {
        request = request.header("x-api-key", key);
    }
    let response = request.send().await
        .map_err(|e| format!("Archive download failed: {e}"))?;
    if !response.status().is_success() {
        let status = response.status();
        if status.is_redirection() && api_key.is_some() {
            return Err("CurseForge redirected the archive to an untrusted download host. NOVA stopped the download rather than risk sending credentials or saving a redirect page as a ZIP.".into());
        }
        return Err(format!("Archive server returned HTTP {status}; no archive was saved."));
    }
    if response.url().scheme() != "https" {
        return Err("The archive download redirected to an insecure connection; it was blocked.".into());
    }
    if response.headers().get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().contains("text/html")) {
        return Err("The archive server returned a web page instead of a ZIP file. CurseForge may have blocked or redirected this download; try another pack version.".into());
    }
    let total = response.content_length();
    if total.is_some_and(|size| size > MAX_ARCHIVE_TOTAL_BYTES) {
        return Err("The downloaded archive exceeds NOVA's 2 GiB safety limit.".into());
    }
    let parent = destination.parent().ok_or("Archive destination has no parent folder.")?;
    tokio::fs::create_dir_all(parent).await
        .map_err(|e| format!("Could not create modpack folder: {e}"))?;
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".nova-pack-{}-{sequence}.partial", std::process::id()));
    let result: Result<(), String> = async {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(&temporary).await
            .map_err(|e| format!("Could not create temporary archive: {e}"))?;
        let mut response = response;
        let mut downloaded = 0u64;
        let mut last_reported = 0u64;
        while let Some(chunk) = response.chunk().await
            .map_err(|e| format!("Could not read modpack archive: {e}"))? {
            downloaded = downloaded.checked_add(chunk.len() as u64)
                .ok_or("Archive download size overflowed.")?;
            if downloaded > MAX_ARCHIVE_TOTAL_BYTES {
                return Err("The downloaded archive exceeds NOVA's 2 GiB safety limit.".into());
            }
            file.write_all(&chunk).await
                .map_err(|e| format!("Could not write temporary archive: {e}"))?;
            if downloaded.saturating_sub(last_reported) >= 1024 * 1024 {
                let detail = if let Some(size) = total {
                    format!("Downloading modpack archive: {:.1} / {:.1} MB", downloaded as f64 / 1_048_576.0, size as f64 / 1_048_576.0)
                } else {
                    format!("Downloading modpack archive: {:.1} MB", downloaded as f64 / 1_048_576.0)
                };
                let percent = total.map(|size| ((downloaded as f64 / size.max(1) as f64) * 15.0).round() as u32);
                let _ = app.emit("nova-pack-progress", json!({"phase":"archive","completed":downloaded,"total":total.unwrap_or(0),"percent":percent,"detail":detail}));
                last_reported = downloaded;
            }
        }
        if let Some(size) = total {
            if downloaded != size {
                return Err(format!("The archive download was incomplete (expected {size} bytes, received {downloaded})."));
            }
        }
        file.flush().await.map_err(|e| format!("Could not flush downloaded archive: {e}"))?;
        drop(file);
        // Validate the ZIP central directory before promoting the temporary file.
        // This catches HTML/error bodies and truncated CDN responses at download
        // time, instead of leaving a corrupt archive for the installer to extract.
        let zip_check = std::fs::File::open(&temporary)
            .map_err(|e| format!("Could not verify downloaded archive: {e}"))
            .and_then(|archive| ZipArchive::new(archive).map(|_| ())
                .map_err(|e| format!("The download did not contain a complete valid ZIP archive: {e}")));
        if let Err(error) = zip_check {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(error);
        }
        replace_file(&temporary, destination)
            .map_err(|e| format!("Could not finalize downloaded archive: {e}"))?;
        Ok(())
    }.await;
    if result.is_err() { let _ = tokio::fs::remove_file(&temporary).await; }
    result
}

// Windows std::fs::rename does not replace an existing destination. Move the
// previous file aside first, install the completed temporary file, and restore
// the previous file if the final move fails.
fn replace_file(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    let parent = destination.parent().ok_or_else(|| std::io::Error::new(
        std::io::ErrorKind::InvalidInput, "destination has no parent folder"
    ))?;
    if !destination.exists() {
        return std::fs::rename(source, destination);
    }
    let file_name = destination.file_name().and_then(|n| n.to_str()).unwrap_or("download");
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let backup = parent.join(format!(".{file_name}.{}.{sequence}.backup", std::process::id()));
    std::fs::rename(destination, &backup)?;
    match std::fs::rename(source, destination) {
        Ok(()) => {
            let _ = std::fs::remove_file(backup);
            Ok(())
        }
        Err(error) => {
            // Keep the original failure as the returned error. If restoration also
            // fails, the backup remains beside the destination for manual recovery.
            let _ = std::fs::rename(&backup, destination);
            Err(error)
        }
    }
}

fn atomic_write(destination: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = destination.parent().ok_or("Download destination has no parent folder.")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("Could not create download folder: {e}"))?;
    let file_name = destination.file_name().and_then(|n| n.to_str()).unwrap_or("download");
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".{file_name}.{}.{sequence}.partial", std::process::id()));
    std::fs::write(&temporary, bytes).map_err(|e| format!("Could not write temporary download: {e}"))?;
    if let Err(error) = replace_file(&temporary, destination) {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("Could not finalize downloaded file: {error}"));
    }
    Ok(())
}

async fn download_pack_file(app: &tauri::AppHandle, url: &str, destination: &Path, expected_size: Option<u64>, expected_sha1: Option<&str>) -> Result<(), String> {
    download_pack_file_with_curseforge_key(app, url, destination, expected_size, expected_sha1, None).await
}

async fn download_pack_file_with_curseforge_key(
    app: &tauri::AppHandle,
    url: &str,
    destination: &Path,
    expected_size: Option<u64>,
    expected_sha1: Option<&str>,
    api_key: Option<&str>,
) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("The pack supplied a non-HTTPS download URL; it was blocked.".into());
    }
    if expected_size.is_some_and(|size| size > MAX_ARCHIVE_FILE_BYTES) {
        return Err("A mod file exceeds NOVA's 512 MiB per-file safety limit.".into());
    }
    let client = if api_key.is_some() { curseforge_download_client()? } else { download_client()? };
    let mut request = client.get(url);
    if let Some(key) = api_key.filter(|_| is_curseforge_cdn_url(url)) {
        request = request.header("x-api-key", key);
    }
    let response = request.send().await
        .map_err(|e| format!("Mod download failed: {e}"))?
        .error_for_status().map_err(|e| format!("Mod download server returned an error: {e}"))?;
    if response.url().scheme() != "https" {
        return Err("The mod download redirected to an insecure connection; it was blocked.".into());
    }
    let expected_total = response.content_length().or(expected_size);
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
        let mut last_progress = 0u64;
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
            if downloaded.saturating_sub(last_progress) >= 1024 * 1024 {
                let detail = if let Some(total) = expected_total {
                    format!("Downloading mod file: {:.1} / {:.1} MB", downloaded as f64 / 1_048_576.0, total as f64 / 1_048_576.0)
                } else {
                    format!("Downloading mod file: {:.1} MB", downloaded as f64 / 1_048_576.0)
                };
                emit_pack_progress(app, "files", 0, 0, &detail);
                last_progress = downloaded;
            }
        }
        if let Some(total) = expected_total {
            if downloaded != total {
                return Err(format!("A mod download was incomplete (expected {total} bytes, received {downloaded})."));
            }
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
        replace_file(&temporary, destination)
            .map_err(|e| format!("Could not finalize downloaded mod: {e}"))?;
        Ok(())
    }.await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
    }
    result
}

fn curseforge_loader_matches(mod_loader_type: u64, loader: &str) -> bool {
    let expected = match loader.to_ascii_lowercase().as_str() {
        "forge" => 1,
        "fabric" => 4,
        "quilt" => 5,
        "neoforge" => 6,
        // CurseForge uses 0 for an unspecified/any loader.
        _ => return false,
    };
    mod_loader_type == 0 || mod_loader_type == expected
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
async fn install_mod_to_instance(
    app: tauri::AppHandle,
    provider: String,
    project_id: String,
    version_id: String,
    instance_name: String,
) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let _reservation = InstanceReservation::acquire(name.clone())?;
    install_mod_to_instance_inner(app, provider, project_id, version_id, name).await
}

async fn install_mod_to_instance_inner(
    app: tauri::AppHandle,
    provider: String,
    project_id: String,
    version_id: String,
    name: String,
) -> Result<Value, String> {
    if project_id.is_empty() || !project_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("The selected mod project ID is invalid.".into());
    }
    if version_id.is_empty() || !version_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("The selected mod version ID is invalid.".into());
    }
    let folder = instance_folder(&name)?;
    let metadata_path = folder.join("nova-instance.json");
    let metadata_bytes = std::fs::read(&metadata_path)
        .map_err(|_| "Choose an existing NOVA instance before installing a mod.".to_string())?;
    let metadata: Value = serde_json::from_slice(&metadata_bytes)
        .map_err(|_| "The selected instance metadata is damaged. Recreate the instance before installing mods.".to_string())?;
    let minecraft_version = metadata["minecraftVersion"].as_str()
        .ok_or("The selected instance has no Minecraft version.")?.to_owned();
    let loader = metadata["loader"].as_str().unwrap_or("vanilla").to_owned();
    if loader == "vanilla" {
        return Err("This instance uses Vanilla. Create or choose a Fabric, Forge, NeoForge, or Quilt instance to install mods.".into());
    }
    let client = http_client()?;
    let (download_url, filename, expected_size, expected_sha1, mod_title) = match provider.as_str() {
        "modrinth" => {
            let response: Value = client.get(format!("https://api.modrinth.com/v2/version/{version_id}"))
                .send().await.map_err(|e| format!("Could not reach Modrinth: {e}"))?
                .error_for_status().map_err(|e| format!("Modrinth could not find that mod version: {e}"))?
                .json().await.map_err(|e| format!("Modrinth returned an unreadable mod version: {e}"))?;
            if response["project_id"].as_str() != Some(project_id.as_str()) {
                return Err("The selected Modrinth version does not belong to this project.".into());
            }
            let supported_versions = response["game_versions"].as_array().ok_or("Modrinth did not list compatible Minecraft versions.")?;
            if !supported_versions.iter().any(|v| v.as_str() == Some(minecraft_version.as_str())) {
                return Err(format!("This mod version does not support Minecraft {minecraft_version}."));
            }
            let loaders = response["loaders"].as_array().ok_or("Modrinth did not list compatible mod loaders.")?;
            if !loaders.iter().any(|v| v.as_str().is_some_and(|s| s.eq_ignore_ascii_case(&loader))) {
                return Err(format!("This mod version does not support the {} loader.", loader));
            }
            let file = response["files"].as_array()
                .and_then(|files| files.iter().find(|f| f["primary"].as_bool() == Some(true)).or_else(|| files.first()))
                .ok_or("Modrinth did not provide a downloadable mod file.")?;
            (
                file["url"].as_str().ok_or("Modrinth download URL is missing.")?.to_owned(),
                file["filename"].as_str().ok_or("Modrinth file name is missing.")?.to_owned(),
                file["size"].as_u64(),
                file["hashes"]["sha1"].as_str().map(str::to_owned),
                response["name"].as_str().unwrap_or("Mod").to_owned(),
            )
        },
        "curseforge" => {
            let mod_id: u64 = project_id.parse().map_err(|_| "The CurseForge project ID is invalid.")?;
            let file_id: u64 = version_id.parse().map_err(|_| "The CurseForge file ID is invalid.")?;
            let api_key = curseforge_api_key()?;
            let mut headers = HeaderMap::new();
            headers.insert("x-api-key", HeaderValue::from_str(&api_key).map_err(|_| "The saved CurseForge API key is invalid.")?);
            headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
            let response: Value = client.get(format!("{CURSEFORGE_API}/mods/{mod_id}/files/{file_id}"))
                .headers(headers.clone()).send().await.map_err(|e| format!("Could not reach CurseForge: {e}"))?
                .error_for_status().map_err(|e| format!("CurseForge could not find that mod file: {e}"))?
                .json().await.map_err(|e| format!("CurseForge returned an unreadable mod file: {e}"))?;
            let file = &response["data"];
            if file["modId"].as_u64() != Some(mod_id) {
                return Err("The selected CurseForge file does not belong to this project.".into());
            }
            let versions = file["gameVersions"].as_array().ok_or("CurseForge did not list compatible game versions.")?;
            if !versions.iter().any(|v| v.as_str() == Some(minecraft_version.as_str())) {
                return Err(format!("This mod file does not list Minecraft {minecraft_version} as compatible."));
            }
            let mod_loader_type = file["modLoaderType"].as_u64().unwrap_or(0);
            if !curseforge_loader_matches(mod_loader_type, &loader) {
                return Err(format!("This CurseForge file does not support the {} loader.", loader));
            }
            let curseforge_sha1 = file["hashes"].as_array()
                .and_then(|hashes| hashes.iter().find(|hash| hash["algo"].as_u64() == Some(1)))
                .and_then(|hash| hash["value"].as_str())
                .filter(|hash| hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()))
                .map(str::to_owned)
                .ok_or("CurseForge did not provide a valid SHA-1 checksum for this mod file; installation was stopped.")?;
            let expected_file_size = file["fileLength"].as_u64()
                .ok_or("CurseForge did not provide the expected mod file size; installation was stopped.")?;
            let url = if let Some(url) = file["downloadUrl"].as_str() {
                url.to_owned()
            } else {
                client.get(format!("{CURSEFORGE_API}/mods/{mod_id}/files/{file_id}/download-url"))
                    .headers(headers).send().await.map_err(|e| format!("Could not request the CurseForge download: {e}"))?
                    .error_for_status().map_err(|e| format!("CurseForge did not provide a download URL: {e}"))?
                    .json::<Value>().await.map_err(|e| format!("CurseForge download response was unreadable: {e}"))?["data"]
                    .as_str().ok_or("CurseForge did not provide a download URL for this mod.")?.to_owned()
            };
            (
                url,
                file["fileName"].as_str().ok_or("CurseForge file name is missing.")?.to_owned(),
                Some(expected_file_size),
                Some(curseforge_sha1),
                file["displayName"].as_str().unwrap_or("Mod").to_owned(),
            )
        },
        _ => return Err("Choose Modrinth or CurseForge as the mod source.".into()),
    };
    if !download_url.starts_with("https://") {
        return Err("The mod source supplied a non-HTTPS download URL; it was blocked.".into());
    }
    let expected_size = expected_size
        .filter(|size| *size > 0 && *size <= MAX_ARCHIVE_FILE_BYTES)
        .ok_or("The mod source did not provide a valid file size; installation was stopped.")?;
    let expected_sha1 = expected_sha1
        .filter(|hash| hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()))
        .ok_or("The mod source did not provide a valid SHA-1 checksum; installation was stopped.")?;
    let safe_path = safe_relative_path(&filename)?;
    if safe_path.components().count() != 1 {
        return Err("The selected mod filename contains an invalid path.".into());
    }
    let safe_filename = Path::new(&filename).file_name().and_then(|s| s.to_str())
        .filter(|s| !s.is_empty() && s.to_ascii_lowercase().ends_with(".jar"))
        .ok_or("The selected mod file is not a valid JAR filename.")?;
    let mods_folder = folder.join("mods");
    std::fs::create_dir_all(&mods_folder).map_err(|e| format!("Could not create instance mods folder: {e}"))?;
    let destination = mods_folder.join(safe_filename);
    if destination.exists() {
        return Err(format!("A mod named '{safe_filename}' is already installed in this instance."));
    }
    let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let staging = mods_folder.join(format!(".{safe_filename}.{}.{}.installing", std::process::id(), sequence));
    emit_pack_progress(&app, "files", 0, 0, "Downloading selected mod");
    download_pack_file(&app, &download_url, &staging, Some(expected_size), Some(&expected_sha1)).await?;
    let jar_check = std::fs::File::open(&staging)
        .map_err(|e| format!("Could not verify downloaded mod archive: {e}"))
        .and_then(|file| ZipArchive::new(file).map(|_| ()).map_err(|e| format!("The downloaded file is not a valid JAR archive: {e}")));
    if let Err(error) = jar_check {
        let _ = std::fs::remove_file(&staging);
        return Err(error);
    }
    // The outer command reserves this instance for the whole download, preventing
    // launch or deletion until the validated JAR has been committed.
    if destination.exists() {
        let _ = std::fs::remove_file(&staging);
        return Err(format!("A mod named '{safe_filename}' is already installed in this instance."));
    }
    std::fs::rename(&staging, &destination).map_err(|e| {
        let _ = std::fs::remove_file(&staging);
        format!("Could not install downloaded mod: {e}")
    })?;
    Ok(json!({"name":mod_title,"fileName":safe_filename,"instanceName":name,"minecraftVersion":minecraft_version,"loader":loader}))
}


fn modrinth_minecraft_version(index: &Value, version: &Value) -> Result<String, String> {
    index["dependencies"]["minecraft"]["version"].as_str()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| version["game_versions"].as_array()
            .and_then(|versions| versions.iter().filter_map(Value::as_str).find(|value| !value.trim().is_empty())))
        .map(str::to_owned)
        .ok_or_else(|| "This Modrinth pack does not declare a Minecraft version, and Modrinth's version metadata does not provide one either.".to_string())
}

#[tauri::command]
async fn install_modrinth_modpack(app: tauri::AppHandle, project_id: String, instance_name: String, version_id: Option<String>) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let _reservation = InstanceReservation::acquire(name.clone())?;
    if project_id.is_empty() || !project_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("The selected Modrinth modpack project ID is invalid.".into());
    }
    if version_id.as_deref().is_some_and(|id| id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')) {
        return Err("The selected Modrinth modpack version ID is invalid.".into());
    }
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
    let folder = instance_folder(&name)?;
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create instance folder: {e}"))?;
    let archive_path = folder.join(".nova-modrinth-pack.zip");
    emit_pack_progress(&app, "archive", 0, 0, "Downloading Modrinth archive");
    download_pack_archive(&app, download_url, &archive_path).await?;
    let mut archive = ZipArchive::new(std::fs::File::open(&archive_path)
        .map_err(|e| format!("Could not open downloaded Modrinth archive: {e}"))?)
        .map_err(|e| format!("The Modrinth pack is not a valid ZIP archive: {e}"))?;
    let mut index_bytes = Vec::new();
    let mut index_entry = archive.by_name("modrinth.index.json")
        .map_err(|_| "This Modrinth archive is missing modrinth.index.json.".to_string())?;
    if index_entry.size() > MAX_PACK_MANIFEST_BYTES {
        return Err("The Modrinth manifest exceeds NOVA's 16 MiB safety limit.".into());
    }
    index_entry.read_to_end(&mut index_bytes)
        .map_err(|e| format!("Could not read Modrinth manifest: {e}"))?;
    drop(index_entry);
    let index: Value = serde_json::from_slice(&index_bytes)
        .map_err(|e| format!("Modrinth manifest is invalid: {e}"))?;
    drop(archive);
    if index["formatVersion"].as_u64() != Some(1) {
        return Err("This Modrinth modpack format is not supported.".into());
    }
    // Some exported Modrinth packs omit dependencies.minecraft. In that case,
    // use the selected Modrinth version's game_versions metadata as the source
    // of truth instead of rejecting an otherwise valid pack.
    let mc_version = modrinth_minecraft_version(&index, latest)?;
    let loader_entry = ["fabric-loader", "forge", "neoforge", "quilt-loader", "legacy-fabric"]
        .iter().find_map(|key| index["dependencies"][*key].as_str().map(|v| (*key, v)));
    let (loader, loader_build) = if let Some((key, build)) = loader_entry {
        (match key { "fabric-loader"=>"fabric", "quilt-loader"=>"quilt", "legacy-fabric"=>"legacyfabric", other=>other }.to_owned(), build.to_owned())
    } else { ("vanilla".to_owned(), String::new()) };
    let pack_files = index["files"].as_array().ok_or("Modrinth manifest has no files list.")?;
    if pack_files.len() > MAX_ARCHIVE_ENTRIES {
        return Err("The Modrinth manifest contains too many files and was blocked for safety.".into());
    }
    let declared_download_bytes = pack_files.iter()
        .filter(|item| item["env"]["client"].as_str() != Some("unsupported"))
        .try_fold(0u64, |total, item| {
            let size = item["fileSize"].as_u64()
                .filter(|size| *size > 0 && *size <= MAX_ARCHIVE_FILE_BYTES)
                .ok_or_else(|| "A Modrinth file has missing or invalid size metadata.".to_string())?;
            total.checked_add(size).ok_or_else(|| "Modrinth pack download size overflowed.".to_string())
        })?;
    if declared_download_bytes > MAX_ARCHIVE_TOTAL_BYTES {
        return Err("The Modrinth pack's required downloads exceed NOVA's 2 GiB safety limit.".into());
    }
    let total_files = pack_files.iter().filter(|item| item["env"]["client"].as_str() != Some("unsupported")).count() as u64;
    let mut completed_files = 0u64;
    emit_pack_progress(&app, "files", 0, total_files, "Downloading Modrinth files");
    for (i, item) in pack_files.iter().enumerate() {
        if item["env"]["client"].as_str() == Some("unsupported") { continue; }
        let relative = safe_relative_path(item["path"].as_str().ok_or("A Modrinth file path is missing.")?)?;
        let urls = item["downloads"].as_array().ok_or("A Modrinth file has no download URLs.")?;
        if urls.is_empty() { return Err("A Modrinth file has no download URLs.".into()); }
        let size = item["fileSize"].as_u64()
            .filter(|size| *size > 0 && *size <= MAX_ARCHIVE_FILE_BYTES)
            .ok_or_else(|| format!("Modrinth file {} has missing or invalid size metadata.", i + 1))?;
        let sha1 = item["hashes"]["sha1"].as_str()
            .filter(|hash| hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .ok_or_else(|| format!("Modrinth file {} has missing or invalid SHA-1 metadata.", i + 1))?;
        let mut last_error = None;
        let mut downloaded = false;
        for candidate in urls.iter().filter_map(Value::as_str) {
            match download_pack_file(&app, candidate, &folder.join(relative.clone()), Some(size), Some(sha1)).await {
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
    extract_pack_overrides(&archive_path, "overrides/", &folder)?;
    extract_pack_overrides(&archive_path, "client-overrides/", &folder)?;
    let _ = std::fs::remove_file(&archive_path);
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
    let _reservation = InstanceReservation::acquire(name.clone())?;
    if mod_id.parse::<u64>().ok().filter(|id| *id > 0).is_none() {
        return Err("The selected CurseForge modpack project ID is invalid.".into());
    }
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
    let folder = instance_folder(&name)?;
    std::fs::create_dir_all(&folder).map_err(|e| format!("Could not create instance folder: {e}"))?;
    let archive_path = folder.join(".nova-curseforge-pack.zip");
    emit_pack_progress(&app, "archive", 0, 0, "Downloading CurseForge archive");
    download_pack_archive_with_curseforge_key(&app, &manifest_url, &archive_path, Some(&api_key)).await?;
    let mut archive = ZipArchive::new(std::fs::File::open(&archive_path)
        .map_err(|e| format!("Could not open downloaded CurseForge archive: {e}"))?)
        .map_err(|e| format!("The CurseForge pack is not a valid ZIP archive: {e}"))?;
    let mut manifest_bytes = Vec::new();
    let mut manifest_entry = archive.by_name("manifest.json")
        .map_err(|_| "This CurseForge archive is missing manifest.json.".to_string())?;
    if manifest_entry.size() > MAX_PACK_MANIFEST_BYTES {
        return Err("The CurseForge manifest exceeds NOVA's 16 MiB safety limit.".into());
    }
    manifest_entry.read_to_end(&mut manifest_bytes)
        .map_err(|e| format!("Could not read CurseForge manifest: {e}"))?;
    drop(manifest_entry);
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("CurseForge manifest is invalid: {e}"))?;
    drop(archive);
    if manifest["manifestType"].as_str() != Some("minecraftModpack") {
        return Err("The downloaded CurseForge file is not a Minecraft modpack.".into());
    }
    let mc_version = manifest["minecraft"]["version"].as_str()
        .ok_or("CurseForge pack does not specify a Minecraft version.")?.to_owned();
    let loader_entry = manifest["minecraft"]["modLoaders"].as_array()
        .and_then(|a| a.iter().find(|v| v["primary"].as_bool() == Some(true)).or_else(|| a.first()))
        .and_then(|v| v["id"].as_str());
    let (loader, loader_build) = if let Some(id) = loader_entry { loader_from_id(id)? } else { ("vanilla".into(), String::new()) };
    let pack_files = manifest["files"].as_array().ok_or("CurseForge manifest has no files list.")?;
    if pack_files.len() > MAX_ARCHIVE_ENTRIES {
        return Err("The CurseForge manifest contains too many files and was blocked for safety.".into());
    }
    let total_files = pack_files.iter().filter(|item| item["required"].as_bool() != Some(false)).count() as u64;
    let mut total_download_bytes = 0u64;
    let mut completed_files = 0u64;
    emit_pack_progress(&app, "files", 0, total_files, "Downloading CurseForge files");
    for (i, item) in pack_files.iter().enumerate() {
        if item["required"].as_bool() == Some(false) { continue; }
        let project = item["projectID"].as_u64().ok_or("CurseForge project ID is missing.")?;
        let file = item["fileID"].as_u64().ok_or("CurseForge file ID is missing.")?;
        // Read file metadata first: CurseForge may include a direct downloadUrl.
        // Prefer it, and only call the download-url endpoint when it is absent.
        let mod_details: Value = client.get(format!("{CURSEFORGE_API}/mods/{project}/files/{file}"))
            .headers(headers.clone()).send().await
            .map_err(|e| format!("Could not verify CurseForge mod {}: {e}", i+1))?
            .error_for_status().map_err(|e| format!("CurseForge could not verify mod {}: {e}", i+1))?
            .json().await.map_err(|e| format!("CurseForge mod {} metadata was unreadable: {e}", i+1))?;
        let details = &mod_details["data"];
        if details["modId"].as_u64() != Some(project) || details["id"].as_u64() != Some(file) {
            return Err(format!("CurseForge returned mismatched metadata for mod {project}/{file}."));
        }
        let url: String = if let Some(direct_url) = details["downloadUrl"].as_str().filter(|url| url.starts_with("https://")) {
            direct_url.to_owned()
        } else {
            let response = client.get(format!("{CURSEFORGE_API}/mods/{project}/files/{file}/download-url"))
                .headers(headers.clone()).send().await
                .map_err(|e| format!("Could not resolve CurseForge mod {}: {e}", i+1))?;
            let status = response.status();
            if status == reqwest::StatusCode::FORBIDDEN {
                let file_name = details["fileName"].as_str().unwrap_or("this file");
                return Err(format!("CurseForge refused the download URL for required mod {} ({file_name}, {project}/{file}) with HTTP 403. This may be a third-party-download restriction on the file or an API-key permission issue. NOVA cannot bypass that restriction; try another modpack version or install this file through CurseForge.", i+1));
            }
            let payload = response.error_for_status()
                .map_err(|e| format!("CurseForge could not provide mod {}: {e}", i+1))?
                .json::<Value>().await
                .map_err(|e| format!("CurseForge mod {} download response was unreadable: {e}", i+1))?;
            payload["data"].as_str()
                .ok_or_else(|| format!("CurseForge did not provide a download URL for required mod {project}/{file}."))?
                .to_owned()
        };
        let filename = format!("{project}-{file}.jar");
        let mods_folder = folder.join("mods");
        std::fs::create_dir_all(&mods_folder).map_err(|e| format!("Could not create mods folder: {e}"))?;
        let expected_size = details["fileLength"].as_u64()
            .filter(|size| *size > 0 && *size <= MAX_ARCHIVE_FILE_BYTES)
            .ok_or_else(|| format!("CurseForge did not provide a valid size for mod {project}/{file}."))?;
        total_download_bytes = total_download_bytes.checked_add(expected_size)
            .ok_or("CurseForge pack download size overflowed.")?;
        if total_download_bytes > MAX_ARCHIVE_TOTAL_BYTES {
            return Err("The CurseForge pack's required downloads exceed NOVA's 2 GiB safety limit.".into());
        }
        let expected_sha1 = details["hashes"].as_array()
            .and_then(|hashes| hashes.iter().find(|hash| hash["algo"].as_u64() == Some(1)))
            .and_then(|hash| hash["value"].as_str())
            .filter(|hash| hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .ok_or_else(|| format!("CurseForge did not provide a valid SHA-1 checksum for mod {project}/{file}."))?;
        let destination = mods_folder.join(&filename);
        let sequence = TEMP_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let staging = mods_folder.join(format!(".{filename}.{}.{sequence}.partial", std::process::id()));
        download_pack_file_with_curseforge_key(&app, &url, &staging, Some(expected_size), Some(expected_sha1), Some(&api_key)).await
            .map_err(|e| format!("Could not save CurseForge mod {}: {e}", i+1))?;
        let jar_check = std::fs::File::open(&staging)
            .map_err(|e| format!("Could not verify CurseForge mod {}: {e}", i+1))
            .and_then(|file| ZipArchive::new(file).map(|_| ()).map_err(|e| format!("CurseForge mod {} is not a valid JAR archive: {e}", i+1)));
        if let Err(error) = jar_check {
            let _ = std::fs::remove_file(&staging);
            return Err(error);
        }
        replace_file(&staging, &destination).map_err(|e| {
            let _ = std::fs::remove_file(&staging);
            format!("Could not finalize CurseForge mod {}: {e}", i+1)
        })?;
        completed_files += 1;
        emit_pack_progress(&app, "files", completed_files, total_files, &format!("Downloaded file {completed_files} of {total_files}"));
    }
    extract_pack_overrides(&archive_path, "overrides/", &folder)?;
    let _ = std::fs::remove_file(&archive_path);
    let title = manifest["name"].as_str().filter(|s| !s.trim().is_empty()).unwrap_or(&name).to_owned();
    save_instance_metadata(&folder, json!({
        "instanceName":name,"name":title,"minecraftVersion":mc_version,"loader":loader,
        "loaderVersion":loader_build,"source":"curseforge","projectId":mod_id,"packFileId":file_id
    }))
}


#[tauri::command]
fn create_instance(instance_name: String, display_name: String, version: String, loader: String) -> Result<Value, String> {
    let name = safe_instance_name(&instance_name)?;
    let _reservation = InstanceReservation::acquire(name.clone())?;
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
    match save_instance_metadata(&folder, metadata) {
        Ok(saved) => Ok(saved),
        Err(error) => {
            let _ = std::fs::remove_dir_all(&folder);
            Err(error)
        }
    }
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
    // Hold the operation lock through deletion so launch/install cannot start
    // after the running check but before the directory is removed.
    let operations = active.lock().map_err(|_| "Could not inspect running Minecraft processes.")?;
    if operations.contains(&name) {
        return Err("This instance is currently running or being modified. Close Minecraft and try again.".into());
    }
    let folder = instance_folder(&name)?;
    if folder.exists() {
        std::fs::remove_dir_all(&folder)
            .map_err(|e| format!("Could not delete instance files: {e}"))?;
    }
    drop(operations);
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::{curseforge_loader_matches, modrinth_minecraft_version, safe_instance_name, safe_relative_path, InstanceReservation, TEMP_FILE_COUNTER};
    use serde_json::json;

    #[test]
    fn curseforge_cdn_authentication_is_limited_to_official_https_hosts() {
        assert!(super::is_curseforge_cdn_url("https://edge.forgecdn.net/files/1/2/mod.jar"));
        assert!(super::is_curseforge_cdn_url("https://mediafilez.forgecdn.net/files/1/2/mod.jar"));
        assert!(!super::is_curseforge_cdn_url("http://edge.forgecdn.net/files/1/2/mod.jar"));
        assert!(!super::is_curseforge_cdn_url("https://example.com/mod.jar"));
        assert!(!super::is_curseforge_cdn_url("https://mediafilez.forgecdn.net.evil.example/mod.jar"));
        assert!(!super::is_curseforge_cdn_url("https://edge.forgecdn.net.evil.example/mod.jar"));
    }

    #[test]
    fn updater_compares_release_versions_numerically() {
        assert!(super::version_is_newer("0.1.1", "0.1.0"));
        assert!(super::version_is_newer("1.0.0", "0.9.99"));
        assert!(!super::version_is_newer("0.1.0", "0.1.0"));
        assert!(!super::version_is_newer("0.1.0", "0.1.1"));
        assert!(super::version_is_newer("v0.2.0", "0.1.9"));
    }

    #[test]
    fn modrinth_minecraft_version_prefers_manifest_dependency() {
        let index = json!({"dependencies":{"minecraft":{"version":"1.21.1"}}});
        let version = json!({"game_versions":["1.20.1"]});
        assert_eq!(modrinth_minecraft_version(&index, &version).unwrap(), "1.21.1");
    }

    #[test]
    fn modrinth_minecraft_version_falls_back_to_selected_version_metadata() {
        let index = json!({"dependencies":{}});
        let version = json!({"game_versions":["1.21.4","1.21.3"]});
        assert_eq!(modrinth_minecraft_version(&index, &version).unwrap(), "1.21.4");
    }

    #[test]
    fn modrinth_minecraft_version_rejects_missing_metadata() {
        let index = json!({"dependencies":{}});
        let version = json!({"game_versions":[]});
        assert!(modrinth_minecraft_version(&index, &version).is_err());
    }

    #[test]
    fn atomic_write_replaces_existing_metadata_on_windows() {
        let root = std::env::temp_dir().join(format!(
            "nova-atomic-write-test-{}-{}",
            std::process::id(),
            TEMP_FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("nova-instance.json");
        std::fs::write(&target, b"old metadata").unwrap();
        super::atomic_write(&target, b"new metadata").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(), b"new metadata");
        let _ = std::fs::remove_dir_all(root);
    }

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
    fn curseforge_loader_types_are_checked() {
        assert!(curseforge_loader_matches(4, "fabric"));
        assert!(curseforge_loader_matches(1, "forge"));
        assert!(curseforge_loader_matches(6, "neoforge"));
        assert!(curseforge_loader_matches(5, "quilt"));
        assert!(curseforge_loader_matches(0, "fabric"));
        assert!(!curseforge_loader_matches(1, "fabric"));
        assert!(!curseforge_loader_matches(4, "forge"));
    }

    #[test]
    fn instance_reservations_prevent_conflicting_operations_and_release() {
        let key = format!("reservation-test-{}", TEMP_FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        let first = InstanceReservation::acquire(key.clone()).unwrap();
        assert!(InstanceReservation::acquire(key.clone()).is_err());
        drop(first);
        assert!(InstanceReservation::acquire(key).is_ok());
    }

    #[test]
    fn relative_paths_accept_normal_mod_paths() {
        assert!(safe_relative_path("mods/example.jar").is_ok());
    }

    #[test]
    fn relative_paths_reject_windows_special_names_and_streams() {
        assert!(safe_relative_path("mods/CON.jar").is_err());
        assert!(safe_relative_path("mods/NUL").is_err());
        assert!(safe_relative_path("mods/example.jar:payload").is_err());
        assert!(safe_relative_path("mods/trailing.").is_err());
        assert!(safe_relative_path("mods/trailing ").is_err());
        assert!(safe_relative_path("mods/bad?.jar").is_err());
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            curseforge_search,
            begin_microsoft_login,
            minecraft_account_name,
            sign_out_minecraft,
            check_for_updates,
            install_update,
            get_updater_logs,
            get_game_logs,
            minecraft_versions,
            launch_minecraft,
            prepare_modpack_instance,
            install_modrinth_modpack,
            curseforge_modpack_versions,
            install_curseforge_modpack,
            install_mod_to_instance,
            create_instance,
            list_instances,
            delete_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running NOVA Client");
}
