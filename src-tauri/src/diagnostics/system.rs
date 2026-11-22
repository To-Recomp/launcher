//! Operating system and graphics card names, best effort: a field is absent when unknown.

use std::process::Command;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct System {
    pub family: String,
    pub name: Option<String>,
    pub arch: String,
    pub gpus: Vec<String>,
}

impl System {
    pub fn collect() -> System {
        System {
            family: std::env::consts::OS.to_string(),
            name: os_name(),
            arch: std::env::consts::ARCH.to_string(),
            gpus: gpus(),
        }
    }
}

/// `PRETTY_NAME` of an os-release file.
pub fn parse_os_release(text: &str) -> Option<String> {
    let value = text
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))?;
    let value = value.trim().trim_matches(['"', '\'']).trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn os_name() -> Option<String> {
    if cfg!(target_os = "linux") {
        let text = std::fs::read_to_string("/etc/os-release")
            .or_else(|_| std::fs::read_to_string("/usr/lib/os-release"))
            .ok()?;
        return parse_os_release(&text);
    }
    None
}

/// The display adapters of `lspci -mm` output: `vendor device` of each VGA, 3D or display
/// controller (no revision, no driver).
pub fn parse_lspci(text: &str) -> Vec<String> {
    let mut gpus = Vec::new();
    for line in text.lines() {
        let fields = quoted_fields(line);
        let [class, vendor, device, ..] = fields.as_slice() else {
            continue;
        };
        let class = class.to_ascii_lowercase();
        if !(class.contains("vga")
            || class.contains("3d controller")
            || class.contains("display controller"))
        {
            continue;
        }
        let name = format!("{vendor} {device}");
        if !gpus.contains(&name) {
            gpus.push(name);
        }
    }
    gpus
}

fn quoted_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('"') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('"') else { break };
        fields.push(after[..end].to_string());
        rest = &after[end + 1..];
    }
    fields
}

/// One adapter name per non-empty line.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn parse_lines(text: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if !names.iter().any(|n| n == line) {
            names.push(line.to_string());
        }
    }
    names
}

fn gpus() -> Vec<String> {
    if cfg!(target_os = "linux") {
        return Command::new("lspci")
            .arg("-mm")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| parse_lspci(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or_default();
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        return Command::new("powershell")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(Get-CimInstance Win32_VideoController).Name",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| parse_lines(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or_default();
    }
    #[allow(unreachable_code)]
    Vec::new()
}
