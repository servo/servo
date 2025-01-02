## Highlight pseudos

There are many subtle browser differences in rendering these pseudo-elements,
most of which comply with the specs, so here are some hints.

Use the helpers in `support/highlights.css` and `support/selections.js` where
possible. They include rules and functions for “basic” but tricky tasks like
setting up a test area, selecting content, and triggering spellcheck.

When creating complex layered references, start by wrapping your text in a
relative container, then prepend absolute layers with copies of that text, then
mark up those copies with spans. Make everything transparent initially, and set
visible styles on the spans only. The absolute layers will perfectly overlap
your original text, which you can keep for external layout. For example:

```html
<div class="container">
    <div class="spelling-error"><span>Teh</span> <span>dgo</span> and
        <span>teh</span> <span>sphixn</span>.</div>
    <div class="selection">Teh d<span>go and te</span>h sphixn.</div>
    Teh dgo and teh sphixn.
</div>
```
```css
.container { position: relative; color: transparent; }
.container > * { position: absolute; }
.spelling-error > span { background: ...; color: ...; }
.selection > span { background: ...; text-shadow: ...; }
```

Simplify this pattern at your own peril! For example, if you set backgrounds
directly on layers as your highlight backgrounds, they will always be exactly
`line-height` tall, but even if your `line-height` is 1, the actual line boxes
and so on can still be taller (unless they contain Ahem text only).


## Selection regression tests

Four tests are based on the properties described in <https://crrev.com/915543>,
and were designed to catch regressions as bugs were fixed in Chromium:

*   selection-originating-underline-order.html (P1)
*   selection-originating-decoration-color.html (P3)
*   selection-originating-strikethrough-order.html (P4)
*   selection-background-painting-order.html (P5)

Ideally we would want a test for property P2, that line-through decorations are
always painted over text when selecting some of that text. But unfortunately,
originating decoration recoloring (when correctly implemented) essentially makes
it impossible to tell whether the text or the decoration was painted on top.

Some ways this test could become possible:

*   Wider impl support for ::target-text or ::highlight decorations.
    Decorations introduced by highlight pseudos aren’t recolored, so
    we could move the originating text-decoration to any highlight
    that paints under ::selection (currently all of them), choose
    another ::selection color, and check which is painted on top.

*   SVG adds support for text-decoration-color, or HTML adds support
    for stroke and stroke-width via CSS, as long as we continue to
    recolor originating decorations to color only. Then we could
    stroke in another color, and check which is painted on top.

*   css-pseudo adds some kind of support for suppressing or otherwise
    tweaking the recoloring of originating decorations.

*   Some other standard means for text to contain colors other than
    the color property, such as color fonts.
