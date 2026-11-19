//! Registry access through `winreg`. `Hive::Hkcu` maps to `HKEY_USERS\<sid>`
//! of the interactive user, never to the (possibly different) elevated
//! account's own `HKEY_CURRENT_USER`.

use std::borrow::Cow;
use std::io::ErrorKind;

use anyhow::{anyhow, bail, Context};
use winreg::enums::{
    RegType, HKEY_LOCAL_MACHINE, HKEY_USERS, KEY_QUERY_VALUE, KEY_READ, KEY_SET_VALUE, KEY_WRITE,
};
use winreg::{RegKey, RegValue};

use super::NO_USER_SESSION;
use crate::backend::Result;
use crate::catalog::{Hive, RegData};

/// `path` relative to `HKEY_USERS` for the HKCU path of `sid`.
pub fn hkcu_subpath(sid: &str, path: &str) -> String {
    let path = path.trim_matches('\\');
    if path.is_empty() {
        sid.to_string()
    } else {
        format!("{sid}\\{path}")
    }
}

/// Root key and the subpath below it for a catalog (hive, path).
fn locate(h: Hive, sid: &str, path: &str) -> Result<(RegKey, String)> {
    match h {
        Hive::Hklm => Ok((
            RegKey::predef(HKEY_LOCAL_MACHINE),
            path.trim_matches('\\').to_string(),
        )),
        Hive::Hkcu => {
            if sid.is_empty() {
                bail!(NO_USER_SESSION);
            }
            let users = RegKey::predef(HKEY_USERS);
            // The hive is only present while the user is logged on.
            if users.open_subkey_with_flags(sid, KEY_QUERY_VALUE).is_err() {
                bail!(NO_USER_SESSION);
            }
            Ok((users, hkcu_subpath(sid, path)))
        }
    }
}

fn display(h: Hive, path: &str) -> String {
    let root = match h {
        Hive::Hklm => "HKLM",
        Hive::Hkcu => "HKCU",
    };
    format!("{root}\\{}", path.trim_matches('\\'))
}

/// Opens an existing key; `Ok(None)` when it does not exist.
fn open(h: Hive, sid: &str, path: &str, flags: u32) -> Result<Option<RegKey>> {
    let (root, sub) = locate(h, sid, path)?;
    let opened = if sub.is_empty() {
        root.open_subkey_with_flags("", flags)
    } else {
        root.open_subkey_with_flags(&sub, flags)
    };
    match opened {
        Ok(k) => Ok(Some(k)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(anyhow!(e)).with_context(|| format!("cannot open {}", display(h, path))),
    }
}

pub fn get(h: Hive, sid: &str, path: &str, name: &str) -> Result<Option<RegData>> {
    let Some(key) = open(h, sid, path, KEY_READ)? else {
        return Ok(None);
    };
    match key.get_raw_value(name) {
        Ok(v) => decode(&v.vtype, &v.bytes)
            .map(Some)
            .with_context(|| format!("{}\\{name}", display(h, path))),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => {
            Err(anyhow!(e)).with_context(|| format!("cannot read {}\\{name}", display(h, path)))
        }
    }
}

pub fn key_exists(h: Hive, sid: &str, path: &str) -> Result<bool> {
    Ok(open(h, sid, path, KEY_QUERY_VALUE)?.is_some())
}

pub fn set(h: Hive, sid: &str, path: &str, name: &str, data: &RegData) -> Result<()> {
    let (root, sub) = locate(h, sid, path)?;
    if sub.is_empty() {
        bail!("refusing to write a value at a hive root");
    }
    let (key, _) = root
        .create_subkey_with_flags(&sub, KEY_WRITE)
        .with_context(|| format!("cannot create/open {}", display(h, path)))?;
    let value = encode(data)?;
    key.set_raw_value(name, &value)
        .with_context(|| format!("cannot write {}\\{name}", display(h, path)))
}

pub fn delete_value(h: Hive, sid: &str, path: &str, name: &str) -> Result<()> {
    let Some(key) = open(h, sid, path, KEY_SET_VALUE)? else {
        return Ok(());
    };
    match key.delete_value(name) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => {
            Err(anyhow!(e)).with_context(|| format!("cannot delete {}\\{name}", display(h, path)))
        }
    }
}

/// Deletes the key at `path` only when it has neither values nor subkeys.
/// Hive roots (and `HKU\<sid>` itself) are never deleted.
pub fn delete_key_if_empty(h: Hive, sid: &str, path: &str) -> Result<()> {
    let rel = path.trim_matches('\\');
    let Some((parent, leaf)) = split_parent(rel) else {
        // A key directly below the hive root is never deleted either.
        return Ok(());
    };
    let Some(key) = open(h, sid, rel, KEY_READ)? else {
        return Ok(());
    };
    let info = key
        .query_info()
        .with_context(|| format!("cannot query {}", display(h, path)))?;
    if info.sub_keys != 0 || info.values != 0 {
        return Ok(());
    }
    drop(key);
    let Some(parent_key) = open(h, sid, parent, KEY_WRITE)? else {
        return Ok(());
    };
    match parent_key.delete_subkey(leaf) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => {
            Err(anyhow!(e)).with_context(|| format!("cannot delete key {}", display(h, path)))
        }
    }
}

/// ("a\b", "c") for "a\b\c"; `None` when there is no parent below the root.
pub fn split_parent(path: &str) -> Option<(&str, &str)> {
    let path = path.trim_matches('\\');
    let (parent, leaf) = path.rsplit_once('\\')?;
    if parent.is_empty() || leaf.is_empty() {
        return None;
    }
    Some((parent, leaf))
}

fn utf16_bytes(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn hex_to_bytes(h: &str) -> Result<Vec<u8>> {
    if !h.len().is_multiple_of(2) {
        bail!("binary data must be even-length hex");
    }
    if !h.is_ascii() {
        bail!("invalid hex in binary data");
    }
    (0..h.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&h[i..i + 2], 16).map_err(|_| anyhow!("invalid hex in binary data"))
        })
        .collect()
}

/// Catalog data to a raw registry value.
pub fn encode(data: &RegData) -> Result<RegValue<'static>> {
    let (vtype, bytes) = match data {
        RegData::Dword(v) => (RegType::REG_DWORD, v.to_le_bytes().to_vec()),
        RegData::Qword(v) => (RegType::REG_QWORD, v.to_le_bytes().to_vec()),
        RegData::Sz(s) => (RegType::REG_SZ, utf16_bytes(s)),
        RegData::ExpandSz(s) => (RegType::REG_EXPAND_SZ, utf16_bytes(s)),
        RegData::MultiSz(items) => {
            let mut units: Vec<u16> = Vec::new();
            for s in items {
                units.extend(s.encode_utf16());
                units.push(0);
            }
            if items.is_empty() {
                units.push(0);
            }
            units.push(0);
            (
                RegType::REG_MULTI_SZ,
                units.into_iter().flat_map(u16::to_le_bytes).collect(),
            )
        }
        RegData::Binary(h) => (RegType::REG_BINARY, hex_to_bytes(h)?),
    };
    Ok(RegValue {
        bytes: Cow::Owned(bytes),
        vtype,
    })
}

fn utf16_units(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect()
}

fn utf16_string(bytes: &[u8]) -> String {
    let units = utf16_units(bytes);
    let end = units.iter().position(|&u| u == 0).unwrap_or(units.len());
    String::from_utf16_lossy(&units[..end])
}

/// Raw registry value to catalog data. Types the catalog cannot express
/// (REG_NONE, REG_LINK, resource lists, …) are an error so that no write ever
/// changes a value's type behind the user's back.
pub fn decode(vtype: &RegType, bytes: &[u8]) -> Result<RegData> {
    Ok(match vtype {
        RegType::REG_DWORD => {
            let b: [u8; 4] = bytes
                .get(..4)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| anyhow!("REG_DWORD shorter than 4 bytes"))?;
            RegData::Dword(u32::from_le_bytes(b))
        }
        RegType::REG_QWORD => {
            let b: [u8; 8] = bytes
                .get(..8)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| anyhow!("REG_QWORD shorter than 8 bytes"))?;
            RegData::Qword(u64::from_le_bytes(b))
        }
        RegType::REG_SZ => RegData::Sz(utf16_string(bytes)),
        RegType::REG_EXPAND_SZ => RegData::ExpandSz(utf16_string(bytes)),
        RegType::REG_MULTI_SZ => {
            let units = utf16_units(bytes);
            let items = units
                .split(|&u| u == 0)
                .take_while(|s| !s.is_empty())
                .map(String::from_utf16_lossy)
                .collect();
            RegData::MultiSz(items)
        }
        RegType::REG_BINARY => RegData::Binary(bytes.iter().map(|b| format!("{b:02x}")).collect()),
        other => bail!("unsupported registry value type {other:?}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(d: RegData) {
        let v = encode(&d).unwrap();
        assert_eq!(decode(&v.vtype, &v.bytes).unwrap(), d);
    }

    #[test]
    fn encodes_and_decodes_every_type() {
        roundtrip(RegData::Dword(0xdead_beef));
        roundtrip(RegData::Qword(0x0102_0304_0506_0708));
        roundtrip(RegData::Sz("Çalışma ğüşiöç".into()));
        roundtrip(RegData::Sz(String::new()));
        roundtrip(RegData::ExpandSz("%SystemRoot%\\x".into()));
        roundtrip(RegData::MultiSz(vec!["a".into(), "bc".into()]));
        roundtrip(RegData::MultiSz(vec![]));
        roundtrip(RegData::Binary("00ff10".into()));
    }

    #[test]
    fn dword_layout_is_little_endian() {
        let v = encode(&RegData::Dword(1)).unwrap();
        assert_eq!(v.vtype, RegType::REG_DWORD);
        assert_eq!(&*v.bytes, &[1, 0, 0, 0]);
    }

    #[test]
    fn sz_without_terminator_and_with_padding_decodes() {
        let raw: Vec<u8> = "ab".encode_utf16().flat_map(u16::to_le_bytes).collect();
        assert_eq!(
            decode(&RegType::REG_SZ, &raw).unwrap(),
            RegData::Sz("ab".into())
        );
        let mut padded = raw.clone();
        padded.extend([0, 0, 0, 0, 0]);
        assert_eq!(
            decode(&RegType::REG_SZ, &padded).unwrap(),
            RegData::Sz("ab".into())
        );
    }

    #[test]
    fn binary_decodes_lowercase_hex_and_rejects_bad_hex() {
        assert_eq!(
            decode(&RegType::REG_BINARY, &[0xAB, 0x01]).unwrap(),
            RegData::Binary("ab01".into())
        );
        assert!(encode(&RegData::Binary("abc".into())).is_err());
        assert!(encode(&RegData::Binary("zz".into())).is_err());
        // Corrupt backup text must be an error, not a slice-boundary panic.
        assert!(encode(&RegData::Binary("çç".into())).is_err());
        assert!(encode(&RegData::Binary("aça".into())).is_err());
    }

    #[test]
    fn unsupported_and_short_values_are_errors() {
        assert!(decode(&RegType::REG_NONE, &[]).is_err());
        assert!(decode(&RegType::REG_DWORD, &[1, 2]).is_err());
    }

    #[test]
    fn hkcu_maps_below_sid() {
        assert_eq!(
            hkcu_subpath("S-1-5-21-1-2-3-1001", "\\Software\\X\\"),
            "S-1-5-21-1-2-3-1001\\Software\\X"
        );
        assert_eq!(hkcu_subpath("S-1-5-21-1", ""), "S-1-5-21-1");
    }

    #[test]
    fn hkcu_without_sid_is_no_user_session() {
        let e = get(Hive::Hkcu, "", "Software", "x").unwrap_err();
        assert_eq!(e.to_string(), NO_USER_SESSION);
    }

    #[test]
    fn split_parent_never_yields_the_root() {
        assert_eq!(split_parent("SOFTWARE\\A\\B"), Some(("SOFTWARE\\A", "B")));
        assert_eq!(split_parent("SOFTWARE"), None);
        assert_eq!(split_parent("\\SOFTWARE\\"), None);
    }
}
