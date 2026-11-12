# App Spec: StudyTracker

Nama kerja: **StudyTracker**. Aplikasi desktop pelacak waktu belajar pribadi, pengganti Yeolpumta, dibangun dengan **Rust**.
Dokumen ini mendefinisikan **apa yang dibangun, bagaimana struktur dan datanya, dan urutan pengerjaannya**. Tampilan dan interaksi ada di `UI_UX_SPEC.md`, yang menjadi sumber kebenaran untuk semua hal visual.

---

## 1. Tujuan

Mencatat setiap sesi belajar dengan cepat (mulai/berhenti per mata pelajaran), lalu menampilkan riwayat dan statistik supaya pengguna tetap konsisten: target harian, streak, D-day ujian.

**Prinsip produk**
- **Untuk satu pengguna.** Tidak ada akun, login, atau fitur sosial.
- **Offline-first.** Semua data tersimpan lokal di satu file SQLite. Aplikasi tidak butuh internet.
- **Ringan.** Cepat dibuka, hemat memori, installer kecil.
- **Data milik pengguna.** Bisa diekspor dan diimpor kapan saja dalam format terbuka (CSV, JSON).

## 2. Ruang lingkup

**Termasuk (v1)**
- Desktop saja (target OS lihat bagian 14).
- Timer per mata pelajaran, riwayat, statistik, target, D-day, ikon tray, pintasan global, mini mode, ekspor/impor.

**Tidak termasuk**
- Android atau iOS.
- Ranking, teman, grup, akun, sinkronisasi cloud.
- Pemblokir aplikasi lain, Pomodoro, dan tema gelap (tema gelap masuk v1.1).

## 3. Fitur dan prioritas

| Fitur | Prioritas |
|---|---|
| Timer mulai/jeda/lanjut/berhenti, per mata pelajaran | MVP |
| Simpan sesi otomatis, pulihkan sesi yang tertinggal | MVP |
| Kelola mata pelajaran (tambah, ubah, warna, arsipkan) | MVP |
| Riwayat per hari, ubah/hapus/tambah sesi manual | MVP |
| Statistik harian/mingguan/bulanan, grafik per mata pelajaran | v1 |
| Target harian dan mingguan, bar progres | v1 |
| D-day ujian | v1 |
| Streak, rekor, heatmap 26 minggu | v1 |
| Ikon tray, pintasan global, notifikasi target tercapai | v1 |
| Mini mode (jendela kecil, opsional selalu di atas) | v1 |
| Deteksi tidak aktif | v1 |
| Mulai otomatis saat komputer menyala | v1 |
| Ekspor/impor CSV dan JSON | v1 |
| Tema gelap | v1.1 |

## 4. Persyaratan jendela

- Jendela utama **bisa diubah ukurannya**: default 1100x720, minimum 420x560.
- Ukuran dan posisi diingat antar sesi (`tauri-plugin-window-state`).
- Hanya satu instance aplikasi yang boleh berjalan (`tauri-plugin-single-instance`). Membuka kedua kali memfokuskan jendela yang sudah ada.
- Mini mode adalah jendela kedua yang bisa diubah ukurannya (minimum 280x140), dengan opsi selalu di atas.
- Menutup jendela bisa diatur: keluar atau menyembunyikan ke tray.
- Layout menyesuaikan lebar jendela. Detail breakpoint ada di `UI_UX_SPEC.md` bagian 5.

## 5. Tech stack

| Lapisan | Pilihan |
|---|---|
| Bahasa inti | Rust (stable), Cargo workspace |
| Shell desktop | Tauri 2 |
| Tampilan | Svelte + TypeScript + Vite |
| Grafik | uPlot atau Chart.js (flat, tanpa gradien; lihat UI spec) |
| Ikon | Lucide |
| Database | SQLite lewat `rusqlite` + `rusqlite_migration` |
| Waktu | `chrono` (simpan UTC, tampilkan waktu lokal) |
| Serialisasi/error/log | `serde`, `thiserror`, `tracing` |
| Plugin Tauri | `global-shortcut`, `notification`, `autostart`, `window-state`, `single-instance`, tray bawaan |
| Deteksi idle | Crate lintas platform (mis. `user-idle`), dibungkus trait agar bisa diganti |
| Test | `cargo test` dengan SQLite in-memory; `vitest` untuk logika frontend |
| Rilis | `tauri build` + GitHub Actions |

Aturan dependensi: jangan menambah library baru tanpa alasan yang ditulis di PR atau commit. Pilih yang kecil dan terawat.

## 6. Arsitektur dan struktur folder

```
studytracker/
├─ APP_SPEC.md
├─ UI_UX_SPEC.md
├─ Cargo.toml              # workspace
├─ crates/
│  └─ core/                # logika murni, tanpa UI dan tanpa Tauri
│     └─ src/
│        ├─ lib.rs
│        ├─ models.rs      # Subject, Session, Goal, Exam
│        ├─ timer.rs       # state machine sesi
│        ├─ db.rs          # koneksi, migrasi, query
│        ├─ stats.rs       # agregasi, streak, heatmap
│        ├─ export.rs      # CSV/JSON
│        └─ error.rs
├─ src-tauri/              # lapisan tipis: command, tray, hotkey, jendela
│  └─ src/
│     ├─ main.rs
│     ├─ commands.rs
│     └─ tray.rs
└─ ui/                     # Svelte
   └─ src/
      ├─ lib/              # komponen, store, i18n
      ├─ routes/           # timer, riwayat, statistik, target, mapel, pengaturan
      └─ styles/tokens.css # token dari UI_UX_SPEC.md
```

**Prinsip:** semua logika bisnis berada di `crates/core` dan diuji tanpa UI. `src-tauri` hanya menjembatani (menerima command, memanggil `core`, mengirim event). Frontend tidak menghitung durasi atau statistik sendiri, kecuali tampilan angka yang berdetak.

## 7. Model data

Semua waktu disimpan sebagai **unix seconds UTC**. Tanggal ujian disimpan sebagai teks `YYYY-MM-DD` (tanggal lokal).

```sql
CREATE TABLE subjects (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  color       TEXT NOT NULL,                 -- nama token: rose|butter|tangerine|lime|mint|aqua|sky|lavender
  archived    INTEGER NOT NULL DEFAULT 0,
  created_at  INTEGER NOT NULL
);

CREATE TABLE sessions (
  id               INTEGER PRIMARY KEY,
  subject_id       INTEGER NOT NULL REFERENCES subjects(id),
  started_at       INTEGER NOT NULL,
  ended_at         INTEGER,                  -- NULL = sedang berjalan atau dijeda
  paused_at        INTEGER,                  -- NULL jika tidak sedang dijeda
  paused_total_sec INTEGER NOT NULL DEFAULT 0,
  duration_sec     INTEGER,                  -- diisi saat selesai
  note             TEXT,
  source           TEXT NOT NULL DEFAULT 'timer'   -- timer | manual
);
-- hanya boleh ada satu sesi aktif
CREATE UNIQUE INDEX one_active_session ON sessions((ended_at IS NULL)) WHERE ended_at IS NULL;
CREATE INDEX idx_sessions_started ON sessions(started_at);

CREATE TABLE goals (
  scope       TEXT PRIMARY KEY CHECK (scope IN ('daily','weekly')),
  target_sec  INTEGER NOT NULL
);

CREATE TABLE exams (
  id         INTEGER PRIMARY KEY,
  title      TEXT NOT NULL,
  exam_date  TEXT NOT NULL
);

CREATE TABLE settings (
  key    TEXT PRIMARY KEY,
  value  TEXT NOT NULL
);
```

Lokasi file database: folder data aplikasi sesuai OS (`app_data_dir` Tauri). Migrasi dijalankan otomatis saat start, dan database dicadangkan otomatis sebelum migrasi yang mengubah skema.

## 8. Logika timer

- **Sumber kebenaran ada di Rust dan database**, bukan di frontend. Saat mulai, baris sesi dibuat dengan `ended_at = NULL`. Dengan begitu crash atau mati listrik tidak menghilangkan sesi.
- **Durasi dihitung dari selisih timestamp**: `duration = ended_at - started_at - paused_total_sec`. Jangan menambah satu detik per tick.
- Jeda: isi `paused_at = now`. Lanjut: `paused_total_sec += now - paused_at`, lalu `paused_at = NULL`.
- Frontend menampilkan angka dengan interval sekitar 250ms dari `started_at` dan `paused_total_sec` yang diberikan backend. Angka ini hanya tampilan.
- Keadaan: `Idle`, `Running`, `Paused`. Transisi tidak valid mengembalikan error yang jelas, bukan panik.
- **Awal hari belajar** (setting `day_start_hour`, default 4): sesi yang dimulai sebelum jam itu dihitung sebagai hari sebelumnya untuk riwayat, target, streak, dan heatmap.
- Sesi yang melewati batas hari disimpan utuh, lalu dipecah per hari hanya saat agregasi statistik.

## 9. API antara frontend dan backend (Tauri commands)

Semua mengembalikan `Result` dengan error berpesan jelas (lihat microcopy di UI spec).

| Kelompok | Command |
|---|---|
| Timer | `get_timer_state`, `start_session(subject_id)`, `pause_session`, `resume_session`, `stop_session`, `undo_stop(session_id)`, `discard_session`, `finish_orphan_session(ended_at)` |
| Sesi | `list_sessions(from, to)`, `add_manual_session`, `update_session`, `delete_session` |
| Mata pelajaran | `list_subjects(include_archived)`, `upsert_subject`, `archive_subject` |
| Statistik | `stats_range(from, to, granularity)`, `subject_breakdown(from, to)`, `heatmap(weeks)`, `streaks` |
| Target dan D-day | `get_goals`, `set_goal`, `list_exams`, `upsert_exam`, `delete_exam` |
| Data | `export_data(format, path)`, `import_data(path)`, `open_data_dir` |
| Pengaturan | `get_settings`, `set_setting(key, value)` |

Backend mengirim event `session-changed` setiap ada perubahan sesi supaya semua jendela (utama, mini, tray) tetap sinkron.

## 10. Kasus tepi yang harus ditangani

- **Aplikasi ditutup/crash saat timer berjalan:** saat dibuka, tampilkan dialog sesi tertinggal (UI spec bagian 9).
- **Komputer tidur:** durasi tetap dari selisih timestamp. Perilaku default diatur setting "hitung waktu saat komputer tidur" (default: tidak dihitung, sesi dijeda otomatis saat tidur).
- **Perubahan zona waktu atau jam sistem:** data UTC tidak berubah, tampilan mengikuti zona waktu lokal. Waktu sistem mundur: tolak `ended_at` lebih kecil dari `started_at` dan minta pengguna memperbaikinya.
- **Sesi sangat panjang (misal lebih dari 12 jam):** tampilkan konfirmasi sebelum menyimpan.
- **Mata pelajaran dengan riwayat:** tidak bisa dihapus, hanya diarsipkan.
- **Sesi manual yang tumpang tindih** dengan sesi lain: izinkan tapi beri peringatan.
- **File database terkunci atau rusak:** pesan error yang menjelaskan cara memperbaiki, dan tawarkan memulihkan dari cadangan otomatis.
- **Impor:** validasi sebelum menulis, jalankan dalam satu transaksi, jangan menimpa data tanpa konfirmasi.

## 11. Non-fungsional (target, ukur dan laporkan)

- Cold start di bawah 1,5 detik pada laptop biasa.
- Memori saat diam di bawah 150 MB.
- Ukuran installer di bawah 15 MB.
- Timer tidak boleh melenceng. Uji dengan sesi berjalan beberapa jam dan dengan sleep/resume.
- Tidak ada panggilan jaringan. Tidak ada telemetri.
- Semua string UI lewat i18n (default Indonesia).

Angka di atas adalah target awal. Jika tidak tercapai, laporkan hasil ukurnya, jangan mengklaim tercapai.

## 12. Fase pengerjaan

| Fase | Isi | Selesai ketika |
|---|---|---|
| 0. Persiapan | Workspace Cargo, Tauri + Svelte kosong berjalan, token CSS dari UI spec, i18n dasar, CI minimal | `cargo test` dan `tauri dev` jalan |
| 1. Inti | `crates/core`: model, migrasi, state machine timer, CRUD sesi dan mata pelajaran | Test unit lulus, termasuk jeda, sesi tertinggal, batas hari |
| 2. Kerangka dan timer | Layout global (rail + area konten), layar Timer dengan tiga keadaan panel, simpan otomatis | Bisa mulai/jeda/berhenti, sesi tersimpan, panel berganti warna |
| 3. Riwayat dan mata pelajaran | Layar Riwayat, ubah/hapus/tambah manual, layar Mata pelajaran | Semua CRUD bisa dari UI |
| 4. Statistik | Agregasi di `core`, grafik, rincian, ringkasan kalimat | Angka cocok dengan data uji |
| 5. Target, D-day, streak, heatmap | Layar Target dan D-day, panel "Hari ini", heatmap | Sesuai UI spec |
| 6. Integrasi desktop | Tray, pintasan global, notifikasi, mini mode, deteksi idle, autostart, single instance | Semua jalan di OS target |
| 7. Responsif dan polish | Tiga kelas layout, skala antarmuka, aksesibilitas, QA visual | Checklist UI spec bagian 12 lulus |
| 8. Data dan rilis | Ekspor/impor, cadangan, pengaturan, ikon, installer, CI rilis | Installer terpasang di mesin bersih |

Setiap fase harus bisa dijalankan dan diuji sendiri sebelum lanjut ke fase berikutnya.

## 13. Aturan kerja untuk AI coding agent

1. Baca `UI_UX_SPEC.md` sebelum menulis kode UI. Jangan mengubah token, radius, atau palet tanpa persetujuan.
2. **Test dulu untuk `crates/core`:** timer, perhitungan durasi, batas hari, streak. Logika ini tidak boleh hanya diuji lewat UI.
3. Kerjakan satu fase pada satu waktu. Commit kecil dengan pesan jelas.
4. Jangan menaruh logika bisnis di frontend atau di `src-tauri`.
5. Jangan memakai gradien, blur shadow, atau emoji sebagai ikon (lihat UI spec bagian 4).
6. Tidak menambah dependensi tanpa alasan tertulis.
7. Bila spesifikasi ambigu atau dua dokumen bertentangan, **berhenti dan tanyakan**, jangan menebak.
8. Setelah tiap fase, tulis ringkasan: apa yang selesai, apa yang diuji, apa yang belum.
9. Kode yang khusus OS harus berada di balik trait atau modul terpisah supaya aplikasi tetap lintas platform.

## 14. Pertanyaan terbuka

- **OS target:** belum ditentukan (Windows, macOS, Linux). Sementara, tulis kode lintas platform dan uji di OS yang dipakai pemilik proyek. Kerjakan fitur khusus OS (deteksi idle, tray, autostart) di OS tersebut lebih dulu.
- **Nama final aplikasi** dan ikon aplikasi.
- **Perilaku default saat komputer tidur** (dijeda otomatis atau tetap dihitung). Default saat ini: dijeda otomatis.
