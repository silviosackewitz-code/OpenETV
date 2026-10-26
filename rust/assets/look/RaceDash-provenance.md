# Native appearance provenance

The first native RaceDash milestone copies the existing RaceAnalysis appearance and control
implementation. The reference checkout was read only; no build, network fetch or runtime link to
RaceAnalysis was used.

Source repository: `Race_Analysis`, commit `803c004cff13a7649bbce7bee4e78d12c778281c`.
The source files below had no working-tree modifications when copied.

## Code copied and adapted

| RaceAnalysis source | RaceDash destination | Adaptation |
|---|---|---|
| `rust/racelib-ui/src/look/theme.rs` | `rust/racedash-ui/src/look/theme.rs`, `theme_tests.rs` | Same four palettes, typography, radii, spacing, 45% disabled alpha, system theme selection, 70–150% sizing, and 90% default. Omit Analyzer-only plot, map and formula colours/state. Extract tests to their own file. |
| `rust/racelib-session/src/theme.rs` | `rust/racedash-ui/src/look/accent.rs` | Copy only pure colour/contrast/accent derivation and its contrast test; use standard `BTreeMap` instead of `IndexMap` because callers only address roles by name. No session backend or subprocess dependency. |
| `rust/racelib-ui/src/look/fonts.rs` | `rust/racedash-ui/src/look/fonts.rs` | Same font definitions, families, weights and fallback order; add font-chain and headless control tests. |
| `rust/racelib-ui/src/look/icons.rs` | `rust/racedash-ui/src/look/icons.rs` | Direct copy, including accessible button names, hover hints, stable toggle width and existing tests. |
| `rust/racelib-ui/src/parts/kit.rs` | `rust/racedash-ui/src/controls.rs` | Copy icon-and-word buttons, dialog headings/footers/hints and labelled fields/pickers. Omit level gating and log-window notes. Inline the same glyph constants from `mainwin/glyphs.rs` and the `Close` label. |
| `tools/racelib/web/theme.css` | `rust/ui-assets/theme-reference.css` | Unchanged test reference for palette roles, font sizes and radii; no web frontend or runtime stylesheet dependency. |

`look/mod.rs` only exports the copied modules. RaceDash keeps its own copies as required by
`CLAUDE.md`; no symlink or dependency points into the Analyzer checkout.

## Source code fingerprints

SHA-256 hashes of the reference files, before adaptation:

| Source | SHA-256 |
|---|---|
| `rust/racelib-ui/src/look/theme.rs` | `25f461a9232f4142c79bae5a39e5f12f8f5f12c40f391112206a91190f3047f1` |
| `rust/racelib-ui/src/look/fonts.rs` | `184348e100d3bfb39f180ad60d2fd350bf3a3a63862ace80ab91d1651abc74eb` |
| `rust/racelib-ui/src/look/icons.rs` | `3f14d11cd2a6443213a5add7a29b35e0b64ce6b19521101455f251984d680d0b` |
| `rust/racelib-ui/src/parts/kit.rs` | `e1513bc1d5a1cd335a51f411a5ad006c461c0c10b360f6469eb562013d9408fd` |
| `rust/racelib-ui/src/mainwin/glyphs.rs` | `6e37a71b0dd3f2cdb0a7dceb341d73096561dc4b260ec00a63826619efe6b89f` |
| `rust/racelib-session/src/theme.rs` | `a71d3aa3fbe1bbb4a8e732feabe48802274a6866c5c638fa2674e555bd6d3cae` |
| `tools/racelib/web/theme.css` | `2696cfe7ffe3daa26fc67e14617593138769702eb82cf798d55edcb166f4e4bd` |

## Bundled assets and licences

All assets below are byte-identical copies from the reference commit. IBM Plex and Cascadia
licences accompany their fonts. Lucide uses the matching `0.441.0` font and generated codepoint
table, with its licence. The two existing symbol/math fallbacks retain their upstream notice
and provenance file. Unused `IBMPlexMono-SemiBold.ttf` is not copied.

| Source and destination relative to `rust/ui-assets/` | SHA-256 |
|---|---|
| `fonts/CascadiaMono-Bold.ttf` | `b22cb603ed23cac36e8444846e1841caca21719a5e852780a8d37ae7a49b0a36` |
| `fonts/CascadiaMono-Regular.ttf` | `06520d032ec274fa5040b22c6f4a1d829081b24ba40b2da56dae89bf10c7b481` |
| `fonts/CascadiaMono-SemiBold.ttf` | `8e04c1b811913a20773a3761d8994f15efe3029509c8d0556d74ae989558146d` |
| `fonts/DejaVuSansMono.ttf` | `b4a6c3e4faab8773f4ff761d56451646409f29abedd68f05d38c2df667d3c582` |
| `fonts/IBMPlexMono-Medium.ttf` | `98fbd727aae340b236955879dabed4d991aac9e8e90b3b2a67ce4a59221cc97c` |
| `fonts/IBMPlexMono-Regular.ttf` | `7c6fbddca4b700be918f5f6183d9bd4464fa427fe435f0b480d77fe2bb8c5a43` |
| `fonts/IBMPlexSans-Bold.ttf` | `9e6c74a889a700d707613d24548fe4ffa6bc59559a0689d2cf9e133bdcdafb2f` |
| `fonts/IBMPlexSans-Medium.ttf` | `331c8639d7598b2cde62a911a71db195e30cb655cd6bdf2e324a7e984955f907` |
| `fonts/IBMPlexSans-Regular.ttf` | `975dcda37d80f038dcd143c22e33ca2d97a0cc5a929aace1c749153b0fe1afa5` |
| `fonts/IBMPlexSans-SemiBold.ttf` | `a20caf8286023a6a7a85e40b1d2a4ae9fc3e3b1f9eda8f4c542dd4986af67bb1` |
| `fonts/NOTICE-PDF-fallbacks.txt` | `2fe0ab87916eb54b6f31e17523c3d661e65b21dc558392cd3cd1c28e46255b28` |
| `fonts/NewCMMath-Regular.otf` | `ad746f5307fed53d8e5d7d201f5054abe73f1f47c51040767d898e3481c6e648` |
| `fonts/OFL-Cascadia.txt` | `51882cd3cdba4e16f220f44ddb08a635c38c44ea6e0975db2574f4be6f958238` |
| `fonts/OFL-IBM-Plex.txt` | `7e6b2818edbd8f6a01ae80641cc8f16a51080d08fb4e532be3a0b6f74adb07da` |
| `fonts/PDF-fallbacks-provenance.json` | `ccd5476b2abd88d0005d257b4d29600fd93d112ce09ade799ff039f1724d4185` |
| `icons/LICENSE-lucide.txt` | `1e7290b35280a048667bbf0ebabac1c7fd52a75300e8b2946ac165715997f2bc` |
| `icons/lucide.rs` | `a8d53b7b7fbd88c75a7eb2d09f42d2a64d2b5d9cce62be3c566b3a31d9508860` |
| `icons/lucide.ttf` | `3400d6dd6f7086308a39ecc0b0c3fa4118305902ba657eb413fffd25503718ed` |

## Verification scope

Tests preserve the upstream light/dark CSS-role comparison, all four themes’ text contrast,
custom accent derivation, automatic system-theme selection, sizing steps, every Lucide glyph,
and stable toggle geometry. Added tests check every font family and tessellate real copied
controls headlessly for every theme/font combination. These tests produce geometry without a
native OS window; they are not screenshots or evidence of complete configurator parity.

The migration coordinator records actual test commands and results in the milestone record.
No device, installed application or reference checkout is modified by this package.

## First Appearance selector

`controls::appearance` also adapts `rust/racelib-ui/src/settings/appearance.rs` from the same
reference commit (SHA-256 `7f04972c778e523e8560191d02993ff9ef1c4903d8243f96661c01d5652808b1`).
It retains the grid, segmented choices, accent swatches, size slider and Terminal font switch.
The first selector edits an in-memory resolved Look: four concrete themes, no Auto preference
or persisted settings yet. The custom colour popup is egui's; plot settings are not copied.
These limits are explicit on the migration ledger and in the development README.

## Complete editor and native menu increment

The same read-only Analyzer commit supplied these additional interaction patterns:

| Source | Native adaptation | Source SHA-256 |
|---|---|---|
| `rust/racelib-ui/src/settings/mod.rs` | Tile/shift sections use the section chooser and field-body layout. | `39ce014ea3e2913f408d9b0e6a5c828f191e5f05f6cdcae9e8b80087e91efc63` |
| `rust/racelib-ui/src/settings/colour_field.rs` | `editor/fields.rs` uses its cross-platform palette/hex/picker interaction, Enter and Escape behavior. | `cbd4291ade9d70e4424d0c42a2ea30419523fbe14247ffa14aca28fab7d56733` |
| `rust/racelib-ui/src/workspace/tiles.rs` | `editor/canvas.rs` uses temporary drag geometry and focusable hit areas; actual grid rules come from RaceDash. | `83da6481914d017800bf0c157e205e2cfb1c3131328f24a6a6d6a7e2909b3c92` |
| `rust/racelib-ui/src/menubar.rs` | `menubar.rs` and `app/menus.rs` reuse the native macOS menu/event queue and guarded Dock termination; actions target RaceDash documents. | `49db9b64d070d841e673299e705be1e18cd8ffea02ae35ec51bcd85095e01b53` |

Appearance now persists the copied theme/font/accent/size settings, including system following.
The menu uses the Analyzer's exact cached `muda = 0.20.0` and `objc2 = 0.6.5` dependencies on
macOS. No dependency links to its checkout. The earlier session-only Appearance limitation
above describes the first increment and is superseded.

RaceDash's own JavaScript behavior remains the domain reference: `pages/view.js`, `grid.js`,
`tiles/view.js`, `alarms.js`, `banners/{logic,view}.js`, `shift/{logic,view}.js`, `issues.js`,
and the channel/track/device/demo modules. Native controls replace their DOM machinery;
the existing backend routes retain compilation, device and recorded-lap contracts.
