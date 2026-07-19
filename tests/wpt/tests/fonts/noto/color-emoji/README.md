# Fonts

A subset of [Noto Color Emoji] used to test the `text-overflow` property with an
emoji ellipsis string (`css/css-ui/text-overflow-string-003.html`).

The system emoji font, and therefore the advance width of an emoji ellipsis,
differs between platforms. Subsetting a known font with fixed metrics makes the
test's layout deterministic while still rendering real color emoji.

## NotoColorEmoji-subset.ttf

Covers only the code points used by the test: 🟢 (U+1F7E2), 😀 (U+1F600),
🤷 (U+1F937), ♂ (U+2642), ZWJ (U+200D) and VS16 (U+FE0F). Every emoji glyph
advances 1.245em, and the ZWJ sequence "🤷‍♂️" (U+1F937 U+200D U+2642 U+FE0F)
is kept as a single glyph, so the string "🟢😀🤷‍♂️" is three glyphs wide.

Please see `subset.sh` to generate. The source font is Noto Color Emoji 2.047,
licensed under the SIL Open Font License, Version 1.1 (see `LICENSE.txt`).

[Noto Color Emoji]: https://github.com/googlefonts/noto-emoji
