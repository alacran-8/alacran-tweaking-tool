// Category detail: tweak rows with tags, state, toggle/option, footer actions.
import { revertCategory, type SystemInfo, type TweakView, type Vendor } from "../api";
import { runApply, runRevert } from "../apply";
import { modal } from "../components/modal";
import { openSheet } from "../components/sheet";
import { toggle } from "../components/toggle";
import { catalogViews, lockWhileBusy, setBlocked, systemInfo } from "../data";
import { button, h, ic } from "../dom";
import { t, tx } from "../i18n";
import {
  chosenOptionId,
  isCategoryId,
  partialInfo,
  pendingSelections,
  tagsFor,
  turnOnWarning,
  visibleTweaks,
  type RowChoice,
} from "../logic";
import { pageHead, withSidebar } from "./layout";

const VENDORS: (Vendor | null)[] = ["nvidia", "amd", "intel", null];

function stateBadge(v: TweakView): HTMLElement {
  if (!v.available) return h("span", "state state-na", t("category.unavailable"));
  const part = partialInfo(v);
  if (part) {
    const hint = t("category.partialHint", part);
    const el = h("span", "state state-partial", h("i", ""), t("category.partialCount", part));
    el.title = hint;
    el.tabIndex = 0; // keyboard users reach the explanation too
    el.setAttribute("aria-label", `${t("category.partialCount", part)}. ${hint}`);
    return el;
  }
  return h("span", `state state-${v.state}`, h("i", ""), t(`category.state.${v.state}`));
}

function staticSwitch(on: boolean, hint: string): HTMLElement {
  const s = h("div", on ? "tg locked" : "tg off locked");
  s.setAttribute("role", "switch");
  s.setAttribute("aria-checked", String(on));
  s.setAttribute("aria-disabled", "true");
  s.title = hint;
  return s;
}

function row(v: TweakView, info: SystemInfo, choice: RowChoice, changed: () => void, i: number) {
  const tags = tagsFor(v, info.is_laptop).map((tag) => h("span", `tag ${tag}`, t(`category.tag.${tag}`)));
  const title = h("div", "row-title", h("b", "", tx(v.title)), ...tags);
  const body = h("div", "tx", title, h("p", "row-desc", tx(v.description)));
  const el = h("div", v.available ? "row" : "row dim", body);
  el.style.setProperty("--i", String(Math.min(i, 14)));
  const mark = () => {
    el.classList.toggle("pending", pendingSelections([choice]).length > 0);
    changed();
  };

  if (v.options.length) {
    const sel = h("select", "opt-select");
    sel.setAttribute("aria-label", t("category.option"));
    v.options.forEach((o) => {
      const opt = h("option", "", tx(o.label));
      opt.value = o.id;
      sel.append(opt);
    });
    sel.value = choice.opt ?? "";
    sel.disabled = !v.available;
    sel.addEventListener("change", () => {
      choice.opt = sel.value;
      mark();
    });
    const optRow = h("div", "row-opt", sel);
    if (v.auto_option === "ram_gb")
      optRow.append(h("span", "auto-hint", ic("sparkle"), t("category.autoRam", { gb: info.ram_gb })));
    body.append(optRow);
  }

  const side = h("div", "row-side", stateBadge(v));
  if (!v.available) {
    side.append(staticSwitch(false, t("category.unavailable")));
  } else if (choice.initialOn) {
    side.append(staticSwitch(true, t("category.appliedLocked")));
  } else {
    const sw = h("div", "");
    sw.setAttribute("aria-label", tx(v.title));
    const tg = toggle(sw, false, async (on) => {
      const warn = on ? turnOnWarning(v) : null;
      if (warn) {
        const body: string[] = [];
        if (warn.caution) body.push(v.caution_note ? tx(v.caution_note) : t("category.cautionDefault"));
        if (warn.oneWay) body.push(t("category.oneWayBody"));
        const key = warn.caution ? "caution" : "oneWay";
        const ok = await modal.confirm({
          title: t(`category.${key}Title`, { title: tx(v.title) }),
          body: body.join(" "),
          confirmText: t(`category.${key}Confirm`),
          danger: true,
        });
        if (!ok) {
          tg.set(false);
          return;
        }
      }
      choice.on = on;
      mark();
    });
    side.append(sw);
  }
  el.append(side);
  return el;
}

export async function renderCategory(el: HTMLElement, id = ""): Promise<void> {
  const [info, views] = await Promise.all([systemInfo(), catalogViews()]);
  const main = withSidebar(el, views, `category/${id}`);
  const name = isCategoryId(id) ? t(`cat.${id}`) : id;
  const list = visibleTweaks(views, id, info);
  const usable = list.filter((v) => v.available);
  const onCount = usable.filter((v) => v.state === "applied").length;

  main.append(
    pageHead(
      id,
      t("category.heading", { name }),
      undefined,
      list.length ? h("span", "pill", t("category.countOn", { on: onCount, total: usable.length })) : undefined,
    ),
  );

  if (!list.length) {
    main.append(h("div", "empty", ic(id, "ic empty-ic"), h("p", "", t("category.empty"))));
    return;
  }

  const choices: RowChoice[] = [];
  const applyBtn = lockWhileBusy(
    button("btn btn-primary btn-grow", [ic("sparkle"), h("span", "", t("category.applySelected"))], () => {
      const sel = pendingSelections(choices);
      if (sel.length) void runApply(openSheet(), sel);
    }),
  );
  const applyLabel = applyBtn.querySelector("span:last-child")!;
  const changed = () => {
    const n = pendingSelections(choices).length;
    applyLabel.textContent = n
      ? t("category.applyCount", { count: n })
      : t("category.applySelected");
    setBlocked(applyBtn, n === 0);
  };
  setBlocked(applyBtn, true);

  const revertBtn = lockWhileBusy(
    button("btn btn-ghost", [ic("restore"), t("category.revertPage")], async () => {
      const ok = await modal.confirm({
        title: t("category.revertPageTitle", { name }),
        body: t("category.revertPageBody"),
        confirmText: t("category.revertPage"),
        danger: true,
      });
      if (ok) void runRevert(openSheet(), () => revertCategory(id));
    }),
  );

  const rows = h("div", "rows");
  let i = 0;
  const addRows = (items: TweakView[]) =>
    items.forEach((v) => {
      const opt = chosenOptionId(v, info);
      const initialOn = v.available && v.state === "applied";
      const c: RowChoice = { id: v.id, initialOn, on: initialOn, initialOpt: opt, opt };
      choices.push(c);
      rows.append(row(v, info, c, changed, i++));
    });

  if (id === "gpu") {
    for (const vendor of VENDORS) {
      const group = list.filter((v) => v.vendor === vendor);
      if (!group.length) continue;
      rows.append(h("h4", "group-head", t(`category.vendor.${vendor ?? "general"}`)));
      addRows(group);
    }
  } else {
    addRows(list);
  }

  main.append(rows, h("footer", "foot", applyBtn, revertBtn));
}
