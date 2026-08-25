# Tests for HTML Ruby Markup extensions

Specification: https://www.w3.org/TR/html-ruby-extensions/

WARNING: These are manual tests.
There is some support for automation,
but the results must be evaluated manually.
Simply relying on automated reports of tests passing is not sufficient.

These tests are hard to write reliably,
because without relying on a styling mechanism (which should be tested separately),
there's no prescribed rendering,
yet the rendering is how we can tell whether the markup did the right thing:
ruby must be segmented correctly,
and the correct ruby annotation must be paired with the correct base.
That is something you can tell visually.

The approach taken here follows the same logic
as the pre-existing html/semantics/text-level-semantics/the-ruby-element/ruby-usage.html:
use a mismatch ref-test against what the rendering is likely to be
if the browser didn't do anything, or did the wrong thing.
In that original example, the mismatch reference is simply what you'd get in a browser with no support for ruby at all.
The tests in this directory do that too,
and add a few variants of possible wrong renderings,
some attested in existing layout engines,
some "just in case".

As such, automated test failures are indicative of something being wrong with the implementation,
but tests passing could be false positives:
maybe it is implemented right,
or maybe it is implemented wrong in a novel way.

Therefore, each test is written including a description of the pass condition,
in a way that can be evaluated by a person looking at the test.

It would be better to write these tests so that their pass condition can be automated,
but as far as I can tell,
that's not reliably doable.

For instance, it might be tempting to use `<ruby>X<rt>1</rt>Y<rt>2</rt></ruby>`
or `<ruby>X<rt>1</ruby><ruby>Y<rt>2</ruby>`
as a reference for `<ruby><rb>X<rb>Y<rt>1<rt>2</ruby>`
as they are defined to have the same base/annotation pairing,
but it is not required that they have precisely the same rendering.
And indeed, some implementations do vary
(notably in terms of base/annotation alignment).

So we're left with semi-manual tests.

Anyone finding false positives is encouraged
to add corresponding mismatch references.
