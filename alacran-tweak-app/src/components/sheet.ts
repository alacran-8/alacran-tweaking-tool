// Glass overlay panel for multi-step flows (confirm -> progress -> result).
// Unlike modal.confirm it stays open while its content changes.
import { h } from "../dom";

export interface Sheet {
  body: HTMLElement;
  /** Replace the content with a short cross-fade. */
  show(...nodes: Node[]): void;
  /** While false, Escape and backdrop clicks do nothing (operation running). */
  setDismissable(v: boolean): void;
  close(): void;
  closed: Promise<void>;
}

export function openSheet(wide = false): Sheet {
  const previous = document.activeElement as HTMLElement | null;
  const body = h("div", "sheet-body");
  const panel = h("div", wide ? "sheet sheet-wide" : "sheet", body);
  panel.setAttribute("role", "dialog");
  panel.setAttribute("aria-modal", "true");
  const overlay = h("div", "sheet-overlay", panel);
  let dismissable = true;
  let done!: () => void;
  const closed = new Promise<void>((r) => (done = r));

  const onKey = (e: KeyboardEvent) => {
    if (e.key === "Escape" && dismissable) {
      e.preventDefault();
      close();
    }
  };
  function close() {
    document.removeEventListener("keydown", onKey, true);
    overlay.classList.add("leaving");
    window.setTimeout(() => overlay.remove(), 160);
    previous?.focus?.();
    done();
  }
  overlay.addEventListener("mousedown", (e) => {
    if (e.target === overlay && dismissable) close();
  });
  document.addEventListener("keydown", onKey, true);
  document.body.appendChild(overlay);

  return {
    body,
    show(...nodes) {
      body.classList.remove("swap");
      void body.offsetWidth; // restart the animation
      body.classList.add("swap");
      body.replaceChildren(...nodes);
      body.querySelector<HTMLElement>("[data-autofocus]")?.focus();
    },
    setDismissable(v) {
      dismissable = v;
    },
    close,
    closed,
  };
}
