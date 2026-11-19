use std::io::Write;
use std::path::PathBuf;

use super::{Result, SystemBackend};
use crate::catalog::{CleanupTarget, DeviceClass, Hive, RegData};

/// Wraps a backend: getters delegate, setters only log `WOULD <OP> <target>=<value>`.
pub struct DryRunBackend<B: SystemBackend> {
    inner: B,
    pub log: Vec<String>,
    log_path: Option<PathBuf>,
}

impl<B: SystemBackend> DryRunBackend<B> {
    pub fn new(inner: B, log_path: Option<PathBuf>) -> Self {
        Self { inner, log: Vec::new(), log_path }
    }

    pub fn inner(&self) -> &B {
        &self.inner
    }

    pub fn into_inner(self) -> B {
        self.inner
    }

    fn record(&mut self, line: String) -> Result<()> {
        if let Some(p) = &self.log_path {
            if let Some(dir) = p.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let mut f = std::fs::OpenOptions::new().create(true).append(true).open(p)?;
            writeln!(f, "{line}")?;
        }
        self.log.push(line);
        Ok(())
    }
}

impl<B: SystemBackend> SystemBackend for DryRunBackend<B> {
    fn interactive_sid(&self) -> String {
        self.inner.interactive_sid()
    }
    fn reg_get(&self, h: Hive, path: &str, name: &str) -> Result<Option<RegData>> {
        self.inner.reg_get(h, path, name)
    }
    fn reg_key_exists(&self, h: Hive, path: &str) -> Result<bool> {
        self.inner.reg_key_exists(h, path)
    }
    fn reg_set(&mut self, h: Hive, path: &str, name: &str, data: &RegData) -> Result<()> {
        self.record(format!("WOULD REG_SET {h:?}\\{path}\\{name}={data:?}"))
    }
    fn reg_delete_value(&mut self, h: Hive, path: &str, name: &str) -> Result<()> {
        self.record(format!("WOULD REG_DELETE_VALUE {h:?}\\{path}\\{name}="))
    }
    fn reg_delete_key_if_empty(&mut self, h: Hive, path: &str) -> Result<()> {
        self.record(format!("WOULD REG_DELETE_KEY_IF_EMPTY {h:?}\\{path}="))
    }
    fn service_start_get(&self, name: &str) -> Result<Option<u32>> {
        self.inner.service_start_get(name)
    }
    fn service_start_set(&mut self, name: &str, start: u32) -> Result<()> {
        self.record(format!("WOULD SERVICE_START {name}={start}"))
    }
    fn task_enabled_get(&self, path: &str) -> Result<Option<bool>> {
        self.inner.task_enabled_get(path)
    }
    fn task_enabled_set(&mut self, path: &str, enabled: bool) -> Result<()> {
        self.record(format!("WOULD TASK_ENABLED {path}={enabled}"))
    }
    fn power_get(&self, scheme: &str, subgroup: &str, setting: &str) -> Result<(u32, u32)> {
        self.inner.power_get(scheme, subgroup, setting)
    }
    fn power_set(
        &mut self,
        scheme: &str,
        subgroup: &str,
        setting: &str,
        ac: Option<u32>,
        dc: Option<u32>,
    ) -> Result<()> {
        self.record(format!("WOULD POWER_SET {scheme}/{subgroup}/{setting} ac={ac:?} dc={dc:?}"))
    }
    fn active_scheme(&self) -> Result<String> {
        self.inner.active_scheme()
    }
    fn set_active_scheme(&mut self, guid: &str) -> Result<()> {
        self.record(format!("WOULD SET_ACTIVE_SCHEME scheme={guid}"))
    }
    fn duplicate_scheme(&mut self, source: &str) -> Result<String> {
        let guid = format!("dryrun-duplicate-of-{source}");
        self.record(format!("WOULD DUPLICATE_SCHEME {source}={guid}"))?;
        Ok(guid)
    }
    fn bcd_get(&self, name: &str) -> Result<Option<String>> {
        self.inner.bcd_get(name)
    }
    fn bcd_set(&mut self, name: &str, value: &str) -> Result<()> {
        self.record(format!("WOULD BCD_SET {name}={value}"))
    }
    fn bcd_delete(&mut self, name: &str) -> Result<()> {
        self.record(format!("WOULD BCD_DELETE {name}="))
    }
    fn appx_remove(&mut self, pattern: &str) -> Result<u32> {
        self.record(format!("WOULD APPX_REMOVE {pattern}="))?;
        Ok(0)
    }
    fn cleanup(&mut self, t: CleanupTarget) -> Result<u64> {
        self.record(format!("WOULD CLEANUP {t:?}="))?;
        Ok(0)
    }
    fn devices(&self, class: DeviceClass) -> Result<Vec<String>> {
        self.inner.devices(class)
    }
    fn nic_class_keys(&self) -> Result<Vec<String>> {
        self.inner.nic_class_keys()
    }
}
