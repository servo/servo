#!/bin/bash
#
# Produces subset font files for testing the `text-spacing-trim` property.
#
# Requires `pyftsubset` from <https://github.com/fonttools/fonttools>.
#
range="--unicodes=20-7E,2018-201F,56FD,6C34,3000-301F,30FB,FF01-FF1F,FF5B-FF65"
subrange="--unicodes=20-7E,56FD,FF08-FF09"
features="--layout-features+=halt,fwid,hwid,palt,pwid,vhal,vpal"
features_chws="$features,chws,vchw"
for path in "$@"; do
  filename="$(basename -- "$path")"
  stem="${filename%.*}"
  output_halt="$stem-subset-halt.otf"
  output_chws="$stem-subset-chws.otf"
  output_min="$stem-subset-halt-min.otf"
  (set -x;
    pyftsubset "$path" $range $features --output-file="$output_halt"
    pyftsubset "$path" $range $features_chws --output-file="$output_chws"
    pyftsubset "$output_halt" $subrange $features --output-file="$output_min"
  )
done
