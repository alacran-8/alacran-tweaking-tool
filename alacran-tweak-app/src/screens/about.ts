// Hakkında: version, no-download statement, licences, GitHub, background toggle.
import { getSettings, openLink, setSettings } from "../api";
import { errorText } from "../apply";
import { setBackgroundEnabled } from "../bg/tubes";
import { toast } from "../components/toast";
import { toggle } from "../components/toggle";
import { catalogViews } from "../data";
import { button, h, ic } from "../dom";
import { getLang, t } from "../i18n";
import { pageHead, withSidebar } from "./layout";

export const VERSION = "2.0.0";
const GITHUB = "https://github.com/AIacran/alacran-tweaking-tool";
const RELEASES = "https://github.com/AIacran/alacran-tweaking-tool/releases";

/** Bundled third-party code and its licence. */
const LICENCES: [name: string, licence: string, roleKey: string][] = [
  ["Tauri", "MIT / Apache-2.0", "about.lic.tauri"],
  ["threejs-components", "ISC", "about.lic.tubes"],
  ["three.js r180", "MIT", "about.lic.three"],
  ["Figtree", "SIL OFL 1.1", "about.lic.font"],
];

function link(url: string, iconName: string, label: string): HTMLButtonElement {
  return button("btn btn-ghost", [ic(iconName), label], () => {
    openLink(url).catch((e) => toast(errorText(e), "err"));
  });
}

export async function renderAbout(el: HTMLElement): Promise<void> {
  const main = withSidebar(el, await catalogViews(), "about");
  const settings = await getSettings().catch(() => ({ lang: getLang(), animated_bg: true }));

  const sw = h("div", "");
  sw.setAttribute("aria-label", t("about.bg"));
  toggle(sw, settings.animated_bg, async (on) => {
    setBackgroundEnabled(on);
    try {
      const s = await getSettings();
      await setSettings({ ...s, animated_bg: on });
    } catch (e) {
      toast(errorText(e), "err");
    }
  });

  const lic = h("div", "lic");
  LICENCES.forEach(([name, licence, role]) =>
    lic.append(
      h("div", "lic-row", h("b", "", name), h("span", "lic-role", t(role)), h("span", "tag lic-tag", licence)),
    ),
  );

  main.append(
    pageHead("about", t("about.title")),
    h(
      "div",
      "about-hero",
      h("div", "about-logo"),
      h(
        "div",
        "",
        h("h3", "", t("titlebar.title")),
        h("span", "pill", t("about.version", { v: VERSION })),
      ),
    ),
    h("div", "notice ok", ic("shieldCheck"), h("span", "", t("about.noDownload"))),
    h(
      "div",
      "setting-row",
      h("div", "tx", h("b", "", t("about.bg")), h("p", "row-desc", t("about.bgHint"))),
      sw,
    ),
    h("h4", "group-head", t("about.licences")),
    lic,
    h("div", "toolbar", link(GITHUB, "github", t("about.github")), link(RELEASES, "external", t("about.releases"))),
  );
}
