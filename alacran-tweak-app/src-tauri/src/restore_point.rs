//! System Restore: create a restore point, check and enable protection.
//!
//! `create_restore_point` does not touch `SystemRestorePointCreationFrequency`
//! itself: the caller sets it to 0 through the engine as a normal `RegSet`
//! (HKLM `SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore`) so the
//! change is backed up and revertible, then calls this.

use anyhow::Result;

/// `\\?\Volume{…}\:(C%3A)`-style SPP client entries name the protected
/// volumes. True when one of `entries` mentions `volume_name`'s GUID.
pub fn spp_lists_volume<S: AsRef<str>>(entries: &[S], volume_name: &str) -> bool {
    let Some(guid) = volume_guid(volume_name) else {
        return false;
    };
    entries
        .iter()
        .any(|e| e.as_ref().to_ascii_lowercase().contains(&guid))
}

/// Lower-cased `{guid}` from `\\?\Volume{guid}\`.
fn volume_guid(volume_name: &str) -> Option<String> {
    let lower = volume_name.to_ascii_lowercase();
    let start = lower.find('{')?;
    let end = lower[start..].find('}')? + start;
    Some(lower[start..=end].to_string())
}

/// `C:`, `c`, `C:\` → `C:\`.
pub fn normalize_drive(drive: &str) -> Result<String> {
    let letter = drive
        .trim()
        .chars()
        .next()
        .filter(|c| c.is_ascii_alphabetic())
        .ok_or_else(|| anyhow::anyhow!("invalid drive: {drive:?}"))?;
    let rest = drive.trim()[1..].trim_end_matches('\\');
    if !(rest.is_empty() || rest == ":") {
        anyhow::bail!("invalid drive: {drive:?}");
    }
    Ok(format!("{}:\\", letter.to_ascii_uppercase()))
}

#[cfg(windows)]
pub use imp::{create_restore_point, enable_system_protection, system_protection_enabled};

#[cfg(windows)]
mod imp {
    use std::mem::ManuallyDrop;

    use ::windows::core::{BSTR, PCWSTR};
    use ::windows::Win32::Storage::FileSystem::GetVolumeNameForVolumeMountPointW;
    use ::windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeSecurity, CoSetProxyBlanket, CLSCTX_INPROC_SERVER, EOAC_NONE,
        RPC_C_AUTHN_LEVEL_CALL, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE,
    };
    use ::windows::Win32::System::Rpc::{RPC_C_AUTHN_WINNT, RPC_C_AUTHZ_NONE};
    use ::windows::Win32::System::Variant::{
        VariantClear, VariantToInt32, VARIANT, VT_BSTR, VT_I4,
    };
    use ::windows::Win32::System::Wmi::{
        IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_RETURN_WBEM_COMPLETE,
    };
    use anyhow::{bail, Context, Result};
    use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    use super::{normalize_drive, spp_lists_volume};
    use crate::backend::windows::{registry, wide, ComGuard};
    use crate::catalog::RegData;

    /// Restore point type `MODIFY_SETTINGS`.
    const MODIFY_SETTINGS: i32 = 12;
    /// Event type `BEGIN_SYSTEM_CHANGE`.
    const BEGIN_SYSTEM_CHANGE: i32 = 100;

    struct Var(VARIANT);

    impl Var {
        fn bstr(s: &str) -> Self {
            let mut v = VARIANT::default();
            unsafe {
                let inner = &mut *v.Anonymous.Anonymous;
                inner.vt = VT_BSTR;
                inner.Anonymous.bstrVal = ManuallyDrop::new(BSTR::from(s));
            }
            Var(v)
        }

        fn i4(n: i32) -> Self {
            let mut v = VARIANT::default();
            unsafe {
                let inner = &mut *v.Anonymous.Anonymous;
                inner.vt = VT_I4;
                inner.Anonymous.lVal = n;
            }
            Var(v)
        }
    }

    impl Drop for Var {
        fn drop(&mut self) {
            let _ = unsafe { VariantClear(&mut self.0) };
        }
    }

    fn connect() -> Result<IWbemServices> {
        unsafe {
            // Process-wide and allowed once; "too late" is fine because
            // the proxy blanket below sets what WMI needs.
            let _ = CoInitializeSecurity(
                None,
                -1,
                None,
                None,
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            );
            let loc: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
                .context("cannot create the WMI locator")?;
            let svc = loc
                .ConnectServer(
                    &BSTR::from(r"ROOT\DEFAULT"),
                    &BSTR::new(),
                    &BSTR::new(),
                    &BSTR::new(),
                    0,
                    &BSTR::new(),
                    None,
                )
                .context(r"cannot connect to WMI ROOT\DEFAULT")?;
            CoSetProxyBlanket(
                &svc,
                RPC_C_AUTHN_WINNT,
                RPC_C_AUTHZ_NONE,
                PCWSTR::null(),
                RPC_C_AUTHN_LEVEL_CALL,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
            )
            .context("cannot set the WMI proxy security")?;
            Ok(svc)
        }
    }

    /// Calls static `SystemRestore.<method>` with `args` and returns its
    /// `ReturnValue` (0 = success).
    fn call_system_restore(method: &str, args: &[(&str, Var)]) -> Result<i32> {
        let _com = ComGuard::new()?;
        let svc = connect()?;
        unsafe {
            let mut class: Option<IWbemClassObject> = None;
            svc.GetObject(
                &BSTR::from("SystemRestore"),
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                Some(&mut class),
                None,
            )
            .context("cannot open WMI class SystemRestore")?;
            let class = class.context("WMI class SystemRestore is missing")?;
            let m = wide(method);
            let mut in_sig: Option<IWbemClassObject> = None;
            class
                .GetMethod(PCWSTR(m.as_ptr()), 0, &mut in_sig, std::ptr::null_mut())
                .with_context(|| format!("SystemRestore.{method} is missing"))?;
            let input = match in_sig {
                Some(sig) => Some(sig.SpawnInstance(0)?),
                None => None,
            };
            if let Some(input) = &input {
                for (name, value) in args {
                    let n = wide(name);
                    input
                        .Put(PCWSTR(n.as_ptr()), 0, &value.0, 0)
                        .with_context(|| format!("cannot set {method} parameter {name}"))?;
                }
            }
            let mut out: Option<IWbemClassObject> = None;
            svc.ExecMethod(
                &BSTR::from("SystemRestore"),
                &BSTR::from(method),
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                input.as_ref(),
                Some(&mut out),
                None,
            )
            .with_context(|| format!("SystemRestore.{method} failed"))?;
            let out = out.with_context(|| format!("SystemRestore.{method} returned nothing"))?;
            let rv = wide("ReturnValue");
            let mut v = Var(VARIANT::default());
            out.Get(PCWSTR(rv.as_ptr()), 0, &mut v.0, None, None)
                .with_context(|| format!("SystemRestore.{method} has no ReturnValue"))?;
            Ok(VariantToInt32(&v.0).unwrap_or(-1))
        }
    }

    /// Creates a `MODIFY_SETTINGS` restore point named `desc`.
    pub fn create_restore_point(desc: &str) -> Result<()> {
        let rv = call_system_restore(
            "CreateRestorePoint",
            &[
                ("Description", Var::bstr(desc)),
                ("RestorePointType", Var::i4(MODIFY_SETTINGS)),
                ("EventType", Var::i4(BEGIN_SYSTEM_CHANGE)),
            ],
        )?;
        if rv != 0 {
            bail!("SystemRestore.CreateRestorePoint returned {rv}");
        }
        Ok(())
    }

    /// Turns on System Protection for `drive` (e.g. `C:`). Only after the
    /// user confirmed it in the UI.
    pub fn enable_system_protection(drive: &str) -> Result<()> {
        let root = normalize_drive(drive)?;
        let rv = call_system_restore("Enable", &[("Drive", Var::bstr(&root))])?;
        if rv != 0 {
            bail!("SystemRestore.Enable({root}) returned {rv}");
        }
        Ok(())
    }

    /// True when System Protection is on for `drive` (its volume is listed
    /// under `SPP\Clients`). Read-only.
    pub fn system_protection_enabled(drive: &str) -> Result<bool> {
        let root = normalize_drive(drive)?;
        let mount = wide(&root);
        let mut buf = [0u16; 64];
        unsafe { GetVolumeNameForVolumeMountPointW(PCWSTR(mount.as_ptr()), &mut buf) }
            .with_context(|| format!("cannot resolve the volume of {root}"))?;
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let volume = String::from_utf16_lossy(&buf[..end]);

        let Ok(clients) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SPP\Clients",
            KEY_READ,
        ) else {
            return Ok(false);
        };
        let mut entries: Vec<String> = Vec::new();
        for (_, value) in clients.enum_values().filter_map(|v| v.ok()) {
            match registry::decode(&value.vtype, &value.bytes) {
                Ok(RegData::MultiSz(items)) => entries.extend(items),
                Ok(RegData::Sz(s)) | Ok(RegData::ExpandSz(s)) => entries.push(s),
                _ => {}
            }
        }
        Ok(spp_lists_volume(&entries, &volume))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_volume_in_spp_entries() {
        let entries = vec![r"\\?\Volume{8f1c2d3e-0000-4000-8000-123456789abc}\:(C%3A)".to_string()];
        assert!(spp_lists_volume(
            &entries,
            r"\\?\Volume{8F1C2D3E-0000-4000-8000-123456789ABC}\"
        ));
        assert!(!spp_lists_volume(
            &entries,
            r"\\?\Volume{00000000-0000-4000-8000-123456789abc}\"
        ));
        assert!(!spp_lists_volume(&entries, "garbage"));
        assert!(!spp_lists_volume::<String>(
            &[],
            r"\\?\Volume{8f1c2d3e-0000-4000-8000-123456789abc}\"
        ));
    }

    #[test]
    fn normalizes_drives() {
        assert_eq!(normalize_drive("C:").unwrap(), r"C:\");
        assert_eq!(normalize_drive("c").unwrap(), r"C:\");
        assert_eq!(normalize_drive(r"d:\").unwrap(), r"D:\");
        assert!(normalize_drive("").is_err());
        assert!(normalize_drive(r"C:\Windows").is_err());
        assert!(normalize_drive("1:").is_err());
    }
}
