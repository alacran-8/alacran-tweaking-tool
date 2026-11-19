//! Real Win32 implementation of `SystemBackend`.
//!
//! Each op family lives in its own submodule; this file only wires them to
//! the trait. Pure parsing/matching helpers in the submodules are unit-tested
//! with fixture strings. The setters are never exercised on a development host
//! (see the project constraints); they run inside the test VM only.

use crate::catalog::{CleanupTarget, DeviceClass, Hive, RegData};

use super::{Result, SystemBackend};

pub mod appx;
pub mod bcd;
pub mod cleanup;
pub mod devices;
pub mod power;
pub mod registry;
pub mod services;
pub mod session;
pub mod tasks;

/// Error text used whenever an HKCU operation has no interactive user hive.
pub const NO_USER_SESSION: &str = "kullanıcı oturumu bulunamadı";

/// NUL-terminated UTF-16 copy of `s`.
pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// `%SystemRoot%`, falling back to `C:\Windows`.
pub fn system_root() -> std::path::PathBuf {
    std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
}

/// Initializes COM (multithreaded) for the current thread for the guard's
/// lifetime. If the thread already runs an STA, COM is still usable and the
/// guard does not uninitialize it.
pub struct ComGuard {
    owns: bool,
}

impl ComGuard {
    pub fn new() -> Result<Self> {
        use ::windows::Win32::Foundation::RPC_E_CHANGED_MODE;
        use ::windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if hr == RPC_E_CHANGED_MODE {
            return Ok(Self { owns: false });
        }
        hr.ok()?;
        Ok(Self { owns: true })
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.owns {
            unsafe { ::windows::Win32::System::Com::CoUninitialize() };
        }
    }
}

/// The real system. The interactive user's SID is resolved once at
/// construction so that every call (and every backup session) sees the same
/// owner.
pub struct WinBackend {
    sid: String,
}

impl WinBackend {
    pub fn new() -> Self {
        Self {
            sid: session::interactive_sid().unwrap_or_default(),
        }
    }
}

impl Default for WinBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemBackend for WinBackend {
    fn interactive_sid(&self) -> String {
        self.sid.clone()
    }

    fn reg_get(&self, h: Hive, path: &str, name: &str) -> Result<Option<RegData>> {
        registry::get(h, &self.sid, path, name)
    }

    fn reg_key_exists(&self, h: Hive, path: &str) -> Result<bool> {
        registry::key_exists(h, &self.sid, path)
    }

    fn reg_set(&mut self, h: Hive, path: &str, name: &str, data: &RegData) -> Result<()> {
        registry::set(h, &self.sid, path, name, data)
    }

    fn reg_delete_value(&mut self, h: Hive, path: &str, name: &str) -> Result<()> {
        registry::delete_value(h, &self.sid, path, name)
    }

    fn reg_delete_key_if_empty(&mut self, h: Hive, path: &str) -> Result<()> {
        registry::delete_key_if_empty(h, &self.sid, path)
    }

    fn service_start_get(&self, name: &str) -> Result<Option<u32>> {
        services::start_get(name)
    }

    fn service_start_set(&mut self, name: &str, start: u32) -> Result<()> {
        services::start_set(name, start)
    }

    fn task_enabled_get(&self, path: &str) -> Result<Option<bool>> {
        tasks::enabled_get(path)
    }

    fn task_enabled_set(&mut self, path: &str, enabled: bool) -> Result<()> {
        tasks::enabled_set(path, enabled)
    }

    fn power_get(&self, scheme: &str, subgroup: &str, setting: &str) -> Result<(u32, u32)> {
        power::get(scheme, subgroup, setting)
    }

    fn power_set(
        &mut self,
        scheme: &str,
        subgroup: &str,
        setting: &str,
        ac: Option<u32>,
        dc: Option<u32>,
    ) -> Result<()> {
        power::set(scheme, subgroup, setting, ac, dc)
    }

    fn active_scheme(&self) -> Result<String> {
        power::active_scheme()
    }

    fn set_active_scheme(&mut self, guid: &str) -> Result<()> {
        power::set_active_scheme(guid)
    }

    fn duplicate_scheme(&mut self, source: &str) -> Result<String> {
        power::duplicate_scheme(source)
    }

    fn bcd_get(&self, name: &str) -> Result<Option<String>> {
        bcd::get(name)
    }

    fn bcd_set(&mut self, name: &str, value: &str) -> Result<()> {
        bcd::set(name, value)
    }

    fn bcd_delete(&mut self, name: &str) -> Result<()> {
        bcd::delete(name)
    }

    fn appx_remove(&mut self, pattern: &str) -> Result<u32> {
        appx::remove(pattern, &self.sid)
    }

    fn cleanup(&mut self, t: CleanupTarget) -> Result<u64> {
        cleanup::run(t, &self.sid)
    }

    fn devices(&self, class: DeviceClass) -> Result<Vec<String>> {
        devices::present(class)
    }

    fn nic_class_keys(&self) -> Result<Vec<String>> {
        devices::nic_class_keys()
    }
}
