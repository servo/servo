# Manual Tests

Some testing scenarios are intrinsically difficult to automate and
require a human to run the test and check the pass condition.

## When to Write Manual Tests

Whenever possible it's best to write a fully automated test. For a
browser vendor it's possible to run an automated test hundreds of
times a day, but manual tests are likely to be run at most a handful
of times a year (and quite possibly approximately never!). This makes
them significantly less useful for catching regressions than automated
tests.

However, there are certain scenarios in which this is not yet
possible. For example:

* Test which require observing animation (e.g., a test for CSS
  animation or for video playback),

* Tests that require interaction with browser security UI (e.g., a
  test in which a user refuses a geolocation permissions grant),

* Tests that require interaction with the underlying OS (e.g., tests
  for drag and drop from the desktop onto the browser),

* Tests that require non-default browser configuration (e.g., images
  disabled), and

* Tests that require interaction with the physical environment (e.g.,
  tests that the vibration API causes the device to vibrate or that
  various sensor APIs respond in the expected way).

## Requirements for a Manual Test

Manual tests are distinguished by their filename; all manual tests
have filenames of the form `name-manual.ext` (i.e., a `-manual` suffix
after the main filename but before the extension).

Manual tests must be
fully
[self-describing](general-guidelines).
It is particularly important for these tests that it is easy to
determine the result from the information provided in the page to the
tester, because a tester may have hundreds of tests to get through and
little understanding of the features that they are testing. As a
result, minimalism is especially a virtue for manual tests.

A test should have, at a minimum step-by-step instructions for
performing the test, and a clear statement of either the test result
if it can be automatically determined after some setup or how to
otherwise determine the outcome.

Any information other than this (e.g., quotes from the spec) should be
avoided (though, as always, can be provided in
HTML/CSS/JS/etc. comments).

## Using testharness.js for Manual Tests

A convenient way to present the results of a test that can have the
result determined by script after some manual setup steps is to use
testharness.js to determine and present the result. In this case one
must pass `{explicit_timeout: true}` in a call to `setup()` in order
to disable the automatic timeout of the test. For example:

```html
<!doctype html>
<title>Manual click on button triggers onclick handler</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>
setup({explicit_timeout: true})
</script>
<p>Click on the button below. If a "PASS" result appears the test
passes, otherwise it fails</p>
<button onclick="done()">Click Here</button>
```
