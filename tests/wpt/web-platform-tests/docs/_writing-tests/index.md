---
layout: page
title: Writing Tests
order: -1
---

If you haven't already, it's strongly recommended to read
the [introduction][] first, as it introduces the various test types.

There's also a load of [general guidelines][] that apply to all tests.

## Test Type

There are four main test types:

* [Reftests][] should be used to test rendering and layout. They
  consist of two or more pages with assertions as to whether they
  render identically or not.

* [testharness.js][] tests should be used (where
  possible!) for testing everything else. They are built with the
  testharness.js unit testing framework, and consist of assertions
  written in JavaScript.

* [Visual tests][visual] should be used for checking rendering where
  there is a large number of conforming renderings such that reftests
  are impractical. They consist of a page that renders to final state
  at which point a screenshot can be taken and compared to an expected
  rendering for that user agent on that platform.

* [Manual tests][manual] are used as a last resort for anything
  that can't be tested using any of the above. They consist of a page
  that needs manual interaction or verification of the final result.

In general, there is a strong preference towards the first two test
types (as they can be easily run without human interaction), so they
should be used in preference to the others even if it results in a
somewhat cumbersome test; there is a far weaker preference between the
first two, and it is at times advisable to use testharness.js tests
for things which would typically be tested using reftests but for
which it would be overly cumbersome.

In addition to the four main test types, there are also WebDriver
tests, which are used exclusively for testing the WebDriver protocol
itself. There is currently no documentation about these tests,
however.

## Submitting Tests

Once you've written tests, please submit them using
the [typical GitHub Pull Request workflow][submission-process]; please
make sure you run the [`lint` script][lint-tool] before opening a pull request!

[introduction]: {{ site.baseurl }}{% link introduction.md %}
[general guidelines]: {{ site.baseurl }}{% link _writing-tests/general-guidelines.md %}
[reftests]: {{ site.baseurl }}{% link _writing-tests/reftests.md %}
[testharness.js]: {{ site.baseurl }}{% link _writing-tests/testharness.md %}
[visual]: {{ site.baseurl }}{% link _writing-tests/visual.md %}
[manual]: {{ site.baseurl }}{% link _writing-tests/manual.md %}
[submission-process]: {{ site.baseurl }}{% link _writing-tests/submission-process.md %}
[lint-tool]: {{ site.baseurl }}{% link _writing-tests/lint-tool.md %}
