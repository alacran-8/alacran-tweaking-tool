//! Backup sessions on disk: one `<session-id>.json` per apply, written atomically.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::{Op, RegData};

/// Session file schema version.
pub const SCHEMA: u32 = 1;

/// Hidden tweak that makes `SystemRestorePointCreationFrequency` 0 (captured and
/// revertible); never part of `get_catalog`.
pub const RESTORE_FREQUENCY_ID: &str = "internal_restore_frequency";

/// What a target looked like before the session touched it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "state")]
pub enum PriorState {
    Reg {
        value: Option<RegData>,
        key_existed: bool,
    },
    Service(Option<u32>),
    Task(Option<bool>),
    Power {
        /// Plan GUID the values belong to; empty in older backups (= active plan).
        #[serde(default)]
        scheme: String,
        ac: u32,
        dc: u32,
    },
    Scheme(String),
    Bcd(Option<String>),
    /// Cannot be restored (appx removal, cleanup).
    OneWay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupEntry {
    pub tweak_id: String,
    pub category: String,
    /// Canonical id, e.g. `reg:HKLM\...\name` or `svc:Name`.
    pub target: String,
    /// Resolved op: device/NIC/DX ops are stored as the concrete `RegSet`.
    pub op: Op,
    pub prior: PriorState,
    /// Value written, in the same JSON shape as a fresh read of the target.
    /// Set to the intended value before writing, replaced by a read-back after.
    pub written: Option<serde_json::Value>,
    pub applied: bool,
    pub error: Option<String>,
    /// Registry keys (relative to the op's hive, deepest first) that did not
    /// exist before and are removed on revert if they are empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created_keys: Vec<String>,
    /// Set once this entry has been restored.
    #[serde(default)]
    pub reverted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub schema: u32,
    /// `YYYYMMDD-HHMMSS` (UTC), with `-2`, `-3`… when several start in one second.
    pub id: String,
    /// RFC 3339, UTC.
    pub created: String,
    pub dry_run: bool,
    /// SID of the interactive user whose hive the HKCU entries were written to.
    #[serde(default)]
    pub interactive_sid: String,
    pub entries: Vec<BackupEntry>,
    pub reverted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub created: String,
    /// Number of changes actually made.
    pub count: usize,
    pub reverted: bool,
    /// Only the restore point frequency setting (the app's own hidden tweak).
    pub internal: bool,
}

/// Session files in one directory. A store opened with `new_dry_run` stamps the
/// sessions it creates as dry-run, and the engine only reverts sessions whose
/// mode matches the store's, so dry-run backups never drive real writes.
#[derive(Debug, Clone)]
pub struct BackupStore {
    pub dir: PathBuf,
    pub dry_run: bool,
}

impl BackupStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            dry_run: false,
        }
    }

    pub fn new_dry_run(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            dry_run: true,
        }
    }

    fn path(&self, id: &str) -> Result<PathBuf> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit() || c == '-') {
            bail!("invalid session id: {id:?}");
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    /// First free id for `base`: `base`, then `base-2`, `base-3`…
    pub fn allocate_id(&self, base: &str) -> String {
        (1u32..)
            .map(|n| {
                if n == 1 {
                    base.to_string()
                } else {
                    format!("{base}-{n}")
                }
            })
            .find(|id| self.path(id).map(|p| !p.exists()).unwrap_or(false))
            .expect("unbounded id search")
    }

    /// Saves a new session; fails if a file with its id already exists.
    pub fn create(&self, s: &Session) -> Result<()> {
        if self.path(&s.id)?.exists() {
            bail!("session {} already exists", s.id);
        }
        self.save(s)
    }

    /// Writes `<id>.json.tmp`, flushes it, then renames it over `<id>.json`.
    pub fn save(&self, s: &Session) -> Result<()> {
        let path = self.path(&s.id)?;
        std::fs::create_dir_all(&self.dir)
            .with_context(|| format!("cannot create backup directory {}", self.dir.display()))?;
        let tmp = path.with_extension("json.tmp");
        let body = serde_json::to_vec_pretty(s)?;
        write_synced(&tmp, &body).with_context(|| format!("cannot write {}", tmp.display()))?;
        std::fs::rename(&tmp, &path).with_context(|| format!("cannot write {}", path.display()))?;
        Ok(())
    }

    /// Readable sessions, newest first. Unreadable files are not in this list;
    /// see `scan`/`list_broken`. A missing directory is empty.
    pub fn list(&self) -> Result<Vec<SessionSummary>> {
        Ok(self.scan()?.0)
    }

    /// `(file name, error)` for every session file that cannot be loaded.
    pub fn list_broken(&self) -> Result<Vec<(String, String)>> {
        Ok(self.scan()?.1)
    }

    /// Readable sessions (newest first) and broken files (`(file name, error)`).
    pub fn scan(&self) -> Result<(Vec<SessionSummary>, Vec<(String, String)>)> {
        let mut out = Vec::new();
        let mut broken = Vec::new();
        if !self.dir.exists() {
            return Ok((out, broken));
        }
        for entry in std::fs::read_dir(&self.dir)? {
            let path = entry?.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let file = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let s = match read_session(&path) {
                Ok(s) => s,
                Err(e) => {
                    broken.push((file, format!("{e:#}")));
                    continue;
                }
            };
            out.push(SessionSummary {
                count: s.entries.iter().filter(|e| e.applied).count(),
                internal: !s.entries.is_empty()
                    && s.entries.iter().all(|e| e.tweak_id == RESTORE_FREQUENCY_ID),
                id: s.id,
                created: s.created,
                reverted: s.reverted,
            });
        }
        out.sort_by(|a, b| sort_key(&b.id).cmp(&sort_key(&a.id)));
        broken.sort();
        Ok((out, broken))
    }

    pub fn load(&self, id: &str) -> Result<Session> {
        read_session(&self.path(id)?)
    }
}

fn read_session(path: &Path) -> Result<Session> {
    let bytes = std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    let s: Session = serde_json::from_slice(&bytes)
        .with_context(|| format!("corrupt backup {}", path.display()))?;
    if s.schema != SCHEMA {
        bail!(
            "backup {} has unsupported schema {}",
            path.display(),
            s.schema
        );
    }
    Ok(s)
}

fn write_synced(path: &Path, body: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    f.write_all(body)?;
    f.sync_all()
}

/// Orders `base` < `base-2` < `base-10`.
fn sort_key(id: &str) -> (String, u32) {
    match id.get(15..) {
        Some(rest) if rest.starts_with('-') => {
            (id[..15].to_string(), rest[1..].parse().unwrap_or(0))
        }
        _ => (id.to_string(), 1),
    }
}

/// `(session id base, RFC 3339)` for the current UTC time.
pub fn now_stamp() -> (String, String) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_stamp(secs as i64)
}

/// `(YYYYMMDD-HHMMSS, YYYY-MM-DDTHH:MM:SSZ)` for a Unix time in seconds.
pub fn format_stamp(unix_secs: i64) -> (String, String) {
    let (y, m, d) = civil_from_days(unix_secs.div_euclid(86_400));
    let rem = unix_secs.rem_euclid(86_400);
    let (hh, mm, ss) = (rem / 3600, rem % 3600 / 60, rem % 60);
    (
        format!("{y:04}{m:02}{d:02}-{hh:02}{mm:02}{ss:02}"),
        format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z"),
    )
}

/// Days since 1970-01-01 to (year, month, day), proleptic Gregorian (H. Hinnant).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}
