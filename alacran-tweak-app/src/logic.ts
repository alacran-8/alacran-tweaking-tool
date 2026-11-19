// Pure screen logic (no DOM, no backend): unit-tested in logic.test.ts.
import type { Selection, SystemInfo, Tweak, TweakView } from "./api";

/** Every category id, in sidebar order. */
export const CATEGORY_IDS = [
  "network",
  "power",
  "cpu",
  "gpu",
  "game",
  "input",
  "visual",
  "performance",
  "storage",
  "usb",
  "services",
  "privacy",
  "cleanup",
] as const;
export type CategoryId = (typeof CATEGORY_IDS)[number];

/** Categories with their own card on Home. */
export const HOME_CATEGORIES: CategoryId[] = [
  "network",
  "power",
  "game",
  "gpu",
  "cpu",
  "input",
  "cleanup",
  "privacy",
];

/** Categories behind the Home "Other settings" card. */
export const OTHER_CATEGORIES: CategoryId[] = [
  "visual",
  "performance",
  "storage",
  "usb",
  "services",
];

export const isCategoryId = (s: string): s is CategoryId =>
  (CATEGORY_IDS as readonly string[]).includes(s);

/** Categories that contain at least one tweak, in sidebar order. */
export function categoriesWithTweaks(views: TweakView[]): CategoryId[] {
  const present = new Set(views.map((v) => v.category));
  return CATEGORY_IDS.filter((c) => present.has(c));
}

/** First "other" category that has tweaks (falls back to the first one). */
export function otherSettingsTarget(views: TweakView[]): CategoryId {
  const present = new Set(views.map((v) => v.category));
  return OTHER_CATEGORIES.find((c) => present.has(c)) ?? OTHER_CATEGORIES[0];
}

/** Tweaks shown on a category page; GPU tweaks of absent vendors are hidden. */
export function visibleTweaks(
  views: TweakView[],
  category: string,
  info: SystemInfo,
): TweakView[] {
  return views.filter(
    (v) =>
      v.category === category &&
      (v.vendor === null || info.gpu_vendors.includes(v.vendor)),
  );
}

/** Same rule as the engine's one_click_selection. */
export function oneClickTweaks(views: TweakView[], info: SystemInfo): TweakView[] {
  return views.filter(
    (v) =>
      v.in_one_click && v.available && !(v.laptop === false && info.is_laptop),
  );
}

function firstNumber(s: string): number | null {
  const m = /\d+/.exec(s);
  return m ? Number(m[0]) : null;
}

/**
 * `auto_option: "ram_gb"`: option ids carry a GB number (e.g. "16gb"); picks the
 * smallest one not below the installed RAM, else the largest (svchost splits only
 * when RAM is above the threshold, so the threshold must be >= RAM).
 */
function autoOptionId(t: Tweak, info: SystemInfo): string | null {
  if (t.auto_option !== "ram_gb") return null;
  const sized = t.options
    .map((o) => ({ id: o.id, gb: firstNumber(o.id) }))
    .filter((o): o is { id: string; gb: number } => o.gb !== null)
    .sort((a, b) => a.gb - b.gb);
  if (!sized.length) return null;
  const fit = sized.find((o) => o.gb >= info.ram_gb);
  return (fit ?? sized[sized.length - 1]).id;
}

/** Option preselected in the row / sent by one-click; null = no options. */
export function chosenOptionId(t: Tweak, info: SystemInfo): string | null {
  if (!t.options.length) return null;
  return autoOptionId(t, info) ?? t.default_option ?? t.options[0].id;
}

export const toSelection = (t: Tweak, info: SystemInfo): Selection => ({
  tweak_id: t.id,
  option_id: chosenOptionId(t, info),
});

export type Tag = "recommended" | "new" | "caution" | "reboot" | "one_way";

/** ÖNERİLEN = part of the one-click set and usable on this system. */
export const isRecommended = (v: TweakView, isLaptop = false): boolean =>
  v.in_one_click && v.available && !(v.laptop === false && isLaptop);

export function tagsFor(v: TweakView, isLaptop = false): Tag[] {
  const tags: Tag[] = [];
  if (isRecommended(v, isLaptop)) tags.push("recommended");
  if (v.new) tags.push("new");
  if (v.risk === "caution") tags.push("caution");
  if (v.reboot !== "none") tags.push("reboot");
  if (v.one_way) tags.push("one_way");
  return tags;
}

/** "Kısmen uygulandı (x/y)": x of y checkable values already match; null unless partial. */
export function partialInfo(v: TweakView): { x: number; y: number; rest: number } | null {
  if (v.state !== "partial" || v.total <= 0) return null;
  return { x: v.matched, y: v.total, rest: v.total - v.matched };
}

/** How many of `want` new orb sparks may spawn with `alive` on screen (GPU cap). */
export const sparkRoom = (alive: number, want: number, cap = 40): number =>
  Math.max(0, Math.min(want, cap - alive));

/** What turning a tweak on must warn about first, or null when no confirm is needed. */
export function turnOnWarning(t: Tweak): { caution: boolean; oneWay: boolean } | null {
  const caution = t.risk === "caution";
  return caution || t.one_way ? { caution, oneWay: t.one_way } : null;
}

export interface RowChoice {
  id: string;
  /** Toggle state when the page was rendered (on = already applied). */
  initialOn: boolean;
  on: boolean;
  initialOpt: string | null;
  opt: string | null;
}

/** Rows to apply: newly switched on, or applied rows with another option. */
export function pendingSelections(rows: RowChoice[]): Selection[] {
  return rows
    .filter((r) => r.on && (!r.initialOn || r.opt !== r.initialOpt))
    .map((r) => ({ tweak_id: r.id, option_id: r.opt }));
}

export function formatMB(bytes: number): string {
  const mb = bytes / (1024 * 1024);
  if (mb === 0) return "0";
  return mb < 10 ? String(Math.round(mb * 10) / 10) : String(Math.round(mb));
}

export function windowsLabel(info: SystemInfo): string {
  const name = info.build >= 22000 ? "Windows 11" : "Windows 10";
  return info.display_version ? `${name} · ${info.display_version}` : name;
}
