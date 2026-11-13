# NOVA Client

NOVA Client is a Windows-first Minecraft Java launcher built with React, Vite, Tauri 2 and Rust.

## Current implementation

- Microsoft browser sign-in using authorization code + PKCE; account credentials are stored with the operating system credential manager.
- Minecraft version discovery and launch preparation through the Rust launcher backend.
- Modrinth and CurseForge catalog search.
- Modrinth and CurseForge modpack version selection, download, instance preparation and launch.
- Per-instance metadata stored on disk so installed instances can be rediscovered after launcher storage is cleared.
- A Fabric 26.2 client mod with an early in-game NOVA menu. The module catalogue is still a prototype; most module toggles do not yet implement gameplay/HUD features.

## Known limitations

This project is under active development. Download progress is currently file/phase based rather than byte-accurate, cancellation is not implemented, and some settings and PvP screens remain incomplete. A CurseForge API credential compiled into a desktop executable is extractable by anyone who can inspect that executable; private distribution reduces exposure but does not make a client-embedded secret secure. A production deployment should proxy CurseForge requests through a controlled service.

## Development

```bash
npm install
npm run dev
```

Build the Windows desktop application after installing the Rust toolchain and Tauri prerequisites:

```bash
npm run tauri dev
npm run tauri build
```

The Windows GitHub Actions workflow produces `NOVA-Client-Setup.exe`. The workflow runs on `main` and on `fix/**` branches so repair branches can be validated before merging.

## Safety principles

- Never collect or store Microsoft passwords.
- Validate downloaded archive paths, sizes and hashes before treating a modpack as installed.
- Preserve separate instance folders and prevent deletion while a game process is running.
- Clearly label prototype features instead of presenting visual-only controls as functional modules.
