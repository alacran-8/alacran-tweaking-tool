// Home: status pills, OPTİMİZE ET orb (one-click flow), category cards, CTA.
import { openLink, type SystemInfo, type TweakView } from "../api";
import { errorText, runApply } from "../apply";
import { openSheet } from "../components/sheet";
import { toast } from "../components/toast";
import { catalogViews, lockWhileBusy, restorePointStatus, systemInfo } from "../data";
import { button, h, ic, reducedMotion } from "../dom";
import { t, tx } from "../i18n";
import {
  HOME_CATEGORIES,
  oneClickTweaks,
  OTHER_CATEGORIES,
  otherSettingsTarget,
  sparkRoom,
  toSelection,
  visibleTweaks,
  windowsLabel,
} from "../logic";
import * as router from "../router";

const KICK = "https://kick.com/alacranfps";

/** One spark flying from the orb centre to (x, y) px; removed when done. */
function spark(stage: HTMLElement, x: number, y: number, ms: number): void {
  const s = h("span", "orb-spark");
  s.style.setProperty("--x", `${x}px`);
  s.style.setProperty("--y", `${y}px`);
  s.style.animationDuration = `${ms}ms`;
  stage.appendChild(s);
  window.setTimeout(() => s.remove(), ms + 100);
}

const aliveSparks = (stage: HTMLElement) => stage.querySelectorAll(".orb-spark").length;

/** Continuous drift outwards while the page shows and the window is visible. */
function driftSparks(stage: HTMLElement): void {
  const timer = window.setInterval(() => {
    if (!stage.isConnected) return window.clearInterval(timer); // page left
    if (document.hidden || reducedMotion() || !sparkRoom(aliveSparks(stage), 1)) return;
    const a = Math.random() * Math.PI * 2;
    const r = 110 + Math.random() * 40;
    spark(stage, Math.cos(a) * r, Math.sin(a) * r, 2800);
  }, 180);
}

/** Click: a radial ring of sparks (within the shared cap). */
function burstSparks(stage: HTMLElement): void {
  if (reducedMotion()) return;
  const n = sparkRoom(aliveSparks(stage), 24);
  for (let i = 0; i < n; i++) {
    const a = (i / n) * Math.PI * 2;
    spark(stage, Math.cos(a) * 170, Math.sin(a) * 170, 1000);
  }
}

function pills(info: SystemInfo): HTMLElement {
  const rp = restorePointStatus();
  return h(
    "div",
    "stat",
    h(
      "span",
      "pill",
      ic("window"),
      windowsLabel(info),
      h("span", "pill-dim", ` · ${t("home.build", { build: info.build })}`),
    ),
    h(
      "span",
      "pill",
      ic("shieldCheck"),
      `${t("home.rpLabel")} `,
      h("b", `rp-${rp}`, t(`home.rp.${rp}`)),
    ),
  );
}

function card(iconName: string, label: string, count: number | null, go: () => void, extra = "") {
  const c = button(
    `card ${extra}`.trim(),
    [
      h("span", "card-ic", ic(iconName)),
      h("span", "card-label", label),
      count === null ? null : h("span", "card-count", t("home.count", { count })),
    ],
    go,
  );
  // Cursor-following spotlight (CSS reads --mx / --my).
  c.addEventListener("mousemove", (e) => {
    const r = c.getBoundingClientRect();
    c.style.setProperty("--mx", `${e.clientX - r.left}px`);
    c.style.setProperty("--my", `${e.clientY - r.top}px`);
  });
  return c;
}

/** Confirm sheet: count + expandable list (one-way marked), then apply. */
function confirmOneClick(list: TweakView[], info: SystemInfo): void {
  const sheet = openSheet();
  const oneWay = list.filter((v) => v.one_way).length;
  const ul = h("ul", "oc-list");
  list.forEach((v) =>
    ul.append(
      h(
        "li",
        "",
        h("span", "oc-dot"),
        h("span", "oc-title", tx(v.title)),
        h("span", "tag recommended", t("category.tag.recommended")),
        v.one_way ? h("span", "tag one_way", t("category.tag.one_way")) : null,
      ),
    ),
  );
  const details = h("details", "oc-details", h("summary", "", t("apply.showList", { count: list.length })), ul);
  const start = button("btn btn-primary btn-lg", [ic("sparkle"), t("apply.start")], () => {
    void runApply(
      sheet,
      list.map((v) => toSelection(v, info)),
    );
  });
  start.dataset.autofocus = "";
  sheet.show(
    h(
      "div",
      "stage",
      h("div", "stage-badge", ic("sparkle")),
      h("h3", "stage-title", t("apply.confirmTitle")),
      h("p", "stage-sub", t("apply.confirmBody", { count: list.length })),
      oneWay ? h("p", "result-note", ic("warn"), t("apply.oneWayNote", { count: oneWay })) : null,
      details,
      h(
        "div",
        "stage-actions",
        button("btn btn-ghost", t("common.cancel"), () => sheet.close()),
        start,
      ),
    ),
  );
}

export async function renderHome(el: HTMLElement): Promise<void> {
  const [info, views] = await Promise.all([systemInfo(), catalogViews()]);
  const count = (c: string) => visibleTweaks(views, c, info).length;
  const orbSub = t("home.orbSub", { count: oneClickTweaks(views, info).length });

  const stage = h(
    "div",
    "orb-stage",
    h("span", "orb-ring r2"),
    h("span", "orb-ring r0"),
    h("span", "orb-ring r1"),
    h("span", "orb-ring r3"),
    h("span", "orb-dot"),
    h("span", "orb-dot d2"),
    h("span", "orb-ripple"),
    h("span", "orb-ripple p2"),
  );
  const orb = lockWhileBusy(
    button(
      "orb",
      h("span", "orb-tx", h("span", "orb-t", t("home.orbTitle")), h("span", "orb-s", orbSub)),
      async () => {
        burstSparks(stage);
        const list = oneClickTweaks(await catalogViews(), info);
        if (!list.length) {
          toast(t("home.nothing"), "err");
          return;
        }
        confirmOneClick(list, info);
      },
    ),
  );
  orb.setAttribute("aria-label", `${t("home.orbTitle")} – ${orbSub}`);
  stage.append(orb);
  stage.querySelectorAll(".orb-ring, .orb-dot, .orb-ripple").forEach((e) =>
    e.setAttribute("aria-hidden", "true"),
  );
  driftSparks(stage);

  const other = otherSettingsTarget(views);
  const otherCount = views.filter((v) =>
    (OTHER_CATEGORIES as string[]).includes(v.category),
  ).length;
  const cards = h(
    "div",
    "cards",
    ...HOME_CATEGORIES.map((c) =>
      card(c, t(`cat.${c}`), count(c), () => void router.go(`category/${c}`)),
    ),
    card("other", t("home.other"), otherCount, () => void router.go(`category/${other}`)),
    card("restore", t("nav.restore"), null, () => void router.go("restore"), "card-restore"),
  );

  const kick = button("link kick", [ic("kick"), t("home.kick")], () => {
    openLink(KICK).catch((e) => toast(errorText(e), "err"));
  });
  const cta = h(
    "div",
    "cta",
    h("span", "cta-ic", ic("sparkle")),
    h("span", "cta-tx", t("home.ctaText")),
    kick,
    button("btn btn-primary btn-sm", [t("home.ctaGo"), ic("arrow")], () => void router.go("contact")),
  );

  const links = h(
    "div",
    "home-links",
    button("link", [ic("tools"), t("nav.tools")], () => void router.go("tools")),
    button("link", [ic("about"), t("nav.about")], () => void router.go("about")),
  );

  el.classList.add("home");
  el.append(
    pills(info),
    stage,
    h("p", "hint", t("home.hint")),
    cards,
    cta,
    links,
  );
}
