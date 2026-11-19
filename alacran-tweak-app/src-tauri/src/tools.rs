//! Maintenance tools the user starts explicitly from the Tools screen. They
//! are not tweaks: nothing is backed up and nothing here runs during
//! apply/revert.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolId {
    Sfc,
    Dism,
    DnsFlush,
    WinsockReset,
    PerfCounters,
    OneDriveUninstall,
    DefragGui,
    PerfOptionsGui,
    MouseCpl,
    KeyboardCpl,
    SystemRestoreGui,
}

/// How a tool is started. Executable names are resolved in `System32`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    /// Visible console window (`CREATE_NEW_CONSOLE`): exe + arguments.
    Console(&'static str, &'static str),
    /// `ShellExecuteW` "open": exe + parameters.
    Shell(&'static str, &'static str),
    /// `DnsFlushResolverCache` in `dnsapi.dll`.
    DnsFlush,
    /// `OneDriveSetup.exe /uninstall` from `System32`, else `SysWOW64`.
    OneDriveUninstall,
}

pub fn launch_of(id: ToolId) -> Launch {
    match id {
        ToolId::Sfc => Launch::Console("sfc.exe", "/scannow"),
        ToolId::Dism => Launch::Console("DISM.exe", "/Online /Cleanup-Image /RestoreHealth"),
        ToolId::WinsockReset => Launch::Console("netsh.exe", "winsock reset"),
        ToolId::PerfCounters => Launch::Console("lodctr.exe", "/r"),
        ToolId::DnsFlush => Launch::DnsFlush,
        ToolId::OneDriveUninstall => Launch::OneDriveUninstall,
        ToolId::DefragGui => Launch::Shell("dfrgui.exe", ""),
        ToolId::PerfOptionsGui => Launch::Shell("SystemPropertiesPerformance.exe", ""),
        ToolId::MouseCpl => Launch::Shell("control.exe", "main.cpl"),
        ToolId::KeyboardCpl => Launch::Shell("control.exe", "keyboard"),
        ToolId::SystemRestoreGui => Launch::Shell("rstrui.exe", ""),
    }
}

#[cfg(windows)]
pub use imp::run_tool;

#[cfg(windows)]
mod imp {
    use std::path::Path;

    use ::windows::core::{s, w, BOOL, PCWSTR, PWSTR};
    use ::windows::Win32::Foundation::CloseHandle;
    use ::windows::Win32::System::LibraryLoader::{
        GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
    };
    use ::windows::Win32::System::Threading::{
        CreateProcessW, CREATE_NEW_CONSOLE, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION,
        STARTUPINFOW,
    };
    use ::windows::Win32::UI::Shell::ShellExecuteW;
    use ::windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use anyhow::{anyhow, bail, Context, Result};

    use super::{launch_of, Launch, ToolId};
    use crate::backend::windows::{system_root, wide};

    /// Starts the tool and returns without waiting for it to finish.
    pub fn run_tool(id: ToolId) -> Result<()> {
        let sys32 = system_root().join("System32");
        match launch_of(id) {
            Launch::Console(exe, args) => spawn(&sys32.join(exe), args, CREATE_NEW_CONSOLE),
            Launch::Shell(exe, params) => shell_open(&sys32.join(exe), params),
            Launch::DnsFlush => flush_dns(),
            Launch::OneDriveUninstall => {
                let primary = sys32.join("OneDriveSetup.exe");
                let fallback = system_root().join(r"SysWOW64\OneDriveSetup.exe");
                let exe = if primary.is_file() {
                    primary
                } else if fallback.is_file() {
                    fallback
                } else {
                    bail!("OneDriveSetup.exe not found");
                };
                spawn(&exe, "/uninstall", PROCESS_CREATION_FLAGS(0))
            }
        }
    }

    /// `CreateProcessW` without redirected handles, so a console tool writes
    /// to its own new window.
    fn spawn(exe: &Path, args: &str, flags: PROCESS_CREATION_FLAGS) -> Result<()> {
        let exe_s = exe.to_string_lossy();
        let app = wide(&exe_s);
        let mut cmdline = wide(&format!("\"{exe_s}\" {args}"));
        let si = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();
        unsafe {
            CreateProcessW(
                PCWSTR(app.as_ptr()),
                Some(PWSTR(cmdline.as_mut_ptr())),
                None,
                None,
                false,
                flags,
                None,
                PCWSTR::null(),
                &si,
                &mut pi,
            )
            .with_context(|| format!("cannot start {exe_s}"))?;
            let _ = CloseHandle(pi.hThread);
            let _ = CloseHandle(pi.hProcess);
        }
        Ok(())
    }

    fn shell_open(exe: &Path, params: &str) -> Result<()> {
        let file = wide(&exe.to_string_lossy());
        let params_w = wide(params);
        let params_ptr = if params.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(params_w.as_ptr())
        };
        let h = unsafe {
            ShellExecuteW(
                None,
                w!("open"),
                PCWSTR(file.as_ptr()),
                params_ptr,
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        // Values above 32 mean success.
        if h.0 as usize <= 32 {
            bail!("cannot open {} (code {})", exe.display(), h.0 as usize);
        }
        Ok(())
    }

    /// `DnsFlushResolverCache` is exported by dnsapi.dll but not declared in
    /// the SDK headers, so it is resolved at run time from System32 only.
    fn flush_dns() -> Result<()> {
        unsafe {
            let lib = LoadLibraryExW(w!("dnsapi.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32)
                .context("cannot load dnsapi.dll")?;
            let f = GetProcAddress(lib, s!("DnsFlushResolverCache"))
                .ok_or_else(|| anyhow!("DnsFlushResolverCache not found"))?;
            let f: unsafe extern "system" fn() -> BOOL = std::mem::transmute(f);
            if !f().as_bool() {
                bail!("DnsFlushResolverCache failed");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_tools_use_fixed_arguments() {
        assert_eq!(
            launch_of(ToolId::Sfc),
            Launch::Console("sfc.exe", "/scannow")
        );
        assert_eq!(
            launch_of(ToolId::Dism),
            Launch::Console("DISM.exe", "/Online /Cleanup-Image /RestoreHealth")
        );
        assert_eq!(
            launch_of(ToolId::WinsockReset),
            Launch::Console("netsh.exe", "winsock reset")
        );
        assert_eq!(
            launch_of(ToolId::PerfCounters),
            Launch::Console("lodctr.exe", "/r")
        );
    }

    #[test]
    fn gui_tools_open_through_the_shell() {
        assert_eq!(
            launch_of(ToolId::MouseCpl),
            Launch::Shell("control.exe", "main.cpl")
        );
        assert_eq!(
            launch_of(ToolId::KeyboardCpl),
            Launch::Shell("control.exe", "keyboard")
        );
        assert_eq!(
            launch_of(ToolId::DefragGui),
            Launch::Shell("dfrgui.exe", "")
        );
        assert_eq!(
            launch_of(ToolId::PerfOptionsGui),
            Launch::Shell("SystemPropertiesPerformance.exe", "")
        );
        assert_eq!(
            launch_of(ToolId::SystemRestoreGui),
            Launch::Shell("rstrui.exe", "")
        );
        assert_eq!(launch_of(ToolId::DnsFlush), Launch::DnsFlush);
        assert_eq!(
            launch_of(ToolId::OneDriveUninstall),
            Launch::OneDriveUninstall
        );
    }

    #[test]
    fn tool_ids_serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&ToolId::OneDriveUninstall).unwrap(),
            "\"one_drive_uninstall\""
        );
        assert_eq!(
            serde_json::from_str::<ToolId>("\"dns_flush\"").unwrap(),
            ToolId::DnsFlush
        );
    }
}
