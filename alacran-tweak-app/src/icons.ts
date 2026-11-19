// Small inline stroke icons (24x24, currentColor). Drawn for this app.
const P: Record<string, string> = {
  home: '<path d="M3 11l9-7 9 7"/><path d="M5 10v10h14V10"/><path d="M10 20v-6h4v6"/>',
  network:
    '<path d="M2 9a15 15 0 0 1 20 0"/><path d="M5 12.5a10 10 0 0 1 14 0"/><path d="M8.5 16a5 5 0 0 1 7 0"/><circle cx="12" cy="19.5" r="1"/>',
  power: '<path d="M13 2L4 14h7l-1 8 9-12h-7z"/>',
  cpu: '<rect x="6" y="6" width="12" height="12" rx="2"/><rect x="9.5" y="9.5" width="5" height="5" rx="1"/><path d="M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3"/>',
  gpu: '<rect x="2" y="6" width="20" height="12" rx="2"/><circle cx="9" cy="12" r="3"/><circle cx="16.5" cy="12" r="1.5"/><path d="M6 18v3M18 18v3"/>',
  game: '<path d="M6 8h12a4 4 0 0 1 4 4.5l-.6 3.4a2.5 2.5 0 0 1-4.3 1.2L15 15H9l-2.1 2.1a2.5 2.5 0 0 1-4.3-1.2L2 12.5A4 4 0 0 1 6 8z"/><path d="M7.5 11v3M6 12.5h3"/><circle cx="15.5" cy="11.5" r=".6"/><circle cx="17.5" cy="13.5" r=".6"/>',
  keyboard: '<rect x="2" y="6" width="20" height="12" rx="2.5"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10"/>',
  input: '<rect x="6" y="2.5" width="12" height="19" rx="6"/><path d="M12 6.5v4"/>',
  visual: '<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
  performance: '<path d="M4 18a8 8 0 1 1 16 0"/><path d="M12 18l4-5"/><path d="M7 14.5l-1-.6M17 14.5l1-.6M12 10V9"/>',
  storage: '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 14h18"/><circle cx="16.5" cy="17" r=".7"/><circle cx="13.5" cy="17" r=".7"/>',
  usb: '<path d="M12 2v16"/><circle cx="12" cy="20" r="2"/><path d="M12 13l-5-3V7"/><circle cx="7" cy="5.5" r="1.5"/><path d="M12 10l5-3V5"/><rect x="15.5" y="2" width="3" height="3"/>',
  services:
    '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9 7 7M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1"/>',
  privacy: '<path d="M12 2l8 3v6c0 5-3.4 9.3-8 11-4.6-1.7-8-6-8-11V5z"/><path d="M9 12l2 2 4-4"/>',
  cleanup: '<path d="M12 3l1.8 4.7L18.5 9.5l-4.7 1.8L12 16l-1.8-4.7L5.5 9.5l4.7-1.8z"/><path d="M19 15l.8 2.2L22 18l-2.2.8L19 21l-.8-2.2L16 18l2.2-.8z"/>',
  restore: '<path d="M3 12a9 9 0 1 0 3-6.7"/><path d="M3 3v6h6"/><path d="M12 7v5l3 2"/>',
  other: '<rect x="3" y="3" width="7" height="7" rx="2"/><rect x="14" y="3" width="7" height="7" rx="2"/><rect x="3" y="14" width="7" height="7" rx="2"/><rect x="14" y="14" width="7" height="7" rx="2"/>',
  tools: '<path d="M14.7 6.3a4 4 0 0 0-5.4 5.2L3 17.8V21h3.2l6.3-6.3a4 4 0 0 0 5.2-5.4l-2.5 2.5-2.6-.6-.6-2.6z"/>',
  contact: '<path d="M21 12a8 8 0 0 1-11.6 7.1L4 20.5l1.4-4.9A8 8 0 1 1 21 12z"/><path d="M8.5 12h.01M12 12h.01M15.5 12h.01"/>',
  about: '<circle cx="12" cy="12" r="10"/><path d="M12 11v6M12 7.5h.01"/>',
  discord:
    '<path d="M8 6.5A15 15 0 0 1 12 6c1.4 0 2.7.2 4 .5l1-1.5c1.8.4 3.3 1.1 4 1.8 1 2.8 1.5 6 1 9.7-1.4 1.1-3 1.9-4.6 2.3l-1-1.8M8 6.5 7 5C5.2 5.4 3.7 6.1 3 6.8c-1 2.8-1.5 6-1 9.7 1.4 1.1 3 1.9 4.6 2.3l1-1.8"/><path d="M6.5 16.5c3.5 1.6 7.5 1.6 11 0"/><circle cx="9" cy="12.5" r="1.3"/><circle cx="15" cy="12.5" r="1.3"/>',
  kick: '<rect x="3" y="3" width="18" height="18" rx="4"/><path d="M9 7v10M9 12l5-5M9 12l5 5"/>',
  youtube: '<rect x="2" y="5" width="20" height="14" rx="4"/><path d="M10 9.5v5l4.5-2.5z"/>',
  mail: '<rect x="2.5" y="5" width="19" height="14" rx="2.5"/><path d="M3 7l9 6 9-6"/>',
  globe: '<circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15 15 0 0 1 0 20 15 15 0 0 1 0-20z"/>',
  github:
    '<path d="M9 19c-4 1.3-4-2-6-2.5M15 21.5v-3.4a3 3 0 0 0-.8-2.3c2.7-.3 5.6-1.3 5.6-6a4.6 4.6 0 0 0-1.3-3.2 4.3 4.3 0 0 0-.1-3.2s-1-.3-3.4 1.3a11.6 11.6 0 0 0-6 0C6.6 3.1 5.6 3.4 5.6 3.4a4.3 4.3 0 0 0-.1 3.2A4.6 4.6 0 0 0 4.2 9.8c0 4.6 2.8 5.7 5.5 6a3 3 0 0 0-.8 2.3v3.4"/>',
  copy: '<rect x="8" y="8" width="13" height="13" rx="2"/><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/>',
  check: '<path d="M4 12.5l5 5L20 6.5"/>',
  arrow: '<path d="M5 12h14M13 6l6 6-6 6"/>',
  external: '<path d="M14 4h6v6M20 4l-9 9"/><path d="M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"/>',
  warn: '<path d="M12 3l10 18H2z"/><path d="M12 10v5M12 18h.01"/>',
  sparkle: '<path d="M12 2l2.2 6.3L20.5 10.5l-6.3 2.2L12 19l-2.2-6.3L3.5 10.5l6.3-2.2z"/>',
  terminal: '<rect x="2.5" y="4" width="19" height="16" rx="2.5"/><path d="M7 9l3 3-3 3M12.5 15H17"/>',
  window: '<rect x="2.5" y="4" width="19" height="16" rx="2.5"/><path d="M2.5 8.5h19"/>',
  shieldCheck: '<path d="M12 2l8 3v6c0 5-3.4 9.3-8 11-4.6-1.7-8-6-8-11V5z"/><path d="M9 12l2 2 4-4"/>',
};

export function icon(name: string, cls = "ic"): string {
  const body = P[name] ?? P.other;
  return `<svg class="${cls}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${body}</svg>`;
}

