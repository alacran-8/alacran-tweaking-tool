import { getCurrentWindow } from "@tauri-apps/api/window";
import { getLang, setLang, t } from "../i18n";
import { setBackgroundPaused } from "../bg/tubes";

const inTauri = () => "__TAURI_INTERNALS__" in window;

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  cls: string,
  drag = false,
): HTMLElementTagNameMap[K] {
  const e = document.createElement(tag);
  e.className = cls;
  if (drag) e.setAttribute("data-tauri-drag-region", "");
  return e;
}

export interface Titlebar {
  render(): void;
}

export function mountTitlebar(host: HTMLElement, dryRun: boolean): Titlebar {
  function render() {
    const bar = el("div", "titlebar", true);
    const logo = el("div", "logo", true);
    const title = el("b", "tb-title", true);
    title.textContent = t("titlebar.title");
    const spacer = el("div", "tb-spacer", true);
    bar.append(logo, title, spacer);

    if (dryRun) {
      const pill = el("span", "pill pill-warn", true);
      pill.textContent = t("titlebar.dryRun");
      pill.title = t("titlebar.dryRunHint");
      bar.append(pill);
    }

    const langBox = el("div", "lang");
    langBox.setAttribute("role", "group");
    langBox.setAttribute("aria-label", t("titlebar.language"));
    (["tr", "en"] as const).forEach((l) => {
      const b = el("button", l === getLang() ? "lang-btn on" : "lang-btn");
      b.setAttribute("aria-pressed", String(l === getLang()));
      b.textContent = l.toUpperCase();
      b.addEventListener("click", () => void setLang(l));
      langBox.append(b);
    });
    bar.append(langBox);

    const min = el("button", "win-btn");
    min.textContent = "–";
    min.title = t("titlebar.minimize");
    min.setAttribute("aria-label", t("titlebar.minimize"));
    min.addEventListener("click", () => {
      if (!inTauri()) return;
      // Pause right away; tubes.ts resumes on visibilitychange/focus.
      setBackgroundPaused(true);
      getCurrentWindow()
        .minimize()
        .catch(() => setBackgroundPaused(document.hidden)); // minimise failed
    });
    const close = el("button", "win-btn win-close");
    close.textContent = "✕";
    close.title = t("titlebar.close");
    close.setAttribute("aria-label", t("titlebar.close"));
    close.addEventListener("click", () => {
      if (inTauri()) void getCurrentWindow().close();
    });
    bar.append(min, close);

    host.replaceChildren(bar);
  }
  render();
  return { render };
}
