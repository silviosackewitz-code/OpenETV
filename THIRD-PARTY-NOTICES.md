# Native application assets

The Rust app embeds the Race Analysis / RaceDash native look. Font and icon files
are copied with their licences and provenance in `rust/assets/look/`:

- IBM Plex Sans and Mono: SIL OFL, `fonts/OFL-IBM-Plex.txt`.
- JetBrains Mono: SIL OFL, `fonts/OFL-JetBrains-Mono.txt`.
- Lucide font: ISC/MIT, `icons/LICENSE-lucide.txt`.
- DejaVu Sans Mono and New Computer Modern Math are symbol fallbacks;
  `fonts/NOTICE-PDF-fallbacks.txt` carries their notices.

No Cascadia font is bundled. The following notices describe the frozen Python/browser
reference implementation.

# Third-party notices

OpenETV itself is under the MIT licence ([LICENSE](LICENSE)). It ships the
following work of others.

## IBM Plex

SIL Open Font License, Version 1.1, copyright 2017, 2019 IBM Corp. The typeface
of the window (`tools/etvlib/fonts/`, served at `/fonts.css` and
`/fonts/*.woff2`): Regular, Medium, SemiBold and Bold of IBM Plex Sans;
Regular, Medium and SemiBold of IBM Plex Mono; Latin subset, as woff2 from the
`@fontsource/ibm-plex-sans` and `@fontsource/ibm-plex-mono` packages (5.3.0),
which repackage IBM's own release (`github.com/IBM/plex`) without changing the
font data. The full licence text ships with the files:
[`tools/etvlib/fonts/OFL.txt`](tools/etvlib/fonts/OFL.txt).

## Lucide icons

ISC License, copyright the Lucide Contributors; portions copyright Cole Bemis
as part of Feather (MIT License). A few icons from `lucide-static` 0.441.0,
kept as their SVG paths in `tools/etvlib/icons.js`, unchanged — nothing is
loaded from the network.

    ISC License

    Copyright (c) for portions of Lucide are held by Cole Bemis 2013-2022 as part of Feather (MIT). All other copyright (c) for Lucide are held by Lucide Contributors 2022.

    Permission to use, copy, modify, and/or distribute this software for any
    purpose with or without fee is hereby granted, provided that the above
    copyright notice and this permission notice appear in all copies.

    THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
    WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
    MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
    ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
    WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
    ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
    OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

## The look of Race Analysis

`theme.css`, `fonts.css`, `theme.py`, `theme.js`, `prefs.js`, `common.js` and
`icons.js` in `tools/etvlib/` are files of Race Analysis, by the same
maintainer, copied unchanged (`tools/sync_look.py`; `tools/etvlib/look.json`
names the commit they come from).
