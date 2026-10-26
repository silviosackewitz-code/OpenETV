# Native appearance

The native desktop window reuses the independent RaceDash port of RaceAnalysis appearance. Copies live in `rust/src/look/` and `rust/assets/look/`; [provenance](../rust/assets/look/PROVENANCE.md) records their origin and adaptations. There is no runtime link to either project and no browser stylesheet dependency.

The four concrete themes are Dark, Light, Industrial and Terminal. System following selects Light or Dark. The copied implementation retains palette roles, neutral selection, text contrast, control radii and spacing, disabled opacity and the original accent derivation. Industrial and Terminal retain their own signal colours. Font families are IBM Plex and JetBrains Mono with the shared weights and icon/symbol fallbacks. Interface scale changes the whole window, not just text sizes.

Use semantic palette roles for controls. Cyan and amber identify labelled chart series on the dark scope; table heatmaps encode data, so they are separate from the appearance accent. Never rely on colour alone to explain warnings or the meaning of a series.

Changes to shared appearance belong upstream first and are copied here with provenance and licences. Existing focused tests retain palette/CSS comparisons, contrast, system selection, font rendering and icon glyph checks. Native window inspection remains necessary for layout and interaction.

Preferences live in `~/.openetv/prefs.json`; `OPENETV_HOME` selects a separate directory for verification. Save new appearance fields without dropping unrelated legacy keys.

## Legacy interface

The files under `tools/etvlib/` retain the previous browser appearance and its old sync mechanism for historical reference. They are not loaded by the Rust application. The archived interface manual remains in `docs/manual/`; the native manual is `rust/MANUAL.md` and is embedded in the executable.
