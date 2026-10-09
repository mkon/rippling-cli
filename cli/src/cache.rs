use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use rippling_api::{time_entries::TimeEntry, Client};
use serde::{Deserialize, Serialize};
use time::{serde::rfc3339, Duration, OffsetDateTime};

use crate::spinner_wrap;

const FILE_NAME: &str = "current_time_entry.json";

#[derive(Serialize, Deserialize)]
struct CachedEntry {
    #[serde(with = "rfc3339")]
    fetched_at: OffsetDateTime,
    entry: Option<TimeEntry>,
}

/// Returns the current time entry, from the cache if it is younger than `ttl`.
/// Without a `ttl` the API is always called. Fresh results are written back to the cache.
pub fn current_time_entry(client: &Client, ttl: Option<Duration>) -> Result<Option<TimeEntry>, rippling_api::Error> {
    if let Some(ttl) = ttl {
        if let Some(cached) = load() {
            if OffsetDateTime::now_utc() - cached.fetched_at < ttl {
                return Ok(cached.entry);
            }
        }
    }
    let entry = spinner_wrap!(client.current_time_entry())?;
    store(entry.as_ref());
    Ok(entry)
}

/// Stores the current time entry, e.g. after clocking in or starting a break.
/// Pass `None` after clocking out.
pub fn store(entry: Option<&TimeEntry>) {
    let Some(path) = path() else { return };
    let cached = CachedEntry { fetched_at: OffsetDateTime::now_utc(), entry: entry.cloned() };
    let result = fs::create_dir_all(path.parent().unwrap())
        .and_then(|_| Ok(serde_json::to_vec(&cached)?))
        // Write to a temporary file first, so concurrent readers never see a partial file
        .and_then(|json| fs::write(path.with_extension("tmp"), json))
        .and_then(|_| fs::rename(path.with_extension("tmp"), &path));
    if let Err(e) = result {
        log::warn!("Could not write cache {}: {e}", path.display());
    }
}

fn load() -> Option<CachedEntry> {
    let json = fs::read(path()?).ok()?;
    serde_json::from_slice(&json).ok()
}

fn path() -> Option<PathBuf> {
    ProjectDirs::from("rs", "", "rippling-cli").map(|dirs| dirs.cache_dir().join(FILE_NAME))
}
