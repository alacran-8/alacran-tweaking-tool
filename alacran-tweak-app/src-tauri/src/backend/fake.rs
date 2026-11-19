use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, bail};

use super::{Result, SystemBackend};
use crate::catalog::{CleanupTarget, DeviceClass, Hive, RegData};

/// In-memory backend for tests. Registry key paths and value names are
/// case-insensitive (both stored lower-cased), as on Windows.
#[derive(Debug, Clone, Default)]
pub struct FakeBackend {
    /// (normalized full key, lower-cased value name) -> data
    pub reg: BTreeMap<(String, String), RegData>,
    /// Normalized full key paths that exist (including parents).
    pub keys: BTreeSet<String>,
    pub services: BTreeMap<String, u32>,
    pub tasks: BTreeMap<String, bool>,
    /// (scheme, subgroup, setting) -> (ac, dc)
    pub power: BTreeMap<(String, String, String), (u32, u32)>,
    pub active: String,
    pub bcd: BTreeMap<String, String>,
    pub devices: Vec<(DeviceClass, String)>,
    pub nic_keys: Vec<String>,
    /// Full key paths (case-insensitive) whose writes fail with "access denied".
    pub fail_on: Vec<String>,
    pub sid: String,
    /// Human-readable line per successful setter call.
    pub writes: Vec<String>,
}

impl FakeBackend {
    pub fn new() -> Self {
        Self { sid: "S-1-5-21-0-0-0-1001".into(), ..Default::default() }
    }

    /// Full key path as the backend sees it, lower-cased for comparisons.
    pub fn full_key(&self, h: Hive, path: &str) -> String {
        let path = path.trim_matches('\\');
        let full = match h {
            Hive::Hklm => format!("HKLM\\{path}"),
            Hive::Hkcu => format!("HKU\\{}\\{path}", self.sid),
        };
        full.trim_end_matches('\\').to_lowercase()
    }

    /// Puts a value in place without recording a write (test setup).
    pub fn seed_reg(&mut self, h: Hive, path: &str, name: &str, data: RegData) {
        let key = self.full_key(h, path);
        self.insert_key_chain(&key);
        self.reg.insert((key, name.to_lowercase()), data);
    }

    fn check_write(&self, key: &str) -> Result<()> {
        if self.fail_on.iter().any(|f| f.trim_matches('\\').to_lowercase() == key) {
            bail!("access denied: {key}");
        }
        Ok(())
    }

    /// Records `key` and its parents, but not the hive root (`hklm`, `hku\<sid>`),
    /// which always exists on Windows and is never created or deleted.
    fn insert_key_chain(&mut self, key: &str) {
        let root_depth = if key.starts_with("hku\\") { 2 } else { 1 };
        let mut acc = String::new();
        for (depth, part) in key.split('\\').enumerate() {
            if !acc.is_empty() {
                acc.push('\\');
            }
            acc.push_str(part);
            if depth >= root_depth {
                self.keys.insert(acc.clone());
            }
        }
    }
}

impl SystemBackend for FakeBackend {
    fn interactive_sid(&self) -> String {
        self.sid.clone()
    }

    fn reg_get(&self, h: Hive, path: &str, name: &str) -> Result<Option<RegData>> {
        Ok(self.reg.get(&(self.full_key(h, path), name.to_lowercase())).cloned())
    }

    fn reg_key_exists(&self, h: Hive, path: &str) -> Result<bool> {
        Ok(self.keys.contains(&self.full_key(h, path)))
    }

    fn reg_set(&mut self, h: Hive, path: &str, name: &str, data: &RegData) -> Result<()> {
        let key = self.full_key(h, path);
        self.check_write(&key)?;
        self.insert_key_chain(&key);
        self.reg.insert((key.clone(), name.to_lowercase()), data.clone());
        self.writes.push(format!("reg_set {key}\\{name}={data:?}"));
        Ok(())
    }

    fn reg_delete_value(&mut self, h: Hive, path: &str, name: &str) -> Result<()> {
        let key = self.full_key(h, path);
        self.check_write(&key)?;
        self.reg.remove(&(key.clone(), name.to_lowercase()));
        self.writes.push(format!("reg_delete_value {key}\\{name}"));
        Ok(())
    }

    fn reg_delete_key_if_empty(&mut self, h: Hive, path: &str) -> Result<()> {
        let key = self.full_key(h, path);
        self.check_write(&key)?;
        let prefix = format!("{key}\\");
        let has_values = self.reg.keys().any(|(k, _)| *k == key);
        let has_subkeys = self.keys.iter().any(|k| k.starts_with(&prefix));
        if !has_values && !has_subkeys && self.keys.remove(&key) {
            self.writes.push(format!("reg_delete_key {key}"));
        }
        Ok(())
    }

    fn service_start_get(&self, name: &str) -> Result<Option<u32>> {
        Ok(self.services.get(name).copied())
    }

    fn service_start_set(&mut self, name: &str, start: u32) -> Result<()> {
        self.services.insert(name.to_string(), start);
        self.writes.push(format!("service_start_set {name}={start}"));
        Ok(())
    }

    fn task_enabled_get(&self, path: &str) -> Result<Option<bool>> {
        Ok(self.tasks.get(path).copied())
    }

    fn task_enabled_set(&mut self, path: &str, enabled: bool) -> Result<()> {
        self.tasks.insert(path.to_string(), enabled);
        self.writes.push(format!("task_enabled_set {path}={enabled}"));
        Ok(())
    }

    fn power_get(&self, scheme: &str, subgroup: &str, setting: &str) -> Result<(u32, u32)> {
        self.power
            .get(&(scheme.to_string(), subgroup.to_string(), setting.to_string()))
            .copied()
            .ok_or_else(|| anyhow!("power setting not found: {scheme}/{subgroup}/{setting}"))
    }

    fn power_set(
        &mut self,
        scheme: &str,
        subgroup: &str,
        setting: &str,
        ac: Option<u32>,
        dc: Option<u32>,
    ) -> Result<()> {
        let e = self
            .power
            .get_mut(&(scheme.to_string(), subgroup.to_string(), setting.to_string()))
            .ok_or_else(|| anyhow!("power setting not found: {scheme}/{subgroup}/{setting}"))?;
        if let Some(v) = ac {
            e.0 = v;
        }
        if let Some(v) = dc {
            e.1 = v;
        }
        self.writes.push(format!("power_set {scheme}/{subgroup}/{setting} ac={ac:?} dc={dc:?}"));
        Ok(())
    }

    fn active_scheme(&self) -> Result<String> {
        Ok(self.active.clone())
    }

    fn set_active_scheme(&mut self, guid: &str) -> Result<()> {
        self.active = guid.to_string();
        self.writes.push(format!("set_active_scheme {guid}"));
        Ok(())
    }

    fn duplicate_scheme(&mut self, source: &str) -> Result<String> {
        let n = self.writes.len();
        let guid = format!("fake-copy-{n}-of-{source}");
        // Like PowerDuplicateScheme: the copy starts with the source's values.
        let copied: Vec<_> = self
            .power
            .iter()
            .filter(|((s, _, _), _)| s == source)
            .map(|((_, g, k), v)| ((guid.clone(), g.clone(), k.clone()), *v))
            .collect();
        self.power.extend(copied);
        self.writes.push(format!("duplicate_scheme {source} -> {guid}"));
        Ok(guid)
    }

    fn bcd_get(&self, name: &str) -> Result<Option<String>> {
        Ok(self.bcd.get(name).cloned())
    }

    fn bcd_set(&mut self, name: &str, value: &str) -> Result<()> {
        self.bcd.insert(name.to_string(), value.to_string());
        self.writes.push(format!("bcd_set {name}={value}"));
        Ok(())
    }

    fn bcd_delete(&mut self, name: &str) -> Result<()> {
        self.bcd.remove(name);
        self.writes.push(format!("bcd_delete {name}"));
        Ok(())
    }

    fn appx_remove(&mut self, pattern: &str) -> Result<u32> {
        self.writes.push(format!("appx_remove {pattern}"));
        Ok(0)
    }

    fn cleanup(&mut self, t: CleanupTarget) -> Result<u64> {
        self.writes.push(format!("cleanup {t:?}"));
        Ok(0)
    }

    fn devices(&self, class: DeviceClass) -> Result<Vec<String>> {
        Ok(self.devices.iter().filter(|(c, _)| *c == class).map(|(_, id)| id.clone()).collect())
    }

    fn nic_class_keys(&self) -> Result<Vec<String>> {
        Ok(self.nic_keys.clone())
    }
}
