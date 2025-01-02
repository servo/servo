# Writing Tests

So you'd like to write new tests for WPT? Great! For starters, we recommend
reading [the introduction](../index) to learn how the tests are organized and
interpreted. You might already have an idea about what needs testing, but it's
okay if you don't know where to begin. In either case, [the guide on making a
testing plan](making-a-testing-plan) will help you decide what to write.

There's also a load of [general guidelines](general-guidelines) that apply to all tests.

## Test Types

There are various different ways of writing tests:

* [JavaScript tests (testharness.js)](testharness) are preferred for testing APIs and may be used
  for other features too. They are built with the testharness.js unit testing framework, and consist
  of assertions written in JavaScript. A high-level [testharness.js tutorial](testharness-tutorial)
  is available.

* Rendering tests should be used to verify that the browser graphically
  displays pages as expected. See the [rendering test guidelines](rendering)
  for tips on how to write great rendering tests. There are a few different
  ways to write rendering tests:

  * [Reftests](reftests) should be used to test rendering and layout. They
    consist of two or more pages with assertions as to whether they render
    identically or not. A high-level [reftest tutorial](reftest-tutorial) is available. A
    [print reftests](print-reftests) variant is available too.

  * [Visual tests](visual) should be used for checking rendering where there is
    a large number of conforming renderings such that reftests are impractical.
    They consist of a page that renders to final state at which point a
    screenshot can be taken and compared to an expected rendering for that user
    agent on that platform.

* [Crashtests](crashtest) tests are used to check that the browser is
  able to load a given document without crashing or experiencing other
  low-level issues (asserts, leaks, etc.). They pass if the load
  completes without error.

* [wdspec](wdspec) tests are written in Python using
  [pytest](https://docs.pytest.org/en/latest/) and test [the WebDriver browser
  automation protocol](https://w3c.github.io/webdriver/)

* [Manual tests](manual) are used as a last resort for anything that can't be
  tested using any of the above. They consist of a page that needs manual
  interaction or verification of the final result.

See [file names](file-names) for test types and features determined by the file names,
and [server features](server-features) for advanced testing features.

## Submitting Tests

Once you've written tests, please submit them using
the [typical GitHub Pull Request workflow](submission-process); please
make sure you run the [`lint` script](lint-tool) before opening a pull request!

## Table of Contents

```eval_rst
.. toctree::
   :maxdepth: 1

   general-guidelines
   making-a-testing-plan
   testharness
   testharness-tutorial
   rendering
   reftests
   reftest-tutorial
   print-reftests
   visual
   crashtest
   wdspec
   manual
   file-names
   server-features
   submission-process
   lint-tool
   ahem
   assumptions
   css-metadata
   css-user-styles
   h2tests
   testdriver
   testdriver-extension-tutorial
   tools
   test-templates
   github-intro
   ../tools/webtransport/README.md
```
