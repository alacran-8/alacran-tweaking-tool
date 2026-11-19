//! Power scheme values through the Power Management API. Values are read from
//! and written to an explicit scheme; GUIDs are lower-case, no braces.

use ::windows::core::GUID;
use ::windows::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL, WIN32_ERROR};
use ::windows::Win32::System::Power::{
    PowerDuplicateScheme, PowerGetActiveScheme, PowerReadACValueIndex, PowerReadDCValueIndex,
    PowerSetActiveScheme, PowerWriteACValueIndex, PowerWriteDCValueIndex,
};
use anyhow::{anyhow, bail};

use crate::backend::Result;

/// Parses `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`, optionally in braces.
pub fn parse_guid(s: &str) -> Result<GUID> {
    let t = s.trim();
    let t = t
        .strip_prefix('{')
        .and_then(|x| x.strip_suffix('}'))
        .unwrap_or(t);
    let groups: Vec<&str> = t.split('-').collect();
    let lens = [8, 4, 4, 4, 12];
    if groups.len() != 5
        || groups
            .iter()
            .zip(lens)
            .any(|(g, n)| g.len() != n || !g.chars().all(|c| c.is_ascii_hexdigit()))
    {
        bail!("invalid GUID: {s}");
    }
    let hex: String = groups.concat();
    let v = u128::from_str_radix(&hex, 16).map_err(|_| anyhow!("invalid GUID: {s}"))?;
    Ok(GUID::from_u128(v))
}

pub fn format_guid(g: &GUID) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        g.data1,
        g.data2,
        g.data3,
        g.data4[0],
        g.data4[1],
        g.data4[2],
        g.data4[3],
        g.data4[4],
        g.data4[5],
        g.data4[6],
        g.data4[7]
    )
}

fn check(code: u32, what: &str) -> Result<()> {
    if code == ERROR_SUCCESS.0 {
        return Ok(());
    }
    let err = ::windows::core::Error::from(WIN32_ERROR(code).to_hresult());
    Err(anyhow::Error::new(err).context(what.to_string()))
}

/// Takes ownership of a GUID allocated by the power API.
unsafe fn take_guid(p: *mut GUID) -> Result<GUID> {
    if p.is_null() {
        bail!("power API returned no GUID");
    }
    let g = *p;
    let _ = LocalFree(Some(HLOCAL(p.cast())));
    Ok(g)
}

fn active() -> Result<GUID> {
    let mut p: *mut GUID = std::ptr::null_mut();
    unsafe {
        check(
            PowerGetActiveScheme(None, &mut p).0,
            "cannot read the active power scheme",
        )?;
        take_guid(p)
    }
}

pub fn active_scheme() -> Result<String> {
    Ok(format_guid(&active()?))
}

pub fn get(scheme: &str, subgroup: &str, setting: &str) -> Result<(u32, u32)> {
    let scheme = parse_guid(scheme)?;
    let sub = parse_guid(subgroup)?;
    let set = parse_guid(setting)?;
    let what = format!("power setting not found: {subgroup}/{setting}");
    let (mut ac, mut dc) = (0u32, 0u32);
    unsafe {
        check(
            PowerReadACValueIndex(None, Some(&scheme), Some(&sub), Some(&set), &mut ac).0,
            &what,
        )?;
        check(
            PowerReadDCValueIndex(None, Some(&scheme), Some(&sub), Some(&set), &mut dc),
            &what,
        )?;
    }
    Ok((ac, dc))
}

pub fn set(
    scheme: &str,
    subgroup: &str,
    setting: &str,
    ac: Option<u32>,
    dc: Option<u32>,
) -> Result<()> {
    let scheme = parse_guid(scheme)?;
    let is_active = active()? == scheme;
    let sub = parse_guid(subgroup)?;
    let set = parse_guid(setting)?;
    let what = format!("cannot write power setting {subgroup}/{setting}");
    unsafe {
        if let Some(v) = ac {
            check(
                PowerWriteACValueIndex(None, &scheme, Some(&sub), Some(&set), v).0,
                &what,
            )?;
        }
        if let Some(v) = dc {
            check(
                PowerWriteDCValueIndex(None, &scheme, Some(&sub), Some(&set), v),
                &what,
            )?;
        }
        // Re-activating the active scheme makes the new values take effect;
        // an inactive scheme is never activated.
        if is_active {
            check(
                PowerSetActiveScheme(None, Some(&scheme)).0,
                "cannot re-apply the power scheme",
            )?;
        }
    }
    Ok(())
}

pub fn set_active_scheme(guid: &str) -> Result<()> {
    let g = parse_guid(guid)?;
    unsafe {
        check(
            PowerSetActiveScheme(None, Some(&g)).0,
            &format!("cannot activate power scheme {guid}"),
        )
    }
}

pub fn duplicate_scheme(source: &str) -> Result<String> {
    let src = parse_guid(source)?;
    let mut p: *mut GUID = std::ptr::null_mut();
    unsafe {
        check(
            PowerDuplicateScheme(None, &src, &mut p).0,
            &format!("cannot duplicate power scheme {source}"),
        )?;
        Ok(format_guid(&take_guid(p)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guid_roundtrip_and_braces() {
        let s = "e9a42b02-d5df-448d-aa00-03f14749eb61";
        assert_eq!(format_guid(&parse_guid(s).unwrap()), s);
        assert_eq!(
            format_guid(&parse_guid("{E9A42B02-D5DF-448D-AA00-03F14749EB61}").unwrap()),
            s
        );
        assert_eq!(
            parse_guid("54533251-82be-4824-96c1-47b60b740d00").unwrap(),
            GUID::from_u128(0x54533251_82be_4824_96c1_47b60b740d00)
        );
    }

    #[test]
    fn rejects_aliases_and_garbage() {
        assert!(parse_guid("SUB_PROCESSOR").is_err());
        assert!(parse_guid("e9a42b02-d5df-448d-aa00-03f14749eb6").is_err());
        assert!(parse_guid("e9a42b02d5df448daa0003f14749eb61").is_err());
        assert!(parse_guid("g9a42b02-d5df-448d-aa00-03f14749eb61").is_err());
    }
}
