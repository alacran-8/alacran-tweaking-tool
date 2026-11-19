// Geri Al: recorded sessions, per-session / all revert, Windows System Restore.
import {
  listSessions,
  revertAll,
  revertSession,
  runTool,
  type SessionList,
  type SessionSummary,
} from "../api";
import { errorText, runRevert } from "../apply";
import { modal } from "../components/modal";
import { openSheet } from "../components/sheet";
import { toast } from "../components/toast";
import { catalogViews, lockWhileBusy, setBlocked } from "../data";
import { button, h, ic } from "../dom";
import { getLang, t } from "../i18n";
import { pageHead, withSidebar } from "./layout";

function formatDate(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString(getLang() === "tr" ? "tr-TR" : "en-GB", {
    dateStyle: "medium",
    timeStyle: "short",
  });
}

function sessionCard(s: SessionSummary, i: number): HTMLElement {
  const date = formatDate(s.created);
  const revert = lockWhileBusy(
    button("btn btn-ghost btn-sm", [ic("restore"), t("restore.revert")], async () => {
      const ok = await modal.confirm({
        title: t("restore.sessionTitle"),
        body: t("restore.sessionBody", { date, count: s.count }),
        confirmText: t("restore.revert"),
        danger: true,
      });
      if (ok) void runRevert(openSheet(), () => revertSession(s.id));
    }),
  );
  setBlocked(revert, s.reverted);
  const el = h(
    "div",
    s.reverted ? "session reverted" : "session",
    h("div", "session-ic", ic(s.reverted ? "check" : "restore")),
    h(
      "div",
      "session-tx",
      h("b", "", date),
      h("span", "", s.internal ? t("restore.internalSession") : t("restore.changes", { count: s.count })),
    ),
    s.reverted ? h("span", "tag ok", t("restore.reverted")) : null,
    revert,
  );
  el.style.setProperty("--i", String(Math.min(i, 14)));
  return el;
}

export async function renderRestore(el: HTMLElement): Promise<void> {
  const views = await catalogViews();
  const main = withSidebar(el, views, "restore");
  let data: SessionList | null = null;
  let loadError = "";
  try {
    data = await listSessions();
  } catch (e) {
    loadError = errorText(e);
  }
  const sessions = [...(data?.sessions ?? [])].sort((a, b) => b.id.localeCompare(a.id));
  const pending = sessions.some((s) => !s.reverted);

  const all = lockWhileBusy(
    button("btn btn-danger-soft", [ic("restore"), t("restore.revertAll")], async () => {
      const ok = await modal.confirm({
        title: t("restore.revertAllTitle"),
        body: t("restore.revertAllBody"),
        confirmText: t("restore.revertAll"),
        danger: true,
      });
      if (ok) void runRevert(openSheet(), revertAll);
    }),
  );
  setBlocked(all, !pending);

  const rstrui = lockWhileBusy(
    button("btn btn-ghost", [ic("external"), t("restore.openRstrui")], () => {
      runTool("system_restore_gui").catch((e) => toast(errorText(e), "err"));
    }),
  );

  main.append(
    pageHead("restore", t("restore.title"), t("restore.subtitle")),
    h("div", "toolbar", all, rstrui),
  );

  if (loadError) main.append(h("div", "notice err", ic("warn"), h("span", "selectable", loadError)));

  const broken = data?.broken ?? [];
  if (broken.length) {
    const ul = h("ul", "");
    broken.forEach(([file, err]) => ul.append(h("li", "selectable", `${file}: ${err}`)));
    main.append(
      h(
        "div",
        "notice warn",
        ic("warn"),
        h("div", "", h("b", "", t("restore.broken")), h("p", "", t("restore.brokenHint")), ul),
      ),
    );
  }

  if (!sessions.length && !loadError) {
    main.append(h("div", "empty", ic("restore", "ic empty-ic"), h("p", "", t("restore.empty"))));
    return;
  }
  main.append(h("div", "rows", ...sessions.map(sessionCard)));
}
