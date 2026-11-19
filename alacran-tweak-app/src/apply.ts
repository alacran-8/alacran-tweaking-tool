// Apply / revert flows shared by Home, Category and Restore: progress ring,
// restore point recovery dialogs, result cards. Runs inside a Sheet so the
// page behind can refresh (and the user cannot start a second operation).
import {
  applyTweaks,
  applyTweaksForce,
  enableProtection,
  isCommandError,
  type ApplyReport,
  type RevertReport,
  type Selection,
} from "./api";
import { burst } from "./components/burst";
import { modal } from "./components/modal";
import { ring } from "./components/progress";
import type { Sheet } from "./components/sheet";
import {
  catalogViews,
  isDryRunMode,
  restorePointStatus,
  setBusy,
  setRestorePointStatus,
} from "./data";
import { button, h, ic } from "./dom";
import { t, tx } from "./i18n";
import { formatMB } from "./logic";
import * as router from "./router";

export function errorText(e: unknown): string {
  if (isCommandError(e)) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

/**
 * applyTweaks with the restore point recovery dialogs:
 * protection off -> offer to enable it and retry; restore point failed ->
 * offer to continue without one. Resolves null when the user declines.
 */
export async function applyWithRecovery(sel: Selection[]): Promise<ApplyReport | null> {
  let force = false;
  for (let attempt = 0; attempt < 4; attempt++) {
    try {
      if (!force && restorePointStatus() !== "created") setRestorePointStatus("creating");
      const rep = force ? await applyTweaksForce(sel) : await applyTweaks(sel);
      if (!force) setRestorePointStatus("created");
      return rep;
    } catch (e) {
      if (!isCommandError(e)) throw e;
      if (e.code === "system_protection_off") {
        setRestorePointStatus("failed");
        const ok = await modal.confirm({
          title: t("apply.protectionOffTitle"),
          body: t("apply.protectionOff"),
          confirmText: t("apply.protectionOn"),
        });
        if (!ok) return null;
        await enableProtection();
        continue;
      }
      if (e.code === "restore_point_failed") {
        setRestorePointStatus("failed");
        const ok = await modal.confirm({
          title: t("apply.rpFailedTitle"),
          body: t("apply.rpFailed"),
          confirmText: t("apply.continue"),
          danger: true,
        });
        if (!ok) return null;
        force = true;
        continue;
      }
      throw e;
    }
  }
  throw new Error(t("apply.giveUp"));
}

/** Re-read tweak states and repaint the current page (after the lock is released). */
async function afterChange(): Promise<void> {
  try {
    await catalogViews(true);
  } catch (e) {
    console.warn("[apply] catalog refresh failed", e);
  }
  await router.refresh();
}

function progressStage(title: string, sub: string) {
  const ringHost = h("div", "ring-host");
  const r = ring(ringHost, title);
  const node = h(
    "div",
    "stage stage-center",
    ringHost,
    h("h3", "stage-title", title),
    h("p", "stage-sub", sub),
  );
  return { node, ring: r };
}

function closeRow(sheet: Sheet, extra?: HTMLButtonElement): HTMLElement {
  const close = button("btn btn-primary", t("common.close"), () => sheet.close());
  close.dataset.autofocus = "";
  return h("div", "stage-actions", extra ?? null, close);
}

function detailsList(summary: string, items: string[], cls = ""): HTMLElement | null {
  if (!items.length) return null;
  const d = h("details", `result-list ${cls}`, h("summary", "", summary));
  const ul = h("ul", "");
  items.forEach((i) => ul.append(h("li", "", i)));
  d.append(ul);
  return d;
}

function errorStage(sheet: Sheet, message: string): HTMLElement {
  return h(
    "div",
    "stage stage-center",
    h("div", "result-icon err", ic("warn")),
    h("h3", "stage-title", t("apply.errorTitle")),
    h("p", "stage-sub selectable", message),
    closeRow(sheet),
  );
}

async function titleOf(id: string): Promise<string> {
  try {
    const v = (await catalogViews()).find((x) => x.id === id);
    return v ? tx(v.title) : id;
  } catch {
    return id;
  }
}

async function applyResult(sheet: Sheet, rep: ApplyReport): Promise<HTMLElement> {
  const ok = rep.failed.length === 0;
  const lines: (HTMLElement | null)[] = [
    h(
      "p",
      "result-summary",
      t("apply.summary", {
        applied: rep.applied,
        skipped: rep.skipped,
        failed: rep.failed.length,
      }),
    ),
  ];
  if (rep.reboot !== "none")
    lines.push(
      h(
        "p",
        "result-note reboot",
        ic("restore"),
        t(rep.reboot === "required" ? "apply.rebootRequired" : "apply.rebootRecommended"),
      ),
    );
  if (rep.bytes_freed > 0)
    lines.push(
      h("p", "result-note", ic("cleanup"), t("apply.freed", { mb: formatMB(rep.bytes_freed) })),
    );
  if (rep.one_way > 0)
    lines.push(h("p", "result-note", ic("warn"), t("apply.oneWay", { count: rep.one_way })));
  if (isDryRunMode()) lines.push(h("p", "result-note dry", t("apply.dryRunNote")));
  const failed = await Promise.all(
    rep.failed.map(async ([id, err]) => `${await titleOf(id)}: ${err}`),
  );
  lines.push(detailsList(t("apply.failedList", { count: failed.length }), failed, "err"));
  const toRestore = button("btn btn-ghost", t("nav.restore"), () => {
    sheet.close();
    void router.go("restore");
  });
  return h(
    "div",
    "stage stage-center",
    h("div", ok ? "result-icon ok" : "result-icon warn", ic(ok ? "check" : "warn")),
    h("h3", "stage-title", t(ok ? "apply.resultTitle" : "apply.resultTitleErr")),
    ...lines,
    closeRow(sheet, toRestore),
  );
}

/** Progress ring -> apply (with recovery dialogs) -> result card. */
export async function runApply(sheet: Sheet, sel: Selection[]): Promise<ApplyReport | null> {
  sheet.setDismissable(false);
  const stage = progressStage(t("apply.running"), t("apply.runningSub"));
  sheet.show(stage.node);
  stage.ring.creep();
  setBusy(true);
  let rep: ApplyReport | null = null;
  let error: string | null = null;
  try {
    rep = await applyWithRecovery(sel);
  } catch (e) {
    error = errorText(e);
  } finally {
    setBusy(false);
    stage.ring.stop(); // error / declined paths never reach finish()
  }
  sheet.setDismissable(true);
  if (error !== null) {
    sheet.show(errorStage(sheet, error));
  } else if (rep === null) {
    sheet.close();
  } else {
    await stage.ring.finish();
    const node = await applyResult(sheet, rep);
    sheet.show(node);
    if (rep.failed.length === 0) burst(node.querySelector<HTMLElement>(".result-icon")!);
  }
  await afterChange();
  return rep;
}

/** Progress ring -> revert -> result with skipped_newer and failures. */
export async function runRevert(
  sheet: Sheet,
  revert: () => Promise<RevertReport>,
): Promise<RevertReport | null> {
  sheet.setDismissable(false);
  const stage = progressStage(t("revert.running"), t("apply.runningSub"));
  sheet.show(stage.node);
  stage.ring.creep();
  setBusy(true);
  let rep: RevertReport | null = null;
  let error: string | null = null;
  try {
    rep = await revert();
  } catch (e) {
    error = errorText(e);
  } finally {
    setBusy(false);
    stage.ring.stop();
  }
  sheet.setDismissable(true);
  if (rep === null) {
    sheet.show(errorStage(sheet, error ?? ""));
  } else {
    await stage.ring.finish();
    const ok = rep.failed.length === 0;
    const nothing = rep.reverted === 0 && ok && rep.skipped_newer.length === 0;
    const node = h(
      "div",
      "stage stage-center",
      h("div", ok ? "result-icon ok" : "result-icon warn", ic(ok ? "check" : "warn")),
      h("h3", "stage-title", t(ok ? "revert.doneTitle" : "revert.doneTitleErr")),
      h(
        "p",
        "result-summary",
        nothing ? t("revert.nothing") : t("revert.summary", { reverted: rep.reverted }),
      ),
      isDryRunMode() ? h("p", "result-note dry", t("apply.dryRunNote")) : null,
      detailsList(
        t("revert.skippedNewer", { count: rep.skipped_newer.length }),
        rep.skipped_newer,
        "warn",
      ),
      detailsList(
        t("revert.failed", { count: rep.failed.length }),
        rep.failed.map(([target, err]) => `${target}: ${err}`),
        "err",
      ),
      closeRow(sheet),
    );
    // These lists matter after a revert: show them without an extra click.
    node.querySelectorAll("details").forEach((d) => (d.open = true));
    sheet.show(node);
    if (ok && rep.reverted > 0) burst(node.querySelector<HTMLElement>(".result-icon")!);
  }
  await afterChange();
  return rep;
}
