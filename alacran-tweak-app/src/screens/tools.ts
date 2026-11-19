// Araçlar: one card per ToolId; OneDrive uninstall needs a confirm.
import { runTool, type ToolId } from "../api";
import { errorText } from "../apply";
import { modal } from "../components/modal";
import { toast } from "../components/toast";
import { catalogViews, lockWhileBusy } from "../data";
import { button, h, ic } from "../dom";
import { t } from "../i18n";
import { pageHead, withSidebar } from "./layout";

/** [id, icon, opens a console window] — every ToolId, in display order. */
const TOOLS: [ToolId, string, boolean][] = [
  ["sfc", "privacy", true],
  ["dism", "services", true],
  ["dns_flush", "network", false],
  ["winsock_reset", "network", true],
  ["perf_counters", "performance", true],
  ["defrag_gui", "storage", false],
  ["perf_options_gui", "visual", false],
  ["mouse_cpl", "input", false],
  ["keyboard_cpl", "keyboard", false],
  ["system_restore_gui", "restore", false],
  ["one_drive_uninstall", "warn", false],
];

async function run(id: ToolId): Promise<void> {
  const name = t(`tools.${id}.name`);
  if (id === "one_drive_uninstall") {
    const ok = await modal.confirm({
      title: name,
      body: t("tools.oneDriveConfirm"),
      confirmText: t("tools.run"),
      danger: true,
    });
    if (!ok) return;
  }
  try {
    await runTool(id);
    toast(t("tools.started", { name }));
  } catch (e) {
    toast(errorText(e), "err");
  }
}

export async function renderTools(el: HTMLElement): Promise<void> {
  const main = withSidebar(el, await catalogViews(), "tools");
  const grid = h("div", "tool-grid");
  TOOLS.forEach(([id, iconName, console], i) => {
    const caution = id === "one_drive_uninstall";
    const card = h(
      "div",
      caution ? "tool caution" : "tool",
      h("div", "tool-ic", ic(iconName)),
      h(
        "div",
        "tool-tx",
        h(
          "b",
          "",
          t(`tools.${id}.name`),
          caution ? h("span", "tag caution", t("category.tag.caution")) : null,
        ),
        h("p", "", t(`tools.${id}.desc`)),
        console ? h("span", "tool-meta", ic("terminal"), t("tools.console")) : null,
      ),
      lockWhileBusy(button("btn btn-ghost btn-sm", t("tools.run"), () => void run(id))),
    );
    card.style.setProperty("--i", String(i));
    grid.append(card);
  });
  main.append(pageHead("tools", t("tools.title"), t("tools.subtitle")), grid);
}
