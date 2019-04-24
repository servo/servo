---
layout: page
title: Lint Tool
order: 9
---
We have a lint tool for catching common mistakes in test files. You can run
it manually by running the `wpt lint` command from the root of your local
web-platform-tests working directory like this:

```
./wpt lint
```

The lint tool is also run automatically for every submitted pull request,
and reviewers will not merge branches with tests that have lint errors, so
you must either [fix all lint errors](#fixing-lint-errors), or you must
[whitelist test files](#updating-the-whitelist) to suppress the errors.

## Fixing lint errors

You must fix any errors the lint tool reports, unless an error is for
something essential to a certain test or that for some other
exceptional reason shouldn't prevent the test from being merged; in
those cases you can [whitelist test files](#updating-the-whitelist)
to suppress the errors. In all other cases, follow the instructions
below to fix all errors reported.

* **CONSOLE**: Test-file line has a `console.*(...)` call; **fix**: remove
  the `console.*(...)` call (and in some cases, consider adding an
  `assert_*` of some kind in place of it).

* **CR AT EOL**: Test-file line ends with CR (U+000D) character; **fix**:
  reformat file so each line just has LF (U+000A) line ending (standard,
  cross-platform "Unix" line endings instead of, e.g., DOS line endings).

* **EARLY-TESTHARNESSREPORT**: Test file has an instance of
  `<script src='/resources/testharnessreport.js'>` prior to
  `<script src='/resources/testharness.js'>`; **fix**: flip the order.

* **GENERATE_TESTS**: Test file line has a generate_tests call; **fix**: remove
  the call and call `test()` a number of times instead.

* **INDENT TABS**: Test-file line starts with one or more tab characters;
  **fix**: use spaces to replace any tab characters at beginning of lines.

* **INVALID-TIMEOUT**: Test file with `<meta name='timeout'...>` element
  that has a `content` attribute whose value is not `long`; **fix**:
  replace the value of the `content` attribute with `long`.

* **LATE-TIMEOUT**: Test file with `<meta name="timeout"...>` element after
  `<script src='/resources/testharnessreport.js'>` element ; **fix**: move
  the `<meta name="timeout"...>` element to precede the `script` element.

* **LAYOUTTESTS APIS**: Test file uses `eventSender`, `testRunner`, or
  `window.internals` which are LayoutTests-specific APIs used in WebKit/Blink.

* **MALFORMED-VARIANT**: Test file with a `<meta name='variant'...>`
  element whose `content` attribute has a malformed value; **fix**: ensure
  the value of the `content` attribute starts with `?` or `#` or is empty.

* **MISSING-LINK**: CSS test file is missing a link to a spec. **fix**: Ensure that there is a `<link rel="help" href="[url]">` for the spec.
  * Note: `MISSING-LINK` is designed to ensure that the CSS build tool can find the tests. Note that the CSS build system is primarily used by [test.csswg.org/](http://test.csswg.org/), which doesn't use `wptserve`, so `*.any.js` and similar tests won't work there; stick with the `.html` equivalent.

* **MISSING-TESTHARNESSREPORT**: Test file is missing an instance of
  `<script src='/resources/testharnessreport.js'>`; **fix**: ensure each
  test file contains `<script src='/resources/testharnessreport.js'>`.

* **MULTIPLE-TESTHARNESS**: Test file with multiple instances of
  `<script src='/resources/testharness.js'>`; **fix**: ensure each test
  has only one `<script src='/resources/testharness.js'>` instance.

* **MULTIPLE-TESTHARNESSREPORT**: Test file with multiple instances of
  `<script src='/resources/testharnessreport.js'>`; **fix**: ensure each test
  has only one `<script src='/resources/testharnessreport.js'>` instance.

* **MULTIPLE-TIMEOUT**: Test file with multiple `<meta name="timeout"...>`
  elements; **fix**: ensure each test file has only one instance of a
  `<meta name="timeout"...>` element.

* **PARSE-FAILED**: Test file failed parsing by manifest builder; **fix**:
  examine the file to find the causes of any parse errors, and fix them.

* **PATH LENGTH**: Test file's pathname has a total length greater than 150
  characters; **fix**: use shorter filename to rename the test file.

* **PRINT STATEMENT**: A server-side python support file contains a `print`
  statement; **fix**: remove the `print` statement or replace it with
  something else that achieves the intended effect (e.g., a logging call).

* **SET TIMEOUT**: Test-file line has `setTimeout(...)` call; **fix**:
  replace all `setTimeout(...)` calls with `step_timeout(...)` calls.

* **TRAILING WHITESPACE**: Test-file line has trailing whitespace; **fix**:
  remove trailing whitespace from all lines in the file.

* **VARIANT-MISSING**: Test file with a `<meta name='variant'...>` element
  that's missing a `content` attribute; **fix**: add a `content` attribute
  with an appropriate value to the `<meta name='variant'...>` element.

* **W3C-TEST.ORG**: Test-file line has the string `w3c-test.org`; **fix**:
  either replace the `w3c-test.org` string with the expression
  `{{host}}:{{ports[http][0]}}` or a generic hostname like `example.org`.

## Updating the whitelist

Normally you must [fix all lint errors](#fixing-lint-errors). But in the
unusual case of error reports for things essential to certain tests or that
for other exceptional reasons shouldn't prevent a merge of a test, you can
update and commit the `lint.whitelist` file in the web-platform-tests root
directory to suppress errors the lint tool would report for a test file.

To add a test file or directory to the whitelist, use the following format:

```
ERROR TYPE:file/name/pattern
```

For example, to whitelist the file `example/file.html` such that all
`TRAILING WHITESPACE` errors the lint tool would report for it are
suppressed, add the following line to the `lint.whitelist` file:

```
TRAILING WHITESPACE:example/file.html
```

To whitelist an entire directory rather than just one file, use the `*`
wildcard. For example, to whitelist the `example` directory such that all
`TRAILING WHITESPACE` errors the lint tool would report for any files in it
are suppressed, add the following line to the `lint.whitelist` file:

```
TRAILING WHITESPACE:example/*
```

Similarly, you can also
use
[shell-style wildcards](https://docs.python.org/2/library/fnmatch.html) to
express other filename patterns or directory-name patterns.

Finally, to whitelist just one line in a file, use the following format:

```
ERROR TYPE:file/name/pattern:line_number
```

For example, to whitelist just line 128 of the file `example/file.html`
such that any `TRAILING WHITESPACE` error the lint tool would report for
that line is suppressed, add the following to the `lint.whitelist` file:

```
TRAILING WHITESPACE:example/file.html:128
```
