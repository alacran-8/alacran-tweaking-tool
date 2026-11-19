//! Reversible apply / revert / state detection on top of `SystemBackend`.
//!
//! Apply captures the prior state of every target, saves the session (no write
//! happens if that save fails), then writes. Revert restores entries newest
//! first and leaves values alone that changed after the session wrote them.

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::backend::SystemBackend;
use crate::catalog::{Catalog, Hive, Op, Reboot, RegData, Tweak};
use crate::store::{now_stamp, BackupStore, SCHEMA};
use crate::sysinfo::SystemInfo;

pub use crate::store::{BackupEntry, PriorState, Session, SessionSummary};

/// Device instance keys live under `HKLM\<ENUM_ROOT>\<instance id>`.
const ENUM_ROOT: &str = "SYSTEM\\CurrentControlSet\\Enum";
pub const DX_PATH: &str = "Software\\Microsoft\\DirectX\\UserGpuPreferences";
pub const DX_NAME: &str = "DirectXUserGlobalSettings";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub tweak_id: String,
    /// `None` = the tweak's `default_option` (if it has options).
    #[serde(default)]
    pub option_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyReport {
    pub session_id: String,
    /// Tweaks whose every op was written.
    pub applied: usize,
    /// Tweaks not available on this system (build/vendor).
    pub skipped: usize,
    /// `(tweak id, error)` per failed op (or unknown tweak/option).
    pub failed: Vec<(String, String)>,
    /// One-way ops executed (appx removal, cleanup).
    pub one_way: usize,
    pub reboot: Reboot,
    pub bytes_freed: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevertReport {
    pub reverted: usize,
    /// Targets left alone because they changed after the session wrote them.
    pub skipped_newer: Vec<String>,
    /// `(target, error)`.
    pub failed: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TweakState {
    Applied,
    Partial,
    NotApplied,
    Unknown,
}

/// Build range and GPU vendor. Laptop status is not an availability filter.
pub fn is_available(t: &Tweak, info: &SystemInfo) -> bool {
    info.build >= t.min_build
        && t.max_build.is_none_or(|m| info.build <= m)
        && t.vendor.is_none_or(|v| info.gpu_vendors.contains(&v))
}

/// What the one-click button applies: available `in_one_click` tweaks, minus
/// `laptop: Some(false)` tweaks on laptops.
pub fn one_click_selection(c: &Catalog, info: &SystemInfo) -> Vec<Selection> {
    c.tweaks
        .iter()
        .filter(|t| t.in_one_click && is_available(t, info))
        .filter(|t| !(t.laptop == Some(false) && info.is_laptop))
        .map(|t| Selection {
            tweak_id: t.id.clone(),
            option_id: None,
        })
        .collect()
}

/// Top-level ops plus the chosen (or default) option's ops.
fn tweak_ops<'a>(t: &'a Tweak, opt: Option<&str>) -> Result<Vec<&'a Op>> {
    let mut ops: Vec<&Op> = t.ops.iter().collect();
    if let Some(id) = opt.or(t.default_option.as_deref()) {
        let o = t
            .options
            .iter()
            .find(|o| o.id == id)
            .ok_or_else(|| anyhow!("unknown option '{id}'"))?;
        ops.extend(o.ops.iter());
    }
    Ok(ops)
}

fn join_path(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|p| p.trim_matches('\\'))
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\\")
}

/// Expands device/NIC ops per instance and turns a DX setting into the merged
/// `RegSet` (from the current value). Other ops are returned unchanged.
fn resolve(op: &Op, be: &dyn SystemBackend) -> Result<Vec<Op>> {
    Ok(match op {
        Op::DeviceReg {
            class,
            suffix,
            name,
            data,
        } => be
            .devices(*class)?
            .iter()
            .map(|id| Op::RegSet {
                hive: Hive::Hklm,
                path: join_path(&[ENUM_ROOT, id, suffix]),
                name: name.clone(),
                data: data.clone(),
            })
            .collect(),
        // Only adapters whose driver declares the keyword (`Ndi\Params\<keyword>`),
        // as Set-NetAdapterAdvancedProperty enforces; other adapters are not touched.
        Op::NicProperty { keyword, value } => {
            let mut ops = Vec::new();
            for path in be.nic_class_keys()? {
                if be.reg_key_exists(Hive::Hklm, &format!("{path}\\Ndi\\Params\\{keyword}"))? {
                    ops.push(Op::RegSet {
                        hive: Hive::Hklm,
                        path,
                        name: keyword.clone(),
                        data: RegData::Sz(value.clone()),
                    });
                }
            }
            ops
        }
        Op::DxGlobalSetting { key, value } => vec![dx_reg_set(be, key, value)?],
        // Services/tasks vary by SKU, build and driver; a missing one is skipped, not a failure.
        Op::ServiceStart { name, .. } if be.service_start_get(name)?.is_none() => vec![],
        Op::TaskDisable { path } if be.task_enabled_get(path)?.is_none() => vec![],
        other => vec![other.clone()],
    })
}

fn dx_reg_set(be: &dyn SystemBackend, key: &str, value: &str) -> Result<Op> {
    let current = be.reg_get(Hive::Hkcu, DX_PATH, DX_NAME)?;
    Ok(Op::RegSet {
        hive: Hive::Hkcu,
        path: DX_PATH.into(),
        name: DX_NAME.into(),
        data: RegData::Sz(merge_dx(current, key, value)?),
    })
}

/// Non-empty `;`-separated entries of a DX settings string.
fn dx_entries(s: &str) -> impl Iterator<Item = &str> {
    s.split(';').map(str::trim).filter(|p| !p.is_empty())
}

fn dx_key(entry: &str) -> &str {
    entry.split_once('=').map_or(entry, |(k, _)| k)
}

/// Sets `key=value;` in a `k=v;k=v;` string, keeping every other entry and its order.
fn merge_dx(current: Option<RegData>, key: &str, value: &str) -> Result<String> {
    let s = match current {
        None => String::new(),
        Some(RegData::Sz(s)) => s,
        Some(other) => bail!("{DX_NAME} has unexpected type {other:?}"),
    };
    let mine = format!("{key}={value};");
    let mut out = String::new();
    let mut found = false;
    for entry in dx_entries(&s) {
        if dx_key(entry).eq_ignore_ascii_case(key) {
            if !found {
                out.push_str(&mine);
                found = true;
            }
        } else {
            out.push_str(entry);
            out.push(';');
        }
    }
    if !found {
        out.push_str(&mine);
    }
    Ok(out)
}

fn hive_name(h: Hive) -> &'static str {
    match h {
        Hive::Hklm => "HKLM",
        Hive::Hkcu => "HKCU",
    }
}

/// Canonical target id of a backup entry: power values name their plan.
fn entry_target(op: &Op, prior: &PriorState) -> String {
    match (op, prior) {
        (Op::PowerValue { subgroup, setting, .. }, PriorState::Power { scheme, .. }) => {
            format!("power:{scheme}/{subgroup}/{setting}")
        }
        _ => target(op),
    }
}

/// Plan a power entry acts on; older backups recorded none (= the active plan).
fn power_scheme(be: &dyn SystemBackend, prior: &PriorState) -> Result<String> {
    match prior {
        PriorState::Power { scheme, .. } if !scheme.is_empty() => Ok(scheme.clone()),
        _ => be.active_scheme(),
    }
}

/// Canonical target id of a resolved op (compare case-insensitively).
pub fn target(op: &Op) -> String {
    match op {
        Op::RegSet {
            hive, path, name, ..
        }
        | Op::RegDeleteValue { hive, path, name } => {
            format!(
                "reg:{}\\{}\\{name}",
                hive_name(*hive),
                path.trim_matches('\\')
            )
        }
        Op::ServiceStart { name, .. } => format!("svc:{name}"),
        Op::TaskDisable { path } => format!("task:{path}"),
        Op::PowerValue {
            subgroup, setting, ..
        } => format!("power:{subgroup}/{setting}"),
        Op::PowerSchemeDuplicateActivate { .. } => "scheme:active".into(),
        Op::BcdSet { name, .. } | Op::BcdDelete { name } => format!("bcd:{name}"),
        Op::AppxRemove { pattern } => format!("appx:{pattern}"),
        Op::Cleanup { target } => {
            format!(
                "cleanup:{}",
                serde_json::to_value(target)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_default()
            )
        }
        Op::DeviceReg {
            class,
            suffix,
            name,
            ..
        } => format!("device:{class:?}\\{suffix}\\{name}"),
        Op::NicProperty { keyword, .. } => format!("nic:{keyword}"),
        Op::DxGlobalSetting { .. } => format!("reg:HKCU\\{DX_PATH}\\{DX_NAME}"),
    }
}

fn is_one_way(op: &Op) -> bool {
    matches!(op, Op::AppxRemove { .. } | Op::Cleanup { .. })
}

/// Fresh read of a resolved op's target, as JSON. One-way ops read as `null`.
fn read_current(be: &dyn SystemBackend, op: &Op, prior: &PriorState) -> Result<Value> {
    Ok(match op {
        Op::RegSet {
            hive, path, name, ..
        }
        | Op::RegDeleteValue { hive, path, name } => {
            serde_json::to_value(be.reg_get(*hive, path, name)?)?
        }
        Op::ServiceStart { name, .. } => json!(be.service_start_get(name)?),
        Op::TaskDisable { path } => json!(be.task_enabled_get(path)?),
        Op::PowerValue {
            subgroup, setting, ..
        } => {
            let (ac, dc) = be.power_get(&power_scheme(be, prior)?, subgroup, setting)?;
            json!([ac, dc])
        }
        Op::PowerSchemeDuplicateActivate { .. } => json!(be.active_scheme()?),
        Op::BcdSet { name, .. } | Op::BcdDelete { name } => json!(be.bcd_get(name)?),
        Op::AppxRemove { .. } | Op::Cleanup { .. } => Value::Null,
        Op::DeviceReg { .. } | Op::NicProperty { .. } | Op::DxGlobalSetting { .. } => {
            bail!("unresolved op")
        }
    })
}

/// The prior state as JSON in fresh-read shape; `None` for one-way.
fn prior_json(p: &PriorState) -> Option<Value> {
    Some(match p {
        PriorState::Reg { value, .. } => serde_json::to_value(value).ok()?,
        PriorState::Service(v) => json!(v),
        PriorState::Task(v) => json!(v),
        PriorState::Power { ac, dc, .. } => json!([ac, dc]),
        PriorState::Scheme(g) => json!(g),
        PriorState::Bcd(v) => json!(v),
        PriorState::OneWay => return None,
    })
}

/// Registry keys on the way to `path` that do not exist yet, deepest first.
fn missing_keys(be: &dyn SystemBackend, h: Hive, path: &str) -> Result<Vec<String>> {
    let parts: Vec<&str> = path.split('\\').filter(|p| !p.is_empty()).collect();
    let mut out = Vec::new();
    for n in (1..=parts.len()).rev() {
        let key = parts[..n].join("\\");
        if be.reg_key_exists(h, &key)? {
            break;
        }
        out.push(key);
    }
    Ok(out)
}

/// Prior state, keys the op would create, and the intended written value.
fn capture(be: &dyn SystemBackend, op: &Op) -> Result<(PriorState, Vec<String>, Option<Value>)> {
    Ok(match op {
        Op::RegSet {
            hive,
            path,
            name,
            data,
        } => {
            let created = missing_keys(be, *hive, path)?;
            let value = be.reg_get(*hive, path, name)?;
            let prior = PriorState::Reg {
                value,
                key_existed: created.is_empty(),
            };
            (prior, created, Some(serde_json::to_value(data)?))
        }
        Op::RegDeleteValue { hive, path, name } => {
            let key_existed = be.reg_key_exists(*hive, path)?;
            let value = be.reg_get(*hive, path, name)?;
            (
                PriorState::Reg { value, key_existed },
                Vec::new(),
                Some(Value::Null),
            )
        }
        Op::ServiceStart { name, start } => {
            let v = be
                .service_start_get(name)?
                .ok_or_else(|| anyhow!("service not found: {name}"))?;
            (PriorState::Service(Some(v)), Vec::new(), Some(json!(start)))
        }
        Op::TaskDisable { path } => {
            let v = be
                .task_enabled_get(path)?
                .ok_or_else(|| anyhow!("task not found: {path}"))?;
            (PriorState::Task(Some(v)), Vec::new(), Some(json!(false)))
        }
        Op::PowerValue {
            subgroup,
            setting,
            ac,
            dc,
        } => {
            let scheme = be.active_scheme()?;
            let (pac, pdc) = be.power_get(&scheme, subgroup, setting)?;
            let written = json!([ac.unwrap_or(pac), dc.unwrap_or(pdc)]);
            (
                PriorState::Power {
                    scheme,
                    ac: pac,
                    dc: pdc,
                },
                Vec::new(),
                Some(written),
            )
        }
        Op::PowerSchemeDuplicateActivate { .. } => {
            (PriorState::Scheme(be.active_scheme()?), Vec::new(), None)
        }
        Op::BcdSet { name, value } => (
            PriorState::Bcd(be.bcd_get(name)?),
            Vec::new(),
            Some(json!(value)),
        ),
        Op::BcdDelete { name } => (
            PriorState::Bcd(be.bcd_get(name)?),
            Vec::new(),
            Some(Value::Null),
        ),
        Op::AppxRemove { .. } | Op::Cleanup { .. } => (PriorState::OneWay, Vec::new(), None),
        Op::DeviceReg { .. } | Op::NicProperty { .. } | Op::DxGlobalSetting { .. } => {
            bail!("unresolved op")
        }
    })
}

/// Performs a resolved op. Returns bytes freed (cleanup only).
fn write(be: &mut dyn SystemBackend, op: &Op, prior: &PriorState) -> Result<u64> {
    match op {
        Op::RegSet {
            hive,
            path,
            name,
            data,
        } => be.reg_set(*hive, path, name, data)?,
        Op::RegDeleteValue { hive, path, name } => {
            if matches!(prior, PriorState::Reg { value: Some(_), .. }) {
                be.reg_delete_value(*hive, path, name)?;
            }
        }
        Op::ServiceStart { name, start } => be.service_start_set(name, *start)?,
        Op::TaskDisable { path } => be.task_enabled_set(path, false)?,
        Op::PowerValue {
            subgroup,
            setting,
            ac,
            dc,
        } => be.power_set(&power_scheme(&*be, prior)?, subgroup, setting, *ac, *dc)?,
        Op::PowerSchemeDuplicateActivate { source } => {
            let guid = be.duplicate_scheme(source)?;
            be.set_active_scheme(&guid)?;
        }
        Op::BcdSet { name, value } => be.bcd_set(name, value)?,
        Op::BcdDelete { name } => {
            if matches!(prior, PriorState::Bcd(Some(_))) {
                be.bcd_delete(name)?;
            }
        }
        Op::AppxRemove { pattern } => {
            be.appx_remove(pattern)?;
        }
        Op::Cleanup { target } => return be.cleanup(*target),
        Op::DeviceReg { .. } | Op::NicProperty { .. } | Op::DxGlobalSetting { .. } => {
            bail!("unresolved op")
        }
    }
    Ok(0)
}

fn reboot_rank(r: Reboot) -> u8 {
    match r {
        Reboot::None => 0,
        Reboot::Recommended => 1,
        Reboot::Required => 2,
    }
}

fn write_entry(
    be: &mut dyn SystemBackend,
    e: &mut BackupEntry,
    dx: Option<&(String, String)>,
) -> Result<u64> {
    if let Some((key, value)) = dx {
        // Merge onto the current string so several DX keys in one session combine.
        e.op = dx_reg_set(&*be, key, value)?;
        if let Op::RegSet { data, .. } = &e.op {
            e.written = Some(serde_json::to_value(data)?);
        }
    }
    if let PriorState::Power { scheme, .. } = &e.prior {
        // An earlier op in this session (duplicate-activate) may have switched the
        // plan: target the plan active now and take the prior from it.
        if !be.active_scheme()?.eq_ignore_ascii_case(scheme) {
            let (prior, _, written) = capture(&*be, &e.op)?;
            e.prior = prior;
            e.written = written;
            e.target = entry_target(&e.op, &e.prior);
        }
    }
    write(be, &e.op, &e.prior)
}

struct Run<'a> {
    tweak: &'a Tweak,
    failed: bool,
    wrote: bool,
}

/// Applies `sel` as one backup session. Errors only when the backup cannot be
/// saved before writing (nothing written) or updated after writing (the file on
/// disk still lists every capture, so the session stays revertible).
pub fn apply(
    c: &Catalog,
    sel: &[Selection],
    be: &mut dyn SystemBackend,
    store: &BackupStore,
    info: &SystemInfo,
) -> Result<ApplyReport> {
    let (base, created) = now_stamp();
    let id = store.allocate_id(&base);
    let mut report = ApplyReport {
        session_id: id.clone(),
        applied: 0,
        skipped: 0,
        failed: Vec::new(),
        one_way: 0,
        reboot: Reboot::None,
        bytes_freed: 0,
    };

    // Phase 1: capture every prior state; nothing is written yet.
    let mut runs: Vec<Run> = Vec::new();
    let mut entries: Vec<BackupEntry> = Vec::new();
    // Per entry: owning run, and the DX key/value to re-merge right before writing.
    let mut meta: Vec<(usize, Option<(String, String)>)> = Vec::new();
    for s in sel {
        let Some(t) = c.tweaks.iter().find(|t| t.id == s.tweak_id) else {
            report
                .failed
                .push((s.tweak_id.clone(), "unknown tweak".into()));
            continue;
        };
        if !is_available(t, info) {
            report.skipped += 1;
            continue;
        }
        let ops = match tweak_ops(t, s.option_id.as_deref()) {
            Ok(ops) => ops,
            Err(e) => {
                report.failed.push((t.id.clone(), format!("{e:#}")));
                continue;
            }
        };
        let run = runs.len();
        runs.push(Run {
            tweak: t,
            failed: false,
            wrote: false,
        });
        for op in ops {
            let resolved = match resolve(op, &*be) {
                Ok(r) => r,
                Err(e) => {
                    runs[run].failed = true;
                    report
                        .failed
                        .push((t.id.clone(), format!("{}: {e:#}", target(op))));
                    continue;
                }
            };
            let dx = match op {
                Op::DxGlobalSetting { key, value } => Some((key.clone(), value.clone())),
                _ => None,
            };
            for r in resolved {
                let tgt = target(&r);
                match capture(&*be, &r) {
                    Ok((prior, created_keys, written)) => {
                        entries.push(BackupEntry {
                            tweak_id: t.id.clone(),
                            category: t.category.clone(),
                            target: entry_target(&r, &prior),
                            op: r,
                            prior,
                            written,
                            applied: false,
                            error: None,
                            created_keys,
                            reverted: false,
                        });
                        meta.push((run, dx.clone()));
                    }
                    Err(e) => {
                        runs[run].failed = true;
                        report.failed.push((t.id.clone(), format!("{tgt}: {e:#}")));
                    }
                }
            }
        }
    }

    let mut session = Session {
        schema: SCHEMA,
        id,
        created,
        dry_run: store.dry_run,
        interactive_sid: be.interactive_sid(),
        entries,
        reverted: false,
    };
    store
        .create(&session)
        .context("could not save the backup; nothing was changed")?;

    // Phase 2: write.
    for (e, (run, dx)) in session.entries.iter_mut().zip(&meta) {
        match write_entry(be, e, dx.as_ref()) {
            Ok(bytes) => {
                e.applied = true;
                runs[*run].wrote = true;
                report.bytes_freed += bytes;
                if is_one_way(&e.op) {
                    report.one_way += 1;
                } else if !store.dry_run {
                    if let Ok(v) = read_current(&*be, &e.op, &e.prior) {
                        e.written = Some(v);
                    }
                }
            }
            Err(err) => {
                runs[*run].failed = true;
                e.error = Some(format!("{err:#}"));
                report
                    .failed
                    .push((e.tweak_id.clone(), format!("{}: {err:#}", e.target)));
            }
        }
    }

    for r in &runs {
        if !r.failed {
            report.applied += 1;
        }
        if r.wrote && reboot_rank(r.tweak.reboot) > reboot_rank(report.reboot) {
            report.reboot = r.tweak.reboot;
        }
    }
    store.save(&session).context(
        "changes were applied but the backup could not be updated; it can still be reverted",
    )?;
    Ok(report)
}

/// Entries that may have been written and are not restored yet. Entries whose
/// write failed are excluded; `applied: false` without an error means the
/// session was interrupted mid-write, so those are included.
fn pending(e: &BackupEntry) -> bool {
    !e.reverted && !matches!(e.prior, PriorState::OneWay) && (e.applied || e.error.is_none())
}

fn remove_created_keys(be: &mut dyn SystemBackend, e: &BackupEntry) -> Result<()> {
    if let Op::RegSet { hive, .. } | Op::RegDeleteValue { hive, .. } = &e.op {
        for k in &e.created_keys {
            if be.reg_key_exists(*hive, k)? {
                be.reg_delete_key_if_empty(*hive, k)?;
            }
        }
    }
    Ok(())
}

fn restore(be: &mut dyn SystemBackend, e: &BackupEntry) -> Result<()> {
    match (&e.op, &e.prior) {
        (
            Op::RegSet {
                hive, path, name, ..
            }
            | Op::RegDeleteValue { hive, path, name },
            PriorState::Reg { value, .. },
        ) => {
            match value {
                Some(d) => be.reg_set(*hive, path, name, d)?,
                None => {
                    if be.reg_get(*hive, path, name)?.is_some() {
                        be.reg_delete_value(*hive, path, name)?;
                    }
                }
            }
            remove_created_keys(be, e)?;
        }
        (Op::ServiceStart { name, .. }, PriorState::Service(Some(v))) => {
            be.service_start_set(name, *v)?
        }
        (Op::TaskDisable { path }, PriorState::Task(Some(v))) => be.task_enabled_set(path, *v)?,
        (
            Op::PowerValue {
                subgroup, setting, ..
            },
            PriorState::Power { ac, dc, .. },
        ) => {
            let scheme = power_scheme(&*be, &e.prior)?;
            be.power_set(&scheme, subgroup, setting, Some(*ac), Some(*dc))?
        }
        (Op::PowerSchemeDuplicateActivate { .. }, PriorState::Scheme(g)) => {
            be.set_active_scheme(g)?
        }
        (Op::BcdSet { name, .. } | Op::BcdDelete { name }, PriorState::Bcd(v)) => match v {
            Some(v) => be.bcd_set(name, v)?,
            None => be.bcd_delete(name)?,
        },
        (op, prior) => bail!("cannot restore {op:?} from {prior:?}"),
    }
    Ok(())
}

fn revert_session(
    s: &mut Session,
    be: &mut dyn SystemBackend,
    check_newer: bool,
    only: &dyn Fn(&BackupEntry) -> bool,
    rep: &mut RevertReport,
) {
    if s.reverted {
        return;
    }
    let owner = s.interactive_sid.clone();
    let other_user = !owner.eq_ignore_ascii_case(&be.interactive_sid());
    for e in s.entries.iter_mut().rev().filter(|e| pending(e) && only(e)) {
        if other_user && is_hkcu(&e.op) {
            // Restoring would write another user's prior values into this user's hive.
            let msg = format!("kayıt farklı bir kullanıcıya ait: {owner}");
            rep.failed.push((e.target.clone(), msg));
            continue;
        }
        let current = match read_current(&*be, &e.op, &e.prior) {
            Ok(v) => v,
            Err(err) => {
                rep.failed.push((e.target.clone(), format!("{err:#}")));
                continue;
            }
        };
        let result = if prior_json(&e.prior).as_ref() == Some(&current) {
            // Already at the prior value (e.g. an earlier entry on the same target,
            // or never written): only drop keys this entry created.
            remove_created_keys(be, e)
        } else if check_newer && e.written.as_ref() != Some(&current) {
            rep.skipped_newer.push(e.target.clone());
            continue;
        } else {
            restore(be, e)
        };
        match result {
            Ok(()) => {
                e.reverted = true;
                rep.reverted += 1;
            }
            Err(err) => rep.failed.push((e.target.clone(), format!("{err:#}"))),
        }
    }
    s.reverted = !s.entries.iter().any(pending);
}

fn is_hkcu(op: &Op) -> bool {
    matches!(
        op,
        Op::RegSet {
            hive: Hive::Hkcu,
            ..
        } | Op::RegDeleteValue {
            hive: Hive::Hkcu,
            ..
        }
    )
}

fn check_mode(s: &Session, store: &BackupStore) -> Result<()> {
    if s.dry_run != store.dry_run {
        let kind = if s.dry_run { "a dry-run" } else { "a real" };
        bail!(
            "session {} is {kind} session and cannot be reverted in this mode",
            s.id
        );
    }
    Ok(())
}

/// Reverts one session. Values changed after it wrote them are left alone and
/// listed in `skipped_newer`; they stay pending for a later revert.
pub fn revert(id: &str, be: &mut dyn SystemBackend, store: &BackupStore) -> Result<RevertReport> {
    let mut s = store.load(id)?;
    check_mode(&s, store)?;
    let mut rep = RevertReport::default();
    if s.reverted {
        return Ok(rep);
    }
    revert_session(&mut s, be, true, &|_| true, &mut rep);
    store.save(&s)?;
    Ok(rep)
}

/// Reverts every session of this store's mode, newest first, restoring each
/// prior value regardless of later changes, so the end state is the state
/// before the oldest session. Unreadable session files are reported in
/// `failed` as `("session:<file>", error)`.
pub fn revert_all(be: &mut dyn SystemBackend, store: &BackupStore) -> Result<RevertReport> {
    revert_matching(be, store, &|_| true)
}

/// Reverts every pending entry of tweaks in category `cat`, across all
/// sessions of this store's mode, newest first (same rules as `revert_all`).
/// An entry belongs to `cat` when its tweak is in `cat` in the catalog, or,
/// for tweaks no longer in the catalog, when its recorded category is `cat`.
pub fn revert_category(
    cat: &str,
    c: &Catalog,
    be: &mut dyn SystemBackend,
    store: &BackupStore,
) -> Result<RevertReport> {
    let in_cat = |e: &BackupEntry| {
        c.tweaks
            .iter()
            .find(|t| t.id == e.tweak_id)
            .map_or(e.category == cat, |t| t.category == cat)
    };
    revert_matching(be, store, &in_cat)
}

fn revert_matching(
    be: &mut dyn SystemBackend,
    store: &BackupStore,
    only: &dyn Fn(&BackupEntry) -> bool,
) -> Result<RevertReport> {
    let mut rep = RevertReport::default();
    let (sessions, broken) = store.scan()?;
    for (file, err) in broken {
        rep.failed.push((format!("session:{file}"), err));
    }
    for summary in sessions {
        if summary.reverted {
            continue;
        }
        let mut s = match store.load(&summary.id) {
            Ok(s) => s,
            Err(e) => {
                rep.failed
                    .push((format!("session:{}", summary.id), format!("{e:#}")));
                continue;
            }
        };
        if s.dry_run != store.dry_run {
            continue;
        }
        // Leave sessions with nothing to do for this filter untouched.
        if s.entries.iter().any(pending) && !s.entries.iter().any(|e| pending(e) && only(e)) {
            continue;
        }
        revert_session(&mut s, be, false, only, &mut rep);
        if let Err(e) = store.save(&s) {
            rep.failed
                .push((format!("session:{}", s.id), format!("{e:#}")));
        }
    }
    Ok(rep)
}

/// `Some(matches)` for checkable ops, `None` for ops whose result cannot be read back.
fn op_matches(be: &dyn SystemBackend, op: &Op) -> Result<Option<bool>> {
    Ok(Some(match op {
        Op::RegSet {
            hive,
            path,
            name,
            data,
        } => be.reg_get(*hive, path, name)?.as_ref() == Some(data),
        Op::RegDeleteValue { hive, path, name } => be.reg_get(*hive, path, name)?.is_none(),
        Op::ServiceStart { name, start } => match be.service_start_get(name)? {
            Some(v) => v == *start,
            None => return Ok(None),
        },
        Op::TaskDisable { path } => match be.task_enabled_get(path)? {
            Some(v) => !v,
            None => return Ok(None),
        },
        Op::PowerValue {
            subgroup,
            setting,
            ac,
            dc,
        } => {
            let (a, d) = be.power_get(&be.active_scheme()?, subgroup, setting)?;
            ac.is_none_or(|v| v == a) && dc.is_none_or(|v| v == d)
        }
        Op::BcdSet { name, value } => be
            .bcd_get(name)?
            .is_some_and(|v| v.eq_ignore_ascii_case(value)),
        Op::BcdDelete { name } => be.bcd_get(name)?.is_none(),
        Op::DxGlobalSetting { key, value } => match be.reg_get(Hive::Hkcu, DX_PATH, DX_NAME)? {
            Some(RegData::Sz(s)) => dx_entries(&s).any(|e| {
                e.split_once('=')
                    .is_some_and(|(k, v)| k.eq_ignore_ascii_case(key) && v == value)
            }),
            _ => false,
        },
        Op::DeviceReg { .. } | Op::NicProperty { .. } => {
            let resolved = resolve(op, be)?;
            if resolved.is_empty() {
                return Ok(None);
            }
            let mut all = true;
            for r in &resolved {
                all &= op_matches(be, r)?.unwrap_or(true);
            }
            all
        }
        Op::PowerSchemeDuplicateActivate { .. } | Op::AppxRemove { .. } | Op::Cleanup { .. } => {
            return Ok(None)
        }
    }))
}

/// `(matched, total)` over the checkable ops (ops that cannot be read back,
/// e.g. appx removal or cleanup, are left out of both); None when a read fails.
pub fn match_counts(t: &Tweak, opt: Option<&str>, be: &dyn SystemBackend) -> Option<(u32, u32)> {
    let ops = tweak_ops(t, opt).ok()?;
    let (mut matched, mut total) = (0u32, 0u32);
    for op in ops {
        if let Some(m) = op_matches(be, op).ok()? {
            total += 1;
            matched += u32::from(m);
        }
    }
    Some((matched, total))
}

/// Applied when every checkable op matches, NotApplied when none do, Partial in
/// between; Unknown when nothing is checkable or a read fails.
pub fn state_from_counts(counts: Option<(u32, u32)>) -> TweakState {
    match counts {
        None | Some((_, 0)) => TweakState::Unknown,
        Some((m, t)) if m == t => TweakState::Applied,
        Some((0, _)) => TweakState::NotApplied,
        _ => TweakState::Partial,
    }
}

pub fn state(t: &Tweak, opt: Option<&str>, be: &dyn SystemBackend) -> TweakState {
    state_from_counts(match_counts(t, opt, be))
}
