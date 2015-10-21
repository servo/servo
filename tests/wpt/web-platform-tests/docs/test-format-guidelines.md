This page describes the available test types and the requirements for
authoring that apply to all test types. There is also a supplementary
[guide to writing good testcases](test-style-guidelines.html).

## Test Locations

Each top level directory in the repository corresponds to tests for a
single specification. For W3C specs, these directories are named after
the shortname of the spec (i.e. the name used for snapshot
publications under `/TR/`).

Within the specification-specific directory there are two common ways
of laying out tests. The first is a flat structure which is sometimes
adopted for very short specifications. The alternative is a nested
structure with each subdirectory corresponding to the id of a heading
in the specification. This layout provides some implicit metadata
about the part of a specification being tested according to its
location in the filesystem, and is preferred for larger
specifications.

When adding new tests to existing specifications, try to follow the
structure of existing tests.

Because of path length limitations on Windows, test paths must be less
that 150 characters relative to the test root directory (this gives
vendors just over 100 characters for their own paths when running in
automation).

## Choosing the Test Type

Tests should be written using the mechanism that is most conducive to
running in automation. In general the following order of preference holds:

* [idlharness.js](testharness-idlharness.html) tests - for testing
  anything in a WebIDL block.

* [testharness.js](testharness.html) tests - for any test that can be
  written using script alone.

* [Reftests][reftests] - for most tests of rendering.

* WebDriver tests - for testing the webdriver protocol itself or (in
  the future) for certain tests that require access to privileged APIs.

* [Manual tests][manual-tests] - as a last resort for anything that can't be tested
  using one of the above techniques.

Some scenarios demand certain test types. For example:

* Tests for layout will generally be reftests. In some cases it will
  not be possible to construct a reference and a test that will always
  render the same, in which case a manual test, accompanied by
  testharness tests that inspect the layout via the DOM must be
  written.

* Features that require human interaction for security reasons
  (e.g. to pick a file from the local filesystem) typically have to be
  manual tests.

## General Test Design Requirements

### Short

Tests should be as short as possible. For reftests in particular
scrollbars at 800&#xD7;600px window size must be avoided unless scrolling
behaviour is specifically being tested. For all tests extraneous
elements on the page should be avoided so it is clear what is part of
the test (for a typical testharness test, the only content on the page
will be rendered by the harness itself).

### Minimal

Tests should generally avoid depending on edge case behaviour of
features that they don't explicitly intend to test. For example,
except where testing parsing, tests should contain no
[parse errors][validator]. Of course tests which intentionally address
the interactions between multiple platform features are not only
acceptable but encouraged.

### Cross-platform

Tests should be as cross-platform as reasonably possible, working
across different devices, screen resolutions, paper sizes, etc.
Exceptions should document their assumptions.

### Self-Contained

Tests must not depend on external network resources, including
w3c-test.org. When these tests are run on CI systems they are
typically configured with access to external resources disabled, so
tests that try to access them will fail. Where tests want to use
multiple hosts this is possible thorough a known set of subdomains and
features of wptserve (see
["Tests Involving Multiple Origins"](#tests-involving-multiple-origins)).

## File Names

Generally file names should be somewhat descriptive of what is being
tested; very generic names like `001.html` are discouraged. A common
format, required by CSS tests, is described in
[CSS Naming Conventions](css-naming.html).

## File Formats

Tests must be HTML, XHTML or SVG files.

Note: For CSS tests, the test source will be parsed and
re-serialized. This re-serialization will cause minor changes to the
test file, notably: attribute values will always be quoted, whitespace
between attributes will be collapsed to a single space, duplicate
attributes will be removed, optional closing tags will be inserted,
and invalid markup will be normalized.  If these changes should make
the test inoperable, for example if the test is testing markup error
recovery, add the [flag][requirement-flags] `asis` to prevent
re-serialization. This flag will also prevent format conversions so it
may be necessary to provide alternate versions of the test in other
formats (XHTML, HTML, etc.)

## Character Encoding

Except when specifically testing encoding, tests must be encoded in
UTF-8, marked through the use of e.g. `<meta charset=utf-8>`, or in
pure ASCII.

## Support files

Various support files are available in in the `/common/` and `/media/`
directories (web-platform-tests) and `/support/` (CSS). Reusing
existing resources is encouraged where possible, as is adding
generally useful files to these common areas rather than to specific
testsuites.

For CSS tests the following standard images are available in the
support directory:

 * 1x1 color swatches
 * 15x15 color swatches
 * 15x15 bordered color swatches
 * assorted rulers and red/green grids
 * a cat
 * a 4-part picture

## Tools
Sometimes you may want to add a script to the repository that's meant
to be used from the command line, not from a browser (e.g., a script
for generating test files). If you want to ensure (e.g., or security
reasons) that such scripts won't be handled by the HTTP server, but
will instead only be usable from the command line, then place them
in either:

* the `tools` subdir at the root of the repository, or
* the `tools` subdir at the root of any top-level directory in the
  repo which contains the tests the script is meant to be used with

Any files in those `tools` directories won't be handled by the HTTP
server; instead the server will return a 404 if a user navigates to
the URL for a file within them.

If you want to add a script for use with a particular set of tests
but there isn't yet any `tools` subdir at the root of a top-level
directory in the repository containing those tests, you can create
a `tools` subdir at the root of that top-level directory and place
your scripts there.

For example, if you wanted to add a script for use with tests in the
`notifications` directory, create the `notifications/tools` subdir
and put your script there.

## Style Rules

A number of style rules should be applied to the test file. These are
not uniformly enforced throughout the existing tests, but will be for
new tests. Any of these rules may be broken if the test demands it:

 * No trailing whitespace

 * Use spaces rather than tabs for indentation

 * Use UNIX-style line endings (i.e. no CR characters at EOL).

## Advanced Testing Features

Certain test scenarios require more than just static HTML
generation. This is supported through the
[wptserve](http://github.com/w3c/wptserve) server. Several scenarios
in particular are common:

### Standalone workers tests

Tests that only require assertions in a dedicated worker scope can use
standalone workers tests. In this case, the test is a JavaScript file
with extension `.worker.js` that imports `testharness.js`. The test can
then use all the usual APIs, and can be run from the path to the
JavaScript file with the `.js` removed.

For example, one could write a test for the `Blob` constructor by
creating a `FileAPI/Blob-constructor.worker.js` as follows:

    importScripts("/resources/testharness.js");
    test(function () {
      var blob = new Blob();
      assert_equals(blob.size, 0);
      assert_equals(blob.type, "");
      assert_false(blob.isClosed);
    }, "The Blob constructor.");
    done();

This test could then be run from `FileAPI/Blob-constructor.worker`.

### Tests Involving Multiple Origins

In the test environment, five subdomains are available; `www`, `www1`,
`www2`, `天気の良い日` and `élève`. These must be used for
cross-origin tests. In addition two ports are available for http and
one for websockets. Tests must not hardcode the hostname of the server
that they expect to be running on or the port numbers, as these are
not guaranteed by the test environment. Instead tests can get this
information in one of two ways:

* From script, using the `location` API.

* By using a textual substitution feature of the server.

In order for the latter to work, a file must either have a name of the
form `{name}.sub.{ext}` e.g. `example-test.sub.html` or be referenced
through a URL containing `pipe=sub` in the query string
e.g. `example-test.html?pipe=sub`. The substitution syntax uses {% raw %} `{{
}}` {% endraw %} to delimit items for substitution. For example to substitute in
the host name on which the tests are running, one would write:

{% raw %}
    {{host}}
{% endraw %}

As well as the host, one can get full domains, including subdomains
using the `domains` dictionary. For example:

{% raw %}
    {{domains[www]}}
{% endraw %}

would be replaced by the fully qualified domain name of the `www`
subdomain. Ports are also available on a per-protocol basis e.g.

{% raw %}
    {{ports[ws][0]}}
{% endraw %}

is replaced with the first (and only) websockets port, whilst

{% raw %}
    {{ports[http][1]}}
{% endraw %}

is replaced with the second HTTP port.

The request URL itself can be used as part of the substitution using
the `location` dictionary, which has entries matching the
`window.location` API. For example

{% raw %}
    {{location[host]}}
{% endraw %}

is replaced by `hostname:port` for the current request.

### Tests Requiring Special Headers

For tests requiring that a certain HTTP header is set to some static
value, a file with the same path as the test file except for an an
additional `.headers` suffix may be created. For example for
`/example/test.html`, the headers file would be
`/example/test.html.headers`. This file consists of lines of the form

    header-name: header-value

For example

    Content-Type: text/html; charset=big5

To apply the same headers to all files in a directory use a
`__dir__.headers` file. This will only apply to the immediate
directory and not subdirectories.

Headers files may be used in combination with substitutions by naming
the file e.g. `test.html.sub.headers`.

### Tests Requiring Full Control Over The HTTP Response

For full control over the request and response the server provides the
ability to write `.asis` files; these are served as literal HTTP
responses. It also provides the ability to write python scripts that
have access to request data and can manipulate the content and timing
of the response. For details see the
[wptserve documentation](http://wptserve.readthedocs.org).

## CSS-Specific Requirements

Tests for CSS specs have some additional requirements that have to be
met in order to be included in an official specification testsuite.

* [Naming conventions](css-naming.html)

* [User style sheets](css-user-styles.html)

* [Metadata](css-metadata.html)

## Lint tool

We have a lint tool for catching common mistakes in test files. You can run
it manually by starting the `lint` executable from the root of your local
web-platform-tests working directory like this:

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

[lint-tool]: ./lint-tool.html
[reftests]: ./reftests.html
[manual-tests]: ./manual-test.html
[test-templates]: ./test-templates.html
[requirement-flags]: ./test-templates.html#requirement-flags
[testharness-documentation]: ./testharness-documentation.html
[validator]: http://validator.w3.org
