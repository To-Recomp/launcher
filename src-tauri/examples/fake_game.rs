//! A stand-in for a recompiled game, used by the library tests: it behaves like `prepare` and
//! `start` of a release. Not part of the launcher.
//!
//!   --prepare --data D --disc=FILE...   builds D/built.bin from the discs and keeps their
//!                                       content in D/extracted.bin; with no disc it rebuilds
//!                                       from D/extracted.bin. A disc containing FAIL makes it
//!                                       fail, HANG makes it never end.
//!   --play --data D [--exit N]          records its arguments and environment in D/played.txt

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let data = PathBuf::from(value("--data").expect("--data"));
    fs::create_dir_all(&data).unwrap();

    if has("--prepare") {
        println!("progress 0.1 Reading the discs");
        let discs: Vec<&str> = args
            .iter()
            .filter_map(|a| a.strip_prefix("--disc="))
            .collect();
        let content = if discs.is_empty() {
            match fs::read_to_string(data.join("extracted.bin")) {
                Ok(kept) => kept,
                Err(_) => {
                    eprintln!("nothing kept and no disc given");
                    exit(9);
                }
            }
        } else {
            discs
                .iter()
                .map(|d| fs::read_to_string(d).unwrap_or_default())
                .collect::<Vec<_>>()
                .join("+")
        };
        if content.contains("FAIL") {
            eprintln!("disc is unreadable");
            exit(7);
        }
        if content.contains("HANG") {
            println!("progress 0.2 Working");
            loop {
                thread::sleep(Duration::from_millis(100));
            }
        }
        fs::write(data.join("extracted.bin"), &content).unwrap();
        println!("progress 0.6 Compiling");
        fs::write(data.join("built.bin"), format!("built from {content}")).unwrap();
        let env = |n: &str| std::env::var(n).unwrap_or_default();
        fs::write(
            data.join("prepare-env.txt"),
            format!(
                "RECOMPLAUNCHER={}\nVERSION={}\nDATA={}\nTMPDIR={}\nCWD={}\n",
                env("RECOMPLAUNCHER"),
                env("RECOMPLAUNCHER_VERSION"),
                env("RECOMPLAUNCHER_DATA"),
                env("TMPDIR"),
                std::env::current_dir().unwrap().display()
            ),
        )
        .unwrap();
        println!("progress 1 Done");
    } else if has("--play") {
        println!("playing");
        let mut played = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(data.join("played.txt"))
            .unwrap();
        writeln!(
            played,
            "args={}\ndata={}",
            args.join(" "),
            std::env::var("RECOMPLAUNCHER_DATA").unwrap_or_default()
        )
        .unwrap();
        exit(value("--exit").and_then(|c| c.parse().ok()).unwrap_or(0));
    }
}
