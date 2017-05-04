---
layout: page
title: General Test Guidelines
order: 2
---

### File Paths and Names

When choosing where in the directory structure to put any new tests,
try to follow the structure of existing tests for that specification;
if there are no existing tests, it is generally recommended to create
subdirectories for each section.

Due to path length limitations on Windows, test paths must be less
that 150 characters relative to the test root directory (this gives
vendors just over 100 characters for their own paths when running in
automation).

File names should generally be somewhat descriptive of what is being
tested; very generic names like `001.html` are discouraged. A common
format is `test-topic-001.html`, where `test-topic` is a short
identifier that describes the test. It should avoid conjunctions,
articles, and prepositions as it should be as concise as possible. The
integer that follows is normally just increased incrementally, and
padded to three digits. (If you'd end up with more than 999 tests,
your `test-topic` is probably too broad!)

In csswg-test, the file names should be unique within the repository,
regardless of where they are in the directory structure.


#### Support Files

Various support files are available in in the `/common/` and `/media/`
directories (web-platform-tests) and `/support/` (csswg-test). Reusing
existing resources is encouraged where possible, as is adding
generally useful files to these common areas rather than to specific
testsuites.


#### Tools

Sometimes you may want to add a script to the repository that's meant
to be used from the command line, not from a browser (e.g., a script
for generating test files). If you want to ensure (e.g., for security
reasons) that such scripts will only be usable from the command line
but won't be handled by the HTTP server then place them in a `tools`
subdirectory at the appropriate level—the server will then return a
404 if they are requested.

For example, if you wanted to add a script for use with tests in the
`notifications` directory, create the `notifications/tools`
subdirectory and put your script there.


### File Formats

Tests must be HTML or XML (inc. XHTML and SVG) files; some
testharness.js tests
are [auto-generated from JS files][server features].


### Character Encoding

Except when specifically testing encoding, files must be encoded in
UTF-8. In file formats where UTF-8 is not the default encoding, they
must contain metadata to mark them as such (e.g., `<meta
charset=utf-8>` in HTML files) or be pure ASCII.


### Server Side Support

The custom web server
supports [a variety of features][server features] useful for testing
browsers, including (but not limited to!) support for writing out
appropriate domains and custom (per-file and per-directory) HTTP
headers.


### Be Short

Tests should be as short as possible. For reftests in particular
scrollbars at 800&#xD7;600px window size must be avoided unless scrolling
behavior is specifically being tested. For all tests extraneous
elements on the page should be avoided so it is clear what is part of
the test (for a typical testharness test, the only content on the page
will be rendered by the harness itself).


### Be Minimal

Tests should generally avoid depending on edge case behavior of
features that they don't explicitly intend on testing. For example,
except where testing parsing, tests should contain
no [parse errors](https://validator.nu).

This is not, however, to discourage testing of edge cases or
interactions between multiple features; such tests are an essential
part of ensuring interoperability of the web platform.


### Be Cross-Platform

Tests should be as cross-platform as reasonably possible, working
across different devices, screen resolutions, paper sizes, etc. The
assumptions that can be relied on are documented [here][assumptions];
tests that rely on anything else should be manual tests that document
their assumptions.

Aside from the [Ahem font][ahem], fonts cannot be relied on to be
either installed or to have specific metrics. As such, in most cases
when a known font is needed Ahem should be used. In other cases,
`@font-face` should be used.


### Be Self-Contained

Tests must not depend on external network resources, including
w3c-test.org. When these tests are run on CI systems they are
typically configured with access to external resources disabled, so
tests that try to access them will fail. Where tests want to use
multiple hosts this is possible through a known set of subdomains and
the
[text substitution features of wptserve]({{ site.baseurl }}{% link _writing-tests/server-features.md %}#tests-involving-multiple-origins).


### Be Self-Describing

Tests should make it obvious when they pass and when they fail. It
shouldn't be necessary to consult the specification to figure out
whether a test has passed of failed.


### Style Rules

A number of style rules should be applied to the test file. These are
not uniformly enforced throughout the existing tests, but will be for
new tests. Any of these rules may be broken if the test demands it:

 * No trailing whitespace

 * Use spaces rather than tabs for indentation

 * Use UNIX-style line endings (i.e. no CR characters at EOL).

We have a lint tool for catching these and other common mistakes. You
can run it manually by starting the `lint` executable from the root of
your local web-platform-tests working directory like this:

```
./lint
```

The lint tool is also run automatically for every submitted pull request,
and reviewers will not merge branches with tests that have lint errors, so
you must fix any errors the lint tool reports. For details on doing that,
see the [lint-tool documentation][lint-tool].

But in the unusual case of error reports for things essential to a certain
test or that for other exceptional reasons shouldn't prevent a merge of a
test, update and commit the `lint.whitelist` file in the web-platform-tests
root directory to suppress the error reports. For details on doing that,
see the [lint-tool documentation][lint-tool].


## CSS-Specific Requirements

In order to be included in an official specification testsuite, tests
for CSS have some additional requirements for:

* [Metadata][css-metadata], and
* [User style sheets][css-user-styles].


[server features]: {{ site.baseurl }}{% link _writing-tests/server-features.md %}
[assumptions]: {{ site.baseurl }}{% link _writing-tests/assumptions.md %}
[ahem]: {{ site.baseurl }}{% link _writing-tests/ahem.md %}
[lint-tool]: {{ site.baseurl }}{% link _writing-tests/lint-tool.md %}
[css-metadata]: {{ site.baseurl }}{% link _writing-tests/css-metadata.md %}
[css-user-styles]: {{ site.baseurl }}{% link _writing-tests/css-user-styles.md %}
