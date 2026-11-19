//! Present devices (SetupAPI) and network adapter class keys.

use ::windows::core::{GUID, PCWSTR};
use ::windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
    SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceRegistryPropertyW, DIGCF_PRESENT,
    GUID_DEVCLASS_DISPLAY, GUID_DEVCLASS_NET, GUID_DEVCLASS_USB, HDEVINFO, SPDRP_HARDWAREID,
    SP_DEVINFO_DATA,
};
use anyhow::Context;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

use crate::backend::Result;
use crate::catalog::{DeviceClass, Vendor};

/// Network adapter setup class, below `HKLM`.
pub const NIC_CLASS_PATH: &str =
    r"SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002BE10318}";

fn class_guid(class: DeviceClass) -> GUID {
    match class {
        DeviceClass::Usb => GUID_DEVCLASS_USB,
        DeviceClass::Net => GUID_DEVCLASS_NET,
        DeviceClass::Display => GUID_DEVCLASS_DISPLAY,
    }
}

struct DevInfo(HDEVINFO);

impl Drop for DevInfo {
    fn drop(&mut self) {
        let _ = unsafe { SetupDiDestroyDeviceInfoList(self.0) };
    }
}

/// Calls `f` for every present device of `class`.
fn for_each_present(
    class: DeviceClass,
    mut f: impl FnMut(&DevInfo, &SP_DEVINFO_DATA),
) -> Result<()> {
    let guid = class_guid(class);
    let set = unsafe { SetupDiGetClassDevsW(Some(&guid), PCWSTR::null(), None, DIGCF_PRESENT) }
        .with_context(|| format!("cannot list {class:?} devices"))?;
    let set = DevInfo(set);
    let mut i = 0u32;
    loop {
        let mut data = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        // Fails with ERROR_NO_MORE_ITEMS after the last device.
        if unsafe { SetupDiEnumDeviceInfo(set.0, i, &mut data) }.is_err() {
            break;
        }
        f(&set, &data);
        i += 1;
    }
    Ok(())
}

fn instance_id(set: &DevInfo, data: &SP_DEVINFO_DATA) -> Option<String> {
    let mut needed = 0u32;
    let _ = unsafe { SetupDiGetDeviceInstanceIdW(set.0, data, None, Some(&mut needed)) };
    if needed == 0 {
        return None;
    }
    let mut buf = vec![0u16; needed as usize];
    unsafe { SetupDiGetDeviceInstanceIdW(set.0, data, Some(&mut buf), None) }.ok()?;
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..end]))
}

fn hardware_ids(set: &DevInfo, data: &SP_DEVINFO_DATA) -> Vec<String> {
    let mut needed = 0u32;
    let _ = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            set.0,
            data,
            SPDRP_HARDWAREID,
            None,
            None,
            Some(&mut needed),
        )
    };
    if needed == 0 {
        return Vec::new();
    }
    let mut buf = vec![0u8; needed as usize];
    if unsafe {
        SetupDiGetDeviceRegistryPropertyW(set.0, data, SPDRP_HARDWAREID, None, Some(&mut buf), None)
    }
    .is_err()
    {
        return Vec::new();
    }
    let units: Vec<u16> = buf
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect();
    units
        .split(|&u| u == 0)
        .filter(|s| !s.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}

/// Instance ids (e.g. `PCI\VEN_10DE&DEV_2684&…\4&…`) of present devices.
pub fn present(class: DeviceClass) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for_each_present(class, |set, data| {
        if let Some(id) = instance_id(set, data) {
            ids.push(id);
        }
    })?;
    Ok(ids)
}

/// Hardware ids of all present display adapters.
pub fn display_hardware_ids() -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for_each_present(DeviceClass::Display, |set, data| {
        ids.extend(hardware_ids(set, data))
    })?;
    Ok(ids)
}

/// GPU vendor from a PCI hardware/instance id.
pub fn vendor_from_id(id: &str) -> Option<Vendor> {
    let up = id.to_ascii_uppercase();
    if up.contains("VEN_10DE") {
        Some(Vendor::Nvidia)
    } else if up.contains("VEN_1002") {
        Some(Vendor::Amd)
    } else if up.contains("VEN_8086") {
        Some(Vendor::Intel)
    } else {
        None
    }
}

/// Distinct vendors in first-seen order.
pub fn vendors_from_ids<S: AsRef<str>>(ids: &[S]) -> Vec<Vendor> {
    let mut out = Vec::new();
    for v in ids.iter().filter_map(|id| vendor_from_id(id.as_ref())) {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

/// `HKLM`-relative paths of the adapter class subkeys that belong to a
/// network adapter (they carry `NetCfgInstanceId`). Subkeys that cannot be
/// opened (e.g. `Properties`) are skipped.
pub fn nic_class_keys() -> Result<Vec<String>> {
    let class = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(NIC_CLASS_PATH, KEY_READ)
        .context("cannot open the network adapter class key")?;
    let mut out = Vec::new();
    for name in class.enum_keys().filter_map(|k| k.ok()) {
        let Ok(sub) = class.open_subkey_with_flags(&name, KEY_READ) else {
            continue;
        };
        if sub.get_raw_value("NetCfgInstanceId").is_ok() {
            out.push(format!("{NIC_CLASS_PATH}\\{name}"));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_ids_map_to_vendors() {
        assert_eq!(
            vendor_from_id(r"PCI\VEN_10DE&DEV_2684&SUBSYS_16F110DE"),
            Some(Vendor::Nvidia)
        );
        assert_eq!(vendor_from_id(r"pci\ven_1002&dev_744c"), Some(Vendor::Amd));
        assert_eq!(
            vendor_from_id(r"PCI\VEN_8086&DEV_A780&REV_04"),
            Some(Vendor::Intel)
        );
        assert_eq!(vendor_from_id(r"ROOT\BasicDisplay"), None);
        assert_eq!(vendor_from_id(r"PCI\VEN_15AD&DEV_0405"), None);
    }

    #[test]
    fn vendors_are_distinct_in_order() {
        let ids = [
            r"PCI\VEN_8086&DEV_A780&SUBSYS_1",
            r"PCI\VEN_8086&DEV_A780",
            r"PCI\VEN_10DE&DEV_2684",
            r"ROOT\BasicDisplay",
        ];
        assert_eq!(vendors_from_ids(&ids), vec![Vendor::Intel, Vendor::Nvidia]);
    }
}
