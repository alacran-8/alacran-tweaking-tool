//! User settings in `%LOCALAPPDATA%\AlacranTweak\settings.json`.

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// `"tr"` or `"en"`.
    pub lang: String,
    pub animated_bg: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            lang: "tr".into(),
            animated_bg: true,
        }
    }
}

impl Settings {
    fn validate(&self) -> Result<()> {
        if self.lang != "tr" && self.lang != "en" {
            bail!("unsupported language '{}'", self.lang);
        }
        Ok(())
    }
}

/// Settings from `path`; a missing, unreadable or invalid file gives the defaults.
pub fn load(path: &Path) -> Settings {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice::<Settings>(&b).ok())
        .filter(|s| s.validate().is_ok())
        .unwrap_or_default()
}

pub fn save(path: &Path, s: &Settings) -> Result<()> {
    s.validate()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("cannot create {}", dir.display()))?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(s)?)
        .with_context(|| format!("cannot write {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("cannot write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("alacran-settings-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn settings_roundtrip() {
        let dir = temp("rt");
        let p = dir.join("sub").join("settings.json");
        assert_eq!(load(&p), Settings { lang: "tr".into(), animated_bg: true });
        let s = Settings { lang: "en".into(), animated_bg: false };
        save(&p, &s).unwrap();
        assert_eq!(load(&p), s);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unreadable_file_gives_defaults() {
        let dir = temp("bad");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("settings.json");
        std::fs::write(&p, b"{not json").unwrap();
        assert_eq!(load(&p), Settings::default());
        std::fs::write(&p, br#"{"lang":"xx","animated_bg":false}"#).unwrap();
        assert_eq!(load(&p), Settings::default());
        std::fs::write(&p, br#"{"animated_bg":false}"#).unwrap();
        assert_eq!(load(&p), Settings { lang: "tr".into(), animated_bg: false });
        assert!(save(&p, &Settings { lang: "de".into(), animated_bg: true }).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
