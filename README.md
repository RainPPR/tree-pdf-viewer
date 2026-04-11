# Tree PDF Viewer

A lightweight PDF viewer with tree-style file navigation, built with Tauri 2, React and Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Features

- **Tree Navigation** — Recursive directory scanning that only shows folders containing PDF files
- **Tabbed Viewing** — Open multiple PDFs with three display modes:
  - **Scroll** — Fixed-width tabs with horizontal scrolling (Chrome/Edge style)
  - **Shrink** — Auto-compress tabs to fit width (VS Code style)
  - **Wrap** — Multi-line layout for many tabs (Firefox style)
- **Memory Management** — Automatic monitoring with configurable limits; oldest inactive tabs are released when memory is exceeded
- **Settings** — Max tabs, memory limit (MB), and tab display mode
- **Cross-Platform** — Windows (WebView2), Linux (WebKitGTK)

## Screenshots

> Coming soon

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop | Tauri 2 |
| Frontend | React 19 + TypeScript + Vite 7 |
| State | Zustand |
| Icons | Lucide React |
| Backend | Rust (sysinfo) |

## Prerequisites

- [Node.js](https://nodejs.org/) >= 22
- [Rust](https://www.rust-lang.org/tools/install) >= 1.80
- **Windows**: WebView2 Runtime (preinstalled on Windows 11)
- **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build frontend only
npm run build
```

## Building

```bash
# Build release for your current platform
npm run tauri build
```

Artifacts are output to `src-tauri/target/release/bundle/`.

## Configuration

### Max Tabs

Maximum number of open PDF tabs (default: 20, range: 1–50).

### Memory Limit

Process memory limit in MB (default: 1024, minimum: 512). When exceeded, the oldest inactive tabs are auto-closed.

### Tab Display Mode

| Mode | Behavior |
|------|----------|
| Scroll | Horizontal scroll when tabs overflow |
| Shrink | Auto-compress tabs to fit width, then scroll at minimum |
| Wrap | Multi-line layout, max 3 rows |

## CI/CD

Pushing a tag like `v1.0.0` triggers the release workflow:

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

This builds Windows (MSI + NSIS) and Linux (deb + AppImage) artifacts and creates a draft GitHub Release with the version auto-updated in `package.json` and `Cargo.toml`.

## License

[MIT](LICENSE)
