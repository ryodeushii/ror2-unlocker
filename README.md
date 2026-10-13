# RoR2 Unlocker

A local Risk of Rain 2 XML profile editor with a Rust editing engine, a Tauri 2 desktop application, and a Rust/WebAssembly browser edition.

**Web editor:** <https://ryodeushii.github.io/ror2-unlocker/>

The catalog covers Hallowed Concepts and earlier expansions, using installed Steam build **25475991**. Neither edition needs a game installation to inspect or edit a selected profile.

## Supported edits

| Section | Catalog choices | Behavior |
| --- | ---: | --- |
| Survivors | 14 | Grant character unlocks and their achievements. Default-available survivors have no unlock entry. |
| Skills | 65 | Grant alternate skill unlocks and their achievements. |
| Achievements | 205 | Complete profile achievements and grant their rewards. |
| Skins | 62 | Grant skin unlocks and their achievements. |
| Artifacts | 20 | Grant artifact unlocks and their achievements. |
| Items | 282 | Mark items as found and grant associated gameplay requirements where applicable. |
| Equipment | 51 | Mark equipment as found and grant associated gameplay requirements where applicable. |

The catalog has 699 choices. Achievement and reward choices overlap; the engine deduplicates field additions. Some installed item definitions are hidden or internal and do not produce visible logbook entries.

Lunar coins accept whole numbers from `0` through `2147483647`. **Max** uses that upper bound. Further coin gains can overflow game-side calculations; a smaller balance is safer.

Both editions use the same Rust validation, revision, unlock-dependency, preview, and byte-preserving rendering implementation. They only add progress. DLC ownership, run statistics, and Steam-account achievements are not controlled by this editor.

## Interface

Both editions share the **Obsidian & ember** design: a charcoal category rail, searchable unlock catalog, and separate **Your changes** pane. Labeled states distinguish missing entries, staged edits, and existing completed progress. The **Stage → Review → Save/Download** indicator follows the current editing state.

At smaller web widths, categories scroll horizontally and the changes pane stacks below the catalog. Short desktop windows retain a usable catalog height and scroll the main workspace. Browser-local processing instructions are available in the disclosure below the profile opener; save/backup safety guidance stays beside the final actions.

`PRODUCT.md` records product constraints. `DESIGN.md` and `.impeccable/design.json` record the approved palette, typography, layout, and interaction rules.

## Browser edition

1. Close Risk of Rain 2 and let Steam Cloud synchronization finish.
2. Open the web editor and choose **Open profile XML**.
3. Select individual entries, stage visible missing entries, or stage all missing entries. Search and category/expansion filters narrow the manifest.
4. Optionally enter a lunar coin balance, then choose **Preview changes**.
5. Review the exact additions and choose **Prepare downloads**.
6. Choose **Download original backup**, then **Download modified profile**.
7. Keep the backup. Manually replace the intended game profile with the downloaded modified XML while the game is closed.

The browser reads the selected file into memory and edits it through WebAssembly. It does **not** upload XML, persist it in browser storage, or write to the source file. The original snapshot and staged draft remain visible after preparation; preparing downloads does not mark the source as saved. The modified file is named `<stem>.unlocked.xml`; its exact-byte backup is named `<original-name>.backup.xml`.

Changing staged entries or coins invalidates the preview and disposes old download URLs. Opening another file requires confirmation when a draft exists. Invalid files and canceled pickers preserve the loaded source. A no-op preparation creates no modified download or backup.

A normal browser cannot monitor game writes or atomically replace the original through this download workflow. Unlike the desktop edition, it cannot provide native on-disk stale-file protection. Keep the game closed and resolve any Steam Cloud conflict carefully.

## Desktop edition

1. Close the game and finish Steam Cloud synchronization.
2. Choose **Open profile XML** and select the profile in the native file picker.
3. Stage unlocks and optional coin changes.
4. Choose **Preview changes**, review the field additions, then choose **Save with backup**.

Each changed save creates a unique sibling `<profile>.backup-<timestamp>-<process>-<counter>.xml` containing the exact previous file. The replacement is written and synchronized in the same directory, then committed atomically while preserving standard file permissions. The engine checks the file revision immediately before replacement. Changed files must be reopened; no-op saves neither write nor create backups.

External processes cannot be transactionally locked by the editor. To restore a desktop backup, close the game and editor, copy the backup over the intended profile, then reopen it.

## Profile safety and compatibility

The engine accepts UTF-8, unnamespaced `UserProfile` XML up to 32 MiB. It rejects malformed or ambiguous important fields, unsupported encodings, DTD/external entities, invalid coin values, and unknown client selection IDs.

Edits are limited to `achievementsList`, `discoveredPickups`, `coins`, and `<unlock>` records inside `stats`. All other XML bytes remain unchanged, including career statistics, unknown identifiers, comments, line endings, and escaped controller mappings.

`catalog.json` is a versioned snapshot, not an automatic game scanner. New game releases or mod-specific unlocks need catalog updates. Preserve save identifiers and link item/equipment discoveries to their achievement and reward requirements, including grouped elemental-ring and lunar-skill unlocks. Do not derive identifiers from display names.

## Development prerequisites

Install current stable Rust, Node.js 24 or newer, pnpm **11.28.5**, and matching WebAssembly tooling:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
pnpm install --frozen-lockfile
```

The binding crate pins the same wasm-bindgen version as the CLI. `pnpm build:wasm` generates ignored files in `wasm/pkg/`; they are required for frontend type checking. Desktop bundles exclude the unused WASM/browser adapter, and web bundles exclude Tauri commands.

For desktop development, also install the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/):

- **Linux:** GTK 3, WebKitGTK 4.1 development libraries, and a development toolchain. Debian/Ubuntu dependencies are in `.github/workflows/build.yml`; Arch needs `webkit2gtk-4.1`. AppImage bundling also needs `patchelf`.
- **Windows:** Rust MSVC, Microsoft C++ Build Tools with **Desktop development with C++**, and Microsoft Edge WebView2.

### Run either edition

```sh
pnpm desktop:dev
pnpm web:dev
```

Desktop development uses `http://127.0.0.1:1427`. Browser development uses `http://127.0.0.1:1428/ror2-unlocker/`. `pnpm dev` is the desktop frontend; without Tauri it displays guidance rather than pretending to edit files. The browser edition requires Vite mode `web`.

The desktop development CSP permits the local Vite WebSocket; production retains its IPC-only connection policy. Web production uses a self-only CSP with WebAssembly compilation enabled and no remote fonts or scripts.

### Checks

```sh
pnpm check
cargo fmt --all -- --check
cargo test -p ror2-profile-core --locked
cargo test -p ror2-profile-core --no-default-features --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Use `pnpm format` and `cargo fmt --all` to format changed files. `.gitattributes` enforces LF checkouts across platforms, including Windows with `core.autocrlf=true`.

### Builds and deployment

```sh
pnpm desktop:build
pnpm web:build
```

- **Linux:** AppImage only, in `target/release/bundle/appimage/`.
- **Windows:** NSIS `.exe` and WiX `.msi` installers, in `target/release/bundle/nsis/` and `target/release/bundle/msi/`.
- **Web:** static assets in `dist-web/`, with the project-site base `/ror2-unlocker/`.

Build desktop installers on their target operating systems. Linux binaries depend on host library compatibility; use the Ubuntu 22.04 CI artifact for that baseline rather than assuming an Arch-built AppImage runs on older distributions. Use `APPIMAGE_EXTRACT_AND_RUN=1` if FUSE is unavailable. Windows installers are unsigned unless signing is configured separately.

The raw Linux executable at `target/release/ror2-unlocker` embeds the configured window icon; it is not exclusive to AppImage. The AppImage also contains a `.desktop` launcher and icon-theme assets. A bare ELF executable does not install those assets or acquire a custom file-manager icon. Linux menu and Wayland taskbar identity depend on desktop integration; carrying an AppImage alone does not guarantee that its launcher has been registered with the desktop.

`.github/workflows/build.yml` checks and packages both native platforms. `.github/workflows/pages.yml` checks the filesystem-free core and browser build, then deploys `dist-web/` to GitHub Pages on `main`. PRs run checks without deploying. GitHub Pages is configured to use GitHub Actions.

## Project layout

- `crates/profile-core/`: shared byte validation, change planning, and XML rendering; native filesystem operations behind the default `native` feature.
- `crates/profile-wasm/`: thin wasm-bindgen exports, using the core without filesystem features.
- `src-tauri/`: native command handlers, picker capabilities, and packaging.
- `src/`: shared manifest interface plus desktop and browser transport adapters.
- `scripts/build-wasm.mjs`: reproducible WASM build and binding generation.
- `catalog.json`: unlock/discovery identifiers, labels, and provenance.

## Verification

Local checks passed with 20 native-feature Rust behavior tests and 6 filesystem-free tests, TypeScript/Biome, workspace Clippy, and both frontend build modes. Browser production verification exercised the real WASM engine, file selection, exact previews, explicit downloads, no-op behavior, malformed-file recovery, picker cancellation, invalid coins, stale-download disposal, and all seven unlock sections.

The supplied `baseline.xml` already contains all 699 catalog choices. It was verified on isolated copies and remains unchanged. Controlled copies with missing entries verified that downloaded backups match the original byte-for-byte and modified files change only requested fields. Request logs contained only asset reads; no XML uploads occurred. Profiles, backups, generated WASM, dependencies, and build outputs are excluded from Git and deployment artifacts.

The redesign passed `pnpm check`, `pnpm build`, and `pnpm web:build`; the Impeccable HTML/CSS detector reported no findings. Final browser captures covered 1440×900, 1024×768, 860×620, 390×844, and 320×740. Real browser interactions exercised staging, coin validation, exact preview/download content, filters, discard confirmation, and malformed-profile recovery.

During the initial redesign, before removal of the manual-path opener, an isolated KWin virtual desktop ran the Linux application at 1320×900 and 860×620. Its local WebKit inspector exercised the actual frontend and Rust commands: open a disposable profile by path, stage Acrid, change coins from 7 to 250, preview, and save with backup. The saved XML matched browser output and the sibling backup matched original bytes. Native automation did not inject input into the active user desktop. TypeScript and Biome language-server diagnostics were checked; TypeScript reports only the existing deprecated `beforeunload` `returnValue` hint.

The picker-only update passed `pnpm check`, `pnpm web:build`, and `pnpm desktop:build`. The browser loaded `baseline.xml` through file selection and kept its filename visible; the actual Linux native file dialog opened from **Open profile XML**. Native selection automation was unavailable in the isolated display, so that path was not claimed as verified. The manual-path form and its unused frontend opening API were removed; the native picker still uses the existing Rust `open_profile` command.

The picker-only detector pass reported existing design-documentation drift for unchanged colors, font sizes, radii, and the monospace fallback. These findings were not repaired as part of removing the manual opener.

Native Windows and Linux CI previously passed after the LF checkout fix; Linux produces AppImage only. Windows GUI runtime and in-game loading remain unverified locally. The redesign has not been deployed; the published Pages site requires verification after deployment.
