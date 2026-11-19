// Minimal i18n: nested JSON strings, `{name}` placeholders, TR default.
import tr from "./i18n/tr.json";
import en from "./i18n/en.json";
import { getSettings, setSettings, type Lang, type Text } from "./api";

type Tree = { [k: string]: string | Tree };

const dicts: Record<Lang, Tree> = { tr, en };
let lang: Lang = "tr";
const warned = new Set<string>();
const listeners = new Set<(l: Lang) => void>();

function lookup(tree: Tree, key: string): string | undefined {
  let node: string | Tree | undefined = tree;
  for (const part of key.split(".")) {
    if (typeof node !== "object" || node === null) return undefined;
    node = node[part];
  }
  return typeof node === "string" ? node : undefined;
}

export function t(key: string, vars?: Record<string, string | number>): string {
  const raw = lookup(dicts[lang], key);
  if (raw === undefined) {
    if (import.meta.env?.DEV && !warned.has(key)) {
      warned.add(key);
      console.warn(`[i18n] missing key: ${key}`);
    }
    return key;
  }
  if (!vars) return raw;
  return raw.replace(/\{(\w+)\}/g, (m, name: string) =>
    name in vars ? String(vars[name]) : m,
  );
}

export const getLang = (): Lang => lang;

/** Pick the active language from a catalog `Text`. */
export const tx = (text: Text): string => text[lang] || text.tr;

/** Set the active language without persisting (startup). */
export function initLang(l: Lang): void {
  lang = l;
  document.documentElement.lang = l;
}

/** Switch language, persist via settings, notify subscribers. */
export async function setLang(l: Lang): Promise<void> {
  if (l === lang) return;
  initLang(l);
  listeners.forEach((cb) => cb(l));
  try {
    const s = await getSettings();
    await setSettings({ ...s, lang: l });
  } catch (e) {
    console.warn("[i18n] could not persist language", e);
  }
}

export function onLangChange(cb: (l: Lang) => void): void {
  listeners.add(cb);
}

export const _dicts = dicts;
