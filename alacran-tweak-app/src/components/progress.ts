export interface Progress {
  el: HTMLElement;
  /** pct 0..100; optional label text. */
  set(pct: number, label?: string): void;
}

export function progress(parent: HTMLElement): Progress {
  const el = document.createElement("div");
  el.className = "progress";
  el.setAttribute("role", "progressbar");
  el.setAttribute("aria-valuemin", "0");
  el.setAttribute("aria-valuemax", "100");
  const bar = document.createElement("div");
  bar.className = "progress-bar";
  const label = document.createElement("span");
  label.className = "progress-label";
  el.append(bar, label);
  parent.appendChild(el);
  return {
    el,
    set(pct, text) {
      const v = Math.max(0, Math.min(100, pct));
      bar.style.width = `${v}%`;
      el.setAttribute("aria-valuenow", String(Math.round(v)));
      if (text !== undefined) label.textContent = text;
    },
  };
}

export interface Ring {
  el: HTMLElement;
  set(pct: number): void;
  /** Creep towards 92 % while the backend works (it reports no progress). */
  creep(): void;
  /** Jump to 100 % and resolve after the fill animation. */
  finish(): Promise<void>;
  /** Stop creeping (error / declined paths that never finish). */
  stop(): void;
}

/** Circular progress with a gradient stroke and a percentage in the centre. */
export function ring(parent: HTMLElement, label = ""): Ring {
  const R = 54;
  const C = 2 * Math.PI * R;
  const el = document.createElement("div");
  el.className = "ring";
  el.setAttribute("role", "progressbar");
  el.setAttribute("aria-valuemin", "0");
  el.setAttribute("aria-valuemax", "100");
  if (label) el.setAttribute("aria-label", label);
  el.innerHTML =
    `<svg viewBox="0 0 128 128" aria-hidden="true">` +
    `<defs><linearGradient id="ringGrad" x1="0" y1="0" x2="1" y2="1">` +
    `<stop offset="0" stop-color="#FFD700"/><stop offset=".5" stop-color="#FFA500"/><stop offset="1" stop-color="#ff3b1a"/>` +
    `</linearGradient></defs>` +
    `<circle class="ring-track" cx="64" cy="64" r="${R}"/>` +
    `<circle class="ring-fill" cx="64" cy="64" r="${R}" stroke-dasharray="${C}" stroke-dashoffset="${C}"/>` +
    `</svg><div class="ring-pct">0%</div>`;
  parent.appendChild(el);
  const fill = el.querySelector<SVGCircleElement>(".ring-fill")!;
  const pctEl = el.querySelector<HTMLElement>(".ring-pct")!;
  let value = 0;
  let timer = 0;
  const set = (pct: number) => {
    value = Math.max(0, Math.min(100, pct));
    fill.style.strokeDashoffset = String(C * (1 - value / 100));
    pctEl.textContent = `${Math.round(value)}%`;
    el.setAttribute("aria-valuenow", String(Math.round(value)));
  };
  return {
    el,
    set,
    creep() {
      window.clearInterval(timer);
      timer = window.setInterval(() => set(value + (92 - value) * 0.07), 120);
    },
    finish() {
      window.clearInterval(timer);
      set(100);
      el.classList.add("done");
      return new Promise((r) => window.setTimeout(r, 420));
    },
    stop() {
      window.clearInterval(timer);
    },
  };
}
