# Ginger Code

> macOS desktop development workspace with bundled Neovim, multi-agent coding, and a sarcastic mascot.

## Status

v0.1 — Design candidate for implementation

## Tech Stack

- **Desktop:** Tauri 2 + Rust
- **UI:** React + TypeScript + Zustand
- **Editor:** Bundled Neovim (nvim --embed, Msgpack-RPC)
- **Terminal:** Rust PTY + xterm.js
- **Persistence:** SQLite + filesystem + macOS Keychain

## Getting Started

```bash
# Install dependencies
pnpm install

# Run in dev mode
pnpm tauri dev

# Build for release
pnpm tauri build
```

## License

MIT