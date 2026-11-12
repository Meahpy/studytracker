# UI/UX Spec: StudyTracker

Nama kerja: **StudyTracker** (nama final belum ditentukan).
Dokumen ini adalah sumber kebenaran untuk **tampilan dan interaksi**. Fitur, data, dan arsitektur ada di `APP_SPEC.md`. Kalau dua dokumen bertentangan, tanyakan ke pemilik proyek, jangan menebak.

---

## 1. Konteks

- **Pengguna:** satu orang (pemilik proyek) yang belajar sendiri dan ingin mencatat waktu belajarnya di desktop. Pengganti aplikasi Yeolpumta.
- **Tugas utama:** memulai dan menghentikan timer belajar dengan cepat, lalu melihat berapa lama sudah belajar hari ini.
- **Tugas kedua:** melihat riwayat dan statistik untuk menjaga konsistensi (target harian, streak, D-day ujian).
- **Bahasa antarmuka:** Indonesia. Semua teks lewat file i18n, jangan menulis string langsung di komponen.
- **Konteks pakai:** jendela sering dibiarkan terbuka di samping materi belajar, jadi ukuran jendela bervariasi, dari kecil sampai layar penuh.

## 2. Prinsip desain

1. **Satu hal yang paling menonjol: panel timer.** Seluruh panel berganti warna menurut keadaan (diam, berjalan, jeda). Sisa antarmuka dibuat tenang supaya panel ini jadi pusat perhatian.
2. **Kedalaman lewat lapisan warna, bukan bayangan.** Tidak ada blur shadow. Lapisan dibedakan dengan tingkat warna pink dan garis tipis.
3. **Radius mengikuti hierarki.** Elemen yang lebih besar dan lebih penting memiliki sudut lebih bulat. Tidak ada satu radius yang dipakai di semua tempat.
4. **Putih berarti "bisa diisi".** Warna putih (`--paper`) hanya dipakai untuk kolom input dan popover. Semua permukaan lain berwarna pink.
5. **Teks seperlunya.** Kalimat biasa (sentence case), kata kerja aktif, tanpa basa-basi. Nama aksi sama di seluruh alur.

## 3. Design tokens

### 3.1 Warna

Basis: **pink**. Pink dipakai luas (latar, permukaan, aksi utama). Warna lain hanya sebagai penanda mata pelajaran dan status.

**Pink dan netral berwarna pink**

| Token | Hex | Peran |
|---|---|---|
| `--blush` | `#FFF1F5` | Latar aplikasi |
| `--petal` | `#FFE3EC` | Permukaan (rail, panel, panel timer saat diam) |
| `--petal-strong` | `#FFD0DF` | Permukaan terangkat, hover, tombol sekunder |
| `--line` | `#F4B6CB` | Garis dan border |
| `--rose` | `#FF9DBB` | Penekanan dekoratif, mata pelajaran "rose" |
| `--raspberry` | `#D92D6A` | Aksi utama, panel timer saat berjalan |
| `--raspberry-press` | `#B81F55` | Keadaan ditekan tombol utama |
| `--plum` | `#4B1B33` | Teks utama |
| `--mauve` | `#7A5B69` | Teks sekunder |
| `--paper` | `#FFFFFF` | Hanya untuk input dan popover |

Kontras sudah dicek: teks putih di atas `--raspberry` sekitar 4,6:1, `--plum` dan `--mauve` di atas `--blush` dan `--petal` di atas 4,9:1. Jangan mengubah pasangan ini tanpa mengukur ulang.

**Warna mata pelajaran** (8 pilihan, teks di atasnya selalu `--plum`)

| Token | Hex |
|---|---|
| `--s-rose` | `#FF9DBB` |
| `--s-butter` | `#F6C945` |
| `--s-tangerine` | `#FF9A62` |
| `--s-lime` | `#B5D95A` |
| `--s-mint` | `#5FD0A5` |
| `--s-aqua` | `#5CCBD6` |
| `--s-sky` | `#6FB7F0` |
| `--s-lavender` | `#A68DF2` |

Warna mata pelajaran selalu disertai nama atau label. Jangan mengandalkan warna saja.

**Status**

| Token | Hex | Pakai untuk |
|---|---|---|
| `--ok` | `#17694B` | Target tercapai, berhasil |
| `--warn` | `#8A5A00` | Peringatan |
| `--danger` | `#B42318` | Error, hapus |
| `--paused` | `#F6C945` | Latar panel timer saat jeda (sama dengan butter) |

**Skala heatmap** (5 tingkat datar, bukan gradasi)

`--heat-0 #FFDDE8`, `--heat-1 #FFB8CF`, `--heat-2 #FF8AB0`, `--heat-3 #F0568A`, `--heat-4 #D92D6A`
Pemetaan berdasarkan persen dari target harian: 0%, kurang dari 25%, kurang dari 50%, kurang dari 100%, 100% atau lebih.

**Tema gelap** (prioritas v1.1, tapi token disiapkan dari awal)

`--blush #2A1220`, `--petal #3B1A2D`, `--petal-strong #4D2339`, `--line #6B3550`, `--plum #FFE9F0` (teks), `--mauve #D9AFC2`, `--raspberry #FF6FA3` (teks di atasnya `#2A1220`), `--paper #4D2339`.

### 3.2 Tipografi

Dua keluarga huruf, **disematkan lokal** (file `woff2` di dalam aplikasi, jangan memuat dari CDN karena aplikasi harus jalan offline). Keduanya berlisensi OFL.

| Peran | Font | Bobot |
|---|---|---|
| Tampilan: angka timer, judul layar | **Bricolage Grotesque** | 600, 700 |
| Isi: UI, label, daftar | **Figtree** | 400, 500, 600 |

Fallback: `system-ui, sans-serif`.

Skala (px): `12` keterangan, `14` kecil, `16` isi, `20` subjudul, `28` judul layar, `clamp(56px, 14cqi, 144px)` angka timer.
Angka selalu `font-variant-numeric: tabular-nums`. Jika font tidak mendukung angka tabular, render tiap digit dalam kotak berlebar tetap supaya timer tidak bergoyang.
Panjang baris teks maksimal 70 karakter. Tinggi baris isi 1.5, judul 1.15.

### 3.3 Radius

| Token | Nilai | Dipakai untuk |
|---|---|---|
| `--r-sm` | 8px | Tag kecil, sel heatmap (4px), indikator |
| `--r-md` | 14px | Input, tombol sekunder, baris daftar, tooltip |
| `--r-lg` | 22px | Panel dan kelompok konten |
| `--r-xl` | 32px | Panel timer, dialog |
| `--r-pill` | 999px | Tombol utama, chip, segmented control, toggle, toast |

Radius elemen di dalam kontainer = radius kontainer dikurangi padding, minimal 8px.

### 3.4 Spasi, garis, ikon

- Spasi berbasis 4px: `4, 8, 12, 16, 24, 32, 48`. Padding panel 24px (16px di layout compact).
- Border: `1.5px solid var(--line)`. Panel yang berada di atas `--blush` memakai fill `--petal` tanpa border.
- Ikon: **Lucide**, stroke 1.75 sampai 2px, ujung bulat, satu keluarga ikon saja. Menu navigasi selalu ikon **dan** label (label disembunyikan hanya di rail sempit, dengan tooltip).
- Target klik minimal 44x44px.

### 3.5 Gerak

- Hanya gerak sebagai **respons aksi**: perubahan warna panel timer 150ms ease-out, buka/tutup dialog 120ms, toast muncul.
- Tidak ada animasi masuk otomatis, tidak ada hover transition di setiap kartu, tidak ada efek parallax.
- Angka timer tidak dianimasikan.
- Hormati `prefers-reduced-motion`: matikan semua transisi.

### 3.6 Variabel CSS siap pakai

```css
:root {
  --blush:#FFF1F5; --petal:#FFE3EC; --petal-strong:#FFD0DF; --line:#F4B6CB;
  --rose:#FF9DBB; --raspberry:#D92D6A; --raspberry-press:#B81F55;
  --plum:#4B1B33; --mauve:#7A5B69; --paper:#FFFFFF;
  --ok:#17694B; --warn:#8A5A00; --danger:#B42318; --paused:#F6C945;
  --s-rose:#FF9DBB; --s-butter:#F6C945; --s-tangerine:#FF9A62; --s-lime:#B5D95A;
  --s-mint:#5FD0A5; --s-aqua:#5CCBD6; --s-sky:#6FB7F0; --s-lavender:#A68DF2;
  --heat-0:#FFDDE8; --heat-1:#FFB8CF; --heat-2:#FF8AB0; --heat-3:#F0568A; --heat-4:#D92D6A;
  --r-sm:8px; --r-md:14px; --r-lg:22px; --r-xl:32px; --r-pill:999px;
  --font-display:"Bricolage Grotesque", system-ui, sans-serif;
  --font-body:"Figtree", system-ui, sans-serif;
}
[data-theme="dark"] {
  --blush:#2A1220; --petal:#3B1A2D; --petal-strong:#4D2339; --line:#6B3550;
  --plum:#FFE9F0; --mauve:#D9AFC2; --raspberry:#FF6FA3; --raspberry-press:#FF8AB5;
  --paper:#4D2339;
}
```

## 4. Aturan anti-"AI slop"

**Dilarang:**
- Gradien dalam bentuk apa pun (`linear-gradient`, `radial-gradient`, `conic-gradient`), termasuk untuk latar, tombol, grafik, dan border.
- Blur shadow, glow, glassmorphism, `backdrop-filter`. Kedalaman hanya dari lapisan warna dan garis.
- Emoji sebagai ikon. Maskot, ilustrasi stok, percikan atau bintang dekoratif.
- Label HURUF KAPITAL dan teks kecil di atas judul (eyebrow). Gunakan sentence case.
- Rangkaian meta dengan titik tengah ("A · B · C") dan tanda panah di akhir teks tombol atau tautan.
- Grid kartu identik dengan ukuran dan radius sama. Konten dikelompokkan sesuai hierarkinya.
- Penomoran (01, 02, 03) kecuali isinya memang urutan langkah.
- Kotak statistik "angka besar + label kecil" berulang. Ringkasan ditulis sebagai kalimat (lihat layar Statistik).
- Menandai satu kata di judul dengan warna atau miring berbeda.

**Wajib:**
- Hanya warna dari token di atas. Tidak ada hex baru tanpa persetujuan.
- Teks isi selalu sentence case dan kata kerja aktif.
- Satu keluarga ikon, satu skala radius, satu skala spasi.

## 5. Jendela dan responsivitas

Jendela **bisa diubah ukurannya** dengan bebas. Konfigurasi jendela utama (Tauri):

```json
{ "resizable": true, "width": 1100, "height": 720, "minWidth": 420, "minHeight": 560, "center": true, "decorations": true }
```

- Ingat ukuran dan posisi jendela antar sesi (`tauri-plugin-window-state`).
- Gunakan titlebar bawaan OS di v1.
- Layout berubah berdasarkan **lebar jendela**, bukan perangkat. Pakai CSS container query untuk komponen, bukan hanya media query.

| Kelas | Lebar jendela | Layout |
|---|---|---|
| Expanded | 1100px ke atas | Rail berlabel 220px, area utama, panel "Hari ini" di kanan 320px |
| Medium | 760 sampai 1099px | Rail ikon 72px, area utama, panel "Hari ini" pindah ke bawah panel timer |
| Compact | di bawah 760px | Tab bar di bawah (5 ikon), satu kolom, padding 16px |
| Mini mode | jendela terpisah 280 sampai 360px | Hanya timer (lihat 7.7) |

Aturan umum: konten tidak pernah menimbulkan scroll horizontal. Tabel dan grafik yang lebar menyusut atau berpindah ke tampilan daftar. Angka timer mengikuti lebar panelnya (`cqi`).

## 6. Layout global

```
Expanded (>= 1100px)
+----------+-------------------------------------+----------------+
| Timer    |  [Matematika] [Fisika] [Bahasa] [+] |  Hari ini      |
| Riwayat  |                                     |  3 j 20 m      |
| Statistik|  +-------------------------------+  |  [====----] 67%|
| Target   |  |                               |  |                |
| Mapel    |  |         01:24:09              |  |  Sesi          |
|          |  |                               |  |  Matematika    |
| Pengatur |  |      ( Mulai belajar )        |  |  Fisika        |
|          |  +-------------------------------+  |                |
+----------+-------------------------------------+----------------+

Compact (< 760px)
+-------------------------------+
| [Matematika] [Fisika] [+]     |
| +---------------------------+ |
| |        01:24:09           | |
| |    ( Mulai belajar )      | |
| +---------------------------+ |
| Hari ini: 3 j 20 m, 67%       |
| Sesi hari ini ...             |
+-------------------------------+
| Timer Riwayat Stat Target Lain|
+-------------------------------+
```

Perataan: konten rata kiri. Hanya angka timer dan tombol di dalam panel timer yang rata tengah.
Rail memakai fill `--petal`, item aktif memakai fill `--petal-strong` berbentuk pill dengan teks `--plum` tebal. Pengaturan ditempatkan di dasar rail.

## 7. Layar

### 7.1 Timer (layar awal)

Isi, berurutan dari atas:
1. **Pemilih mata pelajaran:** baris chip pill. Chip terpilih memakai fill warna mata pelajarannya dan outline 2px `--plum`. Chip terakhir "Mata pelajaran baru" memakai ikon plus dan garis putus-putus `--line`. Chip berpindah baris jika sempit. Saat timer berjalan, chip dikunci (ganti mata pelajaran = berhenti lalu mulai lagi).
2. **Panel timer** (`--r-xl`), pusat layar:
   - **Diam:** fill `--petal`, angka `--plum` menampilkan `00:00:00`, tombol utama "Mulai belajar" (pill `--raspberry`, teks putih, tinggi 64px).
   - **Berjalan:** fill **`--raspberry`**, angka putih, nama mata pelajaran di atas angka. Tombol "Berhenti" berupa pill putih dengan teks `--raspberry`, tombol "Jeda" berupa pill dengan outline putih.
   - **Jeda:** fill **`--paused`**, angka `--plum`, tombol "Lanjut" dan "Berhenti".
   - Di bawah angka: waktu mulai ("Mulai 09.12") dan, bila ada target, sisa target hari ini.
3. **Panel "Hari ini"** (`--r-lg`, fill `--petal`): total belajar hari ini sebagai kalimat besar ("3 jam 20 menit"), bar progres target (track `--petal-strong`, isi `--raspberry`, ujung bulat), lalu daftar sesi hari ini (baris `--r-md` dengan titik warna mata pelajaran, nama, rentang jam, durasi).

Saat berhenti: sesi **langsung tersimpan**, tanpa dialog. Muncul toast pill selama 8 detik: "Sesi tersimpan. Tambah catatan | Urungkan".
Saat target harian tercapai: toast sekali, "Target hari ini tercapai", dan bar progres berubah ke `--ok`.

### 7.2 Riwayat

- Kiri (atau atas pada layout medium/compact): kalender bulan. Hari yang punya sesi memakai fill skala heatmap, hari terpilih punya outline `--plum` 2px. Panah bulan sebelumnya/berikutnya.
- Kanan: daftar sesi pada hari terpilih, dengan total di judul ("Selasa, 10 November, 4 jam 5 menit").
- Tiap baris: titik warna, mata pelajaran, rentang jam, durasi, catatan (satu baris, terpotong). Klik baris membuka dialog **Ubah sesi** (mata pelajaran, jam mulai, jam selesai, catatan, tombol "Simpan perubahan" dan "Hapus sesi").
- Tombol "Tambah sesi manual" di atas daftar.
- Kosong: "Belum ada sesi di hari ini."

### 7.3 Statistik

Urutan dari atas:
1. **Segmented control** pill: Hari, Minggu, Bulan. Dengan tombol periode sebelumnya/berikutnya.
2. **Ringkasan dalam satu kalimat**, mis. "Minggu ini kamu belajar 18 jam 20 menit, naik 2 jam dari minggu lalu." Ukuran 20px, bukan kotak angka.
3. **Grafik batang** per hari, bertumpuk per mata pelajaran. Batang bagian atas berujung bulat 6px, jarak antar-batang 8px, tanpa grid vertikal, grid horizontal `--line` tipis. Garis target harian putus-putus `--raspberry`. Tooltip: popover putih `--r-md`.
4. **Rincian per mata pelajaran:** baris dengan titik warna, nama, durasi, bar horizontal datar.
5. **Heatmap 26 minggu** (sel `--r-sm`/4px, jarak 3px, skala heat-0 sampai heat-4). Legenda skala ditulis dengan kata: "Kurang" sampai "Target tercapai".
6. **Streak dan rekor** sebagai daftar biasa: streak saat ini, streak terpanjang, hari terpanjang, minggu terpanjang.

Kosong: "Statistik muncul setelah kamu menyelesaikan sesi pertama."

### 7.4 Target dan D-day

- **Target:** dua baris (harian, mingguan), masing-masing dengan stepper jam dan menit. Perubahan tersimpan otomatis.
- **D-day:** daftar ujian terurut tanggal terdekat. Setiap baris: judul, tanggal, dan sisa waktu ("23 hari lagi", "Hari ini", "Terlewat 2 hari"). Tombol "Tambah D-day" membuka dialog (judul, tanggal).
- Kosong: "Belum ada D-day. Tambahkan tanggal ujian untuk melihat hitung mundur."

### 7.5 Mata pelajaran

- Daftar semua mata pelajaran dengan titik warna, nama, total jam, tombol ubah.
- Dialog ubah/tambah: nama, pilihan warna (8 swatch bulat, terpilih diberi cincin `--plum`), "Simpan".
- Mata pelajaran yang punya sesi **tidak bisa dihapus**, hanya "Arsipkan". Yang diarsipkan hilang dari pemilih tapi tetap muncul di riwayat.

### 7.6 Pengaturan

Dikelompokkan dengan judul biasa (bukan huruf kapital):
- **Tampilan:** tema (Terang, Gelap, Ikuti sistem), skala antarmuka (90%, 100%, 110%, 125%).
- **Timer:** awal hari belajar (default 04.00, supaya belajar lewat tengah malam tetap dihitung hari sebelumnya), deteksi tidak aktif (mati, 5, 10, 15 menit), hitung waktu saat komputer tidur (ya/tidak), pintasan global.
- **Jendela:** tutup ke tray, mulai saat komputer menyala, ingat ukuran jendela.
- **Data:** ekspor CSV, ekspor JSON, impor, buka folder data.

Toggle berbentuk pill. Pengaturan tersimpan seketika, tanpa tombol "Simpan".

### 7.7 Mini mode

Jendela kecil terpisah (default 300x160, bisa diubah ukuran, minimal 280x140), opsional **selalu di atas**. Isi: nama mata pelajaran, angka timer, satu tombol (Mulai/Jeda), ikon untuk kembali ke jendela penuh. Warna mengikuti keadaan sama seperti panel timer. Dibuka dari tombol di layar Timer atau menu tray.

## 8. Komponen

| Komponen | Spesifikasi |
|---|---|
| Tombol utama | Pill, fill `--raspberry`, teks putih 600, tinggi 48px (64px di panel timer), ditekan: `--raspberry-press` |
| Tombol sekunder | `--r-md`, fill `--petal-strong`, teks `--plum` |
| Tombol teks | Tanpa fill, teks `--raspberry` tebal, garis bawah saat hover |
| Tombol bahaya | Outline `--danger` 1.5px, teks `--danger`; fill `--danger` hanya di dialog konfirmasi |
| Chip | Pill, tinggi 36px, fill warna mata pelajaran, teks `--plum` |
| Segmented control | Wadah pill `--petal-strong`, segmen aktif pill `--raspberry` teks putih |
| Input | Fill `--paper`, border 1.5px `--line`, `--r-md`, tinggi 44px; fokus: cincin 3px `--plum` dengan jarak 2px |
| Toggle | Pill 44x26, mati `--line`, hidup `--raspberry`, kenop putih bulat |
| Baris daftar | `--r-md`, hover `--petal-strong`, tinggi minimal 56px |
| Dialog | `--r-xl`, fill `--blush`, border `--line`, lebar maks 480px; scrim `--plum` datar opasitas 40% |
| Toast | Pill, fill `--plum`, teks putih, muncul di bawah tengah, aksi berupa tombol teks |
| Tooltip/popover | Fill `--paper`, border `--line`, `--r-md`, tanpa bayangan |
| Bar progres | Tinggi 12px, pill, track `--petal-strong`, isi `--raspberry` |

## 9. Interaksi dan pintasan

- **Spasi:** mulai/jeda/lanjut timer (saat fokus tidak berada di input).
- **Ctrl+Enter:** berhenti dan simpan sesi.
- **Ctrl+1 sampai 5:** pindah layar. **Esc:** tutup dialog.
- **Pintasan global** (bisa diubah di Pengaturan): mulai/jeda dari aplikasi mana pun. Default `Ctrl+Alt+S`.
- **Ikon tray:** menampilkan keadaan (diam/berjalan/jeda). Menu: Mulai/Jeda, Berhenti, Buka StudyTracker, Mini mode, Keluar.
- **Deteksi tidak aktif:** saat melewati ambang, tampilkan dialog "Kamu tidak aktif selama 10 menit. Hitung waktu itu sebagai belajar?" dengan tombol "Hitung" dan "Buang 10 menit".
- **Sesi tertinggal** (aplikasi ditutup saat timer berjalan): saat dibuka lagi, dialog "Ada sesi yang belum selesai sejak 09.12. Selesaikan pada jam berapa?" dengan input jam dan tombol "Simpan sesi" dan "Buang sesi".

## 10. Microcopy

Aturan: sentence case, kata kerja aktif, satu aksi satu nama di seluruh alur, error menjelaskan masalah **dan** cara memperbaikinya, tidak meminta maaf.

| Konteks | Teks |
|---|---|
| Tombol mulai | Mulai belajar |
| Tombol jeda / lanjut | Jeda / Lanjut |
| Tombol berhenti | Berhenti |
| Toast setelah berhenti | Sesi tersimpan |
| Aksi pada toast | Tambah catatan, Urungkan |
| Simpan edit | Simpan perubahan |
| Hapus | Hapus sesi (konfirmasi: "Hapus sesi Matematika 09.12 sampai 10.40? Sesi tidak bisa dikembalikan.") |
| Target tercapai | Target hari ini tercapai |
| Error simpan | Sesi belum tersimpan. File data sedang dipakai program lain. Tutup program itu, lalu coba lagi. |
| Error impor | File tidak bisa dibaca. Pastikan file hasil ekspor StudyTracker berformat JSON atau CSV. |

## 11. Aksesibilitas

- Kontras teks minimal 4,5:1 (ukuran besar 3:1). Cek ulang bila ada warna baru.
- Semua kontrol bisa dijangkau keyboard dengan urutan tab logis dan cincin fokus selalu terlihat.
- Informasi tidak hanya dibawa warna: mata pelajaran punya nama, status timer punya teks ("Berjalan", "Jeda") selain warna panel.
- Pembaca layar: timer diumumkan lewat `aria-live="off"` saat berjalan (jangan membacakan tiap detik), dengan pengumuman saat mulai, jeda, dan berhenti.
- Hormati `prefers-reduced-motion` dan tema sistem.
- Skala antarmuka 90 sampai 125% tidak boleh merusak layout.

## 12. Checklist QA visual

- [ ] Tidak ada gradien, blur shadow, atau glow di CSS (cari `gradient`, `box-shadow`, `backdrop-filter`).
- [ ] Hanya token warna di bagian 3.1 yang dipakai (cari hex di luar token).
- [ ] Radius sesuai hierarki, bukan satu nilai untuk semua.
- [ ] Layout benar di 420x560, 760x600, 1100x720, dan layar penuh. Tidak ada scroll horizontal.
- [ ] Panel timer berganti warna sesuai tiga keadaan, angka tidak bergoyang saat berubah.
- [ ] Tidak ada label huruf kapital, eyebrow, tanda panah di tombol, atau emoji sebagai ikon.
- [ ] Semua string ada di file i18n.
- [ ] Navigasi keyboard lengkap dan fokus terlihat.
