# Native look provenance

Theme, accent derivation, icons and palette reference copied from RaceDash rust/racedash-ui/src/look and rust/ui-assets. RaceDash independently ported these from RaceAnalysis; its source provenance is retained in RaceDash-provenance.md. Font definitions and JetBrains Mono assets copied from the current RaceAnalysis rust/racelib-ui/src/look/fonts.rs and rust/ui-assets/fonts.

OpenETV adaptations: local include paths and a string-based compatibility entry point. Four palettes, font weight/fallback chains, spacing, radii, neutral selections, accent contrast derivation and existing focused tests are retained. No product backend is imported.

IBM Plex and JetBrains Mono OFL files, Lucide licence, and existing symbol/math fallback notices accompany the bundled assets.

Cascadia is excluded from OpenETV by maintainer choice. The upstream provenance file is historical, not an inventory of the shipped fonts.

The upstream CSS reference was reduced to `rust/tests/fixtures/theme-roles.json` for native palette checks. No CSS is bundled.
