#!/bin/bash
#
# Produces a subset of Noto Sans CJK KR containing only the code points used by
# the `css/css-ui/text-overflow-string-*` tests, so that the ellipsis has
# predictable advance widths.
#
# Get the input font from the noto-cjk release:
# https://github.com/notofonts/noto-cjk/releases/tag/Sans2.004
#
# Requires `pyftsubset` from [fonttools]. If [uv] is installed, this script can
# run without installing [fonttools] globally.
#
# Usage: ./subset.sh /path/to/NotoSansCJKkr-Regular.otf
#
# [fonttools]: https://github.com/fonttools/fonttools
# [uv]: https://docs.astral.sh/uv/

# Only the Hangul syllables used by the tests: 안 녕 낮. Other scripts in the
# ellipsis strings (CJK ideographs, kana, full-width Latin) are full-width, so
# the tests size them with the `ic` unit instead of this font.
range="--unicodes=C548,B155,B0AE"
ext=".otf"
if [[ -z "$subsetter" ]]; then
  if command -v uvx &>/dev/null; then
    subsetter="uvx --from fonttools pyftsubset"
  else
    subsetter="pyftsubset"
  fi
fi
subsetter=${subsetter:-pyftsubset}
for path in "$@"; do
  (set -x;
    $subsetter "$path" $range --layout-features='*' \
      --output-file="NotoSansCJKkr-Regular-subset$ext"
  )
done
