# NOVA Client

A Windows-first Minecraft Java client and launcher focused on a fast native shell, polished interface, PvP tools, and straightforward instance management.

## Current status

This repository starts the NOVA desktop application foundation. The current milestone is the launcher UI and project structure; Minecraft authentication, asset/version installation, process launching, mod services, and update delivery are not implemented yet.

## Stack

- React + Vite for the interface
- Tauri 2 + Rust for the native Windows desktop shell
- TypeScript-ready component structure (JavaScript in the first UI milestone)

## Development

```bash
npm install
npm run dev
```

To build the desktop app after installing the Rust toolchain and Tauri prerequisites:

```bash
npm run tauri dev
npm run tauri build
```

## Design principles

- Windows 10/11 and Minecraft Java Edition first
- Keep the native shell lightweight; avoid bundling a full browser runtime
- Make every navigation destination usable and clearly distinguish implemented features from planned integrations
- Never collect or store Microsoft passwords; authentication will use Microsoft's supported OAuth flow
