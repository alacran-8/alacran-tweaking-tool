use std::os::windows::ffi::OsStrExt;

use webview2_com::Microsoft::Web::WebView2::Win32::GetAvailableCoreWebView2BrowserVersionString;
use windows::core::{w, PCWSTR, PWSTR};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, IDYES, MB_ICONWARNING, MB_YESNO, SW_SHOWNORMAL,
};

pub mod backend;
pub mod catalog;
pub mod commands;
pub mod engine;
pub mod restore_point;
pub mod settings;
pub mod store;
pub mod sysinfo;
pub mod tools;

fn wide(s: &str) -> Vec<u16> {
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// True when an installed WebView2 runtime is found.
fn webview2_present() -> bool {
    let mut version = PWSTR::null();
    let ok = unsafe { GetAvailableCoreWebView2BrowserVersionString(PCWSTR::null(), &mut version) };
    let present = ok.is_ok() && !version.is_null() && unsafe { *version.0 } != 0;
    // The runtime allocates the string with CoTaskMemAlloc.
    unsafe { windows::Win32::System::Com::CoTaskMemFree(Some(version.0 as *const _)) };
    present
}

fn ensure_webview2() {
    if webview2_present() {
        return;
    }
    let text = wide("Bu program için Microsoft WebView2 gerekli. İndirme sayfası açılsın mı?");
    let answer = unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("Alacran Tweak Tool"),
            MB_YESNO | MB_ICONWARNING,
        )
    };
    if answer == IDYES {
        unsafe {
            ShellExecuteW(
                None,
                w!("open"),
                w!("https://developer.microsoft.com/microsoft-edge/webview2/"),
                None,
                None,
                SW_SHOWNORMAL,
            );
        }
    }
    std::process::exit(1);
}

pub fn run() {
    let dry_run = std::env::args().any(|a| a == "--dry-run");
    ensure_webview2();
    tauri::Builder::default()
        .manage(commands::AppState::new(dry_run))
        .invoke_handler(tauri::generate_handler![
            commands::is_dry_run,
            commands::get_system_info,
            commands::get_catalog,
            commands::apply_tweaks,
            commands::apply_tweaks_force,
            commands::enable_protection,
            commands::list_sessions,
            commands::revert_session,
            commands::revert_all,
            commands::revert_category,
            commands::run_tool,
            commands::open_link,
            commands::get_settings,
            commands::set_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Alacran Tweak Tool");
}
