# NOVA Client

NOVA Client is a Windows-first Minecraft Java launcher built with React, Vite, Tauri 2 and Rust.

## Current implementation

- Microsoft account sign-in using the browser-based authorization-code + PKCE flow.
- Minecraft version discovery and Java/Minecraft preparation through the Rust launcher.
- Isolated instance folders and Modrinth/CurseForge modpack installation.
- Modpack version selection, installation progress events and instance recovery from disk.
- Process supervision, duplicate-launch protection and safe refusal to delete a running instance.
- Windows NSIS installer generation through GitHub Actions.
- A separate early-development Fabric client mod with a NOVA menu.

## Important limitations

- The Fabric menu is an early UI preview. Its module switches are not gameplay features yet; do not treat them as working HUD modules.
- Download cancellation, byte-level progress, streaming archive extraction, complete crash-log viewing and configurable memory allocation are still in development.
- The CurseForge API credential is compiled into private Windows builds. A desktop binary cannot keep an embedded API key secret; distribute builds only to trusted users and rotate the key if exposure is suspected.
- A successful CI build confirms compilation and packaging, not full runtime testing on every Windows machine.

## Development

Install Node.js, the Rust toolchain and the Tauri Windows prerequisites, then run:

```bash
npm install
npm run dev
```

Build the Windows installer:

```bash
npm run tauri build
```

## GitHub Actions

- **Frontend validation** runs the Vite production build.
- **Windows build** creates `NOVA-Client-Setup.exe` and uploads it as the `NOVA-Client-Windows-Installer` artifact.
- **NOVA In-Game Mod Build** builds the Fabric JAR separately.

Keep implemented features, previews and planned work clearly distinguished in the UI and documentation.
