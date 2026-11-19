import { t } from "../i18n";

export interface ConfirmOptions {
  title: string;
  body: string;
  confirmText?: string;
  danger?: boolean;
}

function confirm(opts: ConfirmOptions): Promise<boolean> {
  return new Promise((resolve) => {
    const previous = document.activeElement as HTMLElement | null;
    const overlay = document.createElement("div");
    overlay.className = "modal-overlay";
    const box = document.createElement("div");
    box.className = "modal";
    box.setAttribute("role", "dialog");
    box.setAttribute("aria-modal", "true");

    const h = document.createElement("h3");
    h.textContent = opts.title;
    const p = document.createElement("p");
    p.textContent = opts.body;
    const actions = document.createElement("div");
    actions.className = "modal-actions";
    const cancel = document.createElement("button");
    cancel.className = "btn btn-ghost";
    cancel.textContent = t("common.cancel");
    const ok = document.createElement("button");
    ok.className = opts.danger ? "btn btn-danger" : "btn btn-primary";
    ok.textContent = opts.confirmText ?? t("common.confirm");
    actions.append(cancel, ok);
    box.append(h, p, actions);
    overlay.appendChild(box);

    const finish = (v: boolean) => {
      document.removeEventListener("keydown", onKey, true);
      overlay.remove();
      previous?.focus?.();
      resolve(v);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        finish(false);
      } else if (e.key === "Tab") {
        // keep focus inside the dialog
        e.preventDefault();
        (document.activeElement === ok ? cancel : ok).focus();
      }
    };
    cancel.addEventListener("click", () => finish(false));
    ok.addEventListener("click", () => finish(true));
    overlay.addEventListener("mousedown", (e) => {
      if (e.target === overlay) finish(false);
    });
    document.addEventListener("keydown", onKey, true);
    document.body.appendChild(overlay);
    cancel.focus(); // safe default
  });
}

export const modal = { confirm };
