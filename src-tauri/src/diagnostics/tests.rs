use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::scrub::{limit_tail, scrub, Scrub, MAX_BYTES, MAX_LINES};
use super::system::{parse_lines, parse_lspci, parse_os_release, System};
use super::*;

fn known() -> Scrub {
    Scrub {
        paths: vec![
            ("/data/games".into(), "{library}".into()),
            ("/data/games/tos/release".into(), "{install}".into()),
            ("/data/games/tos/instances/gc/data".into(), "{data}".into()),
            ("/mnt/dumps/Tales Disc.iso".into(), "{input:disc}".into()),
            ("/home/alice/.local/share/app".into(), "{app}".into()),
        ],
        home: vec!["/home/alice".into()],
        users: vec!["alice".into()],
        hosts: vec!["fedora-box".into()],
    }
}

fn names(users: &[&str]) -> Scrub {
    Scrub {
        users: users.iter().map(|u| u.to_string()).collect(),
        ..Scrub::default()
    }
}

#[test]
fn scrubs_by_table() {
    let cases: &[(&str, &str)] = &[
        // known paths, longest first, placeholders
        (
            "open /data/games/tos/release/bin/game",
            "open {install}/bin/game",
        ),
        (
            "save /data/games/tos/instances/gc/data/x.sav",
            "save {data}/x.sav",
        ),
        ("see /data/games/other", "see {library}/other"),
        (
            "disc /mnt/dumps/Tales Disc.iso failed",
            "disc {input:disc} failed",
        ),
        (
            "cfg /home/alice/.local/share/app/c.toml",
            "cfg {app}/c.toml",
        ),
        // either slash, doubled slashes
        (r"\data\games\tos\release\x", r"{install}\x"),
        // a path inside a quoted argument
        (
            r#"args: "--rom=/data/games/tos/release/a.bin""#,
            r#"args: "--rom={install}/a.bin""#,
        ),
        (
            r#"run "/home/alice/My Games/x" now"#,
            r#"run "~/My Games/x" now"#,
        ),
        // a sibling that only shares a prefix is not the known path
        ("/data/gamesx/y", "/data/gamesx/y"),
        // home and user areas
        ("/home/bob/file", "~/file"),
        ("/Users/Zoë/Games", "~/Games"),
        (r"C:\Users\BOB\AppData", r"~\AppData"),
        (r"D:/Users/bob/x", "~/x"),
        (r"C:\Users\BOBSMI~1\x", r"~\x"),
        ("/mnt/c/Users/bob/Games", "~/Games"),
        (r"\\wsl$\Ubuntu\home\bob\game", r"~\game"),
        (r"\\wsl.localhost\Ubuntu-22.04\home\bob", "~"),
        // disc image on removable media
        (
            "/run/media/bob/TALES_DISC/game.iso",
            "/run/media/<user>/<volume>/game.iso",
        ),
        ("/media/bob/DISC/a.iso", "/run/media/<user>/<volume>/a.iso"),
        (r"\\nas01\games\tos.iso", r"\\<host>\<share>\tos.iso"),
        // user and host names: spaces, accents, CJK
        ("hello alice!", "hello <user>!"),
        ("login ALICE ok", "login <user> ok"),
        ("on fedora-box now", "on <host> now"),
        // patterns
        ("mail me@example.com now", "mail <email> now"),
        ("peer 192.168.1.20:7777", "peer <ip>:7777"),
        ("peer 10.0.19045.1234", "peer 10.0.19045.1234"),
        ("v 999.1.1.1", "v 999.1.1.1"),
        ("peer 2001:db8::ff00:42:8329 up", "peer <ip> up"),
        ("peer fe80::1 up", "peer <ip> up"),
        ("loop ::1 up", "loop <ip> up"),
        ("std::vector add::Foo", "std::vector add::Foo"),
        ("nic aa:bb:cc:dd:ee:ff", "nic <mac>"),
        ("nic AA-BB-CC-DD-EE-FF", "nic <mac>"),
        ("GET https://x.org/a?token=1&u=2#f", "GET https://x.org/a#f"),
        (
            "git https://bob:hunter2@host.org/r.git",
            "git https://host.org/r.git",
        ),
        (
            "token ghp_abcdefghijklmnopqrstuvwxyz0123456789",
            "token <secret>",
        ),
        ("key AKIAABCDEFGHIJKLMNOP", "key <secret>"),
        (
            "sha 0123456789abcdef0123456789abcdef01234567",
            "sha <secret>",
        ),
        (
            "b64 QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVo0MTIzNDU2Nzg5",
            "b64 <secret>",
        ),
        // long lowercase run (a path segment) is not a secret
        (
            "lib /usr/lib/x86_64-linux-gnu/libvulkan_radeon",
            "lib /usr/lib/x86_64-linux-gnu/libvulkan_radeon",
        ),
        // control characters
        ("a\u{1b}[31mred\u{1b}[0m\u{0}b\r\n", "aredb"),
    ];
    let known = known();
    for (input, expected) in cases {
        let expected = if *input == "a\u{1b}[31mred\u{1b}[0m\u{0}b\r\n" {
            "aredb"
        } else {
            expected
        };
        assert_eq!(scrub(input, &known), expected, "input: {input:?}");
    }
}

#[test]
fn user_names_with_spaces_accents_and_cjk() {
    let known = Scrub {
        home: vec![r"C:\Users\Jean Müller".into(), "/home/李 雷".into()],
        users: vec!["Jean Müller".into(), "李 雷".into(), "李雷".into()],
        ..Scrub::default()
    };
    assert_eq!(
        scrub(r"C:\Users\Jean Müller\Documents\x", &known),
        r"~\Documents\x"
    );
    assert_eq!(scrub("/home/李 雷/games", &known), "~/games");
    assert_eq!(scrub("user=Jean Müller", &known), "user=<user>");
    assert_eq!(scrub("owner 李雷 here", &known), "owner <user> here");
    assert_eq!(scrub("c:/users/jean müller/x", &known), "~/x");
}

#[test]
fn a_foreign_user_name_with_spaces_loses_only_its_first_word() {
    // Documented limitation: only the running user's own name is known.
    assert_eq!(
        scrub("/Users/Zoë Müller/Games", &Scrub::default()),
        "~ Müller/Games"
    );
}

#[test]
fn a_user_named_like_a_common_word_keeps_longer_words() {
    let dev = names(&["dev"]);
    // A path segment equal to the name is replaced, as is any whole word.
    assert_eq!(
        scrub("device /dev/dri/card0 by dev", &dev),
        "device /<user>/dri/card0 by <user>"
    );
    assert_eq!(
        scrub("devices and dev_tools", &dev),
        "devices and dev_tools"
    );
    // Documented limitation: a short name is not found inside a longer word.
    assert_eq!(scrub("devbox", &dev), "devbox");
    // A name of 6+ characters is replaced inside words too.
    let long = names(&["johnny"]);
    assert_eq!(scrub("johnnyboy-pc", &long), "<user>boy-pc");
}

#[test]
fn placeholders_are_never_rewritten() {
    let k = names(&["data", "library", "user", "input"]);
    assert_eq!(
        scrub("{data}/x {library}/y <user> {input:a}", &k),
        "{data}/x {library}/y <user> {input:a}"
    );
}

#[test]
fn scrubbing_twice_changes_nothing() {
    let inputs = [
        "open /data/games/tos/release/bin/game for alice on fedora-box",
        "/run/media/bob/TALES_DISC/game.iso and /media/bob/D/x",
        r"\\nas01\games\tos.iso C:\Users\BOB\x /mnt/c/Users/b/y",
        "me@example.com 192.168.1.20 ::1 aa:bb:cc:dd:ee:ff https://u:p@h.org/a?q=1",
        "ghp_abcdefghijklmnopqrstuvwxyz0123456789 0123456789abcdef0123456789abcdef",
        "user user user host host",
    ];
    let mut known = known();
    known.users.push("user".into());
    known.hosts.push("host".into());
    for input in inputs {
        let once = scrub(input, &known);
        assert_eq!(scrub(&once, &known), once, "input: {input:?}");
    }
}

#[test]
fn size_caps_keep_the_end() {
    let text: String = (0..500).map(|i| format!("line {i}\n")).collect();
    let tail = scrub(&text, &Scrub::default());
    assert_eq!(tail.lines().count(), MAX_LINES);
    assert!(tail.ends_with("line 499"));
    assert!(tail.starts_with("line 300"));

    let big: String = (0..2000)
        .map(|i| format!("{i:04} {}\n", "x".repeat(60)))
        .collect();
    let capped = limit_tail(&big, 10_000, 4096);
    assert!(capped.len() <= 4096);
    assert!(capped.ends_with(&format!("1999 {}", "x".repeat(60))));
    assert!(capped.lines().all(|l| l.len() == 65), "cut at a line start");

    let one_line = "é".repeat(5000);
    let capped = limit_tail(&one_line, 10, 1001);
    assert!(capped.len() <= 1001 && capped.chars().all(|c| c == 'é'));
}

fn facts<'a>(
    system: &'a System,
    game: Option<&'a Game>,
    error: &'a str,
    log: Option<(&'a str, &'a str)>,
) -> Facts<'a> {
    Facts {
        kind: Kind::Prepare,
        game,
        error,
        log,
        system,
        version: "0.1.0",
        build: "release",
    }
}

fn linux() -> System {
    System {
        family: "linux".into(),
        name: Some("Fedora Linux 45".into()),
        arch: "x86_64".into(),
        gpus: vec!["AMD Radeon RX 7800 XT".into()],
    }
}

#[test]
fn renders_the_fixed_template() {
    let game = Game {
        id: "tos".into(),
        release: Some("v1.2.0".into()),
        platform: "GameCube".into(),
        state: "ready".into(),
    };
    let system = linux();
    let text = render(
        &facts(
            &system,
            Some(&game),
            "Failed in /home/alice/x",
            Some(("prepare.log", "a\nb\n/home/alice/y")),
        ),
        &known(),
    );
    assert!(text.starts_with("Diagnostic format: 1\n"));
    for row in [
        "| Launcher | 0.1.0 (release) |",
        "| OS | Fedora Linux 45 |",
        "| GPU | AMD Radeon RX 7800 XT |",
        "| Game | tos |",
        "| Release | v1.2.0 |",
        "| State | ready |",
        "| Context | prepare |",
    ] {
        assert!(text.contains(row), "missing {row}\n{text}");
    }
    assert!(text.contains("## Error\n\n```text\nFailed in ~/x\n```"));
    assert!(text.contains("<details><summary>prepare.log, last 3 lines</summary>"));
    assert!(text.contains("~/y\n```"));
    assert!(!text.contains("alice"));
}

#[test]
fn install_failure_has_the_message_alone() {
    let system = System {
        gpus: vec![],
        name: None,
        ..linux()
    };
    let text = render(
        &facts(&system, None, "Download failed", None),
        &Scrub::default(),
    );
    assert!(text.contains("Download failed"));
    assert!(!text.contains("<details>"));
    assert!(!text.contains("| GPU |"));
    assert!(!text.contains("| OS |"));
}

#[test]
fn a_log_cannot_break_out_of_its_fence_or_the_table() {
    let game = Game {
        id: "tos".into(),
        state: "a | b\nc".into(),
        ..Game::default()
    };
    let system = linux();
    let text = render(
        &facts(
            &system,
            Some(&game),
            "```\n# hi @everyone",
            Some(("game.log", "````\n</details>")),
        ),
        &Scrub::default(),
    );
    assert!(text.contains("| State | a / b c |"));
    assert!(text.contains("````text\n```\n# hi @everyone\n````"));
    assert!(text.contains("`````text\n````\n</details>\n`````"));
}

#[test]
fn the_whole_file_stays_under_the_cap() {
    let game = Game {
        id: "tos".into(),
        ..Game::default()
    };
    let system = linux();
    let long_line = "y".repeat(10_000);
    let log: String = (0..200).map(|_| format!("{long_line}\n")).collect();
    let error = "e".repeat(100_000);
    let text = render(
        &facts(&system, Some(&game), &error, Some(("game.log", &log))),
        &Scrub::default(),
    );
    assert!(text.len() <= MAX_BYTES, "{} bytes", text.len());
    assert!(text.contains("<details>"), "the log still gets its room");
}

fn at(secs: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(secs)
}

#[test]
fn file_names_carry_the_game_and_the_date() {
    let dir = tempfile::tempdir().unwrap();
    // 2026-11-22 15:40:45 UTC
    let path = write_file(dir.path(), "tos", "hello", at(1_795_362_045)).unwrap();
    assert_eq!(
        path.file_name().unwrap(),
        "tos-diagnostic-2026-11-22-154045.md"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), "hello");
    let again = write_file(dir.path(), "tos", "second", at(1_795_362_045)).unwrap();
    assert_eq!(
        again.file_name().unwrap(),
        "tos-diagnostic-2026-11-22-154045-2.md"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "hello",
        "never overwrites"
    );
    let manual = write_file(dir.path(), "../evil", "x", at(0)).unwrap();
    assert_eq!(
        manual.file_name().unwrap(),
        "evil-diagnostic-1970-01-01-000000.md"
    );
}

#[test]
fn keeps_the_last_ten_files_and_nothing_else_is_touched() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("game.log"), "keep").unwrap();
    fs::write(dir.path().join("notes.md"), "keep").unwrap();
    let mut written = Vec::new();
    for i in 0..13u64 {
        let base = if i % 2 == 0 { "tos" } else { "launcher" };
        let path = write_file(dir.path(), base, "x", at(1_000_000 + i * 60)).unwrap();
        let file = fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(at(1_000_000 + i * 60)).unwrap();
        written.push(path);
    }
    let left: Vec<_> = written.iter().filter(|p| p.exists()).collect();
    assert_eq!(left.len(), KEEP_FILES);
    assert!(
        written[..3].iter().all(|p| !p.exists()),
        "the oldest went first"
    );
    assert!(written[3..].iter().all(|p| p.exists()));
    assert!(dir.path().join("game.log").exists() && dir.path().join("notes.md").exists());
}

#[test]
fn copied_text_is_the_file_content() {
    let dir = tempfile::tempdir().unwrap();
    let system = linux();
    let text = render(
        &facts(&system, None, "é — 日本語 error", None),
        &Scrub::default(),
    );
    let path = write_file(dir.path(), "launcher", &text, at(5)).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), text);
}

#[test]
fn reads_system_names() {
    assert_eq!(
        parse_os_release(
            "NAME=\"Fedora Linux\"\nPRETTY_NAME=\"Fedora Linux 45 (Workstation Edition)\"\n"
        )
        .as_deref(),
        Some("Fedora Linux 45 (Workstation Edition)")
    );
    assert_eq!(parse_os_release("NAME=x\n"), None);
    let lspci = concat!(
        "00:02.0 \"VGA compatible controller\" \"Intel Corporation\" \"Alder Lake-P GT2\" -r0c -p00 \"Dell\" \"0a\"\n",
        "01:00.0 \"3D controller\" \"NVIDIA Corporation\" \"GA107M [RTX 3050]\" -ra1 \"Dell\" \"0b\"\n",
        "00:1f.3 \"Audio device\" \"Intel Corporation\" \"Alder Lake PCH\" -r01\n",
    );
    assert_eq!(
        parse_lspci(lspci),
        [
            "Intel Corporation Alder Lake-P GT2",
            "NVIDIA Corporation GA107M [RTX 3050]"
        ]
    );
    assert_eq!(parse_lines("A\r\n\nB\nA\n"), ["A", "B"]);
}

mod panic_file {
    use super::*;
    use crate::diagnostics::panic::{dismiss, pending, record};

    #[test]
    fn a_panic_leaves_a_scrubbed_file_and_a_pending_marker_until_dismissed() {
        let dir = tempfile::tempdir().unwrap();
        assert!(pending(dir.path()).is_none());
        let path = record(
            dir.path(),
            &known(),
            "The launcher panicked: no such file /home/alice/secret.iso\nat /home/alice/.cargo/x.rs:1:2",
            at(10),
        )
        .unwrap();
        assert_eq!(
            path.file_name().unwrap(),
            "launcher-diagnostic-1970-01-01-000010.md"
        );

        let found = pending(dir.path()).expect("pending after a panic");
        assert_eq!(found.text, fs::read_to_string(&path).unwrap());
        assert!(found.text.contains("| Context | launcher |"));
        assert!(found.text.contains("~/secret.iso") && !found.text.contains("alice"));
        assert!(!found.text.contains("| Game |"));

        dismiss(dir.path());
        assert!(pending(dir.path()).is_none());
        assert!(path.exists(), "dismissing keeps the file");
    }

    #[test]
    fn a_pending_file_that_was_rotated_away_offers_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = record(dir.path(), &Scrub::default(), "boom", at(1)).unwrap();
        fs::remove_file(path).unwrap();
        assert!(pending(dir.path()).is_none());
        assert!(!dir.path().join("panic-pending").exists());
    }

    #[test]
    fn the_marker_cannot_point_outside_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        fs::write(outside.path(), "private").unwrap();
        fs::write(
            dir.path().join("panic-pending"),
            outside.path().to_string_lossy().as_ref(),
        )
        .unwrap();
        assert!(pending(dir.path()).is_none());
    }

    const CHILD: &str = "LAUNCHER_TEST_PANIC_DIR";

    /// The hook is process-global, so it runs in a child process: the same test, re-executed.
    #[test]
    fn the_installed_hook_writes_the_file_of_the_first_panic_only() {
        if let Ok(dir) = std::env::var(CHILD) {
            crate::diagnostics::panic::arm(dir.into(), Scrub::default());
            crate::diagnostics::panic::install();
            let _ = std::panic::catch_unwind(|| panic!("first boom"));
            let _ = std::panic::catch_unwind(|| panic!("second boom"));
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "diagnostics::tests::panic_file::the_installed_hook_writes_the_file_of_the_first_panic_only"])
            .env(CHILD, dir.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let found = pending(dir.path()).expect("the hook wrote a file");
        assert!(
            found.text.contains("The launcher panicked: first boom"),
            "{}",
            found.text
        );
        assert!(found.text.contains("tests.rs"));
        assert!(!found.text.contains("second boom"));
        let files = fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
            .count();
        assert_eq!(files, 1);
    }
}

#[test]
fn cleaning_removes_the_reports_and_nothing_else_in_the_folder() {
    let dir = tempfile::TempDir::new().unwrap();
    let outside = tempfile::TempDir::new().unwrap();
    let keep_file = |name: &str| std::fs::write(dir.path().join(name), "keep").unwrap();
    std::fs::write(
        dir.path().join("launcher-diagnostic-2026-11-22.md"),
        "12345",
    )
    .unwrap();
    std::fs::write(dir.path().join("toe-diagnostic-2026-11-21-2.md"), "123").unwrap();
    std::fs::write(
        dir.path().join("panic-pending"),
        "launcher-diagnostic-2026-11-22.md",
    )
    .unwrap();
    keep_file("notes.md");
    keep_file("launcher.log");
    std::fs::create_dir(dir.path().join("sub")).unwrap();
    std::fs::write(
        dir.path().join("sub").join("x-diagnostic-1.md"),
        "kept: below the folder",
    )
    .unwrap();
    let precious = outside.path().join("precious-diagnostic-1.md");
    std::fs::write(&precious, "outside").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&precious, dir.path().join("link-diagnostic-1.md")).unwrap();

    let cleaned = clean(dir.path());

    assert_eq!(cleaned, Cleaned { files: 2, bytes: 8 });
    let mut left: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    let expected = if cfg!(unix) {
        vec!["launcher.log", "link-diagnostic-1.md", "notes.md", "sub"]
    } else {
        vec!["launcher.log", "notes.md", "sub"]
    };
    assert_eq!(left, expected);
    assert!(dir.path().join("sub/x-diagnostic-1.md").exists());
    assert!(precious.exists(), "a link is never followed");
}

#[test]
fn cleaning_a_folder_that_does_not_exist_removes_nothing() {
    let dir = tempfile::TempDir::new().unwrap();
    assert_eq!(
        clean(&dir.path().join("missing")),
        Cleaned { files: 0, bytes: 0 }
    );
}
