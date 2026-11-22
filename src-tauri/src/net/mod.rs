//! What every HTTP client of the launcher shares: the proxy.
//!
//! The settings (`proxy` in `launcher.json`) and the password (system keychain) are loaded at
//! startup and when the player saves them. Clients ask [`agent_builder`] (ureq) or
//! [`updater_proxy`] (the updater's reqwest) for their proxy when they start a request, so a
//! change applies to new requests while running downloads finish on the old one. "System" leaves
//! each client on its own defaults: the `*_PROXY` / `NO_PROXY` environment variables, plus the
//! Windows Internet settings.

pub mod limit;
mod secret;

use std::path::Path;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use ureq::config::ConfigBuilder;
use ureq::typestate::AgentScope;

use crate::launcher_settings::config;
use secret::{Keychain, SecretStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// The system's own settings.
    #[default]
    System,
    /// A direct connection, whatever the system says.
    None,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    #[default]
    Http,
    Https,
    Socks5,
}

#[cfg(any(feature = "updater", test))]
impl Protocol {
    fn scheme(self) -> &'static str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
            Protocol::Socks5 => "socks5",
        }
    }
}

/// What the proxy form starts with, and what a file that lacks a port means.
const DEFAULT_PORT: u16 = 8080;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProxySettings {
    pub mode: Mode,
    pub protocol: Protocol,
    pub host: String,
    pub port: u16,
    pub username: String,
    /// Hosts that bypass the proxy, separated by commas or spaces: `example.com`,
    /// `.example.com` (and `*.example.com`) for subdomains, `*` for everything.
    pub no_proxy: String,
}

impl Default for ProxySettings {
    fn default() -> Self {
        ProxySettings {
            mode: Mode::default(),
            protocol: Protocol::default(),
            host: String::new(),
            port: DEFAULT_PORT,
            username: String::new(),
            no_proxy: String::new(),
        }
    }
}

fn is_host_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '.' || c == '-'
}

impl ProxySettings {
    pub fn exclusions(&self) -> Vec<&str> {
        self.no_proxy
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|entry| !entry.is_empty())
            .collect()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.mode != Mode::Manual {
            return Ok(());
        }
        let host = self.host.trim();
        if host.is_empty() || !host.chars().all(is_host_char) || host.starts_with(['.', '-']) {
            return Err(format!(
                "`{}` isn't a proxy address (a host name or an IP address).",
                self.host
            ));
        }
        if self.port == 0 {
            return Err("Give the proxy's port (1 to 65535).".to_string());
        }
        if self.username.chars().count() > 256 || self.username.chars().any(char::is_control) {
            return Err("The proxy's user name is too long or has control characters.".to_string());
        }
        for entry in self.exclusions() {
            let rest = entry
                .strip_prefix("*.")
                .or_else(|| entry.strip_prefix('.'))
                .unwrap_or(entry);
            let ok = entry == "*"
                || (!rest.is_empty()
                    && rest.chars().all(is_host_char)
                    && !rest.starts_with(['.', '-']));
            if !ok {
                return Err(format!(
                    "`{entry}` isn't a host to leave out: use example.com, .example.com or *."
                ));
            }
        }
        Ok(())
    }

    fn uses_credentials(&self) -> bool {
        self.mode == Mode::Manual && !self.username.is_empty()
    }
}

#[derive(Default)]
struct Live {
    settings: ProxySettings,
    password: Option<String>,
}

fn live() -> &'static RwLock<Live> {
    static LIVE: OnceLock<RwLock<Live>> = OnceLock::new();
    LIVE.get_or_init(|| RwLock::new(Live::default()))
}

/// Makes `settings` the proxy of the requests that start from now on.
fn configure(settings: ProxySettings, password: Option<String>) {
    *live().write().unwrap() = Live { settings, password };
}

/// The proxy's host and user name, for the diagnostic files to hide.
pub fn scrub_names() -> (Option<String>, Option<String>) {
    let live = live().read().unwrap();
    let named = |name: &String| (!name.is_empty()).then(|| name.clone());
    (named(&live.settings.host), named(&live.settings.username))
}

fn build_proxy(settings: &ProxySettings, password: Option<&str>) -> Result<ureq::Proxy, String> {
    let protocol = match settings.protocol {
        Protocol::Http => ureq::ProxyProtocol::Http,
        Protocol::Https => ureq::ProxyProtocol::Https,
        Protocol::Socks5 => ureq::ProxyProtocol::Socks5,
    };
    let mut builder = ureq::Proxy::builder(protocol)
        .host(settings.host.trim())
        .port(settings.port);
    if settings.uses_credentials() {
        builder = builder.username(&settings.username);
        if let Some(password) = password {
            builder = builder.password(password);
        }
    }
    for entry in settings.exclusions() {
        builder = builder.no_proxy(entry);
    }
    builder
        .build()
        .map_err(|e| format!("The proxy can't be used: {e}"))
}

fn with_proxy(
    builder: ConfigBuilder<AgentScope>,
    settings: &ProxySettings,
    password: Option<&str>,
) -> Result<ConfigBuilder<AgentScope>, String> {
    Ok(match settings.mode {
        Mode::System => builder,
        Mode::None => builder.proxy(None),
        Mode::Manual => builder.proxy(Some(build_proxy(settings, password)?)),
    })
}

/// The configuration of a ureq client with the proxy the player chose. A manual proxy that
/// can't be built is an error: the request must not go out directly instead.
pub fn agent_builder() -> Result<ConfigBuilder<AgentScope>, String> {
    let live = live().read().unwrap();
    with_proxy(
        ureq::Agent::config_builder(),
        &live.settings,
        live.password.as_deref(),
    )
}

#[cfg(any(feature = "updater", test))]
/// What the updater's HTTP client (reqwest) must do.
#[derive(Debug, PartialEq, Eq)]
pub enum UpdaterProxy {
    /// Its own defaults.
    System,
    Direct,
    Proxy {
        url: String,
        no_proxy: String,
    },
}

#[cfg(any(feature = "updater", test))]
fn escape(part: &str) -> String {
    part.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

#[cfg(any(feature = "updater", test))]
fn updater_proxy_of(settings: &ProxySettings, password: Option<&str>) -> UpdaterProxy {
    match settings.mode {
        Mode::System => UpdaterProxy::System,
        Mode::None => UpdaterProxy::Direct,
        Mode::Manual => {
            let credentials = if settings.uses_credentials() {
                match password {
                    Some(password) => {
                        format!("{}:{}@", escape(&settings.username), escape(password))
                    }
                    None => format!("{}@", escape(&settings.username)),
                }
            } else {
                String::new()
            };
            UpdaterProxy::Proxy {
                url: format!(
                    "{}://{credentials}{}:{}",
                    settings.protocol.scheme(),
                    settings.host.trim(),
                    settings.port
                ),
                no_proxy: settings.exclusions().join(","),
            }
        }
    }
}

#[cfg(feature = "updater")]
pub fn updater_proxy() -> UpdaterProxy {
    let live = live().read().unwrap();
    updater_proxy_of(&live.settings, live.password.as_deref())
}

// --- Saving ---------------------------------------------------------------------------------

/// What the settings page shows.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProxyView {
    pub settings: ProxySettings,
    pub has_password: bool,
    /// Why the keychain can't be used, if it can't.
    pub keychain_error: Option<String>,
}

pub fn load_settings(config_dir: &Path) -> Result<ProxySettings, String> {
    match config::read(config_dir)?.get("proxy") {
        None => Ok(ProxySettings::default()),
        Some(value) => {
            let mut settings: ProxySettings = serde_json::from_value(value.clone())
                .map_err(|e| format!("The proxy settings in launcher.json aren't valid: {e}"))?;
            if settings.port == 0 {
                // Saved by an earlier version while no proxy was in use: show a usable port.
                settings.port = DEFAULT_PORT;
            }
            Ok(settings)
        }
    }
}

fn view(settings: ProxySettings, secrets: &dyn SecretStore) -> ProxyView {
    let (has_password, keychain_error) = match secrets.get() {
        Ok(secret) => (secret.is_some(), None),
        Err(e) => (false, Some(e)),
    };
    ProxyView {
        settings,
        has_password,
        keychain_error,
    }
}

/// Validates and keeps `settings`; `password` is `None` to keep the saved one, empty to forget
/// it. A password that can't go to the keychain stops everything: nothing is saved.
fn save(
    config_dir: &Path,
    secrets: &dyn SecretStore,
    settings: ProxySettings,
    password: Option<String>,
) -> Result<ProxyView, String> {
    settings.validate()?;
    match password.as_deref() {
        Some("") => secrets.delete()?,
        Some(_) if settings.username.is_empty() => {
            return Err("A password needs a user name.".to_string());
        }
        Some(secret) => secrets.set(secret)?,
        None => {}
    }
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    config::update(config_dir, |root| {
        root.insert("proxy".to_string(), value);
        Ok(())
    })?;
    Ok(view(settings, secrets))
}

/// Puts the saved settings, with the keychain's password, to work.
fn activate(settings: &ProxySettings, secrets: &dyn SecretStore) {
    let password = if settings.uses_credentials() {
        secrets.get().ok().flatten()
    } else {
        None
    };
    configure(settings.clone(), password);
}

/// Puts the saved download limit to work; running downloads follow it from their next chunk.
pub fn apply_download_limit(config_dir: &Path) -> Result<(), String> {
    let megabytes = config::read(config_dir)?
        .pointer("/download/limitMBps")
        .and_then(|v| v.as_f64());
    limit::shared().set(limit::bytes_per_second(megabytes));
    Ok(())
}

#[tauri::command(async)]
pub fn apply_download_limit_now(app: tauri::AppHandle) -> Result<(), String> {
    apply_download_limit(&config_dir(&app)?)
}

/// Called once at startup: the clients use the saved proxy and limit from their first request.
pub fn load(config_dir: &Path) {
    let _ = apply_download_limit(config_dir);
    match load_settings(config_dir) {
        Ok(settings) => activate(&settings, &Keychain),
        Err(_) => {
            // Unreadable settings must not send requests around a proxy the player may rely on:
            // "system" is the least surprising default, and the page shows the error.
        }
    }
}

// --- Testing a connection -------------------------------------------------------------------

const TEST_URL: &str = "https://api.github.com/zen";

/// A message for the player about a failed request, from the proxy's point of view.
fn explain(error: &ureq::Error, settings: &ProxySettings) -> String {
    use ureq::Error::*;
    let proxy = format!("{}:{}", settings.host.trim(), settings.port);
    let manual = settings.mode == Mode::Manual;
    match error {
        ConnectProxyFailed(reason) if reason.contains("407") => {
            "The proxy wants a user name and password, or refused the ones given.".to_string()
        }
        ConnectProxyFailed(reason) => format!("The proxy refused the connection: {reason}"),
        Timeout(_) => {
            "Nothing answered in time.".to_string()
                + if manual {
                    " Check the proxy's address and port."
                } else {
                    ""
                }
        }
        Io(e) if e.kind() == std::io::ErrorKind::ConnectionRefused && manual => {
            format!("The proxy refused the connection: nothing is listening at {proxy}.")
        }
        Io(e) => format!("Network error: {e}"),
        HostNotFound if manual => format!(
            "The proxy's address can't be found: {}.",
            settings.host.trim()
        ),
        HostNotFound => "A host name can't be found: the network may be down.".to_string(),
        ConnectionFailed if manual => format!("Couldn't connect to the proxy at {proxy}."),
        other => other.to_string(),
    }
}

/// Makes one small request with `settings`, without saving them.
fn check(
    settings: &ProxySettings,
    password: Option<&str>,
    url: &str,
    timeout: Duration,
) -> Result<String, String> {
    settings.validate()?;
    let agent: ureq::Agent = with_proxy(ureq::Agent::config_builder(), settings, password)?
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .build()
        .into();
    match agent.get(url).call() {
        Ok(response) if response.status().as_u16() == 407 => {
            Err("The proxy wants a user name and password, or refused the ones given.".to_string())
        }
        Ok(response) => Ok(format!(
            "Connected: GitHub answered {}.",
            response.status().as_u16()
        )),
        Err(e) => Err(explain(&e, settings)),
    }
}

// --- Commands -------------------------------------------------------------------------------

fn config_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    crate::launcher_settings::config_dir(app)
}

#[tauri::command(async)]
pub fn proxy_settings(app: tauri::AppHandle) -> Result<ProxyView, String> {
    Ok(view(load_settings(&config_dir(&app)?)?, &Keychain))
}

#[tauri::command(async)]
pub fn set_proxy(
    app: tauri::AppHandle,
    settings: ProxySettings,
    password: Option<String>,
) -> Result<ProxyView, String> {
    let view = save(&config_dir(&app)?, &Keychain, settings, password)?;
    activate(&view.settings, &Keychain); // new requests use it from now on
    Ok(view)
}

/// The password a test uses: the typed one; none while the player is about to forget the saved
/// one (`Some("")`); the saved one when nothing was typed.
fn password_for_test(
    typed: Option<String>,
    settings: &ProxySettings,
    saved: impl FnOnce() -> Option<String>,
) -> Option<String> {
    if !settings.uses_credentials() {
        return None;
    }
    match typed {
        Some(typed) if typed.is_empty() => None,
        Some(typed) => Some(typed),
        None => saved(),
    }
}

/// Tries the form's values against GitHub.
#[tauri::command(async)]
pub fn test_proxy(settings: ProxySettings, password: Option<String>) -> Result<String, String> {
    let password = password_for_test(password, &settings, || Keychain.get().ok().flatten());
    check(
        &settings,
        password.as_deref(),
        TEST_URL,
        Duration::from_secs(10),
    )
}

#[cfg(test)]
mod tests;
