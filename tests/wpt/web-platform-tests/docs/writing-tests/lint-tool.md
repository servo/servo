# Lint Tool

We have a lint tool for catching common mistakes in test files. You can run
it manually by running the `wpt lint` command from the root of your local
web-platform-tests working directory like this:

```
./wpt lint
```

The lint tool is also run automatically for every submitted pull request,
and reviewers will not merge branches with tests that have lint errors, so
you must either [fix all lint errors](#fixing-lint-errors), or you must
[add an exception](#updating-the-ignored-files) to suppress the errors.

## Fixing lint errors

You must fix any errors the lint tool reports, unless an error is for something
essential to a certain test or that for some other exceptional reason shouldn't
prevent the test from being merged; in those cases you can [add an
exception](#updating-the-ignored-files) to suppress the errors. In all other
cases, follow the instructions below to fix all errors reported.

<!--
  This listing is automatically generated from the linting tool's Python source
  code.
-->

```eval_rst
.. wpt-lint-rules:: tools.lint.rules
```

## Updating the ignored files

Normally you must [fix all lint errors](#fixing-lint-errors). But in the
unusual case of error reports for things essential to certain tests or that
for other exceptional reasons shouldn't prevent a merge of a test, you can
update and commit the `lint.ignore` file in the web-platform-tests root
directory to suppress errors the lint tool would report for a test file.

To add a test file or directory to the list, use the following format:

```
ERROR TYPE:file/name/pattern
```

For example, to ignore all `TRAILING WHITESPACE` errors in the file
`example/file.html`, add the following line to the `lint.ignore` file:

```
TRAILING WHITESPACE:example/file.html
```

To ignore errors for an entire directory rather than just one file, use the `*`
wildcard. For example, to ignore all `TRAILING WHITESPACE` errors in the
`example` directory, add the following line to the `lint.ignore` file:

```
TRAILING WHITESPACE:example/*
```

Similarly, you can also
use
[shell-style wildcards](https://docs.python.org/library/fnmatch.html) to
express other filename patterns or directory-name patterns.

Finally, to ignore just one line in a file, use the following format:

```
ERROR TYPE:file/name/pattern:line_number
```

For example, to ignore the `TRAILING WHITESPACE` error for just line 128 of the
file `example/file.html`, add the following to the `lint.ignore` file:

```
TRAILING WHITESPACE:example/file.html:128
```
