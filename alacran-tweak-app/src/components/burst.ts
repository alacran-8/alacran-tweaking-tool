// Subtle success burst: a ring of small glowing dots flying outwards.
import { reducedMotion } from "../dom";

const COLORS = ["#FFD700", "#FFA500", "#fcbc23", "#ff7a1a", "#7dffb0"];

export function burst(anchor: HTMLElement, count = 26): void {
  if (reducedMotion()) return;
  const layer = document.createElement("div");
  layer.className = "burst";
  for (let i = 0; i < count; i++) {
    const d = document.createElement("i");
    const a = (i / count) * Math.PI * 2 + Math.random() * 0.4;
    const r = 70 + Math.random() * 60;
    d.style.setProperty("--dx", `${Math.cos(a) * r}px`);
    d.style.setProperty("--dy", `${Math.sin(a) * r}px`);
    d.style.setProperty("--c", COLORS[i % COLORS.length]);
    d.style.animationDelay = `${Math.random() * 120}ms`;
    layer.appendChild(d);
  }
  anchor.appendChild(layer);
  window.setTimeout(() => layer.remove(), 1400);
}
