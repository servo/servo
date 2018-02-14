# Tests related to Cross-Origin Resource Blocking (CORB).

This directory contains tests related to the
[Cross-Origin Resource Blocking (CORB)](https://chromium.googlesource.com/chromium/src/+/master/content/browser/loader/cross_origin_read_blocking_explainer.md) algorithm.

Note that CORB is currently in very early stages of standardization path.  At
the same time, some tests in this directory (e.g.
`css-with-json-parser-breaker`) cover behavior spec-ed outside of CORB (making
sure that CORB doesn't change the existing web behavior) and therefore are
valuable independently from CORB's standardization efforts.

Tests that cover behavior that is changed by CORB have to be marked as
[tentative](http://web-platform-tests.org/writing-tests/file-names.html)
(using `.tentative` substring in their filename) until CORB
is included in the official
[Fetch spec](https://fetch.spec.whatwg.org/).

The tests in this directory interact with various, random features,
but the tests have been grouped together into the `fetch/corb` directory,
because all of these tests verify behavior that is important to the CORB
algorithm.
