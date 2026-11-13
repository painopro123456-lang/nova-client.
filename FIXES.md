# NOVA Client — 42-point repair checklist

Only check an item after its code change is committed and the relevant build/test has passed. Windows-only behaviour still requires a real Windows runtime test.

## Installation and launch
- [ ] 01. Prevent duplicate launches of the same instance.
- [ ] 02. Track game process exit and surface exit status.
- [ ] 03. Replace fixed 4 GB RAM with configurable allocation.
- [x] 04. Keep install progress and result available when navigating between pages.
- [ ] 05. Clean up interrupted/failed installs without deleting a prior working install.
- [ ] 06. Recover installed instances by scanning their on-disk metadata at startup.
- [ ] 07. Do not lose installed instances when localStorage is cleared.
- [ ] 08. Prevent concurrent writes/installs to the same instance.
- [ ] 09. Prevent deleting an instance while its game process is running.
- [ ] 10. Implement actual standalone mod installation, not just browsing.

## Downloads and modpacks
- [ ] 11. Report byte-based progress for large downloads.
- [ ] 12. Add cancellation and bounded network timeouts.
- [ ] 13. Avoid loading unbounded archives entirely into memory.
- [ ] 14. Enforce archive entry, path and expanded-size limits.
- [ ] 15. Verify Modrinth archive and file hashes.
- [ ] 16. Verify CurseForge archive and file hashes/sizes.
- [ ] 17. Retry alternate Modrinth download mirrors.
- [ ] 18. Use atomic writes and clean partial files after interruption.
- [ ] 19. Check available disk space before large installs.

## Catalog and interface
- [x] 20. Prevent stale search results from remaining clickable during provider/search changes.
- [x] 21. Disable pagination beyond the final page.
- [x] 22. Validate localStorage data shapes before rendering.
- [x] 23. Show the signed-in account in the profile UI.
- [x] 24. Provide a working sign-out control.
- [ ] 25. Replace the settings placeholder with functional settings.
- [ ] 26. Replace or clearly label the PvP placeholder.
- [x] 27. Keep progress event handling alive while the catalog screen is unmounted.

## Microsoft authentication and security
- [ ] 28. Bound OAuth callback socket reads with a timeout.
- [ ] 29. Show callback success only after OAuth state/token validation succeeds.
- [ ] 30. Reject unrelated local callback connections without aborting sign-in.
- [ ] 31. Remove client-embedded CurseForge secret exposure (requires a trusted proxy service).
- [ ] 32. Serialize token refreshes to avoid concurrent refresh conflicts.

## NOVA in-game Fabric mod
- [ ] 33. Implement advertised modules instead of visual-only toggles.
- [ ] 34. Make category buttons filter the module list.
- [ ] 35. Add per-module settings.
- [ ] 36. Persist module enabled states.
- [ ] 37. Verify Right Shift menu open/close behaviour in Minecraft 26.2.
- [ ] 38. Add the keybind translation entry.

## Maintenance and validation
- [x] 39. Update README to match the implemented product.
- [ ] 40. Commit dependency lockfiles and use deterministic installs.
- [ ] 41. Add automated tests for critical installation/authentication flows.
- [x] 42. Ensure async event listeners clean up correctly under React StrictMode.

