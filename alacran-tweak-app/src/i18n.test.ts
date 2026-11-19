import { describe, expect, it } from "vitest";
import tr from "./i18n/tr.json";
import en from "./i18n/en.json";
import { t, _dicts } from "./i18n";

function keys(tree: object, prefix = ""): string[] {
  return Object.entries(tree).flatMap(([k, v]) =>
    typeof v === "object" && v !== null
      ? keys(v, `${prefix}${k}.`)
      : [`${prefix}${k}`],
  );
}

describe("i18n", () => {
  it("tr_and_en_have_identical_keys", () => {
    expect(keys(tr).sort()).toEqual(keys(en).sort());
  });

  it("t_interpolates_vars", () => {
    expect(t("common.ok")).toBe("Tamam"); // Turkish is the default
    (_dicts.tr as Record<string, unknown>).test = {
      greet: "Merhaba {name}, {n} ayar, {x}",
    };
    expect(t("test.greet", { name: "Ali", n: 3 })).toBe(
      "Merhaba Ali, 3 ayar, {x}",
    );
  });

  it("t_returns_key_when_missing", () => {
    expect(t("no.such.key")).toBe("no.such.key");
  });
});
