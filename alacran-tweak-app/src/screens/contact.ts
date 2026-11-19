// İletişim: pitch (contact.pitch.* strings) and the five contact channels.
import { openLink } from "../api";
import { errorText } from "../apply";
import { toast } from "../components/toast";
import { catalogViews } from "../data";
import { button, h, ic } from "../dom";
import { t } from "../i18n";
import { pageHead, withSidebar } from "./layout";

const DISCORD = "alacran_";
const KICK = "https://kick.com/alacranfps";
const YOUTUBE = "https://www.youtube.com/@alacrann";
const EMAIL = "zorseker.3322@gmail.com";
const SITE = "https://www.alacrantweak.com";

async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    // Fallback for WebViews without clipboard permission.
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    ta.remove();
  }
}

function channel(cls: string, iconName: string, label: string, value: string, action: HTMLButtonElement) {
  return h(
    "div",
    `channel ${cls}`,
    h("div", "channel-ic", ic(iconName)),
    h("div", "channel-tx", h("span", "", label), h("b", "selectable", value)),
    action,
  );
}

function openBtn(url: string): HTMLButtonElement {
  return button("btn btn-ghost btn-sm", [ic("external"), t("contact.open")], () => {
    openLink(url).catch((e) => toast(errorText(e), "err"));
  });
}

export async function renderContact(el: HTMLElement): Promise<void> {
  const main = withSidebar(el, await catalogViews(), "contact");

  const copy = button("btn btn-ghost btn-sm", [ic("copy"), h("span", "", t("contact.copy"))], async () => {
    await copyText(DISCORD);
    copy.classList.add("copied");
    copy.querySelector("span:last-child")!.textContent = t("contact.copied");
    toast(t("contact.copiedToast", { name: DISCORD }));
    window.setTimeout(() => {
      copy.classList.remove("copied");
      const label = copy.querySelector("span:last-child");
      if (label) label.textContent = t("contact.copy");
    }, 1800);
  });

  const points = ["p1", "p2", "p3"].map((k) =>
    h("li", "", ic("check"), h("span", "", t(`contact.pitch.${k}`))),
  );

  main.append(
    pageHead("contact", t("contact.title"), t("contact.subtitle")),
    h(
      "div",
      "pitch",
      h("div", "pitch-glow"),
      h("h3", "gold-text", t("contact.pitch.title")),
      h("p", "", t("contact.pitch.body")),
      h("ul", "pitch-points", ...points),
    ),
    h(
      "div",
      "channels",
      channel("ch-kick", "kick", t("contact.kick"), "alacranfps", openBtn(KICK)),
      channel("ch-discord", "discord", t("contact.discord"), DISCORD, copy),
      channel("ch-youtube", "youtube", t("contact.youtube"), "@alacrann", openBtn(YOUTUBE)),
      channel("ch-mail", "mail", t("contact.email"), EMAIL, openBtn(`mailto:${EMAIL}`)),
      channel("ch-site", "globe", t("contact.site"), "alacrantweak.com", openBtn(SITE)),
    ),
  );
}
