---
layout: page
title: Reftests
order: 3
---

Reftests are one of the primary tools for testing things relating to
rendering; they are made up of the test and one or more other pages
("references") with assertions as to whether they render identically
or not.

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

## Complex Pass Conditions

Sometimes it is desirable for a file to match multiple references or,
in rare cases, to allow it to match more than one possible reference.

References can have links to other references (through the same `link`
element relation), and in this case for the test to pass the test must
render identically (assuming a `match` relation) to the reference, and
the reference must render identically to its reference (again,
assuming a `match` relation). Note that this can continue indefinitely
to require tests to match an arbitrary number of references; also that
`match` is used here purely for explanatory reasons: both `match` and
`mismatch` can be used (and mixed on one sequence of references). This
can be thought of as an AND operator!

Similarly, multiple references can be linked from a single file to
implement alternates and allow multiple renderings. In this case, the
file passes if it matches one of the references provided (and that
reference likewise matches any references, etc.). This can be thought
of as an OR operator!

These two techniques can be combined to build up arbitrarily complex
pass conditions with boolean logic. For example, consider when:

 * `a.html` has `<link rel=match href=b.html>` and `<link rel=match
href=c.html>`,
 * `b.html` has `<link rel=match href=b1.html>`, and
 * `c.html` has `<link rel=mismatch href=c1.html>`.

Or, graphically:

<img src="{{ site.baseurl }}{% link assets/reftest_graph_example.svg %}"
     alt="diagram of the above reftest graph as a directed graph">

In this case, to pass we must either have `a.html`, `b.html` and
`b1.html` all rendering identically, or `a.html` and `c.html`
rendering identically with `c1.html` rendering differently. (These
are, in terms of the graph, all the paths from the source nodes to
leaf nodes.)

## Controlling When Comparison Occurs

By default reftest screenshots are taken after the `load` event has
fired, and web fonts (if any) are loaded. In some cases it is
necessary to delay the screenshot later than this, for example because
some DOM manipulation is required to set up the desired test
conditions. To enable this, the test may have a `class="reftest-wait"`
attribute specified on the root element. This will cause the
screenshot to be delayed until the `load` event has fired and the
`reftest-wait` class has been removed from the root element. Note that
in neither case is exact timing of the screenshot guaranteed: it is
only guaranteed to be after those events.

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

## Limitations

In some cases, a test cannot be a reftest. For example, there is no
way to create a reference for underlining, since the position and
thickness of the underline depends on the UA, the font, and/or the
platform. However, once it's established that underlining an inline
element works, it's possible to construct a reftest for underlining
a block element, by constructing a reference using underlines on a
```<span>``` that wraps all the content inside the block.

## Example Reftests

This example follows the recommended approach in being a
self-describing test as it has a simple statement on the page
describing how it should render to pass the tests.

### Test File

This test verifies that a right-to-left rendering of **SAW** within a
```<bdo>``` element displays as **WAS**.

```html
<!DOCTYPE html>
<meta charset="utf-8">
<title>BDO element dir=rtl</title>
<link rel="help" href="https://html.spec.whatwg.org/#the-bdo-element">
<meta name="assert" content="BDO element's DIR content attribute renders corrently given value of 'rtl'.">
<link rel="match" href="test-bdo-001.html">
<p>Pass if you see WAS displayed below.</p>
<bdo dir="rtl">SAW</bdo>
```

### Reference File

The reference file must look exactly like the test file,
except that the code behind it is different.

* All metadata is removed.
* The ```title``` need not match.
* The markup that created the actual test data is
  different: here, the same effect is created with
  very mundane, dependable technology.

```html
<!DOCTYPE html>
<meta charset="utf-8">
<title>HTML Reference File</title>
<p>Pass if you see WAS displayed below.</p>
<p>WAS</p>
```


[general guidelines]: {{ site.baseurl }}{% link _writing-tests/general-guidelines.md %}
[rendering]: {{ site.baseurl }}{% link _writing-tests/rendering.md %}
