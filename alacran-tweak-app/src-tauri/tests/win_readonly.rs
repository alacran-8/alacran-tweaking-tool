//! Read-only checks of the real Windows backend. They never call a setter.
//! Run manually: `cargo test --test win_readonly -- --ignored`
//! (`bcd_get_reads_current_entry` needs an elevated shell, like bcdedit).
#![cfg(windows)]

use alacran_tweak_lib::backend::{SystemBackend, WinBackend};
use alacran_tweak_lib::catalog::{DeviceClass, Hive, RegData};

#[test]
#[ignore]
fn reads_known_value() {
    let be = WinBackend::new();
    let v = be
        .reg_get(
            Hive::Hklm,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "CurrentBuildNumber",
        )
        .unwrap();
    assert!(
        matches!(v, Some(RegData::Sz(ref s)) if s.parse::<u32>().is_ok()),
        "{v:?}"
    );
}

#[test]
#[ignore]
fn missing_value_and_key_read_as_absent() {
    let be = WinBackend::new();
    let path = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion";
    assert_eq!(
        be.reg_get(Hive::Hklm, path, "AlacranDoesNotExist").unwrap(),
        None
    );
    assert!(be.reg_key_exists(Hive::Hklm, path).unwrap());
    assert!(!be
        .reg_key_exists(Hive::Hklm, r"SOFTWARE\AlacranDoesNotExist")
        .unwrap());
}

#[test]
#[ignore]
fn interactive_sid_is_stable_and_hkcu_readable() {
    let be = WinBackend::new();
    let sid = be.interactive_sid();
    assert!(sid.starts_with("S-1-"), "{sid:?}");
    assert_eq!(sid, WinBackend::new().interactive_sid());
    assert!(be
        .reg_key_exists(Hive::Hkcu, r"Software\Microsoft")
        .unwrap());
}

#[test]
#[ignore]
fn sysinfo_detects_build_and_ram() {
    let info = alacran_tweak_lib::sysinfo::detect();
    assert!(info.build >= 19044, "{info:?}");
    assert!(info.ram_gb > 0, "{info:?}");
    assert!(!info.edition.is_empty(), "{info:?}");
}

#[test]
#[ignore]
fn lists_display_devices() {
    let be = WinBackend::new();
    let ids = be.devices(DeviceClass::Display).unwrap();
    assert!(!ids.is_empty());
}

#[test]
#[ignore]
fn lists_nic_class_keys() {
    let be = WinBackend::new();
    for k in be.nic_class_keys().unwrap() {
        assert!(
            k.starts_with(r"SYSTEM\CurrentControlSet\Control\Class\{4D36E972"),
            "{k}"
        );
    }
}

#[test]
#[ignore]
fn reads_service_power_and_task_state() {
    let be = WinBackend::new();
    assert!(be.service_start_get("EventLog").unwrap().is_some());
    assert_eq!(be.service_start_get("AlacranDoesNotExist").unwrap(), None);
    let scheme = be.active_scheme().unwrap();
    assert_eq!(scheme.len(), 36, "{scheme}");
    // Processor power management / minimum processor state.
    be.power_get(
        &scheme,
        "54533251-82be-4824-96c1-47b60b740d00",
        "893dee8e-2bef-41e0-89c6-b55d0929964c",
    )
    .unwrap();
    assert_eq!(
        be.task_enabled_get(r"\Microsoft\AlacranDoesNotExist\Nope")
            .unwrap(),
        None
    );
}

#[test]
#[ignore]
fn system_protection_state_reads() {
    alacran_tweak_lib::restore_point::system_protection_enabled("C:").unwrap();
}

#[test]
#[ignore]
fn bcd_get_reads_current_entry() {
    let be = WinBackend::new();
    let d = be.bcd_get("description").unwrap();
    assert!(d.is_some_and(|s| !s.is_empty()));
}

#[test]
#[ignore]
fn process_account_matches_interactive_user_when_not_ots_elevated() {
    use alacran_tweak_lib::backend::windows::session;
    let be = WinBackend::new();
    let process = session::process_user_sid().expect("process token SID");
    assert!(process.starts_with("S-1-"), "{process}");
    // Test shells run as the logged-on user, so the guard must pass.
    session::require_process_is_user(&be.interactive_sid()).unwrap();
}
