Test Metadata
=============

Directory Layout
----------------

Metadata files must be stored under the ``metadata`` directory passed
to the test runner. The directory layout follows that of
web-platform-tests with each test source path having a corresponding
metadata file. Because the metadata path is based on the source file
path, files that generate multiple URLs e.g. tests with multiple
variants, or multi-global tests generated from an ``any.js`` input
file, share the same metadata file for all their corresponding
tests. The metadata path under the ``metadata`` directory is the same
as the source path under the ``tests`` directory, with an additional
``.ini`` suffix.

For example a test with URL::

  /spec/section/file.html?query=param

generated from a source file with path::

  <tests root>/spec/section.file.html

would have a metadata file ::

  <metadata root>/spec/section/file.html.ini

As an optimisation, files which produce only default results
(i.e. ``PASS`` or ``OK``), and which don't have any other associated
metadata, don't require a corresponding metadata file.

Directory Metadata
~~~~~~~~~~~~~~~~~~

In addition to per-test metadata, default metadata can be applied to
all the tests in a given source location, using a ``__dir__.ini``
metadata file. For example to apply metadata to all tests under
``<tests root>/spec/`` add the metadata in ``<tests
root>/spec/__dir__.ini``.

Metadata Format
---------------
The format of the metadata files is based on the ini format. Files are
divided into sections, each (apart from the root section) having a
heading enclosed in square braces. Within each section are key-value
pairs. There are several notable differences from standard .ini files,
however:

 * Sections may be hierarchically nested, with significant whitespace
   indicating nesting depth.

 * Only ``:`` is valid as a key/value separator

A simple example of a metadata file is::

  root_key: root_value

  [section]
    section_key: section_value

    [subsection]
       subsection_key: subsection_value

  [another_section]
    another_key: [list, value]

Conditional Values
~~~~~~~~~~~~~~~~~~

In order to support values that depend on some external data, the
right hand side of a key/value pair can take a set of conditionals
rather than a plain value. These values are placed on a new line
following the key, with significant indentation. Conditional values
are prefixed with ``if`` and terminated with a colon, for example::

  key:
    if cond1: value1
    if cond2: value2
    value3

In this example, the value associated with ``key`` is determined by
first evaluating ``cond1`` against external data. If that is true,
``key`` is assigned the value ``value1``, otherwise ``cond2`` is
evaluated in the same way. If both ``cond1`` and ``cond2`` are false,
the unconditional ``value3`` is used.

Conditions themselves use a Python-like expression syntax. Operands
can either be variables, corresponding to data passed in, numbers
(integer or floating point; exponential notation is not supported) or
quote-delimited strings. Equality is tested using ``==`` and
inequality by ``!=``. The operators ``and``, ``or`` and ``not`` are
used in the expected way. Parentheses can also be used for
grouping. For example::

  key:
    if (a == 2 or a == 3) and b == "abc": value1
    if a == 1 or b != "abc": value2
    value3

Here ``a`` and ``b`` are variables, the value of which will be
supplied when the metadata is used.

Web-Platform-Tests Metadata
---------------------------

When used for expectation data, metadata files have the following format:

 * A section per test URL provided by the corresponding source file,
   with the section heading being the part of the test URL following
   the last ``/`` in the path (this allows multiple tests in a single
   metadata file with the same path part of the URL, but different
   query parts). This may be omitted if there's no non-default
   metadata for the test.

 * A subsection per subtest, with the heading being the title of the
   subtest. This may be omitted if there's no non-default metadata for
   the subtest.

 * The following known keys:

   :expected:
      The expectation value or values of each (sub)test. In
      the case this value is a list, the first value represents the
      typical expected test outcome, and subsequent values indicate
      known intermittent outcomes e.g. ``expected: [PASS, ERROR]``
      would indicate a test that usually passes but has a known-flaky
      ``ERROR`` outcome.

   :disabled:
     Any values apart from the special value ``@False``
     indicates that the (sub)test is disabled and should either not be
     run (for tests) or that its results should be ignored (subtests).

   :restart-after:
     Any value apart from the special value ``@False``
     indicates that the runner should restart the browser after running
     this test (e.g. to clear out unwanted state).

   :fuzzy:
     Used for reftests. This is interpreted as a list containing
     entries like ``<meta name=fuzzy>`` content value, which consists of
     an optional reference identifier followed by a colon, then a range
     indicating the maximum permitted pixel difference per channel, then
     semicolon, then a range indicating the maximum permitted total
     number of differing pixels. The reference identifier is either a
     single relative URL, resolved against the base test URL, in which
     case the fuzziness applies to any comparison with that URL, or
     takes the form lhs URL, comparison, rhs URL, in which case the
     fuzziness only applies for any comparison involving that specific
     pair of URLs. Some illustrative examples are given below.

   :implementation-status:
     One of the values ``implementing``,
     ``not-implementing`` or ``backlog``. This is used in conjunction
     with the ``--skip-implementation-status`` command line argument to
     ``wptrunner`` to ignore certain features where running the test is
     low value.

   :tags:
     A list of labels associated with a given test that can be
     used in conjunction with the ``--tag`` command line argument to
     ``wptrunner`` for test selection.

   In addition there are extra arguments which are currently tied to
   specific implementations. For example Gecko-based browsers support
   ``min-asserts``, ``max-asserts``, ``prefs``, ``lsan-disabled``,
   ``lsan-allowed``, ``lsan-max-stack-depth``, ``leak-allowed``, and
   ``leak-threshold`` properties.

 * Variables taken from the ``RunInfo`` data which describe the
   configuration of the test run. Common properties include:

   :product: A string giving the name of the browser under test
   :browser_channel: A string giving the release channel of the browser under test
   :debug: A Boolean indicating whether the build is a debug build
   :os: A string  the operating system
   :version: A string indicating the particular version of that operating system
   :processor: A string indicating the processor architecture.

   This information is typically provided by :py:mod:`mozinfo`, but
   different environments may add additional information, and not all
   the properties above are guaranteed to be present in all
   environments. The definitive list of available properties for a
   specific run may be determined by looking at the ``run_info`` key
   in the ``wptreport.json`` output for the run.

 * Top level keys are taken as defaults for the whole file. So, for
   example, a top level key with ``expected: FAIL`` would indicate
   that all tests and subtests in the file are expected to fail,
   unless they have an ``expected`` key of their own.

An simple example metadata file might look like::

  [test.html?variant=basic]
    type: testharness

    [Test something unsupported]
       expected: FAIL

    [Test with intermittent statuses]
       expected: [PASS, TIMEOUT]

  [test.html?variant=broken]
    expected: ERROR

  [test.html?variant=unstable]
    disabled: http://test.bugs.example.org/bugs/12345

A more complex metadata file with conditional properties might be::

  [canvas_test.html]
    expected:
      if os == "mac": FAIL
      if os == "windows" and version == "XP": FAIL
      PASS

Note that ``PASS`` in the above works, but is unnecessary since it's
the default expected result.

A metadata file with fuzzy reftest values might be::

  [reftest.html]
    fuzzy: [10;200, ref1.html:20;200-300, subtest1.html==ref2.html:10-15;20]

In this case the default fuzziness for any comparison would be to
require a maximum difference per channel of less than or equal to 10
and less than or equal to 200 total pixels different. For any
comparison involving ref1.html on the right hand side, the limits
would instead be a difference per channel not more than 20 and a total
difference count of not less than 200 and not more than 300. For the
specific comparison ``subtest1.html == ref2.html`` (both resolved against
the test URL) these limits would instead be 10 to 15 and 0 to 20,
respectively.

Generating Expectation Files
----------------------------

wpt provides the tool ``wpt update-expectations`` command to generate
expectation files from the results of a set of test runs. The basic
syntax for this is::

  ./wpt update-expectations [options] [logfile]...

Each ``logfile`` is a wptreport log file from a previous run. These
can be generated from wptrunner using the ``--log-wptreport`` option
e.g. ``--log-wptreport=wptreport.json``.

``update-expectations`` takes several options:

--full  Overwrite all the expectation data for any tests that have a
        result in the passed log files, not just data for the same run
        configuration.

--disable-intermittent  When updating test results, disable tests that
                        have inconsistent results across many
                        runs. This can precede a message providing a
                        reason why that test is disable. If no message
                        is provided, ``unstable`` is the default text.

--update-intermittent  When this option is used, the ``expected`` key
                       stores expected intermittent statuses in
                       addition to the primary expected status. If
                       there is more than one status, it appears as a
                       list. The default behaviour of this option is to
                       retain any existing intermittent statuses in the
                       list unless ``--remove-intermittent`` is
                       specified.

--remove-intermittent  This option is used in conjunction with
                       ``--update-intermittent``.  When the
                       ``expected`` statuses are updated, any obsolete
                       intermittent statuses that did not occur in the
                       specified log files are removed from the list.

Property Configuration
~~~~~~~~~~~~~~~~~~~~~~

In cases where the expectation depends on the run configuration ``wpt
update-expectations`` is able to generate conditional values. Because
the relevant variables depend on the range of configurations that need
to be covered, it's necessary to specify the list of configuration
variables that should be used. This is done using a ``json`` format
file that can be specified with the ``--properties-file`` command line
argument to ``wpt update-expectations``. When this isn't supplied the
defaults from ``<metadata root>/update_properties.json`` are used, if
present.

Properties File Format
++++++++++++++++++++++

The file is JSON formatted with two top-level keys:

:``properties``:
  A list of property names to consider for conditionals
  e.g ``["product", "os"]``.

:``dependents``:
  An optional dictionary containing properties that
  should only be used as "tie-breakers" when differentiating based on a
  specific top-level property has failed. This is useful when the
  dependent property is always more specific than the top-level
  property, but less understandable when used directly. For example the
  ``version`` property covering different OS versions is typically
  unique amongst different operating systems, but using it when the
  ``os`` property would do instead is likely to produce metadata that's
  too specific to the current configuration and more difficult to
  read. But where there are multiple versions of the same operating
  system with different results, it can be necessary. So specifying
  ``{"os": ["version"]}`` as a dependent property means that the
  ``version`` property will only be used if the condition already
  contains the ``os`` property and further conditions are required to
  separate the observed results.

So an example ``update-properties.json`` file might look like::

  {
    "properties": ["product", "os"],
    "dependents": {"product": ["browser_channel"], "os": ["version"]}
  }

Examples
~~~~~~~~

Update all the expectations from a set of cross-platform test runs::

  wpt update-expectations --full osx.log linux.log windows.log

Add expectation data for some new tests that are expected to be
platform-independent::

  wpt update-expectations tests.log

Why a Custom Format?
--------------------

Introduction
------------

Given the use of the metadata files in CI systems, it was desirable to
have something with the following properties:

 * Human readable

 * Human editable

 * Machine readable / writable

 * Capable of storing key-value pairs

 * Suitable for storing in a version control system (i.e. text-based)

The need for different results per platform means either having
multiple expectation files for each platform, or having a way to
express conditional values within a certain file. The former would be
rather cumbersome for humans updating the expectation files, so the
latter approach has been adopted, leading to the requirement:

 * Capable of storing result values that are conditional on the platform.

There are few extant formats that clearly meet these requirements. In
particular although conditional properties could be expressed in many
existing formats, the representation would likely be cumbersome and
error-prone for hand authoring. Therefore it was decided that a custom
format offered the best tradeoffs given the requirements.
