//! Removes what identifies a person or a machine from text that ends up in a public issue.
//!
//! One pure function, [`scrub`]. The player reviews the file but scrubbing is the only privacy
//! barrier, so every rule errs towards removing too much. Applying it twice changes nothing.
//!
//! Known limits (kept honest by tests): a *foreign* user name with spaces that the launcher can't
//! know (`C:\Users\Jane Doe` while running as someone else) loses only its first word; a volume
//! label with spaces keeps its tail; names shorter than 6 characters are only replaced as whole
//! words, so a user named `dev` keeps `device` intact but also leaves `devbox`.

use std::sync::LazyLock;

use regex::{Captures, Match, Regex};

pub const MAX_LINES: usize = 200;
pub const MAX_BYTES: usize = 48 * 1024;

/// A name shorter than this is only replaced as a whole word.
const SUBSTRING_MIN: usize = 6;

/// What the launcher knows about this machine and this game.
#[derive(Debug, Default, Clone)]
pub struct Scrub {
    /// Known paths and the placeholder each becomes (`{library}`, `{data}`, ...).
    pub paths: Vec<(String, String)>,
    /// `$HOME` / `%USERPROFILE%`, which become `~`.
    pub home: Vec<String>,
    pub users: Vec<String>,
    pub hosts: Vec<String>,
}

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid built-in pattern")
}

/// A path segment: stops at separators, whitespace, quotes and our own `<placeholder>` marks.
const SEG: &str = r#"[^\\/\s"'<>|?*:]+"#;

static ANSI: LazyLock<Regex> =
    LazyLock::new(|| regex(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)"));
static WSL_HOME: LazyLock<Regex> = LazyLock::new(|| {
    regex(&format!(
        r"(?i)[\\/]{{2}}wsl(?:\$|\.localhost)[\\/]{SEG}[\\/]home[\\/]{SEG}"
    ))
});
static WINDOWS_USERS: LazyLock<Regex> = LazyLock::new(|| {
    regex(&format!(
        r"(?i)(?:\b[a-z]:|/mnt/[a-z])[\\/]+Users[\\/]+{SEG}"
    ))
});
static UNIX_HOME: LazyLock<Regex> = LazyLock::new(|| regex(&format!(r"(?i)/(?:home|Users)/{SEG}")));
static MEDIA: LazyLock<Regex> =
    LazyLock::new(|| regex(&format!(r"(?i)/(?:run/media|media)/{SEG}/{SEG}")));
static UNC: LazyLock<Regex> = LazyLock::new(|| regex(&format!(r"\\\\{SEG}\\{SEG}")));

static URL_CREDENTIALS: LazyLock<Regex> =
    LazyLock::new(|| regex(r#"(?i)\b([a-z][a-z0-9+.\-]*://)[^/\s@"'<>]+@"#));
static URL_QUERY: LazyLock<Regex> =
    LazyLock::new(|| regex(r#"(?i)(\b[a-z][a-z0-9+.\-]*://[^\s?#"'<>]*)\?[^\s#"'<>]*"#));
static EMAIL: LazyLock<Regex> =
    LazyLock::new(|| regex(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9\-]+(?:\.[A-Za-z0-9\-]+)+"));
static TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    regex(
        r"\bgithub_pat_[A-Za-z0-9_]{20,}|\bgh[pousr]_[A-Za-z0-9]{20,}|\b(?:AKIA|ASIA)[A-Z0-9]{16}\b",
    )
});
static MAC: LazyLock<Regex> =
    LazyLock::new(|| regex(r"\b[0-9A-Fa-f]{2}(?:[:-][0-9A-Fa-f]{2}){5}\b"));
static IPV4: LazyLock<Regex> = LazyLock::new(|| regex(r"\b(?:\d{1,3}\.){3}\d{1,3}\b"));
static IPV6: LazyLock<Regex> = LazyLock::new(|| {
    let h = "[0-9A-Fa-f]{1,4}";
    regex(&format!(
        "(?:{h}:){{7}}{h}|(?:{h}(?::{h}){{0,6}})?::(?:{h}(?::{h}){{0,6}})?"
    ))
});
static HEX_RUN: LazyLock<Regex> = LazyLock::new(|| regex(r"[0-9A-Fa-f]{32,}"));
static BASE64_RUN: LazyLock<Regex> = LazyLock::new(|| regex(r"[A-Za-z0-9+_\-]{32,}={0,2}"));

/// `<user>`, `{data}`, `{input:x}`: text we wrote ourselves, never to be rewritten.
fn inside_placeholder(text: &str, m: &Match) -> bool {
    let before = text[..m.start()].chars().next_back();
    let after = text[m.end()..].chars().next();
    matches!(
        (before, after),
        (Some('<'), Some('>')) | (Some('{'), Some('}')) | (Some('{'), Some(':'))
    )
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn replace_if(text: &str, re: &Regex, with: &str, keep: impl Fn(&str, &Match) -> bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for m in re.find_iter(text) {
        out.push_str(&text[last..m.start()]);
        out.push_str(if keep(text, &m) { m.as_str() } else { with });
        last = m.end();
    }
    out.push_str(&text[last..]);
    out
}

/// Step 0: ANSI sequences, control characters and invisible direction marks.
fn clean(text: &str) -> String {
    ANSI.replace_all(text, "")
        .chars()
        .filter(|&c| {
            let hidden = matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}');
            !hidden && (!c.is_control() || c == '\n' || c == '\t')
        })
        .collect()
}

/// A path as a pattern that accepts either slash, doubled slashes and any letter case.
fn path_regex(path: &str) -> Option<Regex> {
    let path = path.trim_end_matches(['/', '\\']);
    if path.is_empty() {
        return None;
    }
    let parts: Vec<String> = path.split(['/', '\\']).map(regex::escape).collect();
    Regex::new(&format!("(?i){}", parts.join(r"[\\/]+"))).ok()
}

fn replace_path(text: &str, path: &str, with: &str) -> String {
    let Some(re) = path_regex(path) else {
        return text.to_string();
    };
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for m in re.find_iter(text) {
        let before = text[..m.start()].chars().next_back();
        let after = text[m.end()..].chars().next();
        let whole = !before.is_some_and(is_word) && !after.is_some_and(|c| is_word(c) || c == '-');
        out.push_str(&text[last..m.start()]);
        out.push_str(if whole { with } else { m.as_str() });
        last = m.end();
    }
    out.push_str(&text[last..]);
    out
}

fn replace_name(text: &str, name: &str, with: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return text.to_string();
    }
    let whole_only = name.chars().count() < SUBSTRING_MIN;
    let re = regex(&format!("(?i){}", regex::escape(name)));
    replace_if(text, &re, with, |text, m| {
        if inside_placeholder(text, m) {
            return true;
        }
        if whole_only {
            let before = text[..m.start()].chars().next_back();
            let after = text[m.end()..].chars().next();
            return before.is_some_and(is_word) || after.is_some_and(is_word);
        }
        false
    })
}

fn is_secret_run(run: &str) -> bool {
    if run.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }
    // Base64 needs mixed case and a digit: a long lowercase path segment is not a secret.
    run.chars().any(|c| c.is_ascii_digit())
        && run.chars().any(|c| c.is_ascii_lowercase())
        && run.chars().any(|c| c.is_ascii_uppercase())
}

fn valid_ipv4(text: &str) -> bool {
    text.split('.').all(|octet| octet.parse::<u8>().is_ok())
}

fn valid_ipv6(text: &str, m: &Match, whole: &str) -> bool {
    let before = whole[..m.start()].chars().next_back();
    let after = whole[m.end()..].chars().next();
    let edge = |c: Option<char>| !c.is_some_and(|c| is_word(c) || c == ':');
    text.chars().any(|c| c.is_ascii_hexdigit())
        && (text.matches(':').count() >= 2)
        && edge(before)
        && edge(after)
}

fn patterns(text: &str) -> String {
    let text = URL_CREDENTIALS.replace_all(text, "$1");
    let text = URL_QUERY.replace_all(&text, |c: &Captures| c[1].to_string());
    let text = EMAIL.replace_all(&text, "<email>");
    let text = TOKEN.replace_all(&text, "<secret>");
    let text = MAC.replace_all(&text, "<mac>");
    let text = replace_if(&text, &IPV4, "<ip>", |_, m| !valid_ipv4(m.as_str()));
    let text = replace_if(&text, &IPV6, "<ip>", |t, m| !valid_ipv6(m.as_str(), m, t));
    let text = replace_if(&text, &HEX_RUN, "<secret>", |_, _| false);
    replace_if(&text, &BASE64_RUN, "<secret>", |_, m| {
        !is_secret_run(m.as_str())
    })
}

/// Keeps the end of `text`: at most `max_lines` lines and `max_bytes` bytes, cut at a line start.
pub fn limit_tail(text: &str, max_lines: usize, max_bytes: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let kept = lines[lines.len().saturating_sub(max_lines)..].join("\n");
    if kept.len() <= max_bytes {
        return kept;
    }
    let mut start = kept.len() - max_bytes;
    while !kept.is_char_boundary(start) {
        start += 1;
    }
    match kept[start..].find('\n') {
        Some(newline) if kept.as_bytes()[start - 1] != b'\n' => {
            kept[start + newline + 1..].to_string()
        }
        _ => kept[start..].to_string(),
    }
}

/// Scrubs `text` for publication, then keeps its last [`MAX_LINES`] lines / [`MAX_BYTES`] bytes.
pub fn scrub(text: &str, known: &Scrub) -> String {
    let mut text = clean(text);

    // 1. Known paths, longest first.
    let mut paths: Vec<&(String, String)> = known.paths.iter().collect();
    paths.sort_by_key(|(path, _)| std::cmp::Reverse(path.len()));
    for (path, placeholder) in paths {
        text = replace_path(&text, path, placeholder);
    }

    // 2. Home and user areas.
    let mut homes: Vec<&String> = known.home.iter().collect();
    homes.sort_by_key(|home| std::cmp::Reverse(home.len()));
    for home in homes {
        text = replace_path(&text, home, "~");
    }
    text = WSL_HOME.replace_all(&text, "~").into_owned();
    text = WINDOWS_USERS.replace_all(&text, "~").into_owned();
    text = UNIX_HOME.replace_all(&text, "~").into_owned();
    text = MEDIA
        .replace_all(&text, "/run/media/<user>/<volume>")
        .into_owned();
    text = UNC.replace_all(&text, r"\\<host>\<share>").into_owned();

    // 3. Literal user and host names, longest first.
    let mut names: Vec<(&String, &str)> = known
        .users
        .iter()
        .map(|n| (n, "<user>"))
        .chain(known.hosts.iter().map(|n| (n, "<host>")))
        .collect();
    names.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
    for (name, with) in names {
        text = replace_name(&text, name, with);
    }

    // 4. Patterns, then 5. size.
    limit_tail(&patterns(&text), MAX_LINES, MAX_BYTES)
}
