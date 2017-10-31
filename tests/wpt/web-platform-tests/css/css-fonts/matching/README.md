# Stretch, Style, Weight Matching Tests
This directory contains a set of tests for [the font style matching algorithm, section 5.2](https://drafts.csswg.org/css-fonts-4/#font-style-matching) of the [CSS Fonts Module Level 4](https://drafts.csswg.org/css-fonts-4/) specification.

In Level 4 of the spec, the font style matching algorithm has been extended to
support [OpenType Variations](https://www.microsoft.com/typography/otspec/otvaroverview.htm). This means, the
`@font-face` property descriptors accept ranges for `font-style`, `font-stretch` and `font-weight` and the matching
algorithm has been modified to take these into account, and to set variable fonts axis parameters accordingly.

## Test Font

For testing font matching with Variations support a test font called **Variable Test Axis Matching** was created (`variabletest_matching.ttf`).

The design goal for this font is to match variable glyphs against non-variable, static reference glyphs. The variable glyphs are
scaled according to variation axes for stretch, style, weight. The reference glyphs are not subject to variation interpolation.

### Test Glyphs
The test font contains glyphs M, N, O, P which scale according to the `wdth`, `slnt`, `ital`, and `wght` registered axes respectively. Glyphs M, N, O have zero advance width. When they are combined with the last, the P glyph, which has a width of 2000 FUnits, they form a full "test grapheme". The glyphs M, N, O, P line up vertically to form something resembling a bar chart of the applied axis parameters. For example, when the `wdth` design space coordinate is set to 100, the M bar glyph is 200 FUnits wide, when it is set to 500, the M bar glyph is 1000 FUnits wide.

### Reference Glyphs

The **Variable Test Axis Matching** font contains reference glyphs 0-9 to match different stops in the design coordinates space of the `wdth` axis, from 0 matching 200 FUnits to 9 matching 2000 FUnits. Analogously, glyphs p, q, w, e, r, t, y, u (the row between the numbers on a QWERTY keyboard) line up to match the N glyph at various stops for `slnt`. Glyphs ;, a, s, d, e, f, g, h, j, k, l match the O glyph for `ital`, and finally /, z, x, c, v, b, n, m match the P glyph for `wdth`.


### Building reference tests

Using the **Variable Test Axis Matching** font, [reference tests](http://web-platform-tests.org/writing-tests/reftests.html) in this directory are created as follows:

 1. Define `@font-face`s with range expressions, which trigger variation axes to be applied to the variable font.
 2. Use CSS style definitions to request font faces from the set of declared `@font-face`s and use blocks of the glyph sequence MNOP.
 3. To avoid flakiness, add reftest-wait to the html root element and use JS to remove it once font loading is complete.
 4. When the test is run, a screenshot is generated from the resulting output rendering.
 5. Define a reference rendering in a *-ref.html file, using only the non-variable reference glyphs q-p, a-;, z-/.
 6. When the test is run, a screenshot for the reference rendering is generated.
 7. For the test to pass the screenshot from 4. using OpenType Variations is compared to the reference screenshot from 6. (which is no using OpenType variations).

## Font Glyphs Reference

The following table explains the relationship between the M, N, O, P variation axis controlled glyphs and the non-scaled glyphs used as references.

| Bar Length in FUnits | 200 | 400 | 600 | 800 | 1000 | 1200 | 1400 | 1600 | 1800
| :---: | :---: |:---: |:---: |:---: |:---: |:---: |:---: |:---: |:---: |
| Glyph **N**, Style, `slnt` | -90.00% | -67.50% | -45.00% | -20.00% | 0.00% | 20.00% | 45.00% | 67.50% | 90.00%
| Glyph **M**, Stretch Axis `wdth` | 50% | 62.50% | 75% | 87.50% | 100% | 112.50% | 125% | 150% | 200%
| Glyph **O**, Style, `ital` | 0 | 0.125 | 0.25 | 0.375 | 0.5 | 0.625 | 0.75 | 0.875 | 1
| Glyph **P**, Weight, `wght` | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
| **Ref Glyphs for:** |  |  |  |  |  |  |  |  |
| **M** | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
| **N** | p | q | w | e | r | t | y | u | i
| **O** | ; | a | s | d | f | g | h | j | k
| **P** | / | z | x | c | v | b | n | m | ,
