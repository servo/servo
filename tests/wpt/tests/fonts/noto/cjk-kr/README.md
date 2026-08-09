# Fonts

A subset of [Noto Sans CJK KR] used to test the `text-overflow` property with
Korean ellipsis strings (`css/css-ui/text-overflow-string-*`).

The system Korean font, and therefore the advance width of a Hangul ellipsis,
differs between platforms, and no CSS unit measures Hangul (unlike `ch` for
Latin or `ic` for full-width CJK). Subsetting a known font with fixed metrics
makes the test's layout deterministic while still rendering real glyphs.

## NotoSansCJKkr-Regular-subset.otf

Covers only the Hangul syllables used by the tests: 안 (U+C548), 녕 (U+B155) and
낮 (U+B0AE). The CJK ideographs, kana and full-width Latin that also appear in
the ellipsis strings are full-width, so the tests size them with the `ic` unit
rather than this font.

Please see `subset.sh` to generate. The source font is Noto Sans CJK KR 2.004,
licensed under the SIL Open Font License, Version 1.1 (see `LICENSE.txt`).

[Noto Sans CJK KR]: https://github.com/notofonts/noto-cjk
