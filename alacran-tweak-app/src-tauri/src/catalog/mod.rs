//! Declarative tweak catalog: types, embedded loader and validator.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hive {
    Hklm,
    Hkcu,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum RegData {
    Dword(u32),
    Qword(u64),
    Sz(String),
    ExpandSz(String),
    MultiSz(Vec<String>),
    /// Hex string, even length.
    Binary(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Safe,
    Caution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reboot {
    None,
    Recommended,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vendor {
    Nvidia,
    Amd,
    Intel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceClass {
    Usb,
    Net,
    Display,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupTarget {
    UserTemp,
    WindowsTemp,
    UpdateDownload,
    ThumbCache,
    RecycleBin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum Op {
    RegSet { hive: Hive, path: String, name: String, data: RegData },
    RegDeleteValue { hive: Hive, path: String, name: String },
    ServiceStart { name: String, start: u32 },
    TaskDisable { path: String },
    PowerValue { subgroup: String, setting: String, ac: Option<u32>, dc: Option<u32> },
    PowerSchemeDuplicateActivate { source: String },
    BcdSet { name: String, value: String },
    BcdDelete { name: String },
    AppxRemove { pattern: String },
    Cleanup { target: CleanupTarget },
    DeviceReg { class: DeviceClass, suffix: String, name: String, data: RegData },
    NicProperty { keyword: String, value: String },
    DxGlobalSetting { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub tr: String,
    pub en: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TweakOption {
    pub id: String,
    pub label: Text,
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tweak {
    pub id: String,
    pub category: String,
    pub title: Text,
    pub description: Text,
    pub risk: Risk,
    pub in_one_click: bool,
    pub new: bool,
    pub reboot: Reboot,
    pub one_way: bool,
    pub min_build: u32,
    #[serde(default)]
    pub max_build: Option<u32>,
    #[serde(default)]
    pub vendor: Option<Vendor>,
    /// One-click gate only, never a visibility filter: `Some(false)` = left out of
    /// the one-click selection on laptops (still offered and applicable when the
    /// user selects it explicitly). `None`/`Some(true)` = no laptop restriction.
    #[serde(default)]
    pub laptop: Option<bool>,
    #[serde(default)]
    pub caution_note: Option<Text>,
    #[serde(default)]
    pub ops: Vec<Op>,
    #[serde(default)]
    pub options: Vec<TweakOption>,
    #[serde(default)]
    pub default_option: Option<String>,
    /// Automatic option selector, e.g. `"ram_gb"`.
    #[serde(default)]
    pub auto_option: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Catalog {
    pub tweaks: Vec<Tweak>,
}

pub const CATEGORIES: &[&str] = &[
    "network", "power", "cpu", "gpu", "game", "input", "visual", "performance", "storage", "usb",
    "services", "privacy", "cleanup",
];

pub const MIN_SUPPORTED_BUILD: u32 = 19044;

/// Every embedded catalog file. Add new catalog files here only.
const CATALOG_FILES: &[&str] = &[
    include_str!("../../../catalog/new.json"),
    include_str!("../../../catalog/network.json"),
    include_str!("../../../catalog/power.json"),
    include_str!("../../../catalog/cpu.json"),
    include_str!("../../../catalog/gpu.json"),
    include_str!("../../../catalog/game.json"),
    include_str!("../../../catalog/input.json"),
    include_str!("../../../catalog/visual.json"),
    include_str!("../../../catalog/performance.json"),
    include_str!("../../../catalog/storage.json"),
    include_str!("../../../catalog/usb.json"),
    include_str!("../../../catalog/services.json"),
    include_str!("../../../catalog/privacy.json"),
    include_str!("../../../catalog/cleanup.json"),
    include_str!("../../../catalog/bcd.json"),
];

pub fn load_embedded() -> Catalog {
    let mut all = Catalog::default();
    for (i, src) in CATALOG_FILES.iter().enumerate() {
        let part: Catalog = serde_json::from_str(src)
            .unwrap_or_else(|e| panic!("embedded catalog file #{i} is invalid JSON: {e}"));
        all.tweaks.extend(part.tweaks);
    }
    all
}

fn is_hex(s: &str) -> bool {
    s.len() % 2 == 0 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn validate_data(id: &str, data: &RegData, errs: &mut Vec<String>) {
    if let RegData::Binary(h) = data {
        if !is_hex(h) {
            errs.push(format!("{id}: binary data must be even-length hex"));
        }
    }
}

fn validate_ops(id: &str, ops: &[Op], errs: &mut Vec<String>) {
    for op in ops {
        match op {
            Op::RegSet { path, data, .. } => {
                if path.trim().is_empty() {
                    errs.push(format!("{id}: reg_set has empty path"));
                }
                validate_data(id, data, errs);
            }
            Op::RegDeleteValue { path, .. } => {
                if path.trim().is_empty() {
                    errs.push(format!("{id}: reg_delete_value has empty path"));
                }
            }
            Op::DeviceReg { data, .. } => validate_data(id, data, errs),
            _ => {}
        }
    }
}

fn needs_one_way(ops: &[Op]) -> bool {
    ops.iter()
        .any(|o| matches!(o, Op::AppxRemove { .. } | Op::Cleanup { .. }))
}

fn blank(t: &Text) -> bool {
    t.tr.trim().is_empty() || t.en.trim().is_empty()
}

/// Returns a list of human-readable problems; empty means the catalog is valid.
pub fn validate(c: &Catalog) -> Vec<String> {
    let mut errs = Vec::new();
    let mut seen = HashSet::new();
    for t in &c.tweaks {
        let id = t.id.as_str();
        if id.trim().is_empty() {
            errs.push("tweak with empty id".to_string());
        } else if !seen.insert(id) {
            errs.push(format!("{id}: duplicate id"));
        }
        if !CATEGORIES.contains(&t.category.as_str()) {
            errs.push(format!("{id}: unknown category '{}'", t.category));
        }
        if blank(&t.title) {
            errs.push(format!("{id}: title is missing a translation"));
        }
        if blank(&t.description) {
            errs.push(format!("{id}: description is missing a translation"));
        }
        if t.caution_note.as_ref().is_some_and(blank) {
            errs.push(format!("{id}: caution_note is missing a translation"));
        }
        if t.risk == Risk::Caution && t.in_one_click {
            errs.push(format!("{id}: caution tweak must not be in_one_click"));
        }
        if t.min_build < MIN_SUPPORTED_BUILD {
            errs.push(format!("{id}: min_build {} is below {MIN_SUPPORTED_BUILD}", t.min_build));
        }
        if t.ops.is_empty() && t.options.is_empty() {
            errs.push(format!("{id}: tweak has neither ops nor options"));
        }
        validate_ops(id, &t.ops, &mut errs);
        let mut opt_ids = HashSet::new();
        let mut one_way_op = needs_one_way(&t.ops);
        for o in &t.options {
            if !opt_ids.insert(o.id.as_str()) {
                errs.push(format!("{id}: duplicate option id '{}'", o.id));
            }
            if blank(&o.label) {
                errs.push(format!("{id}: option '{}' label is missing a translation", o.id));
            }
            validate_ops(id, &o.ops, &mut errs);
            one_way_op |= needs_one_way(&o.ops);
        }
        if one_way_op && !t.one_way {
            errs.push(format!("{id}: appx_remove/cleanup ops require one_way = true"));
        }
        if let Some(d) = &t.default_option {
            if !opt_ids.contains(d.as_str()) {
                errs.push(format!("{id}: default_option '{d}' does not exist in options"));
            }
        }
    }
    errs
}
