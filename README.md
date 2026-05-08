# DigiCode Finder

mDNS-based device discovery helper for [DigiCode](https://github.com/fablab-westharima/digicode)'s WiFi OTA workflow.

A small desktop app that scans the local network for DigiCode-compatible ESP32 devices advertising themselves via mDNS, then surfaces their IP addresses for use in the DigiCode OTA flow. Particularly useful on Windows where Bonjour is not pre-installed.

## Stack

[Tauri v2](https://tauri.app) + Rust + React. Builds to native binaries for macOS, Windows, and Linux.

## Features

- Auto-discovers DigiCode-compatible ESP32 devices on the local network via mDNS (`_arduino._tcp` / `_http._tcp`)
- Multi-language UI (Japanese / English) with manual switch
- Health-check (reachability ping) for discovered devices
- Search timeout adjustable; offline devices excluded after timeout
- Releases distributed via [GitHub Releases](https://github.com/fablab-westharima/DigiCode-Finder/releases)

## Development

```bash
npm install
npm run tauri dev          # local dev with HMR
npm run tauri build        # production binary
```

## License

GNU Affero General Public License version 3 (AGPL-3.0). Copyright © 2024-2026 DigiCo LLC. See [LICENSE](./LICENSE) for the full text.

## DigiCode Ecosystem

- [DigiCode](https://github.com/fablab-westharima/digicode) — Main repo: Blockly-based ESP32 firmware builder.
- [digicode-compile-api](https://github.com/fablab-westharima/digicode-compile-api) — PlatformIO compile server.
- [DigiCode-Finder](https://github.com/fablab-westharima/DigiCode-Finder) — This repo: mDNS device discovery.
- [digicode-installer](https://github.com/fablab-westharima/digicode-installer) — One-command local install for end-users.

## Contact

Digi Co LLC (合同会社デジコ) — contact@digital-fab.jp
