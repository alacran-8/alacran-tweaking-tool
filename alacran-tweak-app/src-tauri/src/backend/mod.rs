//! System access abstraction. The engine talks only to `SystemBackend`;
//! the real Win32 implementation is `windows::WinBackend`, tests use `FakeBackend`,
//! and `--dry-run` wraps any backend in `DryRunBackend`.

use crate::catalog::{CleanupTarget, DeviceClass, Hive, RegData};

pub use anyhow::Result;

pub mod dryrun;
pub mod fake;
#[cfg(windows)]
pub mod windows;

pub use dryrun::DryRunBackend;
pub use fake::FakeBackend;
#[cfg(windows)]
pub use self::windows::WinBackend;

/// A registry key location. `Hive::Hkcu` is resolved by the backend to `HKU\<sid>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegKeyRef {
    pub root: Hive,
    pub path: String,
}

pub trait SystemBackend: Send {
    fn interactive_sid(&self) -> String;

    fn reg_get(&self, h: Hive, path: &str, name: &str) -> Result<Option<RegData>>;
    fn reg_key_exists(&self, h: Hive, path: &str) -> Result<bool>;
    fn reg_set(&mut self, h: Hive, path: &str, name: &str, data: &RegData) -> Result<()>;
    fn reg_delete_value(&mut self, h: Hive, path: &str, name: &str) -> Result<()>;
    fn reg_delete_key_if_empty(&mut self, h: Hive, path: &str) -> Result<()>;

    fn service_start_get(&self, name: &str) -> Result<Option<u32>>;
    fn service_start_set(&mut self, name: &str, start: u32) -> Result<()>;

    fn task_enabled_get(&self, path: &str) -> Result<Option<bool>>;
    fn task_enabled_set(&mut self, path: &str, enabled: bool) -> Result<()>;

    /// Returns (ac, dc) of `scheme`.
    fn power_get(&self, scheme: &str, subgroup: &str, setting: &str) -> Result<(u32, u32)>;
    /// Writes `scheme`'s values; re-applies it only if it is the active scheme.
    fn power_set(
        &mut self,
        scheme: &str,
        subgroup: &str,
        setting: &str,
        ac: Option<u32>,
        dc: Option<u32>,
    ) -> Result<()>;
    fn active_scheme(&self) -> Result<String>;
    fn set_active_scheme(&mut self, guid: &str) -> Result<()>;
    fn duplicate_scheme(&mut self, source: &str) -> Result<String>;

    fn bcd_get(&self, name: &str) -> Result<Option<String>>;
    fn bcd_set(&mut self, name: &str, value: &str) -> Result<()>;
    fn bcd_delete(&mut self, name: &str) -> Result<()>;

    /// Returns the number of packages removed.
    fn appx_remove(&mut self, pattern: &str) -> Result<u32>;
    /// Returns bytes freed.
    fn cleanup(&mut self, t: CleanupTarget) -> Result<u64>;

    /// Device instance ids of the given class.
    fn devices(&self, class: DeviceClass) -> Result<Vec<String>>;
    /// HKLM paths of the network adapter class subkeys.
    fn nic_class_keys(&self) -> Result<Vec<String>>;
}
