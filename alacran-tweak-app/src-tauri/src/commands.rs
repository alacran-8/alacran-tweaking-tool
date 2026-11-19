//! Tauri command layer. All logic lives in `AppState` methods and free
//! functions so it can be tested with `FakeBackend` and temp dirs; the
//! `#[tauri::command]` functions only forward to them.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::backend::SystemBackend;
use crate::catalog::{Catalog, Hive, Op, Reboot, RegData, Risk, Text, Tweak};
use crate::engine::{self, ApplyReport, RevertReport, Selection, SessionSummary, TweakState};
use crate::settings::{self, Settings};
use crate::store::{now_stamp, BackupStore};
use crate::sysinfo::SystemInfo;
use crate::tools::ToolId;

pub use crate::store::RESTORE_FREQUENCY_ID;
const SYSTEM_DRIVE: &str = "C:";
const RESTORE_POINT_NAME: &str = "Alacran Tweak Tool";

/// Exact-match allowlist for `open_link`.
pub const ALLOWED_LINKS: &[&str] = &[
    "https://kick.com/alacranfps",
    "https://www.youtube.com/@alacrann",
    "https://www.alacrantweak.com",
    "https://github.com/alacran-8/alacran-tweaking-tool",
    "https://github.com/alacran-8/alacran-tweaking-tool/releases",
    "mailto:zorseker.3322@gmail.com",
    "https://developer.microsoft.com/microsoft-edge/webview2/",
];

/// Error sent to the frontend as `{ code, message }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    fn internal(e: anyhow::Error) -> Self {
        Self::new("internal", format!("{e:#}"))
    }
}

/// Host side effects that must never run in dry-run mode or in tests.
pub trait SystemOps: Send + Sync {
    fn protection_enabled(&self, drive: &str) -> Result<bool>;
    fn create_restore_point(&self, desc: &str) -> Result<()>;
    fn enable_protection(&self, drive: &str) -> Result<()>;
    fn run_tool(&self, id: ToolId) -> Result<()>;
}

pub struct RealOps;

impl SystemOps for RealOps {
    fn protection_enabled(&self, drive: &str) -> Result<bool> {
        crate::restore_point::system_protection_enabled(drive)
    }
    fn create_restore_point(&self, desc: &str) -> Result<()> {
        crate::restore_point::create_restore_point(desc)
    }
    fn enable_protection(&self, drive: &str) -> Result<()> {
        crate::restore_point::enable_system_protection(drive)
    }
    fn run_tool(&self, id: ToolId) -> Result<()> {
        crate::tools::run_tool(id)
    }
}

pub struct AppState {
    pub dry_run: bool,
    pub backend: Mutex<Box<dyn SystemBackend + Send>>,
    pub catalog: Catalog,
    pub sysinfo: SystemInfo,
    pub store: BackupStore,
    pub settings_path: PathBuf,
    pub log_path: PathBuf,
    pub restore_point_done: Mutex<bool>,
    pub ops: Box<dyn SystemOps>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn env_dir(var: &str, fallback: impl FnOnce() -> PathBuf) -> PathBuf {
    std::env::var_os(var)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(fallback)
}

impl AppState {
    /// Real paths: `%ProgramData%\AlacranTweak` and `%LOCALAPPDATA%\AlacranTweak`.
    /// Dry-run backups go to the `dry-run` subdirectory of the backup directory
    /// so they never mix with real sessions.
    pub fn new(dry_run: bool) -> Self {
        let data = env_dir("ProgramData", || PathBuf::from(r"C:\ProgramData")).join("AlacranTweak");
        let local = env_dir("LOCALAPPDATA", std::env::temp_dir).join("AlacranTweak");
        let log_path = data.join("apply.log");
        let backups = data.join("backups");
        let (backend, store): (Box<dyn SystemBackend + Send>, BackupStore) = if dry_run {
            (
                Box::new(crate::backend::DryRunBackend::new(
                    crate::backend::WinBackend::new(),
                    Some(log_path.clone()),
                )),
                BackupStore::new_dry_run(backups.join("dry-run")),
            )
        } else {
            (
                Box::new(crate::backend::WinBackend::new()),
                BackupStore::new(backups),
            )
        };
        Self {
            dry_run,
            backend: Mutex::new(backend),
            catalog: crate::catalog::load_embedded(),
            sysinfo: crate::sysinfo::detect(),
            store,
            settings_path: local.join("settings.json"),
            log_path,
            restore_point_done: Mutex::new(false),
            ops: Box::new(RealOps),
        }
    }

    // ---- log -------------------------------------------------------------

    /// Best-effort append; a log failure never fails the user's action.
    fn append_log(&self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        if let Some(dir) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            for l in lines {
                let _ = writeln!(f, "{l}");
            }
        }
    }

    fn mode(&self) -> &'static str {
        if self.dry_run {
            "dry-run"
        } else {
            "real"
        }
    }

    /// One line per entry of the session, then a summary line.
    fn log_session(&self, kind: &str, session_id: &str, summary: String) {
        let ts = now_stamp().1;
        let mode = self.mode();
        let mut lines = Vec::new();
        match self.store.load(session_id) {
            Ok(s) => {
                for e in &s.entries {
                    let outcome = if kind == "REVERT" {
                        if e.reverted {
                            "ok".to_string()
                        } else if e.applied {
                            "error: not restored".to_string()
                        } else {
                            continue;
                        }
                    } else if e.applied && e.error.is_none() {
                        "ok".to_string()
                    } else {
                        format!("error: {}", e.error.as_deref().unwrap_or("not applied"))
                    };
                    lines.push(format!(
                        "{ts} {mode} {kind} session={} tweak={} target={} {outcome}",
                        s.id, e.tweak_id, e.target
                    ));
                }
            }
            Err(e) => lines.push(format!(
                "{ts} {mode} {kind} session={session_id} error: cannot read session: {e:#}"
            )),
        }
        lines.push(format!("{ts} {mode} {kind} SUMMARY session={session_id} {summary}"));
        self.append_log(&lines);
    }

    fn log_note(&self, text: &str) {
        self.append_log(&[format!("{} {} {text}", now_stamp().1, self.mode())]);
    }

    // ---- apply -----------------------------------------------------------

    fn apply_with(
        &self,
        catalog: &Catalog,
        sel: &[Selection],
        be: &mut dyn SystemBackend,
    ) -> Result<ApplyReport, CommandError> {
        let rep = engine::apply(catalog, sel, be, &self.store, &self.sysinfo)
            .map_err(|e| CommandError::new("backup_failed", format!("{e:#}")))?;
        self.log_session(
            "APPLY",
            &rep.session_id,
            format!(
                "applied={} skipped={} failed={} one_way={}",
                rep.applied,
                rep.skipped,
                rep.failed.len(),
                rep.one_way
            ),
        );
        Ok(rep)
    }

    /// Once per run, before the first real apply.
    fn ensure_restore_point(&self, be: &mut dyn SystemBackend) -> Result<(), CommandError> {
        let mut done = lock(&self.restore_point_done);
        if *done {
            return Ok(());
        }
        let rp_err = |m: String| CommandError::new("restore_point_failed", m);
        if !self.dry_run {
            match self.ops.protection_enabled(SYSTEM_DRIVE) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(CommandError::new(
                        "system_protection_off",
                        "System Protection is off for the system drive",
                    ))
                }
                Err(e) => return Err(rp_err(format!("{e:#}"))),
            }
        }
        let rep = self.apply_with(
            &restore_frequency_catalog(),
            &[Selection {
                tweak_id: RESTORE_FREQUENCY_ID.into(),
                option_id: None,
            }],
            be,
        )?;
        if rep.applied != 1 || !rep.failed.is_empty() {
            let why = rep
                .failed
                .first()
                .map(|(_, m)| m.clone())
                .unwrap_or_else(|| "restore point frequency was not applied".into());
            return Err(rp_err(why));
        }
        if self.dry_run {
            self.log_note(&format!("WOULD CREATE_RESTORE_POINT {RESTORE_POINT_NAME}"));
        } else {
            if let Err(e) = self.ops.create_restore_point(RESTORE_POINT_NAME) {
                // Don't leave the frequency override behind for a restore point that never existed.
                let undo = engine::revert(&rep.session_id, be, &self.store);
                self.log_session("REVERT", &rep.session_id, format!("{undo:?}"));
                return Err(rp_err(format!("{e:#}")));
            }
            self.log_note(&format!("RESTORE_POINT created {RESTORE_POINT_NAME}"));
        }
        *done = true;
        Ok(())
    }

    pub fn apply_tweaks(
        &self,
        sel: &[Selection],
        with_restore_point: bool,
    ) -> Result<ApplyReport, CommandError> {
        let mut guard = lock(&self.backend);
        let be: &mut dyn SystemBackend = &mut **guard;
        if with_restore_point {
            self.ensure_restore_point(be)?;
        }
        self.apply_with(&self.catalog, sel, be)
    }

    // ---- revert / sessions ----------------------------------------------

    pub fn revert_session(&self, id: &str) -> Result<RevertReport, CommandError> {
        let mut guard = lock(&self.backend);
        let rep = engine::revert(id, &mut **guard, &self.store).map_err(CommandError::internal)?;
        self.log_session("REVERT", id, revert_summary(&rep));
        Ok(rep)
    }

    pub fn revert_all(&self) -> Result<RevertReport, CommandError> {
        let mut guard = lock(&self.backend);
        let pending: Vec<String> = self
            .store
            .list()
            .map_err(CommandError::internal)?
            .into_iter()
            .filter(|s| !s.reverted)
            .map(|s| s.id)
            .collect();
        let rep = engine::revert_all(&mut **guard, &self.store).map_err(CommandError::internal)?;
        let summary = revert_summary(&rep);
        for id in &pending {
            self.log_session("REVERT", id, format!("revert_all {summary}"));
        }
        Ok(rep)
    }

    pub fn revert_category(&self, category: &str) -> Result<RevertReport, CommandError> {
        let mut guard = lock(&self.backend);
        let rep = engine::revert_category(category, &self.catalog, &mut **guard, &self.store)
            .map_err(CommandError::internal)?;
        self.log_note(&format!(
            "REVERT_CATEGORY SUMMARY category={category} {}",
            revert_summary(&rep)
        ));
        Ok(rep)
    }

    pub fn list_sessions(&self) -> Result<SessionList, CommandError> {
        let (sessions, broken) = self.store.scan().map_err(CommandError::internal)?;
        Ok(SessionList { sessions, broken })
    }

    // ---- catalog ---------------------------------------------------------

    pub fn catalog_views(&self) -> Vec<TweakView> {
        let guard = lock(&self.backend);
        let be: &dyn SystemBackend = &**guard;
        self.catalog
            .tweaks
            .iter()
            .map(|t| {
                let counts = engine::match_counts(t, None, be);
                let (matched, total) = counts.unwrap_or((0, 0));
                TweakView {
                    available: engine::is_available(t, &self.sysinfo),
                    state: engine::state_from_counts(counts),
                    matched,
                    total,
                    tweak: t.clone(),
                }
            })
            .collect()
    }

    // ---- misc ------------------------------------------------------------

    pub fn enable_protection(&self) -> Result<(), CommandError> {
        if self.dry_run {
            self.log_note(&format!("WOULD ENABLE_SYSTEM_PROTECTION {SYSTEM_DRIVE}"));
            return Ok(());
        }
        self.ops
            .enable_protection(SYSTEM_DRIVE)
            .map_err(CommandError::internal)?;
        self.log_note(&format!("ENABLE_SYSTEM_PROTECTION {SYSTEM_DRIVE}"));
        Ok(())
    }

    pub fn run_tool(&self, id: ToolId) -> Result<(), CommandError> {
        if self.dry_run {
            self.log_note(&format!("WOULD RUN_TOOL {id:?}"));
            return Ok(());
        }
        self.log_note(&format!("RUN_TOOL {id:?}"));
        self.ops.run_tool(id).map_err(CommandError::internal)
    }
}

fn revert_summary(r: &RevertReport) -> String {
    format!(
        "reverted={} skipped_newer={} failed={}",
        r.reverted,
        r.skipped_newer.len(),
        r.failed.len()
    )
}

fn simple_tweak(id: &str, ops: Vec<Op>) -> Tweak {
    let text = |s: &str| Text {
        tr: s.into(),
        en: s.into(),
    };
    Tweak {
        id: id.into(),
        category: "internal".into(),
        title: text(id),
        description: text(id),
        risk: Risk::Safe,
        in_one_click: false,
        new: false,
        reboot: Reboot::None,
        one_way: false,
        min_build: 0,
        max_build: None,
        vendor: None,
        laptop: None,
        caution_note: None,
        ops,
        options: Vec::new(),
        default_option: None,
        auto_option: None,
    }
}

fn restore_frequency_catalog() -> Catalog {
    Catalog {
        tweaks: vec![simple_tweak(
            RESTORE_FREQUENCY_ID,
            vec![Op::RegSet {
                hive: Hive::Hklm,
                path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore".into(),
                name: "SystemRestorePointCreationFrequency".into(),
                data: RegData::Dword(0),
            }],
        )],
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TweakView {
    #[serde(flatten)]
    pub tweak: Tweak,
    pub state: TweakState,
    pub available: bool,
    /// Checkable values already at the tweak's target / all checkable values.
    pub matched: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionList {
    pub sessions: Vec<SessionSummary>,
    /// `(file name, error)` of unreadable backup files.
    pub broken: Vec<(String, String)>,
}

pub fn check_link(url: &str) -> Result<(), CommandError> {
    if ALLOWED_LINKS.contains(&url) {
        Ok(())
    } else {
        Err(CommandError::new(
            "link_not_allowed",
            format!("link not allowed: {url}"),
        ))
    }
}

fn open_url(url: &str) -> Result<()> {
    use ::windows::core::{w, PCWSTR};
    use ::windows::Win32::UI::Shell::ShellExecuteW;
    use ::windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wurl = crate::backend::windows::wide(url);
    let r = unsafe {
        ShellExecuteW(None, w!("open"), PCWSTR(wurl.as_ptr()), None, None, SW_SHOWNORMAL)
    };
    if r.0 as isize <= 32 {
        anyhow::bail!("cannot open link (ShellExecute code {})", r.0 as isize);
    }
    Ok(())
}

// ---- Tauri commands (JS names = fn names) ----------------------------------

#[tauri::command]
pub fn is_dry_run(state: State<'_, AppState>) -> bool {
    state.dry_run
}

#[tauri::command]
pub async fn get_system_info(state: State<'_, AppState>) -> Result<SystemInfo, CommandError> {
    Ok(state.sysinfo.clone())
}

#[tauri::command]
pub async fn get_catalog(state: State<'_, AppState>) -> Result<Vec<TweakView>, CommandError> {
    Ok(state.catalog_views())
}

#[tauri::command]
pub async fn apply_tweaks(
    state: State<'_, AppState>,
    selections: Vec<Selection>,
) -> Result<ApplyReport, CommandError> {
    state.apply_tweaks(&selections, true)
}

#[tauri::command]
pub async fn apply_tweaks_force(
    state: State<'_, AppState>,
    selections: Vec<Selection>,
) -> Result<ApplyReport, CommandError> {
    state.apply_tweaks(&selections, false)
}

#[tauri::command]
pub async fn enable_protection(state: State<'_, AppState>) -> Result<(), CommandError> {
    state.enable_protection()
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<SessionList, CommandError> {
    state.list_sessions()
}

#[tauri::command]
pub async fn revert_session(
    state: State<'_, AppState>,
    id: String,
) -> Result<RevertReport, CommandError> {
    state.revert_session(&id)
}

#[tauri::command]
pub async fn revert_all(state: State<'_, AppState>) -> Result<RevertReport, CommandError> {
    state.revert_all()
}

#[tauri::command]
pub async fn revert_category(
    state: State<'_, AppState>,
    category: String,
) -> Result<RevertReport, CommandError> {
    state.revert_category(&category)
}

#[tauri::command]
pub async fn run_tool(state: State<'_, AppState>, id: ToolId) -> Result<(), CommandError> {
    state.run_tool(id)
}

#[tauri::command]
pub async fn open_link(url: String) -> Result<(), CommandError> {
    check_link(&url)?;
    open_url(&url).map_err(CommandError::internal)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, CommandError> {
    Ok(settings::load(&state.settings_path))
}

#[tauri::command]
pub async fn set_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), CommandError> {
    settings::save(&state.settings_path, &settings).map_err(CommandError::internal)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex as StdMutex};

    use super::*;
    use crate::backend::{DryRunBackend, FakeBackend};

    #[derive(Default)]
    struct Calls {
        protection_on: bool,
        create_fails: bool,
        log: Vec<String>,
    }

    struct FakeOps(Arc<StdMutex<Calls>>);

    impl SystemOps for FakeOps {
        fn protection_enabled(&self, _: &str) -> Result<bool> {
            let mut c = self.0.lock().unwrap();
            c.log.push("protection_enabled".into());
            Ok(c.protection_on)
        }
        fn create_restore_point(&self, d: &str) -> Result<()> {
            let mut c = self.0.lock().unwrap();
            c.log.push(format!("create_restore_point {d}"));
            if c.create_fails {
                anyhow::bail!("vss busy");
            }
            Ok(())
        }
        fn enable_protection(&self, d: &str) -> Result<()> {
            self.0.lock().unwrap().log.push(format!("enable_protection {d}"));
            Ok(())
        }
        fn run_tool(&self, id: ToolId) -> Result<()> {
            self.0.lock().unwrap().log.push(format!("run_tool {id:?}"));
            Ok(())
        }
    }

    const PATH: &str = r"SOFTWARE\Test";

    fn test_catalog() -> Catalog {
        Catalog {
            tweaks: vec![simple_tweak(
                "t1",
                vec![Op::RegSet {
                    hive: Hive::Hklm,
                    path: PATH.into(),
                    name: "v".into(),
                    data: RegData::Dword(1),
                }],
            )],
        }
    }

    fn state(name: &str, dry: bool) -> (AppState, Arc<StdMutex<Calls>>, PathBuf) {
        let root = std::env::temp_dir().join(format!("alacran-cmd-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let calls = Arc::new(StdMutex::new(Calls::default()));
        let log_path = root.join("apply.log");
        let backups = root.join("backups");
        let (backend, store): (Box<dyn SystemBackend + Send>, BackupStore) = if dry {
            (
                Box::new(DryRunBackend::new(FakeBackend::new(), Some(log_path.clone()))),
                BackupStore::new_dry_run(&backups),
            )
        } else {
            (Box::new(FakeBackend::new()), BackupStore::new(&backups))
        };
        let st = AppState {
            dry_run: dry,
            backend: Mutex::new(backend),
            catalog: test_catalog(),
            sysinfo: SystemInfo {
                build: 26100,
                ..Default::default()
            },
            store,
            settings_path: root.join("local").join("settings.json"),
            log_path,
            restore_point_done: Mutex::new(false),
            ops: Box::new(FakeOps(calls.clone())),
        };
        (st, calls, root)
    }

    fn sel() -> Vec<Selection> {
        vec![Selection {
            tweak_id: "t1".into(),
            option_id: None,
        }]
    }

    fn freq(st: &AppState) -> Option<RegData> {
        lock(&st.backend)
            .reg_get(
                Hive::Hklm,
                r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore",
                "SystemRestorePointCreationFrequency",
            )
            .unwrap()
    }

    #[test]
    fn open_link_rejects_unknown_url() {
        for bad in [
            "https://evil.example.com",
            "http://www.alacrantweak.com",
            "https://www.alacrantweak.com.evil.com",
            "https://www.alacrantweak.com/",
            "file:///C:/Windows/System32/cmd.exe",
            "mailto:other@example.com",
            "https://kick.com/alacranfps/",
            "http://kick.com/alacranfps",
            "https://kick.com/alacranfps?x=1",
            "",
        ] {
            let e = check_link(bad).unwrap_err();
            assert_eq!(e.code, "link_not_allowed", "{bad}");
        }
        for ok in ALLOWED_LINKS {
            check_link(ok).unwrap();
        }
        assert_eq!(ALLOWED_LINKS.len(), 7);
        check_link("https://kick.com/alacranfps").unwrap();
    }

    #[test]
    fn protection_off_blocks_apply_and_writes_nothing() {
        let (st, calls, root) = state("off", false);
        let e = st.apply_tweaks(&sel(), true).unwrap_err();
        assert_eq!(e.code, "system_protection_off");
        assert_eq!(freq(&st), None);
        assert_eq!(calls.lock().unwrap().log, ["protection_enabled"]);
        assert!(st.list_sessions().unwrap().sessions.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restore_point_flow_runs_once_and_is_revertible() {
        let (st, calls, root) = state("flow", false);
        calls.lock().unwrap().protection_on = true;
        let rep = st.apply_tweaks(&sel(), true).unwrap();
        assert_eq!(rep.applied, 1);
        assert_eq!(freq(&st), Some(RegData::Dword(0)));
        assert_eq!(
            calls.lock().unwrap().log,
            ["protection_enabled", "create_restore_point Alacran Tweak Tool"]
        );
        // second apply: no new restore point
        st.apply_tweaks(&sel(), true).unwrap();
        assert_eq!(calls.lock().unwrap().log.len(), 2);
        // frequency session (flagged internal) + 2 tweak sessions
        let list = st.list_sessions().unwrap();
        assert_eq!(list.sessions.len(), 3);
        assert!(list.broken.is_empty());
        let internal: Vec<bool> = list.sessions.iter().map(|s| s.internal).collect();
        assert_eq!(internal.iter().filter(|i| **i).count(), 1, "{internal:?}");
        // revert everything: the frequency value is gone again
        let r = st.revert_all().unwrap();
        assert!(r.failed.is_empty(), "{r:?}");
        assert_eq!(freq(&st), None);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn restore_point_failure_and_force() {
        let (st, calls, root) = state("fail", false);
        {
            let mut c = calls.lock().unwrap();
            c.protection_on = true;
            c.create_fails = true;
        }
        let e = st.apply_tweaks(&sel(), true).unwrap_err();
        assert_eq!(e.code, "restore_point_failed");
        // the frequency tweak is rolled back: no value, its session already reverted
        assert_eq!(freq(&st), None);
        assert!(st.list_sessions().unwrap().sessions.iter().all(|s| s.reverted && s.internal));
        // user chose to continue; force never touches the restore point
        let rep = st.apply_tweaks(&sel(), false).unwrap();
        assert_eq!(rep.applied, 1);
        assert_eq!(
            calls.lock().unwrap().log,
            ["protection_enabled", "create_restore_point Alacran Tweak Tool"]
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn apply_log_has_one_line_per_entry_and_summary() {
        let (st, calls, root) = state("log", false);
        calls.lock().unwrap().protection_on = true;
        st.apply_tweaks(&sel(), true).unwrap();
        let log = std::fs::read_to_string(&st.log_path).unwrap();
        assert!(
            log.lines()
                .any(|l| l.contains("APPLY session=") && l.contains("tweak=t1 target=") && l.ends_with(" ok")),
            "{log}"
        );
        assert!(log.lines().any(|l| l.contains("SUMMARY") && l.contains("applied=1")), "{log}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn dry_run_never_calls_host_ops_and_logs_would() {
        let (st, calls, root) = state("dry", true);
        st.apply_tweaks(&sel(), true).unwrap();
        st.enable_protection().unwrap();
        st.run_tool(ToolId::Sfc).unwrap();
        assert!(calls.lock().unwrap().log.is_empty());
        let log = std::fs::read_to_string(&st.log_path).unwrap();
        assert!(log.contains("WOULD CREATE_RESTORE_POINT"), "{log}");
        assert!(log.contains("WOULD REG_SET"), "{log}");
        assert!(log.contains("WOULD ENABLE_SYSTEM_PROTECTION"), "{log}");
        assert!(log.contains("WOULD RUN_TOOL Sfc"), "{log}");
        assert_eq!(st.list_sessions().unwrap().sessions.len(), 2);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn revert_category_reverts_and_logs_summary() {
        let (mut st, calls, root) = state("revcat", false);
        calls.lock().unwrap().protection_on = true;
        st.catalog.tweaks[0].category = "network".into();
        st.apply_tweaks(&sel(), false).unwrap();
        assert_eq!(st.revert_category("power").unwrap().reverted, 0);
        assert_eq!(st.revert_category("network").unwrap().reverted, 1);
        let log = std::fs::read_to_string(&st.log_path).unwrap();
        assert!(
            log.contains("REVERT_CATEGORY SUMMARY category=network reverted=1"),
            "{log}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn catalog_views_hide_internal_tweak_and_flag_availability() {
        let (mut st, _, root) = state("cat", false);
        st.catalog = crate::catalog::load_embedded();
        st.sysinfo.build = 1;
        let views = st.catalog_views();
        assert_eq!(views.len(), st.catalog.tweaks.len());
        assert!(views.iter().all(|v| v.tweak.id != RESTORE_FREQUENCY_ID));
        assert!(views.iter().any(|v| !v.available));
        let json = serde_json::to_value(&views[0]).unwrap();
        assert!(json.get("id").is_some() && json.get("state").is_some());
        assert_eq!(json["available"], false);
        let _ = std::fs::remove_dir_all(root);
    }
}
