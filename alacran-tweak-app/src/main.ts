import "@fontsource/figtree/400.css";
import "@fontsource/figtree/600.css";
import "@fontsource/figtree/800.css";
import "@fontsource/figtree/900.css";
import "./styles/app.css";
import { getSettings, isDryRun, type Settings } from "./api";
import { initLang, onLangChange } from "./i18n";
import { setDryRun, whenIdle } from "./data";
import { renderHome } from "./screens/home";
import { renderCategory } from "./screens/category";
import { renderRestore } from "./screens/restore";
import { renderContact } from "./screens/contact";
import { renderAbout } from "./screens/about";
import { renderTools } from "./screens/tools";
import * as router from "./router";
import { initBackground } from "./bg/tubes";
import { mountTitlebar } from "./components/titlebar";

const DEFAULT_SETTINGS: Settings = { lang: "tr", animated_bg: true };

async function safe<T>(p: () => Promise<T>, fallback: T): Promise<T> {
  try {
    return await p();
  } catch (e) {
    console.warn("[main] backend call failed (no Tauri?)", e);
    return fallback;
  }
}

router.register("home", renderHome);
router.register("category", renderCategory);
router.register("restore", renderRestore);
router.register("contact", renderContact);
router.register("about", renderAbout);
router.register("tools", renderTools);

async function main() {
  // Browser preview without Tauri (npm run dev): fake backend. Dead code in
  // production builds, so the mock module is never bundled.
  if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
    await import("./dev/mock").then((m) => m.installMock());
  }
  const [settings, dryRun] = await Promise.all([
    safe(getSettings, DEFAULT_SETTINGS),
    safe(isDryRun, false),
  ]);
  initLang(settings.lang);
  setDryRun(dryRun);

  const app = document.querySelector<HTMLDivElement>("#app")!;
  app.innerHTML =
    '<div id="bg" aria-hidden="true"></div>' +
    '<div id="titlebar-host"></div>' +
    '<main id="view"></main>';

  initBackground(app.querySelector<HTMLElement>("#bg")!, settings.animated_bg);
  const titlebar = mountTitlebar(
    app.querySelector<HTMLElement>("#titlebar-host")!,
    dryRun,
  );
  router.mount(app.querySelector<HTMLElement>("#view")!);

  onLangChange(() => {
    titlebar.render();
    // Never repaint a page while an apply/revert is running.
    whenIdle(() => void router.refresh());
  });

  // DEV preview only: ?route=restore opens a screen directly (screenshots).
  const start = import.meta.env.DEV
    ? new URLSearchParams(location.search).get("route")
    : null;
  await router.go((start ?? "home") as router.Route);
}

void main();
