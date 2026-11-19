# Release checklist — Alacran Tweak Tool

Run from the repo root on the build host. Nothing here applies tweaks on the host.

1. Tests green
   - `cmd /c alacran-tweak-app\scripts\test.cmd`
   - `npx vitest run` in `alacran-tweak-app`
2. VM gate green for the catalog being shipped (`docs/vm-testing.md`): oneclick, all-safe,
   caution round, revert-category round. `compare before after` must report 0 differences.
3. Version bumped in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `package.json`
   and `VERSION` in `src/screens/about.ts`.
4. Build: `cmd /c alacran-tweak-app\scripts\build.cmd` → `C:\at-target\release\AlacranTweak.exe`.
5. PE metadata (PowerShell):
   `(Get-Item C:\at-target\release\AlacranTweak.exe).VersionInfo | fl CompanyName,ProductName,FileVersion,LegalCopyright`
   → CompanyName `Alacran`, ProductName `Alacran Tweak Tool`, FileVersion = release version.
6. Static hygiene scan of the exe strings (ASCII + UTF-16):
   - no `powershell`, `reg.exe`, `wscript`, `cscript`, `mshta`, `rundll32`, `schtasks`, `wmic`
     launches; allowed: the Tools page launches (`sfc`, `DISM`, `netsh winsock reset`, `lodctr`,
     `dfrgui`, `SystemPropertiesPerformance`, `control`, `rstrui`, `OneDriveSetup`) and the
     backend's `bcdedit.exe`. `cmd.exe` comes from the Rust standard library (batch-file
     argument quoting) and is never launched.
   - URLs only from `ALLOWED_LINKS` in `src-tauri/src/commands.rs` plus the WebView2 download
     page; the rest are HTML/brotli dictionary fragments inside the WebView libraries.
7. Smoke test: copy the exe to a temp folder, run `--dry-run`, click through Home, one category,
   Restore, Tools, About; the log shows only `WOULD` lines.
8. Optional signing (when a certificate exists, e.g. Azure Trusted Signing): sign before hashing,
   `signtool sign /fd SHA256 /tr http://timestamp.acs.microsoft.com /td SHA256 <signing options> C:\at-target\release\AlacranTweak.exe`
   (Trusted Signing adds `/dlib Azure.CodeSigning.Dlib.dll /dmdf metadata.json`), then
   `signtool verify /pa C:\at-target\release\AlacranTweak.exe`.
9. SHA-256: `Get-FileHash C:\at-target\release\AlacranTweak.exe -Algorithm SHA256`;
   put the hash in the release notes.
10. VirusTotal (owner): upload the exact release exe, record the result in the release notes.
    If flagged, submit a false-positive report to Microsoft (WDSI,
    https://www.microsoft.com/wdsi/filesubmission) and to each flagging vendor.
11. Publish on GitHub Releases (owner): upload the exe, paste the hash and the changelog.
