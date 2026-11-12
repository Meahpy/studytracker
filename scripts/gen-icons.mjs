/**
 * Membuat semua ikon PNG/ICO secara lokal. Tidak ada dependensi gambar:
 * piksel digambar manual, lalu dikemas dengan zlib bawaan Node.
 *
 *   node scripts/gen-icons.mjs
 */

import { deflateSync } from 'node:zlib';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const iconsDir = join(root, 'src-tauri', 'icons');

const COLORS = {
  blush: [0xff, 0xf1, 0xf5],
  raspberry: [0xd9, 0x2d, 0x6a],
  plum: [0x4b, 0x1b, 0x33],
  paused: [0xf6, 0xc9, 0x45],
  white: [0xff, 0xff, 0xff]
};

const SS = 4; // supersampling untuk tepi yang halus

/** Gambar RGBA dengan pensil supersampled. */
function render(size, paint) {
  const big = size * SS;
  const bits = new Uint8Array(big * big); // 0 = transparan, 1 = opaque
  const rgb = new Uint8Array(big * big * 3);
  const set = (x, y, color) => {
    if (x < 0 || y < 0 || x >= big || y >= big) return;
    const index = y * big + x;
    bits[index] = 1;
    rgb[index * 3] = color[0];
    rgb[index * 3 + 1] = color[1];
    rgb[index * 3 + 2] = color[2];
  };

  const disc = (cx, cy, r, color) => {
    const from = Math.floor(cy - r);
    const to = Math.ceil(cy + r);
    for (let y = from; y <= to; y += 1) {
      for (let x = Math.floor(cx - r); x <= Math.ceil(cx + r); x += 1) {
        const dx = x + 0.5 - cx;
        const dy = y + 0.5 - cy;
        if (dx * dx + dy * dy <= r * r) set(x, y, color);
      }
    }
  };

  const ring = (cx, cy, r, width, color) => {
    const inner = r - width;
    for (let y = Math.floor(cy - r); y <= Math.ceil(cy + r); y += 1) {
      for (let x = Math.floor(cx - r); x <= Math.ceil(cx + r); x += 1) {
        const dx = x + 0.5 - cx;
        const dy = y + 0.5 - cy;
        const d = dx * dx + dy * dy;
        if (d <= r * r && d >= inner * inner) set(x, y, color);
      }
    }
  };

  const roundedSquare = (left, top, sizePx, radius, color) => {
    const right = left + sizePx;
    const bottom = top + sizePx;
    for (let y = Math.floor(top); y < Math.ceil(bottom); y += 1) {
      for (let x = Math.floor(left); x < Math.ceil(right); x += 1) {
        const px = x + 0.5;
        const py = y + 0.5;
        const cx = Math.min(Math.max(px, left + radius), right - radius);
        const cy = Math.min(Math.max(py, top + radius), bottom - radius);
        const dx = px - cx;
        const dy = py - cy;
        if (dx * dx + dy * dy <= radius * radius) set(x, y, color);
      }
    }
  };

  const segment = (cx, cy, from, to, angleDeg, width, color) => {
    const radians = (angleDeg * Math.PI) / 180;
    const dx = Math.cos(radians);
    const dy = Math.sin(radians);
    const steps = Math.ceil((to - from) * 2);
    for (let step = 0; step <= steps; step += 1) {
      const t = from + ((to - from) * step) / steps;
      disc(cx + dx * t, cy + dy * t, width / 2, color);
    }
  };

  paint(big, { set, disc, ring, roundedSquare, segment });

  // Turunkan resolusi dengan rata-rata warna tertimbang cakupan.
  const out = Buffer.alloc(size * size * 4);
  for (let y = 0; y < size; y += 1) {
    for (let x = 0; x < size; x += 1) {
      let r = 0;
      let g = 0;
      let b = 0;
      let hits = 0;
      for (let sy = 0; sy < SS; sy += 1) {
        for (let sx = 0; sx < SS; sx += 1) {
          const index = (y * SS + sy) * big + (x * SS + sx);
          if (!bits[index]) continue;
          hits += 1;
          r += rgb[index * 3];
          g += rgb[index * 3 + 1];
          b += rgb[index * 3 + 2];
        }
      }
      const offset = (y * size + x) * 4;
      if (hits === 0) continue;
      out[offset] = Math.round(r / hits);
      out[offset + 1] = Math.round(g / hits);
      out[offset + 2] = Math.round(b / hits);
      out[offset + 3] = Math.round((hits / (SS * SS)) * 255);
    }
  }
  return out;
}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([length, body, crc]);
}

function png(size, paint) {
  const pixels = render(size, paint);
  const raw = Buffer.alloc((size * 4 + 1) * size);
  for (let y = 0; y < size; y += 1) {
    raw[y * (size * 4 + 1)] = 0; // filter none
    pixels.copy(raw, y * (size * 4 + 1) + 1, y * size * 4, (y + 1) * size * 4);
  }
  const header = Buffer.alloc(13);
  header.writeUInt32BE(size, 0);
  header.writeUInt32BE(size, 4);
  header[8] = 8; // bit depth
  header[9] = 6; // RGBA
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0))
  ]);
}

/** Jam: satu lingkaran dan dua jarum. */
function clock(background, face, hand) {
  return (size, { disc, ring, segment }) => {
    const unit = size / 64;
    if (background) disc(size / 2, size / 2, size / 2, background);
    ring(size / 2, size / 2, 22 * unit, 3.5 * unit, face);
    segment(size / 2, size / 2, 0, 12 * unit, -90, 3.5 * unit, hand);
    segment(size / 2, size / 2, 0, 8.5 * unit, 0, 3.5 * unit, hand);
  };
}

/** Ikon aplikasi: ubin membulat raspberry dengan jam putih. */
function appIcon() {
  return (size, tools) => {
    const unit = size / 64;
    tools.roundedSquare(0, 0, size, 14 * unit, COLORS.raspberry);
    tools.ring(size / 2, size / 2, 20 * unit, 3.4 * unit, COLORS.white);
    tools.segment(size / 2, size / 2, 0, 10.5 * unit, -90, 3.4 * unit, COLORS.white);
    tools.segment(size / 2, size / 2, 0, 7.5 * unit, 0, 3.4 * unit, COLORS.white);
  };
}

function ico(pngBuffers) {
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);
  header.writeUInt16LE(1, 2);
  header.writeUInt16LE(pngBuffers.length, 4);

  let offset = 6 + pngBuffers.length * 16;
  const entries = [];
  for (const { size, data } of pngBuffers) {
    const entry = Buffer.alloc(16);
    entry[0] = size >= 256 ? 0 : size;
    entry[1] = size >= 256 ? 0 : size;
    entry.writeUInt16LE(1, 4);
    entry.writeUInt16LE(32, 6);
    entry.writeUInt32LE(data.length, 8);
    entry.writeUInt32LE(offset, 12);
    entries.push(entry);
    offset += data.length;
  }
  return Buffer.concat([header, ...entries, ...pngBuffers.map((item) => item.data)]);
}

mkdirSync(iconsDir, { recursive: true });

const targets = [
  ['32x32.png', 32, appIcon()],
  ['128x128.png', 128, appIcon()],
  ['128x128@2x.png', 256, appIcon()],
  ['icon.png', 512, appIcon()],
  ['tray-idle.png', 32, clock(null, COLORS.raspberry, COLORS.raspberry)],
  ['tray-running.png', 32, clock(COLORS.raspberry, COLORS.white, COLORS.white)],
  ['tray-paused.png', 32, clock(COLORS.paused, COLORS.plum, COLORS.plum)]
];

for (const [name, size, paint] of targets) {
  const data = png(size, paint);
  writeFileSync(join(iconsDir, name), data);
  console.log(`${name} ${size}x${size} ${data.length} B`);
}

const icoData = ico([
  { size: 32, data: png(32, appIcon()) },
  { size: 256, data: png(256, appIcon()) }
]);
writeFileSync(join(iconsDir, 'icon.ico'), icoData);
console.log(`icon.ico ${icoData.length} B`);
