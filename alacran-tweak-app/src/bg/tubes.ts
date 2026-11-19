// Animated tubes background (threejs-components, ISC licence, three.js is
// bundled inside the package: no network access). Falls back to static gradients.
import TubesCursor from "threejs-components/build/cursors/tubes1.min.js";

interface TubesApp {
  tubes: {
    setColors(c: string[]): void;
    setLightsColors(c: string[]): void;
  };
  dispose(): void;
}

// Warm site palette first; a click on empty background cycles gold/red variants.
const PALETTES = [
  { t: ["#FFD700", "#FFA500", "#ff3b1a"], l: ["#ffcf3d", "#ff7a1a", "#880202", "#FFD700"] },
  { t: ["#ff3b1a", "#880202", "#ffb347"], l: ["#ff4a1a", "#b3120a", "#ffd27a", "#ff7a1a"] },
  { t: ["#fcbc23", "#ffe08a", "#c0392b"], l: ["#FFA500", "#fff1b8", "#880202", "#fcbc23"] },
  { t: ["#b3120a", "#ff5a5a", "#FFD700"], l: ["#880202", "#ff2814", "#FFA500", "#ffcf3d"] },
];

let host: HTMLElement;
let app: TubesApp | null = null;
let canvas: HTMLCanvasElement | null = null;
let enabled = false;
let paused = false;
let paletteIdx = 0;
let wired = false;

function webgl2Ok(): boolean {
  try {
    const c = document.createElement("canvas");
    const gl = c.getContext("webgl2");
    gl?.getExtension("WEBGL_lose_context")?.loseContext();
    return !!gl;
  } catch {
    return false;
  }
}

// The library's dispose() cannot remove its resize/visibilitychange listeners
// (it re-binds them), so the renderer is created at most once and never
// disposed. Pause/disable hides the canvas: display:none makes the library's
// own IntersectionObserver stop its render loop; showing it resumes the loop.
function start(): void {
  if (app) return;
  const p = PALETTES[paletteIdx];
  canvas = document.createElement("canvas");
  canvas.id = "tubes";
  host.appendChild(canvas);
  try {
    app = TubesCursor(canvas, {
      tubes: { colors: p.t, lights: { intensity: 180, colors: p.l } },
    }) as TubesApp;
  } catch (e) {
    console.warn("[bg] tubes failed, using static gradient", e);
    canvas.remove();
    canvas = null;
    host.classList.add("bg-fallback");
  }
}

function sync(): void {
  const active = enabled && !paused && !host.classList.contains("bg-fallback");
  if (active) start(); // first activation only; later calls are no-ops
  if (canvas) canvas.style.display = active ? "block" : "none";
  host.classList.toggle("bg-static", !enabled);
}

function isEmptyBackground(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target === document.body ||
      target.id === "app" ||
      target.id === "view" ||
      target.classList.contains("view-page"))
  );
}

/** Pause/resume rendering (hidden document, minimise button). */
export function setBackgroundPaused(p: boolean): void {
  paused = p;
  if (host) sync();
}

export function setBackgroundEnabled(on: boolean): void {
  enabled = on;
  if (host) sync();
}

export function initBackground(container: HTMLElement, on: boolean): void {
  host = container;
  host.classList.remove("bg-fallback");
  if (!webgl2Ok()) host.classList.add("bg-fallback");
  if (!wired) {
    wired = true;
    // The only pause/resume wiring: hidden document (minimised, other virtual
    // desktop) pauses; becoming visible or focused again resumes. Blur alone
    // never pauses: the window may sit visible beside another app.
    const follow = () => setBackgroundPaused(document.hidden);
    document.addEventListener("visibilitychange", follow);
    window.addEventListener("focus", follow);
    // Click on empty background (not UI) cycles the palette.
    document.body.addEventListener("click", (e) => {
      if (!app || !isEmptyBackground(e.target)) return;
      paletteIdx = (paletteIdx + 1) % PALETTES.length;
      const p = PALETTES[paletteIdx];
      app.tubes.setColors(p.t);
      app.tubes.setLightsColors(p.l);
    });
  }
  paused = document.hidden;
  setBackgroundEnabled(on);
}
