// Sidebar + content layout shared by every screen except Home.
import type { TweakView } from "../api";
import { h, ic } from "../dom";
import { t } from "../i18n";
import { categoriesWithTweaks, CATEGORY_IDS, isCategoryId } from "../logic";
import * as router from "../router";

function navItem(route: router.Route, iconName: string, label: string, active: boolean) {
  const b = h("button", active ? "nav on" : "nav", ic(iconName), h("span", "", label));
  b.type = "button";
  if (active) b.setAttribute("aria-current", "page");
  b.addEventListener("click", () => {
    if (!active) void router.go(route);
  });
  return b;
}

/**
 * Renders the sidebar into `el` and returns the content panel.
 * `active` is a route ("restore", "category/network", ...).
 */
export function withSidebar(el: HTMLElement, views: TweakView[], active: string): HTMLElement {
  const cats = categoriesWithTweaks(views);
  const activeCat = active.startsWith("category/") ? active.slice(9) : "";
  // A category opened from Home that has no tweaks yet still shows as current.
  if (isCategoryId(activeCat) && !cats.includes(activeCat)) {
    cats.push(activeCat);
    cats.sort((a, b) => CATEGORY_IDS.indexOf(a) - CATEGORY_IDS.indexOf(b));
  }
  const top = h(
    "nav",
    "side-group",
    navItem("home", "home", t("nav.home"), false),
    ...cats.map((c) => navItem(`category/${c}`, c, t(`cat.${c}`), activeCat === c)),
  );
  const bottom = h(
    "div",
    "side-group side-bottom",
    navItem("tools", "tools", t("nav.tools"), active === "tools"),
    navItem("restore", "restore", t("nav.restore"), active === "restore"),
    navItem("contact", "contact", t("nav.contact"), active === "contact"),
    navItem("about", "about", t("nav.about"), active === "about"),
  );
  const side = h("aside", "side glass", top, bottom);
  side.setAttribute("aria-label", t("nav.label"));
  const main = h("section", "main glass");
  el.classList.add("with-side");
  el.append(side, main);
  return main;
}

/** Page heading with an optional element on the right. */
export function pageHead(iconName: string, title: string, sub?: string, right?: Node): HTMLElement {
  return h(
    "header",
    "page-head",
    h("div", "head-ic", ic(iconName)),
    h("div", "head-tx", h("h2", "", title), sub ? h("p", "", sub) : null),
    right ?? null,
  );
}
