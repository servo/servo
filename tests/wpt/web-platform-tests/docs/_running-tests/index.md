---
layout: page
title: Running Tests
---
In simple cases individual tests can be run by simply loading the page
in a browser window. For running larger groups of tests, or running
tests frequently, this is not a practical approach and several better
options exist.

## From Inside a Browser

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

## By Automating the Browser

For automated test running designed to be robust enough to use in a CI
environment, the [wptrunner](https://github.com/w3c/wptrunner) test runner
can be used. This is a test runner written in Python and designed to
control the browser from the outside using some remote control
protocol such as WebDriver. This allows it to handle cases such as the
browser crashing that cannot be handled by an in-browser harness. It
also has the ability to automatically run both testharness-based tests
and reftests.

Full instructions for using wptrunner are provided in its own
[documentation](https://wptrunner.readthedocs.org).

## Writing Your Own Runner

Most test runners have two stages: finding all tests, followed by
executing them (or a subset thereof).

To find all tests in the repository, it is **strongly** recommended to
use the included `manifest` tool: the required behaviors are more
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


[public-test-infra]: https://lists.w3.org/Archives/Public/public-test-infra/
[IRC]: irc://irc.w3.org:6667/testing
[web irc]: http://irc.w3.org
