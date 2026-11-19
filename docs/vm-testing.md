# VM apply/revert gate

The real Windows backend writes system state, so it is exercised only inside
a throwaway VirtualBox guest. On the development host you only install
VirtualBox, manage the VM, copy files and build. Never run `vm_gate.exe`
`apply`/`revert-*` (or the app without `--dry-run`) on the host.

## Tools

- `alacran-tweak-app/src-tauri/src/bin/vm_gate.rs`: dev-only headless runner
  built on the real lib (`catalog`, `WinBackend`, `engine`, `store`,
  `sysinfo`). Not shipped; `AlacranTweak.exe` is unchanged. Modes:
  - `dump <out.json>`: every op of every tweak (all options), resolved per
    device / NIC / active power plan, read through the backend, written as a
    sorted map `target -> value`. One-way ops (appx, cleanup) have no state.
  - `apply oneclick | all-safe | ids:<a,b>`: `engine::apply` with the real
    store `C:\ProgramData\AlacranTweak\backups`; prints the ApplyReport.
    `all-safe` = every available `risk: safe` tweak that is not one-way.
    Exit 2 if `failed` is non-empty.
  - `revert-all`, `revert-category <cat>` ("Bu sayfayı geri al"): prints the
    RevertReport; exit 2 if `failed` is non-empty.
  - `state`: id -> state + matched/total for every available tweak.
  - `compare <a.json> <b.json>`: prints differing targets; exit 1 if any
    (appx/cleanup targets ignored).

## Test VM (throwaway)

Test credentials (exist only inside this VM): user `gate`, password in
`C:\AlacranVM\vm\gate-password.txt` on the host (generated test password, not
used anywhere else).

```sh
VB="/c/Program Files/Oracle/VirtualBox/VBoxManage.exe"; VM=AlacranGate
winget install --id Oracle.VirtualBox -e
# ISO: Turkish Windows 11 from microsoft.com/tr-tr/software-download/windows11
"$VB" createvm --name $VM --ostype Windows11_64 --basefolder 'C:\AlacranVM\vm' --register
"$VB" modifyvm $VM --cpus 4 --memory 6144 --firmware efi --tpm-type 2.0 \
  --graphicscontroller vboxsvga --vram 128 --nic1 nat --audio-enabled off \
  --clipboard-mode disabled --drag-and-drop disabled --usb-xhci on
"$VB" modifynvram $VM inituefivarstore
"$VB" modifynvram $VM enrollmssignatures
"$VB" modifynvram $VM enrollorclpk
"$VB" modifynvram $VM secureboot --enable
"$VB" createmedium disk --filename 'C:\AlacranVM\vm\AlacranGate\AlacranGate.vdi' --size 65536 --format VDI
"$VB" storagectl $VM --name SATA --add sata --controller IntelAhci --portcount 4
"$VB" storageattach $VM --storagectl SATA --port 0 --device 0 --type hdd \
  --medium 'C:\AlacranVM\vm\AlacranGate\AlacranGate.vdi'
"$VB" sharedfolder add $VM --name release --hostpath 'C:\at-target\release' --readonly
# Generic Windows 11 Pro install key (no activation; unactivated is fine).
# UAC is turned off in the guest so guestcontrol processes are elevated.
"$VB" unattended install $VM --iso='C:\AlacranVM\iso\Win11_tr-tr_x64.iso' \
  --user=gate --password="$PW" --full-user-name="Gate Test" \
  --locale=tr_TR --country=TR --time-zone=Europe/Istanbul \
  --hostname=alacran-gate.local --image-index=4 \
  --key=VK7JG-NPHTM-C97JM-9MPGT-3V66T --install-additions \
  --post-install-command='cmd /c reg add HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System /v EnableLUA /t REG_DWORD /d 0 /f'
"$VB" startvm $VM --type headless
# wait until `waitup` (below) succeeds
```

### After install (done 2026-11-19)

The unattended Guest Additions install froze on the first logon (VBoxSVGA display
driver copy), so `VBoxService` was never installed and the post-install command
(UAC off) never ran. Repaired by hand through the VM console:

1. Mount `C:\Program Files\Oracle\VirtualBox\VBoxGuestAdditions.iso` on SATA port 1.
2. In the guest: elevated cmd, `reg add HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System /v EnableLUA /t REG_DWORD /d 0 /f`,
   `start /wait D:\VBoxWindowsAdditions-amd64.exe /S`, reboot.
3. Check no tweak state (`C:\gate` and `C:\ProgramData\AlacranTweak` absent), power off,
   take the offline snapshot `clean`.

Never take or restore a snapshot while the VM runs (a live snapshot hung for hours once).
The guest keyboard layout is US, so `controlvm keyboardputstring` types correctly.

## Build

```bat
cmd /c alacran-tweak-app\scripts\build.cmd
cmd /c "alacran-tweak-app\scripts\env.cmd && cargo build --release --bin vm_gate --manifest-path src-tauri\Cargo.toml"
```

Both land in `C:\at-target\release`.

## Gate run

Helper (host, Git Bash):

```sh
export MSYS_NO_PATHCONV=1   # else Git Bash turns `/c` into a path
grun() { timeout 960 "$VB" guestcontrol $VM run --username gate --password "$PW" \
  --exe 'C:\Windows\System32\cmd.exe' --timeout 900000 -- cmd.exe /c "$@"; }
gate() { grun "C:\\gate\\vm_gate.exe $*"; }
# LoggedInUsers is never set in this guest; wait for an active console session
# instead (HKCU reads fail before auto-logon finishes).
waitup() { for i in $(seq 1 50); do grun 'query user' 2>/dev/null | grep -q Active \
  && { sleep 15; return 0; }; sleep 10; done; return 1; }
greboot() { grun "shutdown /r /t 0"; sleep 60; waitup; }
```

Per round (starting from snapshot `clean`):

```sh
"$VB" snapshot $VM restore clean && "$VB" startvm $VM --type headless   # wait until up
"$VB" guestcontrol $VM mkdir --username gate --password "$PW" 'C:\gate'
"$VB" guestcontrol $VM copyto --username gate --password "$PW" \
  'C:\at-target\release\vm_gate.exe' 'C:\at-target\release\AlacranTweak.exe' 'C:\gate\'
gate dump 'C:\gate\before.json'
gate apply <oneclick|all-safe|ids:...>
greboot
gate state                       # applied tweaks must read "applied"
gate dump 'C:\gate\applied.json'
gate compare 'C:\gate\before.json' 'C:\gate\applied.json'   # must exit 1
gate revert-all                  # or revert-category <cat>
greboot
gate dump 'C:\gate\after.json'
gate compare 'C:\gate\before.json' 'C:\gate\after.json'     # must exit 0
"$VB" controlvm $VM poweroff
```

Rounds: `oneclick`; `all-safe`; caution round `ids:` with `perf_bcd_*`,
`net_irq_affinity`, `cpu_process_mitigations_off`, `pow_hibernate_off`,
`prv_background_apps_off`; a `revert-category` round. UI check (guestcontrol
processes have no desktop, so use the console): Win+R, `keyboardputstring
'C:\gate\AlacranTweak.exe --dry-run'`, Enter, then
`"$VB" controlvm $VM screenshotpng <png>`.

## Results

Run 2026-11-19, guest Windows 11 Pro 26H2 build 26300 (Turkish), 4 vCPU, 6 GB,
detected as laptop (VirtualBox ACPI battery). Exit code of `guestcontrol run` is
32 + the guest exit code (33 = 1, 34 = 2).

| Round | Apply | State after reboot | Revert | before vs after |
|---|---|---|---|---|
| oneclick (21 on a laptop) | 20 ok, 2 one-way cleanups, 1 failed: `new_widgets_off` (access denied) | 18 applied; `sto_last_access_off` read not applied (value reset at boot) | revert-all ok | **0 differences** |
| all-safe (45) after fixes | 44 ok, 1 failed: `new_widgets_off` | all applied except NIC tweaks with no matching property on the VM NIC (unknown, 0 targets) and `new_ultimate_power` (no readable state) | revert-all ok | **0 differences** |
| caution: `perf_bcd_*` (4), `cpu_process_mitigations_off`, `net_irq_affinity`, `pow_hibernate_off`, `prv_background_apps_off` | 8 ok, 0 failed (Turkish bcdedit output parsed fine) | all applied | revert-all ok (15 entries) | **0 differences** |
| revert-category `visual` (4 visual + `game_mode_on` as control) | 5 ok | all applied | 9 entries reverted | only `GameBar\AutoGameModeEnabled` differs (the control, expected) |
| UI `--dry-run` | renders under the CSP with the tubes background | | | |

Fixes from the gate:
- `sto_last_access_off`: a plain `1` was reset to `0x80000002` (system-managed) at boot;
  now writes `0x80000001`, as `fsutil behavior set disablelastaccess 1` does. Re-verified in all-safe.
- `new_widgets_off` removed: UCPD denies writes to `Policies\Microsoft\Dsh\AllowNewsAndInterests`
  (other values under `Dsh` can be written; checked with `reg add` in the guest, `ucpd` driver running).

VM notes: the guest froze twice (once at first boot before any tweak, once on the
"Yeniden başlatılıyor" screen after the caution revert). After a hard power-off it booted
normally, the after-dump matched `before` exactly, and the next reboot was clean, so this
is treated as VirtualBox instability, not a tweak problem.
