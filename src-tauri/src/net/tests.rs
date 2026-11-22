use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

use super::*;

/// A keychain that lives in memory, and can be made to refuse.
#[derive(Default)]
struct Memory {
    secret: RefCell<Option<String>>,
    broken: bool,
}

impl SecretStore for Memory {
    fn get(&self) -> Result<Option<String>, String> {
        if self.broken {
            return Err("The system keychain isn't available".into());
        }
        Ok(self.secret.borrow().clone())
    }
    fn set(&self, secret: &str) -> Result<(), String> {
        if self.broken {
            return Err("The system keychain isn't available".into());
        }
        *self.secret.borrow_mut() = Some(secret.to_string());
        Ok(())
    }
    fn delete(&self) -> Result<(), String> {
        if self.broken {
            return Err("The system keychain isn't available".into());
        }
        *self.secret.borrow_mut() = None;
        Ok(())
    }
}

fn manual(host: &str, port: u16) -> ProxySettings {
    ProxySettings {
        mode: Mode::Manual,
        host: host.into(),
        port,
        ..ProxySettings::default()
    }
}

#[test]
fn a_manual_proxy_needs_an_address_a_port_and_well_formed_exclusions() {
    assert!(manual("proxy.corp", 8080).validate().is_ok());
    assert!(manual("", 8080).validate().is_err());
    assert!(manual("http://proxy", 8080).validate().is_err());
    assert!(manual("proxy.corp", 0).validate().is_err());
    let with = |list: &str| {
        let mut s = manual("proxy.corp", 8080);
        s.no_proxy = list.into();
        s.validate()
    };
    assert!(with("localhost, .corp.example *.lan 10.0.0.1").is_ok());
    assert!(with("*").is_ok());
    assert!(with("bad/entry").is_err());
    assert!(with("..").is_err());
    // Not manual: nothing to check, whatever was typed before.
    let system = ProxySettings {
        host: "???".into(),
        ..ProxySettings::default()
    };
    assert!(system.validate().is_ok());
}

#[test]
fn saving_keeps_the_settings_in_the_file_and_the_password_only_in_the_keychain() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("launcher.json"),
        r#"{ "updates": { "auto": false } }"#,
    )
    .unwrap();
    let keychain = Memory::default();
    let mut settings = manual("proxy.corp", 8080);
    settings.username = "yuri".into();

    let view = save(
        dir.path(),
        &keychain,
        settings.clone(),
        Some("hunter2".into()),
    )
    .unwrap();

    assert!(view.has_password && view.keychain_error.is_none());
    assert_eq!(keychain.secret.borrow().as_deref(), Some("hunter2"));
    let text = std::fs::read_to_string(dir.path().join("launcher.json")).unwrap();
    assert!(
        !text.contains("hunter2"),
        "the password never reaches launcher.json"
    );
    let saved: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["updates"]["auto"], false);
    assert_eq!(saved["proxy"]["host"], "proxy.corp");
    assert_eq!(saved["proxy"]["noProxy"], "");
    assert_eq!(load_settings(dir.path()).unwrap(), settings);

    // `None` keeps it, an empty password forgets it.
    save(dir.path(), &keychain, settings.clone(), None).unwrap();
    assert!(keychain.secret.borrow().is_some());
    let view = save(dir.path(), &keychain, settings, Some(String::new())).unwrap();
    assert!(!view.has_password);
}

#[test]
fn a_password_the_keychain_refuses_stops_the_save_instead_of_going_to_a_file() {
    let dir = TempDir::new().unwrap();
    let keychain = Memory {
        broken: true,
        ..Memory::default()
    };
    let mut settings = manual("proxy.corp", 8080);
    settings.username = "yuri".into();

    let err = save(
        dir.path(),
        &keychain,
        settings.clone(),
        Some("hunter2".into()),
    )
    .unwrap_err();

    assert!(err.contains("keychain"), "{err}");
    assert!(!dir.path().join("launcher.json").exists());
    // Without a password the keychain isn't needed, and a broken one is reported, not fatal.
    let view = save(dir.path(), &keychain, settings, None).unwrap();
    assert!(view.keychain_error.is_some());
}

#[test]
fn a_password_needs_a_user_name_and_invalid_settings_are_not_saved() {
    let dir = TempDir::new().unwrap();
    let keychain = Memory::default();
    assert!(save(
        dir.path(),
        &keychain,
        manual("proxy.corp", 8080),
        Some("x".into())
    )
    .is_err());
    assert!(save(dir.path(), &keychain, manual("", 8080), None).is_err());
    assert!(keychain.secret.borrow().is_none());
    assert!(!dir.path().join("launcher.json").exists());
}

#[test]
fn the_updater_gets_the_same_proxy_with_its_credentials_escaped() {
    let mut s = manual("proxy.corp", 3128);
    s.protocol = Protocol::Socks5;
    s.username = "yu@ri".into();
    s.no_proxy = "localhost .corp".into();
    assert_eq!(
        updater_proxy_of(&s, Some("p:ss/w")),
        UpdaterProxy::Proxy {
            url: "socks5://yu%40ri:p%3Ass%2Fw@proxy.corp:3128".into(),
            no_proxy: "localhost,.corp".into(),
        }
    );
    assert_eq!(
        updater_proxy_of(&ProxySettings::default(), None),
        UpdaterProxy::System
    );
    let direct = ProxySettings {
        mode: Mode::None,
        ..ProxySettings::default()
    };
    assert_eq!(updater_proxy_of(&direct, None), UpdaterProxy::Direct);
}

/// A proxy for tests: every connection gets `reply` to its first request; after a CONNECT that is
/// answered with 200 it acts as the server behind it.
fn fake_proxy(connect_reply: &'static str) -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let heads = Arc::new(Mutex::new(Vec::new()));
    let seen = heads.clone();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let seen = seen.clone();
            thread::spawn(move || serve(stream, connect_reply, seen));
        }
    });
    (port, heads)
}

fn read_head(stream: &mut TcpStream) -> String {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte).unwrap_or(0) == 0 {
            break;
        }
        head.push(byte[0]);
    }
    String::from_utf8_lossy(&head).into_owned()
}

fn serve(mut stream: TcpStream, connect_reply: &str, seen: Arc<Mutex<Vec<String>>>) {
    let head = read_head(&mut stream);
    seen.lock().unwrap().push(head);
    if connect_reply == "silent" {
        thread::sleep(Duration::from_secs(5));
        return;
    }
    let _ = stream.write_all(connect_reply.as_bytes());
    if connect_reply.starts_with("HTTP/1.1 200") {
        let _ = read_head(&mut stream); // the request made through the tunnel
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok");
    }
}

const ESTABLISHED: &str = "HTTP/1.1 200 Connection established\r\n\r\n";
const WANTS_LOGIN: &str = "HTTP/1.1 407 Proxy Authentication Required\r\nContent-Length: 0\r\n\r\n";
const URL: &str = "http://example.test/zen";

#[test]
fn a_request_goes_through_the_proxy_with_the_login_and_skips_it_for_an_excluded_host() {
    let (port, heads) = fake_proxy(ESTABLISHED);
    let mut settings = manual("127.0.0.1", port);
    settings.username = "yuri".into();

    let ok = check(&settings, Some("hunter2"), URL, Duration::from_secs(5)).unwrap();

    assert!(ok.contains("200"), "{ok}");
    let head = heads.lock().unwrap()[0].clone();
    assert!(head.starts_with("CONNECT example.test:80"), "{head}");
    // "yuri:hunter2" in base64
    assert!(head.contains("Basic eXVyaTpodW50ZXIy"), "{head}");

    heads.lock().unwrap().clear();
    settings.no_proxy = "example.test".into();
    let direct = check(&settings, Some("hunter2"), URL, Duration::from_secs(5));
    assert!(
        direct.is_err(),
        "example.test doesn't resolve: the request really went direct"
    );
    assert!(
        heads.lock().unwrap().is_empty(),
        "an excluded host never reaches the proxy"
    );
}

#[test]
fn a_test_says_what_went_wrong_in_words_about_the_proxy() {
    // Refused: nothing listens.
    let free = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let err = check(
        &manual("127.0.0.1", free),
        None,
        URL,
        Duration::from_secs(5),
    )
    .unwrap_err();
    assert!(err.contains("nothing is listening"), "{err}");

    // Authentication.
    let (port, _) = fake_proxy(WANTS_LOGIN);
    let err = check(
        &manual("127.0.0.1", port),
        None,
        URL,
        Duration::from_secs(5),
    )
    .unwrap_err();
    assert!(err.contains("user name and password"), "{err}");

    // Too slow.
    let (port, _) = fake_proxy("silent");
    let err = check(
        &manual("127.0.0.1", port),
        None,
        URL,
        Duration::from_millis(400),
    )
    .unwrap_err();
    assert!(err.contains("in time"), "{err}");

    // An address that is not one is refused before any request.
    let err = check(&manual("", 8080), None, URL, Duration::from_secs(1)).unwrap_err();
    assert!(err.contains("proxy address"), "{err}");
}

#[test]
fn each_mode_gives_the_client_its_proxy_and_a_broken_manual_one_never_goes_direct() {
    let build = |settings: &ProxySettings| {
        with_proxy(ureq::Agent::config_builder(), settings, None).map(|builder| builder.build())
    };
    assert!(build(&manual("127.0.0.1", 8080)).unwrap().proxy().is_some());
    let none = ProxySettings {
        mode: Mode::None,
        ..ProxySettings::default()
    };
    assert!(build(&none).unwrap().proxy().is_none());
    assert!(build(&manual("bad host", 8080)).is_err());
}
