// Shared screen state: cached catalog/system info, the "operation running"
// lock and the restore point status shown on Home.
import {
  getCatalog,
  getSystemInfo,
  type SystemInfo,
  type TweakView,
} from "./api";

let catalog: TweakView[] | null = null;
let info: SystemInfo | null = null;
let dryRun = false;

export const setDryRun = (v: boolean): void => {
  dryRun = v;
};
export const isDryRunMode = (): boolean => dryRun;

export async function systemInfo(): Promise<SystemInfo> {
  info ??= await getSystemInfo();
  return info;
}

/**
 * Cached catalog. `force` re-fetches the live state, but never while an
 * apply/revert runs (the backend holds its lock for the whole operation).
 */
export async function catalogViews(force = false): Promise<TweakView[]> {
  if (catalog && (!force || busy)) return catalog;
  catalog = await getCatalog();
  return catalog;
}

// ---- busy lock -------------------------------------------------------------

let busy = false;
const idleWaiters: (() => void)[] = [];

export const isBusy = (): boolean => busy;

/** Mark a button as one that must be disabled while an operation runs. */
export function lockWhileBusy(btn: HTMLButtonElement): HTMLButtonElement {
  btn.dataset.busyLock = "";
  btn.disabled = busy || btn.dataset.blocked === "1";
  return btn;
}

/** Own (non-busy) disabled condition of a lockable button. */
export function setBlocked(btn: HTMLButtonElement, blocked: boolean): void {
  btn.dataset.blocked = blocked ? "1" : "";
  btn.disabled = busy || blocked;
}

function syncLocked(): void {
  document
    .querySelectorAll<HTMLButtonElement>("[data-busy-lock]")
    .forEach((b) => (b.disabled = busy || b.dataset.blocked === "1"));
  document.body.classList.toggle("is-busy", busy);
}

export function setBusy(v: boolean): void {
  busy = v;
  syncLocked();
  if (!v) idleWaiters.splice(0).forEach((cb) => cb());
}

/** Run `cb` now if idle, otherwise once the running operation finishes. */
export function whenIdle(cb: () => void): void {
  if (busy) idleWaiters.push(cb);
  else cb();
}

// ---- restore point status (Home pill) ----------------------------------------

/** ready = will be created before the first change of this run. */
export type RestorePointStatus = "ready" | "creating" | "created" | "failed";
let rpStatus: RestorePointStatus = "ready";
export const restorePointStatus = (): RestorePointStatus => rpStatus;
export const setRestorePointStatus = (s: RestorePointStatus): void => {
  rpStatus = s;
};
