<div align="center">

# DrawOver

### Draw on any screen. Zero lag.

[![CI](https://github.com/codecoradev/drawover/actions/workflows/ci.yml/badge.svg)](https://github.com/codecoradev/drawover/actions/workflows/ci.yml)
[![Release](https://github.com/codecoradev/drawover/actions/workflows/release.yml/badge.svg)](https://github.com/codecoradev/drawover/releases)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![macOS](https://img.shields.io/badge/platform-macOS-000000?logo=apple&logoColor=white)](#install)

A blazing-fast macOS screen annotation tool built with Rust and Tauri 2.

Draw on any window, any app, any screen — instantly.

</div>

---

## ✨ Features

DrawOver is designed to be the fastest, lightest screen annotation tool on macOS. No bloat, no lag, just drawing.

### MVP Features

| Feature | Description |
|---------|-------------|
| 🖊️ **Pen** | Smooth, pressure-responsive freehand drawing |
| 🖌️ **Highlighter** | Semi-transparent strokes for emphasis |
| 🧹 **Eraser** | Remove individual strokes or clear all |
| ↩️ **Undo / Redo** | Full history — never lose your flow |
| 🎨 **4 Colors** | Quick-switch palette (red, yellow, green, blue) |

### Why DrawOver?

- **Zero lag** — Powered by a custom Rust rendering engine using [`tiny-skia`](https://github.com/RazrFalcon/tiny-skia)
- **Native feel** — Built with Tauri 2 for a tiny binary and low memory usage
- **Minimal UI** — Floating toolbar that gets out of your way
- **Always on top** — Draw over any application without switching windows

## 🗺️ Roadmap

- [ ] **Shapes** — Rectangle, circle, arrow, line
- [ ] **Text tool** — Type annotations directly on screen
- [ ] **Cursor spotlight** — Highlight your cursor for presentations
- [ ] **Laser pointer** — Animated pointer for screen sharing
- [ ] **Whiteboard mode** — Blank canvas overlay
- [ ] **Screenshot capture** — Save your annotated screen
- [ ] **Keyboard shortcuts** — Fully keyboard-driven workflow
- [ ] **Custom color picker** — Beyond the default 4 colors

> Have an idea? [Open a feature request](https://github.com/codecoradev/drawover/issues/new?template=feature_request.md)!

## 💾 Install

### Homebrew (recommended)

```bash
brew install codecoradev/tap/drawover
```

### Download

Download the latest `.dmg` from the [Releases page](https://github.com/codecoradev/drawover/releases), then drag DrawOver to your Applications folder.

> **Note:** On first launch, macOS may show a security prompt. Right-click the app → **Open** to allow it. You'll also be prompted to grant Screen Recording permission in System Settings → Privacy & Security.

## 🛠️ Development

### Prerequisites

- **macOS** 12+ (required — DrawOver uses macOS-specific APIs)
- [**Rust**](https://rustup.rs/) (stable)
- [**bun**](https://bun.sh/) (latest)

### Quick Start

```bash
git clone https://github.com/codecoradev/drawover.git
cd drawover

# Install dependencies
bun install

# Launch dev build with hot reload
bun run tauri dev
```

### Useful Commands

```bash
# Format Rust code
cargo fmt

# Lint
cargo clippy -- -D warnings

# Run tests
cargo test

# Type-check frontend
bun run check

# Build for release
cargo tauri build --target universal-apple-darwin
```

## 🤝 Contributing

Contributions are welcome! Whether it's a bug fix, a new feature, or just a typo — we appreciate every contribution.

👉 See **[CONTRIBUTING.md](CONTRIBUTING.md)** for the full guide.

Before your first PR, you'll need to sign our [Contributor License Agreement](CLA.md) (one-time, quick process).

## 🔧 Built With

| Technology | Role |
|-----------|------|
| [**Rust**](https://www.rust-lang.org/) | Core logic, rendering engine, screen overlay |
| [**Tauri 2**](https://v2.tauri.app/) | Cross-platform desktop framework, native window management |
| [**Svelte 5**](https://svelte.dev/) | Frontend UI — toolbar, canvas state, interactions |
| [**tiny-skia**](https://github.com/RazrFalcon/tiny-skia) | CPU-accelerated 2D rendering — zero GPU dependency, instant startup |

## 📄 License

DrawOver is licensed under the **Apache License 2.0**. See [LICENSE](LICENSE) for the full text.

Contributors must sign the [CLA](CLA.md) before contributions can be accepted.

## 💬 Community

- [Report a bug](https://github.com/codecoradev/drawover/issues/new?template=bug_report.md)
- [Request a feature](https://github.com/codecoradev/drawover/issues/new?template=feature_request.md)
- [Start a discussion](https://github.com/codecoradev/drawover/discussions)

---

<div align="center">

Built with ❤️ by [**CodeCoraDev**](https://github.com/codecoradev)

Draw on any screen. Zero lag.

</div>
