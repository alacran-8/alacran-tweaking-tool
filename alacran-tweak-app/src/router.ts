// Tiny in-memory router. Screens register by route prefix:
//   register("category", (el, id) => ...)  handles "category/<id>".
import { t } from "./i18n";

export type Route =
  | "home"
  | `category/${string}`
  | "restore"
  | "contact"
  | "about"
  | "tools";

export type Render = (el: HTMLElement, param?: string) => void | Promise<void>;

const screens = new Map<string, Render>();
let container: HTMLElement | null = null;
let currentRoute: Route = "home";
let token = 0;
const listeners = new Set<(r: Route) => void>();

export function register(prefix: string, render: Render): void {
  screens.set(prefix, render);
}

export function mount(el: HTMLElement): void {
  container = el;
}

export const current = (): Route => currentRoute;

export function onRouteChange(cb: (r: Route) => void): void {
  listeners.add(cb);
}

async function paint(route: Route): Promise<void> {
  if (!container) return;
  const my = ++token;
  const slash = route.indexOf("/");
  const prefix = slash < 0 ? route : route.slice(0, slash);
  const param = slash < 0 ? undefined : route.slice(slash + 1);
  const render = screens.get(prefix) ?? screens.get("home");
  const next = document.createElement("div");
  next.className = "view-page"; // CSS: 200 ms fade/slide-in
  try {
    if (render) await render(next, param);
  } catch (e) {
    console.error("[router] render failed", route, e);
    next.textContent = t("common.error");
  }
  if (my !== token) return; // a newer navigation won
  container.replaceChildren(next);
}

export async function go(route: Route): Promise<void> {
  currentRoute = route;
  listeners.forEach((cb) => cb(route));
  await paint(route);
}

/** Re-render the current route (e.g. after a language change). */
export const refresh = (): Promise<void> => paint(currentRoute);
