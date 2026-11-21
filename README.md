# 🚀 doon

> Download and sync all GitHub repositories from any user or organization with a single command — with colored output, size calculation, and confirmation prompts!

```text
✨ 🚀 DOON — GITHUB DOWNLOADER 🚀 ✨
──────────────────────────────────────────────────────────
🎯 Target:      https://github.com/nextcloud
📂 Destination: /Users/mohsen/git
──────────────────────────────────────────────────────────
🏢 Account:     nextcloud (Organization)
🔍 Scanning repositories on GitHub...

┌────────────────────────────────────────────────────────┐
│  📊 DOWNLOAD SUMMARY                                   │
├────────────────────────────────────────────────────────┤
│  🎯 Account:               nextcloud                   │
│  📦 Total Repositories:    248                         │
│  ⬇️  To Clone (new):        240 (18.25 GB)              │
│  🔄 Already Present (pull):8 (1.10 GB)                 │
├────────────────────────────────────────────────────────┤
│  💾 Total Estimated Size:  ~19.35 GB                   │
└────────────────────────────────────────────────────────┘

❓ Do you want to proceed with downloading? [y/N]: y
```

## ✨ Features

- 🔍 **Auto-Detection**: Automatically detects whether the target is an **Organization** or an individual **User**.
- 📄 **Full Pagination**: Fetches *all* repositories regardless of how many exist (Nextcloud, Google, etc.).
- 📊 **Download Size Estimation**: Calculates and formats total size before starting.
- 🔄 **Smart Syncing**: Runs `git clone` for new repositories, and `git pull` for existing ones.
- 🎨 **Rich Terminal Output**: ANSI-colored progress messages with emojis.
- ⚡ **Dual Implementation**: Available both as an instant Python script and a blazing-fast Rust CLI.

---

## 📦 Installation

### Quick Install (Python Script)

```bash
# Clone this repo
git clone https://github.com/dehbag/doon.git
cd doon

# Copy to your PATH
cp doon ~/.local/bin/doon
chmod +x ~/.local/bin/doon
```

### Install with Rust / Cargo

```bash
cd doon
cargo install --path .
```

---

## 🛠️ Usage

```bash
# Basic usage with GitHub URL
doon https://github.com/nextcloud

# Or simply with username/org name
doon nextcloud

# Specify a custom download folder
doon nextcloud -d /path/to/folder

# Auto-confirm without prompt
doon nextcloud -y

# Pass a GitHub Token (avoids 60 req/hr rate limit)
doon nextcloud -t "ghp_yourTokenHere"
```

---

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.
