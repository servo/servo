wptrunner: A web-platform-tests harness
=======================================

wptrunner is a harness for running the W3C `web-platform-tests testsuite`_.

.. contents::

Installation
~~~~~~~~~~~~

wptrunner is expected to be installed into a virtualenv using pip. For
development, it can be installed using the `-e` option::

  pip install -e ./

Running the Tests
~~~~~~~~~~~~~~~~~

After installation, the command ``wptrunner`` should be available to run
the tests.

The ``wptrunner`` command  takes multiple options, of which the
following are most significant:

``--product`` (defaults to `firefox`)
  The product to test against: `chrome`, `firefox`, or `servo`.

``--binary`` (required if product is `firefox` or `servo`)
  The path to a binary file for the product (browser) to test against.

``--webdriver-binary`` (required if product is `chrome`)
  The path to a `driver` binary; e.g., a `chromedriver` binary.

``--certutil-binary`` (required if product is `firefox` [#]_)
  The path to a `certutil` binary (for tests that must be run over https).

``--metadata`` (required)
  The path to a directory containing test metadata. [#]_

``--tests`` (required)
  The path to a directory containing a web-platform-tests checkout.

``--prefs-root`` (required only when testing a Firefox binary)
  The path to a directory containing Firefox test-harness preferences. [#]_

``--config`` (should default to `wptrunner.default.ini`)
  The path to the config (ini) file.

.. [#] The ``--certutil-binary`` option is required when the product is
   ``firefox`` unless ``--ssl-type=none`` is specified.

.. [#] The ``--metadata`` path is to a directory that contains:

  * a ``MANIFEST.json`` file (instructions on generating this file are
    available in the `detailed documentation
    <http://wptrunner.readthedocs.org/en/latest/usage.html#installing-wptrunner>`_);
    and
  * (optionally) any expectation files (see below)

.. [#] Example ``--prefs-root`` value: ``~/mozilla-central/testing/profiles``.

There are also a variety of other options available; use ``--help`` to
list them.

-------------------------------
Example: How to start wptrunner
-------------------------------

To test a Firefox Nightly build in an OS X environment, you might start
wptrunner using something similar to the following example::

  wptrunner --metadata=~/web-platform-tests/ --tests=~/web-platform-tests/ \
    --binary=~/mozilla-central/obj-x86_64-apple-darwin14.3.0/dist/Nightly.app/Contents/MacOS/firefox \
    --certutil-binary=~/mozilla-central/obj-x86_64-apple-darwin14.3.0/security/nss/cmd/certutil/certutil \
    --prefs-root=~/mozilla-central/testing/profiles

And to test a Chromium build in an OS X environment, you might start
wptrunner using something similar to the following example::

  wptrunner --metadata=~/web-platform-tests/ --tests=~/web-platform-tests/ \
    --binary=~/chromium/src/out/Release/Chromium.app/Contents/MacOS/Chromium \
    --webdriver-binary=/usr/local/bin/chromedriver --product=chrome

-------------------------------------
Example: How to run a subset of tests
-------------------------------------

To restrict a test run just to tests in a particular web-platform-tests
subdirectory, specify the directory name in the positional arguments after
the options; for example, run just the tests in the `dom` subdirectory::

  wptrunner --metadata=~/web-platform-tests/ --tests=~/web-platform-tests/ \
    --binary=/path/to/firefox --certutil-binary=/path/to/certutil \
    --prefs-root=/path/to/testing/profiles \
    dom

Output
~~~~~~

By default wptrunner just dumps its entire output as raw JSON messages
to stdout. This is convenient for piping into other tools, but not ideal
for humans reading the output.

As an alternative, you can use the ``--log-mach`` option, which provides
output in a reasonable format for humans. The option requires a value:
either the path for a file to write the `mach`-formatted output to, or
"`-`" (a hyphen) to write the `mach`-formatted output to stdout.

When using ``--log-mach``, output of the full raw JSON log is still
available, from the ``--log-raw`` option. So to output the full raw JSON
log to a file and a human-readable summary to stdout, you might start
wptrunner using something similar to the following example::

  wptrunner --metadata=~/web-platform-tests/ --tests=~/web-platform-tests/ \
    --binary=/path/to/firefox --certutil-binary=/path/to/certutil \
    --prefs-root=/path/to/testing/profiles \
    --log-raw=output.log --log-mach=-

Expectation Data
~~~~~~~~~~~~~~~~

wptrunner is designed to be used in an environment where it is not
just necessary to know which tests passed, but to compare the results
between runs. For this reason it is possible to store the results of a
previous run in a set of ini-like "expectation files". This format is
documented below. To generate the expectation files use `wptrunner` with
the `--log-raw=/path/to/log/file` option. This can then be used as
input to the `wptupdate` tool.

Expectation File Format
~~~~~~~~~~~~~~~~~~~~~~~

Metadata about tests, notably including their expected results, is
stored in a modified ini-like format that is designed to be human
editable, but also to be machine updatable.

Each test file that requires metadata to be specified (because it has
a non-default expectation or because it is disabled, for example) has
a corresponding expectation file in the `metadata` directory. For
example a test file `html/test1.html` containing a failing test would
have an expectation file called `html/test1.html.ini` in the
`metadata` directory.

An example of an expectation file is::

  example_default_key: example_value

  [filename.html]
    [subtest1]
      expected: FAIL

    [subtest2]
      expected:
        if platform == 'win': TIMEOUT
        if platform == 'osx': ERROR
        FAIL

  [filename.html?query=something]
    disabled: bug12345

The file consists of two elements, key-value pairs and
sections.

Sections are delimited by headings enclosed in square brackets. Any
closing square bracket in the heading itself my be escaped with a
backslash. Each section may then contain any number of key-value pairs
followed by any number of subsections. So that it is clear which data
belongs to each section without the use of end-section markers, the
data for each section (i.e. the key-value pairs and subsections) must
be indented using spaces. Indentation need only be consistent, but
using two spaces per level is recommended.

In a test expectation file, each resource provided by the file has a
single section, with the section heading being the part after the last
`/` in the test url. Tests that have subsections may have subsections
for those subtests in which the heading is the name of the subtest.

Simple key-value pairs are of the form::

  key: value

Note that unlike ini files, only `:` is a valid separator; `=` will
not work as expected. Key-value pairs may also have conditional
values of the form::

  key:
    if condition1: value1
    if condition2: value2
    default

In this case each conditional is evaluated in turn and the value is
that on the right hand side of the first matching conditional. In the
case that no condition matches, the unconditional default is used. If
no condition matches and no default is provided it is equivalent to
the key not being present. Conditionals use a simple python-like expression
language e.g.::

  if debug and (platform == "linux" or platform == "osx"): FAIL

For test expectations the available variables are those in the
`run_info` which for desktop are `version`, `os`, `bits`, `processor`,
`debug` and `product`.

Key-value pairs specified at the top level of the file before any
sections are special as they provide defaults for the rest of the file
e.g.::

  key1: value1

  [section 1]
    key2: value2

  [section 2]
    key1: value3

In this case, inside section 1, `key1` would have the value `value1`
and `key2` the value `value2` whereas in section 2 `key1` would have
the value `value3` and `key2` would be undefined.

The web-platform-test harness knows about several keys:

`expected`
  Must evaluate to a possible test status indicating the expected
  result of the test. The implicit default is PASS or OK when the
  field isn't present.

`disabled`
  Any value indicates that the test is disabled.

`reftype`
  The type of comparison for reftests; either `==` or `!=`.

`refurl`
  The reference url for reftests.

.. _`web-platform-tests testsuite`: https://github.com/web-platform-tests/wpt
