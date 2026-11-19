// DEV ONLY: fake backend for previewing the UI in a plain browser (no Tauri).
// main.ts imports this only when import.meta.env.DEV is true, so production
// builds never contain it. Nothing here touches the system.
//
// URL flags: ?mockErr=system_protection_off | restore_point_failed | internal
// (first apply fails with that code), ?mockLaptop=1, ?mockLang=en.
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type {
  ApplyReport,
  RevertReport,
  Selection,
  SessionSummary,
  Settings,
  SystemInfo,
  Tweak,
  TweakView,
} from "../api";
import realCatalog from "../../catalog/new.json";

const params = new URLSearchParams(location.search);

const info: SystemInfo = {
  build: 26100,
  display_version: "24H2",
  edition: "Professional",
  gpu_vendors: ["nvidia"],
  ram_gb: 16,
  is_laptop: params.get("mockLaptop") === "1",
  interactive_sid: "S-1-5-21-MOCK",
};

const txt = (tr: string, en: string) => ({ tr, en });

function base(t: Partial<Tweak> & Pick<Tweak, "id" | "category" | "title" | "description">): Tweak {
  return {
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
    ...t,
  };
}

/** Extra demo rows so every tag / control kind is visible (mock-only ids). */
const demo: Tweak[] = [
  base({
    id: "mock_demo_nagle",
    category: "network",
    title: txt("Nagle algoritmasını kapat", "Disable Nagle's algorithm"),
    description: txt(
      "Küçük paketler beklemeden gönderilir, oyunlarda ping dalgalanması azalır.",
      "Small packets are sent without waiting, so ping jitter drops in games.",
    ),
  }),
  base({
    id: "mock_demo_ipv6",
    category: "network",
    risk: "caution",
    in_one_click: false,
    reboot: "required",
    title: txt("IPv6'yı kapat", "Disable IPv6"),
    description: txt(
      "Bazı ağlarda faydalı, bazılarında internet bağlantısını bozabilir.",
      "Helps on some networks, can break the connection on others.",
    ),
    caution_note: txt(
      "IPv6 gerektiren ağlarda (bazı fiber ve mobil bağlantılar) internet çalışmayabilir.",
      "On networks that need IPv6 (some fibre and mobile links) the internet may stop working.",
    ),
  }),
  base({
    id: "mock_demo_svchost",
    category: "performance",
    title: txt("Svchost bölme eşiği", "Svchost split threshold"),
    description: txt(
      "Servislerin RAM miktarına göre gruplanmasını ayarlar; işlem sayısını azaltır.",
      "Groups services by installed RAM to cut the number of processes.",
    ),
    auto_option: "ram_gb",
    default_option: "8gb",
    options: ["4gb", "8gb", "16gb", "32gb"].map((id) => ({
      id,
      label: txt(id.toUpperCase(), id.toUpperCase()),
      ops: [],
    })),
  }),
  base({
    id: "mock_demo_nv",
    category: "gpu",
    vendor: "nvidia",
    title: txt("NVIDIA telemetrisini kapat", "Disable NVIDIA telemetry"),
    description: txt("NVIDIA sürücüsünün veri toplama görevlerini kapatır.", "Turns off the NVIDIA driver telemetry tasks."),
  }),
  base({
    id: "mock_demo_amd",
    category: "gpu",
    vendor: "amd",
    title: txt("AMD ULPS kapat", "Disable AMD ULPS"),
    description: txt("Gizli (AMD yok).", "Hidden (no AMD GPU)."),
  }),
  base({
    id: "mock_demo_temp",
    category: "cleanup",
    one_way: true,
    title: txt("Geçici dosyaları temizle", "Clean temporary files"),
    description: txt(
      "Kullanıcı ve Windows geçici klasörlerini boşaltır.",
      "Empties the user and Windows temp folders.",
    ),
  }),
  base({
    id: "mock_demo_sticky",
    category: "input",
    title: txt("Yapışkan tuş kısayollarını kapat", "Disable sticky keys shortcuts"),
    description: txt(
      "Oyunda Shift'e 5 kez basınca çıkan pencereyi engeller.",
      "Stops the window that pops up after pressing Shift 5 times in a game.",
    ),
  }),
  base({
    id: "mock_demo_mouse",
    category: "input",
    reboot: "recommended",
    title: txt("Fare veri kuyruğu", "Mouse data queue size"),
    description: txt("Fare girişinin arabellek boyutu.", "Buffer size of mouse input."),
    in_one_click: false,
    default_option: "16",
    options: ["14", "16", "20"].map((id) => ({ id, label: txt(id, id), ops: [] })),
  }),
];

/** Demo states: one applied row and one partly applied row (1 of 3 values). */
function mockState(id: string): Pick<TweakView, "state" | "matched" | "total"> {
  if (id === "mock_demo_nagle") return { state: "applied", matched: 2, total: 2 };
  if (id === "mock_demo_sticky") return { state: "partial", matched: 1, total: 3 };
  return { state: "not_applied", matched: 0, total: 1 };
}

let views: TweakView[] = [
  ...(realCatalog.tweaks as unknown as Partial<Tweak>[]).map((t) =>
    base(t as Tweak),
  ),
  ...demo,
].map((t) => ({
  ...t,
  ...mockState(t.id),
  available:
    info.build >= t.min_build &&
    (t.max_build === null || info.build <= t.max_build) &&
    (t.vendor === null || info.gpu_vendors.includes(t.vendor)),
}));

interface MockSession extends SessionSummary {
  ids: string[];
}
const sessions: MockSession[] = [
  {
    id: "20261116-201500",
    created: "2026-11-16T20:15:00Z",
    count: 1,
    reverted: false,
    internal: false,
    ids: ["mock_demo_nagle"],
  },
  {
    id: "20261116-201459",
    created: "2026-11-16T20:14:59Z",
    count: 1,
    reverted: false,
    internal: true,
    ids: [],
  },
  { id: "20261110-093000", created: "2026-11-10T09:30:00Z", count: 4, reverted: true, internal: false, ids: [] },
];
let settings: Settings = {
  lang: params.get("mockLang") === "en" ? "en" : "tr",
  animated_bg: true,
};
let pendingError = params.get("mockErr");

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

function stamp(): { id: string; created: string } {
  const d = new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return {
    id: `${d.getUTCFullYear()}${p(d.getUTCMonth() + 1)}${p(d.getUTCDate())}-${p(d.getUTCHours())}${p(d.getUTCMinutes())}${p(d.getUTCSeconds())}`,
    created: d.toISOString(),
  };
}

async function apply(sel: Selection[], force: boolean): Promise<ApplyReport> {
  await wait(1600);
  if (pendingError && !(force && pendingError === "restore_point_failed")) {
    const code = pendingError;
    if (code !== "system_protection_off") pendingError = null;
    throw { code, message: `mock ${code}` };
  }
  const picked = views.filter((v) => sel.some((s) => s.tweak_id === v.id));
  const ok = picked.filter((v) => v.available);
  views = views.map((v) =>
    ok.includes(v) ? { ...v, state: "applied", matched: v.total, total: v.total } : v,
  );
  const s = stamp();
  sessions.unshift({
    ...s,
    count: ok.length,
    reverted: false,
    internal: false,
    ids: ok.map((v) => v.id),
  });
  return {
    session_id: s.id,
    applied: ok.length,
    skipped: picked.length - ok.length,
    failed: [],
    one_way: ok.filter((v) => v.one_way).length,
    reboot: ok.some((v) => v.reboot === "required")
      ? "required"
      : ok.some((v) => v.reboot === "recommended")
        ? "recommended"
        : "none",
    bytes_freed: ok.some((v) => v.category === "cleanup") ? 734_003_200 : 0,
  };
}

async function revert(filter: (id: string) => boolean): Promise<RevertReport> {
  await wait(1200);
  let reverted = 0;
  for (const s of sessions) {
    if (s.reverted) continue;
    const hit = s.ids.filter(filter);
    reverted += hit.length;
    s.ids = s.ids.filter((id) => !filter(id));
    views = views.map((v) =>
      hit.includes(v.id) ? { ...v, state: "not_applied", matched: 0 } : v,
    );
    if (!s.ids.length) s.reverted = true;
  }
  return { reverted, skipped_newer: [], failed: [] };
}

export function installMock(): void {
  mockWindows("main");
  mockIPC(async (cmd, args) => {
    const a = (args ?? {}) as Record<string, unknown>;
    switch (cmd) {
      case "is_dry_run":
        return true;
      case "get_system_info":
        return info;
      case "get_catalog":
        return views;
      case "apply_tweaks":
        return apply(a.selections as Selection[], false);
      case "apply_tweaks_force":
        return apply(a.selections as Selection[], true);
      case "enable_protection":
        if (pendingError === "system_protection_off") pendingError = null;
        return null;
      case "list_sessions":
        return {
          sessions: sessions.map(({ ids: _ids, ...s }) => s),
          broken: [["20260101-000000.json", "mock: expected value at line 1 column 20"]],
        };
      case "revert_session": {
        const s = sessions.find((x) => x.id === a.id);
        const ids = new Set(s?.ids ?? []);
        const rep = await revert((id) => ids.has(id));
        if (s) s.reverted = true;
        return rep;
      }
      case "revert_all":
        return revert(() => true);
      case "revert_category":
        return revert((id) => views.find((v) => v.id === id)?.category === a.category);
      case "run_tool":
      case "open_link":
        console.info("[mock]", cmd, a);
        return null;
      case "get_settings":
        return settings;
      case "set_settings":
        settings = a.settings as Settings;
        return null;
      default:
        // window plugin calls (minimise/close/drag) and anything else
        return null;
    }
  });
}
