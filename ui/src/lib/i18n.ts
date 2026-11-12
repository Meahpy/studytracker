/**
 * Semua teks antarmuka. Tidak ada string tampilan yang ditulis langsung di komponen.
 * Bahasa: Indonesia, sentence case, kata kerja aktif.
 */

const two = (n: number) => String(n).padStart(2, '0');

export const m = {
  app: {
    name: 'StudyTracker',
    previewBanner:
      'Mode pratinjau. Data disimpan di browser ini saja. Jalankan `npm run tauri dev` untuk memakai database asli.'
  },
  nav: {
    timer: 'Timer',
    history: 'Riwayat',
    stats: 'Statistik',
    targets: 'Target',
    subjects: 'Mapel',
    settings: 'Pengaturan',
    more: 'Lain',
    primary: 'Navigasi utama'
  },
  timer: {
    title: 'Timer',
    start: 'Mulai belajar',
    pause: 'Jeda',
    resume: 'Lanjut',
    stop: 'Berhenti',
    startedAt: (time: string) => `Mulai ${time}`,
    targetLeft: (duration: string) => `${duration} lagi dari target hari ini`,
    targetReached: 'Target hari ini tercapai',
    running: 'Berjalan',
    paused: 'Jeda',
    idle: 'Belum mulai',
    lockedHint: 'Mata pelajaran dikunci selama timer berjalan. Berhenti dulu untuk mengganti.',
    newSubject: 'Mata pelajaran baru',
    mini: 'Mini mode',
    noSubjects: 'Belum ada mata pelajaran. Tambahkan satu untuk mulai mencatat.',
    addSubject: 'Tambah mata pelajaran'
  },
  today: {
    heading: 'Hari ini',
    total: (duration: string) => (duration === '0 menit' ? 'Belum ada sesi hari ini.' : duration),
    goalProgress: (percent: number, target: string) => `${percent}% dari target ${target}`,
    sessions: 'Sesi',
    empty: 'Sesi hari ini akan muncul di sini.',
    week: (duration: string) => `Minggu ini ${duration}`
  },
  history: {
    title: 'Riwayat',
    previousMonth: 'Bulan sebelumnya',
    nextMonth: 'Bulan berikutnya',
    addManual: 'Tambah sesi manual',
    empty: 'Belum ada sesi di hari ini.',
    totalOn: (title: string, duration: string) => `${title}, ${duration}`,
    pickDay: 'Pilih hari untuk melihat sesi.'
  },
  session: {
    editTitle: 'Ubah sesi',
    addTitle: 'Tambah sesi manual',
    subject: 'Mata pelajaran',
    start: 'Jam mulai',
    end: 'Jam selesai',
    note: 'Catatan',
    notePlaceholder: 'Bab berapa, halaman, atau apa yang dipelajari',
    save: 'Simpan perubahan',
    saveNew: 'Simpan sesi',
    delete: 'Hapus sesi',
    deleteConfirm: (subject: string, from: string, to: string) =>
      `Hapus sesi ${subject} ${from} sampai ${to}? Sesi tidak bisa dikembalikan.`,
    overlapWarning: 'Rentang ini bertumpang tindih dengan sesi lain. Tetap simpan?',
    invalidRange: 'Jam selesai harus setelah jam mulai.',
    saved: 'Sesi tersimpan',
    deleted: 'Sesi dihapus'
  },
  toast: {
    sessionSaved: 'Sesi tersimpan',
    addNote: 'Tambah catatan',
    undo: 'Urungkan',
    undone: 'Sesi dibatalkan',
    exported: (path: string) => `Data diekspor ke ${path}`,
    imported: (sessions: number) => `${sessions} sesi diimpor`,
    backedUp: (path: string) => `Cadangan dibuat: ${path}`,
    targetReached: 'Target hari ini tercapai',
    copied: 'Disalin'
  },
  stats: {
    title: 'Statistik',
    day: 'Hari',
    week: 'Minggu',
    month: 'Bulan',
    empty: 'Statistik muncul setelah kamu menyelesaikan sesi pertama.',
    summary: (total: string, delta: string, up: boolean) =>
      up ? `Kamu belajar ${total}, naik ${delta} dari periode sebelumnya.` : `Kamu belajar ${total}, turun ${delta} dari periode sebelumnya.`,
    summarySame: (total: string) => `Kamu belajar ${total}, sama dengan periode sebelumnya.`,
    summaryZero: 'Belum ada sesi pada periode ini.',
    targetLine: 'Target harian',
    breakdown: 'Per mata pelajaran',
    heatmap: 'Konsistensi 26 minggu',
    legend: { less: 'Kurang', more: 'Target tercapai' },
    streaks: 'Streak dan rekor',
    currentStreak: 'Streak saat ini',
    longestStreak: 'Streak terpanjang',
    bestDay: 'Hari terpanjang',
    bestWeek: 'Minggu terpanjang',
    days: (n: number) => `${n} hari`,
    previous: 'Periode sebelumnya',
    next: 'Periode berikutnya',
    sessionCount: (n: number) => `${n} sesi`,
    noData: 'Tidak ada data'
  },
  targets: {
    title: 'Target dan D-day',
    daily: 'Target harian',
    weekly: 'Target mingguan',
    dailyHint: 'Dihitung per hari belajar. Sesi dini hari masuk hari sebelumnya.',
    weeklyHint: 'Dihitung Senin sampai Minggu.',
    dday: 'D-day',
    hours: 'jam',
    minutes: 'menit',
    addExam: 'Tambah D-day',
    examTitle: 'Judul',
    examDate: 'Tanggal',
    save: 'Simpan',
    delete: 'Hapus',
    empty: 'Belum ada D-day. Tambahkan tanggal ujian untuk melihat hitung mundur.',
    daysLeft: (n: number) => `${n} hari lagi`,
    today: 'Hari ini',
    overdue: (n: number) => `Terlewat ${n} hari`,
    invalid: 'Judul dan tanggal perlu diisi.'
  },
  subjects: {
    title: 'Mata pelajaran',
    add: 'Tambah',
    edit: 'Ubah',
    save: 'Simpan',
    name: 'Nama',
    color: 'Warna',
    total: (duration: string) => duration,
    archive: 'Arsipkan',
    unarchive: 'Aktifkan lagi',
    delete: 'Hapus',
    archivedTag: 'Diarsipkan',
    empty: 'Belum ada mata pelajaran. Tambahkan satu untuk mulai mencatat.',
    onlyNoSessions: 'Mata pelajaran yang punya sesi hanya bisa diarsipkan.',
    nameRequired: 'Nama mata pelajaran perlu diisi.'
  },
  settings: {
    title: 'Pengaturan',
    appearance: 'Tampilan',
    theme: 'Tema',
    themeLight: 'Terang',
    themeDark: 'Gelap',
    themeSystem: 'Ikuti sistem',
    scale: 'Skala antarmuka',
    timer: 'Timer',
    dayStart: 'Awal hari belajar',
    dayStartHint: 'Sesi sebelum jam ini dihitung sebagai hari sebelumnya.',
    idle: 'Deteksi tidak aktif',
    idleOff: 'Mati',
    idleMinutes: (n: number) => `${n} menit`,
    sleep: 'Hitung waktu saat komputer tidur',
    globalShortcut: 'Pintasan global',
    window: 'Jendela',
    closeToTray: 'Tutup ke tray',
    autostart: 'Mulai saat komputer menyala',
    rememberWindow: 'Ingat ukuran jendela',
    miniOnTop: 'Mini mode selalu di atas',
    data: 'Data',
    exportCsv: 'Ekspor CSV',
    exportJson: 'Ekspor JSON',
    import: 'Impor',
    openDir: 'Buka folder data',
    backup: 'Buat cadangan',
    about: 'Tentang',
    version: (v: string) => `Versi ${v}`,
    database: 'Database',
    notSupported: 'Tidak tersedia di mode pratinjau.'
  },
  dialog: {
    close: 'Tutup',
    cancel: 'Batal',
    confirm: 'Lanjutkan',
    discard: 'Buang'
  },
  orphan: {
    title: 'Sesi belum selesai',
    body: (time: string) => `Ada sesi yang belum selesai sejak ${time}. Selesaikan pada jam berapa?`,
    save: 'Simpan sesi',
    discard: 'Buang sesi'
  },
  idle: {
    title: 'Kamu tidak aktif',
    body: (minutes: number) => `Kamu tidak aktif selama ${minutes} menit. Hitung waktu itu sebagai belajar?`,
    count: 'Hitung',
    drop: (minutes: number) => `Buang ${minutes} menit`
  },
  error: {
    title: 'Ada yang tidak beres',
    generic: 'Perubahan belum tersimpan. Coba lagi.'
  },
  mini: {
    back: 'Kembali ke jendela penuh',
    pin: 'Selalu di atas'
  },
  time: {
    hourShort: 'j',
    minuteShort: 'm'
  },
  a11y: {
    timerIdle: 'Timer diam',
    timerRunning: 'Timer berjalan',
    timerPaused: 'Timer jeda',
    progress: (percent: number) => `Progres target ${percent} persen`
  },
  monthNames: [
    'Januari',
    'Februari',
    'Maret',
    'April',
    'Mei',
    'Juni',
    'Juli',
    'Agustus',
    'September',
    'Oktober',
    'November',
    'Desember'
  ],
  dayNames: ['Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu', 'Minggu'],
  dayShort: ['Sen', 'Sel', 'Rab', 'Kam', 'Jum', 'Sab', 'Min'],
  clock: (h: number, min: number) => `${two(h)}.${two(min)}`
};

export type Messages = typeof m;
