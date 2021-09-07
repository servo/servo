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
