When reviewing a test, make sure the test follows the
[format][format] and [style][style] guidelines.

In addition, the test should be checked for the following:

## All tests
<input type="checkbox">
The test passes when it's supposed to pass

<input type="checkbox">
The test fails when it's supposed to fail

<input type="checkbox">
The test is testing what it thinks it's testing

<input type="checkbox">
The spec backs up the expected behavior in the test.

<input type="checkbox">
The test is automated as either [reftest][reftest] or a
[script test][scripttest] unless there's a very good reason why the
test must be manual.

<input type="checkbox">
The test does not use external resources.

<input type="checkbox">
The test does not use proprietary features (vendor-prefixed or otherwise).


## Reftests Only
<input type="checkbox">
The test has a [self-describing][selftest] statement

<input type="checkbox">
The self-describing statement is accurate, precise, simple, and
self-explanatory. Your mother/husband/roommate/brother/bus driver
should be able to say whether the test passed or failed within a few
seconds, and not need to spend several minutes thinking or asking
questions.

<input type="checkbox">
The reference file is accurate and will render pixel-perfect
identically to the test on all platforms.

<input type="checkbox">
The reference file uses a different technique that won't fail in
the same way as the test.

<input type="checkbox">
The title is descriptive but not too wordy.

<input type="checkbox">
The test is as cross-platform as reasonably possible, working
across different devices, screen resolutions, paper sizes, etc. If
there are limitations (e.g. the test will only work on 96dpi
devices, or screens wider than 200 pixels) that these are documented
in the instructions.


## Script Tests Only

<input type="checkbox">
The test uses the most specific asserts possible (e.g. doesn't use
`assert_true` for everything).

<input type="checkbox">
The number of tests in each file and the test names are consistent
across runs and browsers. It is best to avoid the pattern where there is
a test that asserts that the feature is supported and bails out without
running the rest of the tests in the file if it isn't.

<input type="checkbox">
The test avoids patterns that make it less likely to be stable.
In particular, tests should avoid setting internal timeouts, since the
time taken to run it may vary on different devices; events should be used
instead (if at all possible).

<input type="checkbox">
The test uses `idlharness.js` if it covers the use case.

<input type="checkbox">
Tests in a single file are separated by one empty line.


## In depth Checklist

<input type="checkbox">
A test does not use self-closing start tag ("/" (U+002F)) when using the
HTML syntax.

<input type="checkbox">
The test does not use the Unicode byte order mark (BOM U+FEFF). The test
uses Unix line endings (LF, no CR). The executable bit is not set
unnecessarily.

<input type="checkbox">
For indentation, spaces are preferred over tabs.

<input type="checkbox">
The test does not contain trailing whitespace (whitespace at the end of
lines).

<input type="checkbox">
The test does not contain commented-out code.

<input type="checkbox">
The test is placed in the relevant directory, based on the /TR latest
version link if available.

<input type="checkbox">
If the test needs code running on the server side, the server code must
be written in python, and the python code must be reviewed carefully to
ensure it isn't doing anything dangerous.



[format]: ./test-format-guidelines.html
[style]: ./test-style-guidelines.html
[reftest]: ./reftests.html
[scripttest]: ./testharness-documentation.html
[selftest]: ./test-style-guidelines.html#self-describing
