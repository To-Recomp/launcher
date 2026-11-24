//! A disk cache for the GitHub API's GET answers, so the launcher stays far from the 60
//! requests an hour GitHub allows without a token. One file per URL holds the body, its ETag and
//! when it was fetched. Within the TTL the saved body is served without any request; after it a
//! conditional request is made (a `304` costs no quota). When GitHub can't be asked (rate limit,
//! network), the saved body is served even if it is old.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::fs_util::write_atomically;

pub const TTL: Duration = Duration::from_secs(15 * 60);

/// What GitHub answered, errors aside.
pub enum Reply {
    Body { text: String, etag: Option<String> },
    NotModified,
    NotFound,
}

#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    etag: Option<String>,
    /// Seconds since the Unix epoch.
    fetched_at: u64,
    body: String,
}

pub struct Cache {
    /// `None`: nothing is saved, every call asks GitHub.
    dir: Option<PathBuf>,
    ttl: Duration,
    now: Box<dyn Fn() -> u64 + Send + Sync>,
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl Cache {
    pub fn in_dir(dir: PathBuf) -> Cache {
        Cache::with_clock(Some(dir), TTL, Box::new(unix_now))
    }

    pub fn disabled() -> Cache {
        Cache::with_clock(None, TTL, Box::new(unix_now))
    }

    pub fn with_clock(
        dir: Option<PathBuf>,
        ttl: Duration,
        now: Box<dyn Fn() -> u64 + Send + Sync>,
    ) -> Cache {
        Cache { dir, ttl, now }
    }

    fn path(&self, url: &str) -> Option<PathBuf> {
        let name: String = Sha256::digest(url.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Some(self.dir.as_ref()?.join(format!("{name}.json")))
    }

    fn load(&self, url: &str) -> Option<Entry> {
        let text = fs::read_to_string(self.path(url)?).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// A cache that can't be written still serves from GitHub, so failures are ignored.
    fn store(&self, url: &str, entry: &Entry) {
        if let (Some(path), Ok(text)) = (self.path(url), serde_json::to_string(entry)) {
            let _ = write_atomically(&path, &text);
        }
    }

    /// The body of `url`. `fetch` asks GitHub, given the saved ETag. `fresh` skips the TTL (the
    /// ETag is still sent). `Ok(None)`: GitHub says it doesn't exist.
    pub fn get(
        &self,
        url: &str,
        fresh: bool,
        fetch: impl FnOnce(Option<&str>) -> Result<Reply, String>,
    ) -> Result<Option<String>, String> {
        let now = (self.now)();
        let entry = self.load(url);
        if let (false, Some(e)) = (fresh, &entry) {
            let young = now
                .checked_sub(e.fetched_at)
                .is_some_and(|age| age < self.ttl.as_secs());
            if young {
                return Ok(Some(e.body.clone()));
            }
        }
        match fetch(entry.as_ref().and_then(|e| e.etag.as_deref())) {
            Ok(Reply::Body { text, etag }) => {
                self.store(
                    url,
                    &Entry {
                        etag,
                        fetched_at: now,
                        body: text.clone(),
                    },
                );
                Ok(Some(text))
            }
            Ok(Reply::NotModified) => match entry {
                Some(mut e) => {
                    e.fetched_at = now;
                    self.store(url, &e);
                    Ok(Some(e.body))
                }
                None => Err("GitHub answered 304 to a request without a saved copy.".to_string()),
            },
            Ok(Reply::NotFound) => {
                if let Some(path) = self.path(url) {
                    let _ = fs::remove_file(path);
                }
                Ok(None)
            }
            Err(error) => match entry {
                Some(e) => Ok(Some(e.body)),
                None => Err(error),
            },
        }
    }
}
