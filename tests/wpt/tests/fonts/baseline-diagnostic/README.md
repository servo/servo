# Baseline Diagnostic Font

## Overview

Font that can be used for validating baseline alignments. Given the embedded
text in the font, this should be used with very large font sizes.

## Baselines and Metrics

| Baseline/Metric | Coordinate | BASE Value | OS/2 Value | hhea Value |
|-----------------|------------|------------|------------|------------|
| ascent | 800 |  | sTypoAscender | ascent |
| ideographic-over | 750 | idtp |  |  |
| hanging | 650 | hang |  |  |
| ideographic-face-over | 650 | icft |  |  |
| cap-height | 550 |  | sCapHeight |  |
| math | 450 | math |  |  |
| /central/ | 350 |  |  |  |
| /em-middle/ | 300 |  |  |  |
| x-height | 250 |  | sxHeight |  |
| /x-middle/ | 150 |  |  |  |
| alphabetic | 50 | romn |  |  |
| ideographic-face-under | 50 | icfb |  |  |
| /zero/ | 0 |  |  |  |
| ideographic-under | -50 | ideo |  |  |
| descent | -200 |  | sTypoDescender | descent |

The `BaselineDiagnosticAlphabeticZero` variant is the same as `BaselineDiagnostic`, except `alphabetic` moves to 0; `x-middle` moves to 125.

## Glyphs

### Diagnostic glyph

| Glyph | Codepoint | Description |
|-------|-----------|-------------|
| `X`   | U+0058    | All baselines drawn with labels |

### Pair glyphs

Each baseline pair has two variants: a **layout** glyph (opaque filled rectangle
between the two baselines) and a **labeled** glyph (lines with text labels, like `X`).

| Pair | Layout | Layout codepoint | Labeled | Labeled codepoint |
|------|--------|------------------|---------|-------------------|
| x-height + alphabetic | `x` | U+0078 | `χ` | U+03C7 |
| cap-height + alphabetic | `B` | U+0042 | `β` | U+03B2 |
| ideographic-over + ideographic-under | `I` (alt: `口`) | U+0049 (U+53E3) | `i` (alt: `日`) | U+0069 (U+65E5) |
| ideographic-face-over + ideographic-face-under | `F` (alt: `中`) | U+0046 (U+4E2D) | `f` (alt: `田`) | U+0066 (U+7530) |
| hanging + alphabetic | `H` (alt: `अ`) | U+0048 (U+0905) | `h` (alt: `आ`) | U+0068 (U+0906) |
| math + alphabetic | `+` | U+002B | `±` | U+00B1 |

### Em-box glyphs

| Variant | Glyph | Codepoint |
|---------|-------|-----------|
| Filled  | `█`  | U+2588  |
| Outline | `□` | U+25A1 |

### Fallback glyph

All other characters in the Latin-1 set (U+0021 to U+00FF) not specified above
render the same as `█` (U+2588), the filled em-box glyph.

## Source and Downloads

Both the source code and built font files can be found in the [`@sajidanwar.com/baseline-diagnostic-font`][tangled-repo]
repository on [Tangled][tangled-home] or the [`kbhomes/baseline-diagnostic-font`][github-repo]
repository on [GitHub][github-home].

This font is built using Python with the [fonttools](https://fonttools.readthedocs.io/en/latest/) library.

[tangled-repo]: https://tangled.org/sajidanwar.com/baseline-diagnostic-font
[tangled-home]: https://tangled.org/
[github-repo]: https://github.com/kbhomes/baseline-diagnostic-font
[github-home]: https://github.com/

## License

This font contains [Noto Sans Mono][noto-sans-mono] glyphs in the rendering
of its baseline labels. Like that font, this font is licensed under the
[SIL Open Font License, Version 1.1][ofl-1.1], and is available at `LICENSE.txt`.

[noto-sans-mono]: https://fonts.google.com/noto/specimen/Noto+Sans+Mono/license
[ofl-1.1]: https://openfontlicense.org/open-font-license-official-text/