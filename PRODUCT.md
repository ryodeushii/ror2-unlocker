# Product

<!-- impeccable:product-schema 1 -->

## Platform

Web and native desktop (Linux and Windows), sharing a browser-rendered TypeScript interface. Desktop uses Tauri 2 and the OS WebView; web uses Rust/WebAssembly. Impeccable's routing schema has no dedicated native-desktop value; browser verification does not establish native runtime correctness.

## Users

Risk of Rain 2 players editing their own XML profiles, with the game closed and Steam Cloud synchronization finished.

## Product Purpose

Inspect existing progress, stage missing unlocks and optional lunar coin changes, review exact additions, and produce a modified profile with an original backup.

## Operating Context

Desktop opens profiles through the native file picker and saves atomically with a sibling backup and stale-file checks. Web opens a local XML file into memory and offers separate modified-profile and original-backup downloads; users replace the game profile manually. Both editions keep the selected profile's path visible for identification.

## Capabilities and Constraints

Seven catalog sections: survivors, skills, achievements, skins, artifacts, items, and equipment. Search, expansion filters, missing-only filtering, individual selection, visible/all missing selection, preview, and coin editing must remain available. The catalog contains 699 choices. Edits only add progress; DLC ownership, Steam achievements, and run statistics are outside scope. Coin values range from 0 to 2147483647. XML is limited to 32 MiB. Web does not upload or persist profile XML and cannot replace source files or monitor game writes. Desktop preserves original bytes outside edited fields and rejects stale saves.

## Evidence on Hand

README.md documents supported behavior and platform workflows. catalog.json supplies real catalog content. baseline.xml is a local populated verification profile and must remain unchanged. assets/icon.svg and src-tauri/icons contain the existing application icon. No photographic assets or approved marketing claims are supplied.

## Product Principles

- Make staged changes distinguishable from loaded progress.
- Review edits before any save or download preparation.
- Keep backups and platform-specific safety limits explicit.
- Preserve all supported edits and local-only processing.
