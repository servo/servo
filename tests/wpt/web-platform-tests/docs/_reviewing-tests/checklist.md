---
layout: page
title: Review Checklist
order: 2
---

The following checklist is provided as a guideline to assist in reviewing
tests; in case of any contradiction with requirements stated elsewhere in the
documentation it should be ignored
(please [file a bug](https://github.com/web-platform-tests/wpt/issues/new)!).

As noted on the [reviewing tests][review index] page, nits need not block PRs
from landing.


## All tests

<label>
<input type="checkbox">
The CI jobs on the pull request have passed.
</label>

<label>
<input type="checkbox">
It is obvious what the test is trying to test.
</label>

<label>
<input type="checkbox">
The test passes when it's supposed to pass.
</label>

<label>
<input type="checkbox">
The test fails when it's supposed to fail.
</label>

<label>
<input type="checkbox">
The test is testing what it thinks it's testing.
</label>

<label>
<input type="checkbox">
The spec backs up the expected behavior in the test.
</label>

<label>
<input type="checkbox">
The test is automated as either [reftest][reftest] or
a [script test][scripttest] unless there's a very good reason for it not to be.
</label>

<label>
<input type="checkbox">
The test does not use external resources.
</label>

<label>
<input type="checkbox">
The test does not use proprietary features (vendor-prefixed or otherwise).
</label>

<label>
<input type="checkbox">
The test does not contain commented-out code.
</label>

<label>
<input type="checkbox">
The test is placed in the relevant directory.
</label>

<label>
<input type="checkbox">
The test has a reasonable and concise filename.
</label>

<label>
<input type="checkbox">
If the test needs code running on the server side, the server code must be
written in Python, and the Python code must not do anything potentially unsafe.
</label>

<label>
<input type="checkbox">
If the test needs to be run in some non-standard configuration or needs user
interaction, it is a manual test.
</label>

<label>
<input type="checkbox">
**Nit**: The title is descriptive but not too wordy.
</label>


## Reftests Only

<label>
<input type="checkbox">
The reference file is accurate and will render pixel-perfect
identically to the test on all platforms.
</label>

<label>
<input type="checkbox">
The reference file uses a different technique that won't fail in
the same way as the test.
</label>

<label>
<input type="checkbox">
The test and reference render within a 600x600 viewport, only displaying
scrollbars if their presence is being tested.
</label>

<label>
<input type="checkbox">
**Nit**: The test has a self-describing statement.
</label>

<label>
<input type="checkbox">
**Nit**: The self-describing statement is accurate, precise, simple, and
self-explanatory. Someone with no technical knowledge should be able to say
whether the test passed or failed within a few seconds, and not need to spend
several minutes thinking or asking questions.
</label>


## Script Tests Only

<label>
<input type="checkbox">
The number of tests in each file and the test names are consistent
across runs and browsers. It is best to avoid the pattern where there is
a test that asserts that the feature is supported and bails out without
running the rest of the tests in the file if it isn't.
</label>

<label>
<input type="checkbox">
The test avoids patterns that make it less likely to be stable.
In particular, tests should avoid setting internal timeouts, since the
time taken to run it may vary on different devices; events should be used
instead (if at all possible).
</label>

<label>
<input type="checkbox">
The test uses the most specific asserts possible (e.g. doesn't use
`assert_true` for everything).
</label>

<label>
<input type="checkbox">
The test uses `idlharness.js` if it is testing basic IDL-defined behavior.
</label>

<label>
<input type="checkbox">
**Nit**: Tests in a single file are separated by one empty line.
</label>


## Visual Tests Only

<label>
<input type="checkbox">
The test has a self-describing statement.
</label>

<label>
<input type="checkbox">
The self-describing statement is accurate, precise, simple, and
self-explanatory. Someone with no technical knowledge should be able to say
whether the test passed or failed within a few seconds, and not need to spend
several minutes thinking or asking questions.
</label>

<label>
<input type="checkbox">
The test renders within a 600x600 viewport, only displaying scrollbars if their
presence is being tested.
</label>

<label>
<input type="checkbox">
The test renders to a fixed, static page with no animation.
</label>


[review index]: {{ site.baseurl }}{% link _reviewing-tests/index.md %}
[general guidelines]: {{ site.baseurl }}{% link _writing-tests/general-guidelines.md %}
[reftest]: {{ site.baseurl }}{% link _writing-tests/reftests.md %}
[scripttest]: {{ site.baseurl }}{% link _writing-tests/testharness.md %}
