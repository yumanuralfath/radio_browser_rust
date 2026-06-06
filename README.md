<div align="center">

```
   ___          ___        ___                               ___
  / _ \___ ____/ (_)__    / _ )_______ _    _____ ___ ____  / _ | ___  ___
 / , _/ _ `/ _  / / _ \  / _  / __/ _ \ |/|/ (_-</ -_) __/ / __ |/ _ \/ _ \
/_/|_|\_,_/\_,_/_/\___/ /____/_/  \___/__,__/___/\__/_/   /_/ |_/ .__/ .__/
                                                               /_/  /_/
```

**Pencarian dan pemutar radio internet langsung dari terminal.**  
Didukung oleh [Radio Browser](https://www.radio-browser.info) · Diputar via `mpv`

[![Crates.io](https://img.shields.io/crates/v/redio?style=flat-square&color=dc5032)](https://crates.io/crates/redio)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square)](https://www.rust-lang.org)

</div>

---

## Fitur

- **TUI interaktif** — antarmuka terminal lengkap dengan panel pencarian, daftar hasil, dan detail stasiun
- **CLI cepat** — cari dan putar radio dalam satu perintah
- **Filter lengkap** — nama, negara, bahasa, tag, codec, bitrate, dan lainnya
- **Auto click** — setiap stasiun yang diputar otomatis tercatat di Radio Browser
- **Vote stasiun** — dukung stasiun favorit langsung dari terminal
- **Shell completions** — Zsh, Bash, Fish, PowerShell

---

## Prasyarat

`mpv` harus terinstal di sistem kamu.

```bash
# Arch Linux
sudo pacman -S mpv

# Ubuntu / Debian
sudo apt install mpv

# Fedora
sudo dnf install mpv

# macOS
brew install mpv
```

Cek apakah semua dependensi sudah siap:

```bash
redio doctor
```

---

## Instalasi

```bash
# Dari crates.io
cargo install redio
```

```bash
# Dari source
git clone https://github.com/YOUR_USERNAME/redio.git
cd redio
cargo install --path .
```

---

## Penggunaan

### TUI (Antarmuka Terminal)

Cara paling nyaman untuk menjelajahi radio:

```bash
redio tui
```

```
┌─ 📻 REDIO  radio browser ────────────────────────────────────────────────────┐
├─ 🔍 Pencarian ──────────┬─ 📋 Hasil Pencarian (20 stasiun) ─────────────────┤
│  Nama Stasiun           │    1.  Jazz FM                         MP3 128kbps │
│  ──────────────         │    2.  BBC World Service               AAC  96kbps │
│  Negara                 │  ▶ 3.  Radio Indonesia          ← playing          │
│  ──────────────         ├─ ℹ Detail Stasiun ──────────────────────────────── ┤
│  Bahasa                 │  Nama     Radio Indonesia                           │
│  ──────────────         │  Negara   Indonesia                                 │
│  Tag/Genre              │  Codec    MP3 / 128 kbps                            │
│  ──────────────         │  Tags     news,talk,indonesian                      │
│  Limit    20            │  Votes    1420                                      │
│                         │  [Enter/p] putar  [v] vote                         │
│  ↵  CARI SEKARANG  ↵   │                                                     │
├─ ▶ Memutar: Radio Indonesia ─ Tab: field  ↵: cari  ↑↓/jk: navigasi  q: exit ┘
```

**Kontrol TUI:**

| Tombol              | Aksi                         |
| ------------------- | ---------------------------- |
| `Tab` / `Shift+Tab` | Pindah antar field pencarian |
| `Enter`             | Cari / Putar stasiun         |
| `↑` `↓` / `j` `k`   | Navigasi daftar              |
| `g` / `G`           | Lompat ke atas / bawah       |
| `p`                 | Putar stasiun terpilih       |
| `v`                 | Vote stasiun terpilih        |
| `/`                 | Kembali ke panel pencarian   |
| `F1`                | Bantuan                      |
| `q` / `Ctrl+C`      | Keluar                       |

---

### CLI

#### Status API

```bash
redio status
```

#### Pencarian

```bash
# Cari berdasarkan nama
redio search --name "jazz"

# Cari berdasarkan negara
redio search --country Indonesia

# Cari berdasarkan bahasa
redio search --language Indonesian

# Cari berdasarkan tag/genre
redio search --tag "lofi"

# Kombinasi filter
redio search --country Japan --language Japanese --tag anime

# Atur jumlah hasil (default: 10)
redio search --name "classical" --limit 20

# Filter lanjutan
redio search --codec MP3 --bitrate-min 128 --bitrate-max 320
```

#### Putar Stasiun

Setiap kali diputar, click otomatis dikirim ke Radio Browser.

```bash
# Putar stasiun pertama dari hasil pencarian
redio search --name "jazz" play

# Putar stasiun ke-3
redio search --name "jazz" play --pick 3

# Putar radio Indonesia berbahasa Indonesia
redio search --country Indonesia --language Indonesian play
```

#### Vote Stasiun

```bash
# Vote stasiun pertama dari hasil pencarian
redio search --name "jazz" vote

# Vote stasiun ke-2
redio search --name "jazz" vote --pick 2

# Vote langsung via UUID
redio vote 960397f0-0c18-4afe-b66d-4e0ca0a3912c
```

---

## Shell Completions

### Zsh

```bash
mkdir -p ~/.zsh/completions
redio completions zsh > ~/.zsh/completions/_redio
```

Tambahkan ke `~/.zshrc`:

```zsh
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

### Bash

```bash
redio completions bash > ~/.local/share/bash-completion/completions/redio
```

### Fish

```bash
redio completions fish > ~/.config/fish/completions/redio.fish
```

### PowerShell

```powershell
redio completions powershell >> $PROFILE
```

---

## Referensi Perintah

```
redio
├── tui                          Antarmuka terminal interaktif
├── status                       Tampilkan status server Radio Browser
├── doctor                       Periksa dependensi (mpv)
├── vote <uuid>                  Vote stasiun via UUID
├── completions <shell>          Generate shell completions
└── search [filter...] [aksi]
    ├── Filter
    │   ├── --name <nama>
    │   ├── --country <negara>
    │   ├── --country-code <kode>
    │   ├── --language <bahasa>
    │   ├── --state <provinsi>
    │   ├── --tag <tag>
    │   ├── --codec <codec>
    │   ├── --bitrate-min <n>
    │   ├── --bitrate-max <n>
    │   └── --limit <n>          (default: 10)
    └── Aksi
        ├── play [--pick <n>]    Putar stasiun ke-n (default: 1)
        └── vote [--pick <n>]    Vote stasiun ke-n (default: 1)
```

---

## Lisensi

[MIT](LICENSE)

---

<div align="center">
Data stasiun disediakan oleh <a href="https://www.radio-browser.info">radio-browser.info</a> — komunitas database radio internet terbuka.
</div>
