// Typed wrappers over the Tauri commands. Interfaces mirror the Rust serde
// output (snake_case field names, as serialized).
import { invoke } from "@tauri-apps/api/core";

export type Lang = "tr" | "en";
export type Hive = "hklm" | "hkcu";
export type Risk = "safe" | "caution";
export type Reboot = "none" | "recommended" | "required";
export type Vendor = "nvidia" | "amd" | "intel";
export type DeviceClass = "usb" | "net" | "display";
export type CleanupTarget =
  | "user_temp"
  | "windows_temp"
  | "update_download"
  | "thumb_cache"
  | "recycle_bin";
export type TweakState = "applied" | "partial" | "not_applied" | "unknown";

export type RegData =
  | { type: "dword"; value: number }
  | { type: "qword"; value: number }
  | { type: "sz"; value: string }
  | { type: "expand_sz"; value: string }
  | { type: "multi_sz"; value: string[] }
  | { type: "binary"; value: string };

export type Op =
  | { op: "reg_set"; hive: Hive; path: string; name: string; data: RegData }
  | { op: "reg_delete_value"; hive: Hive; path: string; name: string }
  | { op: "service_start"; name: string; start: number }
  | { op: "task_disable"; path: string }
  | {
      op: "power_value";
      subgroup: string;
      setting: string;
      ac: number | null;
      dc: number | null;
    }
  | { op: "power_scheme_duplicate_activate"; source: string }
  | { op: "bcd_set"; name: string; value: string }
  | { op: "bcd_delete"; name: string }
  | { op: "appx_remove"; pattern: string }
  | { op: "cleanup"; target: CleanupTarget }
  | {
      op: "device_reg";
      class: DeviceClass;
      suffix: string;
      name: string;
      data: RegData;
    }
  | { op: "nic_property"; keyword: string; value: string }
  | { op: "dx_global_setting"; key: string; value: string };

export interface Text {
  tr: string;
  en: string;
}

export interface TweakOption {
  id: string;
  label: Text;
  ops: Op[];
}

export interface Tweak {
  id: string;
  category: string;
  title: Text;
  description: Text;
  risk: Risk;
  in_one_click: boolean;
  new: boolean;
  reboot: Reboot;
  one_way: boolean;
  min_build: number;
  max_build: number | null;
  vendor: Vendor | null;
  laptop: boolean | null;
  caution_note: Text | null;
  ops: Op[];
  options: TweakOption[];
  default_option: string | null;
  auto_option: string | null;
}

/** Tweak plus live state; `available` = passes the build/vendor filter. */
export interface TweakView extends Tweak {
  state: TweakState;
  available: boolean;
  /** Checkable values already at the target / all checkable values. */
  matched: number;
  total: number;
}

export interface SystemInfo {
  build: number;
  display_version: string;
  edition: string;
  gpu_vendors: Vendor[];
  ram_gb: number;
  is_laptop: boolean;
  interactive_sid: string;
}

export interface Selection {
  tweak_id: string;
  /** Omit/null = the tweak's default option. */
  option_id?: string | null;
}

export interface ApplyReport {
  session_id: string;
  applied: number;
  skipped: number;
  /** [tweak id, error] */
  failed: [string, string][];
  one_way: number;
  reboot: Reboot;
  bytes_freed: number;
}

export interface RevertReport {
  reverted: number;
  skipped_newer: string[];
  /** [target, error] */
  failed: [string, string][];
}

export interface SessionSummary {
  id: string;
  created: string;
  count: number;
  reverted: boolean;
  /** Only the restore point frequency setting (the app's own hidden tweak). */
  internal: boolean;
}

export interface SessionList {
  sessions: SessionSummary[];
  /** [file name, error] of unreadable backup files. */
  broken: [string, string][];
}

export type ToolId =
  | "sfc"
  | "dism"
  | "dns_flush"
  | "winsock_reset"
  | "perf_counters"
  | "one_drive_uninstall"
  | "defrag_gui"
  | "perf_options_gui"
  | "mouse_cpl"
  | "keyboard_cpl"
  | "system_restore_gui";

export interface Settings {
  lang: Lang;
  animated_bg: boolean;
}

export type CommandErrorCode =
  | "system_protection_off"
  | "restore_point_failed"
  | "link_not_allowed"
  | "backup_failed"
  | "internal";

/** Every rejected command promise carries this shape. */
export interface CommandError {
  code: CommandErrorCode | (string & {});
  message: string;
}

export function isCommandError(e: unknown): e is CommandError {
  return (
    typeof e === "object" &&
    e !== null &&
    typeof (e as CommandError).code === "string" &&
    typeof (e as CommandError).message === "string"
  );
}

export const isDryRun = () => invoke<boolean>("is_dry_run");
export const getSystemInfo = () => invoke<SystemInfo>("get_system_info");
export const getCatalog = () => invoke<TweakView[]>("get_catalog");
export const applyTweaks = (selections: Selection[]) =>
  invoke<ApplyReport>("apply_tweaks", { selections });
export const applyTweaksForce = (selections: Selection[]) =>
  invoke<ApplyReport>("apply_tweaks_force", { selections });
export const enableProtection = () => invoke<void>("enable_protection");
export const listSessions = () => invoke<SessionList>("list_sessions");
export const revertSession = (id: string) =>
  invoke<RevertReport>("revert_session", { id });
export const revertAll = () => invoke<RevertReport>("revert_all");
export const revertCategory = (category: string) =>
  invoke<RevertReport>("revert_category", { category });
export const runTool = (id: ToolId) => invoke<void>("run_tool", { id });
export const openLink = (url: string) => invoke<void>("open_link", { url });
export const getSettings = () => invoke<Settings>("get_settings");
export const setSettings = (settings: Settings) =>
  invoke<void>("set_settings", { settings });
