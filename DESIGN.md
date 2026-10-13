---
name: RoR2 Unlocker — Obsidian & ember
description: A charcoal profile-editing workspace with warm ivory text and ember-orange pending edits.
colors:
  ember: "#f6a66c"
  ember-ink: "#26190f"
  ground: "#171716"
  rail: "#111110"
  surface: "#1d1d1b"
  raised: "#282825"
  text: "#f1eee7"
  muted: "#b6b1a6"
  line: "#3a3934"
  complete: "#a8c7a1"
  error: "#ffb4a4"
typography:
  headline:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "26px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.025em"
  section:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "20px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.02em"
  entry:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "14px"
    fontWeight: 600
    lineHeight: 1.4
  body:
    fontFamily: "IBM Plex Sans, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.6
rounded:
  control: "6px"
  profile: "10px"
  dialog: "12px"
spacing:
  compact: "8px"
  group: "16px"
  section: "20px"
  gutter: "28px"
components:
  button-primary:
    backgroundColor: "{colors.ember}"
    textColor: "{colors.ember-ink}"
    rounded: "{rounded.control}"
    padding: "9px 12px"
  input:
    backgroundColor: "{colors.ground}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "9px 12px"
---

# Design System: RoR2 Unlocker

## Overview

**Creative North Star: Obsidian & ember.** User-approved replacement of the incumbent navy/cyan terminal interface for both desktop and web. Surface mode: Operate.

A charcoal workbench with warm ivory text and ember-orange pending edits. Game-adjacent through restrained warmth and dense catalog work, not a fictional HUD. Navigation and controls remain familiar desktop/web controls. Product capabilities and Rust backend contracts are unchanged.

Key characteristics: a full-height category rail, dominant catalog work area, separate changes pane, and explicit Stage → Review → Save/Download sequence.

## Colors

Ground, rail, surface, and raised tones separate navigation, working areas, and controls without decorative effects. Ember identifies staged rows, the current workflow step, primary actions, and selected navigation. Already-complete progress uses muted green; errors use warm red. Missing entries remain neutral. Every progress state has a text label.

## Typography

Self-hosted IBM Plex Sans for headings, navigation, body, and controls. The page title uses 26px/600, section headings 20px/600, row titles 14px/600, and body text 13px. The mobile page title is 23px. System monospace is reserved for exact XML identifiers, paths, and field additions. Counts and coin balances use tabular numerals. No decorative uppercase heading kickers.

## Layout

At wide sizes, a 216px category rail sits beside the workspace. The workspace divides into a flexible catalog and 322px changes pane; the pane grows to 360px above 1600px. Gutters are 28px, with tighter groups inside controls. Catalog and changes pane scroll independently.

The header shows **Profile editor** without an edition badge. Platform differences are communicated by the actual save/download actions and relevant safety instructions, not a decorative runtime label.

Below 1200px, the rail becomes 182px and the changes pane 278px. Between 761px and 959px, the rail is 162px, the changes pane 258px, filters wrap, and row state labels move below the entry information. At desktop heights of 740px or less, the main area scrolls and the workspace retains a 560px minimum height so the catalog remains usable.

At 760px and below, categories become horizontally scrollable buttons and the changes pane follows the catalog. Catalog scrolling is bounded to 65svh. At 380px and below, final actions stack. Profile paths and XML identifiers wrap rather than widening the page. OS-native desktop chrome and file pickers remain intact.

## Elevation & Depth

Flat surfaces use tonal separation and single-pixel dividers. The discard-confirmation dialog alone uses `0 18px 60px #0008` to establish protected focus. No ornamental gradients, glass, texture, or animation.

## Shapes

Controls use 6px corners, the profile opener 10px, and the dialog 12px. State labels use 4px corners. Small circular workflow markers represent actual ordered steps, not decoration.

## Components

### Navigation

Category buttons show completed/total counts, with pressed state exposed to assistive technology. The selected category uses an ember-tinted fill and border. Mobile buttons scroll horizontally without widening the document.

### Catalog rows

Checkbox-led rows show the unlock name, expansion, exact identifier, description, and labeled state. Complete entries cannot be selected again. Staged entries receive an ember tint; existing complete entries receive green state labels. Search and expansion filters sit immediately above their catalog.

### Changes pane

Staged unlocks can be removed individually or cleared. Lunar coin input shows current balance, optional replacement, validation, and maximum-value safety guidance. Exact preview details expand to reveal field additions. Stage, Review, and Save/Download markers follow actual draft, preview, backup, and prepared-download state.

### Actions and safety

**Profile opener:** one **Open profile XML** action. Desktop uses the native file picker; web uses browser file selection. There is no manual-path form. The loaded profile's path stays visible for identification, while opener controls take only the width of the button.

Primary actions use ember with dark ink. Hover, busy, disabled, invalid, and keyboard-focus states remain visible; focus uses a 2px ember outline with 3px offset. Selection uses warm dark brown with light text, caret uses ember, and scrollbars use muted warm tones. No authored motion means reduced-motion users receive the same stable interface.

The browser's local-processing disclosure explains manual replacement. Required safety guidance remains beside final actions. Desktop saves create sibling backups; browser downloads remain explicit links and do not mark the source as saved. Opening another profile with a draft uses a confirmation dialog.

## Do's and Don'ts

- Do distinguish loaded progress from pending edits using labels and color.
- Do keep review before save/download preparation and retain backup guidance.
- Do preserve local-only processing, all seven sections, and the platform-specific save workflow.
- Don't add decorative HUD controls, heading kickers, or glyph-based icons.
- Don't let paths, identifiers, or short windows make the catalog unusable.

## Verification

Web captures inspected at 1440×900, 1024×768, 860×620, 390×844, and 320×740. Browser smoke exercised file opening, staging, coin validation, previews, prepared downloads, exact backup bytes, filters, discard protection, and malformed-profile recovery. The compact-height catalog issue found in the first capture batch was corrected.

Initial redesign verification, before removal of the manual-path opener, ran the Linux desktop in an isolated KWin virtual display. WebKit's local inspector exercised open-path, stage, preview, and save-with-backup; on-disk output matched browser rendering and the backup matched original bytes. Native rendering was captured at 1320×900 and 860×620. The picker-only update verified that the actual native file dialog opens and that the browser loads a selected profile without a manual-path form. Automated native file selection was not verified: isolated-display input injection was ignored, and debugger attachment was denied. Windows GUI and in-game behavior are not locally verified. Finish review is inline, not independent.
