py.test 2.0.0: asserts++, unittest++, reporting++, config++, docs++
===========================================================================

Welcome to pytest-2.0.0, a major new release of "py.test", the rapid
easy Python testing tool.  There are many new features and enhancements,
see below for summary and detailed lists.  A lot of long-deprecated code
has been removed, resulting in a much smaller and cleaner
implementation.  See the new docs with examples here:

    http://pytest.org/en/stable/index.html

A note on packaging: pytest used to part of the "py" distribution up
until version py-1.3.4 but this has changed now:  pytest-2.0.0 only
contains py.test related code and is expected to be backward-compatible
to existing test code. If you want to install pytest, just type one of::

    pip install -U pytest
    easy_install -U pytest

Many thanks to all issue reporters and people asking questions or
complaining.  Particular thanks to Floris Bruynooghe and Ronny Pfannschmidt
for their great coding contributions and many others for feedback and help.

best,
holger krekel


New Features
-----------------------

- new invocations through Python interpreter and from Python::

    python -m pytest      # on all pythons >= 2.5

  or from a python program::

    import pytest ; pytest.main(arglist, pluginlist)

  see http://pytest.org/en/stable/how-to/usage.html for details.

- new and better reporting information in assert expressions
  if comparing lists, sequences or strings.

  see http://pytest.org/en/stable/how-to/assert.html#newreport

- new configuration through ini-files (setup.cfg or tox.ini recognized),
  for example::

    [pytest]
    norecursedirs = .hg data*  # don't ever recurse in such dirs
    addopts = -x --pyargs      # add these command line options by default

  see http://pytest.org/en/stable/reference/customize.html

- improved standard unittest support.  In general py.test should now
  better be able to run custom unittest.TestCases like twisted trial
  or Django based TestCases.  Also you can now run the tests of an
  installed 'unittest' package with py.test::

    py.test --pyargs unittest

- new "-q" option which decreases verbosity and prints a more
  nose/unittest-style "dot" output.

- many many more detailed improvements details

Fixes
-----------------------

- fix issue126 - introduce py.test.set_trace() to trace execution via
  PDB during the running of tests even if capturing is ongoing.
- fix issue124 - make reporting more resilient against tests opening
  files on filedescriptor 1 (stdout).
- fix issue109 - sibling conftest.py files will not be loaded.
  (and Directory collectors cannot be customized anymore from a Directory's
  conftest.py - this needs to happen at least one level up).
- fix issue88 (finding custom test nodes from command line arg)
- fix issue93 stdout/stderr is captured while importing conftest.py
- fix bug: unittest collected functions now also can have "pytestmark"
  applied at class/module level

Important Notes
--------------------

* The usual way in pre-2.0 times to use py.test in python code was
  to import "py" and then e.g. use "py.test.raises" for the helper.
  This remains valid and is not planned to be deprecated.  However,
  in most examples and internal code you'll find "import pytest"
  and "pytest.raises" used as the recommended default way.

* pytest now first performs collection of the complete test suite
  before running any test. This changes for example the semantics of when
  pytest_collectstart/pytest_collectreport are called.  Some plugins may
  need upgrading.

* The pytest package consists of a 400 LOC core.py and about 20 builtin plugins,
  summing up to roughly 5000 LOCs, including docstrings. To be fair, it also
  uses generic code from the "pylib", and the new "py" package to help
  with filesystem and introspection/code manipulation.

(Incompatible) Removals
-----------------------------

- py.test.config is now only available if you are in a test run.

- the following (mostly already deprecated) functionality was removed:

  - removed support for Module/Class/... collection node definitions
    in conftest.py files.  They will cause nothing special.
  - removed support for calling the pre-1.0 collection API of "run()" and "join"
  - removed reading option values from conftest.py files or env variables.
    This can now be done much much better and easier through the ini-file
    mechanism and the "addopts" entry in particular.
  - removed the "disabled" attribute in test classes.  Use the skipping
    and pytestmark mechanism to skip or xfail a test class.

- py.test.collect.Directory does not exist anymore and it
  is not possible to provide an own "Directory" object.
  If you have used this and don't know what to do, get
  in contact.  We'll figure something out.

  Note that pytest_collect_directory() is still called but
  any return value will be ignored.  This allows to keep
  old code working that performed for example "py.test.skip()"
  in collect() to prevent recursion into directory trees
  if a certain dependency or command line option is missing.


see :ref:`changelog` for more detailed changes.
