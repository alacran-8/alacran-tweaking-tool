//! One-way cleanup of temporary files. Files that are locked or otherwise
//! cannot be deleted are skipped; links are removed without following them.

use std::fs;
use std::path::{Path, PathBuf};

use ::windows::core::PCWSTR;
use ::windows::Win32::UI::Shell::{
    SHEmptyRecycleBinW, SHQueryRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI,
    SHERB_NOSOUND, SHQUERYRBINFO,
};
use anyhow::{bail, Context};

use super::{session, system_root};
use crate::backend::Result;
use crate::catalog::CleanupTarget;

pub fn run(t: CleanupTarget, sid: &str) -> Result<u64> {
    match t {
        CleanupTarget::UserTemp => {
            let dir = session::profile_dir(sid)?.join(r"AppData\Local\Temp");
            clear_dir(&dir)
        }
        CleanupTarget::WindowsTemp => clear_dir(&system_root().join("Temp")),
        CleanupTarget::UpdateDownload => {
            clear_dir(&system_root().join(r"SoftwareDistribution\Download"))
        }
        CleanupTarget::ThumbCache => {
            let dir = session::profile_dir(sid)?.join(r"AppData\Local\Microsoft\Windows\Explorer");
            Ok(delete_thumbcache(&dir))
        }
        CleanupTarget::RecycleBin => {
            // SHEmptyRecycleBinW empties the process account's bin.
            session::require_process_is_user(sid)?;
            empty_recycle_bin()
        }
    }
}

/// Refuses roots like `C:\` or `C:\Windows` so a bad path can never wipe a
/// whole drive or system directory.
fn check_root(dir: &Path) -> Result<()> {
    if !dir.is_absolute() || dir.components().count() < 4 {
        bail!("refusing to clean {}", dir.display());
    }
    Ok(())
}

fn clear_dir(dir: &Path) -> Result<u64> {
    check_root(dir)?;
    if !dir.exists() {
        return Ok(0);
    }
    Ok(clear_dir_contents(dir))
}

fn remove_file_forced(p: &Path) -> bool {
    if fs::remove_file(p).is_ok() {
        return true;
    }
    // Read-only files need the attribute cleared first.
    if let Ok(meta) = fs::metadata(p) {
        let mut perm = meta.permissions();
        if perm.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            perm.set_readonly(false);
            if fs::set_permissions(p, perm).is_ok() {
                return fs::remove_file(p).is_ok();
            }
        }
    }
    false
}

/// Deletes everything below `dir` (not `dir` itself) and returns the bytes of
/// the regular files that were actually deleted.
pub fn clear_dir_contents(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut freed = 0u64;
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        let ft = meta.file_type();
        if ft.is_symlink() {
            // Symlinks and junctions: remove the link only.
            if fs::remove_file(&path).is_err() {
                let _ = fs::remove_dir(&path);
            }
        } else if ft.is_dir() {
            freed += clear_dir_contents(&path);
            let _ = fs::remove_dir(&path);
        } else if remove_file_forced(&path) {
            freed += meta.len();
        }
    }
    freed
}

/// `thumbcache_*.db` in the Explorer cache directory.
pub fn is_thumbcache_file(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("thumbcache_") && n.ends_with(".db")
}

fn delete_thumbcache(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut freed = 0u64;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_thumbcache_file(&name) {
            continue;
        }
        let path: PathBuf = entry.path();
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.is_file() && remove_file_forced(&path) {
            freed += meta.len();
        }
    }
    freed
}

fn empty_recycle_bin() -> Result<u64> {
    let mut info = SHQUERYRBINFO {
        cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
        ..Default::default()
    };
    unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut info) }
        .context("cannot query the recycle bin")?;
    if info.i64NumItems == 0 {
        return Ok(0);
    }
    unsafe {
        SHEmptyRecycleBinW(
            None,
            PCWSTR::null(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
        )
    }
    .context("cannot empty the recycle bin")?;
    Ok(info.i64Size.max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbcache_names() {
        assert!(is_thumbcache_file("thumbcache_256.db"));
        assert!(is_thumbcache_file("THUMBCACHE_IDX.DB"));
        assert!(!is_thumbcache_file("iconcache_16.db"));
        assert!(!is_thumbcache_file("thumbcache_256.db-wal"));
    }

    #[test]
    fn refuses_shallow_roots() {
        assert!(check_root(Path::new(r"C:\")).is_err());
        assert!(check_root(Path::new(r"C:\Windows")).is_err());
        assert!(check_root(Path::new(r"relative\a\b")).is_err());
        assert!(check_root(Path::new(r"C:\Windows\Temp")).is_ok());
    }

    /// Works only inside a scratch directory this test creates itself.
    #[test]
    fn clears_contents_but_keeps_root() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "alacran-cleanup-test-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(root.join(r"sub\deeper")).unwrap();
        fs::write(root.join("a.tmp"), [0u8; 10]).unwrap();
        fs::write(root.join(r"sub\b.tmp"), [0u8; 20]).unwrap();
        let ro = root.join(r"sub\deeper\ro.tmp");
        fs::write(&ro, [0u8; 5]).unwrap();
        let mut perm = fs::metadata(&ro).unwrap().permissions();
        perm.set_readonly(true);
        fs::set_permissions(&ro, perm).unwrap();

        assert_eq!(clear_dir_contents(&root), 35);
        assert!(root.exists());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 0);
        fs::remove_dir(&root).unwrap();
    }
}
