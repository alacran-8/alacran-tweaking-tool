//! Facts about the running system that decide which tweaks apply.

use serde::{Deserialize, Serialize};

use crate::catalog::Vendor;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    pub build: u32,
    /// e.g. "24H2".
    pub display_version: String,
    pub edition: String,
    pub gpu_vendors: Vec<Vendor>,
    pub ram_gb: u32,
    pub is_laptop: bool,
    pub interactive_sid: String,
}

/// Installed RAM in whole GiB, rounded to nearest (15.9 GiB usable → 16).
pub fn ram_gb_from_bytes(bytes: u64) -> u32 {
    const GIB: u64 = 1 << 30;
    ((bytes + GIB / 2) / GIB) as u32
}

/// `BatteryFlag` 128 = "no system battery", 255 = "unknown status".
pub fn is_laptop_battery_flag(flag: u8) -> bool {
    flag != 128 && flag != 255
}

/// Reads the facts from the running system. Every field degrades to its
/// default when its source cannot be read.
#[cfg(windows)]
pub fn detect() -> SystemInfo {
    use ::windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    use ::windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    use crate::backend::windows::{devices, session};

    let mut info = SystemInfo::default();
    if let Ok(cv) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
    {
        let s = |name: &str| cv.get_value::<String, _>(name).ok();
        info.build = s("CurrentBuildNumber")
            .and_then(|b| b.trim().parse().ok())
            .unwrap_or(0);
        info.display_version = s("DisplayVersion")
            .or_else(|| s("ReleaseId"))
            .unwrap_or_default();
        info.edition = s("EditionID").unwrap_or_default();
    }
    if let Ok(ids) = devices::display_hardware_ids() {
        info.gpu_vendors = devices::vendors_from_ids(&ids);
    }
    let mut mem = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut mem) }.is_ok() {
        info.ram_gb = ram_gb_from_bytes(mem.ullTotalPhys);
    }
    let mut power = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&mut power) }.is_ok() {
        info.is_laptop = is_laptop_battery_flag(power.BatteryFlag);
    }
    info.interactive_sid = session::interactive_sid().unwrap_or_default();
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ram_rounds_to_nearest_gib() {
        assert_eq!(ram_gb_from_bytes(0), 0);
        assert_eq!(ram_gb_from_bytes(17_093_496_832), 16); // 15.92 GiB usable
        assert_eq!(ram_gb_from_bytes(8 << 30), 8);
        assert_eq!(ram_gb_from_bytes((3 << 30) + (1 << 29)), 4);
    }

    #[test]
    fn battery_flag_decides_laptop() {
        assert!(!is_laptop_battery_flag(128));
        assert!(!is_laptop_battery_flag(255));
        assert!(is_laptop_battery_flag(1));
        assert!(is_laptop_battery_flag(8));
        assert!(is_laptop_battery_flag(0));
    }
}
