In simple cases individual tests can be run by simply loading the page
in a browser window. For running larger groups of tests, or running
tests frequently, this is not a practical approach, and several better
options exist.

## From Inside a Browser

For running multiple tests inside a browser, there is the test runner,
located at

    /tools/runner/index.html

This allows all the tests, or those matching a specific prefix
(e.g. all tests under `/dom/`) to be run. For testharness.js tests,
the results will be automatically collected, whilst the runner
provides a simple UI for manually comparing reftest rendering and
running manual tests.

Because it runs entirely in-browser, this runner cannot deal with
edge-cases like tests that cause the browser to crash or hang.

## By Automating the Browser

For automated test running designed to be robust enough to use in a CI
environment, the [wptrunner](http://github.com/w3c/wptrunner) test runner
can be used. This is a test runner written in Python and designed to
control the browser from the outside using some remote control
protocol such as WebDriver. This allows it to handle cases such as the
browser crashing that cannot be handled by an in-browser harness. It
also has the ability to automatically run both testharness-based tests
and reftests.

Full instructions for using wptrunner are provided in its own
[documentation](http://wptrunner.readthedocs.org).
