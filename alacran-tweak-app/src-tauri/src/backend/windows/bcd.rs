//! Boot configuration of the `{current}` loader entry through `bcdedit.exe`
//! (the only child process allowed during apply/revert), started with
//! `CREATE_NO_WINDOW` and fixed arguments.

use std::os::windows::process::CommandExt;
use std::process::Command;

use ::windows::Win32::Globalization::{
    MultiByteToWideChar, CP_OEMCP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS,
};
use ::windows::Win32::System::Threading::CREATE_NO_WINDOW;
use anyhow::{bail, Context};

use super::system_root;
use crate::backend::Result;

/// English element labels every Windows loader entry shows. If any is missing
/// the labels are localized (or the output is not a loader entry), so an
/// absent element could not be told from a present one.
const ANCHORS: &[&str] = &["identifier", "path", "osdevice", "systemroot"];

/// Localized boolean values bcdedit may print, mapped back to the English
/// tokens `bcdedit /set` accepts. Compared case-insensitively.
const LOCALIZED_BOOLS: &[(&str, &str)] = &[
    ("yes", "Yes"),
    ("no", "No"),
    // Turkish
    ("evet", "Yes"),
    ("hayır", "No"),
    ("hayir", "No"),
    // German / Dutch / Scandinavian
    ("ja", "Yes"),
    ("nein", "No"),
    ("nee", "No"),
    ("nej", "No"),
    ("nei", "No"),
    // French
    ("oui", "Yes"),
    ("non", "No"),
    // Spanish / Italian / Portuguese
    ("sí", "Yes"),
    ("si", "Yes"),
    ("sim", "Yes"),
    ("não", "No"),
    // Polish / Czech / Hungarian / Finnish
    ("tak", "Yes"),
    ("nie", "No"),
    ("ano", "Yes"),
    ("ne", "No"),
    ("igen", "Yes"),
    ("nem", "No"),
    ("kyllä", "Yes"),
    ("ei", "No"),
    // Russian
    ("да", "Yes"),
    ("нет", "No"),
];

/// Elements of the entry in `bcdedit /enum {current} /v` output as
/// (lower-cased label, raw value). Only lines after the `----` separator are
/// elements; indented lines are continuations of multi-valued elements and
/// are skipped. Fails unless every English anchor label is present.
pub fn parse_current(output: &str) -> Result<Vec<(String, String)>> {
    let lines: Vec<&str> = output.lines().collect();
    let start = lines
        .iter()
        .position(|l| {
            let t = l.trim();
            !t.is_empty() && t.chars().all(|c| c == '-')
        })
        .map_or(0, |i| i + 1);
    let mut out = Vec::new();
    for line in &lines[start..] {
        if line.is_empty() || line.starts_with(char::is_whitespace) {
            continue;
        }
        let line = line.trim_end();
        let (key, value) = match line.split_once(char::is_whitespace) {
            Some((k, v)) => (k, v.trim()),
            None => (line, ""),
        };
        out.push((key.to_lowercase(), value.to_string()));
    }
    if let Some(missing) = ANCHORS.iter().find(|a| !out.iter().any(|(k, _)| k == *a)) {
        bail!(
            "bcdedit output is not in the expected English form (no '{missing}' element); \
             refusing to read boot settings"
        );
    }
    Ok(out)
}

/// Turns a displayed value into one `bcdedit /set` accepts back: localized
/// booleans become `Yes`/`No`; anything else must be printable ASCII
/// (non-ASCII text is a localized value that cannot be written back).
pub fn normalize_value(name: &str, raw: &str) -> Result<String> {
    let v = raw.trim();
    let lower = v.to_lowercase();
    if let Some((_, token)) = LOCALIZED_BOOLS.iter().find(|(k, _)| *k == lower) {
        return Ok((*token).to_string());
    }
    if v.is_empty() || !v.chars().all(|c| c.is_ascii_graphic() || c == ' ') {
        bail!("BCD value of {name} cannot be restored safely: {raw:?}");
    }
    Ok(v.to_string())
}

/// Value of element `name`, normalized, from `bcdedit /enum {current} /v`
/// output. `Ok(None)` only when the labels are verified English and the
/// element is really absent.
pub fn read_element(output: &str, name: &str) -> Result<Option<String>> {
    let elements = parse_current(output)?;
    let want = name.to_lowercase();
    match elements.iter().find(|(k, _)| *k == want) {
        Some((_, v)) => normalize_value(name, v).map(Some),
        None => Ok(None),
    }
}

/// BCD element names are plain identifiers; values must be a single argument
/// without quotes or control characters.
pub fn validate(name: &str, value: Option<&str>) -> Result<()> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        bail!("invalid BCD element name: {name:?}");
    }
    if let Some(v) = value {
        if v.trim().is_empty() || v.chars().any(|c| c.is_control() || c == '"') {
            bail!("invalid BCD value for {name}: {v:?}");
        }
    }
    Ok(())
}

fn oem_to_string(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    unsafe {
        let flags = MULTI_BYTE_TO_WIDE_CHAR_FLAGS(0);
        let n = MultiByteToWideChar(CP_OEMCP, flags, bytes, None);
        if n <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut buf = vec![0u16; n as usize];
        let n = MultiByteToWideChar(CP_OEMCP, flags, bytes, Some(&mut buf));
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}

fn bcdedit(args: &[&str]) -> Result<String> {
    let exe = system_root().join("System32").join("bcdedit.exe");
    let out = Command::new(&exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW.0)
        .output()
        .with_context(|| format!("cannot start {}", exe.display()))?;
    let stdout = oem_to_string(&out.stdout);
    if !out.status.success() {
        let stderr = oem_to_string(&out.stderr);
        let msg = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        bail!("bcdedit {} failed: {msg}", args.join(" "));
    }
    Ok(stdout)
}

pub fn get(name: &str) -> Result<Option<String>> {
    validate(name, None)?;
    let out = bcdedit(&["/enum", "{current}", "/v"])?;
    read_element(&out, name)
}

pub fn set(name: &str, value: &str) -> Result<()> {
    validate(name, Some(value))?;
    bcdedit(&["/set", "{current}", name, value]).map(|_| ())
}

pub fn delete(name: &str) -> Result<()> {
    validate(name, None)?;
    bcdedit(&["/deletevalue", "{current}", name]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = concat!(
        "\r\n",
        "Windows Boot Loader\r\n",
        "-------------------\r\n",
        "identifier              {a7c3f1e2-1111-11ef-9a3b-c8d9d2a1b0c4}\r\n",
        "device                  partition=C:\r\n",
        "path                    \\WINDOWS\\system32\\winload.efi\r\n",
        "description             Windows 11 Pro\r\n",
        "locale                  tr-TR\r\n",
        "inherit                 {6efb52bf-1766-41db-a6b3-0ee5eff72bd7}\r\n",
        "recoverysequence        {a7c3f1e4-1111-11ef-9a3b-c8d9d2a1b0c4}\r\n",
        "displaymessageoverride  Recovery\r\n",
        "recoveryenabled         Yes\r\n",
        "badmemorylist           0x1\r\n",
        "                        0x2\r\n",
        "allowedinmemorysettings 0x15000075\r\n",
        "osdevice                partition=C:\r\n",
        "systemroot              \\WINDOWS\r\n",
        "nx                      OptIn\r\n",
        "bootmenupolicy          Standard\r\n",
        "disabledynamictick      Yes\r\n",
    );

    /// English labels, Turkish values (as on a Turkish Windows install).
    const TURKISH_VALUES: &str = concat!(
        "\r\n",
        "Windows Önyükleme Yükleyicisi\r\n",
        "-----------------------------\r\n",
        "identifier              {a7c3f1e2-1111-11ef-9a3b-c8d9d2a1b0c4}\r\n",
        "device                  partition=C:\r\n",
        "path                    \\WINDOWS\\system32\\winload.efi\r\n",
        "description             Windows 11 Pro\r\n",
        "recoveryenabled         Evet\r\n",
        "osdevice                partition=C:\r\n",
        "systemroot              \\WINDOWS\r\n",
        "disabledynamictick      Evet\r\n",
        "useplatformclock        Hayır\r\n",
        "bootmenupolicy          Standart değil\r\n",
    );

    /// Localized element labels: an absent-looking element may be present.
    const LOCALIZED_LABELS: &str = concat!(
        "\r\n",
        "Windows Önyükleme Yükleyicisi\r\n",
        "-----------------------------\r\n",
        "tanımlayıcı             {a7c3f1e2-1111-11ef-9a3b-c8d9d2a1b0c4}\r\n",
        "aygıt                   partition=C:\r\n",
        "yol                     \\WINDOWS\\system32\\winload.efi\r\n",
        "açıklama                Windows 11 Pro\r\n",
        "osaygıtı                partition=C:\r\n",
        "sistemkökü              \\WINDOWS\r\n",
        "dinamikişaretdevredışı  Evet\r\n",
    );

    #[test]
    fn reads_english_elements_case_insensitively() {
        let get = |n| read_element(FIXTURE, n).unwrap();
        assert_eq!(get("description").as_deref(), Some("Windows 11 Pro"));
        assert_eq!(get("DisableDynamicTick").as_deref(), Some("Yes"));
        assert_eq!(get("nx").as_deref(), Some("OptIn"));
        assert_eq!(get("badmemorylist").as_deref(), Some("0x1"));
    }

    #[test]
    fn missing_header_words_and_continuations_are_not_elements() {
        assert_eq!(read_element(FIXTURE, "useplatformclock").unwrap(), None);
        assert_eq!(read_element(FIXTURE, "Windows").unwrap(), None);
        assert_eq!(read_element(FIXTURE, "0x2").unwrap(), None);
    }

    #[test]
    fn turkish_booleans_map_back_to_english_tokens() {
        let get = |n| read_element(TURKISH_VALUES, n).unwrap();
        assert_eq!(get("disabledynamictick").as_deref(), Some("Yes"));
        assert_eq!(get("useplatformclock").as_deref(), Some("No"));
        assert_eq!(get("recoveryenabled").as_deref(), Some("Yes"));
        assert_eq!(get("tscsyncpolicy"), None);
    }

    #[test]
    fn unknown_localized_value_is_an_error() {
        let e = read_element(TURKISH_VALUES, "bootmenupolicy").unwrap_err();
        assert!(e.to_string().contains("cannot be restored safely"), "{e}");
    }

    #[test]
    fn localized_labels_fail_closed_instead_of_absent() {
        for name in ["disabledynamictick", "description", "useplatformclock"] {
            let e = read_element(LOCALIZED_LABELS, name).unwrap_err();
            assert!(e.to_string().contains("expected English form"), "{e}");
        }
    }

    #[test]
    fn output_without_separator_or_anchors_fails() {
        assert!(read_element("", "description").is_err());
        assert!(read_element("description  Windows\r\n", "description").is_err());
    }

    #[test]
    fn normalizes_values() {
        assert_eq!(normalize_value("x", " HAYIR ").unwrap(), "No");
        assert_eq!(normalize_value("x", "Ja").unwrap(), "Yes");
        assert_eq!(normalize_value("x", "Нет").unwrap(), "No");
        assert_eq!(normalize_value("x", "0x15000075").unwrap(), "0x15000075");
        assert_eq!(
            normalize_value("x", "partition=C:").unwrap(),
            "partition=C:"
        );
        assert!(normalize_value("x", "").is_err());
        assert!(normalize_value("x", "Açık").is_err());
    }

    #[test]
    fn validates_names_and_values() {
        assert!(validate("useplatformclock", Some("true")).is_ok());
        assert!(validate("tscsyncpolicy", Some("Enhanced")).is_ok());
        assert!(validate("bad name", None).is_err());
        assert!(validate("x;y", None).is_err());
        assert!(validate("x", Some("")).is_err());
        assert!(validate("x", Some("a\"b")).is_err());
        assert!(validate("x", Some("a\nb")).is_err());
    }
}
