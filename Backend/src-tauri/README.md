# WhytChat Backend (Tauri v2)

Minimal Tauri backend serving static frontend from `../Frontend` with a single `health` command.

## Dev

- Ensure Rust toolchain (stable) is installed.
- Install Tauri CLI (optional runtime helper):

```powershell
# optional
cargo install tauri-cli
```

- Run in dev (serves static `Frontend/index.html`):

```powershell
# from Backend/src-tauri
cargo tauri dev
```

## Build

```powershell
cargo tauri build
```

## Config

- `tauri.conf.json` uses:
  - `build.frontendDist: ../Frontend`
  - `app.withGlobalTauri: true` (allow `window.__TAURI__` in plain HTML)
  - Minimal CSP restricting connections to self

## llama-server binary

- Place the Windows binary at `Backend/llama/server/llama-server.exe`.
- The app only looks at this canonical path to keep updates predictable.

## API

- `health() -> { status, version }`
- `download_model() -> string path`
- `start_server()` / `stop_server()`
- `chat({ message }) -> { reply }`
