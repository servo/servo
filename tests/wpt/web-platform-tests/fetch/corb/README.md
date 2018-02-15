# Tests related to Cross-Origin Resource Blocking (CORB).

This directory contains tests related to the
[Cross-Origin Resource Blocking (CORB)](https://chromium.googlesource.com/chromium/src/+/master/content/browser/loader/cross_origin_read_blocking_explainer.md)
algorithm.

Note that CORB is currently in very early stages of standardization path.  At
the same time, some tests in this directory (e.g.
`css-with-json-parser-breaker`) cover behavior spec-ed outside of CORB (making
sure that CORB doesn't change the existing web behavior) and therefore are
valuable independently from CORB's standardization efforts.

Tests that cover behavior that is changed by CORB have to be marked as
[tentative](http://web-platform-tests.org/writing-tests/file-names.html)
(using `.tentative` substring in their filename) until CORB
is included in the official
[Fetch spec](https://fetch.spec.whatwg.org/).  Such tests may fail unless
CORB is enabled.  In practice this means that:
* Such tests will fail in default Chromium and have to be listed
  in `third_party/WebKit/LayoutTests/TestExpectations` and associated
  with https://crbug.com/802835.
* Such tests will pass in Chromium when either
  1) CORB is explicitly, manually enabled by passing extra cmdline flags to
     `run-webkit-tests`:
     `--additional-driver-flag=--enable-features=CrossSiteDocumentBlockingAlways` and
     `--additional-expectations=third_party/WebKit/LayoutTests/FlagExpectations/site-per-process`.
  2) CORB is implicitly enabled via Site Isolation (e.g. in
     `site_per_process_webkit_layout_tests` step on the test bots).  The
     expectations that the tests pass in this mode is controlled by the
     `third_party/WebKit/LayoutTests/FlagExpectations/site-per-process` file.
* Such tests may fail in other browsers.

The tests in this directory interact with various, random features,
but the tests have been grouped together into the `fetch/corb` directory,
because all of these tests verify behavior that is important to the CORB
algorithm.
