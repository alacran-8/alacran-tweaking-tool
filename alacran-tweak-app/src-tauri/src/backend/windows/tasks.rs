//! Scheduled task enable/disable through the Task Scheduler COM API.

use ::windows::core::{BSTR, HRESULT};
use ::windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, VARIANT_FALSE, VARIANT_TRUE,
};
use ::windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use ::windows::Win32::System::TaskScheduler::{IRegisteredTask, ITaskService, TaskScheduler};
use ::windows::Win32::System::Variant::VARIANT;
use anyhow::{bail, Context};

use super::ComGuard;
use crate::backend::Result;

/// Splits `\Folder\Sub\Name` into (`\Folder\Sub`, `Name`). A task in the root
/// folder yields (`\`, `Name`).
pub fn split_task_path(path: &str) -> Option<(String, String)> {
    let path = path.trim().trim_end_matches('\\');
    let path = if path.starts_with('\\') {
        path.to_string()
    } else {
        format!("\\{path}")
    };
    let (folder, name) = path.rsplit_once('\\')?;
    if name.is_empty() {
        return None;
    }
    let folder = if folder.is_empty() {
        "\\".to_string()
    } else {
        folder.to_string()
    };
    Some((folder, name.to_string()))
}

fn is_not_found(code: HRESULT) -> bool {
    code == ERROR_FILE_NOT_FOUND.to_hresult() || code == ERROR_PATH_NOT_FOUND.to_hresult()
}

/// Runs `f` on the registered task at `path`; `Ok(None)` when it does not exist.
fn with_task<T>(path: &str, f: impl FnOnce(&IRegisteredTask) -> Result<T>) -> Result<Option<T>> {
    let Some((folder, name)) = split_task_path(path) else {
        bail!("invalid task path: {path}");
    };
    let _com = ComGuard::new()?;
    unsafe {
        let svc: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
            .context("cannot create the task scheduler service")?;
        let empty = VARIANT::default();
        svc.Connect(&empty, &empty, &empty, &empty)
            .context("cannot connect to the task scheduler")?;
        let folder = match svc.GetFolder(&BSTR::from(folder.as_str())) {
            Ok(f) => f,
            Err(e) if is_not_found(e.code()) => return Ok(None),
            Err(e) => return Err(e).with_context(|| format!("cannot open task folder of {path}")),
        };
        let task = match folder.GetTask(&BSTR::from(name.as_str())) {
            Ok(t) => t,
            Err(e) if is_not_found(e.code()) => return Ok(None),
            Err(e) => return Err(e).with_context(|| format!("cannot open task {path}")),
        };
        f(&task).map(Some)
    }
}

pub fn enabled_get(path: &str) -> Result<Option<bool>> {
    with_task(path, |t| {
        let v = unsafe { t.Enabled() }.with_context(|| format!("cannot read task {path}"))?;
        Ok(v.as_bool())
    })
}

pub fn enabled_set(path: &str, enabled: bool) -> Result<()> {
    let done = with_task(path, |t| {
        let v = if enabled { VARIANT_TRUE } else { VARIANT_FALSE };
        unsafe { t.SetEnabled(v) }.with_context(|| format!("cannot change task {path}"))
    })?;
    if done.is_none() {
        bail!("task not found: {path}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::split_task_path;

    fn s(a: &str, b: &str) -> Option<(String, String)> {
        Some((a.to_string(), b.to_string()))
    }

    #[test]
    fn splits_folder_and_name() {
        assert_eq!(
            split_task_path(r"\Microsoft\Windows\Application Experience\ProgramDataUpdater"),
            s(
                r"\Microsoft\Windows\Application Experience",
                "ProgramDataUpdater"
            )
        );
        assert_eq!(
            split_task_path(r"Microsoft\Windows\X\Y"),
            s(r"\Microsoft\Windows\X", "Y")
        );
        assert_eq!(split_task_path(r"\RootTask"), s(r"\", "RootTask"));
        assert_eq!(split_task_path(r"\A\B\"), s(r"\A", "B"));
        assert_eq!(split_task_path(""), None);
        assert_eq!(split_task_path(r"\"), None);
    }
}
