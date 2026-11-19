//! Store app removal through `Windows.Management.Deployment.PackageManager`.

use ::windows::core::HSTRING;
use ::windows::Management::Deployment::PackageManager;
use anyhow::{bail, Context};

use super::ComGuard;
use crate::backend::Result;

/// Case-insensitive match of `text` against `pattern`, where `*` matches any
/// run of characters (including none). No other wildcard is special.
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.to_lowercase().chars().collect();
    let t: Vec<char> = text.to_lowercase().chars().collect();
    let (mut pi, mut ti) = (0usize, 0usize);
    let mut star: Option<(usize, usize)> = None;
    while ti < t.len() {
        if pi < p.len() && p[pi] == '*' {
            star = Some((pi, ti));
            pi += 1;
        } else if pi < p.len() && p[pi] == t[ti] {
            pi += 1;
            ti += 1;
        } else if let Some((sp, st)) = star {
            pi = sp + 1;
            ti = st + 1;
            star = Some((sp, st + 1));
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

/// Removes every package of the interactive user (who must be the process
/// account) whose family name matches
/// `pattern`. Returns the number removed; any failed removal is an error that
/// names the packages and how many were removed before it.
pub fn remove(pattern: &str, interactive_sid: &str) -> Result<u32> {
    if pattern.trim().trim_matches('*').is_empty() {
        bail!("refusing an appx pattern that matches every package: {pattern:?}");
    }
    // The package manager acts on the process account; under over-the-shoulder
    // UAC that is another admin and removal would silently do nothing.
    super::session::require_process_is_user(interactive_sid)?;
    let _com = ComGuard::new()?;
    let pm = PackageManager::new().context("cannot create the package manager")?;
    // The Rust projection names `FindPackagesForUser(String)` this way; an
    // empty SID means the current user.
    let packages = pm
        .FindPackagesByUserSecurityId(&HSTRING::new())
        .context("cannot list installed packages")?;
    let mut removed = 0u32;
    let mut failed: Vec<String> = Vec::new();
    for pkg in packages {
        let Ok(id) = pkg.Id() else { continue };
        let (Ok(family), Ok(full)) = (id.FamilyName(), id.FullName()) else {
            continue;
        };
        if !wildcard_match(pattern, &family.to_string_lossy()) {
            continue;
        }
        let outcome = pm
            .RemovePackageAsync(&full)
            .and_then(|op| op.get())
            .and_then(|r| {
                let code = r.ExtendedErrorCode()?;
                if code.is_ok() {
                    Ok(())
                } else {
                    let text = r
                        .ErrorText()
                        .map(|t| t.to_string_lossy())
                        .unwrap_or_default();
                    Err(::windows::core::Error::new(code, text))
                }
            });
        match outcome {
            Ok(()) => removed += 1,
            Err(e) => failed.push(format!("{full}: {e}")),
        }
    }
    if !failed.is_empty() {
        bail!("removed {removed}, failed: {}", failed.join("; "));
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::wildcard_match;

    #[test]
    fn matches_family_names() {
        let fam = "Microsoft.BingWeather_8wekyb3d8bbwe";
        assert!(wildcard_match("Microsoft.BingWeather*", fam));
        assert!(wildcard_match("microsoft.bingweather*", fam));
        assert!(wildcard_match("*BingWeather*", fam));
        assert!(wildcard_match(fam, fam));
        assert!(wildcard_match("Microsoft.*_8wekyb3d8bbwe", fam));
        assert!(!wildcard_match("Microsoft.BingNews*", fam));
        assert!(!wildcard_match("Microsoft.BingWeather", fam));
        assert!(!wildcard_match("BingWeather*", fam));
    }

    #[test]
    fn star_edge_cases() {
        assert!(wildcard_match("*", "anything"));
        assert!(wildcard_match("a**b", "ab"));
        assert!(wildcard_match("a*b*c", "aXbYc"));
        assert!(!wildcard_match("a*b*c", "aXbY"));
        assert!(wildcard_match("", ""));
        assert!(!wildcard_match("", "x"));
    }
}
