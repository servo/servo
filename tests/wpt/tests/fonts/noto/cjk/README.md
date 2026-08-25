# Fonts

Fonts in this directory help testing the `text-spacing-trim` property.

## NotoSansCJKjp-Regular-subset-halt.otf

Please see `subset.sh` to generate.

## NotoSansCJKjp-Regular-subset-chws.otf

This font has `chws` and `vchw` in addition to the font above.

Note there are two variants of Noto CJK; one with `chws` and one without.
The input font for this command must be the one with `chws`, processed by the
[chws_tool](https://github.com/googlefonts/chws_tool).

## NotoSansCJKjp-Regular-subset-halt-min.otf

This font is to test web fonts scenario where not all glyphs are available.
