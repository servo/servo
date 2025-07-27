#!/bin/bash
#
# Produces subset font files for testing the `text-spacing-trim` property.
#
# Requires `pyftsubset` from [fonttools]. If [uv] is installed, this script can
# run without installing [fonttools] globally.
#
# [fonttools]: https://github.com/fonttools/fonttools
# [uv]: https://docs.astral.sh/uv/
#
range="--unicodes=20-7E,2018-201F,56FD,6C34,3000-301F,30FB,FF01-FF1F,FF5B-FF65"
subrange="--unicodes=20-7E,56FD,FF08-FF09"
features="--layout-features+=halt,fwid,hwid,palt,pwid,vhal,vpal"
features_chws="$features,chws,vchw"
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
  filename="$(basename -- "$path")"
  stem="${filename%.*}"
  output_halt="$stem-subset-halt"
  output_chws="$stem-subset-chws"
  output_min="$stem-subset-halt-min"
  (set -x;
    $subsetter "$path" $range $features --output-file="$output_halt$ext"
    $subsetter "$path" $range $features_chws --output-file="$output_chws$ext"
    $subsetter "$output_halt$ext" $subrange $features --output-file="$output_min$ext"
    $subsetter "$output_halt$ext" --unicodes=3002 $features --output-file="$output_halt-3002$ext"
    $subsetter "$output_halt$ext" --unicodes=FF1A $features --output-file="$output_halt-FF1A$ext"
  )
done
