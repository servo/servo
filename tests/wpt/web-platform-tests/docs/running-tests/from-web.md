# Running Tests from the Web

Tests that have been merged on GitHub are mirrored at
[wpt.live](https://wpt.live) and [w3c-test.org](https://w3c-test.org).
[On properly-configured systems](from-local-system), local files may also be
served from the URL [http://web-platform.test](http://web-platform.test).

Not all tests can be executed in-browser, as some tests rely on automation
(e.g. via [testdriver.js](../writing-tests/testdriver)) that is not available
when running a browser in a normal user session.

## Web test runner

For running multiple tests inside a browser, there is a test runner
located at `/tools/runner/index.html`.

This allows all the tests, or those matching a specific prefix
(e.g. all tests under `/dom/`) to be run. For testharness.js tests,
the results will be automatically collected, while the runner
provides a simple UI for manually comparing reftest rendering and
running manual tests.

Note, however, it does not currently handle more complex reftests with
more than one reference involved.

Because it runs entirely in-browser, this runner cannot deal with
edge-cases like tests that cause the browser to crash or hang.
