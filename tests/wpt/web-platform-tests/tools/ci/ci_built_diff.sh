#!/bin/bash
set -ex

SCRIPT_DIR=$(cd $(dirname "$0") && pwd -P)
WPT_ROOT=$SCRIPT_DIR/../..
cd $WPT_ROOT

main() {
    # Diff PNGs based on pixel-for-pixel identity
    echo -e '[diff "img"]\n  textconv = identify -quiet -format "%#"' >> .git/config
    echo -e '*.png diff=img' >> .git/info/attributes

    # Exclude tests that rely on font rendering
    excluded=(
        'html/canvas/element/drawing-text-to-the-canvas/2d.text.draw.fill.basic.png'
        'html/canvas/element/drawing-text-to-the-canvas/2d.text.draw.fill.maxWidth.large.png'
        'html/canvas/element/drawing-text-to-the-canvas/2d.text.draw.fill.rtl.png'
        'html/canvas/element/drawing-text-to-the-canvas/2d.text.draw.stroke.basic.png'
        'html/canvas/offscreen/text/2d.text.draw.fill.basic.png'
        'html/canvas/offscreen/text/2d.text.draw.fill.maxWidth.large.png'
        'html/canvas/offscreen/text/2d.text.draw.fill.rtl.png'
        'html/canvas/offscreen/text/2d.text.draw.stroke.basic.png'
    )

    ./update-built-tests.sh
    git update-index --assume-unchanged ${excluded[*]}
    git diff --exit-code
}

main
