## Introduction

We have a lint tool for catching common mistakes in test files. The tool comes from
[W3C/wpt-tools](https://github.com/w3c/wpt-tools/) with modification for catching
common mistakes in submitted pull request, all WebGL/sdk/tests and specified folder.

The copyright of this tool is belong to W3C and/or the author listed in the test
file. The tool is dual-licensed under the
[W3C Test Suite License](http://www.w3.org/Consortium/Legal/2008/04-testsuite-license)
and [BSD 3-clause License](http://www.w3.org/Consortium/Legal/2008/03-bsd-license),
which are introduced in
[W3C's test suite licensing policy](http://www.w3.org/Consortium/Legal/2008/04-testsuite-copyright).

Now the tool can check html, htm, xhtml, xhtm, js, frag and vert files.
- You can run it manually by starting the `lint.py` executable from the root of your local
WebGL/sdk/tests working directory like this:

```
./py/lint/lint.py
```

You can use the lint tool to check submitted pull request and fix the errors reported by the tool.
Reviewers will not merge branches with tests that have lint errors, so you must either
[fix all lint errors](#fixing-lint-errors) or update
[white-list test files] (#updating-the-whitelist) to suppress the errors.

## Usage of lint tool

1. Check other repos, specify the repo name with `-r`, default
is WebGL/sdk/tests:</br>
<code>
./py/lint/lint.py -r demo-express
</code>
1. Check submitted pull request:</br>
<code>
./py/lint/lint.py -p
</code>
1. Check specified folder, the specified folder must be relative path of
WebGL/sdk/tests:</br>
<code>
./py/lint/lint.py -d conformance/attribs
</code>

## Fixing lint errors

You must fix any errors the lint tool reports, unless an error is for
something essential to a certain test or that for some other exceptional
reason shouldn't prevent the test from being merged. In those cases you can
update [white-list test files](#updating-the-whiteslist) to suppress the errors.
Otherwise, use the details in this section to fix all errors reported.

* **CR AT EOL**: Test-file line ends with CR (U+000D) character; **fix**:
  reformat file so each line just has LF (U+000A) line ending (standard,
  cross-platform "Unix" line endings instead of, e.g., DOS line endings).

* **INDENT TABS**: Test-file line starts with one or more tab characters;
  **fix**: use spaces to replace any tab characters at beginning of lines.

* **TRAILING WHITESPACE**: Test-file line has trailing whitespace; **fix**:
  remove trailing whitespace from all lines in the file.

* **UNNECESSARY EXECUTABLE PERMISSION**: Test file contains unnecessary executable permission; **fix**:
  remove unnecessary executable permission of the file.

* **FILENAME WHITESPACE**: Test file name contains white space; **fix**:
  remove white space from test file name.

## Updating the whitelist

Normally you must [fix all lint errors](#fixing-lint-errors). But in the
unusual case of error reports for things essential to certain tests or that
for other exceptional reasons shouldn't prevent a merge of a test, you can
update and commit the `lint.whitelist` file in the WebGL/sdk/tests/py/lint/
directory to suppress errors the lint tool would report for a test file.

To add a test file or directory to the whitelist, use the following format:

```
ERROR TYPE:file/name/pattern
```

For example, to whitelist the file `example/file.html` such that all
`TRAILING WHITESPACE` errors the lint tool would report for it are
suppressed, add the following line to the `lint.whitelist` file.

```
TRAILING WHITESPACE:example/file.html
```

To whitelist an entire directory rather than just one file, use the `*`
wildcard. For example, to whitelist the `example` directory such that all
`TRAILING WHITESPACE` errors the lint tool would report for any files in it
are suppressed, add the following line to the `lint.whitelist` file.

```
TRAILING WHITESPACE:example/*
```

If needed, you can also use the `*` wildcard to express other filename
patterns or directory-name patterns (just as you would when, e.g.,
executing shell commands from the command line).

Finally, to whitelist just one line in a file, use the following format:

```
ERROR TYPE:file/name/pattern:line_number
```

For example, to whitelist just line 128 of the file `example/file.html`
such that any `TRAILING WHITESPACE` error the lint tool would report for
that line is suppressed, add the following to the `lint.whitelist` file.

```
TRAILING WHITESPACE:example/file.html:128
```
