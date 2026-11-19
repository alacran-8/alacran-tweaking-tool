export interface Toggle {
  set(checked: boolean): void;
  get(): boolean;
}

/** Turn `el` into an accessible switch. onChange fires on user input only. */
export function toggle(
  el: HTMLElement,
  checked: boolean,
  onChange: (checked: boolean) => void,
): Toggle {
  el.classList.add("tg");
  el.setAttribute("role", "switch");
  el.tabIndex = 0;
  let state = checked;
  const paint = () => {
    el.classList.toggle("off", !state);
    el.setAttribute("aria-checked", String(state));
  };
  const flip = () => {
    state = !state;
    paint();
    onChange(state);
  };
  el.addEventListener("click", flip);
  el.addEventListener("keydown", (e) => {
    if (e.key === " " || e.key === "Enter") {
      e.preventDefault();
      flip();
    }
  });
  paint();
  return {
    set(v) {
      state = v;
      paint();
    },
    get: () => state,
  };
}
