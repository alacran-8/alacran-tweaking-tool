@echo off
setlocal
call "%~dp0env.cmd" || exit /b 1
cargo test --manifest-path src-tauri\Cargo.toml %*
