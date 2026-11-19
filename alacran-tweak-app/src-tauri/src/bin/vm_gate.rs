//! Dev-only headless gate runner for the VirtualBox apply/revert test
//! (docs/vm-testing.md). NOT shipped. It writes real system state in
//! `apply` / `revert-*` modes, so run it only inside the throwaway test VM.
//!
//! Modes:
//!   dump <out.json>                 every catalog target -> current value
//!   apply oneclick|all-safe|ids:a,b engine::apply with the real store
//!   revert-all                      engine::revert_all
//!   revert-category <category>      engine::revert_category ("Bu sayfayı geri al")
//!   state                           id -> TweakState (+ matched/total)
//!   compare <a.json> <b.json>       differences; exit 1 if any

use std::collections::BTreeMap;
use std::process::exit;

use alacran_tweak_lib::backend::{SystemBackend, WinBackend};
use alacran_tweak_lib::catalog::{self, Catalog, Hive, Op, RegData, Risk, Tweak};
use alacran_tweak_lib::engine::{self, Selection, DX_NAME, DX_PATH};
use alacran_tweak_lib::store::BackupStore;
use alacran_tweak_lib::sysinfo::{self, SystemInfo};
use anyhow::{bail, Result};
use serde_json::{json, Value};

const BACKUPS: &str = r"C:\ProgramData\AlacranTweak\backups";
const ENUM_ROOT: &str = r"SYSTEM\CurrentControlSet\Enum";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let code = match run(&a) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e:#}");
            3
        }
    };
    exit(code);
}

fn run(a: &[&str]) -> Result<i32> {
    let c = catalog::load_embedded();
    match a {
        ["dump", out] => {
            let map = dump(&c, &WinBackend::new());
            std::fs::write(out, serde_json::to_string_pretty(&map)?)?;
            println!("{} targets -> {out}", map.len());
            Ok(0)
        }
        ["apply", sel] => {
            let info = sysinfo::detect();
            let sel = selection(&c, &info, sel)?;
            let mut be = WinBackend::new();
            let rep = engine::apply(&c, &sel, &mut be, &BackupStore::new(BACKUPS), &info)?;
            print(&rep)?;
            Ok(if rep.failed.is_empty() { 0 } else { 2 })
        }
        ["revert-all"] => {
            let rep = engine::revert_all(&mut WinBackend::new(), &BackupStore::new(BACKUPS))?;
            print(&rep)?;
            Ok(if rep.failed.is_empty() { 0 } else { 2 })
        }
        ["revert-category", cat] => {
            let store = BackupStore::new(BACKUPS);
            let rep = engine::revert_category(cat, &c, &mut WinBackend::new(), &store)?;
            print(&rep)?;
            Ok(if rep.failed.is_empty() { 0 } else { 2 })
        }
        ["state"] => {
            let info = sysinfo::detect();
            let be = WinBackend::new();
            let mut out = BTreeMap::new();
            for t in c.tweaks.iter().filter(|t| engine::is_available(t, &info)) {
                let counts = engine::match_counts(t, option_for(t, &info).as_deref(), &be);
                let (m, n) = counts.unwrap_or((0, 0));
                out.insert(
                    t.id.clone(),
                    json!({ "state": engine::state_from_counts(counts), "matched": m, "total": n }),
                );
            }
            print(&json!({ "sysinfo": info, "tweaks": out }))?;
            Ok(0)
        }
        ["compare", x, y] => {
            let diffs = compare(&load(x)?, &load(y)?);
            for d in &diffs {
                println!("{d}");
            }
            println!("{} difference(s)", diffs.len());
            Ok(if diffs.is_empty() { 0 } else { 1 })
        }
        _ => {
            eprintln!("usage: vm_gate dump <out.json> | apply oneclick|all-safe|ids:<a,b> | revert-all | revert-category <cat> | state | compare <a.json> <b.json>");
            Ok(64)
        }
    }
}

fn print(v: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(v)?);
    Ok(())
}

fn load(path: &str) -> Result<BTreeMap<String, Value>> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

/// The option the UI preselects (logic.ts `chosenOptionId`): `ram_gb` picks the
/// smallest "<n>gb" option not below installed RAM, else the largest; otherwise
/// `None` lets the engine use `default_option`.
fn option_for(t: &Tweak, info: &SystemInfo) -> Option<String> {
    if t.auto_option.as_deref() != Some("ram_gb") {
        return None;
    }
    let gb = |id: &str| id.trim_end_matches("gb").parse::<u32>().ok();
    let mut sized: Vec<(u32, &str)> = t
        .options
        .iter()
        .filter_map(|o| gb(&o.id).map(|g| (g, o.id.as_str())))
        .collect();
    sized.sort();
    let fit = sized.iter().find(|(g, _)| *g >= info.ram_gb).or(sized.last());
    fit.map(|(_, id)| id.to_string())
}

fn selection(c: &Catalog, info: &SystemInfo, spec: &str) -> Result<Vec<Selection>> {
    let pick = |t: &Tweak| Selection {
        tweak_id: t.id.clone(),
        option_id: option_for(t, info),
    };
    Ok(match spec {
        "oneclick" => engine::one_click_selection(c, info),
        "all-safe" => c
            .tweaks
            .iter()
            .filter(|t| t.risk == Risk::Safe && !t.one_way && engine::is_available(t, info))
            .map(pick)
            .collect(),
        _ => match spec.strip_prefix("ids:") {
            Some(list) => list
                .split(',')
                .map(|id| match c.tweaks.iter().find(|t| t.id == id.trim()) {
                    Some(t) => Ok(pick(t)),
                    None => bail!("unknown tweak id '{id}'"),
                })
                .collect::<Result<_>>()?,
            None => bail!("unknown selection '{spec}'"),
        },
    })
}

/// Every op of every tweak (all options), expanded per device / NIC like the
/// engine's `resolve`, keyed by `engine::target`. One-way ops have no state.
fn dump(c: &Catalog, be: &dyn SystemBackend) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    for t in &c.tweaks {
        let ops = t.ops.iter().chain(t.options.iter().flat_map(|o| &o.ops));
        for op in ops {
            match expand(op, be) {
                Ok(list) => {
                    for r in list {
                        let v = read(be, &r).unwrap_or_else(|e| json!({ "error": format!("{e:#}") }));
                        map.insert(engine::target(&r), v);
                    }
                }
                Err(e) => {
                    map.insert(engine::target(op), json!({ "error": format!("{e:#}") }));
                }
            }
        }
    }
    map
}

fn reg(path: String, name: &str) -> Op {
    Op::RegSet {
        hive: Hive::Hklm,
        path,
        name: name.into(),
        data: RegData::Dword(0),
    }
}

fn expand(op: &Op, be: &dyn SystemBackend) -> Result<Vec<Op>> {
    Ok(match op {
        Op::DeviceReg {
            class, suffix, name, ..
        } => be
            .devices(*class)?
            .iter()
            .map(|id| {
                let parts = [ENUM_ROOT, id.as_str(), suffix.as_str()];
                let p: Vec<&str> = parts.iter().map(|p| p.trim_matches('\\')).filter(|p| !p.is_empty()).collect();
                reg(p.join("\\"), name)
            })
            .collect(),
        Op::NicProperty { keyword, .. } => {
            let mut out = Vec::new();
            for path in be.nic_class_keys()? {
                if be.reg_key_exists(Hive::Hklm, &format!("{path}\\Ndi\\Params\\{keyword}"))? {
                    out.push(reg(path, keyword));
                }
            }
            out
        }
        Op::DxGlobalSetting { .. } => vec![Op::RegSet {
            hive: Hive::Hkcu,
            path: DX_PATH.into(),
            name: DX_NAME.into(),
            data: RegData::Dword(0),
        }],
        Op::AppxRemove { .. } | Op::Cleanup { .. } => vec![],
        other => vec![other.clone()],
    })
}

fn read(be: &dyn SystemBackend, op: &Op) -> Result<Value> {
    Ok(match op {
        Op::RegSet {
            hive, path, name, ..
        }
        | Op::RegDeleteValue { hive, path, name } => serde_json::to_value(be.reg_get(*hive, path, name)?)?,
        Op::ServiceStart { name, .. } => json!(be.service_start_get(name)?),
        Op::TaskDisable { path } => json!(be.task_enabled_get(path)?),
        Op::PowerValue {
            subgroup, setting, ..
        } => {
            let scheme = be.active_scheme()?;
            json!(be.power_get(&scheme, subgroup, setting)?)
        }
        Op::PowerSchemeDuplicateActivate { .. } => json!(be.active_scheme()?),
        Op::BcdSet { name, .. } | Op::BcdDelete { name } => json!(be.bcd_get(name)?),
        _ => bail!("not readable"),
    })
}

fn one_way_target(k: &str) -> bool {
    k.starts_with("appx:") || k.starts_with("cleanup:")
}

fn compare(a: &BTreeMap<String, Value>, b: &BTreeMap<String, Value>) -> Vec<String> {
    let keys: std::collections::BTreeSet<&String> = a.keys().chain(b.keys()).collect();
    keys.into_iter()
        .filter(|k| !one_way_target(k))
        .filter_map(|k| {
            let (x, y) = (a.get(k), b.get(k));
            (x != y).then(|| format!("{k}: {} -> {}", show(x), show(y)))
        })
        .collect()
}

fn show(v: Option<&Value>) -> String {
    v.map_or("<absent>".into(), Value::to_string)
}

