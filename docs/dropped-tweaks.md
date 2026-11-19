# Dropped tweaks

Values from `legacy/fixed/Alacran.Tweak.Tool.bat` that were **not** ported to the app catalog, with the reason.
"Batch line" is the line number in `legacy/fixed/Alacran.Tweak.Tool.bat`. Lines already disabled there with
`REM [FIX]` (see `legacy/fixed/DEGISIKLIKLER.md`) are not ported and not repeated here.
"NIC key" means the per-adapter key under
`HKLM\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002BE10318}\<nnnn>`.

## Totals

Catalog (all files in `alacran-tweak-app/catalog/`, including `new.json` and `bcd.json`). "ops" counts the ops of
a tweak plus the ops of all its options.

| category | tweaks | ops | one-click | caution | one-way |
|---|---|---|---|---|---|
| network | 10 | 28 | 2 | 5 | 0 |
| power | 4 | 5 | 1 | 1 | 0 |
| cpu | 7 | 8 | 1 | 3 | 0 |
| gpu | 3 | 3 | 1 | 2 | 0 |
| game | 8 | 14 | 3 | 3 | 0 |
| input | 5 | 15 | 2 | 2 | 0 |
| visual | 4 | 9 | 3 | 1 | 0 |
| performance | 11 | 23 | 3 | 5 | 0 |
| storage | 3 | 3 | 1 | 1 | 0 |
| usb | 2 | 2 | 1 | 0 | 0 |
| services | 5 | 12 | 1 | 3 | 0 |
| privacy | 21 | 103 | 3 | 10 | 8 |
| cleanup | 5 | 5 | 2 | 1 | 5 |
| **total** | **88** | **230** | **24** | **37** | **13** |

This document: every table row below, and the rows that drop something (rows whose reason starts with
"Not dropped" say where a value went). One row often groups many batch lines and values, so the number of
dropped values is much higher than the number of dropped rows.

| section | rows | dropped rows |
|---|---|---|
| network | 85 | 85 |
| power | 41 | 40 |
| cpu | 28 | 25 |
| gpu | 41 | 38 |
| game | 61 | 55 |
| input | 23 | 19 |
| visual | 22 | 14 |
| performance | 50 | 45 |
| storage | 13 | 10 |
| usb | 7 | 6 |
| services | 36 | 30 |
| privacy | 90 | 58 |
| cleanup | 14 | 11 |
| bcd | 24 | 19 |
| **total** | **535** | **455** |

## network

Sources: `NetworkPingTweaks` (653–799), `AutoOpti1` network items (5047, 5131, 5361–5366, 5379, 5425–5430),
`WifiEnabler` (4744–4750). `ApplyAll1` (379–597) contains no network values.

| batch line | key/value | reason |
|---|---|---|
| 655, 681, 694 | `Disable-NetAdapterPowerManagement`: "Allow the computer to turn off this device" (`PnPCapabilities`=0x18 DWORD on the NIC key) | Needs a DWORD on the NIC key; the catalog's `nic_property` op writes REG_SZ only. The adapter-side power features of this cmdlet are kept in `net_nic_power_saving_off` / `net_wake_off`. |
| 655, 681, 694 | `Disable-NetAdapterPowerManagement`: `*DeviceSleepOnDisconnect`, `*PMWiFiRekeyOffload` | Only act while the cable is unplugged / the PC sleeps; no effect while connected and playing. |
| 659 | `netsh winsock reset` | Repair action, not a tweak; offered as the "Reset Winsock" tool. |
| 661 | `netsh int ip set global taskoffload=enabled` | Windows default (task offload is on). |
| 662 | `netsh int ip set global neighborcachelimit=4096` | No registry value the app can write (stored by NSI); the default cache is far larger than a home LAN needs; no latency effect. |
| 663 | `netsh int tcp set global timestamps=disabled` | Windows 10/11 default (timestamps already off). |
| 664 | `netsh int tcp set heuristics disabled` | Windows 10/11 default (window scaling heuristics already off). |
| 665 | `netsh int tcp set global autotuninglevel=normal` | Windows default. |
| 667 | `netsh int tcp set supplemental Internet congestionprovider=CTCP` | Stored only in NSI TCP templates (no plain registry value); Windows 10 1709+ uses CUBIC, which is not worse than CTCP on home links. |
| 669 | `netsh int tcp set global ecncapability=disabled` | Stored only in NSI templates (no plain registry value); no established latency benefit. |
| 670 | `netsh int tcp set global rss=enabled` | Windows default (RSS on). |
| 671 | `netsh int tcp set global rsc=disabled` | Global switch lives in NSI; covered per adapter by `net_lso_rsc_off` (`*RscIPv4`/`*RscIPv6`). |
| 674 | `netsh int tcp set global nonsackrttresiliency=disabled` | Windows default. |
| 675 | `netsh int tcp set security mpp=disabled` | Windows default; no registry equivalent. |
| 676 | `netsh int tcp set security profiles=disabled` | No registry equivalent; no performance effect. |
| 677 | `Disable-NetAdapterChecksumOffload` | Counterproductive: moves checksum work from the adapter to the CPU with no latency gain. |
| 680 | `Disable-NetAdapterIPsecOffload` | Only used with IPsec (VPN/enterprise); no effect for games. |
| 682 | `Disable-NetAdapterQos` | NDIS QoS/DCB is a data-center feature; no effect on home networks. |
| 683 | `Tcpip\Parameters` `EnableICMPRedirect`=1 | Windows default (1). |
| 684 | `Tcpip\Parameters` `EnablePMTUDiscovery`=1 | Windows default (1). |
| 685 | `Tcpip\Parameters` `Tcp1323Opts`=0 | Reduces throughput: turns off TCP window scaling (caps the receive window at 64 KB). |
| 686 | `Tcpip\Parameters` `TcpMaxDupAcks`=2 | Windows default (2). |
| 757 | `Tcpip\Parameters` `TcpTimedWaitDelay`=32 | Duplicate of line 687 (kept as `net_time_wait`). |
| 690, 760 | `Tcpip\Parameters` `MaxConnectionsPerServer`=0 | WinINet setting; not read from the Tcpip key (placebo). |
| 691, 761 | `Tcpip\Parameters` `MaxUserPort`=65534 | Ignored since Windows Vista (dynamic port range is set with `netsh int ipv4 set dynamicport`); default range is ample. |
| 693, 763 | `Tcpip\Parameters` `DefaultTTL`=64 | Only changes the hop limit of outgoing packets; no effect on ping or speed. |
| 696 | NIC key `AutoPowerSaveModeEnabled`="0" | Vendor keyword whose meaning cannot be verified. |
| 699 | NIC key `DisableDelayedPowerUp`="2" | Vendor keyword; value meaning cannot be verified. |
| 701 | NIC key `EEE`="0" | Old non-standard name; drivers read `*EEE` (kept in `net_nic_power_saving_off`). |
| 702 | NIC key `EnablePME`="0" | Wake (PME) setting; only acts in sleep; standard wake keywords kept in `net_wake_off`. |
| 703 | NIC key `EEELinkAdvertisement`="0" | Intel-specific EEE advertisement; covered by the standard `*EEE` keyword. |
| 705 | NIC key `EnableSavePowerNow`="0" | Vendor keyword whose meaning cannot be verified. |
| 706 | NIC key `EnablePowerManagement`="0" | Generic name used differently by vendors; meaning cannot be verified. |
| 707 | NIC key `EnableDynamicPowerGating`="0" | Intel idle/link-down power gating; no verified effect while connected. |
| 708 | NIC key `EnableConnectedPowerGating`="0" | Intel connected-standby power gating; no verified effect while playing. |
| 709 | NIC key `EnableWakeOnLan`="0" | Vendor wake setting; standard wake keywords kept in `net_wake_off`. |
| 711, 724 | NIC key `NicAutoPowerSaver`="2" | Vendor keyword; value meaning cannot be verified. |
| 712, 725 | NIC key `PowerDownPll`="0" | Acts only when the link is down / PC sleeps. |
| 714, 727 | NIC key `ReduceSpeedOnPowerDown`="0" | Acts only on sleep/shutdown. |
| 715, 728 | NIC key `S5NicKeepOverrideMacAddrV2`="0" | Shutdown-state MAC handling; no effect while running. |
| 716, 729 | NIC key `S5WakeOnLan`="0" | Wake from shutdown only; standard wake keywords kept in `net_wake_off`. |
| 717, 730 | NIC key `ULPMode`="0" | Intel Ultra Low Power acts only while the cable is unplugged. |
| 718, 731 | NIC key `WakeOnDisconnect`="0" | Wake setting; acts only in sleep. |
| 734 | NIC key `WakeOnLink`="0" | Wake setting; acts only in sleep. |
| 735 | NIC key `WolShutdownLinkSpeed`="2" | Acts only after shutdown. |
| 721, 722, 723, 726 | NIC key `AutoDisableGigabit`, `AdvancedEEE`, `GigaLite`, `PowerSavingMode` | Duplicates of lines 697, 698, 710, 713 (kept as `net_realtek_power_off`). |
| 732, 733 | NIC key `*WakeOnMagicPacket`, `*WakeOnPattern` | Duplicates of lines 719, 720 (kept as `net_wake_off`). |
| 736 | NIC key `JumboPacket`="1514" | Wrong name (standard is `*JumboPacket`, so the batch line did nothing) and 1514 is already the default. |
| 737 | NIC key `TransmitBuffers`="4096" | Wrong name (standard is `*TransmitBuffers`, batch line did nothing); one fixed value is out of range for some drivers. |
| 738 | NIC key `ReceiveBuffers`="512" | Wrong name (standard is `*ReceiveBuffers`, batch line did nothing); one fixed value does not fit every driver's range. |
| 739, 745, 746, 747, 748 | NIC key `IPChecksumOffloadIPv4`, `TCPChecksumOffloadIPv4/IPv6`, `UDPChecksumOffloadIPv4/IPv6`="0" | Wrong names (no `*`, batch lines did nothing) and counterproductive (see line 677). |
| 740 | NIC key `LsoV1IPv4`="0" | Wrong name (no `*`); LSOv1 is legacy NDIS 5; LSOv2 is covered by `net_lso_rsc_off`. |
| 741, 742 | NIC key `LsoV2IPv4`, `LsoV2IPv6`="0" | Wrong names (no `*`); covered by `net_lso_rsc_off` with `*LsoV2IPv4`/`*LsoV2IPv6`. |
| 743, 744 | NIC key `PMARPOffload`, `PMNSOffload`="0" | Wrong names (no `*`); covered by `net_wake_off` with `*PMARPOffload`/`*PMNSOffload`. |
| 749 | NIC key `RSS`="1" | Wrong name (no `*`); RSS is on by default. |
| 750 | NIC key `*NumRssQueues`="2" | Forcing 2 queues can lower the driver's default (often 4+) and reduce throughput; no latency benefit. |
| 751 | NIC key `RSSProfile`="3" | Wrong name (no `*`); batch line did nothing. |
| 753, 754 | `Control\Nsi\{eb004a03-…}\0` `0200`, `1700` REG_BINARY | Undocumented NSI TCP template blobs; layout varies by build, writing them can corrupt TCP settings. |
| 755 | `Tcpip\Parameters\Winsock` `MinSockAddrLength`=16 | Windows default. |
| 764 | `Tcpip\Parameters` `DelayedAckFrequency`=1 | Not a value Windows reads (real names are `TcpAckFrequency`/`TcpDelAckTicks`). |
| 765 | `Tcpip\Parameters` `DelayedAckTicks`=1 | Not a value Windows reads. |
| 766 | `Tcpip\Parameters` `CongestionAlgorithm`=1 | Not a value Windows reads (congestion provider is set through NSI templates). |
| 767 | `Tcpip\Parameters` `MultihopSets`=15 | Windows NT era value; not read by the current TCP/IP stack. |
| 768, 769 | `Tcpip\Parameters` `FastCopyReceiveThreshold`, `FastSendDatagramThreshold`=16384 | AFD (Winsock driver) values written under the Tcpip key, so never read; placebo. |
| 770 | `Tcpip\Parameters\Interfaces\{GUID}` `InterfaceMetric`=55 | Harmful: gives every adapter the same route metric, so Windows may prefer Wi-Fi/VPN over Ethernet. |
| 771 | `Tcpip\Parameters\Interfaces\{GUID}` `TCPNoDelay`=1 | Not read by the Vista+ TCP/IP stack; Nagle is controlled per socket by the game itself. |
| 772 | `Tcpip\Parameters\Interfaces\{GUID}` `TcpAckFrequency`=1 | Read by Windows, but the catalog has no per-interface (`Interfaces\{GUID}`) op; benefit limited to TCP games (most competitive games use UDP). Candidate for a future op. |
| 773 | `Tcpip\Parameters\Interfaces\{GUID}` `TcpDelAckTicks`=0 | Legacy (Windows Server 2003) value; irrelevant once delayed ACK is off; no per-interface op. |
| 774, 777, 780 | `Tcpip\Parameters\Interfaces` `TCPDelAckTicks`, `TCPNoDelay`, `TcpAckFrequency` | Written to the `Interfaces` parent key, which Windows does not read; placebo. |
| 775, 776, 779 | `Tcpip\Parameters` `TCPDelAckTicks`, `TCPNoDelay`, `TcpAckFrequency` | Global location is not read on the Vista+ stack (documented only per interface); placebo. |
| 778 | `MSMQ\Parameters` `TCPNoDelay`=1 | Only for Microsoft Message Queuing, not installed by default; no effect on games. |
| 782–785 | `Tcpip\ServiceProvider` `LocalPriority`=4, `HostsPriority`=5, `DnsPriority`=6, `NetbtPriority`=7 | Only the relative order matters and it is unchanged from the defaults (499/500/2000/2001); placebo. |
| 786 | `Tcpip\Parameters\Interfaces` `DisableTaskOffload`=0 | Written to the `Interfaces` parent key (not read). |
| 787 | `Tcpip\Parameters` `DisableTaskOffload`=0 | Windows default (offload on). |
| 789, 5361, 5425 | `LanmanServer\Parameters` `autodisconnect`=0xFFFFFFFF | SMB file-server setting; only matters when this PC shares folders; no effect on games or ping. |
| 790, 5362, 5426 | `LanmanServer\Parameters` `Size`=3 | SMB file-server setting; no effect on games or ping. |
| 791, 5363, 5427 | `LanmanServer\Parameters` `EnableOplocks`=0 | SMB file-server setting (SMB1 only); turning oplocks off slows file sharing. |
| 792, 5364, 5428 | `LanmanServer\Parameters` `IRPStackSize`=20 / 32 | SMB file-server setting, only fixes "not enough server storage" errors; the batch also writes two different values. |
| 793, 5365, 5429 | `LanmanServer\Parameters` `SharingViolationDelay`=0 | SMB file-sharing retry setting; no effect on games or ping. |
| 794, 5366, 5430 | `LanmanServer\Parameters` `SharingViolationRetries`=0 | SMB file-sharing retry setting; no effect on games or ping. |
| 798 | NIC `Enum\…\Interrupt Management\MessageSignaledInterruptProperties` `MessageNumberLimit`=256 | Only matters when MSI is enabled and the driver asks for that many vectors; consumer NIC drivers set their own MSI-X count; 256 exceeds what they support. |
| 5047 | `Services\NDIS\Parameters` `ThreadPriority`=31 | Not a documented NDIS value and no evidence `ndis.sys` reads it (same value also at line 2126, `twkusb`). |
| 5131 | `Policies\Microsoft\Windows\WcmSvc\GroupPolicy` `fDisablePowerManagement`=1 | Real ADMX policy ("Disable power management in connected standby mode"), but it only acts while a Modern Standby PC is asleep (connected standby); no effect while the PC is awake and playing. |
| 5379 | `Tcpip\Parameters\Interfaces` `DisableTaskOffload`=1 | Written to the `Interfaces` parent key (not read); contradicts line 787. |
| 4746, 4748, 4750 | `sc start Wcmsvc` / `WlanSvc` / `NativeWifiP` | The catalog has no "start service now" op; `net_wifi_services_restore` sets the start types and the services start on the next boot. |

## power

Sources: `insanepowertw` (862–1184), `AutoOpti1` power items (4977, 5016, 5106–5149, 5185, 5595–5596).
`ApplyAll1` (379–597) contains no power or CPU values (its USB device power values 381–385 are curated
with the usb category). "PS" means `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerSettings`;
"Power key" means `HKLM\SYSTEM\CurrentControlSet\Control\Power`. Processor power-plan values are listed
under `## cpu`.

| batch line | key/value | reason |
|---|---|---|
| 867, 4977 | `powercfg -h off` | No powercfg op; expressed as `HibernateEnabled`=0, the same switch the batch also writes (kept as `pow_hibernate_off`). |
| 937, 1143, 1164, 5185 | Power key `HibernateEnabled`=0 | Duplicates; kept once as `pow_hibernate_off`. |
| 1142, 1148 | Power key `HiberFileSizePercent`=0, `HiberFileType`=0 | Irrelevant once hibernation is off (no hiberfil.sys); a size of 0 % is not a valid setting. |
| 1158 | `Session Manager\Power` `HiberbootEnabled`=0 | Not dropped: merged into `pow_hibernate_off` (Fast Startup needs hibernation). |
| 973, 2632, 2706 | `powercfg SUB_SLEEP HYBRIDSLEEP 0` | Hybrid sleep needs hibernation; covered by `pow_hibernate_off`. |
| 971, 2628, 2702, 1026, 1028 | `powercfg SUB_SLEEP AWAYMODE 0` (also by GUID `25dfa149-…` on scheme `95533644-…`) | Windows default (Away Mode off); lines 1026/1028 also target a scheme GUID that is not a built-in plan. |
| 1137 | Power key `AwayModeEnabled`=0 | Not a value Windows reads (Away Mode is a power-plan setting); placebo. |
| 972, 2630, 2704 | `powercfg SUB_SLEEP ALLOWSTANDBY 0` | Disables sleep completely; a behaviour change with no performance effect (the sleep timeout is already set in Settings). |
| 877–884, 1132, 1160–1163, 5106 | `CoalescingTimerInterval`=0 under `Session Manager`, `…\Power`, `…\Memory Management`, `…\kernel`, `…\Executive`, Power key, `Power\ModernSleep`, `Control` | No evidence any Windows component reads this value under any of these keys; timer coalescing is requested per timer by apps (`SetCoalescableTimer`). |
| 907–909, 928–931, 948, 1166–1169, 5016, 5117–5119 | `Power\EnergyEstimation\TaggedEnergy` `DisableTaggedEnergyLogging`=1, `TelemetryMaxApplication`=0, `TelemetryMaxTagPerApplication`=0, `EnergyEstimationEnabled`=0 | Only feeds the per-app battery usage page; no measurable performance effect. |
| 1114, 1134, 1141, 1150 | Power key `EnergyEstimationDisabled`=1, `EnergyEstimationEnabled`=0 and =1 | Contradictory (the batch writes 0 then 1); battery-usage accounting only. |
| 1135, 1147, 2617, 2691 | Power key `EventProcessorEnabled`=0 (and =1 at 1147) | Undocumented; the batch writes both 0 and 1; no verifiable effect. |
| 1133 | Power key `PlatformAoAcOverride`=0 | Disables Modern Standby (forces S3 where firmware allows); not a performance setting and can leave laptops without working sleep. |
| 1136, 1139 | Power key `CsEnabled`=0 | Connected Standby switch from Windows 8 / early Windows 10; ignored on 2004+ builds. |
| 1140, 1144–1146, 1165 | Power key `CustomizeDuringSetup`=1, `MfBufferingThreshold`=0, `PerfCalculateActualUtilization`=1, `TimerRebaseThresholdOnDripsExit`=30, `SleepReliabilityDetailedDiagnostics`=0 | Undocumented or setup/diagnostic values; no verifiable effect while running. |
| 1119, 1155 | Power key `UserBatteryDischargeEstimator`=1 | Battery time estimate only; no performance effect. |
| 950, 953–957, 1113, 1115–1118, 1138, 1149, 1151–1154, 5122–5129 | Power key `LowLatencyScalingPercentage`, `HighPerformance`, `HighestPerformance`, `MinimumThrottlePercent`, `MaximumThrottlePercent`, `MaximumPerformancePercent`, `Class1InitialUnparkCount`, `Class2InitialUnparkCount`, `InitialUnparkCount`, `PerfBoostAtGuaranteed`, `MfOverridesDisabled`, `PpmMfOverridesDisabled`, `PpmMfBufferingThreshold` | Not values Windows reads from the Power key; processor behaviour comes from the power plan (kept tweaks are in `## cpu`). |
| 1120, 1156 | Power key `PowerThrottlingOff`=1 | Wrong key (the real value is under `Power\PowerThrottling`). |
| 871, 958, 1121, 1157, 5130 | `Power\PowerThrottling` `PowerThrottlingOff`=1 | Duplicates; kept once as `cpu_power_throttling_off`. |
| 5595, 5596 | Interactive choice: "low-end laptop that overheats" sets `PowerThrottlingOff`=1, "desktop" sets 0 | The choice was reversed (it disabled throttling on the overheating laptop); replaced by `cpu_power_throttling_off`, which is left out of one-click on laptops. |
| 1170–1181 | `Power\EnergyEstimation\Storage\SD` / `SSD\IdleState\N` `IdleExitEnergyMicroJoules`, `IdleExitLatencyMs`, `IdlePowerMw`, `IdleTimeLengthMs` | Energy-estimation model inputs, not drive power controls; changing them only skews battery estimates. |
| 916–918 | Every service with `EnableHIPM`: `EnableHIPM`=0, `EnableDIPM`=0 | Per-driver values are undocumented for current storahci; replaced by the power-plan setting "AHCI Link Power Management - HIPM/DIPM" (`pow_sata_link_power_off`). |
| 942 | Every StorPort device in `Enum`: `EnableIdlePowerManagement`=0 | The catalog has no per-device op for storage devices; StorPort idle power is device- and driver-specific. |
| 952 | `PS\2a737441-…\d4e98f31-…\DefaultPowerSchemeValues\381b4222-…` `ACSettingIndex`=0 | Edits only the Balanced plan's factory default, not the active value (the batch line changed nothing until plan defaults were restored); kept as `pow_usb3_link_power_off` on the active plan. |
| 890, 951 | `PS\54533251-…\943c8cb6-…` `Attributes`=2 | Unhides "core parking over-utilization threshold", which no kept tweak uses. |
| 891–900, 5107–5110 | `PS\54533251-…\75b0ae3f-…` `Attributes`=2 and `Affinity`, `Background Only`, `Clock Rate`, `GPU Priority`, `Priority`, `Scheduling Category`, `SFIO Priority`, `BackgroundPriority`, `Latency Sensitive` | MMCSS task values written into a power-setting key; nothing reads them there; placebo. |
| 1002 | `powercfg sub_none DEVICEIDLE 0` | Windows default on AC (device idle policy "Performance"). |
| 5132, 5133 | `Power\PDC\Activators\Default\VetoPolicy` `EA:EnergySaverEngaged`=0, `…\28\VetoPolicy` `EA:PowerStateDischarging`=0 | Undocumented Modern Standby / Energy Saver internals; no verifiable effect. |
| 959, 960, 5134–5147 | `Power\Policy\Settings\Misc` `DeviceIdlePolicy`; `Power\Policy\Settings\Processor` `CPHEADROOM`, `CPCONCURRENCY`, `PerfEnergyPreference`, `CPMinCores`, `CPMaxCores`, `CPMinCores1`, `CPMaxCores1`, `CpLatencyHintUnpark(1)`, `CPDistribution(1)`, `MaxPerformance(1)` | Not where the active plan's values live (Windows reads them from the power schemes). `CPMaxCores`=0 would also contradict the core-parking tweak. |
| 5148, 5149 | `HKCU\Control Panel\PowerCfg\GlobalPowerPolicy` `Policies` REG_BINARY | Pre-Vista power policy blob; not used by the current power manager. |
| 1122 | NVIDIA display key `PerfLevelSrc`=2222 | GPU driver value; duplicate of `nvdiagputw` line 3038 (curated with the gpu category). |
| 1123–1125 | NVIDIA display key `PowerMizerEnable`, `PowerMizerLevel`, `PowerMizerLevelAC`=0 | GPU driver values outside power/CPU scope; undocumented, effect on current drivers unverified (left to the gpu category). |
| 1159 | `Services\GpuEnergyDrv` `Start`=4 | GPU service; duplicate of `nvdiagputw` line 2807 (curated with the gpu category). |
| 1030–1047 | MMCSS `Tasks\Low Latency`, `Tasks\Games` values | Not power/CPU; duplicates of `tweaksprio` lines 1825–1851 (curated with the game category). |
| 1061–1065 | MMCSS `Tasks\Window Manager` values | Not power/CPU; duplicates of `tweaksprio` lines 1853–1858. |
| 1048, 1049, 1059, 1060 | IFEO `svchost.exe` / `csrss.exe` `PerfOptions` `CpuPriorityClass`, `IoPriority` | Not power/CPU; duplicates of `tweaksprio` lines 1870–1871, 1887–1888. |
| 1050, 1051 | `PriorityControl` `Win32PrioritySeparation`=20 | Not power/CPU; duplicate of `tweaksprio` line 1899. |
| 1054 | `Multimedia\SystemProfile` `SystemResponsiveness`=0 | Not power/CPU; duplicate of `tweaksprio` line 1911. |
| 1055–1058 | `PriorityControl` `IRQ8Priority`=1, `IRQ16Priority`=2 | Not power/CPU; duplicates of `tweaksprio` lines 1877–1878. |
| 1052, 1053 | `Services\DXGKrnl\Parameters`, `Services\nvlddmkm\Parameters` `ThreadPriority`=15 | Not power/CPU; duplicates of `tweaksprio` lines 1905–1906. |
| 1066–1107 | `HKLM\SYSTEM\ResourcePolicyStore\ResourceSets\Policies\…` `CapPercentage`, `SchedulingType`, `IsLowPriority`, `BasePriority`=82, `OverTargetPriority`=50, `IOBandwidth`, `CommitLimit`, `CommitTarget` | Undocumented resource-manager policies; giving every importance level the same priority removes the foreground/background distinction instead of improving it. |

## cpu

Sources: `cpu1` (AMD, 2586–2657), `cpu2` (Intel, 2658–2730), processor items of `insanepowertw` (961–1029,
1108–1131) and `AutoOpti1` (5120–5121, 5332–5334, 5356, 5458). `cpu1` and `cpu2` are the same list except
for TSX and two bcdedit lines, so their duplicates are merged. powercfg aliases are given as written in the
batch; the catalog uses the GUIDs.

| batch line | key/value | reason |
|---|---|---|
| 974, 1019, 2634, 2708 | `powercfg sub_processor PROCTHROTTLEMIN 100` (AC) | Duplicates; kept once as `cpu_min_state_100`. The batch labels it "Disabling C-States", which it does not do. |
| 1023 | `powercfg -setdcvalueindex … PROCTHROTTLEMIN 50` | Battery only: forcing a 50 % minimum on DC drains the battery with no gaming benefit unplugged. |
| 1017, 1021 | `powercfg PROCTHROTTLEMAX 100` (AC and DC) | Windows default (100 %). |
| 1015, 2607, 2677 | `powercfg sub_processor CPMINCORES 100` | Duplicates; kept once as `cpu_core_parking_off`. |
| 1110–1112, 2600–2605, 2644, 2670–2675, 2718 | `PS\54533251-…\0cc5b647-…` (`ControlSet001`, `ControlSet002`, `CurrentControlSet`) `ValueMax`=0, `ValueMin`=0 | Edits the allowed range of the core-parking setting, not its value; `ValueMax`=0 even contradicts `CPMINCORES 100`; `ControlSet002` may not exist. |
| 1108, 1109 | `PS\54533251-…\893dee8e-…` (and its `DefaultPowerSchemeValues\8c5e7fda-…`) `ValueMax`=100 | Range metadata of the minimum-state setting; 100 is already the maximum. |
| 2606, 2676 | Power key `CoreParkingDisabled`=0 | Not a value Windows reads; 0 would mean "not disabled" anyway. |
| 1008 | `powercfg sub_processor PERFEPP 0` | Not dropped: kept as `cpu_epp_performance`. |
| 1010, 2643, 2717 | `powercfg sub_processor THROTTLING 0` | Duplicates; kept once as `cpu_throttle_states_off`. |
| 1012 | `powercfg sub_processor PERFBOOSTMODE 1` | 1 = "Enabled" is not more aggressive than the usual default (2 = "Aggressive"). |
| 1013 | `powercfg sub_processor PERFBOOSTPOL 100` | Only used on CPUs without hardware P-state control (pre-2015 Intel); modern CPUs are covered by EPP (`cpu_epp_performance`). |
| 1126 | `powercfg sub_processor PERFAUTONOMOUS 1` | Windows default (autonomous mode on where supported). |
| 1128, 1130 | `powercfg PERFAUTONOMOUSWINDOW 20000`, `PERFCHECK 20` | No verified benefit; with autonomous (HWP/CPPC) mode on, the CPU, not Windows, picks frequencies, so these intervals have little effect. |
| 967–969 | `powercfg IDLEPROMOTE 98`, `IDLEDEMOTE 98`, `IDLECHECK 20000` | Fine-grained idle-state thresholds; effect unverified, no measured gaming benefit. |
| 1004, 2638, 2713 | `powercfg sub_processor IDLESCALING 1` | Only scales idle thresholds by performance state; no measurable effect (the batch labels it "Use higher P-states on lower C-states", which it is not). |
| 2620, 2621, 2694, 2695 | `powercfg SUB_PROCESSOR SYSCOOLPOL 1` (AC and DC) | AC default is already "Active"; the DC change only affects laptops on battery (more fan, more drain); most fans are controlled by firmware, not by this setting. |
| 1006, 1007 | `Services\IntelPPM`, `Services\AmdPPM` `Start`=3 | Windows default (3). |
| 961–966, 976–1001 | `HKLM\SYSTEM\ControlSet001\Control\Processor` `ProccesorThrottlingEnabled`, `CpuIdleThreshold`, `CpuIdle`, `CpuLatencyTimer`, `CpuDebuggingEnabled`, `ProccesorLatencyThrottlingEnabled`, `CpuIdleScrub*` (26 values) | Invented names (two are misspelled); nothing in Windows reads them; placebo. |
| 5120, 5121 | `Control\Processor` `Cstates`=0, `Capabilities`=516198 | Undocumented; not a verifiable C-state switch, and overwriting `Capabilities` can confuse the processor driver. |
| 1025, 2625, 2699 | `bcdedit /set {current} numproc %NUMBER_OF_PROCESSORS%` | Placebo: Windows already uses all logical processors; `numproc` can only limit them. |
| 2681 | `bcdedit /set allowedinmemorysettings 0x0` | Boot-loader / VSM plumbing with no performance effect. |
| 2682 | `bcdedit /set isolatedcontext No` | Duplicate of `bcd` line 4781 (curated with the BCD tweaks). |
| 2611, 2685 | `Session Manager\kernel` `DistributeTimers`=1 | Undocumented; the batch labels it "Disabling Distribute Timers" while setting 1; no verifiable effect. |
| 2614 | `Session Manager\kernel` `DisableTsx`=1 (AMD branch) | AMD CPUs have no TSX, so no effect (the batch labels it "Enabling TSX"). |
| 2688, 5356 | `Session Manager\kernel` `DisableTsx`=0 | Re-enables TSX on Intel CPUs that still have it, exposing the TAA vulnerability, for no gaming benefit (most recent Intel CPUs have TSX turned off in microcode anyway). |
| 5333, 5334 | `Memory Management` `FeatureSettingsOverride`=3, `FeatureSettingsOverrideMask`=3 | Not dropped: kept as `cpu_spectre_meltdown_off` (caution). |
| 5332 | `Memory Management` `FeatureSettings`=1 | Not a documented value (Microsoft documents only the Override/OverrideMask pair). |
| 5458 | `Set-ProcessMitigation -System -Disable <all>` | Not dropped: expressed as kernel `MitigationOptions` (`cpu_process_mitigations_off`, caution). |

## gpu

Sources: `nvdiagputw` (2777–3064), `amdgputw` (3065–3545), `Intelgputw` (3546–3666), `directx` (2488–2541),
and the GPU items of `AutoOpti1` (4910, 4915). "Display class key" means
`HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}`; the batch writes to its
`\0000` subkey, which is only the first display adapter Windows enumerated. On PCs with an integrated plus a
dedicated GPU, or after a driver reinstall, `\0000` is often not the gaming GPU, and the catalog has no
per-adapter class-key op, so every `\0000` value is dropped for that reason in addition to the one given.
"Power key" means `HKLM\SYSTEM\CurrentControlSet\Control\Power`; "GraphicsDrivers" means
`HKLM\SYSTEM\CurrentControlSet\Control\GraphicsDrivers`. No vendor-specific (`vendor`) tweak survived: every
NVIDIA/AMD/Intel driver value in these sections is undocumented, legacy, a `\0000` class-key value, or a
service/task that current drivers no longer install.

| batch line | key/value | reason |
|---|---|---|
| 4910 | GraphicsDrivers `HwSchMode`=2 | Not dropped: kept as `gpu_hags_on` (one-click, restart required). |
| 2796, 2845, 3083 | `Enum\<GPU>\Device Parameters\Interrupt Management\MessageSignaledInterruptProperties` `MSISupported`=1 | Not dropped: duplicates merged into `gpu_msi_mode` (caution; every display adapter via `device_reg` class `display`). |
| 3016 | `HKLM\SOFTWARE\Microsoft\Windows\Dwm` `OverlayTestMode`=5 | Not dropped: kept as `gpu_mpo_off` (caution; NVIDIA's published MPO workaround). |
| 2794, 2795, 3077, 3080 | `reg delete` of `Affinity Policy` `DevicePriority` and `MessageSignaledInterruptProperties` `MessageNumberLimit` | Absent by default (deleting them only restores the driver default); the catalog has no per-device delete op. |
| 4915 | `HKCU\Software\Microsoft\DirectX\UserGpuPreferences` `DirectXUserGlobalSettings`="VRROptimizeEnable=0;" | Turns off Windows' variable refresh rate for games without native VRR: a display preference, not a performance gain. |
| 2807, 3087, 3593 | `Services\GpuEnergyDrv` `Start`=4 | Real service, but it only feeds Task Manager's GPU power estimate; no measurable performance effect. Also covers the power section's 1159 copy. |
| 2799–2805 | Per-adapter display class key `RMHdcpKeyglobZero`=1 | Undocumented NVIDIA value that blanks the HDCP key: protected video (Netflix, Blu-ray, some streaming apps) stops playing; no verified latency gain. |
| 3090 | Display class key `\0000\DAL2_DATA__2_0\DisplayPath_4\EDID_D109_78E9\Option` `ProtectionControl` | Hard-codes one specific monitor (EDID `D109_78E9`) on one display path of the author's PC; does nothing elsewhere. Disables HDCP. |
| 2913, 3012 | `HKLM\SOFTWARE\NVIDIA Corporation\NvControlPanel2\Client` `OptInOrOutPreference`=0 | Undocumented NVIDIA Control Panel usage-data flag; effect cannot be verified. |
| 2914–2916, 3009–3011 | `HKLM\SOFTWARE\NVIDIA Corporation\Global\FTS` `EnableRID66610`, `EnableRID64640`, `EnableRID44231`=0 | Undocumented NVIDIA feature flags; meaning and effect cannot be verified. |
| 3006–3008 | `Services\nvlddmkm\FTS` `EnableRID73779`, `EnableRID73780`, `EnableRID74361`=1 | Undocumented NVIDIA driver feature flags; meaning and effect cannot be verified. |
| 2917–2923 | `schtasks /disable` `NvTmRep_CrashReport1–4_{B2FE1952-…}`, `NvTmMon_{…}`, `NvDriverUpdateCheckDaily_{…}`, `NVIDIA GeForce Experience SelfUpdate_{…}` | These tasks exist only with GeForce Experience 3.x / older drivers (the NVIDIA App ships none of them), so on current installs the engine would skip them and the tweak would do nothing. Disabling the update tasks also stops the driver update notices users expect. |
| 3013 | `Services\NvTelemetryContainer` `Start`=4 | Service is not installed by current NVIDIA drivers, so the engine would skip it and the tweak would do nothing. |
| 3014, 3015 | `nvlddmkm\Global\Startup` `SendTelemetryData`=0; `HKLM\SOFTWARE\NVIDIA Corporation\Global\Startup\SendTelemetryData` value `0`=0 | Undocumented; line 3015 is malformed (the value name is used as a key and the value is named `0`). |
| 2931, 2933 | `nvlddmkm\Global\NVTweak` `DisplayPowerSaving`=0 | Undocumented; effect on current drivers cannot be verified. |
| 3004 | `HKCU\SOFTWARE\NVIDIA Corporation\Global\NVTweak\Devices\509901423-0\Color` `NvCplUseColorCorrection`=0 | Device id of one specific PC; does nothing elsewhere. |
| 2984–3002 | `rd /s /q C:\Nvidia\…` (19 folders) | Leftover driver installer files; deleting them only frees disk space, and the catalog has no folder-delete op. |
| 3036–3038 | Display class key root (no `\0000`) `PowerMizerLevel`=1, `PowerMizerLevelAC`=1, `PerfLevelSrc`=8738 | Written to the class root, which no driver reads; PowerMizer registry overrides come from pre-2010 NVIDIA drivers. Also covers the power section's 1122–1125 copies. |
| 2790, 2983 | Display class key `\0000` `PreferSystemMemoryContiguous`=1 | Undocumented; `\0000`. |
| 2791, 2981 | `GraphicsDrivers\Scheduler` `DisableWriteCombining`=1 | Undocumented; if it were read, disabling write-combining would slow CPU-to-GPU uploads. |
| 2812, 2813, 3039–3057 | Display class key `\0000` `D3PCLatency`, `F1TransitionLatency`, `LOWLATENCY`, `Node3DLowLatency`, `PciLatencyTimerControl`, `RMDeepL1EntryLatencyUsec`, `RmGspc*`, `RMLpwr*IdleThresholdUs`, `VRDirectFlip*`, `vrr*MarginUs`, `vrrDeflickerMaxUs` | Undocumented NVIDIA resource-manager internals; effect cannot be verified; `\0000`. |
| 3003, 3024–3035 | Display class key `\0000` `TCCSupported`, `DesktopStereoShortcuts`, `FeatureControl`, `NVDeviceSupportKFilter`, `RmCacheLoc`, `RmDisableInst2Sys`, `RmFbsrPagedDMA`, `RMGpuId`, `RmProfilingAdminOnly`, `TrackResetEngine`, `UseBestResolution`, `ValidateBlitSubRects` | Undocumented NVIDIA values (TCC is a data-center compute mode); `\0000`. |
| 2982 | Display class key `\0000` `Acceleration.Level`=0 | Windows XP hardware-acceleration slider; 0 is already "full"; not read by WDDM drivers. |
| 2832–2843, 2937–2963, 2964–2975, 3017, 3182–3195, 3310–3322, 3586–3592 | Preemption values in GraphicsDrivers, `GraphicsDrivers\Scheduler`, `GraphicsDrivers\Power`, `Services\nvlddmkm` and display class key `\0000`: `EnablePreemption`, `DisablePreemption`, `GPUPreemptionLevel`, `ComputePreemption(Level)`, `EnableMidGfxPreemption(VGPU)`, `EnableMidBufferPreemption*`, `EnableAsyncMidBufferPreemption`, `EnableSCGMidBufferPreemption`, `PerfAnalyzeMidBufferPreemption`, `EnableCEPreemption`, `DisableCudaContextPreemption`, `DisablePreemptionOnS3S4` | GPU preemption is mandatory in WDDM 2.x; these names are mostly invented or Windows 7-era. Where read, turning preemption off lets one long GPU job freeze the desktop and trigger driver resets (TDR). |
| 2821, 2822, 2824, 2825, 2830, 2831, 2934–2936, 3299–3303, 3308, 3309, 3575–3579, 3584, 3585 | GraphicsDrivers `TdrLevel`=0, `TdrDebugMode`=0, `TdrDelay`=60 / 0, `TdrDdiDelay`=0 | Microsoft documents the TDR keys for driver debugging only. `TdrLevel`=0 turns off GPU hang recovery (a hang freezes the PC instead of resetting the driver); the delays only matter once the GPU is already hanging. The batch also writes conflicting `TdrDelay` values (60, then 0). No performance effect. |
| 2823, 3301, 3577 | `GraphicsDrivers\Scheduler` `VsyncIdleTimeout`=0 | Undocumented; effect cannot be verified. |
| 2826–2829, 3304–3307, 3580–3583 | `Services\VxD\BIOS` `AGPConcur`, `CPUPriority`, `FastDRAM`, `PCIConcur`=1 | Windows 9x VxD settings; the key does not exist on Windows NT. |
| 2814–2820, 3292–3298, 3566–3574 | MMCSS `Tasks\DisplayPostProcessing` `Affinity`=0, `Background Only`=True, `BackgroundPriority`=24, `Clock Rate`=10000, `GPU Priority`=18, `Priority`=8, `Scheduling Category`=High, `SFIO Priority`=High, `Latency Sensitive`=True | Windows' own default values for this task; writing them changes nothing. |
| 2847–2904, 2924–2930, 2932, 3204–3285, 3323–3378, 3594–3650 | Power key `ExitLatency*`, `Latency*`, `LatencyTolerance*`, `DefaultD3TransitionLatency*`, `DefaultLatencyTolerance*`, `DefaultMemoryRefreshLatencyTolerance*`, `Monitor(Refresh)LatencyTolerance`, `TransitionLatency`, `RtlCapabilityCheckLatency`, `MiracastPerfTrackGraphicsLatency`, `MaxIAverageGraphicsLatencyInOneBucket`, `DisableVsyncLatencyUpdate`, `DisableSensorWatchdog`, `InterruptSteeringDisabled`, `QosManagesIdleProcessors`, `LowLatencyScalingPercentage`, `HighPerformance`, `HighestPerformance`, `Min/MaximumThrottlePercent`, `MaximumPerformancePercent`, `InitialUnparkCount`, `EnergyEstimationEnabled`, `CoalescingTimerInterval` | Same values and reasons as the power section (power-key rows for 950–1157): undocumented or invented names with no verifiable effect; `CoalescingTimerInterval` is not read there; energy estimation is battery accounting only. Each block is repeated two to four times. |
| 2905–2908, 3286–3289, 3379–3382, 3651–3654 | GraphicsDrivers and `GraphicsDrivers\Power` `MonitorLatencyTolerance`=0, `MonitorRefreshLatencyTolerance`=0 | Undocumented; effect cannot be verified. |
| 2909, 2910, 2976–2980, 3019–3023, 3290, 3291, 3383, 3384, 3655, 3656 | `RMDisablePostL2Compression`, `RmDisableRegistryCaching`, `RmGpsPsEnablePerCpuCoreDpc` in GraphicsDrivers, `GraphicsDrivers\Scheduler`, `GraphicsDrivers\Power`, `nvlddmkm`, `nvlddmkm\NVAPI`, `nvlddmkm\Global\NVTweak` | NVIDIA resource-manager names written to Windows keys where the NVIDIA driver does not read them (and copied into the AMD and Intel sections); undocumented even where read. |
| 2946, 3005, 3018 | GraphicsDrivers / `GraphicsDrivers\Scheduler` `PlatformSupportMiracast`=0 | Undocumented; if read it only breaks wireless display (Miracast). |
| 2809, 2810, 2911, 2912, 3385, 3386, 3537, 3538, 3657, 3658 | `fsutil behavior set memoryusage 2` | NTFS cache setting, not a GPU setting; the catalog has no fsutil op for it; left to the storage category (same command at 406 and 5444). |
| 3093–3132, 3135–3166, 3198, 3387, 3392–3419, 3455–3481 | Display class key `\0000` AMD values: `EnableUlps`, `EnableUlps_NA`, `EnableAspmL0s`, `EnableAspmL1`, `Disable*PowerGating`, `Enable*ClockGating`, `PP_SclkDeepSleepDisable`, `PP_ThermalAutoThrottlingEnable(d)` (=0 and =1), `PP_GPUPowerDownEnabled` (=1 and =0), `PP_ActivityTarget`, `PP_ODNFeatureEnable`, `PP_AllGraphicLevel_*Hyst`, `PP_MCLKStutterModeThreshold`, `StutterMode`, `KMD_DeLagEnabled`, `KMD_FRTEnabled`, `KMD_EnableComputePreemption`, `KMD_MaxUVDSessions`, `DisableDMACopy`, `DisableBlockWrite`, `DisableBlocknv11`, `DisableFBC*`, `DisableEarlySamuInit`, `GCOOPTION_DisableGPIOPowerSaveMode`, `Dal*`, `WmAgpMaxIdleClk`, `3D_Refresh_Rate_Override_DEF`, `AllowSnapshot`, `AllowSubscription`, `AllowRSOverlay`, `AllowSkins`, `AAF_NA`, `AntiAlias_NA`, `ASTT_NA`, `AreaAniso_NA`, `AutoColorDepthReduction_NA`, `Adaptive De-interlacing` | `\0000` (on Ryzen APU + Radeon or Intel + Radeon PCs it is often the wrong adapter). Most are Catalyst/Crimson-era names that current AMD Software no longer reads; the rest are undocumented. `EnableUlps`=0 is a real AMD value but only matters for CrossFire/multi-GPU stability and costs idle power. Several values are written with opposite settings (power gating, thermal throttling); thermal throttling off would be unsafe if honoured. |
| 3169–3179, 3388–3391, 3482–3531 | Display class key `\0000\UMD` `Main3D(_DEF/_SET)`, `FlipQueueSize`, `ShaderCache`, `TFQ(_DEF)`, `Tessellation(_OPTION)(_DEF)`, `AnisoType/Degree(_DEF)`, `AnisotropyOptimise`, `TrilinearOptimise`, `TextureOpt/Lod(_DEF)`, `ForceZBufferDepth(_DEF/_SET)`, `ForceTripleBuffering(_DEF)`, `SurfaceFormatReplacements(_DEF)`, `TruformMode(_DEF/_NA)`, `LodAdj`, `EQAA`, `PowerState`, `NoOSPowerOptions`, `ZFormats_NA`, `AntiStuttering`, `TurboSync`, `HighQualityAF` | Radeon profile values that AMD Software rewrites from its own settings (the user sets them there); Truform and TurboSync are 2000s features; `\0000`; some are written twice with different types. |
| 3420–3454 | `%REGPATH_AMD%` (35 values, same names as above) | `REGPATH_AMD` is never defined in the batch, so these commands target an empty key path and fail. |
| 3532 | `HKLM\SYSTEM\ControlSet001\Control\Class\{4d36e968-…}\0000` `EnableUlps`=0 | Duplicate of the `\0000` `EnableUlps` row above. |
| 3201 | `Services\amdlog` `Start`=4 | AMD logging driver; not present on every AMD install (the engine skips it where it is missing); no performance effect. |
| 3555–3565 | Intel adapter class keys `Disable_OverlayDSQualityEnhancement`=1, `IncreaseFixedSegment`=1, `AdaptiveVsyncEnabled`=0, `DisablePFonDP`=1, `EnableCompensationForDVI`=1, `NoFastLinkTrainingForeDP`=0, `ACPowerPolicyVersion`=16898, `DCPowerPolicyVersion`=16642 | Undocumented Intel driver values (the power-policy versions are driver-internal version stamps, not settings); effect cannot be verified. |
| 3659 | `HKLM\Software\Intel\GMM` `DedicatedSegmentSize`=512 | Intel GMA-era (pre-2010) setting; current Intel drivers allocate shared memory dynamically through WDDM. |
| 2489–2535 | `HKLM\SOFTWARE\(Wow6432Node\)Microsoft\DirectDraw` and `Direct3D(\Drivers)` `DisableAGPSupport`, `UseNonLocalVidMem`, `DisableDDSCAPSInDDSD`, `EmulationOnly`, `EmulatePointSprites`, `EmulateStateBlocks`, `ForceRgbRasterizer`, `EnableDebugging`, `FullDebug`, `DisableDM`, `EnableMultimonDebugging`, `LoadDebugRuntime`, `EnumReference`, `EnumSeparateMMX`, `EnumRamp`, `EnumNullDevice`, `FewVertices`, `DisableMMX`, `MMX Fast Path`, `MMXFastPath`, `UseMMXForRGB`, `ForceNoSysLock` | DirectX 7–9 era DirectDraw/Direct3D software-rasterizer and debug switches (AGP, MMX, RGB/ramp rasterizers); not read by DirectX 10–12 or modern WDDM drivers; the rest are already the defaults. |

## game

Sources: `gamepriority` (1517–1560, menu only), `fn` (1561–1693), `game` (1694–1864), `tweaksprio` (1865–1919),
`FortniteTweaks1` (2366–2420), `FortniteFix` (4729–4743), and the game items of `ApplyAll1` (414–445, 458,
537–538) and `AutoOpti1` (4865–4902, 5036, 5581–5584). "IFEO" means
`HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options`; "MMCSS" means
`HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile`. Under an IFEO `PerfOptions` key
Windows reads only `CpuPriorityClass`, `IoPriority` and `PagePriority`; every other name the batch writes there
is invented.

| batch line | key/value | reason |
|---|---|---|
| 1517–1560 | `gamepriority` | Menu only; writes no values. |
| 538, 5584 | `HKCU\Software\Microsoft\GameBar` `AutoGameModeEnabled`=1 | Not dropped: kept as `game_mode_on` (one-click). |
| 537, 5583 | `GameBar` `AllowAutoGameMode`=1 | Legacy name from the Windows 10 Creators Update; current builds use `AutoGameModeEnabled`. |
| 5581, 5582 | Interactive choice "Windows 10" sets `AllowAutoGameMode`/`AutoGameModeEnabled`=0 | Turns Game Mode off, against Microsoft's recommendation; replaced by `game_mode_on`. |
| 4867, 4872, 4888, 4893 | `HKCU\System\GameConfigStore` `GameDVR_Enabled`=0, `HKCU\…\CurrentVersion\GameDVR` `AppCaptureEnabled`=0 | Not dropped: duplicates merged into `game_dvr_off` (one-click). |
| 4865, 4866, 4886, 4887 | `HKLM\SOFTWARE\Microsoft\PolicyManager\default\ApplicationManagement\AllowGameDVR` `value`=0; policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR` `AllowGameDVR`=0 | The PolicyManager `default` store is MDM-internal (Windows rewrites it); the policy locks the setting with "managed by your organization". The two user values in `game_dvr_off` already stop recording. |
| 4868–4871, 4889–4892 | `GameConfigStore` `GameDVR_FSEBehaviorMode`=2, `GameDVR_HonorUserFSEBehaviorMode`=0, `GameDVR_DXGIHonorFSEWindowsCompatible`=1, `GameDVR_EFSEFeatureFlags`=0 | Not dropped: kept as `game_fso_off` (caution; effect disputed). With `HonorUserFSEBehaviorMode`=0 Windows does not honour the user-set mode, so the batch's set did not take effect; the tweak uses the usual working set (`HonorUserFSEBehaviorMode`=1 plus `GameDVR_FSEBehavior`=2). |
| 4895–4902 | Interactive choice "No" re-enables Game DVR and FSO | The app's revert restores the previous values instead. |
| 432, 433, 1849, 1850 | MMCSS `Tasks\Games` `Scheduling Category`=High, `SFIO Priority`=High | Not dropped: duplicates merged into `game_mmcss_games_priority`. |
| 426–429, 431, 1843–1846, 1848 | MMCSS `Tasks\Games` `Affinity`=0, `Background Only`=False, `BackgroundPriority`=0, `Clock Rate`=10000, `Priority`=2 | Windows defaults for this task (`BackgroundPriority` is only used when `Background Only` is True). |
| 430, 1847 | MMCSS `Tasks\Games` `GPU Priority`=8 / 18 | Microsoft documents this value as not used; 8 is the default anyway. |
| 434, 1851 | MMCSS `Tasks\Games` `Latency Sensitive`=True | Not a documented MMCSS task value; no verifiable effect. |
| 414–422, 1825–1833 | MMCSS `Tasks\Low Latency` (9 values) | Not a Windows task; only used by programs that register under that exact name, and none do. |
| 1835–1841 | MMCSS `Tasks\Audio` `GPU Priority`=1, `Priority`=2, `SFIO Priority`=High, rest default | Lowers audio thread priority from the default 6 to 2 (can cause crackling); `GPU Priority` is unused; no gaming benefit. |
| 1853–1858 | MMCSS `Tasks\Window Manager` values | Windows defaults for this task; writing them changes nothing. |
| 458, 1911 | MMCSS `SystemResponsiveness`=0 | Not dropped: kept as `game_system_responsiveness` with 10, because Windows treats values below 10 as the default 20. |
| 438, 1899, 5036 | `PriorityControl` `Win32PrioritySeparation`=20 (0x14) | Real value, but 0x14 means long quanta with no foreground boost, which favours background work; the client default (2: short, variable quanta with a 3:1 foreground boost) already suits games. |
| 442–445, 1877, 1878 | `PriorityControl` `IRQ8Priority`=1, `IRQ16Priority`=2 | Not read by any Windows NT version. |
| 1898 | `PriorityControl` `ConvertibleSlateMode`=0 | 2-in-1 tablet/laptop mode indicator for OEMs; no performance effect (also in `AutoOpti1` 5035). |
| 1892, 1893 | `Services\VxD\BIOS` `CPUPriority`, `FastDRAM`=1 | Windows 9x VxD settings; the key does not exist on Windows NT. |
| 1905, 1906 | `Services\DXGKrnl\Parameters`, `Services\nvlddmkm\Parameters` `ThreadPriority`=15 | Undocumented; no evidence either driver reads it. |
| 1870, 1871 | IFEO `csrss.exe\PerfOptions` `CpuPriorityClass`=4, `IoPriority`=4 | csrss already runs at high base priority; raising a critical system process to Realtime gives no input benefit and can starve the rest of the system; `IoPriority` 4 is not a valid value. |
| 1887, 1888 | IFEO `svchost.exe\PerfOptions` `CpuPriorityClass`=1, `IoPriority`=0 | Harmful: starts every Windows service host at Idle CPU and Very Low I/O priority (slow logon, audio and network glitches). |
| 1880 | `HKLM\SOFTWARE\Policies\Microsoft\SQMClient\Windows` `CEIPEnable`=0 | Real privacy policy (Customer Experience Improvement Program), but not a game setting; belongs to the privacy category, not ported here. |
| 1881 | `HKLM\SOFTWARE\Policies\Microsoft\Windows\AppCompat` `AITEnable`=0 | Real privacy policy (Application Telemetry), but not a game setting; belongs to the privacy category, not ported here. |
| 1685, 1821 | IFEO `FortniteClient-Win64-Shipping.exe` / `ValorantClient-Win64-Shipping.exe` `PerfOptions` `CpuPriorityClass`=3 | Not dropped: kept as `game_fortnite_priority` and `game_valorant_priority` (caution). The Valorant game process is `VALORANT-Win64-Shipping.exe`; `ValorantClient-Win64-Shipping.exe` does not exist, so the batch's Valorant entry never applied; the tweak uses the real name. |
| 1576, 1712 | IFEO `…\PerfOptions` `CpuPriorityClass`=8 | Not a valid priority class (valid: 1–6); overwritten by =3 later in the same section. |
| 1580, 1598, 1686, 1716, 1734, 1822 | IFEO `…\PerfOptions` `IoPriority`=8 / 3, `Io Priority`=8 | 8 is invalid; 3 (High) is reserved for system use; `Io Priority` with a space is not read. Normal I/O priority is already the default. |
| 1593, 1729 | IFEO `…\PerfOptions` `IOPriorityClass`=0 | Not a value Windows reads (the real name is `IoPriority`); 0 would mean Very Low. |
| 1577–1579, 1581–1585, 1587–1592, 1594–1597, 1599, 1601–1607, 1609, 1611–1684, 1713–1715, 1717–1721, 1723–1728, 1730–1733, 1735, 1737–1743, 1745, 1747–1820 | IFEO `…\PerfOptions` `GpuPriorityClass`, `CpuPriority`, `GpuPriority`, `Clock Rate`, `Affinity`, `CpuThreadPriority`, `CpuThreadCount`, `CpuCoresAlways`, `RMHdcpKeyGlobZero`, `PowerLimitEnabled`, `Priority`, `GPU Priority`, `SystemResponsiveness`, `MaxPerformance`, `GpuMax*`, `Rendering*`, `Latency*`, `*Spread`, `Gpu*Clock*`, `HwSchMode`, `HwSchedMode`, `DisableRendering*Preemption`, `Smooth*`, `PowerSaving*`, `SwapEffectUpgradeCache`, `TVSupportEnabled`, `WatchdogResumeTimeout`, `POSTTime`, `BootmgrUserInputTime` and the other invented names (about 90 per game) | Not read by Windows: `PerfOptions` honours only `CpuPriorityClass`, `IoPriority` and `PagePriority`. Names copied from real system keys (`HwSchMode`, `SystemResponsiveness`) do nothing here; the real `HwSchMode` is kept as `gpu_hags_on`. |
| 1586, 1722 | IFEO `…\PerfOptions` `PowerThrottlingOff`=1 | Not read here; covered system-wide by `cpu_power_throttling_off`. |
| 1600, 1736 | IFEO `…\PerfOptions` `HiberbootEnabled`=0 | Not read here; the real Fast Startup switch is covered by `pow_hibernate_off`. |
| 1608, 1744 | IFEO `…\PerfOptions` `CoalescingTimerInterval`=0 | Not read here (nor in the keys listed in the power section). |
| 1610, 1746 | IFEO `…\PerfOptions` default value = "True" | Not read by Windows. |
| 1687, 1823 | IFEO `FortniteClient-Win64-Shipping.exe` / `ValorantClient-Win64-Shipping.exe` `UseLargePages`=1 | Only affects programs built to request large pages, and needs the "Lock pages in memory" privilege, which games do not have; no effect. |
| 1566–1575 | `HKLM\SOFTWARE\Policies\Microsoft\Windows\QoS\fortnite` (DSCP 46 policy) | Broken policy: no `Version` value and an invalid `throttle Rate` "_1". On PCs that are not in a domain Windows applies QoS policies only when the NLA check is turned off (`Tcpip\QoS` `Do not use NLA`), which the batch never sets; home routers and ISPs usually ignore or strip DSCP. |
| 1698–1710 | `HKLM\Software\Policies\Microsoft\Windows\QoS\ValorantClient-Win64-Shipping.exe` (DSCP 46 policy) | `Application Name` becomes `ValorantClient-Win64-Shipping.exe.exe` (a wrong exe name plus a doubled `.exe`), so it matches nothing; same NLA and router limits as above. |
| 4730–4734 | `FortniteFix`: deletes the Fortnite QoS key, IFEO `PerfOptions` and `UseLargePages` | Undo action for the batch's own Fortnite tweaks; the app's revert restores the previous state instead (the batch also deletes `QoS\FortniteClient-Win64` while it created `QoS\fortnite`). |
| 2367 | `HKCU\Control Panel\Desktop` `Composition`=0 | Not a value Windows reads; desktop composition cannot be turned off since Windows 8. |
| 2368 | `HKCU\…\Explorer` `Advanced` (DWORD)=0 | `Advanced` is a key, not a value; placebo. |
| 2369 | `HKCU\Software\Microsoft\DirectDraw` `DirectDrawSync`=0 | Not a value DirectX reads. |
| 2370 | `Explorer\Advanced` `ListviewShadow`=1 | Desktop icon label shadow (cosmetic); duplicate of 4268, 4706, 4998, 5687, curated with the visual category. |
| 2371 | `Control Panel\Desktop` `MenuShowDelay`=0 | Menu delay (cosmetic); duplicate of 453, 4381, 4388, 5176, curated with the visual category. |
| 2372 | `HKCU\…\Explorer` `EnableWindowAnimation`=0 | Not a value Windows reads (window animations are `WindowMetrics` `MinAnimate`). |
| 2373, 2374, 2384, 2407, 2408 | `Memory Management` `DisablePagingExecutive`=1, `LargeSystemCache`=1, `PagedPoolSize`=0 | Memory settings, not game settings; duplicates of `reged` 4110–4136 and `AutoOpti1` 5289–5304, curated with the performance category. |
| 2375 | `Tcpip\Parameters` `Tcp1323Opts`=3 | Window scaling is already on by default; adding timestamps only adds 12 bytes per packet. |
| 2376, 2377, 2382, 2414, 2415 | `Tcpip\Parameters` `GlobalMaxTcpWindowSize`, `TcpWindowSize` (65535 / 8388608) | Ignored since Windows Vista (receive-window auto-tuning); 65535 would cap throughput if it were read. |
| 2379 | `LanmanServer\Parameters` `MaxMpxCt`=50 | SMB file-server setting; no effect on games. |
| 2380, 2381, 2387, 2401, 2402, 2412 | `Tcpip\Parameters` `TcpAckFrequency`=1, `TCPNoDelay`/`TcpNoDelay`=1, `TcpDelAckTicks`=0 | The global location is not read by the Vista+ stack; same as the network rows for 775, 776, 779. |
| 2383, 2404, 2410 | `Tcpip\Parameters` `TcpMaxDataRetransmissions`=2, `MaxSynRetransmissions`=2 | Makes connections give up sooner on packet loss (disconnects instead of recovering); no latency gain. |
| 2385, 2403 | `Tcpip\Parameters` `MaxUserPort`=65534 | Ignored since Windows Vista; same as the network row for 691. |
| 2386, 2400 | `Tcpip\Parameters` `TcpTimedWaitDelay`=30 | Covered by `net_time_wait` (32). |
| 2388, 2405, 2406 | `Tcpip\Parameters` `KeepAliveTime`=300000, `KeepAliveInterval`=5000 | Only affects idle connections that ask for keep-alive; no effect on game traffic. |
| 2397–2399, 2413 | `Tcpip\Parameters` `MaxConnParts`, `MaxConnectionsPer1_0Server`, `MaxConnectionsPerServer`, `MaxConnections` | Not Tcpip values (WinINet settings); placebo; same as the network row for 690. |
| 2411 | `Tcpip\Parameters` `TcpInitialRtt`=300 | Windows Server 2003 value; not read by the current stack. |
| 2378 | `Control Panel\Desktop` `ForegroundFlashCount`=0 | Cosmetic taskbar flashing setting; no performance effect. |
| 2389, 2409 | `Explorer\Advanced` / `Explorer` `EnableBalloonTips`=0 | Legacy notification-balloon setting; duplicate of 530 and 5056, curated with the other categories. |
| 2390 | `Control Panel\Desktop` `LowPower`=1 | Legacy screen-saver low-power flag; no effect on games. |
| 2391 | `Control Panel\Desktop` `ForegroundLockTimeout`=0 | Lets background programs steal focus immediately, which can minimise a fullscreen game; duplicate of 4380 and 4387. |
| 2392 | `HKLM\…\Explorer` `MaximizeEnable`=0 | Not a value Windows reads. |
| 2393–2396 | GraphicsDrivers `TdrDelay`=8, `TdrLevel`=0, `TdrDdiDelay`=5 | Same reason as the gpu TDR row; `TdrDdiDelay`=5 is the default. |

## input

Sources: `InputDelay1` (2200–2225), `kb` (4342–4374), `mou` (4376–4428), `data` (4431–4630, with `mous` 4632 and
`keyb` 4645), the input items of `AutoOpti1` (4920–4923, 5752–5852) and `ApplyAll1` (380, 387, 388). "Mouse key"
means `HKCU\Control Panel\Mouse`; "Keyboard Response" means `HKCU\Control Panel\Accessibility\Keyboard Response`.

| batch line | key/value | reason |
|---|---|---|
| 4403–4405, 5820–5822 | Mouse key `MouseSpeed`="0", `MouseThreshold1`="0", `MouseThreshold2`="0" | Not dropped: merged into `in_mouse_accel_off` (one-click). |
| 4407, 4408, 5824, 5825 | Mouse key `SmoothMouseXCurve`, `SmoothMouseYCurve` (custom curves) | Only used while "Enhance pointer precision" is on; `in_mouse_accel_off` turns it off, so the curves do nothing. |
| 4394, 4396–4398, 4400–4402, 4406, 4409, 4410, 5811, 5813–5815, 5817–5819, 5823, 5826, 5827 | Mouse key `ActiveWindowTracking`=0, `DoubleClickHeight`/`DoubleClickWidth`="4", `DoubleClickSpeed`="500", `MouseHoverHeight`/`MouseHoverWidth`="4", `MouseSensitivity`="10", `MouseTrails`="0", `SnapToDefaultButton`="0", `SwapMouseButtons`="0" | Windows defaults. |
| 4395, 4399, 5812, 5816 | Mouse key `Beep`="No", `ExtendedSounds`="No" | Not values Windows reads under this key (the system beep lives under `Control Panel\Sound`); no performance effect either way. |
| 4419 | Mouse key `MouseHoverTime`="8" | Hover delay for tooltips and hover pop-ups (default 400 ms); a preference, not input delay; at 8 ms tooltips pop up under the pointer all the time. |
| 4381–4393 | `HKU\.DEFAULT\Control Panel\Desktop` `MenuShowDelay`, `MouseWheelRouting`; `HKU\.DEFAULT\Control Panel\Mouse` / `Sound` `Beep`, `ExtendedSounds` (each written twice) | `HKU\.DEFAULT` is the profile of the sign-in screen, not of the user; no effect after sign-in. |
| 4411–4418, 5803–5810 | `HKLM\SOFTWARE\Microsoft\Input\Settings\ControllerProcessor\CursorSpeed` / `CursorMagnetism` values | Settings of the gamepad-driven cursor (navigating apps with an Xbox controller); no effect on a mouse. |
| 2215, 2216, 4366, 4420 | `Services\mouclass\Parameters`, `Services\kbdclass\Parameters` `ThreadPriority`=31 | Not a documented class-driver value and no evidence mouclass/kbdclass read it. |
| 4582, 4583, 4595, 4596, 4608, 4609, 5784, 5850 | `mouclass\Parameters` `MouseDataQueueSize`, `kbdclass\Parameters` `KeyboardDataQueueSize` = 32, 37, 48 (menu choices) | Not dropped: the options of `in_mouse_queue_size` / `in_keyboard_queue_size` (caution; the claimed latency gain does not exist, see the descriptions). The `data` menu shows hex labels (20, 25, 30) but writes the same digits as decimal values (32, 37, 48); the app shows the values that are really written. |
| 2213, 2214, 4504–4570, 5753, 5780–5783, 5801, 5846–5849 | `MouseDataQueueSize` / `KeyboardDataQueueSize` = 19 … 25 (menu labels 14 … 19, `InputDelay1` 21, "High End" … "Low End" choices) | Not offered: below about 30 the queue can drop input while the PC is busy (especially with 4000–8000 Hz mice), and smaller values bring no latency gain; the smallest option is 32. |
| 4622, 4623 | "Revert": both queue sizes = 80 | Not the Windows default (100); the app's revert restores the previous value instead. |
| 4634, 4635, 4647, 4648, 5785, 5786, 5851, 5852 | Open regedit at the mouclass/kbdclass key ("manually") | Not a tweak; the options replace manual editing. |
| 5752, 5799 | `main.cpl keyboard`, `main.cpl mouse` | Opens Control Panel; offered as a launcher. |
| 2220, 2221, 4357, 4359, 5754, 5756 | `HKCU\Control Panel\Keyboard` `KeyboardDelay`="0", `KeyboardSpeed`="31" | Not dropped: merged into `in_keyboard_repeat_fast`. |
| 4358, 5755 | `Control Panel\Keyboard` `InitialKeyboardIndicators`="0" | Turns Num Lock off at sign-in; a preference with no input-delay effect. |
| 2222–2225 | Keyboard Response `Flags`="122", `ToggleKeys` `Flags`="58", `StickyKeys` `Flags`="506", `MouseKeys` `Flags`="0" | Not dropped: kept as `in_accessibility_shortcuts_off` (one-click). For Mouse Keys the tweak writes "58" (the default 62 without the shortcut bit) instead of "0", so the feature stays available in Settings like the other three. |
| 4347–4350, 4920–4923, 5758 | The same four `Flags` written as "0" | Duplicates; the tweak uses the values of 2222–2225, which turn the features and their shortcuts off but keep them available in Settings. |
| 4356 | Keyboard Response `Flags`=0 as REG_DWORD | Wrong type: Windows stores this value as a string, so the line replaces a valid value with one it cannot read; covered by `in_accessibility_shortcuts_off`. |
| 4360, 4362–4365, 5757, 5759–5762 | Keyboard Response `DelayBeforeAcceptance`, `Last BounceKey Setting`, `Last Valid Delay`, `Last Valid Repeat`, `Last Valid Wait` | Filter Keys parameters; only used while Filter Keys is on (it is off). |
| 387, 388, 2211, 2212 | Display class key `\0000` `LOWLATENCY`=1, `Node3DLowLatency`=1 | Same as the gpu row for 2812/2813 (undocumented NVIDIA values on the first display adapter only). |
| 2217 | `Session Manager\kernel` `DebugPollInterval`=1000 | Kernel-debugger polling; only used while a kernel debugger is attached. |
| 380, 2201, 2218, 2219 | IFEO `csrss.exe\PerfOptions` `IoPriority`=3, `CpuPriorityClass`=4 | Same as the game row for 1870/1871. |
| 2202–2210 | `Enum\%%a` / `%%s` / `%%i\Device Parameters` `SelectiveSuspendOn`, `AllowIdleIrpInD3`, `EnableSelectiveSuspend`, `WDF\IdleInWorkingState` | Loop variables used outside any `for` loop: reg.exe writes to a literal `Enum\%a` key that no device uses, so nothing applied. The working USB power settings are under `## usb`. |

## visual

Sources: `visual` (4165–4226, with the `performance` launcher 4210–4217), `per` (4227–4274), `VisualFixes`
(4698–4716), the visual items of `ApplyAll1` (453, 490, 529, 530) and `AutoOpti1` (4861, 4994–5002, 5024–5027,
5055, 5056, 5176, 5481, 5482, 5675–5693). "Desktop key" means `HKCU\Control Panel\Desktop`; "Advanced" means
`HKCU\…\Explorer\Advanced`; "VisualEffects" means `HKCU\…\Explorer\VisualEffects`.

| batch line | key/value | reason |
|---|---|---|
| 4700 | Desktop key `UserPreferencesMask`=90 12 03 80 10 00 00 00 | Not dropped: kept as `vis_effects_performance` (one-click). It matches "Adjust for best performance" for animations, fades and shadows, which the batch's `performance` message box asked the user to pick by hand; thumbnails, font smoothing and window-drag settings are left unchanged. |
| 4256, 4707, 5675 | VisualEffects `VisualFXSetting`=3 | Not dropped: merged into `vis_effects_performance`. |
| 4249, 4702, 4994 | `Desktop\WindowMetrics` `MinAnimate`="0" | Not dropped: merged into `vis_effects_performance`. |
| 4705, 4999 | Advanced `TaskbarAnimations`=0 | Not dropped: merged into `vis_effects_performance`. |
| 4708, 5000 | `HKCU\Software\Microsoft\Windows\DWM` `EnableAeroPeek`=0 | Not dropped: merged into `vis_effects_performance`. |
| 4250, 4995 | `Desktop\WindowMetrics` `MaxAnimate`="0" | Not a value Windows reads; the minimise/maximise animation is `MinAnimate`. |
| 4257–4274, 5676–5693 | VisualEffects `<effect>` `DefaultApplied` (18 effects) | Template Windows uses only for "Let Windows choose"; with `VisualFXSetting`=3 (custom) the effects come from `UserPreferencesMask`, which `vis_effects_performance` sets. |
| 4235, 5001 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\DWM` `DWMWA_TRANSITIONS_FORCEDISABLED`=1 | Not a registry value: `DWMWA_TRANSITIONS_FORCEDISABLED` is a window attribute that programs pass to `DwmSetWindowAttribute`. |
| 4236, 5002 | Policy `…\Windows\DWM` `DisallowAnimations`=1 | Machine-wide policy for the same window animations `vis_effects_performance` already turns off per user; as a policy it cannot be changed in Settings until it is removed. |
| 4699, 4701, 4703, 4704, 4706, 4709, 4998, 5024, 5025 | Desktop key `FontSmoothing`="2", `DragFullWindows`="1", `FontSmoothingType`=2; Advanced `ListviewAlphaSelect`=1, `IconsOnly`=0, `ListviewShadow`=1; DWM `AlwaysHibernateThumbnails`=0 | Windows defaults (ClearType, window contents while dragging, translucent selection, thumbnails, icon label shadows); the batch kept them on, so writing them changes nothing on a default PC. `ListviewShadow` is also the game row for 2370. |
| 490, 4243, 4861 | `HKCU\…\Themes\Personalize` `EnableTransparency`=0 | Not dropped: merged into `vis_transparency_off` (one-click). |
| 453, 5176 | Desktop key `MenuShowDelay`="0" | Not dropped: merged into `vis_menu_delay_off` (one-click); also the game row for 2371. |
| 4255 | `HKCU\…\VideoSettings` `VideoQualityOnBattery`=1 | Battery only: makes video playback use more power on battery; no performance effect. |
| 4996 | Advanced `ExtendedUIHoverTime`=196608 | Delays taskbar thumbnail previews by about 3 minutes (in effect turns them off); no performance effect. |
| 4997 | Advanced `DontPrettyPath`=1 | Cosmetic (keeps the letter case of file names as typed). |
| 5026, 5027 | `Explorer\MultitaskingView\AllUpView` `AllUpView`=0, `Remove TaskView`=1 | Names from Windows 10 preview builds; not read by current Windows. |
| 529, 5055 | Advanced `DisallowShaking`=1 | Turns off Aero Shake (shake a window to minimise the others); a behaviour preference with no performance effect. |
| 530, 5056 | Advanced `EnableBalloonTips`=0 | Legacy notification balloons; Windows 10/11 show toast notifications instead (also the game rows for 2389, 2409). |
| 5481 | Desktop key `LogPixels`=96, `Win8DpiScaling`=1 (answer "Yes" to "Do you play on 1920x1080?") | Not dropped: kept as `vis_display_scaling_100` (caution: text gets very small on 4K and small laptop screens). |
| 5482 | Answer "No" | Changes nothing. |
| 4215, 4217 | Message box + `SystemPropertiesPerformance.exe` | Not a tweak; offered as a launcher; `vis_effects_performance` applies the setting the message box asked for. |
| 4712–4716 | Asks to sign out | The app shows "restart recommended" instead. |

## performance

Sources: `Applyall2` (1240–1457), `ram` (3903–3931, menu only), `svc` (3945–4096), `reged` (4106–4156), the
performance items of `ApplyAll1` (449, 485) and `AutoOpti1` (5003–5008, 5064, 5072, 5172, 5289–5309, 5343–5345,
5437, 5452, 5489, 5491, 5563–5570). "MM" means `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory
Management`; "IFEO" as in `## game`. Rows marked "left for the privacy category" are privacy values that
`Applyall2` and `AutoOpti1` also write.

| batch line | key/value | reason |
|---|---|---|
| 4009–4083, 5564–5569 | `HKLM\SYSTEM\ControlSet001\Control` `SvcHostSplitThresholdInKB` (4/8/12/16/32/64 GB choices) | Not dropped: kept as `perf_svchost_split`, one option per RAM size, picked automatically from the installed RAM. The batch values were wrong (for example "4 GB" wrote 68,764,420 KB ≈ 65 GB, "64 GB" about 1.6 TB), which happened to group all services on every PC; the app writes RAM size × 1,048,576 KB to `CurrentControlSet` (`ControlSet001` is not always the active set). |
| 4021, 5563 | "2 GB" choice | Below the Windows default threshold (3.5 GB) services are grouped anyway; no option needed. |
| 4096, 5570 | "Revert": 3670016 | Windows default; the app's revert restores the previous value instead. |
| 4000 | `taskmgr` ("check your RAM") | Not needed: the app reads the installed RAM. |
| 4110, 4120, 4132, 5289, 5300 | MM `LargeSystemCache`=1 and =0 | Contradictory (1, then 0 in the same section); 0 is the client default; 1 is a file-server setting that can take RAM away from programs (also the game row for 2374). |
| 4111, 4147, 5290 | `MM\PrefetchParameters` `EnablePrefetcher`=3 / 0 | 3 is the default; 0, written later in the same section, turns off the prefetcher and makes boot and app start slower. |
| 4112, 4148, 5291 | `MM\PrefetchParameters` `EnableSuperfetch`=0 | Not read on Windows 10/11; SysMain is controlled through its service. |
| 4118, 4130, 5298 | MM `ClearPageFileAtShutdown`=0 | Windows default. |
| 4119, 4131, 5299 | MM `DisablePagingExecutive`=1 | Not dropped: kept as `perf_paging_executive_off` (also the game rows for 2373, 2407). |
| 4121–4123, 4133–4135, 5301–5303 | MM `NonPagedPoolQuota`, `NonPagedPoolSize`, `PagedPoolQuota`=0 | Windows defaults (0 = sized by the system). |
| 4124, 4126, 4127, 4136, 4138, 4139, 5304, 5306, 5307 | MM `PagedPoolSize`, `SessionPoolSize`, `SessionViewSize`=192 | Ignored by 64-bit Windows, which sizes kernel pools dynamically (also the game rows for 2384, 2408). |
| 4125, 4137, 5305 | MM `SecondLevelDataCache`=1024 | Ignored since Windows XP (the CPU reports its own cache size). |
| 4128, 4129, 4140, 4141, 5308, 5309 | MM `SystemPages`=0xFFFFFFFF, `PhysicalAddressExtension`=1 | 32-bit only; PAE is always on in 64-bit Windows. |
| 4145, 4146, 5343, 5344 | MM `IoPageLockLimit`=16710656, `PoolUsageMaximum`=96 | Windows 2000/XP/Server 2003 values; not read by current Windows. |
| 5345 | MM `Start`=4 | `Start` is a service value; meaningless under this key. |
| 1324–1326, 1328–1330, 1382–1389, 4142–4144 | MM `FeatureSettings`=1, `FeatureSettingsOverride`, `FeatureSettingsOverrideMask` (also under `ControlSet001`) | Covered by `cpu_spectre_meltdown_off` (caution), same as the cpu rows for 5332–5334. |
| 4155, 4156, 5452 | `Disable-MMAgent -MemoryCompression` (and `-ApplicationPreLaunch`) | Memory compression keeps more data in RAM instead of the page file and usually helps PCs with little RAM; no shown gaming benefit from turning it off, and the catalog has no MMAgent op. App pre-launch only preloads Store apps. |
| 1241, 1242 | `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\System` `AllowExperimentation`=0, `…\PolicyManager\default\System\AllowExperimentation` `value`=0 | PolicyManager stores are MDM-internal and rewritten by Windows (same as the game row for 4865); a privacy value, left for the privacy category (also `ApplyAll1` 542, 543). |
| 1243 | `OneDriveSetup.exe /UNINSTALL` | Offered as the OneDrive uninstall tool. |
| 1244, 1246, 1247 | `rd` OneDrive temp and app folders | Deletes files; the uninstall tool removes the app itself. |
| 1248, 1249 | `HKCR\(Wow6432Node\)CLSID\{018D5C66-…}\ShellFolder` `Attributes`=0 | Only hides OneDrive in the Explorer navigation pane; cosmetic. |
| 1250, 1251 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\OneDrive` `DisableFileSync`=1, `DisableFileSyncNGSC`=1 | Real policy (blocks OneDrive), but a feature/privacy choice, not performance; left for the privacy category. |
| 1252, 1253 | Same policy `DisableMeteredNetworkFileSync`=0, `DisableLibrariesDefaultSaveToOneDrive`=0 | Same behaviour as when the policy is not set (default). |
| 1254–1301 | `HKLM\SYSTEM\ResourcePolicyStore\ResourceSets\Policies\…` `CapPercentage`, `SchedulingType`, `IsLowPriority`, `BasePriority`, `OverTargetPriority`, `IOBandwidth`, `CommitLimit`, `CommitTarget` | Undocumented internal tables of the Windows resource manager (background app throttling); effect cannot be verified. |
| 1257, 1258 | `sc stop WSearch`, `sc config WSearch start=disabled` | Not dropped: kept as `perf_search_indexer_off` (caution). |
| 1259–1261 | `HKLM\SOFTWARE\Microsoft\Windows Search` `SetupCompletedSuccessfully`=0, `AllowIndexingEncryptedStoresOrItems`=0, `EnablePerUserCatalog`=0 | `SetupCompletedSuccessfully`=0 makes Search rebuild its index from scratch (more disk use, not less); the policy name is in the wrong key and 0 is its default anyway; `EnablePerUserCatalog` is internal. |
| 1305 | `DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity` `Enabled`=0 | Not dropped: kept as `perf_memory_integrity_off` (caution). |
| 1368 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\DeviceGuard` `EnableVirtualizationBasedSecurity`=0 | Turns off all of VBS (Credential Guard, Windows Hello enhanced sign-in, System Guard), not just Memory integrity, which is the part Microsoft names for gaming; as a policy it also locks the setting. |
| 1370 | Same policy `HVCIMATRequired`=0 | Same behaviour as when the policy is not set (default). |
| 1309 | `Set-ProcessMitigation -System -Disable <all>` | Covered by `cpu_process_mitigations_off` (same as the cpu row for 5458). |
| 1318, 1372, 1374, 1400, 1401 | `Session Manager\kernel` `DisableExceptionChainValidation`=1, `KernelSEHOPEnabled`=0 | SEHOP is one of the system mitigations covered by `cpu_process_mitigations_off`; `KernelSEHOPEnabled` is not a documented value. |
| 1331, 1376 | MM `EnableCfg`=0 | Not a documented value; system-wide CFG is covered by `cpu_process_mitigations_off`. |
| 1327, 1332, 1391 | MM `MoveImages`=0 | Turns off ASLR for system images (a security loss) with no measured performance gain; system ASLR is covered by `cpu_process_mitigations_off`. |
| 1313, 1378 | `Session Manager` `ProtectionMode`=0 | Weakens the security of shared kernel objects; no performance effect. |
| 1334 | `Session Manager\I/O System` `PassiveIntRealTimeWorkerPriority`=18 | Undocumented; effect cannot be verified. |
| 1335 | `Control\KernelVelocity` `DisableFGBoostDecay`=1 | Undocumented; effect cannot be verified. |
| 1337–1366 | IFEO `PerfOptions` `CpuPriorityClass`, `IoPriority`, `PagePriority` for `dwm.exe`, `lsass.exe`, `ntoskrnl.exe`, `SearchIndexer.exe`, `svchost.exe`, `TrustedInstaller.exe`, `wuauclt.exe`, `audiodg.exe` (also under `WOW6432Node`) | Harmful or no effect: Idle / Very Low priority for lsass, svchost and TrustedInstaller slows sign-in, services and updates (svchost: same as the game row for 1887); Realtime for dwm can starve the rest of the system; `ntoskrnl.exe` is not started as a process; `wuauclt.exe` is not used since Windows 10; SearchIndexer already runs at low priority; audiodg 2 (Normal) is the default; `WOW6432Node` IFEO keys only apply to 32-bit programs. |
| 1395–1398 | `Services\DXGKrnl` `MonitorLatencyTolerance`=0, `MonitorRefreshLatencyTolerance`=0 | Same as the gpu row for 2905–2908 (undocumented). |
| 1399, 1405 | `Session Manager\kernel` `DpcWatchdogProfileOffset`=0, `DpcWatchdogPeriod`=0 | Undocumented DPC watchdog values; the watchdog catches hung drivers and costs nothing while drivers behave; no performance gain. |
| 1402 | `Session Manager\kernel` `DisableAutoBoost`=0 | Windows default. |
| 1404 | `Session Manager\kernel` `ThreadDpcEnable`=1 | Windows default (threaded DPCs on). |
| 1406 | `Session Manager\kernel` `InterruptSteeringDisabled`=1 | Undocumented; effect cannot be verified. |
| 1437, 1439, 5489, 5491 | `lodctr /r` | Repair action; offered as the performance counter rebuild tool. |
| 1445 | `HKLM\…\Explorer\FlyoutMenuSettings` `ShowSleepOption`=0 | Hides Sleep in the power menu; no performance effect. |
| 1451 | `FSUTIL behavior set disablelastaccess 1` | Covered by `sto_last_access_off` (see `## storage`). |
| 1457, 5064 | `HKCU\…\Explorer\Serialize` `StartupDelayInMSec`=0 | Not dropped: kept as `perf_startup_delay_off` (one-click). |
| 449, 5172 | `HKCU\Control Panel\Desktop` `AutoEndTasks`="1" | Closes programs without asking at shutdown and sign-out, which loses unsaved work; no performance effect. |
| 485, 5437 | `HKLM\…\Schedule\Maintenance` `MaintenanceDisabled`=1 | Automatic Maintenance only runs while the PC is idle; turning it off also stops scheduled drive optimisation (TRIM on SSDs), update maintenance and security scans. |
| 5003–5008 | Policy `HKCU\…\Policies\Explorer` `NoLowDiskSpaceChecks`, `LinkResolveIgnoreLinkInfo`, `NoResolveSearch`, `NoResolveTrack`, `NoInternetOpenWith`, `NoInstrumentation`=1 | Low-disk warning, how broken shortcuts are repaired, the "look for an app online" prompt, and user-activity tracking; none affects performance in normal use (`NoInstrumentation` is a privacy value, left for the privacy category). |
| 5072 | `HKLM\…\Explorer` `Max Cached Icons`="1750" | Windows XP-era icon cache limit; no verified effect on Windows 10/11. |

## storage

Sources: `storage` (1920–1961, menu only), `fsutil` (1963–2003, menu only), `hdd` (2008–2019), `ssd` (2022–2030),
`frag` (2033–2041), the NTFS block of `ApplyAll1` (406–410), `AutoOpti1` (5444, 5445, 5534), the IO latency cap
loops of `Applyall2` (1412–1432), and the `ScheduledDefrag` task lines of `ServiceTweaks1` and `debloating`
(2277, 3715). fsutil values are written as their registry values under
`HKLM\SYSTEM\CurrentControlSet\Control\FileSystem` (no fsutil.exe when a registry/API equivalent exists).

| batch line | key/value | reason |
|---|---|---|
| 408, 1451, 5534 | `fsutil behavior set disablelastaccess 1` | Not dropped: kept as `sto_last_access_off` (one-click) = `NtfsDisableLastAccessUpdate`=0x80000001 (what fsutil writes; a plain 1 is reset to system-managed at boot, seen in the VM gate). |
| 2009 | `disableLastAccess 2` ("HDD") | System-managed, the Windows default. |
| 2023 | `disableLastAccess 0` ("SSD") | Turns last-access updates on (more writes), the opposite of the other lines; replaced by `sto_last_access_off`. |
| 2024 | `disable8dot3 1` ("SSD") | Not dropped: kept as `sto_8dot3_off` (caution) = `NtfsDisable8dot3NameCreation`=1. |
| 2010 | `disable8dot3 0` ("HDD") | Creates 8.3 names on every volume, more than the default (per volume); no benefit. |
| 406, 5445 | `fsutil behavior set memoryusage 2` | Not dropped: kept as `sto_ntfs_memory_usage` = `NtfsMemoryUsage`=2. |
| 5444 | `fsutil behavior query memoryusage` | Query only. |
| 407 | `fsutil behavior set mftzone 4` | Only reserves more room for the MFT on volumes that will hold millions of files; Windows grows the MFT as needed; no effect for games. |
| 409 | `fsutil behavior set disabledeletenotify 0` | Windows default (TRIM on). |
| 410 | `fsutil behavior set encryptpagingfile 0` | Windows default (off). |
| 2037, 2039 | Message box + `dfrgui` | Offered as a launcher; Windows already optimises drives on a schedule. |
| 2277, 3715 | `schtasks /Change /TN "Microsoft\Windows\Defrag\ScheduledDefrag" /Disable` | Harmful: this task also sends TRIM ("retrim") to SSDs; Windows never defragments SSDs, so disabling it only lets SSD write speed drop over time and leaves HDDs fragmented. Not ported in any category. |
| 1412–1432 | `IoLatencyCap`=0 on every service and `DriverDatabase` package that has the value | Not a documented value; the loop also edits the driver store database; it needs a registry search the catalog cannot express; effect cannot be verified. |

## usb

Sources: `usb` (2043–2085, menu only), `savingp` (2087–2116), `twkusb` (2118–2158), the USB device loop of
`ApplyAll1` (381–386) and the USB items of `AutoOpti1` (5044–5047, 5864–5868). USB power saving is also covered
elsewhere: NIC `*SelectiveSuspend` by `net_nic_power_saving_off`, USB 3 link power management by
`pow_usb3_link_power_off`. `usb_selective_suspend_off` sets the documented power-plan switch (USB settings >
USB selective suspend setting) that the batch's per-device values were aiming at.

| batch line | key/value | reason |
|---|---|---|
| 385 | Loop over present USB-class devices: `Device Parameters\WDF` `IdleInWorkingState`=0 | Not dropped: kept as `usb_hub_power_off` (present USB 3 (xHCI) controllers and hubs, whose KMDF drivers usbxhci/USBHUB3 read it; harmless on other USB-class devices). |
| 382–384 | Same loop: `SelectiveSuspendOn`=0, `AllowIdleIrpInD3`=0, `EnableSelectiveSuspend`=0 | Not read by the USB controller and hub drivers (the USB-class devices this loop visits); their idle switch is `IdleInWorkingState` (kept), and suspend of the devices behind them is the power-plan setting (`usb_selective_suspend_off`). These names belong to other drivers (HID, Bluetooth, some vendor drivers). |
| 2092–2111, 2133–2153, 5864–5868 | `Enum\%%a` / `%%s` / `%%i\Device Parameters` `SelectiveSuspendOn`, `SelectiveSuspendEnabled`, `EnhancedPowerManagementEnabled`, `AllowIdleIrpInD3`, `EnableSelectiveSuspend`, `DeviceSelectiveSuspended`, `D3ColdSupported`, `fid_D1Latency`, `fid_D2Latency`, `fid_D3Latency`, `WDF\IdleInWorkingState` | Loop variables used outside any `for` loop: reg.exe writes to a literal `Enum\%a` key that no device uses, so none of these ever applied; the intent is covered by `usb_selective_suspend_off` and `usb_hub_power_off`. `D3ColdSupported`=0 and the `fid_` latencies are undocumented. |
| 2123, 2124, 5044, 5045 | `Services\usbxhci\Parameters`, `Services\USBHUB3\Parameters` `ThreadPriority`=31 | Not a documented value of these drivers; no evidence they read it. |
| 2125, 5046 | `Services\nvlddmkm\Parameters` `ThreadPriority`=31 | Same as the game row for 1905, 1906. |
| 2126 | `Services\NDIS\Parameters` `ThreadPriority`=31 | Same as the network row for 5047. |
| 2127 | `Services\DXGKrnl\Parameters` `ThreadPriority`=15 | Same as the game row for 1905, 1906. |

## services

Sources: `ServiceTweaks` (2229–2260, menu only), `ServiceTweaks1` (2271–2319; it only disables scheduled tasks,
so its rows are under `## privacy`), the service and Windows Update items of `AutoOpti1` (4954–4969, 5350,
5355–5374, 5394–5413, 5506–5516, 5635–5667, 5703–5706). "Start=4" means the service is disabled.

| batch line | key/value | reason |
|---|---|---|
| 4954, 4955, 5705, 5706 | `Services\DiagTrack`, `Services\dmwappushservice` `Start`=4 | Not dropped: kept as `svc_telemetry_off` (one-click). |
| 4956–4960 | `Services\diagsvc`, `DPS`, `diagnosticshub.standardcollector.service`, `WdiServiceHost`, `WdiSystemHost` `Start`=4 | Not dropped: kept as `svc_diagnostics_off` (caution: troubleshooters, network diagnostics and app history stop working). |
| 4962 | "No" branch: `Services\WdiSystemHost` `Start`=4 | Copy-paste bug: answering "No" still disabled the service. |
| 4963–4969 | "No" branch: `DiagTrack`=2, `dmwappushservice`=3, `diagsvc`=3, `DPS`=2, `diagnosticshub.standardcollector.service`=3, `WdiServiceHost`=3, `WdiSystemHost`=3 | Windows defaults (the batch's way back); the app reverts from its backup instead. |
| 5355 | `Services\fhsvc` `Start`=4 | File History starts only when the user turned File History on; disabling it only breaks those backups. |
| 5357 | `Services\WbioSrvc` `Start`=4 | Starts on demand; disabling it breaks Windows Hello fingerprint and face sign-in. |
| 5358 | `Services\FontCache` `Start`=4 | Harmful: without the font cache service every program loads fonts itself, so programs start slower. |
| 5359 | `Services\FontCache3.0.0.0` `Start`=4 | WPF font cache, starts on demand; disabling it makes WPF programs start slower. |
| 5360 | `Services\GraphicsPerfSvc` `Start`=4 | Starts on demand only; no effect while idle. |
| 5372 | `Services\WerSvc` `Start`=4 | Starts only when an error report is made; error reporting itself is turned off by `prv_error_reporting_off`. |
| 5373 | `Services\PcaSvc` `Start`=4 | Program Compatibility Assistant applies compatibility fixes to old programs and installers (old games too); no performance effect. |
| 5374 | `Services\Wecsvc` `Start`=4 | Windows Event Collector starts on demand and is only used for event forwarding in managed networks; never runs on home PCs. |
| 5394, 5395 | `Services\Spooler`, `Services\PrintNotify` `Start`=4 | Not dropped: kept as `svc_print_spooler_off` (caution: printing and Print to PDF stop working). |
| 5396, 5397 | "No / Revert" branch: `Spooler`=2, `PrintNotify`=3 | Windows defaults (the batch's way back). |
| 5405 | `Services\MapsBroker` `Start`=4 | Downloaded Maps Manager only updates offline maps the user downloaded; idle otherwise. |
| 5406, 5407 | `Services\bthserv`, `Services\BTAGService` `Start`=4 | Bluetooth services start only when Bluetooth is used; disabling them breaks Bluetooth controllers, headsets and mice with no gain. |
| 5413 | `Services\GpuEnergyDrv` `Start`=4 | Same as the power row for 1159 (curated with the gpu category). |
| 5506, 5507, 5515, 5516 | `net stop` / `net start` `wuauserv`, `UsoSvc` | No "stop service" op and not needed: `cln_update_download` skips files that are in use. |
| 5508 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\WindowsUpdate` `DoNotConnectToWindowsUpdateInternetLocations`=1 | Without a WSUS server this breaks Windows Update and Microsoft Store downloads; even manual updates fail. |
| 5509 | Policy `…\WindowsUpdate` `SetDisableUXWUAccess`=1 | Removes access to the Windows Update settings page, so the user cannot even update by hand. |
| 5510 | Policy `…\WindowsUpdate\AU` `NoAutoUpdate`=1 | Not dropped: kept as `svc_auto_update_off` (caution, security note; manual updates still work). |
| 5511 | Policy `…\WindowsUpdate` `ExcludeWUDriversInQualityUpdate`=1 | Not dropped: kept as `svc_wu_drivers_off`. |
| 5350 | `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching` `SearchOrderConfig`=0 | Not dropped: merged into `svc_wu_drivers_off`. |
| 5512 | `gpupdate /force` | Not needed: Windows reads these policy values directly. |
| 5513, 5514 | `rd /s /q C:\Windows\SoftwareDistribution`, `md …` | Harmful: also deletes the update database and update history; replaced by `cln_update_download` (only the `Download` folder). |
| 5635–5638 | `sc config` `XboxGipSvc`, `XblAuthManager`, `XblGameSave`, `XboxNetApiSvc` disabled | Manual-start services that run only while the Xbox app or an Xbox / Game Pass game uses them; disabling gives no gain and breaks Xbox sign-in, cloud saves, Xbox networking and controller updates. |
| 5641, 5642 | `sc config` `SteelSeriesUpdateService`, `LGHUBUpdaterService` disabled | Third-party vendor services; disabling them can stop SteelSeries GG / Logitech G HUB from updating or starting. Turn off auto-update in the vendor app instead. |
| 5646, 5649 | `sc config` `SensrSvc`, `SensorService` disabled | Start on demand only; disabling breaks screen rotation, adaptive brightness and other sensors on laptops and tablets. |
| 5652 | `sc config WSearch start=disabled` | Covered by `perf_search_indexer_off` (caution). |
| 5655 | `sc config RemoteAccess start=disabled` | Windows default (Routing and Remote Access is already disabled). |
| 5658 | `sc config ssh-agent start=disabled` | Windows default (the OpenSSH agent is disabled by default). |
| 5661 | `sc config AppVClient start=disabled` | Windows default (the App-V client is disabled unless App-V is used). |
| 5664 | `sc config CertPropSvc start=disabled` | Starts on demand and is only used with smart cards; no effect. |
| 5667 | `sc config tzautoupdate start=disabled` | Windows default on desktops; it only runs when "Set time zone automatically" is on, which disabling breaks. |
| 5703 | `Services\OneSyncSvc` `Start`=4 | Sync Host only syncs Mail, Calendar and People for accounts added to those apps; no performance effect (removing the app is `prv_appx_mail_calendar`). |
| 5704 | `Services\OneSyncSvc_402ac` `Start`=4 | Per-user instance; the `_402ac` suffix is random per PC, so the line does nothing on other PCs. |

## privacy

Sources: `ApplyAll1` privacy, task and notification items (459–574), `ServiceTweaks1` (2271–2319), `debloating`
(3710–3822), `debloating2` (3829–3839), the privacy items of `AutoOpti1` (4840–4855, 4924–4941, 4984–4993,
5078–5085, 5351, 5419–5420) and the values earlier sections left for the privacy category (1241–1242,
1250–1251, 1880–1881, 5008). The category page is "Privacy & Debloat", so the app removals are here.
Task paths are under `\Microsoft\Windows\`. Appx lines are `Get-AppxPackage <pattern> | Remove-AppxPackage`;
the app matches the package family name (`<Name>_<publisher id>`) and never uses a bare `*`.

| batch line | key/value | reason |
|---|---|---|
| 460, 461, 464–467, 470–473, 2272, 2274–2276, 2278, 2279, 2281, 2286, 2301–2303, 2306, 2308, 2309, 3711–3714, 3716–3718, 3723, 3736–3738, 3741, 3743, 3744 | Tasks `Customer Experience Improvement Program\Consolidator`, `KernelCeipTask`, `UsbCeip`; `Application Experience\Microsoft Compatibility Appraiser`, `ProgramDataUpdater`; `Autochk\Proxy`; `DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector`; `Feedback\Siuf\DmClient`, `DmClientOnScenarioDownload`; `PI\Sqm-Tasks`; `Device Information\Device`, `Device User`; `Flighting\FeatureConfig\UsageDataReporting`, `UsageDataFlushing`; `NetTrace\GatherNetworkInfo`; `Power Efficiency Diagnostics\AnalyzeSystem` | Not dropped: merged into `prv_telemetry_tasks_off` (one-click). |
| 460, 462, 464, 466, 468, 470, 472, 474, 475, 477, 479, 481 | `schtasks /end` before disabling | Not needed: a disabled task does not start again, and a running one ends at the next restart at the latest. |
| 462, 463 | Task `Customer Experience Improvement Program\BthSQM` | Windows 8 task; not present on Windows 10/11. |
| 468, 469 | Task `Customer Experience Improvement Program\Uploader` | Windows 8 task; not present on Windows 10/11. |
| 474, 2273 | Task `Application Experience\StartupAppTask` (474 only ends it) | Only computes the "Startup impact" column in Task Manager; no effect on boot time. |
| 475–480 | Tasks `Shell\FamilySafetyMonitor`, `FamilySafetyRefresh`, `FamilySafetyUpload` | Needed for Microsoft Family parental controls; disabling them on a child account breaks those controls. |
| 481, 2296, 3731 | Task `Maintenance\WinSAT` | Runs a short hardware assessment only rarely, during idle maintenance; no effect while playing. |
| 2277, 3715 | Task `Defrag\ScheduledDefrag` | Covered by the storage row (harmful: also stops SSD TRIM). |
| 2280 | Task `DiskCleanup\SilentCleanup` | Runs Disk Cleanup only when the disk is low on space; harmless maintenance. |
| 2282, 3719 | Task `DiskFootprint\Diagnostics` | Storage usage diagnostics; no verified privacy or performance effect. |
| 2283, 3720 | Task `DiskFootprint\StorageSense` | Runs Storage Sense only when the user turned it on (see the cleanup row for 5607). |
| 2284, 3721 | Task `DUSM\dusmtask` | Data usage limits for metered connections; idle otherwise. |
| 2285, 3722 | Task `EnterpriseMgmt\MDMMaintenenceTask` | Only for PCs enrolled in work/school management (MDM); disabling it can break the enrollment. |
| 2287, 2311–2313, 3724, 3746 | Tasks `Input\TouchpadSyncDataAvailable`, `LocalUserSyncDataAvailable`, `MouseSyncDataAvailable`, `PenSyncDataAvailable` | Run only when input settings change, to sync them; no effect. |
| 2288, 3725 | Task `International\Synchronize Language Settings` | Syncs language settings and runs rarely; no effect. |
| 2289–2291, 3726–3728 | Tasks `LanguageComponentsInstaller\Installation`, `ReconcileLanguageResources`, `Uninstallation` | Harmful: breaks installing and removing language packs. |
| 2292, 2293, 3729, 3730 | Task `License Manager\TempSignedLicenseExchange` (written twice in each section) | Harmful: refreshes Microsoft Store app licences; disabling it can stop Store apps and games (Game Pass) from starting. |
| 2294, 2295 | Tasks `Management\Provisioning\Cellular`, `Logon` | Provisioning packages (OEM, enterprise, cellular); no effect on home PCs. |
| 2297, 2298, 3732, 3733 | Tasks `Maps\MapsToastTask`, `MapsUpdateTask` | Only for offline maps; idle otherwise. |
| 2299, 3734 | Task `Mobile Broadband Accounts\MNO Metadata Parser` | Only for cellular modems. |
| 2300, 3735 | Task `MUI\LPRemove` | Removes unused language packs to save space; harmless. |
| 2304, 3739 | Task `PushToInstall\Registration` | Needed to install Store apps from another device; idle otherwise. |
| 2305, 3740 | Task `Ras\MobilityManager` | VPN (IKEv2) mobility; idle without a VPN. |
| 2307, 3742 | Task `Flighting\FeatureConfig\ReconcileFeatures` | Applies Windows feature configuration (not telemetry); disabling it can leave features half-enabled after updates. |
| 2310, 3745 | Task `Flighting\OneSettings\RefreshCache` | Downloads Windows configuration (OneSettings) used by several components; the effect of disabling it is not documented. |
| 2314 | Task `SettingSync\NetworkStateChangeTask` | Windows 10 settings-sync helper; settings sync itself is turned off by `prv_settings_sync_off`. |
| 2315, 2316, 3747, 3748 | Tasks `Setup\SetupCleanupTask`, `SnapshotCleanupTask` | Clean up leftovers after Windows upgrades; disabling them leaves old files on the disk. |
| 2317, 2318, 3749, 3750 | Tasks `SpacePort\SpaceAgentTask`, `SpaceManagerTask` | Storage Spaces maintenance; idle without Storage Spaces, needed with it. |
| 2319, 3751 | Task `Speech\SpeechModelDownloadTask` | Downloads speech recognition models; disabling it only leaves them outdated. Online speech is turned off by `prv_speech_inking_off`. |
| 494, 505 | `HKCU\…\PushNotifications` `ToastEnabled`=0 (written twice) | Not dropped: kept as `prv_notifications_off` (caution: also hides Windows Security threat alerts). |
| 495, 496, 506, 507 | `HKCU\…\Notifications\Settings` `NOC_GLOBAL_SETTING_ALLOW_NOTIFICATION_SOUND`=0, `NOC_GLOBAL_SETTING_ALLOW_CRITICAL_TOASTS_ABOVE_LOCK`=0 | Covered: with notifications off there are no notification sounds or lock-screen toasts. |
| 497, 508 | `…\Notifications\Settings\QuietHours` `Enabled`=0 | No evidence Windows 10/11 read it (Focus assist / Do not disturb keeps its state elsewhere). |
| 498–503, 509–514 | Per-app `…\Notifications\Settings\<sender>` `Enabled`=0 (Settings, AutoPlay, LowDisk, Print.Notification, SecurityAndMaintenance, WiFiNetworkManager) | Covered by the global switch; `SecurityAndMaintenance` on its own also hides security warnings (for example "antivirus is off"). |
| 504, 515 | Policy `HKCU\SOFTWARE\Policies\Microsoft\Windows\Explorer` `DisableNotificationCenter`=1 | Removes the notification center (on Windows 11 also the notification part of the clock flyout); a UI removal that is not needed once notifications are off. |
| 5078 | Policy `HKCU\…\Policies\Microsoft\Windows\CurrentVersion\PushNotifications` `NoTileApplicationNotification`=1 | Only live tiles in the Windows 10 Start menu (Windows 11 has none); no performance effect. |
| 520 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Feeds` `EnableFeeds`=0 | Not dropped: kept as `prv_news_interests_off` (Windows 10 only). |
| 521 | `HKLM\SOFTWARE\Policies\Microsoft` `AllowNewsAndInterests`=0 | Wrong key (the policy lives under `…\Policies\Microsoft\Dsh`); the correct `Dsh` value was the `new_widgets_off` tweak, removed after the VM gate: on current Windows the UCPD driver denies writes to `Dsh\AllowNewsAndInterests` (other values under `Dsh` can be written), so the tweak always failed. Getting around UCPD means turning off a Windows protection, which the app does not do. |
| 522, 525, 4986, 4987 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` `EnableActivityFeed`=0 (written twice), `PublishUserActivities`=0, `UploadUserActivities`=0 | Not dropped: kept as `prv_activity_history_off`. |
| 4926 | `HKCU\…\Explorer\Advanced` `Start_TrackProgs`=0 | Not dropped: merged into `prv_activity_history_off`. |
| 5008 | Policy `HKCU\…\Policies\Explorer` `NoInstrumentation`=1 | Not dropped: merged into `prv_activity_history_off` (left here by the performance section). |
| 523, 4925 | `HKCU\Control Panel\International\User Profile` `HttpAcceptLanguageOptOut`=1 | Not dropped: merged into `prv_ads_suggestions_off`. |
| 524, 4924 | `HKCU\…\AdvertisingInfo` `Enabled`=0 | Not dropped: merged into `prv_ads_suggestions_off`. |
| 533 | Policy `HKLM\…\Policies\Microsoft\Windows\AdvertisingInfo` `DisabledByGroupPolicy`=1 | Not dropped: merged into `prv_ads_suggestions_off`. |
| 531 | `HKCU\…\Explorer\Advanced` `ShowSyncProviderNotifications`=0 | Not dropped: merged into `prv_ads_suggestions_off`. |
| 4931–4933 | `HKCU\…\ContentDeliveryManager` `SubscribedContent-338393Enabled`, `-353694Enabled`, `-353696Enabled`=0 | Not dropped: merged into `prv_ads_suggestions_off` (suggested content in Settings). |
| 4941 | `HKCU\…\Privacy` `TailoredExperiencesWithDiagnosticDataEnabled`=0 | Not dropped: merged into `prv_ads_suggestions_off`. |
| 532, 4988 | `HKLM\…\CapabilityAccessManager\ConsentStore\userNotificationListener` `Value`="Deny" | Not dropped: merged into `prv_app_access_off` (caution). |
| 4989–4991 | `HKCU\…\ConsentStore\location`, `appDiagnostics`, `userAccountInformation` `Value`="Deny" | Not dropped: merged into `prv_app_access_off` (caution). |
| 4992, 4993, 5419, 5420 | `HKCU\…\BackgroundAccessApplications` `GlobalUserDisabled`=1, `HKCU\…\Search` `BackgroundAppGlobalToggle`=0 | Not dropped: kept as `prv_background_apps_off` (one-click), plus the documented policy `HKLMSOFTWAREPoliciesMicrosoftWindowsAppPrivacy` `LetAppsRunInBackground`=2 so it also works on Windows 11. |
| 542, 543 | `HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\System` `AllowExperimentation`=0, `…\PolicyManager\default\System\AllowExperimentation` `value`=0 | Same as the performance row for 1241, 1242: MDM-internal stores that Windows rewrites; there is no Group Policy equivalent for home PCs. |
| 547–556, 4850–4855 | `HKCU\…\SettingSync` `SyncPolicy`=5 and `SettingSync\Groups\<group>` `Enabled`=0 (10 groups) | Not dropped: kept as `prv_settings_sync_off`. |
| 557–566 | `SettingSync\Groups\<group>` `Enabled`=0 | Duplicates of 547–556. |
| 570, 574 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting` `Disabled`=1, `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` `Disabled`=1 | Not dropped: kept as `prv_error_reporting_off`. |
| 571 | Policy `…\Windows Error Reporting` `DoReport`=0 | Not a value of this policy key (`DoReport` belongs to the older PCHealth policy, line 573); covered by `Disabled`=1. |
| 572 | Policy `…\Windows Error Reporting` `LoggingDisabled`=1 | Also stops crash entries in the event log, which makes troubleshooting harder; reporting is already off. |
| 573 | Policy `HKLM\SOFTWARE\Policies\Microsoft\PCHealth\ErrorReporting` `DoReport`=0 | Older "Configure Error Reporting" policy; redundant with the `Windows Error Reporting` `Disabled` policy. |
| 4934 | `HKCU\…\Speech_OneCore\Settings\OnlineSpeechPrivacy` `HasAccepted`=0 | Not dropped: merged into `prv_speech_inking_off`. |
| 4935–4938 | `HKCU\…\Personalization\Settings` `AcceptedPrivacyPolicy`=0, `InputPersonalization` `RestrictImplicitInkCollection`=1, `RestrictImplicitTextCollection`=1, `InputPersonalization\TrainedDataStore` `HarvestContacts`=0 | Not dropped: merged into `prv_speech_inking_off`. |
| 4939 | `HKCU\…\Diagnostics\DiagTrack` `ShowedToastAtLevel`=1 | Only records that the "diagnostic data level" notice was shown; no privacy effect. |
| 4940, 5084 | Policy `HKLM\SOFTWARE\Policies\Microsoft\Windows\DataCollection` `AllowTelemetry`=0 | Not dropped: kept as `prv_telemetry_off` (one-click; applied as "Required" on Home/Pro). |
| 5085 | Policy `…\DataCollection` `AllowDeviceNameInTelemetry`=0 | Not dropped: merged into `prv_telemetry_off`. |
| 1880, 1881 | Policy `SQMClient\Windows` `CEIPEnable`=0, `AppCompat` `AITEnable`=0 | Not dropped: merged into `prv_telemetry_off` (left here by the game section). |
| 4984, 4985 | `HKCU\SOFTWARE\Microsoft\Siuf\Rules` `NumberOfSIUFInPeriod`=0, delete `PeriodInNanoSeconds` | Not dropped: merged into `prv_telemetry_off` (feedback frequency "Never"). |
| 1250 | Policy `HKLM\…\Policies\Microsoft\Windows\OneDrive` `DisableFileSync`=1 | Windows 8.1 OneDrive policy; the current OneDrive client reads `DisableFileSyncNGSC`. |
| 1251 | Policy `…\OneDrive` `DisableFileSyncNGSC`=1 | Not dropped: kept as `prv_onedrive_off` (caution). The batch's OneDrive folder deletion stays disabled; uninstalling is the "Uninstall OneDrive" tool. |
| 4840, 5351 | `HKLM\…\Policies\System` `EnableLUA`=0 | Never ported: turns off UAC, so every program runs with full administrator rights and Store apps stop working; a large security loss with no performance effect. |
| 3834 | `debloating2`: Chris Titus WinUtil (`iwr … \| iex`) | Downloads and runs a remote script; the app does no network I/O and no downloads. |
| 3769–3772 | Appx `Microsoft.BingWeather`, `BingSports`, `BingNews`, `BingFinance` | Not dropped: kept as `prv_appx_bing` (caution: user-facing apps). |
| 3756 | Appx `*bing*` | Catch-all pattern that also matches other Bing packages; replaced by the four full names in `prv_appx_bing`. |
| 3752, 3753, 3757, 3759, 3760, 3764, 3783–3785, 3787, 3788, 3793, 3816, 3818 | Appx `3DBuilder`, `Getstarted` (twice), `MicrosoftOfficeHub`, `SkypeApp` (twice), `*solit*` / `MicrosoftSolitaireCollection`, `WindowsMaps`, `MixedReality.Portal`, `WindowsFeedbackHub`, `OneConnect`, `People`, `549981C3F5F10` (Cortana) | Not dropped: kept as `prv_appx_junk` (caution: includes user-facing apps such as Maps, Solitaire and People); loose patterns are replaced by full family names. |
| 3796–3806 | Appx `king.com.CandyCrushSaga`, `CandyCrushSodaSaga`, `Shazam`, `Flipboard`, `Twitter`, `iHeartRadio`, `Duolingo`, `AdobePhotoshopExpress`, `Pandora`, `EclipseManager`, `ActiproSoftwareLLC` | Not dropped: kept as `prv_appx_promo`. |
| 3762, 3790 | Appx `windowscommunicationsapps` (twice) | Not dropped: kept as `prv_appx_mail_calendar` (caution). |
| 3755 | Appx `WindowsCamera` | Not dropped: kept as `prv_appx_camera` (caution). |
| 3761, 3763, 3791, 3792, 3795, 3819 | Appx `WindowsSoundRecorder` (twice), `*zune*`, `ZuneMusic`, `ZuneVideo`, `Clipchamp` | Not dropped: kept as `prv_appx_media` (caution; ZuneMusic is the Windows 11 Media Player). |
| 3780 | Appx `Microsoft.YourPhone` | Not dropped: kept as `prv_appx_phone_link` (caution). |
| 3754, 3773, 3789, 3811, 3815 | Appx `WindowsAlarms`, `Office.OneNote`, `MicrosoftStickyNotes`, `Todos`, `PowerAutomateDesktop` | Not dropped: kept as `prv_appx_small_apps` (caution). |
| 3807 | Appx `SpotifyAB.SpotifyMusic` | An app the user installed or may use; uninstall it in Settings > Apps if unwanted. |
| 3758, 3775–3778 | Appx `*WindowsPhone*`, `Microsoft.WindowsPhone` (three times), `Microsoft.Windows.Phone` | Windows Phone / Windows 10 Mobile packages; not present on Windows 10 21H2+/11. |
| 3766, 3779 | Appx `CommsPhone` | Windows 10 Mobile era package; not present on current builds. |
| 3767, 3768, 3781, 3794 | Appx `ConnectivityStore`, `Microsoft.Messaging`, `Microsoft.Appconnector`, `Microsoft.GroupMe10` | Early Windows 10 packages that Windows 10 21H2+/11 no longer ship; nothing to remove. |
| 3765, 3774 | Appx `*Sway*`, `Microsoft.Office.Sway` | Sway was preinstalled only on early Windows 10 builds; nothing to remove. |
| 3786 | Appx `Microsoft.Wallet` | Removed from Windows 10 before 21H2; nothing to remove. |
| 3782, 3814 | Appx `Microsoft.GetHelp` (twice) | Windows 11 runs its troubleshooters through Get Help; removing it breaks them. |
| 3808 | Appx `Microsoft.Advertising.Xaml` | Framework package other apps depend on; removal fails while they are installed, or breaks them. |
| 3809, 3821, 3822 | Appx `Microsoft.RemoteDesktop` (twice, and once with one pinned version) | Remote Desktop client the user installed (not preinstalled); the user may need it. |
| 3810 | Appx `Microsoft.NetworkSpeedTest` | User-installed app, not preinstalled. |
| 3812 | Appx `*windowsterminal*` | Windows Terminal is the default command-line host on Windows 11 and a needed tool; no performance effect. |
| 3813 | Appx `Microsoft.ScreenSketch` | Snipping Tool; removing it breaks Win + Shift + S and Print Screen screenshots. |
| 3817 | Appx `*Microsoft.ParentalControls*` | Matches nothing: the package is `Microsoft.Windows.ParentalControls`, a system app that cannot be removed. |
| 3820 | Appx `RealtekSemiconductorCorp.RealtekAudioControl` | Harmful: removes Realtek Audio Console, which controls the audio driver's jack and effect settings. |

## cleanup

Sources: `cleanup` (2464–2478), `clean` (2421–2452, menu only), the update cache step of `AutoOpti1`
(5503–5516, see `## services`) and its Storage Sense question (5600–5608). `cln_windows_temp`
(`C:\Windows\Temp`) is not in the batch; it was added as a standard cleanup location.

| batch line | key/value | reason |
|---|---|---|
| 2465 | `del /s /f /q %temp%\*.*` | Not dropped: kept as `cln_user_temp` (one-click). |
| 2466 | `del … C:\Windows\Prefetch\*.*` | Harmful: Windows uses these files to start programs and boot faster; deleting them only makes the next starts slower until Windows rebuilds them. |
| 2467 | `del … C:\ProgramData\Microsoft\Windows\WER\ReportArchive\*.*` | No cleanup op for this folder; it holds only reports that were already sent (usually a few MB). Error reporting can be turned off with `prv_error_reporting_off`. |
| 2468 | `del … %AppData%\Microsoft\Windows\Recent\*.*` | Deletes the recent files list and jump lists; a privacy preference with no performance effect, and the catalog has no op for it. |
| 2469 | `del … %LocalAppData%\Microsoft\Windows\INetCache\*.*` | Cache of Internet Explorer and old web controls; current browsers keep their own cache; no op and little space. |
| 2470 | `del … %LocalAppData%\Microsoft\Windows\Explorer\thumbcache_*.db` | Not dropped: kept as `cln_thumbnail_cache`. |
| 2471 | `del … C:\Windows\SoftwareDistribution\Download\*.*` | Not dropped: kept as `cln_update_download`. |
| 2472 | `ipconfig /flushdns` | Repair action, not a cleanup; offered as the "DNS cache flush" tool. |
| 2473 | `del … C:\Windows\Logs\*.*` | Harmful: setup, update and servicing logs (CBS, DISM) needed to diagnose update failures; they take little space. |
| 2474 | `rd /s /q C:\$Recycle.Bin` | Deletes the Recycle Bin folders of every user on C: directly; replaced by `cln_recycle_bin` (caution), which empties the Recycle Bin through the Windows shell. |
| 2475 | `del … C:\Windows\SoftwareDistribution\DeliveryOptimization\*.*` | No cleanup op; Windows limits this cache itself and clears it in Settings > Storage / Disk Cleanup, and the service may be using the files. |
| 2476, 2477 | `del … C:\$Windows.~BT\*.*`, `C:\$Windows.~WS\*.*` | Windows upgrade staging folders; deleting them during or after an upgrade breaks the upgrade or the "go back" option. Use Disk Cleanup ("Previous Windows installations") instead. |
| 5607 | `HKCU\…\StorageSense\Parameters\StoragePolicy` `01`=0 | Storage Sense only runs when it is turned on and deletes the same kind of temporary files; turning it off is a preference with no performance effect. |
| 5608 | "No" branch: `StoragePolicy` `01`=1 | Turns Storage Sense on, a setting the user did not ask to change. |

## bcd

Sources: `bcd` (4759–4783), the BCD lines of `ApplyAll1` (398–402) and `AutoOpti1` (5463–5465, 5709–5746).
`numproc`, `allowedinmemorysettings` and `isolatedcontext` from `cpu1`/`cpu2` are under `## cpu`. `ExtrayDelay`
(5878–5885) only prints a success message and contains no values. The BCD tweaks are in `catalog/bcd.json`
with the category `performance`; they act on the `{current}` boot entry only.

| batch line | key/value | reason |
|---|---|---|
| 400, 4766, 5464, 5710 | `bcdedit /deletevalue useplatformclock` | Not dropped: kept as `perf_bcd_platform_clock_default` (removes a forced HPET clock; no change where the value is not set). |
| 398, 4767, 5463, 5713 | `bcdedit /set disabledynamictick yes` | Not dropped: kept as `perf_bcd_dynamic_tick_off` (caution: disputed effect, higher idle power). |
| 402, 5465, 5716 | `bcdedit /set useplatformtick yes` | Not dropped: kept as `perf_bcd_platform_tick_on` (caution: disputed effect). |
| 4765 | `bcdedit /deletevalue useplatformtick` | Contradicts lines 402, 5465 and 5716; deleting the value is the Windows default, which reverting `perf_bcd_platform_tick_on` restores. |
| 4780, 5731 | `bcdedit /set hypervisorlaunchtype off` | Not dropped: kept as `perf_bcd_hypervisor_off` (caution: VBS/HVCI, WSL 2, Hyper-V and Sandbox are lost). |
| 4783 | `bcdedit /set vsmlaunchtype off` | Not dropped: merged into `perf_bcd_hypervisor_off`. |
| 4761 | `bcdedit /set configaccesspolicy default` | Windows default. |
| 4762 | `bcdedit /set MSI default` | Windows default (MSI is set per device; see `gpu_msi_mode`). |
| 4763 | `bcdedit /set usephysicaldestination no` | APIC destination mode for the kernel; no documented performance benefit, and a wrong mode can break interrupt delivery. |
| 4764 | `bcdedit /set usefirmwarepcisettings no` | Windows default. |
| 4768 | `bcdedit /set tscsyncpolicy legacy` | Current CPUs use the default ("enhanced") TSC synchronization; forcing "legacy" has no shown benefit and can cause timer drift between cores. |
| 4769 | `bcdedit /set x2apicpolicy enable` | Windows already uses x2APIC where the firmware supports it; forcing it on unsupported systems can stop Windows from booting. |
| 4770, 4771 | `bcdedit /set ems no`, `bootems no` | Windows default (Emergency Management Services are off). |
| 4772 | `bcdedit /set vm no` | Not a documented boot setting for client Windows; no effect. |
| 4773 | `bcdedit /set sos no` | Windows default (driver names are not shown at boot). |
| 4774, 5737 | `bcdedit /set quietboot yes` | Cosmetic: hides the boot logo and progress; no faster boot, and it hides boot-time messages. |
| 4776, 5725 | `bcdedit /set bootux disabled` | Cosmetic: removes the boot animation; no faster boot. |
| 4777 | `bcdedit /set bootlog no` | Windows default (boot logging is off). |
| 4778, 5734 | `bcdedit /set tpmbootentropy ForceDisable` | Security loss (the boot random-number seed no longer uses the TPM) with no performance effect; the batch label "Disable TPM" is wrong. |
| 4781 | `bcdedit /set isolatedcontext no` | Undocumented VSM element; no shown effect (also the cpu row for 2682). |
| 5719 | `bcdedit /timeout 0` | Only matters with several boot entries (it hides the boot menu, so a second OS cannot be picked); it changes the `{bootmgr}` entry, which the app does not edit. |
| 5722 | `bcdedit /set nx optout` | The batch label "Disable nx" is wrong: OptOut turns DEP on for every program (the default OptIn covers only Windows components); no performance effect, and it can break old programs. |
| 5728 | `bcdedit /set bootmenupolicy standard` | Windows default ("Standard"); no faster boot. |
| 5740, 5743, 5746 | `bcdedit /set {globalsettings} custom:16000067`, `custom:16000069`, `custom:16000068 true` | Cosmetic (hide the boot logo, the spinning animation and boot messages, so boot errors are hidden too); they change `{globalsettings}`, which the app does not edit. |
