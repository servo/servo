## Key Aspects of a Well Designed Test

A badly written test can lead to false passes or false failures, as
well as inaccurate interpretations of the specs. Therefore it is
important that the tests all be of a high standard. All tests must
follow the [test format guidelines][test-format] and well designed
tests should meet the following criteria:

* **The test passes when it's supposed to pass**
* **The test fails when it's supposed to fail**
* **It's testing what it claims to be testing**

## Self-Describing Tests

As the tests are likely to be used by many other people, making them
easy to understand is very important. Ideally, tests are written as
self-describing, which is a test page that describes what the page
should look like when the test has passed. A human examining the
test page can then determine from the description whether the test
has passed or failed.

_Note: The terms "the test has passed" and "the test has failed"
refer to whether the user agent has passed or failed a
particular test — a test can pass in one web browser and fail in
another. In general, the language "the test has passed" is used
when it is clear from context that a particular user agent is
being tested, and the term "this-or-that-user-agent has passed
the test" is used when multiple user agents are being compared._

Self-describing tests have some advantages:

* They can be run easily on any layout engine.
* They can test areas of the spec that are not precise enough to be
  comparable to a reference rendering. (For example, underlining
  cannot be compared to a reference because the position and
  thickness of the underline is UA-dependent.)
* Failures can (should) be easily determined by a human viewing the
  test without needing special tools.

### Manual Tests

While it is highly encouraged to write automatable tests either as [
reftests][reftests] or [script tests][scripttests], in rare cases a
test can only be executed manually. All manual tests must be
self-describing tests. Additionally, manual tests should be:

* Easy & quick to determine the result
* Self explanatory & not require an understanding of the
  specification to determine the result
* Short (a paragraph or so) and certainly not require scrolling
  on even the most modest of screens, unless the test is
  specifically for scrolling or paginating behaviour.

### Reftests

[Reftests][reftests] should be self-describing tests wherever
possible. This means the the descriptive statement included in the
test file must also appear in the reference file so their renderings
may be automatically compared.

### Script Tests

[Script tests][scripttests] may also be self-describing, but rather
than including a supplemental statement on the page, this is
generally done in the test results output from ```testharness.js```.

### Self-Describing Test Examples

The following are some examples of self-describing tests, using some
common [techniques](#techniques) to identify passes:

* [Identical Renderings][identical-renderings]
* [Green Background][green-background]
* [No Red 1][no-red-1]
* [No Red 2][no-red-2]
* [Described Alignment][described-alignment]
* [Overlapping][overlapping]
* [Imprecise Description 1][imprecise-1]
* [Imprecise Description 2][imprecise-2]

## Techniques

In addition to the [self describing](#self-describing) statement
visible in the test, there are many techniques commonly used to add
clarity and robustness to tests. Particularly for reftests, which
rely wholly on how the page is rendered, the following should be
considered and used when designing new tests.

### Indicating success

#### The green paragraph

This is the simplest form of test, and is most often used when
testing the things that are independent of the rendering, like
the CSS cascade or selectors. Such tests consist of a single line of
text describing the pass condition, which will be one of the
following:

<span style="color: green">This line should be green.</span>

<span style="border: 5px solid green">This line should have a green
  border.</span>

<span style="background: green; color: white">This line should have
  a green background.</span>

#### The green page

This is a variant on the green paragraph test. There are certain
parts of CSS that will affect the entire page, when testing these
this category of test may be used. Care has to be taken when writing
tests like this that the test will not result in a single green
paragraph if it fails. This is usually done by forcing the short
descriptive paragraph to have a neutral color (e.g. white).

This [example][green-page] is poorly designed, because it does not
look red when it has failed.

#### The green square

This is the best type of test for cases where a particular rendering
rule is being tested. The test usually consists of two boxes of some
kind that are (through the use of positioning, negative margins,
zero line height, transforms, or other mechanisms) carefully placed
over each other. The bottom box is colored red, and the top box is
colored green. Should the top box be misplaced by a faulty user
agent, it will cause the red to be shown. (These tests sometimes
come in pairs, one checking that the first box is no bigger than the
second, and the other checking the reverse.) These tests frequently
look like:

<p>Test passes if there is a green square and no red.</p>
<div style="width: 100px; height: 100px; background: green"></div>

#### The green paragraph and the blank page

These tests appear to be identical to the green paragraph tests
mentioned above. In reality, however, they actually have more in
common with the green square tests, but with the green square
colored white instead. This type of test is used when the
displacement that could be expected in the case of failure is
likely to be very small, and so any red must be made as obvious as
possible. Because of this, test would appear totally blank when the
test has passed. This is a problem because a blank page is the
symptom of a badly handled network error. For this reason, a single
line of green text is added to the top of the test, reading
something like:

<p style="color: green">This line should be green and there should
be no red on this page.</p>
[Example][green-paragraph]

#### The two identical renderings

It is often hard to make a test that is purely green when the test
passes and visibly red when the test fails. For these cases, it may
be easier to make a particular pattern using the feature that is
being tested, and then have a reference rendering next to the test
showing exactly what the test should look like.

The reference rendering could be either an image, in the case where
the rendering should be identical, to the pixel, on any machine, or
the same pattern made using different features. (Doing the second
has the advantage of making the test a test of both the feature
under test and the features used to make the reference rendering.)

[Visual Example 1][identical-visual-1]

[Visual Example 2][identical-visual-2]

[Text-only Example][identical-text]

### Indicating failure

In addition to having clearly defined characteristics when
they pass, well designed tests should have some clear signs when
they fail. It can sometimes be hard to make a test do something only
when the test fails, because it is very hard to predict how user
agents will fail! Furthermore, in a rather ironic twist, the best
tests are those that catch the most unpredictable failures!

Having said that, here are the best ways to indicate failures:

#### Red

Using the color red is probably the best way of highlighting
failures. Tests should be designed so that if the rendering is a few
pixels off some red is uncovered or otherwise rendered on the page.

[Visual Example][red-visual]

[Text-only Example][red-text]

_View the pages' source to see the usage of the color
red to denote failure._

#### Overlapped text

Tests of the 'line-height', 'font-size' and similar properties can
sometimes be devised in such a way that a failure will result in the
text overlapping.

#### The word "FAIL"

Some properties lend themselves well to this kind of test, for
example 'quotes' and 'content'. The idea is that if the word "FAIL"
appears anywhere, something must have gone wrong.

[Example][fail-example]

_View the page's source to see the usage of the word FAIL._

### Special Fonts

#### Ahem
Todd Fahrner has developed a font called [Ahem][ahem-readme], which
consists of some very well defined glyphs of precise sizes and
shapes. This font is especially useful for testing font and text
properties. Without this font it would be very hard to use the
overlapping technique with text.

The font's em-square is exactly square. It's ascent and descent is
exactly the size of the em square. This means that the font's extent
is exactly the same as its line-height, meaning that it can be
exactly aligned with padding, borders, margins, and so forth.

The font's alphabetic baseline is 0.2em above its bottom, and 0.8em
below its top.

The font has four glyphs:

* X U+0058  A square exactly 1em in height and width.
* p U+0070  A rectangle exactly 0.2em high, 1em wide, and aligned so
that its top is flush with the baseline.
* É U+00C9  A rectangle exactly 0.8em high, 1em wide, and aligned so
that its bottom is flush with the baseline.
* U+0020  A transparent space exactly 1em high and wide.

Most other US-ASCII characters in the font have the same glyph as X.

#### Ahem Usage
__If the test uses the Ahem font, make sure its computed font-size
is a multiple of 5px__, otherwise baseline alignment may be rendered
inconsistently (due to rounding errors introduced by certain
platforms' font APIs). We suggest to use a minimum computed font-
size of 20px.

E.g. Bad:

``` css
{font: 1in/1em Ahem;}  /* Computed font-size is 96px */
{font: 1in Ahem;}
{font: 1em/1em Ahem} /* with computed 1em font-size being 16px */
{font: 1em Ahem;} /* with computed 1em font-size being 16px */
```

E.g. Good:

``` css
{font: 100px/1 Ahem;}
{font: 1.25em/1 Ahem;} /* with computed 1.25em font-size being 20px
*/
```

__If the test uses the Ahem font, make sure the line-height on block
elements is specified; avoid 'line-height: normal'__. Also, for
absolute reliability, the difference between computed line-height
and computed font-size should be dividable by 2.

E.g. Bad:

``` css
{font: 1.25em Ahem;} /* computed line-height value is 'normal' */
{font: 20px Ahem;} /* computed line-height value is 'normal' */
{font-size: 25px; line-height: 50px;} /* the difference between
computed line-height and computed font-size is not dividable by 2. */
```

E.g. Good:

``` css
{font-size: 25px; line-height: 51px;} /* the difference between
computed line-height and computed font-size is dividable by 2. */
```

[Example test using Ahem][ahem-example]

_View the page's source to see how the Ahem font is used._


##### Installing Ahem

1. Download the [TrueType version of Ahem][download-ahem].
2. Open the folder where you downloaded the font file.
3. Right-click the downloaded font file and select "Install".

### Explanatory Text

For tests that must be long (e.g. scrolling tests), it is important
to make it clear that the filler text is not relevant, otherwise the
tester may think he is missing something and therefore waste time
reading the filler text. Good text for use in these situations is,
quite simply, "This is filler text. This is filler text. This is
filler text.". If it looks boring, it's working!

### Color

In general, using colors in a consistent manner is recommended.
Specifically, the following convention has been developed:

#### Red
Any red indicates failure.

#### Green
In the absence of any red, green indicates success.

#### Blue
Tests that do not use red or green to indicate success or failure
should use blue to indicate that the tester should read the text
carefully to determine the pass conditions.

#### Black
Descriptive text is usually black.

#### Fuchsia, Yellow, Teal, Orange
These are useful colors when making complicated patterns for tests
of the two identical renderings type.

#### Dark Gray
Descriptive lines, such as borders around nested boxes, are usually
dark gray. These lines come in useful when trying to reduce the test
for engineers.

#### Silver / Light Gray

Sometimes used for filler text to indicate that it is irrelevant.

### Methodical testing

Some web features can be tested quite thoroughly with a very
methodical approach. For example, testing that all the length units
work for each property taking lengths is relatively easy, and can be
done methodically simply by creating a test for each property/unit
combination.

In practice, the important thing to decide is when to be methodical
and when to simply test, in an ad hoc fashion, a cross section of
the possibilities.

This example is a methodical test of the :not() pseudo-class with
each attribute selector in turn, first for long values and then for
short values:

http://www.hixie.ch/tests/adhoc/css/selectors/not/010.xml

### Overlapping

This technique should not be cast aside as a curiosity -- it is in
fact one of the most useful techniques for testing CSS, especially
for areas like positioning and the table model.

The basic idea is that a red box is first placed using one set of
properties, e.g. the block box model's margin, height and width
properties, and then a second box, green, is placed on top of the
red one using a different set of properties, e.g. using absolute
positioning.

This idea can be extended to any kind of overlapping, for example
overlapping to lines of identical text of different colors.

## Tests to avoid

### The long test

Any manual test that is so long that is needs to be scrolled to be
completed is too long. The reason for this becomes obvious when you
consider how manual tests will be run. Typically, the tester will be
running a program (such as "Loaderman") which cycles through a list
of several hundred tests. Whenever a failure is detected, the tester
will do something (such as hit a key) that takes a note of the test
case name. Each test will be on the screen for about two or three
seconds. If the tester has to scroll the page, that means he has to
stop the test to do so.

Of course, there are exceptions -- the most obvious one being any
tests that examine the scrolling mechanism! However, these tests are
considered tests of user interaction and are not run with the
majority of the tests.

Any test that is so long that it needs scrolling can usually be
split into several smaller tests, so in practice this isn't much of
a problem.

This is an [example][long-test] of a test that is too long.

### The counterintuitive "this should be red" test

As mentioned many times in this document, red indicates a bug, so
nothing should ever be red in a test.

There is one important exception to this rule... the test for the
'red' value for the color properties!

### Unobvious tests

A test that has half a sentence of normal text with the second half
bold if the test has passed is not very obvious, even if the
sentence in question explains what should happen.

There are various ways to avoid this kind of test, but no general
rule can be given since the affected tests are so varied.

The last [subtest on this page][unobvious-test] shows this problem.

[test-format]: ./test-format-guidelines.html
[reftests]: ./reftests.html
[scripttests]: ./testharness-documentation.html
[identical-renderings]: http://test.csswg.org/source/css21/syntax/escapes-000.xht
[green-background]: http://test.csswg.org/source/css21/syntax/escapes-002.xht
[no-red-1]: http://test.csswg.org/source/css21/positioning/abspos-containing-block-003.xht
[no-red-2]: http://test.csswg.org/source/css21/tables/border-conflict-w-079.xht
[described-alignment]: http://test.csswg.org/source/css21/margin-padding-clear/margin-collapse-clear-007.xht
[overlapping]: http://test.csswg.org/source/css21/tables/table-anonymous-objects-021.xht
[imprecise-1]: http://test.csswg.org/source/css21/tables/border-style-inset-001.xht
[imprecise-2]: http://test.csswg.org/source/css21/text/text-decoration-001.xht
[green-page]: http://www.hixie.ch/tests/adhoc/css/background/18.xml
[green-paragraph]: http://www.hixie.ch/tests/adhoc/css/fonts/size/002.xml
[identical-visual-1]: http://test.csswg.org/source/css21/floats-clear/margin-collapse-123.xht
[identical-visual-2]: http://test.csswg.org/source/css21/normal-flow/inlines-016.xht
[identical-text]: http://test.csswg.org/source/css21/fonts/shand-font-000.xht
[red-visual]: http://test.csswg.org/source/css21/positioning/absolute-replaced-height-018.xht
[red-text]: http://test.csswg.org/source/css21/syntax/comments-003.xht
[fail-example]: http://test.csswg.org/source/css21/positioning/abspos-overflow-005.xht
[ahem-example]: http://test.csswg.org/source/css21/positioning/absolute-non-replaced-width-001.xht
[ahem-readme]: http://www.w3.org/Style/CSS/Test/Fonts/Ahem/README
[download-ahem]: http://www.w3.org/Style/CSS/Test/Fonts/Ahem/AHEM____.TTF
[long-test]: http://www.hixie.ch/tests/evil/mixed/lineheight3.html
[unobvious-test]: http://www.w3.org/Style/CSS/Test/CSS1/current/sec525.htm
