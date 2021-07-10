# Reftests

Reftests are one of the primary tools for testing things relating to
rendering; they are made up of the test and one or more other pages
("references") with assertions as to whether they render identically
or not. This page describes their aspects exhaustively; [the tutorial
on writing a reftest](reftest-tutorial) offers a more limited but
grounded guide to the process.

## How to Run Reftests

Reftests can be run manually simply by opening the test and the
reference file in multiple windows or tabs and flipping between the
two. In automation the comparison is done in an automated fashion,
which can lead to differences hard for the human eye to notice to
cause the test to fail.

## Components of a Reftest

In the simplest case, a reftest consists of a pair of files called the
*test* and the *reference*.

The *test* file is the one that makes use of the technology being
tested. It also contains a `link` element with `rel="match"` or
`rel="mismatch"` and `href` attribute pointing to the *reference*
file, e.g. `<link rel=match href=references/green-box-ref.html>`. A
`match` test only passes if the two files render pixel-for-pixel
identically within a 800x600 window *including* scroll-bars if
present; a `mismatch` test only passes if they *don't* render
identically.

The *reference* file is typically written to be as simple as possible,
and does not use the technology under test. It is desirable that the
reference be rendered correctly even in UAs with relatively poor
support for CSS and no support for the technology under test.

## Writing a Good Reftest

In general the files used in a reftest should follow
the [general guidelines][] and
the [rendering test guidelines][rendering]. They should also be
self-describing, to allow a human to determine whether the the
rendering is as expected.

References can be shared between tests; this is strongly encouraged as
it makes it easier to tell at a glance whether a test passes (through
familiarity) and enables some optimizations in automated test
runners. Shared references are typically placed in `references`
directories, either alongside the tests they are expected to be useful
for or at the top level if expected to be generally applicable (e.g.,
many layout tests can be written such that the correct rendering is a
100x100 green square!). For references that are applicable only to a
single test, it is recommended to use the test name with a suffix of
`-ref` as their filename; e.g., `test.html` would have `test-ref.html`
as a reference.

## Multiple References

Sometimes, a test's pass condition cannot be captured in a single
reference.

If a test has multiple links, then the test passes if:

 * If there are any match references, at least one must match, and
 * If there are any mismatch references, all must mismatch.

 If you need multiple matches to succeed, these can be turned into
 multiple tests (for example, by just having a reference be a test
 itself!). If this seems like an unreasonable restriction, please file
 a bug and let us know!

## Controlling When Comparison Occurs

By default, reftest screenshots are taken after the following
conditions are met:

* The `load` event has fired
* Web fonts (if any) are loaded
* Pending paints have completed

In some cases it is necessary to delay the screenshot later than this,
for example because some DOM manipulation is required to set up the
desired test conditions. To enable this, the test may have a
`class="reftest-wait"` attribute specified on the root element. In
this case the harness will run the following sequence of steps:

* Wait for the `load` event to fire and fonts to load.
* Wait for pending paints to complete.
* Fire an event named `TestRendered` at the root element, with the
  `bubbles` attribute set to true.
* Wait for the `reftest-wait` class to be removed from the root
  element.
* Wait for pending paints to complete.
* Screenshot the viewport.

The `TestRendered` event provides a hook for tests to make
modifications to the test document that are not batched into the
initial layout/paint.

## Fuzzy Matching

In some situations a test may have subtle differences in rendering
compared to the reference due to, e.g., anti-aliasing. To allow for
these small differences, we allow tests to specify a fuzziness
characterised by two parameters, both of which must be specified:

 * A maximum difference in the per-channel color value for any pixel.
 * A number of total pixels that may be different.

The maximum difference in the per pixel color value is formally
defined as follows: let <code>T<sub>x,y,c</sub></code> be the value of
colour channel `c` at pixel coordinates `x`, `y` in the test image and
<code>R<sub>x,y,c</sub></code> be the corresponding value in the
reference image, and let <code>width</code> and <code>height</code> be
the dimensions of the image in pixels. Then <code>maxDifference =
max<sub>x=[0,width) y=[0,height), c={r,g,b}</sub>(|T<sub>x,y,c</sub> -
R<sub>x,y,c</sub>|)</code>.

To specify the fuzziness in the test file one may add a `<meta
name=fuzzy>` element (or, in the case of more complex tests, to any
page containing the `<link rel=[mis]match>` elements). In the simplest
case this has a `content` attribute containing the parameters above,
separated by a colon e.g.

```
<meta name=fuzzy content="maxDifference=15;totalPixels=300">
```

would allow for a  difference of exactly 15 / 255 on any color channel
and 300 exactly pixels total difference. The argument names are optional
and may be elided; the above is the same as:

```
<meta name=fuzzy content="15;300">
```

The values may also be given as ranges e.g.

```
<meta name=fuzzy content="maxDifference=10-15;totalPixels=200-300">
```

or

```
<meta name=fuzzy content="10-15;200-300">
```

In this case the maximum pixel difference must be in the range
`10-15` and the total number of different pixels must be in the range
`200-300`.

In cases where a single test has multiple possible refs and the
fuzziness is not the same for all refs, a ref may be specified by
prefixing the `content` value with the relative url for the ref e.g.

```
<meta name=fuzzy content="option1-ref.html:10-15;200-300">
```

One meta element is required per reference requiring a unique
fuzziness value, but any unprefixed value will automatically be
applied to any ref that doesn't have a more specific value.

### Debugging fuzzy reftests

When debugging a fuzzy reftest via `wpt run`, it can be useful to know what the
allowed and detected differences were. Many of the output logger options will
provide this information. For example, by passing `--log-mach=-` for a run of a
hypothetical failing test, one might get:

```
 0:08.15 TEST_START: /foo/bar.html
 0:09.70 INFO Found 250 pixels different, maximum difference per channel 6 on page 1
 0:09.70 INFO Allowed 0-100 pixels different, maximum difference per channel 0-0
 0:09.70 TEST_END: FAIL, expected PASS - /foo/bar.html ['f83385ed9c9bea168108b8c448366678c7941627']
```

For other logging flags, see the output of `wpt run --help`.

## Limitations

In some cases, a test cannot be a reftest. For example, there is no
way to create a reference for underlining, since the position and
thickness of the underline depends on the UA, the font, and/or the
platform. However, once it's established that underlining an inline
element works, it's possible to construct a reftest for underlining
a block element, by constructing a reference using underlines on a
```<span>``` that wraps all the content inside the block.

[general guidelines]: general-guidelines
[rendering]: rendering
