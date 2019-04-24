---
layout: page
title: Running Tests from the Web
---

Tests that have been merged on GitHub are mirrored at [http://w3c-test.org/][w3c-test].
[On properly-configured systems](from-local-system), local files may also be
served from the URL [http://web-platform.test](http://web-platform.test).

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

[w3c-test]: http://w3c-test.org
[from-local-system]: {{ site.baseurl }}{% link _running-tests/from-local-system.md %}
