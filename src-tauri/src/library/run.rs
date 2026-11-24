//! Running a release's `prepare` and `start` commands.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::settings::manifest::Manifest;
use crate::settings::Instance;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Progress {
    /// 0 to 1, when the step reports one.
    pub fraction: Option<f64>,
    pub message: String,
    /// A line of the step's own list of tasks (`stage-start`, `stage-progress`, `stage-end`).
    pub stage: Option<Stage>,
}

/// One task of the step's list, as the step reports it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stage {
    pub id: String,
    #[serde(flatten)]
    pub event: StageEvent,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum StageEvent {
    /// `stage-start extract 2/5 Copying the game files`
    Start { index: u32, count: u32, title: String },
    /// `stage-progress extract 734003200/2919956480 Copying the files of disc 1`
    Progress { done: u64, total: u64, text: String },
    /// `stage-end extract ok 41.3` (`ok`, `skipped`, `failed` or `cancelled`, then the seconds)
    End { status: String, seconds: f64 },
}

/// What a command line's placeholders are filled from.
pub struct Context<'a> {
    pub location: &'a Instance,
    pub manifest: &'a Manifest,
    /// The player's files by input id.
    pub inputs: &'a BTreeMap<String, PathBuf>,
    /// A rebuild from what a previous run kept: the player's copy isn't passed.
    pub without_copy: bool,
}

/// Fills in one argument or value. `None` means the argument is left out: it names an optional
/// file the player didn't give, or a copy the rebuild doesn't need.
pub fn expand(template: &str, ctx: &Context) -> Result<Option<String>, String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find("{input:") {
        out.push_str(&ctx.location.expand_text(&rest[..start])?);
        let after = &rest[start + "{input:".len()..];
        let end = after
            .find('}')
            .ok_or_else(|| format!("`{template}` has an unclosed placeholder"))?;
        let id = &after[..end];
        let input = ctx
            .manifest
            .inputs
            .iter()
            .find(|i| i.id == id)
            .ok_or_else(|| format!("`{template}` refers to the unknown input `{id}`"))?;
        if ctx.without_copy && input.is_copy() {
            return Ok(None);
        }
        match ctx.inputs.get(id) {
            Some(path) => out.push_str(&path.to_string_lossy()),
            None if input.optional => return Ok(None),
            None => {
                let what = input.label.as_deref().unwrap_or(id);
                return Err(format!("Missing file: {what}."));
            }
        }
        rest = &after[end + 1..];
    }
    out.push_str(&ctx.location.expand_text(rest)?);
    Ok(Some(out))
}

pub fn expand_all(templates: &[String], ctx: &Context) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for template in templates {
        if let Some(arg) = expand(template, ctx)? {
            out.push(arg);
        }
    }
    Ok(out)
}

/// The first program of the list that exists in the release.
pub fn find_program(location: &Instance, programs: &[String]) -> Result<PathBuf, String> {
    for program in programs {
        let path = location.resolve(program)?;
        let path = if path.is_absolute() {
            path
        } else {
            location.install.join(path)
        };
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(format!(
        "This release has none of the programs it names ({}).",
        programs.join(", ")
    ))
}

pub struct Launch {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

/// Program, arguments and environment of a `start` or `prepare` entry, with the variables every
/// game the launcher runs gets.
pub fn build(
    ctx: &Context,
    program: &[String],
    args: &[String],
    env: &BTreeMap<String, String>,
    launcher_version: &str,
) -> Result<Launch, String> {
    let mut vars = vec![
        ("RECOMPLAUNCHER".to_string(), "1".to_string()),
        (
            "RECOMPLAUNCHER_VERSION".to_string(),
            launcher_version.to_string(),
        ),
        ("RECOMPLAUNCHER_MODE".to_string(), "desktop".to_string()),
        (
            "RECOMPLAUNCHER_DATA".to_string(),
            ctx.location.data.to_string_lossy().into_owned(),
        ),
    ];
    for (name, template) in env {
        if let Some(value) = expand(template, ctx)? {
            vars.push((name.clone(), value));
        }
    }
    Ok(Launch {
        program: find_program(ctx.location, program)?,
        args: expand_all(args, ctx)?,
        env: vars,
    })
}

#[derive(Debug, PartialEq)]
pub enum RunError {
    Cancelled,
    Failed(String),
}

/// `progress 0.4 Unpacking menu art`
pub fn parse_progress(line: &str) -> Option<Progress> {
    let rest = line.trim().strip_prefix("progress ")?;
    let (number, message) = rest.split_once(' ').unwrap_or((rest, ""));
    let fraction: f64 = number.parse().ok()?;
    Some(Progress {
        fraction: Some(fraction.clamp(0.0, 1.0)),
        message: message.trim().to_string(),
        stage: None,
    })
}

/// `a/b` as two numbers.
fn ratio<T: std::str::FromStr>(text: &str) -> Option<(T, T)> {
    let (a, b) = text.split_once('/')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// The task lines of a step that reports a list of tasks:
/// `stage-start <id> <i>/<n> <title>`, `stage-progress <id> <done>/<total> <text>`,
/// `stage-end <id> <status> <seconds>`.
pub fn parse_stage(line: &str) -> Option<Progress> {
    let line = line.trim();
    let (kind, rest) = line.split_once(' ')?;
    let (id, rest) = rest.split_once(' ')?;
    let (second, text) = rest.split_once(' ').unwrap_or((rest, ""));
    let event = match kind {
        "stage-start" => {
            let (index, count) = ratio(second)?;
            StageEvent::Start { index, count, title: text.trim().to_string() }
        }
        "stage-progress" => {
            let (done, total) = ratio(second)?;
            StageEvent::Progress { done, total, text: text.trim().to_string() }
        }
        "stage-end" => StageEvent::End {
            status: second.to_string(),
            seconds: text.trim().parse().ok()?,
        },
        _ => return None,
    };
    Some(Progress {
        stage: Some(Stage { id: id.to_string(), event }),
        ..Progress::default()
    })
}

/// What a line of a step's output reports, if anything.
pub fn parse_line(line: &str) -> Option<Progress> {
    parse_progress(line).or_else(|| parse_stage(line))
}

/// `result failed 3 This disc is not a known dump`: the step's own explanation of a failure.
/// Returns the text of a `failed` result line (the exit code is the process's).
pub fn parse_failure(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("result failed ")?;
    let (code, message) = rest.split_once(' ')?;
    code.parse::<i32>().ok()?;
    let message = message.trim();
    (!message.is_empty()).then(|| message.to_string())
}

fn pump(mut reader: impl Read, tx: mpsc::Sender<String>) {
    let mut reader = BufReader::new(&mut reader);
    let mut line = Vec::new();
    // Lossy on purpose: a program printing invalid UTF-8 must not stall on a full pipe.
    while matches!(reader.read_until(b'\n', &mut line), Ok(n) if n > 0) {
        if tx
            .send(String::from_utf8_lossy(&line).trim_end().to_string())
            .is_err()
        {
            return;
        }
        line.clear();
    }
}

fn kill_tree(child: &mut Child) {
    #[cfg(unix)]
    // SAFETY: killpg only signals the process group the child leads.
    unsafe {
        libc::killpg(child.id() as libc::pid_t, libc::SIGKILL);
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .output();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn command(cmd: &Launch, cwd: &Path) -> Command {
    let mut process = Command::new(&cmd.program);
    process
        .args(&cmd.args)
        .envs(cmd.env.iter().cloned())
        .current_dir(cwd)
        .stdin(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        process.process_group(0); // so cancelling can stop the compiler it started, too
    }
    process
}

fn log_line(log: &mut File, text: &str) {
    let _ = writeln!(log, "{text}");
}

/// Runs the step with no window, reporting its `progress` lines, until it ends or `cancel`
/// is set. Everything it prints goes to `log`. `tmp` is emptied before and removed after.
pub fn run_prepare(
    cmd: &Launch,
    install: &Path,
    tmp: &Path,
    log_path: &Path,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<(), RunError> {
    let fail = |m: String| RunError::Failed(m);
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).map_err(|e| fail(format!("Can't create {}: {e}", tmp.display())))?;
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| fail(format!("Can't create {}: {e}", parent.display())))?;
    }
    let mut log = File::create(log_path)
        .map_err(|e| fail(format!("Can't write {}: {e}", log_path.display())))?;
    log_line(
        &mut log,
        &format!("$ {} {}", cmd.program.display(), cmd.args.join(" ")),
    );

    let mut process = command(cmd, install);
    for name in ["TMPDIR", "TEMP", "TMP"] {
        process.env(name, tmp);
    }
    let result = run_to_end(process, &mut log, cancel, on_progress);
    let _ = fs::remove_dir_all(tmp);
    match &result {
        Ok(()) => log_line(&mut log, "-- finished"),
        Err(RunError::Cancelled) => log_line(&mut log, "-- cancelled"),
        Err(RunError::Failed(m)) => log_line(&mut log, &format!("-- {m}")),
    }
    result
}

fn run_to_end(
    mut process: Command,
    log: &mut File,
    cancel: &AtomicBool,
    on_progress: &mut dyn FnMut(Progress),
) -> Result<(), RunError> {
    process.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = process
        .spawn()
        .map_err(|e| RunError::Failed(format!("Can't start the preparation: {e}")))?;
    let (tx, rx) = mpsc::channel();
    if let Some(out) = child.stdout.take() {
        let tx = tx.clone();
        thread::spawn(move || pump(out, tx));
    }
    if let Some(err) = child.stderr.take() {
        let tx = tx.clone();
        thread::spawn(move || pump(err, tx));
    }
    drop(tx);

    let mut explained: Option<String> = None;
    let mut handle = |line: String, log: &mut File| {
        log_line(log, &line);
        if let Some(why) = parse_failure(&line) {
            explained = Some(why);
        }
        if let Some(progress) = parse_line(&line) {
            on_progress(progress);
        }
    };
    loop {
        if cancel.load(Ordering::Relaxed) {
            kill_tree(&mut child);
            return Err(RunError::Cancelled);
        }
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(line) => handle(line, log),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // The program is gone but something it started still holds the pipes.
                if matches!(child.try_wait(), Ok(Some(_))) {
                    while let Ok(line) = rx.recv_timeout(Duration::from_millis(200)) {
                        handle(line, log);
                    }
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let status = child.wait().map_err(|e| RunError::Failed(e.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        let code = status.code().map_or("none".to_string(), |c| c.to_string());
        Err(RunError::Failed(match explained {
            Some(why) => format!(
                "The preparation failed: {why} (exit code {code}). The log has the details."
            ),
            None => format!("The preparation failed (exit code {code}). The log has the details."),
        }))
    }
}

/// Starts the game with its output going to `log_path`.
pub fn spawn_game(cmd: &Launch, install: &Path, log_path: &Path) -> Result<Child, String> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Can't create {}: {e}", parent.display()))?;
    }
    let mut log =
        File::create(log_path).map_err(|e| format!("Can't write {}: {e}", log_path.display()))?;
    log_line(
        &mut log,
        &format!("$ {} {}", cmd.program.display(), cmd.args.join(" ")),
    );
    let out = log.try_clone().map_err(|e| e.to_string())?;
    let err = log.try_clone().map_err(|e| e.to_string())?;
    command(cmd, install)
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .spawn()
        .map_err(|e| format!("Can't start the game: {e}"))
}

/// Adds a line to a log (used when the game ends).
pub fn append_log(path: &Path, text: &str) {
    if let Ok(mut log) = OpenOptions::new().append(true).open(path) {
        log_line(&mut log, text);
    }
}

/// The last `max_lines` lines of a log, or nothing if it doesn't exist.
pub fn tail(path: &Path, max_lines: usize) -> String {
    let Ok(text) = fs::read_to_string(path) else {
        return String::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(max_lines)..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress_lines() {
        assert_eq!(
            parse_progress("progress 0.4 Unpacking menu art"),
            Some(Progress {
                fraction: Some(0.4),
                message: "Unpacking menu art".into(),
                stage: None,
            })
        );
        assert_eq!(
            parse_progress("progress 7 big").unwrap().fraction,
            Some(1.0)
        );
        assert_eq!(parse_progress("progress -1").unwrap().fraction, Some(0.0));
        assert_eq!(parse_progress("progress 0.5").unwrap().message, "");
        assert_eq!(parse_progress("progress soon"), None);
        assert_eq!(parse_progress("compiling progress 0.5"), None);
    }

    #[test]
    fn parses_the_task_lines_of_a_step() {
        let stage = |line| parse_line(line).and_then(|p| p.stage);
        assert_eq!(
            stage("stage-start extract 2/5 Copying the game files from the discs"),
            Some(Stage {
                id: "extract".into(),
                event: StageEvent::Start { index: 2, count: 5, title: "Copying the game files from the discs".into() }
            })
        );
        assert_eq!(
            stage("stage-progress compile 12/37 Compiling the game (12/37 files)"),
            Some(Stage {
                id: "compile".into(),
                event: StageEvent::Progress { done: 12, total: 37, text: "Compiling the game (12/37 files)".into() }
            })
        );
        assert_eq!(
            stage("stage-end verify ok 36.6"),
            Some(Stage { id: "verify".into(), event: StageEvent::End { status: "ok".into(), seconds: 36.6 } })
        );
        // A task line carries no overall fraction: it must not reset the bar.
        assert_eq!(parse_line("stage-end verify ok 1.0").unwrap().fraction, None);
        assert_eq!(parse_line("stage-start extract soon x"), None);
        assert_eq!(parse_line("stage-end verify ok slowly"), None);
        assert_eq!(parse_line("warn something"), None);
    }

    #[test]
    fn parses_a_failed_result_line() {
        assert_eq!(
            parse_failure("result failed 3 this disc is not a known dump"),
            Some("this disc is not a known dump".into())
        );
        assert_eq!(parse_failure("result ready 0 the game is prepared"), None);
        assert_eq!(parse_failure("result failed 3"), None);
        assert_eq!(parse_failure("result failed x oops"), None);
    }

    #[test]
    fn keeps_the_last_lines_of_a_log() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("l.log");
        fs::write(&path, "a\nb\nc\nd\n").unwrap();
        assert_eq!(tail(&path, 2), "c\nd");
        assert_eq!(tail(&path, 10), "a\nb\nc\nd");
        assert_eq!(tail(&dir.path().join("none.log"), 3), "");
    }
}
