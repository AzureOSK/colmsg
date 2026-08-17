use std::{
    collections::{HashMap, HashSet},
    env, fs,
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use clap::ArgMatches;
use serde::Deserialize;
use serde_json::{json, Value};
use tungstenite::{connect, stream::MaybeTlsStream, Error as WebSocketError, Message, WebSocket};

use colmsg::{dirs::PROJECT_DIRS, errors::*};

use crate::app::{H_ACCESS_TOKEN_ENV, N_ACCESS_TOKEN_ENV, S_ACCESS_TOKEN_ENV};

const LOGIN_TIMEOUT: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, Copy)]
enum Service {
    Sakurazaka,
    Hinatazaka,
    Nogizaka,
}

impl Service {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        let groups = matches
            .values_of("group")
            .map(|groups| groups.collect::<Vec<_>>())
            .unwrap_or_else(Vec::new);

        if groups.len() != 1 {
            return Err("web login requires exactly one --group".into());
        }

        match groups[0] {
            "sakurazaka" => Ok(Service::Sakurazaka),
            "hinatazaka" => Ok(Service::Hinatazaka),
            "nogizaka" => Ok(Service::Nogizaka),
            _ => Err("--web-login supports sakurazaka, hinatazaka, and nogizaka".into()),
        }
    }

    fn site(self) -> &'static str {
        match self {
            Service::Sakurazaka => "https://message.sakurazaka46.com/",
            Service::Hinatazaka => "https://message.hinatazaka46.com/",
            Service::Nogizaka => "https://message.nogizaka46.com/",
        }
    }

    fn api_hosts(self) -> [&'static str; 2] {
        match self {
            Service::Sakurazaka => ["api.message.sakurazaka46.com", "api.s46.glastonr.net"],
            Service::Hinatazaka => ["api.message.hinatazaka46.com", "api.kh.glastonr.net"],
            Service::Nogizaka => ["api.message.nogizaka46.com", "api.n46.glastonr.net"],
        }
    }

    fn token_env(self) -> &'static str {
        match self {
            Service::Sakurazaka => S_ACCESS_TOKEN_ENV,
            Service::Hinatazaka => H_ACCESS_TOKEN_ENV,
            Service::Nogizaka => N_ACCESS_TOKEN_ENV,
        }
    }
}

#[derive(Deserialize)]
struct Target {
    #[serde(rename = "type")]
    kind: String,
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_url: Option<String>,
}

struct ChromeProcess(Child);

impl ChromeProcess {
    fn close(&mut self) {
        for _ in 0..20 {
            if let Ok(Some(_)) = self.0.try_wait() {
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Drop for ChromeProcess {
    fn drop(&mut self) {
        if let Ok(None) = self.0.try_wait() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

pub fn login(matches: &ArgMatches) -> Result<(&'static str, String)> {
    let service = Service::from_matches(matches)?;
    let profile_dir = profile_dir()?;

    let port_file = profile_dir.join("DevToolsActivePort");
    if port_file.is_file() {
        fs::remove_file(&port_file)?;
    }

    eprintln!("Opening {}", service.site());
    eprintln!("Complete sign-in in Chrome if prompted.");

    let mut chrome = ChromeProcess(launch_chrome(&profile_dir, "about:blank", true)?);
    let port = wait_for_devtools(&port_file, &mut chrome.0)?;
    let target = wait_for_page_target(port)?;
    let token = capture_access_token(target, service);
    chrome.close();
    let token = token?;

    eprintln!("Authenticated. Starting colmsg.");
    Ok((service.token_env(), token))
}

pub fn setup(matches: &ArgMatches) -> Result<()> {
    let service = Service::from_matches(matches)?;
    let profile_dir = profile_dir()?;

    eprintln!("Opening {} in normal Chrome.", service.site());
    eprintln!("Sign in, confirm that the web client works, then close Chrome.");

    let status = launch_chrome(&profile_dir, service.site(), false)?.wait()?;
    if !status.success() {
        return Err(format!("Google Chrome exited with {}", status).into());
    }

    eprintln!("Setup complete. Run colmsg again with --web-login.");
    Ok(())
}

fn profile_dir() -> Result<PathBuf> {
    let profile_dir = env::var_os("COLMSG_WEB_PROFILE")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PROJECT_DIRS.config_dir().join("web-profile"));
    fs::create_dir_all(&profile_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&profile_dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(profile_dir)
}

fn launch_chrome(profile_dir: &Path, page: &str, remote_debugging: bool) -> Result<Child> {
    let mut last_error = None;
    for executable in chrome_candidates() {
        let mut command = Command::new(&executable);
        if remote_debugging {
            command
                .arg("--remote-debugging-address=127.0.0.1")
                .arg("--remote-debugging-port=0")
                .arg("--remote-allow-origins=*");
        }
        let child = command
            .arg(format!("--user-data-dir={}", profile_dir.display()))
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg(page)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        match child {
            Ok(child) => return Ok(child),
            Err(error) => last_error = Some(error),
        }
    }

    Err(format!(
        "could not start Google Chrome; set COLMSG_CHROME to its path ({})",
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "not found".into())
    )
    .into())
}

fn chrome_candidates() -> Vec<PathBuf> {
    if let Some(path) = env::var_os("COLMSG_CHROME").filter(|path| !path.is_empty()) {
        return vec![PathBuf::from(path)];
    }
    default_chrome_candidates()
}

#[cfg(target_os = "windows")]
fn default_chrome_candidates() -> Vec<PathBuf> {
    ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"]
        .iter()
        .filter_map(|name| env::var_os(name))
        .map(PathBuf::from)
        .map(|path| path.join("Google/Chrome/Application/chrome.exe"))
        .filter(|path| path.is_file())
        .collect()
}

#[cfg(target_os = "macos")]
fn default_chrome_candidates() -> Vec<PathBuf> {
    vec![PathBuf::from(
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    )]
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn default_chrome_candidates() -> Vec<PathBuf> {
    [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}

fn wait_for_devtools(port_file: &Path, child: &mut Child) -> Result<u16> {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(content) = fs::read_to_string(port_file) {
            if let Some(port) = content.lines().next().and_then(|port| port.parse().ok()) {
                return Ok(port);
            }
        }
        if let Some(status) = child.try_wait()? {
            return Err(format!("Google Chrome exited before startup ({})", status).into());
        }
        if Instant::now() >= deadline {
            return Err("timed out waiting for Google Chrome to start".into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn wait_for_page_target(port: u16) -> Result<Target> {
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()?;
    let endpoint = format!("http://127.0.0.1:{}/json/list", port);
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        let target = client
            .get(&endpoint)
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json::<Vec<Target>>())
            .ok()
            .and_then(|targets| {
                targets
                    .into_iter()
                    .find(|target| target.kind == "page" && target.web_socket_url.is_some())
            });
        if let Some(target) = target {
            return Ok(target);
        }
        if Instant::now() >= deadline {
            return Err("could not connect to Google Chrome DevTools".into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn capture_access_token(target: Target, service: Service) -> Result<String> {
    let web_socket_url = target.web_socket_url.unwrap();
    let (mut socket, _) = connect(web_socket_url.as_str())
        .map_err(|error| Error::from(format!("Chrome DevTools connection failed: {}", error)))?;
    set_read_timeout(&mut socket)?;

    send_command(&mut socket, json!({"id": 1, "method": "Network.enable"}))?;
    send_command(
        &mut socket,
        json!({"id": 2, "method": "Page.navigate", "params": {"url": service.site()}}),
    )?;

    let result = listen_for_access_token(&mut socket, service);
    let _ = send_command(&mut socket, json!({"id": 3, "method": "Browser.close"}));
    result
}

fn set_read_timeout(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<()> {
    if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    }
    Ok(())
}

fn send_command(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, command: Value) -> Result<()> {
    socket
        .send(Message::Text(command.to_string()))
        .map_err(|error| Error::from(format!("Chrome DevTools error: {}", error)))
}

fn listen_for_access_token(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    service: Service,
) -> Result<String> {
    let deadline = Instant::now() + LOGIN_TIMEOUT;
    let mut relevant_requests = HashSet::new();
    let mut early_tokens = HashMap::new();

    while Instant::now() < deadline {
        match socket.read() {
            Ok(Message::Text(message)) => {
                if let Ok(event) = serde_json::from_str::<Value>(&message) {
                    if let Some(token) =
                        token_from_event(&event, service, &mut relevant_requests, &mut early_tokens)
                    {
                        return Ok(token);
                    }
                }
            }
            Ok(Message::Ping(payload)) => {
                let _ = socket.send(Message::Pong(payload));
            }
            Ok(_) => {}
            Err(WebSocketError::Io(ref error))
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut => {}
            Err(error) => {
                return Err(format!("Chrome DevTools connection failed: {}", error).into());
            }
        }
    }

    Err("timed out waiting for an authenticated API request".into())
}

fn token_from_event(
    event: &Value,
    service: Service,
    relevant_requests: &mut HashSet<String>,
    early_tokens: &mut HashMap<String, String>,
) -> Option<String> {
    let method = event.get("method")?.as_str()?;
    let params = event.get("params")?;
    let request_id = params.get("requestId")?.as_str()?.to_owned();

    match method {
        "Network.requestWillBeSent" => {
            let request = params.get("request")?;
            if !is_relevant_url(request.get("url")?.as_str()?, service) {
                return None;
            }
            relevant_requests.insert(request_id.clone());
            bearer_token(request.get("headers")?).or_else(|| early_tokens.remove(&request_id))
        }
        "Network.requestWillBeSentExtraInfo" => {
            let token = bearer_token(params.get("headers")?)?;
            if relevant_requests.contains(&request_id) {
                Some(token)
            } else {
                early_tokens.insert(request_id, token);
                None
            }
        }
        _ => None,
    }
}

fn is_relevant_url(value: &str, service: Service) -> bool {
    url::Url::parse(value)
        .ok()
        .and_then(|url| {
            url.host_str()
                .map(|host| service.api_hosts().contains(&host))
        })
        .unwrap_or(false)
}

fn bearer_token(headers: &Value) -> Option<String> {
    headers.as_object()?.iter().find_map(|(name, value)| {
        if !name.eq_ignore_ascii_case("authorization") {
            return None;
        }
        value.as_str()?.strip_prefix("Bearer ").map(String::from)
    })
}
