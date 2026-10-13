# RoR2 Unlocker

A local Risk of Rain 2 XML profile editor built with Rust, Tauri 2, and TypeScript. Select individual unlocks or stage missing entries in bulk, review the exact changes, then save with a backup.

The bundled catalog covers Hallowed Concepts and earlier expansions, using installed Steam build **25475991**. The application does not need access to the game installation at runtime.

## Supported edits

| Section | Catalog choices | Behavior |
| --- | ---: | --- |
| Survivors | 14 | Grant the character unlock and its achievement. Default-available survivors have no unlock entry. |
| Skills | 65 | Grant alternate skill unlocks and their achievements. |
| Achievements | 205 | Complete profile achievements and grant their associated rewards. |
| Skins | 62 | Grant skin unlocks and their achievements. |
| Artifacts | 20 | Grant artifact unlocks and their achievements. |
| Items | 282 | Mark item definitions as found; grant associated gameplay requirements where applicable. |
| Equipment | 51 | Mark equipment definitions as found; grant associated gameplay requirements where applicable. |

Lunar coins can be set to an integer from `0` through `2147483647`. The **Max** control uses the upper bound. Gaining more coins at that balance can overflow game-side calculations; a smaller balance is safer.

The catalog has 699 choices. Achievement choices and reward choices can overlap; the editor deduplicates the resulting field additions. Some installed item definitions are hidden or internal and do not produce visible logbook entries.

## Use the application

1. Close Risk of Rain 2. Let Steam Cloud synchronization finish before editing.
2. Choose **Open profile XML**, or enter the full path and choose **Open path**.
3. Choose a section. Search by name, identifier, or description; filter by expansion or missing entries.
4. Select individual entries, choose **Stage visible missing**, or choose **Stage all missing** across every section.
5. Optionally enter a lunar coin balance.
6. Choose **Preview changes**. Expand the field summaries to inspect the exact additions.
7. Choose **Save with backup**.

Selections only add progress. Choosing a reward also completes its supporting achievement. Item and equipment choices can additionally grant their gameplay unlock requirement. Existing and unknown identifiers are retained.

The editor changes only `achievementsList`, `discoveredPickups`, `coins`, and `<unlock>` records inside `stats`. It preserves other XML bytes, including career statistics, settings, comments, and escaped controller mappings. It accepts UTF-8 `UserProfile` XML up to 32 MiB and rejects ambiguous or malformed important fields, external entities, and unsupported encodings.

Each changed save creates a unique sibling backup named `<profile>.backup-<timestamp>-<process>-<counter>.xml`. The backup contains the exact previous file. The replacement is written and synchronized in the same directory, then committed with atomic replacement while preserving file permissions. The editor checks the file revision again immediately before replacement. A changed file must be reopened; a no-op save neither writes the profile nor creates a backup.

External game or Steam writes cannot be locked by this application. Keep the game closed and resolve any Steam Cloud conflict carefully. To restore a backup, close the game and editor, copy the backup over the intended profile, then reopen it.

This edits local profile progress only. It does not grant DLC ownership, edit run statistics, or guarantee Steam-account achievement changes.

## Development

Install current stable Rust, Node.js 24 or newer, pnpm **11.28.5**, and the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

- **Linux:** GTK 3 and WebKitGTK 4.1 development libraries are required. Debian/Ubuntu build dependencies are listed in `.github/workflows/build.yml`; Arch needs `webkit2gtk-4.1` and a development toolchain. Bundling also needs `patchelf`.
- **Windows:** use the Rust MSVC toolchain, Microsoft C++ Build Tools with **Desktop development with C++**, and Microsoft Edge WebView2.

From the project directory:

```sh
pnpm install --frozen-lockfile
pnpm desktop:dev
```

The development frontend uses `http://127.0.0.1:1427`. `pnpm dev` alone starts the browser frontend, which deliberately refuses profile operations without the Tauri runtime. It is not a simulated desktop editor.

The development CSP permits the local Vite WebSocket. Production keeps its stricter IPC-only connection policy.

### Checks

```sh
pnpm check
cargo fmt --all -- --check
cargo test -p ror2-profile-core --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Use `pnpm format` and `cargo fmt --all` to format changed frontend/configuration and Rust files respectively.

### Native packages

```sh
pnpm desktop:build
```

Build on the target operating system:

- Linux: `.deb` and `.AppImage` packages in `target/release/bundle/deb/` and `target/release/bundle/appimage/`.
- Windows: NSIS `.exe` and WiX `.msi` installers in `target/release/bundle/nsis/` and `target/release/bundle/msi/`.

Linux packages are built against the host's libraries. Use the Ubuntu 22.04 CI job for that compatibility baseline rather than assuming an Arch-built package runs on older distributions. If FUSE is unavailable, an AppImage can use `APPIMAGE_EXTRACT_AND_RUN=1`.

The GitHub Actions workflow builds and checks both platforms, then uploads their installers as artifacts. It does not publish releases or require signing secrets. Windows installers are unsigned unless signing is configured separately.

## Project layout

- `crates/profile-core/`: catalog loading, profile parsing, selective change planning, byte-preserving rendering, and filesystem persistence. Independent of Tauri.
- `src-tauri/`: native command handlers, file-dialog plugin, capabilities, and platform packaging configuration.
- `src/`: the TypeScript manifest interface and styles.
- `catalog.json`: bundled achievement/reward/discovery definitions, labels, and provenance.
- `.github/workflows/build.yml`: native Linux and Windows builds.

The catalog is a snapshot, not an automatic game scanner. New game releases or mod-specific unlocks need catalog updates. Preserve existing identifiers when adding entries; link item/equipment discoveries to achievement and reward requirements, including grouped elemental-ring and lunar-skill unlocks. Do not derive save identifiers from display names.

## Verification

Initial local verification:

- 14 Rust behavior tests passed, including empty/self-closing XML fields, dependency grants, unknown-data preservation, coin boundaries, stale saves, no-op saves, exact backups, permissions, and invalid inputs.
- TypeScript/Biome checks and workspace Clippy passed. No language servers were configured; compiler and linter diagnostics were used.
- Linux `.deb` and `.AppImage` packages built. The AppImage was launched and exercised through profile loading, selective Acolyte staging, maximum coins, exact preview, and save with backup.
- The documented `pnpm desktop:dev` command launched the native interface and loaded the real bundled catalog.
- The supplied `baseline.xml` was verified on isolated copies. It already contains all 699 catalog choices. All-unlock saving was a byte-preserving no-op. A controlled copy with missing entries exercised all seven sections, coin changes, idempotent saves, stale-write rejection, and preservation of all bytes outside intended edits.
- The supplied `baseline.xml` was not modified and is excluded from Git with other profile XML files.

Windows builds and runtime behavior, older Linux distributions, and in-game loading were not verified locally. The native Windows CI job is configured but has not been executed in this local project.
