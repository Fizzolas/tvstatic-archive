# Building the GUI

## Desktop (Windows/Linux)

This project uses Tauri v2 for the desktop GUI. Tauri uses the system webview on each platform (WebView2 on Windows). [web:81]

### Prerequisites

- Rust toolchain
- Node.js
- Tauri prerequisites per OS (see Tauri docs) [web:80]

### Run dev

```bash
cd apps/sllv-desktop
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Android (planned)

Android GUI will be added after the core camera/scan pipeline is stabilized.

