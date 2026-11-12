/* Lucide, satu keluarga ikon. Path disimpan inline supaya installer tetap kecil
   dan tidak ada permintaan jaringan sama sekali. */
export const icons: Record<string, string> = {
  timer: '<circle cx="12" cy="13" r="8"/><path d="M12 9v4l2.5 2"/><path d="M9 2h6"/>',
  history: '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M8 3v4M16 3v4M3 11h18"/><path d="M8 15h3"/>',
  stats: '<path d="M4 20V11M10 20V4M16 20v-6M22 20H2"/>',
  target: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3"/>',
  subjects: '<path d="M4 5a2 2 0 0 1 2-2h13v18H6a2 2 0 0 1-2-2z"/><path d="M8 3v18"/>',
  settings: '<path d="M4 8h8M17 8h3M4 16h4M13 16h7"/><circle cx="14.5" cy="8" r="2.5"/><circle cx="10.5" cy="16" r="2.5"/>',
  more: '<circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  play: '<path d="M7 4.5 19 12 7 19.5z"/>',
  pause: '<rect x="6" y="4" width="4" height="16" rx="1.4"/><rect x="14" y="4" width="4" height="16" rx="1.4"/>',
  stop: '<rect x="6" y="6" width="12" height="12" rx="2.5"/>',
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
  'chevron-left': '<path d="M15 6l-6 6 6 6"/>',
  'chevron-right': '<path d="M9 6l6 6-6 6"/>',
  pencil: '<path d="M4 20h4L19 9l-4-4L4 16z"/><path d="M14 5l4 4"/>',
  trash: '<path d="M4 7h16M9 7V5h6v2M6 7l1 13h10l1-13"/>',
  download: '<path d="M12 4v12M7 12l5 5 5-5M5 20h14"/>',
  upload: '<path d="M12 20V8M7 12l5-5 5 5M5 4h14"/>',
  folder: '<path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  archive: '<rect x="4" y="4" width="16" height="4" rx="1"/><path d="M6 8v11a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1V8M10 13h4"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  check: '<path d="M5 13l4 4L19 7"/>',
  shrink: '<path d="M9 4v5H4M15 20v-5h5"/>',
  expand: '<path d="M4 9h5V4M20 15h-5v5"/>',
  pin: '<path d="M12 17v5M8 3h8l-1 7 4 3H5l4-3z"/>',
  alert: '<circle cx="12" cy="12" r="9"/><path d="M12 8v5M12 16h.01"/>',
  note: '<path d="M4 5a2 2 0 0 1 2-2h9l5 5v11a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z"/><path d="M8 11h8M8 15h5"/>',
  flame: '<path d="M12 3c3 4 5 6 5 9a5 5 0 0 1-10 0c0-2 1-3.6 3-5.6"/>',
  arrowLeft: '<path d="M19 12H5M11 6l-6 6 6 6"/>'
};

export const iconNames = Object.keys(icons);
