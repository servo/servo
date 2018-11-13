---
layout: page
title: Running Tests
---
In simple cases individual tests can be run by simply loading the page
in a browser window. For running larger groups of tests, or running
tests frequently, this is not a practical approach and several better
options exist.

## From the Command Line

The simplest way to run tests is to use the `wpt run` command from the
root of the repository. This will automatically load the tests in the
chosen browser, and extract the test results. For example to run the
`dom/historical.html` tests in a local copy of Chrome:

    ./wpt run chrome dom/historical.html

Or to run in a specified copy of Firefox:

    ./wpt run --binary ~/local/firefox/firefox firefox dom/historical.html

`./wpt run --help` lists the supported products.

For details on the supported products and a large number of other options for
customising the test run, see `./wpt run --help`.

Additional browser-specific documentation:

  * [Chrome][chrome]

  * [Chrome for Android][chrome android]

  * [Safari][safari]

## From Inside a Browser
Tests that have been merged on GitHub are mirrored at [http://w3c-test.org/][w3c-test].

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

## Writing Your Own Runner

Most test runners have two stages: finding all tests, followed by
executing them (or a subset thereof).

To find all tests in the repository, it is **strongly** recommended to
use the included `wpt manifest` tool: the required behaviors are more
complex than what are documented (especially when it comes to
precedence of the various possibilities and some undocumented legacy
ways to define test types), and hence its behavior should be
considered the canonical definition of how to enumerate tests and find
their type in the repository.

For test execution, please read the documentation for the various test types
very carefully and then check your understanding on
the [mailing list][public-test-infra] or [IRC][] ([webclient][web irc], join
channel `#testing`). It's possible edge-case behavior isn't properly
documented!


[chrome]: {{ site.baseurl }}{% link _running-tests/chrome.md %}
[chrome android]: {{ site.baseurl }}{% link _running-tests/chrome_android.md %}
[safari]: {{ site.baseurl }}{% link _running-tests/safari.md %}
[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[IRC]: irc://irc.w3.org:6667/testing
[web irc]: http://irc.w3.org
[w3c-test]: http://w3c-test.org
