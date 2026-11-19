// Short status message in the bottom-right corner.
import { h, ic } from "../dom";

let host: HTMLElement | null = null;

export function toast(text: string, kind: "ok" | "err" = "ok"): void {
  if (!host || !host.isConnected) {
    host = h("div", "toasts");
    host.setAttribute("role", "status");
    host.setAttribute("aria-live", "polite");
    document.body.appendChild(host);
  }
  const el = h("div", `toast toast-${kind}`, ic(kind === "ok" ? "check" : "warn"), h("span", "", text));
  host.appendChild(el);
  window.setTimeout(() => {
    el.classList.add("leaving");
    window.setTimeout(() => el.remove(), 220);
  }, 2600);
}
