// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";

const factory = vi.fn((_canvas: unknown, _opts: unknown) => ({
  tubes: { setColors() {}, setLightsColors() {} },
  dispose() {},
}));
vi.mock("threejs-components/build/cursors/tubes1.min.js", () => ({
  default: factory,
}));

describe("tubes background", () => {
  it("constructs_renderer_at_most_once_across_pause_resume", async () => {
    HTMLCanvasElement.prototype.getContext = (() => ({
      getExtension: () => null,
    })) as never;
    const { initBackground, setBackgroundPaused, setBackgroundEnabled } =
      await import("./tubes");
    const host = document.createElement("div");
    document.body.appendChild(host);
    initBackground(host, true);
    for (let i = 0; i < 5; i++) {
      setBackgroundPaused(true);
      expect(host.querySelector("canvas")!.style.display).toBe("none");
      setBackgroundPaused(false);
      expect(host.querySelector("canvas")!.style.display).toBe("block");
    }
    setBackgroundEnabled(false);
    setBackgroundEnabled(true);
    expect(factory).toHaveBeenCalledTimes(1);
    // starts with the alacrantweak.com palette (gold / orange / red)
    expect(factory.mock.calls[0][1]).toEqual({
      tubes: {
        colors: ["#FFD700", "#FFA500", "#ff3b1a"],
        lights: { intensity: 180, colors: ["#ffcf3d", "#ff7a1a", "#880202", "#FFD700"] },
      },
    });
  });

  it("pauses_only_when_hidden_not_on_blur", async () => {
    const { setBackgroundPaused } = await import("./tubes");
    const canvas = () => document.querySelector<HTMLCanvasElement>("#tubes")!;
    let hidden = false;
    Object.defineProperty(document, "hidden", { configurable: true, get: () => hidden });
    window.dispatchEvent(new Event("blur"));
    expect(canvas().style.display).toBe("block");
    hidden = true;
    document.dispatchEvent(new Event("visibilitychange"));
    expect(canvas().style.display).toBe("none");
    hidden = false;
    document.dispatchEvent(new Event("visibilitychange"));
    expect(canvas().style.display).toBe("block");
    // minimise button pauses; focus after restore resumes
    setBackgroundPaused(true);
    expect(canvas().style.display).toBe("none");
    window.dispatchEvent(new Event("focus"));
    expect(canvas().style.display).toBe("block");
  });
});
