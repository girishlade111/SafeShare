# App icons

Placeholder shield mark generated for local development, so `tauri-build` can
produce a Windows resource file and every `bundle.icon` entry resolves.

- `32x32.png`, `128x128.png`, `128x128@2x.png` (256×256)
- `icon.ico` — 16 / 32 / 48 / 256
- `icon.icns` — macOS

Replace with real artwork before shipping. From a single 1024×1024 source:

```bash
npx tauri icon path/to/source.png
```
