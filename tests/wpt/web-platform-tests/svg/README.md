The `import` directory contains tests imported from the [SVG 1.1 Second
Edition test suite](http://www.w3.org/Graphics/SVG/Test/20110816/),
with tests renamed to contain `-manual` in their name.  These tests need
review to verify that they are still correct for the latest version of
SVG (which at the time of writing is [SVG 2](https://svgwg.org/svg2-draft/))
and then need to be converted to reftests or testharness.js-based
tests.

The SVG 1.1 test suite came with [reference
PNGs](http://dev.w3.org/SVG/profiles/1.1F2/test/png/) for each test,
which, while not suitable as exact reftest reference files, at least
give a rough indication of what the test should look like.  For some
tests, such as those involving filters, the test pass criteria are
written with reference to the PNGs.  When converting the tests to
reftests or testharness.js-based tests, you might want to consult the
reference PNG.

Tests should be placed in a directory named after the SVG 2 chapter name
(for example in the `shapes/` directory for Basic Shapes chapter tests).
Scripted tests should be placed under a `scripted/` subdirectory and
reftests under a `reftests/` subdirectory, within the chapter directory.
Filenames for tests of DOM methods and properties should start with
*InterfaceName*.*methodOrPropertyName*, such as
`types/scripted/SVGElement.ownerSVGElement-01.html`.

Direct questions about the imported SVG 1.1 tests to
[Cameron McCormack](mailto:cam@mcc.id.au).
