#!/bin/bash
#
# Produces a subset of Noto Color Emoji containing only the code points used by
# the `css/css-ui/text-overflow-string-003.html` test, so that the emoji
# ellipsis has predictable advance widths.
#
# Requires `pyftsubset` from [fonttools]. If [uv] is installed, this script can
# run without installing [fonttools] globally.
#
# Usage: ./subset.sh /path/to/NotoColorEmoji.ttf
#
# [fonttools]: https://github.com/fonttools/fonttools
# [uv]: https://docs.astral.sh/uv/

# 🟢 U+1F7E2, 😀 U+1F600, 🤷 U+1F937, ♂ U+2642, ZWJ U+200D, VS16 U+FE0F.
# Keeping all layout features preserves the ligature that renders the ZWJ
# sequence "🤷‍♂️" as a single glyph.
range="--unicodes=1F7E2,1F600,1F937,2642,200D,FE0F"
ext=".ttf"
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
      --output-file="NotoColorEmoji-subset$ext"
  )
done
