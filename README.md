# NOVA Client

NOVA Client is a Windows-first Minecraft Java launcher built with React, Vite, Tauri 2 and Rust.

## Current implementation

- Microsoft browser sign-in using authorization code + PKCE. Account credentials are stored with the operating system credential manager.
- Minecraft version discovery, loader preparation and launch, with configurable memory allocation.
- Modrinth and CurseForge mod and modpack catalogues, icons and version selection.
- Standalone mod installation into a selected Fabric, Forge, NeoForge or Quilt instance.
- Modpack installation with streamed archive/file downloads, byte progress, cancellation, size/hash validation, bounded extraction and recovery of installed instances from disk.
- Game process tracking, duplicate-launch protection and protection against deleting a running instance.
- A Fabric 26.2 client mod with working FPS, Keystrokes, Armor Status and Clock HUD modules. Module toggles and per-module background preferences are saved per game instance. Open the menu with Right Shift.

## Security and known limitations

A CurseForge API credential compiled into a desktop executable can be extracted by anyone who can inspect or run that executable. Obfuscation does not make a client-side secret safe. A production deployment needs a trusted server-side proxy to keep the credential private; this cannot be solved securely while the key remains embedded in the client.

Download progress is byte-based for individual files. Minecraft asset/library preparation still uses the launcher library's opaque preparation API, so that stage cannot report detailed byte progress. Cancellation covers modpack downloads and archive extraction, not the underlying Minecraft runtime preparation. Windows runtime behaviour still needs validation on a real Windows machine.

## Development

```bash
npm ci
npm run dev
```

Build the Windows desktop application after installing the Rust toolchain and Tauri prerequisites:

```bash
npm run tauri dev
npm run tauri build
```

The Windows GitHub Actions workflow produces `NOVA-Client-Setup.exe`. It fetches the exact commit that triggered the workflow and runs Rust unit tests before building the installer.

The Fabric mod is built separately by the NOVA In-Game Mod workflow.

## Safety principles

- Never collect or store Microsoft passwords.
- Validate downloaded archive paths, sizes and hashes before treating a modpack as installed.
- Preserve separate instance folders and prevent deletion while a game process is running.
- Clearly label prototype features instead of presenting visual-only controls as functional modules.
