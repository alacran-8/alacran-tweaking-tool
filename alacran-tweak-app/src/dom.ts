// Tiny DOM builder. Text always goes through textContent (catalog strings are
// never parsed as HTML); `html` is only for the app's own icon markup.
import { icon } from "./icons";

type Child = Node | string | null | undefined | false;

export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  cls = "",
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  for (const c of children) {
    if (c === null || c === undefined || c === false) continue;
    e.append(typeof c === "string" ? document.createTextNode(c) : c);
  }
  return e;
}

/** Icon element from the app's icon set. */
export function ic(name: string, cls = "ic"): HTMLElement {
  const s = document.createElement("span");
  s.className = "ic-wrap";
  s.innerHTML = icon(name, cls);
  return s;
}

export function button(
  cls: string,
  label: Child | Child[],
  onClick: (e: MouseEvent) => void,
): HTMLButtonElement {
  const b = h("button", cls, ...(Array.isArray(label) ? label : [label]));
  b.type = "button";
  b.addEventListener("click", onClick);
  return b;
}

export const reducedMotion = (): boolean =>
  window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
