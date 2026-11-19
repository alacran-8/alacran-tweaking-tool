//! Service start type through the Service Control Manager.

use ::windows::core::PCWSTR;
use ::windows::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;
use ::windows::Win32::System::Services::{
    ChangeServiceConfigW, CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceConfigW,
    ENUM_SERVICE_TYPE, QUERY_SERVICE_CONFIGW, SC_HANDLE, SC_MANAGER_CONNECT, SERVICE_CHANGE_CONFIG,
    SERVICE_ERROR, SERVICE_NO_CHANGE, SERVICE_QUERY_CONFIG, SERVICE_START_TYPE,
};
use anyhow::{bail, Context};

use super::wide;
use crate::backend::Result;

struct Handle(SC_HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        let _ = unsafe { CloseServiceHandle(self.0) };
    }
}

/// Opens `name`; `Ok(None)` when the service does not exist.
fn open(name: &str, access: u32) -> Result<Option<(Handle, Handle)>> {
    let scm = unsafe { OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_CONNECT) }
        .context("cannot open the service control manager")?;
    let scm = Handle(scm);
    let w = wide(name);
    match unsafe { OpenServiceW(scm.0, PCWSTR(w.as_ptr()), access) } {
        Ok(h) => Ok(Some((scm, Handle(h)))),
        Err(e) if e.code() == ERROR_SERVICE_DOES_NOT_EXIST.to_hresult() => Ok(None),
        Err(e) => Err(e).with_context(|| format!("cannot open service {name}")),
    }
}

pub fn start_get(name: &str) -> Result<Option<u32>> {
    let Some((_scm, svc)) = open(name, SERVICE_QUERY_CONFIG)? else {
        return Ok(None);
    };
    let mut needed = 0u32;
    let _ = unsafe { QueryServiceConfigW(svc.0, None, 0, &mut needed) };
    if needed == 0 {
        bail!("cannot query service {name}");
    }
    // u64 storage keeps QUERY_SERVICE_CONFIGW suitably aligned.
    let mut buf = vec![0u64; (needed as usize).div_ceil(8)];
    let cfg = buf.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;
    unsafe { QueryServiceConfigW(svc.0, Some(cfg), (buf.len() * 8) as u32, &mut needed) }
        .with_context(|| format!("cannot query service {name}"))?;
    Ok(Some(unsafe { (*cfg).dwStartType.0 }))
}

pub fn start_set(name: &str, start: u32) -> Result<()> {
    if start > 4 {
        bail!("invalid service start type {start}");
    }
    let Some((_scm, svc)) = open(name, SERVICE_CHANGE_CONFIG)? else {
        bail!("service not found: {name}");
    };
    unsafe {
        ChangeServiceConfigW(
            svc.0,
            ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
            SERVICE_START_TYPE(start),
            SERVICE_ERROR(SERVICE_NO_CHANGE),
            PCWSTR::null(),
            PCWSTR::null(),
            None,
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
            PCWSTR::null(),
        )
    }
    .with_context(|| format!("cannot change start type of service {name}"))
}
