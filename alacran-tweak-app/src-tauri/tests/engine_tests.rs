mod backend {
    use alacran_tweak_lib::backend::{DryRunBackend, FakeBackend, SystemBackend};
    use alacran_tweak_lib::catalog::{Hive, RegData};

    fn fake() -> FakeBackend {
        let mut f = FakeBackend::new();
        f.sid = "S-1-5-21-X".into();
        f
    }

    #[test]
    fn hkcu_resolves_to_interactive_user_hive() {
        let mut f = fake();
        f.reg_set(Hive::Hkcu, "Software\\A", "v", &RegData::Dword(1)).unwrap();
        let key = "hku\\s-1-5-21-x\\software\\a".to_string();
        assert_eq!(f.reg.get(&(key.clone(), "v".into())), Some(&RegData::Dword(1)));
        assert!(f.keys.contains(&key));
        assert!(f.keys.contains("hku\\s-1-5-21-x\\software"));
        // case-insensitive lookup
        assert_eq!(f.reg_get(Hive::Hkcu, "SOFTWARE\\a", "v").unwrap(), Some(RegData::Dword(1)));
        assert_eq!(f.writes.len(), 1);
    }

    #[test]
    fn hklm_set_delete_and_key_cleanup() {
        let mut f = fake();
        f.reg_set(Hive::Hklm, "S\\K", "v", &RegData::Dword(1)).unwrap();
        f.reg_delete_key_if_empty(Hive::Hklm, "S\\K").unwrap();
        assert!(f.reg_key_exists(Hive::Hklm, "S\\K").unwrap(), "has a value, must stay");
        f.reg_delete_key_if_empty(Hive::Hklm, "S").unwrap();
        assert!(f.reg_key_exists(Hive::Hklm, "S").unwrap(), "has a subkey, must stay");
        f.reg_delete_value(Hive::Hklm, "S\\K", "v").unwrap();
        f.reg_delete_key_if_empty(Hive::Hklm, "S\\K").unwrap();
        assert!(!f.reg_key_exists(Hive::Hklm, "S\\K").unwrap());
    }

    #[test]
    fn dryrun_never_writes() {
        let mut d = DryRunBackend::new(fake(), None);
        d.reg_set(Hive::Hklm, "Software\\A", "v", &RegData::Dword(1)).unwrap();
        assert!(d.inner().reg.is_empty());
        assert!(d.inner().writes.is_empty());
        assert_eq!(d.log.len(), 1);
        assert!(d.log[0].contains("WOULD"));
    }

    #[test]
    fn dryrun_appends_to_log_file() {
        let dir = std::env::temp_dir().join(format!("alacran-dryrun-{}", std::process::id()));
        let path = dir.join("apply.log");
        let mut d = DryRunBackend::new(fake(), Some(path.clone()));
        d.bcd_set("useplatformclock", "no").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("WOULD BCD_SET useplatformclock=no"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fake_fail_on_returns_access_denied() {
        let mut f = fake();
        f.reg_set(Hive::Hklm, "Software\\Locked", "v", &RegData::Dword(0)).unwrap();
        f.fail_on.push("hklm\\SOFTWARE\\locked".into());
        let err = f.reg_set(Hive::Hklm, "Software\\Locked", "v", &RegData::Dword(1)).unwrap_err();
        assert!(err.to_string().contains("access denied"));
        assert_eq!(f.reg_get(Hive::Hklm, "Software\\Locked", "v").unwrap(), Some(RegData::Dword(0)));
    }

    #[test]
    fn backend_is_object_safe_and_send() {
        fn takes(_: &mut dyn SystemBackend) {}
        fn is_send<T: Send>() {}
        is_send::<Box<dyn SystemBackend>>();
        takes(&mut fake());
    }

    #[test]
    fn fake_value_names_are_case_insensitive() {
        let mut f = fake();
        f.seed_reg(Hive::Hklm, "Software\\A", "Value", RegData::Dword(7));
        assert!(f.writes.is_empty(), "seeding is not a write");
        assert_eq!(f.reg_get(Hive::Hklm, "software\\a", "VALUE").unwrap(), Some(RegData::Dword(7)));
        f.reg_set(Hive::Hklm, "Software\\A", "value", &RegData::Dword(8)).unwrap();
        assert_eq!(f.reg.len(), 1);
        f.reg_delete_value(Hive::Hklm, "Software\\A", "VaLuE").unwrap();
        assert!(f.reg.is_empty());
    }

    #[test]
    fn fake_power_set_unknown_setting_errors() {
        let mut f = fake();
        assert!(f.power_set("a", "sub", "missing", Some(1), None).is_err());
        assert!(f.power.is_empty());
    }
}

mod engine {
    use std::sync::atomic::{AtomicU32, Ordering};

    use alacran_tweak_lib::backend::{DryRunBackend, FakeBackend, SystemBackend};
    use alacran_tweak_lib::catalog::*;
    use alacran_tweak_lib::engine::*;
    use alacran_tweak_lib::store::BackupStore;
    use alacran_tweak_lib::sysinfo::SystemInfo;
    use serde_json::{json, Value};

    static N: AtomicU32 = AtomicU32::new(0);

    /// A backup store in a fresh temp directory, removed on drop.
    struct Tmp {
        store: BackupStore,
    }

    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.store.dir);
        }
    }

    fn tmp() -> Tmp {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("alacran-store-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Tmp { store: BackupStore::new(dir) }
    }

    fn tweak_with(id: &str, ops: Value, extra: Value) -> Tweak {
        let mut v = json!({
            "id": id, "category": "performance",
            "title": { "tr": "t", "en": "t" }, "description": { "tr": "d", "en": "d" },
            "risk": "safe", "in_one_click": true, "new": false, "reboot": "none",
            "one_way": false, "min_build": 19044, "ops": ops
        });
        for (k, val) in extra.as_object().unwrap() {
            v[k] = val.clone();
        }
        serde_json::from_value(v).unwrap()
    }

    fn tweak(id: &str, ops: Value) -> Tweak {
        tweak_with(id, ops, json!({}))
    }

    fn cat(tweaks: Vec<Tweak>) -> Catalog {
        Catalog { tweaks }
    }

    fn sel(id: &str) -> Selection {
        Selection { tweak_id: id.into(), option_id: None }
    }

    fn info() -> SystemInfo {
        SystemInfo { build: 22631, ..Default::default() }
    }

    fn reg(path: &str, name: &str, v: u32) -> Value {
        json!({ "op": "reg_set", "hive": "hklm", "path": path, "name": name,
                "data": { "type": "dword", "value": v } })
    }

    fn get(f: &FakeBackend, path: &str, name: &str) -> Option<RegData> {
        f.reg_get(Hive::Hklm, path, name).unwrap()
    }

    type Snap = (
        std::collections::BTreeMap<(String, String), RegData>,
        std::collections::BTreeSet<String>,
        std::collections::BTreeMap<String, u32>,
        std::collections::BTreeMap<String, bool>,
        std::collections::BTreeMap<(String, String, String), (u32, u32)>,
        String,
        std::collections::BTreeMap<String, String>,
    );

    fn snap(f: &FakeBackend) -> Snap {
        (
            f.reg.clone(),
            f.keys.clone(),
            f.services.clone(),
            f.tasks.clone(),
            f.power.clone(),
            f.active.clone(),
            f.bcd.clone(),
        )
    }

    const DX_PATH: &str = "Software\\Microsoft\\DirectX\\UserGpuPreferences";
    const DX_NAME: &str = "DirectXUserGlobalSettings";

    fn dx(key: &str) -> Value {
        json!({ "op": "dx_global_setting", "key": key, "value": "1" })
    }

    #[test]
    fn apply_then_revert_restores_exact_state() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.seed_reg(Hive::Hklm, "SOFTWARE\\T", "D", RegData::Dword(0));
        f.services.insert("SysMain".into(), 3);
        f.tasks.insert("\\Microsoft\\T".into(), true);
        f.power.insert(("balanced".into(), "sub".into(), "set".into()), (5, 5));
        f.active = "balanced".into();
        f.bcd.insert("useplatformtick".into(), "no".into());
        let c = cat(vec![tweak_with(
            "all",
            json!([
                reg("SOFTWARE\\T", "D", 1),
                { "op": "service_start", "name": "SysMain", "start": 4 },
                { "op": "task_disable", "path": "\\Microsoft\\T" },
                { "op": "power_value", "subgroup": "sub", "setting": "set", "ac": 0, "dc": 1 },
                { "op": "bcd_set", "name": "useplatformtick", "value": "yes" },
                { "op": "bcd_set", "name": "disabledynamictick", "value": "yes" },
                { "op": "power_scheme_duplicate_activate", "source": "balanced" }
            ]),
            json!({ "reboot": "required" }),
        )]);
        let before = snap(&f);

        let r = apply(&c, &[sel("all")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!((r.applied, r.skipped, r.one_way), (1, 0, 0));
        assert_eq!(r.reboot, Reboot::Required);
        assert_eq!(get(&f, "SOFTWARE\\T", "D"), Some(RegData::Dword(1)));
        assert_eq!(f.services["SysMain"], 4);
        assert!(!f.tasks["\\Microsoft\\T"]);
        assert_eq!(f.power[&("balanced".to_string(), "sub".to_string(), "set".to_string())], (0, 1));
        assert_eq!(f.bcd["useplatformtick"], "yes");
        assert_eq!(f.bcd["disabledynamictick"], "yes");
        assert_ne!(f.active, "balanced");
        assert_eq!(state(&c.tweaks[0], None, &f), TweakState::Applied, "scheme op is not checkable, the rest match");

        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert!(rr.failed.is_empty(), "{:?}", rr.failed);
        assert!(rr.skipped_newer.is_empty(), "{:?}", rr.skipped_newer);
        assert_eq!(rr.reverted, 7);
        // Revert re-activates the old plan; the duplicate plan itself is left behind.
        f.power.retain(|(s, _, _), _| s == "balanced");
        assert_eq!(snap(&f), before);
        assert!(t.store.load(&r.session_id).unwrap().reverted);
    }

    #[test]
    fn missing_service_and_task_are_skipped_not_failed() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.services.insert("Present".into(), 3);
        let c = cat(vec![tweak(
            "svc",
            json!([
                { "op": "service_start", "name": "Missing", "start": 4 },
                { "op": "service_start", "name": "Present", "start": 4 },
                { "op": "task_disable", "path": "\\Missing\\Task" }
            ]),
        )]);
        let r = apply(&c, &[sel("svc")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!(r.applied, 1);
        assert_eq!(f.services["Present"], 4);
        assert!(!f.services.contains_key("Missing"));
        assert_eq!(state(&c.tweaks[0], None, &f), TweakState::Applied);
    }

    #[test]
    fn revert_deletes_value_and_created_key() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.seed_reg(Hive::Hklm, "SOFTWARE\\Existing", "Other", RegData::Dword(9));
        let c = cat(vec![tweak("k", json!([reg("SOFTWARE\\Existing\\New", "V", 1)]))]);
        let r = apply(&c, &[sel("k")], &mut f, &t.store, &info()).unwrap();
        assert!(f.reg_key_exists(Hive::Hklm, "SOFTWARE\\Existing\\New").unwrap());
        revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!(get(&f, "SOFTWARE\\Existing\\New", "V"), None);
        assert!(!f.reg_key_exists(Hive::Hklm, "SOFTWARE\\Existing\\New").unwrap());
        assert!(f.reg_key_exists(Hive::Hklm, "SOFTWARE\\Existing").unwrap(), "pre-existing parent stays");
    }

    #[test]
    fn revert_older_session_skips_values_changed_later() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.seed_reg(Hive::Hklm, "SOFTWARE\\S", "V", RegData::Dword(0));
        let ca = cat(vec![tweak("a", json!([reg("SOFTWARE\\S", "V", 1)]))]);
        let cb = cat(vec![tweak("b", json!([reg("SOFTWARE\\S", "V", 2)]))]);
        let a = apply(&ca, &[sel("a")], &mut f, &t.store, &info()).unwrap();
        let b = apply(&cb, &[sel("b")], &mut f, &t.store, &info()).unwrap();
        assert_ne!(a.session_id, b.session_id);

        let ra = revert(&a.session_id, &mut f, &t.store).unwrap();
        assert_eq!(get(&f, "SOFTWARE\\S", "V"), Some(RegData::Dword(2)));
        assert_eq!(ra.skipped_newer.len(), 1);
        assert_eq!(ra.skipped_newer[0].to_lowercase(), "reg:hklm\\software\\s\\v");
        assert_eq!(ra.reverted, 0);

        let all = revert_all(&mut f, &t.store).unwrap();
        assert!(all.failed.is_empty(), "{:?}", all.failed);
        assert_eq!(get(&f, "SOFTWARE\\S", "V"), Some(RegData::Dword(0)));
        assert!(t.store.list().unwrap().iter().all(|s| s.reverted));
    }

    #[test]
    fn apply_aborts_before_writes_when_backup_save_fails() {
        let t = tmp();
        std::fs::create_dir_all(t.store.dir.parent().unwrap()).unwrap();
        std::fs::write(&t.store.dir, b"i am a file, not a directory").unwrap();
        let mut f = FakeBackend::new();
        let c = cat(vec![tweak("k", json!([reg("SOFTWARE\\K", "V", 1)]))]);
        let r = apply(&c, &[sel("k")], &mut f, &t.store, &info());
        let _ = std::fs::remove_file(&t.store.dir);
        assert!(r.is_err());
        assert!(f.writes.is_empty(), "{:?}", f.writes);
    }

    #[test]
    fn partial_failure_continues_and_is_revertible() {
        let t = tmp();
        let mut f = FakeBackend::new();
        for k in ["A", "B", "C"] {
            f.seed_reg(Hive::Hklm, &format!("SOFTWARE\\{k}"), "V", RegData::Dword(0));
        }
        f.fail_on.push("HKLM\\SOFTWARE\\B".into());
        let c = cat(vec![
            tweak("a", json!([reg("SOFTWARE\\A", "V", 1)])),
            tweak("b", json!([reg("SOFTWARE\\B", "V", 1)])),
            tweak("c", json!([reg("SOFTWARE\\C", "V", 1)])),
        ]);
        let before = snap(&f);
        let r = apply(&c, &[sel("a"), sel("b"), sel("c")], &mut f, &t.store, &info()).unwrap();
        assert_eq!(r.failed.len(), 1);
        assert_eq!(r.failed[0].0, "b");
        assert!(r.failed[0].1.contains("access denied"));
        assert_eq!(r.applied, 2);
        assert_eq!(get(&f, "SOFTWARE\\A", "V"), Some(RegData::Dword(1)));
        assert_eq!(get(&f, "SOFTWARE\\C", "V"), Some(RegData::Dword(1)));

        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!(rr.reverted, 2);
        assert!(rr.failed.is_empty() && rr.skipped_newer.is_empty());
        assert_eq!(snap(&f), before);
    }

    #[test]
    fn one_click_selection_contains_no_caution() {
        let c = load_embedded();
        let s = one_click_selection(&c, &info());
        assert!(!s.is_empty());
        for x in &s {
            let t = c.tweaks.iter().find(|t| t.id == x.tweak_id).unwrap();
            assert_eq!(t.risk, Risk::Safe, "{}", t.id);
            assert!(t.in_one_click, "{}", t.id);
        }
    }

    #[test]
    fn one_click_excludes_laptop_false_on_laptops() {
        let c = load_embedded();
        let desktop = info();
        let laptop = SystemInfo { is_laptop: true, ..info() };
        let has = |s: &[Selection]| s.iter().any(|x| x.tweak_id == "new_ultimate_power");
        assert!(has(&one_click_selection(&c, &desktop)));
        assert!(!has(&one_click_selection(&c, &laptop)));
        let t = c.tweaks.iter().find(|t| t.id == "new_ultimate_power").unwrap();
        assert!(is_available(t, &laptop), "still offered on laptops");

        // Explicitly selected on a laptop: applied, not skipped.
        let tmp = tmp();
        let mut f = FakeBackend::new();
        f.active = "balanced".into();
        let r = apply(&c, &[sel("new_ultimate_power")], &mut f, &tmp.store, &laptop).unwrap();
        assert_eq!((r.applied, r.skipped), (1, 0), "{:?}", r.failed);
        assert_ne!(f.active, "balanced");
    }

    #[test]
    fn state_reports_partial() {
        let mut f = FakeBackend::new();
        let t = tweak(
            "s",
            json!([reg("SOFTWARE\\A", "V", 1), reg("SOFTWARE\\B", "V", 1), reg("SOFTWARE\\C", "V", 1)]),
        );
        assert_eq!(state(&t, None, &f), TweakState::NotApplied);
        f.seed_reg(Hive::Hklm, "SOFTWARE\\A", "V", RegData::Dword(1));
        f.seed_reg(Hive::Hklm, "SOFTWARE\\B", "V", RegData::Dword(1));
        f.seed_reg(Hive::Hklm, "SOFTWARE\\C", "V", RegData::Dword(0));
        assert_eq!(state(&t, None, &f), TweakState::Partial);
        f.seed_reg(Hive::Hklm, "SOFTWARE\\C", "V", RegData::Dword(1));
        assert_eq!(state(&t, None, &f), TweakState::Applied);

        let appx = tweak_with(
            "x",
            json!([{ "op": "appx_remove", "pattern": "Microsoft.Bing*" }]),
            json!({ "one_way": true }),
        );
        assert_eq!(state(&appx, None, &f), TweakState::Unknown);
    }

    #[test]
    fn match_counts_skip_uncheckable_ops() {
        let mut f = FakeBackend::new();
        let t = tweak(
            "c",
            json!([
                reg("SOFTWARE\\A", "V", 1),
                reg("SOFTWARE\\B", "V", 1),
                reg("SOFTWARE\\C", "V", 1),
                { "op": "cleanup", "target": "user_temp" }
            ]),
        );
        assert_eq!(match_counts(&t, None, &f), Some((0, 3)));
        f.seed_reg(Hive::Hklm, "SOFTWARE\\B", "V", RegData::Dword(1));
        assert_eq!(match_counts(&t, None, &f), Some((1, 3)));
        assert_eq!(state(&t, None, &f), TweakState::Partial);
        f.seed_reg(Hive::Hklm, "SOFTWARE\\A", "V", RegData::Dword(1));
        f.seed_reg(Hive::Hklm, "SOFTWARE\\C", "V", RegData::Dword(1));
        assert_eq!(match_counts(&t, None, &f), Some((3, 3)));
        assert_eq!(state_from_counts(Some((3, 3))), TweakState::Applied);
        assert_eq!(state_from_counts(Some((0, 0))), TweakState::Unknown);
        assert_eq!(state_from_counts(None), TweakState::Unknown);
    }

    #[test]
    fn dx_global_setting_merges_string() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.seed_reg(Hive::Hkcu, DX_PATH, DX_NAME, RegData::Sz("VRROptimizeEnable=0;".into()));
        let c = cat(vec![tweak("dx", json!([dx("SwapEffectUpgradeEnable")]))]);
        assert_eq!(state(&c.tweaks[0], None, &f), TweakState::NotApplied);
        let r = apply(&c, &[sel("dx")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!(
            f.reg_get(Hive::Hkcu, DX_PATH, DX_NAME).unwrap(),
            Some(RegData::Sz("VRROptimizeEnable=0;SwapEffectUpgradeEnable=1;".into()))
        );
        assert_eq!(state(&c.tweaks[0], None, &f), TweakState::Applied);
        let s = t.store.load(&r.session_id).unwrap();
        assert_eq!(s.entries[0].target, format!("reg:HKCU\\{DX_PATH}\\{DX_NAME}"));
        revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!(
            f.reg_get(Hive::Hkcu, DX_PATH, DX_NAME).unwrap(),
            Some(RegData::Sz("VRROptimizeEnable=0;".into()))
        );
    }

    #[test]
    fn dx_global_setting_replaces_existing_key_and_creates_missing_value() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.seed_reg(Hive::Hkcu, DX_PATH, DX_NAME, RegData::Sz("SwapEffectUpgradeEnable=0;A=1".into()));
        let c = cat(vec![tweak("dx", json!([dx("SwapEffectUpgradeEnable")]))]);
        apply(&c, &[sel("dx")], &mut f, &t.store, &info()).unwrap();
        assert_eq!(
            f.reg_get(Hive::Hkcu, DX_PATH, DX_NAME).unwrap(),
            Some(RegData::Sz("SwapEffectUpgradeEnable=1;A=1;".into()))
        );

        let mut g = FakeBackend::new();
        let before = snap(&g);
        let r = apply(&c, &[sel("dx")], &mut g, &t.store, &info()).unwrap();
        assert_eq!(
            g.reg_get(Hive::Hkcu, DX_PATH, DX_NAME).unwrap(),
            Some(RegData::Sz("SwapEffectUpgradeEnable=1;".into()))
        );
        revert(&r.session_id, &mut g, &t.store).unwrap();
        assert_eq!(snap(&g), before);
    }

    #[test]
    fn filtered_by_build_and_vendor() {
        let t = tmp();
        let mut f = FakeBackend::new();
        let c = cat(vec![
            tweak_with("future", json!([reg("SOFTWARE\\F", "V", 1)]), json!({ "min_build": 30000 })),
            tweak_with("old", json!([reg("SOFTWARE\\O", "V", 1)]), json!({ "max_build": 19045 })),
            tweak_with("nv", json!([reg("SOFTWARE\\N", "V", 1)]), json!({ "vendor": "nvidia" })),
            tweak_with("amd", json!([reg("SOFTWARE\\A", "V", 1)]), json!({ "vendor": "amd" })),
        ]);
        let i = SystemInfo { gpu_vendors: vec![Vendor::Amd], ..info() };
        let all: Vec<Selection> = ["future", "old", "nv", "amd"].iter().map(|s| sel(s)).collect();
        let r = apply(&c, &all, &mut f, &t.store, &i).unwrap();
        assert_eq!((r.applied, r.skipped), (1, 3));
        assert_eq!(f.writes.len(), 1, "{:?}", f.writes);
        let s = t.store.load(&r.session_id).unwrap();
        assert_eq!(s.entries.len(), 1, "filtered tweaks are not captured");
        assert_eq!(s.entries[0].tweak_id, "amd");
        assert!(!is_available(&c.tweaks[0], &i));
        assert!(!is_available(&c.tweaks[2], &i));
    }

    fn pv(ac: u32, dc: u32) -> Value {
        json!({ "op": "power_value", "subgroup": "sub", "setting": "set", "ac": ac, "dc": dc })
    }

    fn pw(f: &FakeBackend, scheme: &str) -> (u32, u32) {
        f.power[&(scheme.to_string(), "sub".to_string(), "set".to_string())]
    }

    #[test]
    fn power_value_after_plan_switch_reverts_cleanly() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.active = "A".into();
        f.power.insert(("A".into(), "sub".into(), "set".into()), (5, 5));
        f.power.insert(("U".into(), "sub".into(), "set".into()), (7, 7));
        let c = cat(vec![
            tweak("ult", json!([{ "op": "power_scheme_duplicate_activate", "source": "U" }])),
            tweak("p", json!([pv(0, 1)])),
        ]);
        let r = apply(&c, &[sel("ult"), sel("p")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        let new = f.active.clone();
        assert_ne!(new, "A");
        assert_eq!(pw(&f, &new), (0, 1), "written into the plan active at write time");
        assert_eq!(pw(&f, "A"), (5, 5));
        let e = &t.store.load(&r.session_id).unwrap().entries[1];
        assert_eq!(e.target, format!("power:{new}/sub/set"));

        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert!(rr.failed.is_empty(), "{:?}", rr.failed);
        assert!(rr.skipped_newer.is_empty(), "{:?}", rr.skipped_newer);
        assert!(t.store.load(&r.session_id).unwrap().reverted);
        assert_eq!(f.active, "A");
        assert_eq!(pw(&f, "A"), (5, 5));
    }

    #[test]
    fn power_value_revert_after_user_switched_plan() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.active = "A".into();
        f.power.insert(("A".into(), "sub".into(), "set".into()), (5, 5));
        f.power.insert(("B".into(), "sub".into(), "set".into()), (9, 9));
        let c = cat(vec![tweak("p", json!([pv(0, 1)]))]);
        let r = apply(&c, &[sel("p")], &mut f, &t.store, &info()).unwrap();
        assert_eq!(pw(&f, "A"), (0, 1));
        f.active = "B".into();

        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert!(rr.failed.is_empty(), "{:?}", rr.failed);
        assert!(rr.skipped_newer.is_empty(), "{:?}", rr.skipped_newer);
        assert_eq!(rr.reverted, 1);
        assert_eq!(pw(&f, "A"), (5, 5));
        assert_eq!(pw(&f, "B"), (9, 9));
        assert_eq!(f.active, "B", "revert does not switch plans");
    }

    #[test]
    fn old_backup_without_scheme_still_reverts() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.active = "A".into();
        f.power.insert(("A".into(), "sub".into(), "set".into()), (5, 5));
        let c = cat(vec![tweak("p", json!([pv(0, 1)]))]);
        let r = apply(&c, &[sel("p")], &mut f, &t.store, &info()).unwrap();
        // Rewrite the file the way an older version stored it: no `scheme`.
        let path = t.store.dir.join(format!("{}.json", r.session_id));
        let mut v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let state = v["entries"][0]["prior"]["state"].as_object_mut().unwrap();
        assert!(state.remove("scheme").is_some());
        v["entries"][0]["target"] = json!("power:sub/set");
        std::fs::write(&path, serde_json::to_string(&v).unwrap()).unwrap();

        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert!(rr.failed.is_empty(), "{:?}", rr.failed);
        assert!(rr.skipped_newer.is_empty(), "{:?}", rr.skipped_newer);
        assert_eq!(rr.reverted, 1);
        assert_eq!(pw(&f, "A"), (5, 5));
    }

    #[test]
    fn same_target_twice_in_one_session_reverts_exactly() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.power.insert(("".into(), "sub".into(), "set".into()), (5, 5));
        let c = cat(vec![
            tweak("d1", json!([dx("A")])),
            tweak("d2", json!([dx("B")])),
            tweak("p1", json!([{ "op": "power_value", "subgroup": "sub", "setting": "set", "ac": 0 }])),
            tweak("p2", json!([{ "op": "power_value", "subgroup": "sub", "setting": "set", "dc": 1 }])),
        ]);
        let before = snap(&f);
        let r = apply(&c, &[sel("d1"), sel("d2"), sel("p1"), sel("p2")], &mut f, &t.store, &info())
            .unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!(
            f.reg_get(Hive::Hkcu, DX_PATH, DX_NAME).unwrap(),
            Some(RegData::Sz("A=1;B=1;".into()))
        );
        assert_eq!(f.power[&("".to_string(), "sub".to_string(), "set".to_string())], (0, 1));
        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert!(rr.skipped_newer.is_empty(), "{:?}", rr.skipped_newer);
        assert_eq!(snap(&f), before);
    }

    #[test]
    fn interrupted_session_is_still_revertible() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.seed_reg(Hive::Hklm, "SOFTWARE\\A", "V", RegData::Dword(0));
        let c = cat(vec![tweak(
            "a",
            json!([reg("SOFTWARE\\A", "V", 1), reg("SOFTWARE\\A", "W", 1)]),
        )]);
        let before = snap(&f);
        let r = apply(&c, &[sel("a")], &mut f, &t.store, &info()).unwrap();
        // Simulate a crash after the writes but before the final save.
        let mut s = t.store.load(&r.session_id).unwrap();
        for e in &mut s.entries {
            e.applied = false;
            e.error = None;
        }
        t.store.save(&s).unwrap();
        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!(rr.reverted, 2, "{rr:?}");
        assert_eq!(snap(&f), before);
    }

    #[test]
    fn store_never_overwrites_and_lists_newest_first() {
        let t = tmp();
        let mut f = FakeBackend::new();
        let c = cat(vec![tweak("a", json!([reg("SOFTWARE\\A", "V", 1)]))]);
        let ids: Vec<String> = (0..3)
            .map(|_| apply(&c, &[sel("a")], &mut f, &t.store, &info()).unwrap().session_id)
            .collect();
        let mut uniq = ids.clone();
        uniq.dedup();
        assert_eq!(uniq.len(), 3, "{ids:?}");
        let id_ok = |id: &str| {
            let b = id.as_bytes();
            b.len() >= 15 && b[8] == b'-' && id[..8].chars().all(|c| c.is_ascii_digit())
        };
        assert!(ids.iter().all(|i| id_ok(i)), "{ids:?}");
        let list = t.store.list().unwrap();
        let listed: Vec<String> = list.iter().map(|s| s.id.clone()).collect();
        let mut expected = ids.clone();
        expected.reverse();
        assert_eq!(listed, expected);
        assert!(list.iter().all(|s| s.count == 1 && !s.reverted));
        let s = t.store.load(&ids[0]).unwrap();
        assert_eq!(s.schema, 1);
        assert!(!s.dry_run);
        assert!(s.created.contains('T'), "RFC3339: {}", s.created);
        let leftovers = std::fs::read_dir(&t.store.dir)
            .unwrap()
            .filter(|e| e.as_ref().unwrap().path().extension().is_some_and(|x| x == "tmp"))
            .count();
        assert_eq!(leftovers, 0);
    }

    #[test]
    fn reverting_twice_is_a_noop() {
        let t = tmp();
        let mut f = FakeBackend::new();
        let c = cat(vec![tweak("a", json!([reg("SOFTWARE\\A", "V", 1)]))]);
        let r = apply(&c, &[sel("a")], &mut f, &t.store, &info()).unwrap();
        assert_eq!(revert(&r.session_id, &mut f, &t.store).unwrap().reverted, 1);
        let writes = f.writes.len();
        let again = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!((again.reverted, again.skipped_newer.len(), again.failed.len()), (0, 0, 0));
        assert_eq!(f.writes.len(), writes);
    }

    #[test]
    fn options_select_ops_and_default_option_applies() {
        let t = tmp();
        let mut f = FakeBackend::new();
        let tw = tweak_with(
            "o",
            json!([reg("SOFTWARE\\O", "Top", 1)]),
            json!({
                "options": [
                    { "id": "small", "label": { "tr": "k", "en": "s" }, "ops": [reg("SOFTWARE\\O", "V", 1)] },
                    { "id": "big", "label": { "tr": "b", "en": "b" }, "ops": [reg("SOFTWARE\\O", "V", 2)] }
                ],
                "default_option": "small"
            }),
        );
        let c = cat(vec![tw]);
        apply(&c, &[sel("o")], &mut f, &t.store, &info()).unwrap();
        assert_eq!(get(&f, "SOFTWARE\\O", "Top"), Some(RegData::Dword(1)));
        assert_eq!(get(&f, "SOFTWARE\\O", "V"), Some(RegData::Dword(1)));
        assert_eq!(state(&c.tweaks[0], None, &f), TweakState::Applied);
        assert_eq!(state(&c.tweaks[0], Some("big"), &f), TweakState::Partial);

        let big = Selection { tweak_id: "o".into(), option_id: Some("big".into()) };
        apply(&c, &[big], &mut f, &t.store, &info()).unwrap();
        assert_eq!(get(&f, "SOFTWARE\\O", "V"), Some(RegData::Dword(2)));

        let bad = Selection { tweak_id: "o".into(), option_id: Some("nope".into()) };
        let r = apply(&c, &[bad, sel("missing")], &mut f, &t.store, &info()).unwrap();
        assert_eq!(r.failed.len(), 2, "{:?}", r.failed);
        assert_eq!(r.applied, 0);
    }

    #[test]
    fn device_and_nic_ops_expand_per_instance() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.devices = vec![
            (DeviceClass::Usb, "USB\\VID_1\\A".into()),
            (DeviceClass::Usb, "USB\\VID_2\\B".into()),
            (DeviceClass::Net, "PCI\\NIC".into()),
        ];
        let nic = "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e972-e325-11ce-bfc1-08002be10318}\\0001";
        f.nic_keys = vec![nic.into()];
        f.seed_reg(Hive::Hklm, nic, "*EEE", RegData::Sz("1".into()));
        f.seed_reg(Hive::Hklm, &format!("{nic}\\Ndi\\Params\\*EEE"), "ParamDesc", RegData::Sz("EEE".into()));
        let c = cat(vec![tweak(
            "dev",
            json!([
                { "op": "device_reg", "class": "usb", "suffix": "Device Parameters",
                  "name": "EnhancedPowerManagementEnabled", "data": { "type": "dword", "value": 0 } },
                { "op": "nic_property", "keyword": "*EEE", "value": "0" }
            ]),
        )]);
        let before = snap(&f);
        let r = apply(&c, &[sel("dev")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        let s = t.store.load(&r.session_id).unwrap();
        assert_eq!(s.entries.len(), 3);
        assert!(s.entries.iter().all(|e| matches!(e.op, Op::RegSet { .. })));
        assert_eq!(
            get(&f, "SYSTEM\\CurrentControlSet\\Enum\\USB\\VID_2\\B\\Device Parameters", "EnhancedPowerManagementEnabled"),
            Some(RegData::Dword(0))
        );
        assert_eq!(get(&f, nic, "*EEE"), Some(RegData::Sz("0".into())));
        assert_eq!(state(&c.tweaks[0], None, &f), TweakState::Applied);
        revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!(snap(&f), before);
    }

    const NIC_A: &str =
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e972-e325-11ce-bfc1-08002be10318}\\0001";
    const NIC_B: &str =
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e972-e325-11ce-bfc1-08002be10318}\\0002";

    fn eee_catalog() -> Catalog {
        cat(vec![tweak("eee", json!([{ "op": "nic_property", "keyword": "*EEE", "value": "0" }]))])
    }

    #[test]
    fn nic_property_only_touches_adapters_that_declare_the_keyword() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.nic_keys = vec![NIC_A.into(), NIC_B.into()];
        f.seed_reg(Hive::Hklm, &format!("{NIC_A}\\Ndi\\Params\\*EEE"), "ParamDesc", RegData::Sz("EEE".into()));
        f.seed_reg(Hive::Hklm, NIC_B, "DriverDesc", RegData::Sz("Other".into()));
        let c = eee_catalog();
        let before = snap(&f);
        let r = apply(&c, &[sel("eee")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!(get(&f, NIC_A, "*EEE"), Some(RegData::Sz("0".into())));
        assert_eq!(get(&f, NIC_B, "*EEE"), None);
        assert_eq!(f.writes.len(), 1, "{:?}", f.writes);
        revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!(get(&f, NIC_A, "*EEE"), None);
        assert_eq!(snap(&f), before);
    }

    #[test]
    fn nic_property_without_qualifying_adapter_writes_nothing() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.nic_keys = vec![NIC_A.into(), NIC_B.into()];
        f.seed_reg(Hive::Hklm, NIC_A, "DriverDesc", RegData::Sz("A".into()));
        f.seed_reg(Hive::Hklm, NIC_B, "DriverDesc", RegData::Sz("B".into()));
        let c = eee_catalog();
        let r = apply(&c, &[sel("eee")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert!(f.writes.is_empty(), "{:?}", f.writes);
        assert_eq!(get(&f, NIC_A, "*EEE"), None);
        assert_eq!(get(&f, NIC_B, "*EEE"), None);
    }

    #[test]
    fn dry_run_sessions_are_isolated_from_real_mode() {
        let t = tmp();
        let dry_store = BackupStore::new_dry_run(t.store.dir.clone());
        let mut seeded = FakeBackend::new();
        seeded.seed_reg(Hive::Hklm, "SOFTWARE\\A", "V", RegData::Dword(0));
        let mut d = DryRunBackend::new(seeded, None);
        let c = cat(vec![tweak("a", json!([reg("SOFTWARE\\A", "V", 1)]))]);
        let r = apply(&c, &[sel("a")], &mut d, &dry_store, &info()).unwrap();
        assert_eq!(r.applied, 1);
        assert!(d.inner().writes.is_empty());
        assert!(t.store.load(&r.session_id).unwrap().dry_run);

        // A real-mode store refuses to touch the dry-run session.
        let mut real = FakeBackend::new();
        assert!(revert(&r.session_id, &mut real, &t.store).is_err());
        assert_eq!(revert_all(&mut real, &t.store).unwrap().reverted, 0);
        assert!(real.writes.is_empty());

        let rr = revert(&r.session_id, &mut d, &dry_store).unwrap();
        assert!(rr.failed.is_empty() && rr.skipped_newer.is_empty(), "{rr:?}");
        assert!(d.inner().writes.is_empty());
        assert!(t.store.load(&r.session_id).unwrap().reverted);
    }

    #[test]
    fn backend_errors_are_reported_not_fatal() {
        let t = tmp();
        let mut f = FakeBackend::new();
        let c = cat(vec![
            tweak("p", json!([{ "op": "power_value", "subgroup": "x", "setting": "missing", "ac": 1 }])),
            tweak("svc", json!([{ "op": "service_start", "name": "NoSuchService", "start": 4 }])),
            tweak("ok", json!([reg("SOFTWARE\\OK", "V", 1)])),
        ]);
        let r = apply(&c, &[sel("p"), sel("svc"), sel("ok")], &mut f, &t.store, &info()).unwrap();
        let failed: Vec<&str> = r.failed.iter().map(|(id, _)| id.as_str()).collect();
        // A missing service is skipped (like a NIC keyword no adapter has), not a failure.
        assert_eq!(failed, ["p"]);
        assert_eq!(r.applied, 2);
        assert!(!f.services.contains_key("NoSuchService"), "missing service is not created");
        assert_eq!(get(&f, "SOFTWARE\\OK", "V"), Some(RegData::Dword(1)));
    }

    #[test]
    fn one_way_ops_are_counted_and_never_reverted() {
        let t = tmp();
        let mut f = FakeBackend::new();
        let c = cat(vec![tweak_with(
            "clean",
            json!([
                { "op": "appx_remove", "pattern": "Microsoft.Bing*" },
                { "op": "cleanup", "target": "user_temp" }
            ]),
            json!({ "one_way": true }),
        )]);
        let r = apply(&c, &[sel("clean")], &mut f, &t.store, &info()).unwrap();
        assert_eq!((r.applied, r.one_way, r.bytes_freed), (1, 2, 0));
        let writes = f.writes.len();
        let rr = revert(&r.session_id, &mut f, &t.store).unwrap();
        assert_eq!(rr.reverted, 0);
        assert_eq!(f.writes.len(), writes);
        assert!(t.store.load(&r.session_id).unwrap().reverted);
        let e = &t.store.load(&r.session_id).unwrap().entries[0];
        assert!(matches!(e.prior, PriorState::OneWay));
    }

    #[test]
    fn revert_category_only_touches_that_category() {
        let t = tmp();
        let mut f = FakeBackend::new();
        for k in ["N1", "N2", "P"] {
            f.seed_reg(Hive::Hklm, &format!("SOFTWARE\\{k}"), "V", RegData::Dword(0));
        }
        let c = cat(vec![
            tweak_with("n1", json!([reg("SOFTWARE\\N1", "V", 1)]), json!({ "category": "network" })),
            tweak_with("n2", json!([reg("SOFTWARE\\N2", "V", 1)]), json!({ "category": "network" })),
            tweak_with("p", json!([reg("SOFTWARE\\P", "V", 1)]), json!({ "category": "power" })),
        ]);
        // session 1 mixes categories, session 2 is network only
        let s1 = apply(&c, &[sel("n1"), sel("p")], &mut f, &t.store, &info()).unwrap();
        let s2 = apply(&c, &[sel("n2")], &mut f, &t.store, &info()).unwrap();

        let rr = revert_category("network", &c, &mut f, &t.store).unwrap();
        assert!(rr.failed.is_empty(), "{:?}", rr.failed);
        assert_eq!(rr.reverted, 2);
        assert_eq!(get(&f, "SOFTWARE\\N1", "V"), Some(RegData::Dword(0)));
        assert_eq!(get(&f, "SOFTWARE\\N2", "V"), Some(RegData::Dword(0)));
        assert_eq!(get(&f, "SOFTWARE\\P", "V"), Some(RegData::Dword(1)), "power untouched");
        assert!(!t.store.load(&s1.session_id).unwrap().reverted, "power entry still pending");
        assert!(t.store.load(&s2.session_id).unwrap().reverted);
        assert_eq!(state(&c.tweaks[2], None, &f), TweakState::Applied);

        // nothing left in that category: a second call is a no-op
        let again = revert_category("network", &c, &mut f, &t.store).unwrap();
        assert_eq!(again, RevertReport::default());

        let rp = revert(&s1.session_id, &mut f, &t.store).unwrap();
        assert_eq!(rp.reverted, 1);
        assert_eq!(get(&f, "SOFTWARE\\P", "V"), Some(RegData::Dword(0)));
        assert!(t.store.load(&s1.session_id).unwrap().reverted);
    }
}

mod store {
    use alacran_tweak_lib::store::{format_stamp, BackupStore};

    #[test]
    fn stamp_formats_utc() {
        let (id, created) = format_stamp(1_794_985_509);
        assert_eq!(id, "20261118-070509");
        assert_eq!(created, "2026-11-18T07:05:09Z");
        assert_eq!(format_stamp(951_782_400).1, "2000-02-29T00:00:00Z");
    }

    #[test]
    fn load_rejects_path_like_ids() {
        let s = BackupStore::new(std::env::temp_dir().join("alacran-store-ids"));
        assert!(s.load("..\\..\\x").is_err());
        assert!(s.list().unwrap().is_empty(), "missing dir lists as empty");
    }
}

mod fix_round_1 {
    use std::sync::atomic::{AtomicU32, Ordering};

    use alacran_tweak_lib::backend::{FakeBackend, SystemBackend};
    use alacran_tweak_lib::catalog::*;
    use alacran_tweak_lib::engine::*;
    use alacran_tweak_lib::store::BackupStore;
    use alacran_tweak_lib::sysinfo::SystemInfo;
    use serde_json::json;

    static N: AtomicU32 = AtomicU32::new(0);

    struct Tmp {
        store: BackupStore,
    }

    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.store.dir);
        }
    }

    fn tmp() -> Tmp {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("alacran-fix1-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Tmp { store: BackupStore::new(dir) }
    }

    fn info() -> SystemInfo {
        SystemInfo { build: 22631, ..Default::default() }
    }

    fn sel(id: &str) -> Selection {
        Selection { tweak_id: id.into(), option_id: None }
    }

    fn mixed_catalog() -> Catalog {
        let t: Tweak = serde_json::from_value(json!({
            "id": "mix", "category": "performance",
            "title": { "tr": "t", "en": "t" }, "description": { "tr": "d", "en": "d" },
            "risk": "safe", "in_one_click": false, "new": false, "reboot": "none",
            "one_way": false, "min_build": 19044,
            "ops": [
                { "op": "reg_set", "hive": "hklm", "path": "SOFTWARE\\M", "name": "V",
                  "data": { "type": "dword", "value": 1 } },
                { "op": "reg_set", "hive": "hkcu", "path": "Software\\U", "name": "V",
                  "data": { "type": "dword", "value": 1 } }
            ]
        }))
        .unwrap();
        Catalog { tweaks: vec![t] }
    }

    const A: &str = "S-1-5-21-A";
    const B: &str = "S-1-5-21-B";

    fn setup() -> (Tmp, FakeBackend, String) {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.sid = A.into();
        f.seed_reg(Hive::Hklm, "SOFTWARE\\M", "V", RegData::Dword(0));
        let r = apply(&mixed_catalog(), &[sel("mix")], &mut f, &t.store, &info()).unwrap();
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        assert_eq!(t.store.load(&r.session_id).unwrap().interactive_sid, A);
        // Another user logs on; they have their own value at the same HKCU path.
        f.sid = B.into();
        f.seed_reg(Hive::Hkcu, "Software\\U", "V", RegData::Dword(7));
        (t, f, r.session_id)
    }

    fn assert_other_user_untouched(f: &FakeBackend, rr: &RevertReport) {
        assert_eq!(rr.failed.len(), 1, "{:?}", rr.failed);
        assert_eq!(rr.failed[0].0, "reg:HKCU\\Software\\U\\V");
        assert!(rr.failed[0].1.contains(&format!("kayıt farklı bir kullanıcıya ait: {A}")));
        assert_eq!(f.reg_get(Hive::Hklm, "SOFTWARE\\M", "V").unwrap(), Some(RegData::Dword(0)));
        assert_eq!(f.reg_get(Hive::Hkcu, "Software\\U", "V").unwrap(), Some(RegData::Dword(7)));
    }

    fn back_to_owner_finishes(t: &Tmp, mut f: FakeBackend, id: &str) {
        assert!(!t.store.load(id).unwrap().reverted, "HKCU entry stays pending");
        f.sid = A.into();
        let rr = revert(id, &mut f, &t.store).unwrap();
        assert_eq!((rr.reverted, rr.failed.len()), (1, 0), "{rr:?}");
        assert_eq!(f.reg_get(Hive::Hkcu, "Software\\U", "V").unwrap(), None);
        assert!(!f.reg_key_exists(Hive::Hkcu, "Software\\U").unwrap());
        assert!(t.store.load(id).unwrap().reverted);
        f.sid = B.into();
        assert_eq!(f.reg_get(Hive::Hkcu, "Software\\U", "V").unwrap(), Some(RegData::Dword(7)));
    }

    #[test]
    fn revert_skips_hkcu_entries_of_another_user() {
        let (t, mut f, id) = setup();
        let rr = revert(&id, &mut f, &t.store).unwrap();
        assert_eq!(rr.reverted, 1, "HKLM entry still reverts");
        assert_other_user_untouched(&f, &rr);
        back_to_owner_finishes(&t, f, &id);
    }

    #[test]
    fn revert_all_skips_hkcu_entries_of_another_user() {
        let (t, mut f, id) = setup();
        let rr = revert_all(&mut f, &t.store).unwrap();
        assert_eq!(rr.reverted, 1, "HKLM entry still reverts");
        assert_other_user_untouched(&f, &rr);
        back_to_owner_finishes(&t, f, &id);
    }

    #[test]
    fn broken_session_files_are_reported() {
        let t = tmp();
        let mut f = FakeBackend::new();
        f.seed_reg(Hive::Hklm, "SOFTWARE\\M", "V", RegData::Dword(0));
        f.seed_reg(Hive::Hkcu, "Software\\U", "V", RegData::Dword(0));
        let r = apply(&mixed_catalog(), &[sel("mix")], &mut f, &t.store, &info()).unwrap();
        std::fs::write(t.store.dir.join("20200101-000000.json"), br#"{"schema":1,"id":"2020"#)
            .unwrap();

        let list = t.store.list().unwrap();
        assert_eq!(list.len(), 1, "good session still listed");
        assert_eq!(list[0].id, r.session_id);
        let broken = t.store.list_broken().unwrap();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].0, "20200101-000000.json");

        let rr = revert_all(&mut f, &t.store).unwrap();
        assert_eq!(rr.failed.len(), 1, "{:?}", rr.failed);
        assert_eq!(rr.failed[0].0, "session:20200101-000000.json");
        assert_eq!(rr.reverted, 2, "good session reverted");
        assert_eq!(f.reg_get(Hive::Hklm, "SOFTWARE\\M", "V").unwrap(), Some(RegData::Dword(0)));
    }
}
