import { describe, expect, it } from "vitest";
import type { SystemInfo, TweakView } from "./api";
import {
  categoriesWithTweaks,
  chosenOptionId,
  formatMB,
  oneClickTweaks,
  otherSettingsTarget,
  partialInfo,
  pendingSelections,
  sparkRoom,
  tagsFor,
  toSelection,
  turnOnWarning,
  visibleTweaks,
  windowsLabel,
} from "./logic";

const info: SystemInfo = {
  build: 26100,
  display_version: "24H2",
  edition: "Professional",
  gpu_vendors: ["nvidia"],
  ram_gb: 16,
  is_laptop: false,
  interactive_sid: "S-1-5-21-1",
};

function tw(id: string, extra: Partial<TweakView> = {}): TweakView {
  return {
    id,
    category: "network",
    title: { tr: id, en: id },
    description: { tr: "", en: "" },
    risk: "safe",
    in_one_click: true,
    new: false,
    reboot: "none",
    one_way: false,
    min_build: 19044,
    max_build: null,
    vendor: null,
    laptop: null,
    caution_note: null,
    ops: [],
    options: [],
    default_option: null,
    auto_option: null,
    state: "not_applied",
    available: true,
    matched: 0,
    total: 1,
    ...extra,
  };
}

const opt = (id: string) => ({ id, label: { tr: id, en: id }, ops: [] });

describe("logic", () => {
  it("one_click_list_takes_only_available_in_one_click_tweaks", () => {
    const views = [
      tw("a"),
      tw("b", { in_one_click: false }),
      tw("c", { available: false }),
      tw("d", { laptop: false }),
      tw("e", { risk: "caution", in_one_click: false }),
    ];
    expect(oneClickTweaks(views, info).map((t) => t.id)).toEqual(["a", "d"]);
    // laptop:false tweaks stay out of one-click on laptops only
    const laptop = { ...info, is_laptop: true };
    expect(oneClickTweaks(views, laptop).map((t) => t.id)).toEqual(["a"]);
  });

  it("one_click_selection_carries_auto_selected_ram_option", () => {
    const ram = tw("ram", {
      auto_option: "ram_gb",
      options: [opt("4gb"), opt("8gb"), opt("16gb"), opt("32gb")],
      default_option: "8gb",
    });
    expect(toSelection(ram, info)).toEqual({ tweak_id: "ram", option_id: "16gb" });
    expect(chosenOptionId(ram, { ...info, ram_gb: 8 })).toBe("8gb");
    expect(chosenOptionId(ram, { ...info, ram_gb: 10 })).toBe("16gb");
    expect(chosenOptionId(ram, { ...info, ram_gb: 16 })).toBe("16gb");
    expect(chosenOptionId(ram, { ...info, ram_gb: 20 })).toBe("32gb");
    expect(chosenOptionId(ram, { ...info, ram_gb: 2 })).toBe("4gb");
    expect(chosenOptionId(ram, { ...info, ram_gb: 200 })).toBe("32gb");
    // no auto option: default, then first option, then none
    const plain = tw("p", { options: [opt("x"), opt("y")], default_option: "y" });
    expect(chosenOptionId(plain, info)).toBe("y");
    expect(chosenOptionId(tw("q", { options: [opt("x")] }), info)).toBe("x");
    expect(toSelection(tw("r"), info)).toEqual({ tweak_id: "r", option_id: null });
  });

  it("tags_follow_new_risk_reboot_and_one_way", () => {
    expect(tagsFor(tw("a", { in_one_click: false }))).toEqual([]);
    expect(
      tagsFor(
        tw("b", {
          in_one_click: false,
          new: true,
          risk: "caution",
          reboot: "required",
          one_way: true,
        }),
      ),
    ).toEqual(["new", "caution", "reboot", "one_way"]);
    expect(tagsFor(tw("c", { in_one_click: false, reboot: "recommended" }))).toEqual(["reboot"]);
  });

  it("recommended_tag_needs_one_click_and_available", () => {
    expect(tagsFor(tw("a"))).toEqual(["recommended"]);
    expect(tagsFor(tw("b", { new: true }))).toEqual(["recommended", "new"]);
    expect(tagsFor(tw("c", { available: false }))).toEqual([]);
    expect(tagsFor(tw("d", { in_one_click: false }))).toEqual([]);
    // desktop-only one-click tweaks are not recommended on laptops
    expect(tagsFor(tw("e", { laptop: false }), true)).toEqual([]);
    expect(tagsFor(tw("f", { laptop: false }), false)).toEqual(["recommended"]);
  });

  it("partial_info_only_for_partial_state_with_counts", () => {
    expect(partialInfo(tw("a", { state: "partial", matched: 1, total: 3 }))).toEqual({
      x: 1,
      y: 3,
      rest: 2,
    });
    expect(partialInfo(tw("b", { state: "applied", matched: 3, total: 3 }))).toBeNull();
    expect(partialInfo(tw("c", { state: "partial", matched: 0, total: 0 }))).toBeNull();
  });

  it("spark_room_caps_alive_sparks", () => {
    expect(sparkRoom(0, 1)).toBe(1);
    expect(sparkRoom(30, 24)).toBe(10);
    expect(sparkRoom(40, 1)).toBe(0);
    expect(sparkRoom(55, 3)).toBe(0);
    expect(sparkRoom(5, 3, 6)).toBe(1);
  });

  it("turn_on_warning_covers_caution_and_one_way_in_one_confirm", () => {
    expect(turnOnWarning(tw("a"))).toBeNull();
    expect(turnOnWarning(tw("b", { risk: "caution" }))).toEqual({ caution: true, oneWay: false });
    expect(turnOnWarning(tw("c", { one_way: true }))).toEqual({ caution: false, oneWay: true });
    expect(turnOnWarning(tw("d", { risk: "caution", one_way: true }))).toEqual({
      caution: true,
      oneWay: true,
    });
  });

  it("other_settings_card_opens_first_category_with_tweaks", () => {
    expect(otherSettingsTarget([tw("a", { category: "network" })])).toBe("visual");
    expect(
      otherSettingsTarget([
        tw("a", { category: "services" }),
        tw("b", { category: "storage" }),
      ]),
    ).toBe("storage");
  });

  it("sidebar_lists_categories_with_tweaks_in_fixed_order", () => {
    const views = [
      tw("a", { category: "privacy" }),
      tw("b", { category: "game" }),
      tw("c", { category: "network" }),
      tw("d", { category: "game" }),
      tw("e", { category: "internal" }),
    ];
    expect(categoriesWithTweaks(views)).toEqual(["network", "game", "privacy"]);
  });

  it("visible_tweaks_hide_other_gpu_vendors", () => {
    const views = [
      tw("n", { category: "gpu", vendor: "nvidia" }),
      tw("a", { category: "gpu", vendor: "amd", available: false }),
      tw("g", { category: "gpu" }),
      tw("x", { category: "cpu" }),
    ];
    expect(visibleTweaks(views, "gpu", info).map((t) => t.id)).toEqual(["n", "g"]);
  });

  it("pending_selections_only_contain_changed_rows", () => {
    const rows = [
      { id: "a", initialOn: false, on: true, initialOpt: null, opt: null },
      { id: "b", initialOn: false, on: false, initialOpt: null, opt: null },
      { id: "c", initialOn: true, on: true, initialOpt: "x", opt: "x" },
      { id: "d", initialOn: true, on: true, initialOpt: "x", opt: "y" },
      { id: "e", initialOn: false, on: true, initialOpt: "8gb", opt: "8gb" },
    ];
    expect(pendingSelections(rows)).toEqual([
      { tweak_id: "a", option_id: null },
      { tweak_id: "d", option_id: "y" },
      { tweak_id: "e", option_id: "8gb" },
    ]);
  });

  it("formats_megabytes_and_windows_label", () => {
    expect(formatMB(0)).toBe("0");
    expect(formatMB(1_572_864)).toBe("1.5");
    expect(formatMB(734_003_200)).toBe("700");
    expect(windowsLabel(info)).toBe("Windows 11 · 24H2");
    expect(windowsLabel({ ...info, build: 19045, display_version: "22H2" })).toBe(
      "Windows 10 · 22H2",
    );
  });
});
