//! The interactive (console) user: SID and profile directory.

use std::path::PathBuf;

use ::windows::core::{PCWSTR, PWSTR};
use ::windows::Win32::Foundation::{CloseHandle, LocalFree, HANDLE, HLOCAL};
use ::windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use ::windows::Win32::Security::{
    GetTokenInformation, LookupAccountNameW, TokenUser, PSID, SID_NAME_USE, TOKEN_USER,
};
use ::windows::Win32::System::Environment::ExpandEnvironmentStringsW;
use ::windows::Win32::System::RemoteDesktop::{
    WTSDomainName, WTSFreeMemory, WTSGetActiveConsoleSessionId, WTSQuerySessionInformationW,
    WTSQueryUserToken, WTSUserName, WTS_INFO_CLASS,
};
use anyhow::{anyhow, Context};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use super::{wide, NO_USER_SESSION};
use crate::backend::Result;

/// SID string ("S-1-5-21-…") of the user logged on at the console, or `None`
/// when nobody is logged on.
///
/// `WTSQueryUserToken` needs `SeTcbPrivilege` (LocalSystem). An elevated
/// administrator does not hold it, so the fallback resolves the console
/// session's user name through `LookupAccountNameW`.
pub fn interactive_sid() -> Option<String> {
    let session = unsafe { WTSGetActiveConsoleSessionId() };
    if session == u32::MAX {
        return None;
    }
    sid_from_user_token(session).or_else(|| sid_from_session_user_name(session))
}

fn sid_from_user_token(session: u32) -> Option<String> {
    unsafe {
        let mut token = HANDLE::default();
        WTSQueryUserToken(session, &mut token).ok()?;
        let sid = token_user_sid(token);
        let _ = CloseHandle(token);
        sid
    }
}

unsafe fn token_user_sid(token: HANDLE) -> Option<String> {
    let mut len = 0u32;
    let _ = GetTokenInformation(token, TokenUser, None, 0, &mut len);
    if len == 0 {
        return None;
    }
    // u64 storage keeps TOKEN_USER suitably aligned.
    let mut buf = vec![0u64; (len as usize).div_ceil(8)];
    GetTokenInformation(
        token,
        TokenUser,
        Some(buf.as_mut_ptr().cast()),
        len,
        &mut len,
    )
    .ok()?;
    let user = &*(buf.as_ptr() as *const TOKEN_USER);
    sid_to_string(user.User.Sid)
}

unsafe fn sid_to_string(sid: PSID) -> Option<String> {
    let mut out = PWSTR::null();
    ConvertSidToStringSidW(sid, &mut out).ok()?;
    let s = out.to_string().ok();
    let _ = LocalFree(Some(HLOCAL(out.0.cast())));
    s
}

unsafe fn session_string(session: u32, class: WTS_INFO_CLASS) -> Option<String> {
    let mut buf = PWSTR::null();
    let mut bytes = 0u32;
    WTSQuerySessionInformationW(None, session, class, &mut buf, &mut bytes).ok()?;
    let s = buf.to_string().ok();
    WTSFreeMemory(buf.0.cast());
    s
}

fn sid_from_session_user_name(session: u32) -> Option<String> {
    unsafe {
        let user = session_string(session, WTSUserName)?;
        if user.is_empty() {
            return None;
        }
        let domain = session_string(session, WTSDomainName).unwrap_or_default();
        let account = if domain.is_empty() {
            user
        } else {
            format!("{domain}\\{user}")
        };
        let account_w = wide(&account);

        let mut sid_len = 0u32;
        let mut dom_len = 0u32;
        let mut use_ = SID_NAME_USE::default();
        let _ = LookupAccountNameW(
            PCWSTR::null(),
            PCWSTR(account_w.as_ptr()),
            None,
            &mut sid_len,
            None,
            &mut dom_len,
            &mut use_,
        );
        if sid_len == 0 {
            return None;
        }
        let mut sid_buf = vec![0u64; (sid_len as usize).div_ceil(8)];
        let mut dom_buf = vec![0u16; dom_len.max(1) as usize];
        LookupAccountNameW(
            PCWSTR::null(),
            PCWSTR(account_w.as_ptr()),
            Some(PSID(sid_buf.as_mut_ptr().cast())),
            &mut sid_len,
            Some(PWSTR(dom_buf.as_mut_ptr())),
            &mut dom_len,
            &mut use_,
        )
        .ok()?;
        sid_to_string(PSID(sid_buf.as_mut_ptr().cast()))
    }
}

/// Expands `%VAR%` references with the process environment.
pub fn expand_env(s: &str) -> String {
    let src = wide(s);
    unsafe {
        let n = ExpandEnvironmentStringsW(PCWSTR(src.as_ptr()), None);
        if n == 0 {
            return s.to_string();
        }
        let mut buf = vec![0u16; n as usize];
        let n = ExpandEnvironmentStringsW(PCWSTR(src.as_ptr()), Some(&mut buf));
        if n == 0 || n as usize > buf.len() {
            return s.to_string();
        }
        String::from_utf16_lossy(&buf[..n as usize - 1])
    }
}

/// Error when the process runs as a different account than the console user
/// (over-the-shoulder UAC elevation).
pub const OTHER_ADMIN: &str =
    "uygulama farklı bir yönetici hesabıyla çalışıyor; bu işlem oturum açan kullanıcı için yapılamaz";

/// SID string of the account this process runs as (its token user).
pub fn process_user_sid() -> Option<String> {
    use ::windows::Win32::Security::TOKEN_QUERY;
    use ::windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).ok()?;
        let sid = token_user_sid(token);
        let _ = CloseHandle(token);
        sid
    }
}

/// Ok only when the process account is the interactive user, so APIs that act
/// on "the current user" (package manager, recycle bin) hit the right one.
pub fn check_same_user(process_sid: Option<&str>, interactive_sid: &str) -> Result<()> {
    if interactive_sid.is_empty() {
        return Err(anyhow!(NO_USER_SESSION));
    }
    match process_sid {
        Some(p) if p.eq_ignore_ascii_case(interactive_sid) => Ok(()),
        _ => Err(anyhow!(OTHER_ADMIN)),
    }
}

/// `check_same_user` against this process's token.
pub fn require_process_is_user(interactive_sid: &str) -> Result<()> {
    check_same_user(process_user_sid().as_deref(), interactive_sid)
}

/// Profile directory (e.g. `C:\Users\name`) of the user with `sid`.
pub fn profile_dir(sid: &str) -> Result<PathBuf> {
    if sid.is_empty() {
        return Err(anyhow!(NO_USER_SESSION));
    }
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(format!(
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList\{sid}"
        ))
        .map_err(|_| anyhow!(NO_USER_SESSION))?;
    let raw: String = key
        .get_value("ProfileImagePath")
        .context("cannot read ProfileImagePath")?;
    let path = PathBuf::from(expand_env(&raw));
    if !path.is_absolute() {
        return Err(anyhow!("invalid profile path: {}", path.display()));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER: &str = "S-1-5-21-1111-2222-3333-1001";
    const ADMIN: &str = "S-1-5-21-1111-2222-3333-500";

    #[test]
    fn same_account_elevation_passes() {
        assert!(check_same_user(Some(USER), USER).is_ok());
        assert!(check_same_user(Some(&USER.to_lowercase()), USER).is_ok());
    }

    #[test]
    fn other_admin_or_unknown_process_user_fails() {
        let e = check_same_user(Some(ADMIN), USER).unwrap_err();
        assert_eq!(e.to_string(), OTHER_ADMIN);
        let e = check_same_user(None, USER).unwrap_err();
        assert_eq!(e.to_string(), OTHER_ADMIN);
    }

    #[test]
    fn no_interactive_user_is_no_session() {
        let e = check_same_user(Some(USER), "").unwrap_err();
        assert_eq!(e.to_string(), NO_USER_SESSION);
    }
}
