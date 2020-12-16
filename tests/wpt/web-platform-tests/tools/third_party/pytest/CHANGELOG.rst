=================
Changelog history
=================

Versions follow `Semantic Versioning <https://semver.org/>`_ (``<major>.<minor>.<patch>``).

Backward incompatible (breaking) changes will only be introduced in major versions
with advance notice in the **Deprecations** section of releases.


..
    You should *NOT* be adding new change log entries to this file, this
    file is managed by towncrier. You *may* edit previous change logs to
    fix problems like typo corrections or such.
    To add a new change log entry, please see
    https://pip.pypa.io/en/latest/development/#adding-a-news-entry
    we named the news folder changelog

.. towncrier release notes start

pytest 4.6.11 (2020-06-04)
==========================

Bug Fixes
---------

- `#6334 <https://github.com/pytest-dev/pytest/issues/6334>`_: Fix summary entries appearing twice when ``f/F`` and ``s/S`` report chars were used at the same time in the ``-r`` command-line option (for example ``-rFf``).

  The upper case variants were never documented and the preferred form should be the lower case.


- `#7310 <https://github.com/pytest-dev/pytest/issues/7310>`_: Fix ``UnboundLocalError: local variable 'letter' referenced before
  assignment`` in ``_pytest.terminal.pytest_report_teststatus()``
  when plugins return report objects in an unconventional state.

  This was making ``pytest_report_teststatus()`` skip
  entering if-block branches that declare the ``letter`` variable.

  The fix was to set the initial value of the ``letter`` before
  the if-block cascade so that it always has a value.


pytest 4.6.10 (2020-05-08)
==========================

Features
--------

- `#6870 <https://github.com/pytest-dev/pytest/issues/6870>`_: New ``Config.invocation_args`` attribute containing the unchanged arguments passed to ``pytest.main()``.

  Remark: while this is technically a new feature and according to our `policy <https://docs.pytest.org/en/latest/py27-py34-deprecation.html#what-goes-into-4-6-x-releases>`_ it should not have been backported, we have opened an exception in this particular case because it fixes a serious interaction with ``pytest-xdist``, so it can also be considered a bugfix.

Trivial/Internal Changes
------------------------

- `#6404 <https://github.com/pytest-dev/pytest/issues/6404>`_: Remove usage of ``parser`` module, deprecated in Python 3.9.


pytest 4.6.9 (2020-01-04)
=========================

Bug Fixes
---------

- `#6301 <https://github.com/pytest-dev/pytest/issues/6301>`_: Fix assertion rewriting for egg-based distributions and ``editable`` installs (``pip install --editable``).


pytest 4.6.8 (2019-12-19)
=========================

Features
--------

- `#5471 <https://github.com/pytest-dev/pytest/issues/5471>`_: JUnit XML now includes a timestamp and hostname in the testsuite tag.



Bug Fixes
---------

- `#5430 <https://github.com/pytest-dev/pytest/issues/5430>`_: junitxml: Logs for failed test are now passed to junit report in case the test fails during call phase.



Trivial/Internal Changes
------------------------

- `#6345 <https://github.com/pytest-dev/pytest/issues/6345>`_: Pin ``colorama`` to ``0.4.1`` only for Python 3.4 so newer Python versions can still receive colorama updates.


pytest 4.6.7 (2019-12-05)
=========================

Bug Fixes
---------

- `#5477 <https://github.com/pytest-dev/pytest/issues/5477>`_: The XML file produced by ``--junitxml`` now correctly contain a ``<testsuites>`` root element.


- `#6044 <https://github.com/pytest-dev/pytest/issues/6044>`_: Properly ignore ``FileNotFoundError`` (``OSError.errno == NOENT`` in Python 2) exceptions when trying to remove old temporary directories,
  for instance when multiple processes try to remove the same directory (common with ``pytest-xdist``
  for example).


pytest 4.6.6 (2019-10-11)
=========================

Bug Fixes
---------

- `#5523 <https://github.com/pytest-dev/pytest/issues/5523>`_: Fixed using multiple short options together in the command-line (for example ``-vs``) in Python 3.8+.


- `#5537 <https://github.com/pytest-dev/pytest/issues/5537>`_: Replace ``importlib_metadata`` backport with ``importlib.metadata`` from the
  standard library on Python 3.8+.


- `#5806 <https://github.com/pytest-dev/pytest/issues/5806>`_: Fix "lexer" being used when uploading to bpaste.net from ``--pastebin`` to "text".


- `#5902 <https://github.com/pytest-dev/pytest/issues/5902>`_: Fix warnings about deprecated ``cmp`` attribute in ``attrs>=19.2``.



Trivial/Internal Changes
------------------------

- `#5801 <https://github.com/pytest-dev/pytest/issues/5801>`_: Fixes python version checks (detected by ``flake8-2020``) in case python4 becomes a thing.


pytest 4.6.5 (2019-08-05)
=========================

Bug Fixes
---------

- `#4344 <https://github.com/pytest-dev/pytest/issues/4344>`_: Fix RuntimeError/StopIteration when trying to collect package with "__init__.py" only.


- `#5478 <https://github.com/pytest-dev/pytest/issues/5478>`_: Fix encode error when using unicode strings in exceptions with ``pytest.raises``.


- `#5524 <https://github.com/pytest-dev/pytest/issues/5524>`_: Fix issue where ``tmp_path`` and ``tmpdir`` would not remove directories containing files marked as read-only,
  which could lead to pytest crashing when executed a second time with the ``--basetemp`` option.


- `#5547 <https://github.com/pytest-dev/pytest/issues/5547>`_: ``--step-wise`` now handles ``xfail(strict=True)`` markers properly.


- `#5650 <https://github.com/pytest-dev/pytest/issues/5650>`_: Improved output when parsing an ini configuration file fails.


pytest 4.6.4 (2019-06-28)
=========================

Bug Fixes
---------

- `#5404 <https://github.com/pytest-dev/pytest/issues/5404>`_: Emit a warning when attempting to unwrap a broken object raises an exception,
  for easier debugging (`#5080 <https://github.com/pytest-dev/pytest/issues/5080>`__).


- `#5444 <https://github.com/pytest-dev/pytest/issues/5444>`_: Fix ``--stepwise`` mode when the first file passed on the command-line fails to collect.


- `#5482 <https://github.com/pytest-dev/pytest/issues/5482>`_: Fix bug introduced in 4.6.0 causing collection errors when passing
  more than 2 positional arguments to ``pytest.mark.parametrize``.


- `#5505 <https://github.com/pytest-dev/pytest/issues/5505>`_: Fix crash when discovery fails while using ``-p no:terminal``.


pytest 4.6.3 (2019-06-11)
=========================

Bug Fixes
---------

- `#5383 <https://github.com/pytest-dev/pytest/issues/5383>`_: ``-q`` has again an impact on the style of the collected items
  (``--collect-only``) when ``--log-cli-level`` is used.


- `#5389 <https://github.com/pytest-dev/pytest/issues/5389>`_: Fix regressions of `#5063 <https://github.com/pytest-dev/pytest/pull/5063>`__ for ``importlib_metadata.PathDistribution`` which have their ``files`` attribute being ``None``.


- `#5390 <https://github.com/pytest-dev/pytest/issues/5390>`_: Fix regression where the ``obj`` attribute of ``TestCase`` items was no longer bound to methods.


pytest 4.6.2 (2019-06-03)
=========================

Bug Fixes
---------

- `#5370 <https://github.com/pytest-dev/pytest/issues/5370>`_: Revert unrolling of ``all()`` to fix ``NameError`` on nested comprehensions.


- `#5371 <https://github.com/pytest-dev/pytest/issues/5371>`_: Revert unrolling of ``all()`` to fix incorrect handling of generators with ``if``.


- `#5372 <https://github.com/pytest-dev/pytest/issues/5372>`_: Revert unrolling of ``all()`` to fix incorrect assertion when using ``all()`` in an expression.


pytest 4.6.1 (2019-06-02)
=========================

Bug Fixes
---------

- `#5354 <https://github.com/pytest-dev/pytest/issues/5354>`_: Fix ``pytest.mark.parametrize`` when the argvalues is an iterator.


- `#5358 <https://github.com/pytest-dev/pytest/issues/5358>`_: Fix assertion rewriting of ``all()`` calls to deal with non-generators.


pytest 4.6.0 (2019-05-31)
=========================

Important
---------

The ``4.6.X`` series will be the last series to support **Python 2 and Python 3.4**.

For more details, see our `Python 2.7 and 3.4 support plan <https://docs.pytest.org/en/latest/py27-py34-deprecation.html>`__.


Features
--------

- `#4559 <https://github.com/pytest-dev/pytest/issues/4559>`_: Added the ``junit_log_passing_tests`` ini value which can be used to enable or disable logging of passing test output in the Junit XML file.


- `#4956 <https://github.com/pytest-dev/pytest/issues/4956>`_: pytester's ``testdir.spawn`` uses ``tmpdir`` as HOME/USERPROFILE directory.


- `#5062 <https://github.com/pytest-dev/pytest/issues/5062>`_: Unroll calls to ``all`` to full for-loops with assertion rewriting for better failure messages, especially when using Generator Expressions.


- `#5063 <https://github.com/pytest-dev/pytest/issues/5063>`_: Switch from ``pkg_resources`` to ``importlib-metadata`` for entrypoint detection for improved performance and import time.


- `#5091 <https://github.com/pytest-dev/pytest/issues/5091>`_: The output for ini options in ``--help`` has been improved.


- `#5269 <https://github.com/pytest-dev/pytest/issues/5269>`_: ``pytest.importorskip`` includes the ``ImportError`` now in the default ``reason``.


- `#5311 <https://github.com/pytest-dev/pytest/issues/5311>`_: Captured logs that are output for each failing test are formatted using the
  ColoredLevelFormatter.


- `#5312 <https://github.com/pytest-dev/pytest/issues/5312>`_: Improved formatting of multiline log messages in Python 3.



Bug Fixes
---------

- `#2064 <https://github.com/pytest-dev/pytest/issues/2064>`_: The debugging plugin imports the wrapped ``Pdb`` class (``--pdbcls``) on-demand now.


- `#4908 <https://github.com/pytest-dev/pytest/issues/4908>`_: The ``pytest_enter_pdb`` hook gets called with post-mortem (``--pdb``).


- `#5036 <https://github.com/pytest-dev/pytest/issues/5036>`_: Fix issue where fixtures dependent on other parametrized fixtures would be erroneously parametrized.


- `#5256 <https://github.com/pytest-dev/pytest/issues/5256>`_: Handle internal error due to a lone surrogate unicode character not being representable in Jython.


- `#5257 <https://github.com/pytest-dev/pytest/issues/5257>`_: Ensure that ``sys.stdout.mode`` does not include ``'b'`` as it is a text stream.


- `#5278 <https://github.com/pytest-dev/pytest/issues/5278>`_: Pytest's internal python plugin can be disabled using ``-p no:python`` again.


- `#5286 <https://github.com/pytest-dev/pytest/issues/5286>`_: Fix issue with ``disable_test_id_escaping_and_forfeit_all_rights_to_community_support`` option not working when using a list of test IDs in parametrized tests.


- `#5330 <https://github.com/pytest-dev/pytest/issues/5330>`_: Show the test module being collected when emitting ``PytestCollectionWarning`` messages for
  test classes with ``__init__`` and ``__new__`` methods to make it easier to pin down the problem.


- `#5333 <https://github.com/pytest-dev/pytest/issues/5333>`_: Fix regression in 4.5.0 with ``--lf`` not re-running all tests with known failures from non-selected tests.



Improved Documentation
----------------------

- `#5250 <https://github.com/pytest-dev/pytest/issues/5250>`_: Expand docs on use of ``setenv`` and ``delenv`` with ``monkeypatch``.


pytest 4.5.0 (2019-05-11)
=========================

Features
--------

- `#4826 <https://github.com/pytest-dev/pytest/issues/4826>`_: A warning is now emitted when unknown marks are used as a decorator.
  This is often due to a typo, which can lead to silently broken tests.


- `#4907 <https://github.com/pytest-dev/pytest/issues/4907>`_: Show XFail reason as part of JUnitXML message field.


- `#5013 <https://github.com/pytest-dev/pytest/issues/5013>`_: Messages from crash reports are displayed within test summaries now, truncated to the terminal width.


- `#5023 <https://github.com/pytest-dev/pytest/issues/5023>`_: New flag ``--strict-markers`` that triggers an error when unknown markers (e.g. those not registered using the `markers option`_ in the configuration file) are used in the test suite.

  The existing ``--strict`` option has the same behavior currently, but can be augmented in the future for additional checks.

  .. _`markers option`: https://docs.pytest.org/en/latest/reference.html#confval-markers


- `#5026 <https://github.com/pytest-dev/pytest/issues/5026>`_: Assertion failure messages for sequences and dicts contain the number of different items now.


- `#5034 <https://github.com/pytest-dev/pytest/issues/5034>`_: Improve reporting with ``--lf`` and ``--ff`` (run-last-failure).


- `#5035 <https://github.com/pytest-dev/pytest/issues/5035>`_: The ``--cache-show`` option/action accepts an optional glob to show only matching cache entries.


- `#5059 <https://github.com/pytest-dev/pytest/issues/5059>`_: Standard input (stdin) can be given to pytester's ``Testdir.run()`` and ``Testdir.popen()``.


- `#5068 <https://github.com/pytest-dev/pytest/issues/5068>`_: The ``-r`` option learnt about ``A`` to display all reports (including passed ones) in the short test summary.


- `#5108 <https://github.com/pytest-dev/pytest/issues/5108>`_: The short test summary is displayed after passes with output (``-rP``).


- `#5172 <https://github.com/pytest-dev/pytest/issues/5172>`_: The ``--last-failed`` (``--lf``) option got smarter and will now skip entire files if all tests
  of that test file have passed in previous runs, greatly speeding up collection.


- `#5177 <https://github.com/pytest-dev/pytest/issues/5177>`_: Introduce new specific warning ``PytestWarning`` subclasses to make it easier to filter warnings based on the class, rather than on the message. The new subclasses are:


  * ``PytestAssertRewriteWarning``

  * ``PytestCacheWarning``

  * ``PytestCollectionWarning``

  * ``PytestConfigWarning``

  * ``PytestUnhandledCoroutineWarning``

  * ``PytestUnknownMarkWarning``


- `#5202 <https://github.com/pytest-dev/pytest/issues/5202>`_: New ``record_testsuite_property`` session-scoped fixture allows users to log ``<property>`` tags at the ``testsuite``
  level with the ``junitxml`` plugin.

  The generated XML is compatible with the latest xunit standard, contrary to
  the properties recorded by ``record_property`` and ``record_xml_attribute``.


- `#5214 <https://github.com/pytest-dev/pytest/issues/5214>`_: The default logging format has been changed to improve readability. Here is an
  example of a previous logging message::

      test_log_cli_enabled_disabled.py    3 CRITICAL critical message logged by test

  This has now become::

      CRITICAL root:test_log_cli_enabled_disabled.py:3 critical message logged by test

  The formatting can be changed through the `log_format <https://docs.pytest.org/en/latest/reference.html#confval-log_format>`__ configuration option.


- `#5220 <https://github.com/pytest-dev/pytest/issues/5220>`_: ``--fixtures`` now also shows fixture scope for scopes other than ``"function"``.



Bug Fixes
---------

- `#5113 <https://github.com/pytest-dev/pytest/issues/5113>`_: Deselected items from plugins using ``pytest_collect_modifyitems`` as a hookwrapper are correctly reported now.


- `#5144 <https://github.com/pytest-dev/pytest/issues/5144>`_: With usage errors ``exitstatus`` is set to ``EXIT_USAGEERROR`` in the ``pytest_sessionfinish`` hook now as expected.


- `#5235 <https://github.com/pytest-dev/pytest/issues/5235>`_: ``outcome.exit`` is not used with ``EOF`` in the pdb wrapper anymore, but only with ``quit``.



Improved Documentation
----------------------

- `#4935 <https://github.com/pytest-dev/pytest/issues/4935>`_: Expand docs on registering marks and the effect of ``--strict``.



Trivial/Internal Changes
------------------------

- `#4942 <https://github.com/pytest-dev/pytest/issues/4942>`_: ``logging.raiseExceptions`` is not set to ``False`` anymore.


- `#5013 <https://github.com/pytest-dev/pytest/issues/5013>`_: pytest now depends on `wcwidth <https://pypi.org/project/wcwidth>`__ to properly track unicode character sizes for more precise terminal output.


- `#5059 <https://github.com/pytest-dev/pytest/issues/5059>`_: pytester's ``Testdir.popen()`` uses ``stdout`` and ``stderr`` via keyword arguments with defaults now (``subprocess.PIPE``).


- `#5069 <https://github.com/pytest-dev/pytest/issues/5069>`_: The code for the short test summary in the terminal was moved to the terminal plugin.


- `#5082 <https://github.com/pytest-dev/pytest/issues/5082>`_: Improved validation of kwargs for various methods in the pytester plugin.


- `#5202 <https://github.com/pytest-dev/pytest/issues/5202>`_: ``record_property`` now emits a ``PytestWarning`` when used with ``junit_family=xunit2``: the fixture generates
  ``property`` tags as children of ``testcase``, which is not permitted according to the most
  `recent schema <https://github.com/jenkinsci/xunit-plugin/blob/master/
  src/main/resources/org/jenkinsci/plugins/xunit/types/model/xsd/junit-10.xsd>`__.


- `#5239 <https://github.com/pytest-dev/pytest/issues/5239>`_: Pin ``pluggy`` to ``< 1.0`` so we don't update to ``1.0`` automatically when
  it gets released: there are planned breaking changes, and we want to ensure
  pytest properly supports ``pluggy 1.0``.


pytest 4.4.2 (2019-05-08)
=========================

Bug Fixes
---------

- `#5089 <https://github.com/pytest-dev/pytest/issues/5089>`_: Fix crash caused by error in ``__repr__`` function with both ``showlocals`` and verbose output enabled.


- `#5139 <https://github.com/pytest-dev/pytest/issues/5139>`_: Eliminate core dependency on 'terminal' plugin.


- `#5229 <https://github.com/pytest-dev/pytest/issues/5229>`_: Require ``pluggy>=0.11.0`` which reverts a dependency to ``importlib-metadata`` added in ``0.10.0``.
  The ``importlib-metadata`` package cannot be imported when installed as an egg and causes issues when relying on ``setup.py`` to install test dependencies.



Improved Documentation
----------------------

- `#5171 <https://github.com/pytest-dev/pytest/issues/5171>`_: Doc: ``pytest_ignore_collect``, ``pytest_collect_directory``, ``pytest_collect_file`` and ``pytest_pycollect_makemodule`` hooks's 'path' parameter documented type is now ``py.path.local``


- `#5188 <https://github.com/pytest-dev/pytest/issues/5188>`_: Improve help for ``--runxfail`` flag.



Trivial/Internal Changes
------------------------

- `#5182 <https://github.com/pytest-dev/pytest/issues/5182>`_: Removed internal and unused ``_pytest.deprecated.MARK_INFO_ATTRIBUTE``.


pytest 4.4.1 (2019-04-15)
=========================

Bug Fixes
---------

- `#5031 <https://github.com/pytest-dev/pytest/issues/5031>`_: Environment variables are properly restored when using pytester's ``testdir`` fixture.


- `#5039 <https://github.com/pytest-dev/pytest/issues/5039>`_: Fix regression with ``--pdbcls``, which stopped working with local modules in 4.0.0.


- `#5092 <https://github.com/pytest-dev/pytest/issues/5092>`_: Produce a warning when unknown keywords are passed to ``pytest.param(...)``.


- `#5098 <https://github.com/pytest-dev/pytest/issues/5098>`_: Invalidate import caches with ``monkeypatch.syspath_prepend``, which is required with namespace packages being used.


pytest 4.4.0 (2019-03-29)
=========================

Features
--------

- `#2224 <https://github.com/pytest-dev/pytest/issues/2224>`_: ``async`` test functions are skipped and a warning is emitted when a suitable
  async plugin is not installed (such as ``pytest-asyncio`` or ``pytest-trio``).

  Previously ``async`` functions would not execute at all but still be marked as "passed".


- `#2482 <https://github.com/pytest-dev/pytest/issues/2482>`_: Include new ``disable_test_id_escaping_and_forfeit_all_rights_to_community_support`` option to disable ascii-escaping in parametrized values. This may cause a series of problems and as the name makes clear, use at your own risk.


- `#4718 <https://github.com/pytest-dev/pytest/issues/4718>`_: The ``-p`` option can now be used to early-load plugins also by entry-point name, instead of just
  by module name.

  This makes it possible to early load external plugins like ``pytest-cov`` in the command-line::

      pytest -p pytest_cov


- `#4855 <https://github.com/pytest-dev/pytest/issues/4855>`_: The ``--pdbcls`` option handles classes via module attributes now (e.g.
  ``pdb:pdb.Pdb`` with `pdb++`_), and its validation was improved.

  .. _pdb++: https://pypi.org/project/pdbpp/


- `#4875 <https://github.com/pytest-dev/pytest/issues/4875>`_: The `testpaths <https://docs.pytest.org/en/latest/reference.html#confval-testpaths>`__ configuration option is now displayed next
  to the ``rootdir`` and ``inifile`` lines in the pytest header if the option is in effect, i.e., directories or file names were
  not explicitly passed in the command line.

  Also, ``inifile`` is only displayed if there's a configuration file, instead of an empty ``inifile:`` string.


- `#4911 <https://github.com/pytest-dev/pytest/issues/4911>`_: Doctests can be skipped now dynamically using ``pytest.skip()``.


- `#4920 <https://github.com/pytest-dev/pytest/issues/4920>`_: Internal refactorings have been made in order to make the implementation of the
  `pytest-subtests <https://github.com/pytest-dev/pytest-subtests>`__ plugin
  possible, which adds unittest sub-test support and a new ``subtests`` fixture as discussed in
  `#1367 <https://github.com/pytest-dev/pytest/issues/1367>`__.

  For details on the internal refactorings, please see the details on the related PR.


- `#4931 <https://github.com/pytest-dev/pytest/issues/4931>`_: pytester's ``LineMatcher`` asserts that the passed lines are a sequence.


- `#4936 <https://github.com/pytest-dev/pytest/issues/4936>`_: Handle ``-p plug`` after ``-p no:plug``.

  This can be used to override a blocked plugin (e.g. in "addopts") from the
  command line etc.


- `#4951 <https://github.com/pytest-dev/pytest/issues/4951>`_: Output capturing is handled correctly when only capturing via fixtures (capsys, capfs) with ``pdb.set_trace()``.


- `#4956 <https://github.com/pytest-dev/pytest/issues/4956>`_: ``pytester`` sets ``$HOME`` and ``$USERPROFILE`` to the temporary directory during test runs.

  This ensures to not load configuration files from the real user's home directory.


- `#4980 <https://github.com/pytest-dev/pytest/issues/4980>`_: Namespace packages are handled better with ``monkeypatch.syspath_prepend`` and ``testdir.syspathinsert`` (via ``pkg_resources.fixup_namespace_packages``).


- `#4993 <https://github.com/pytest-dev/pytest/issues/4993>`_: The stepwise plugin reports status information now.


- `#5008 <https://github.com/pytest-dev/pytest/issues/5008>`_: If a ``setup.cfg`` file contains ``[tool:pytest]`` and also the no longer supported ``[pytest]`` section, pytest will use ``[tool:pytest]`` ignoring ``[pytest]``. Previously it would unconditionally error out.

  This makes it simpler for plugins to support old pytest versions.



Bug Fixes
---------

- `#1895 <https://github.com/pytest-dev/pytest/issues/1895>`_: Fix bug where fixtures requested dynamically via ``request.getfixturevalue()`` might be teardown
  before the requesting fixture.


- `#4851 <https://github.com/pytest-dev/pytest/issues/4851>`_: pytester unsets ``PYTEST_ADDOPTS`` now to not use outer options with ``testdir.runpytest()``.


- `#4903 <https://github.com/pytest-dev/pytest/issues/4903>`_: Use the correct modified time for years after 2038 in rewritten ``.pyc`` files.


- `#4928 <https://github.com/pytest-dev/pytest/issues/4928>`_: Fix line offsets with ``ScopeMismatch`` errors.


- `#4957 <https://github.com/pytest-dev/pytest/issues/4957>`_: ``-p no:plugin`` is handled correctly for default (internal) plugins now, e.g. with ``-p no:capture``.

  Previously they were loaded (imported) always, making e.g. the ``capfd`` fixture available.


- `#4968 <https://github.com/pytest-dev/pytest/issues/4968>`_: The pdb ``quit`` command is handled properly when used after the ``debug`` command with `pdb++`_.

  .. _pdb++: https://pypi.org/project/pdbpp/


- `#4975 <https://github.com/pytest-dev/pytest/issues/4975>`_: Fix the interpretation of ``-qq`` option where it was being considered as ``-v`` instead.


- `#4978 <https://github.com/pytest-dev/pytest/issues/4978>`_: ``outcomes.Exit`` is not swallowed in ``assertrepr_compare`` anymore.


- `#4988 <https://github.com/pytest-dev/pytest/issues/4988>`_: Close logging's file handler explicitly when the session finishes.


- `#5003 <https://github.com/pytest-dev/pytest/issues/5003>`_: Fix line offset with mark collection error (off by one).



Improved Documentation
----------------------

- `#4974 <https://github.com/pytest-dev/pytest/issues/4974>`_: Update docs for ``pytest_cmdline_parse`` hook to note availability liminations



Trivial/Internal Changes
------------------------

- `#4718 <https://github.com/pytest-dev/pytest/issues/4718>`_: ``pluggy>=0.9`` is now required.


- `#4815 <https://github.com/pytest-dev/pytest/issues/4815>`_: ``funcsigs>=1.0`` is now required for Python 2.7.


- `#4829 <https://github.com/pytest-dev/pytest/issues/4829>`_: Some left-over internal code related to ``yield`` tests has been removed.


- `#4890 <https://github.com/pytest-dev/pytest/issues/4890>`_: Remove internally unused ``anypython`` fixture from the pytester plugin.


- `#4912 <https://github.com/pytest-dev/pytest/issues/4912>`_: Remove deprecated Sphinx directive, ``add_description_unit()``,
  pin sphinx-removed-in to >= 0.2.0 to support Sphinx 2.0.


- `#4913 <https://github.com/pytest-dev/pytest/issues/4913>`_: Fix pytest tests invocation with custom ``PYTHONPATH``.


- `#4965 <https://github.com/pytest-dev/pytest/issues/4965>`_: New ``pytest_report_to_serializable`` and ``pytest_report_from_serializable`` **experimental** hooks.

  These hooks will be used by ``pytest-xdist``, ``pytest-subtests``, and the replacement for
  resultlog to serialize and customize reports.

  They are experimental, meaning that their details might change or even be removed
  completely in future patch releases without warning.

  Feedback is welcome from plugin authors and users alike.


- `#4987 <https://github.com/pytest-dev/pytest/issues/4987>`_: ``Collector.repr_failure`` respects the ``--tb`` option, but only defaults to ``short`` now (with ``auto``).


pytest 4.3.1 (2019-03-11)
=========================

Bug Fixes
---------

- `#4810 <https://github.com/pytest-dev/pytest/issues/4810>`_: Logging messages inside ``pytest_runtest_logreport()`` are now properly captured and displayed.


- `#4861 <https://github.com/pytest-dev/pytest/issues/4861>`_: Improve validation of contents written to captured output so it behaves the same as when capture is disabled.


- `#4898 <https://github.com/pytest-dev/pytest/issues/4898>`_: Fix ``AttributeError: FixtureRequest has no 'confg' attribute`` bug in ``testdir.copy_example``.



Trivial/Internal Changes
------------------------

- `#4768 <https://github.com/pytest-dev/pytest/issues/4768>`_: Avoid pkg_resources import at the top-level.


pytest 4.3.0 (2019-02-16)
=========================

Deprecations
------------

- `#4724 <https://github.com/pytest-dev/pytest/issues/4724>`_: ``pytest.warns()`` now emits a warning when it receives unknown keyword arguments.

  This will be changed into an error in the future.



Features
--------

- `#2753 <https://github.com/pytest-dev/pytest/issues/2753>`_: Usage errors from argparse are mapped to pytest's ``UsageError``.


- `#3711 <https://github.com/pytest-dev/pytest/issues/3711>`_: Add the ``--ignore-glob`` parameter to exclude test-modules with Unix shell-style wildcards.
  Add the ``collect_ignore_glob`` for ``conftest.py`` to exclude test-modules with Unix shell-style wildcards.


- `#4698 <https://github.com/pytest-dev/pytest/issues/4698>`_: The warning about Python 2.7 and 3.4 not being supported in pytest 5.0 has been removed.

  In the end it was considered to be more
  of a nuisance than actual utility and users of those Python versions shouldn't have problems as ``pip`` will not
  install pytest 5.0 on those interpreters.


- `#4707 <https://github.com/pytest-dev/pytest/issues/4707>`_: With the help of new ``set_log_path()`` method there is a way to set ``log_file`` paths from hooks.



Bug Fixes
---------

- `#4651 <https://github.com/pytest-dev/pytest/issues/4651>`_: ``--help`` and ``--version`` are handled with ``UsageError``.


- `#4782 <https://github.com/pytest-dev/pytest/issues/4782>`_: Fix ``AssertionError`` with collection of broken symlinks with packages.


pytest 4.2.1 (2019-02-12)
=========================

Bug Fixes
---------

- `#2895 <https://github.com/pytest-dev/pytest/issues/2895>`_: The ``pytest_report_collectionfinish`` hook now is also called with ``--collect-only``.


- `#3899 <https://github.com/pytest-dev/pytest/issues/3899>`_: Do not raise ``UsageError`` when an imported package has a ``pytest_plugins.py`` child module.


- `#4347 <https://github.com/pytest-dev/pytest/issues/4347>`_: Fix output capturing when using pdb++ with recursive debugging.


- `#4592 <https://github.com/pytest-dev/pytest/issues/4592>`_: Fix handling of ``collect_ignore`` via parent ``conftest.py``.


- `#4700 <https://github.com/pytest-dev/pytest/issues/4700>`_: Fix regression where ``setUpClass`` would always be called in subclasses even if all tests
  were skipped by a ``unittest.skip()`` decorator applied in the subclass.


- `#4739 <https://github.com/pytest-dev/pytest/issues/4739>`_: Fix ``parametrize(... ids=<function>)`` when the function returns non-strings.


- `#4745 <https://github.com/pytest-dev/pytest/issues/4745>`_: Fix/improve collection of args when passing in ``__init__.py`` and a test file.


- `#4770 <https://github.com/pytest-dev/pytest/issues/4770>`_: ``more_itertools`` is now constrained to <6.0.0 when required for Python 2.7 compatibility.


- `#526 <https://github.com/pytest-dev/pytest/issues/526>`_: Fix "ValueError: Plugin already registered" exceptions when running in build directories that symlink to actual source.



Improved Documentation
----------------------

- `#3899 <https://github.com/pytest-dev/pytest/issues/3899>`_: Add note to ``plugins.rst`` that ``pytest_plugins`` should not be used as a name for a user module containing plugins.


- `#4324 <https://github.com/pytest-dev/pytest/issues/4324>`_: Document how to use ``raises`` and ``does_not_raise`` to write parametrized tests with conditional raises.


- `#4709 <https://github.com/pytest-dev/pytest/issues/4709>`_: Document how to customize test failure messages when using
  ``pytest.warns``.



Trivial/Internal Changes
------------------------

- `#4741 <https://github.com/pytest-dev/pytest/issues/4741>`_: Some verbosity related attributes of the TerminalReporter plugin are now
  read only properties.


pytest 4.2.0 (2019-01-30)
=========================

Features
--------

- `#3094 <https://github.com/pytest-dev/pytest/issues/3094>`_: `Classic xunit-style <https://docs.pytest.org/en/latest/xunit_setup.html>`__ functions and methods
  now obey the scope of *autouse* fixtures.

  This fixes a number of surprising issues like ``setup_method`` being called before session-scoped
  autouse fixtures (see `#517 <https://github.com/pytest-dev/pytest/issues/517>`__ for an example).


- `#4627 <https://github.com/pytest-dev/pytest/issues/4627>`_: Display a message at the end of the test session when running under Python 2.7 and 3.4 that pytest 5.0 will no longer
  support those Python versions.


- `#4660 <https://github.com/pytest-dev/pytest/issues/4660>`_: The number of *selected* tests now are also displayed when the ``-k`` or ``-m`` flags are used.


- `#4688 <https://github.com/pytest-dev/pytest/issues/4688>`_: ``pytest_report_teststatus`` hook now can also receive a ``config`` parameter.


- `#4691 <https://github.com/pytest-dev/pytest/issues/4691>`_: ``pytest_terminal_summary`` hook now can also receive a ``config`` parameter.



Bug Fixes
---------

- `#3547 <https://github.com/pytest-dev/pytest/issues/3547>`_: ``--junitxml`` can emit XML compatible with Jenkins xUnit.
  ``junit_family`` INI option accepts ``legacy|xunit1``, which produces old style output, and ``xunit2`` that conforms more strictly to https://github.com/jenkinsci/xunit-plugin/blob/xunit-2.3.2/src/main/resources/org/jenkinsci/plugins/xunit/types/model/xsd/junit-10.xsd


- `#4280 <https://github.com/pytest-dev/pytest/issues/4280>`_: Improve quitting from pdb, especially with ``--trace``.

  Using ``q[quit]`` after ``pdb.set_trace()`` will quit pytest also.


- `#4402 <https://github.com/pytest-dev/pytest/issues/4402>`_: Warning summary now groups warnings by message instead of by test id.

  This makes the output more compact and better conveys the general idea of how much code is
  actually generating warnings, instead of how many tests call that code.


- `#4536 <https://github.com/pytest-dev/pytest/issues/4536>`_: ``monkeypatch.delattr`` handles class descriptors like ``staticmethod``/``classmethod``.


- `#4649 <https://github.com/pytest-dev/pytest/issues/4649>`_: Restore marks being considered keywords for keyword expressions.


- `#4653 <https://github.com/pytest-dev/pytest/issues/4653>`_: ``tmp_path`` fixture and other related ones provides resolved path (a.k.a real path)


- `#4667 <https://github.com/pytest-dev/pytest/issues/4667>`_: ``pytest_terminal_summary`` uses result from ``pytest_report_teststatus`` hook, rather than hardcoded strings.


- `#4669 <https://github.com/pytest-dev/pytest/issues/4669>`_: Correctly handle ``unittest.SkipTest`` exception containing non-ascii characters on Python 2.


- `#4680 <https://github.com/pytest-dev/pytest/issues/4680>`_: Ensure the ``tmpdir`` and the ``tmp_path`` fixtures are the same folder.


- `#4681 <https://github.com/pytest-dev/pytest/issues/4681>`_: Ensure ``tmp_path`` is always a real path.



Trivial/Internal Changes
------------------------

- `#4643 <https://github.com/pytest-dev/pytest/issues/4643>`_: Use ``a.item()`` instead of the deprecated ``np.asscalar(a)`` in ``pytest.approx``.

  ``np.asscalar`` has been `deprecated <https://github.com/numpy/numpy/blob/master/doc/release/1.16.0-notes.rst#new-deprecations>`__ in ``numpy 1.16.``.


- `#4657 <https://github.com/pytest-dev/pytest/issues/4657>`_: Copy saferepr from pylib


- `#4668 <https://github.com/pytest-dev/pytest/issues/4668>`_: The verbose word for expected failures in the teststatus report changes from ``xfail`` to ``XFAIL`` to be consistent with other test outcomes.


pytest 4.1.1 (2019-01-12)
=========================

Bug Fixes
---------

- `#2256 <https://github.com/pytest-dev/pytest/issues/2256>`_: Show full repr with ``assert a==b`` and ``-vv``.


- `#3456 <https://github.com/pytest-dev/pytest/issues/3456>`_: Extend Doctest-modules to ignore mock objects.


- `#4617 <https://github.com/pytest-dev/pytest/issues/4617>`_: Fixed ``pytest.warns`` bug when context manager is reused (e.g. multiple parametrization).


- `#4631 <https://github.com/pytest-dev/pytest/issues/4631>`_: Don't rewrite assertion when ``__getattr__`` is broken



Improved Documentation
----------------------

- `#3375 <https://github.com/pytest-dev/pytest/issues/3375>`_: Document that using ``setup.cfg`` may crash other tools or cause hard to track down problems because it uses a different parser than ``pytest.ini`` or ``tox.ini`` files.



Trivial/Internal Changes
------------------------

- `#4602 <https://github.com/pytest-dev/pytest/issues/4602>`_: Uninstall ``hypothesis`` in regen tox env.


pytest 4.1.0 (2019-01-05)
=========================

Removals
--------

- `#2169 <https://github.com/pytest-dev/pytest/issues/2169>`_: ``pytest.mark.parametrize``: in previous versions, errors raised by id functions were suppressed and changed into warnings. Now the exceptions are propagated, along with a pytest message informing the node, parameter value and index where the exception occurred.


- `#3078 <https://github.com/pytest-dev/pytest/issues/3078>`_: Remove legacy internal warnings system: ``config.warn``, ``Node.warn``. The ``pytest_logwarning`` now issues a warning when implemented.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#config-warn-and-node-warn>`__ on information on how to update your code.


- `#3079 <https://github.com/pytest-dev/pytest/issues/3079>`_: Removed support for yield tests - they are fundamentally broken because they don't support fixtures properly since collection and test execution were separated.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#yield-tests>`__ on information on how to update your code.


- `#3082 <https://github.com/pytest-dev/pytest/issues/3082>`_: Removed support for applying marks directly to values in ``@pytest.mark.parametrize``. Use ``pytest.param`` instead.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#marks-in-pytest-mark-parametrize>`__ on information on how to update your code.


- `#3083 <https://github.com/pytest-dev/pytest/issues/3083>`_: Removed ``Metafunc.addcall``. This was the predecessor mechanism to ``@pytest.mark.parametrize``.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#metafunc-addcall>`__ on information on how to update your code.


- `#3085 <https://github.com/pytest-dev/pytest/issues/3085>`_: Removed support for passing strings to ``pytest.main``. Now, always pass a list of strings instead.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#passing-command-line-string-to-pytest-main>`__ on information on how to update your code.


- `#3086 <https://github.com/pytest-dev/pytest/issues/3086>`_: ``[pytest]`` section in **setup.cfg** files is no longer supported, use ``[tool:pytest]`` instead. ``setup.cfg`` files
  are meant for use with ``distutils``, and a section named ``pytest`` has notoriously been a source of conflicts and bugs.

  Note that for **pytest.ini** and **tox.ini** files the section remains ``[pytest]``.


- `#3616 <https://github.com/pytest-dev/pytest/issues/3616>`_: Removed the deprecated compat properties for ``node.Class/Function/Module`` - use ``pytest.Class/Function/Module`` now.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#internal-classes-accessed-through-node>`__ on information on how to update your code.


- `#4421 <https://github.com/pytest-dev/pytest/issues/4421>`_: Removed the implementation of the ``pytest_namespace`` hook.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#pytest-namespace>`__ on information on how to update your code.


- `#4489 <https://github.com/pytest-dev/pytest/issues/4489>`_: Removed ``request.cached_setup``. This was the predecessor mechanism to modern fixtures.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#cached-setup>`__ on information on how to update your code.


- `#4535 <https://github.com/pytest-dev/pytest/issues/4535>`_: Removed the deprecated ``PyCollector.makeitem`` method. This method was made public by mistake a long time ago.


- `#4543 <https://github.com/pytest-dev/pytest/issues/4543>`_: Removed support to define fixtures using the ``pytest_funcarg__`` prefix. Use the ``@pytest.fixture`` decorator instead.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#pytest-funcarg-prefix>`__ on information on how to update your code.


- `#4545 <https://github.com/pytest-dev/pytest/issues/4545>`_: Calling fixtures directly is now always an error instead of a warning.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#calling-fixtures-directly>`__ on information on how to update your code.


- `#4546 <https://github.com/pytest-dev/pytest/issues/4546>`_: Remove ``Node.get_marker(name)`` the return value was not usable for more than a existence check.

  Use ``Node.get_closest_marker(name)`` as a replacement.


- `#4547 <https://github.com/pytest-dev/pytest/issues/4547>`_: The deprecated ``record_xml_property`` fixture has been removed, use the more generic ``record_property`` instead.

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#record-xml-property>`__ for more information.


- `#4548 <https://github.com/pytest-dev/pytest/issues/4548>`_: An error is now raised if the ``pytest_plugins`` variable is defined in a non-top-level ``conftest.py`` file (i.e., not residing in the ``rootdir``).

  See our `docs <https://docs.pytest.org/en/latest/deprecations.html#pytest-plugins-in-non-top-level-conftest-files>`__ for more information.


- `#891 <https://github.com/pytest-dev/pytest/issues/891>`_: Remove ``testfunction.markername`` attributes - use ``Node.iter_markers(name=None)`` to iterate them.



Deprecations
------------

- `#3050 <https://github.com/pytest-dev/pytest/issues/3050>`_: Deprecated the ``pytest.config`` global.

  See https://docs.pytest.org/en/latest/deprecations.html#pytest-config-global for rationale.


- `#3974 <https://github.com/pytest-dev/pytest/issues/3974>`_: Passing the ``message`` parameter of ``pytest.raises`` now issues a ``DeprecationWarning``.

  It is a common mistake to think this parameter will match the exception message, while in fact
  it only serves to provide a custom message in case the ``pytest.raises`` check fails. To avoid this
  mistake and because it is believed to be little used, pytest is deprecating it without providing
  an alternative for the moment.

  If you have concerns about this, please comment on `issue #3974 <https://github.com/pytest-dev/pytest/issues/3974>`__.


- `#4435 <https://github.com/pytest-dev/pytest/issues/4435>`_: Deprecated ``raises(..., 'code(as_a_string)')`` and ``warns(..., 'code(as_a_string)')``.

  See https://docs.pytest.org/en/latest/deprecations.html#raises-warns-exec for rationale and examples.



Features
--------

- `#3191 <https://github.com/pytest-dev/pytest/issues/3191>`_: A warning is now issued when assertions are made for ``None``.

  This is a common source of confusion among new users, which write:

  .. code-block:: python

      assert mocked_object.assert_called_with(3, 4, 5, key="value")

  When they should write:

  .. code-block:: python

      mocked_object.assert_called_with(3, 4, 5, key="value")

  Because the ``assert_called_with`` method of mock objects already executes an assertion.

  This warning will not be issued when ``None`` is explicitly checked. An assertion like:

  .. code-block:: python

      assert variable is None

  will not issue the warning.


- `#3632 <https://github.com/pytest-dev/pytest/issues/3632>`_: Richer equality comparison introspection on ``AssertionError`` for objects created using `attrs <http://www.attrs.org/en/stable/>`__ or `dataclasses <https://docs.python.org/3/library/dataclasses.html>`_ (Python 3.7+, `backported to 3.6 <https://pypi.org/project/dataclasses>`__).


- `#4278 <https://github.com/pytest-dev/pytest/issues/4278>`_: ``CACHEDIR.TAG`` files are now created inside cache directories.

  Those files are part of the `Cache Directory Tagging Standard <http://www.bford.info/cachedir/spec.html>`__, and can
  be used by backup or synchronization programs to identify pytest's cache directory as such.


- `#4292 <https://github.com/pytest-dev/pytest/issues/4292>`_: ``pytest.outcomes.Exit`` is derived from ``SystemExit`` instead of ``KeyboardInterrupt``. This allows us to better handle ``pdb`` exiting.


- `#4371 <https://github.com/pytest-dev/pytest/issues/4371>`_: Updated the ``--collect-only`` option to display test descriptions when ran using ``--verbose``.


- `#4386 <https://github.com/pytest-dev/pytest/issues/4386>`_: Restructured ``ExceptionInfo`` object construction and ensure incomplete instances have a ``repr``/``str``.


- `#4416 <https://github.com/pytest-dev/pytest/issues/4416>`_: pdb: added support for keyword arguments with ``pdb.set_trace``.

  It handles ``header`` similar to Python 3.7 does it, and forwards any
  other keyword arguments to the ``Pdb`` constructor.

  This allows for ``__import__("pdb").set_trace(skip=["foo.*"])``.


- `#4483 <https://github.com/pytest-dev/pytest/issues/4483>`_: Added ini parameter ``junit_duration_report`` to optionally report test call durations, excluding setup and teardown times.

  The JUnit XML specification and the default pytest behavior is to include setup and teardown times in the test duration
  report. You can include just the call durations instead (excluding setup and teardown) by adding this to your ``pytest.ini`` file:

  .. code-block:: ini

      [pytest]
      junit_duration_report = call


- `#4532 <https://github.com/pytest-dev/pytest/issues/4532>`_: ``-ra`` now will show errors and failures last, instead of as the first items in the summary.

  This makes it easier to obtain a list of errors and failures to run tests selectively.


- `#4599 <https://github.com/pytest-dev/pytest/issues/4599>`_: ``pytest.importorskip`` now supports a ``reason`` parameter, which will be shown when the
  requested module cannot be imported.



Bug Fixes
---------

- `#3532 <https://github.com/pytest-dev/pytest/issues/3532>`_: ``-p`` now accepts its argument without a space between the value, for example ``-pmyplugin``.


- `#4327 <https://github.com/pytest-dev/pytest/issues/4327>`_: ``approx`` again works with more generic containers, more precisely instances of ``Iterable`` and ``Sized`` instead of more restrictive ``Sequence``.


- `#4397 <https://github.com/pytest-dev/pytest/issues/4397>`_: Ensure that node ids are printable.


- `#4435 <https://github.com/pytest-dev/pytest/issues/4435>`_: Fixed ``raises(..., 'code(string)')`` frame filename.


- `#4458 <https://github.com/pytest-dev/pytest/issues/4458>`_: Display actual test ids in ``--collect-only``.



Improved Documentation
----------------------

- `#4557 <https://github.com/pytest-dev/pytest/issues/4557>`_: Markers example documentation page updated to support latest pytest version.


- `#4558 <https://github.com/pytest-dev/pytest/issues/4558>`_: Update cache documentation example to correctly show cache hit and miss.


- `#4580 <https://github.com/pytest-dev/pytest/issues/4580>`_: Improved detailed summary report documentation.



Trivial/Internal Changes
------------------------

- `#4447 <https://github.com/pytest-dev/pytest/issues/4447>`_: Changed the deprecation type of ``--result-log`` to ``PytestDeprecationWarning``.

  It was decided to remove this feature at the next major revision.


pytest 4.0.2 (2018-12-13)
=========================

Bug Fixes
---------

- `#4265 <https://github.com/pytest-dev/pytest/issues/4265>`_: Validate arguments from the ``PYTEST_ADDOPTS`` environment variable and the ``addopts`` ini option separately.


- `#4435 <https://github.com/pytest-dev/pytest/issues/4435>`_: Fix ``raises(..., 'code(string)')`` frame filename.


- `#4500 <https://github.com/pytest-dev/pytest/issues/4500>`_: When a fixture yields and a log call is made after the test runs, and, if the test is interrupted, capture attributes are ``None``.


- `#4538 <https://github.com/pytest-dev/pytest/issues/4538>`_: Raise ``TypeError`` for ``with raises(..., match=<non-None falsey value>)``.



Improved Documentation
----------------------

- `#1495 <https://github.com/pytest-dev/pytest/issues/1495>`_: Document common doctest fixture directory tree structure pitfalls


pytest 4.0.1 (2018-11-23)
=========================

Bug Fixes
---------

- `#3952 <https://github.com/pytest-dev/pytest/issues/3952>`_: Display warnings before "short test summary info" again, but still later warnings in the end.


- `#4386 <https://github.com/pytest-dev/pytest/issues/4386>`_: Handle uninitialized exceptioninfo in repr/str.


- `#4393 <https://github.com/pytest-dev/pytest/issues/4393>`_: Do not create ``.gitignore``/``README.md`` files in existing cache directories.


- `#4400 <https://github.com/pytest-dev/pytest/issues/4400>`_: Rearrange warning handling for the yield test errors so the opt-out in 4.0.x correctly works.


- `#4405 <https://github.com/pytest-dev/pytest/issues/4405>`_: Fix collection of testpaths with ``--pyargs``.


- `#4412 <https://github.com/pytest-dev/pytest/issues/4412>`_: Fix assertion rewriting involving ``Starred`` + side-effects.


- `#4425 <https://github.com/pytest-dev/pytest/issues/4425>`_: Ensure we resolve the absolute path when the given ``--basetemp`` is a relative path.



Trivial/Internal Changes
------------------------

- `#4315 <https://github.com/pytest-dev/pytest/issues/4315>`_: Use ``pkg_resources.parse_version`` instead of ``LooseVersion`` in minversion check.


- `#4440 <https://github.com/pytest-dev/pytest/issues/4440>`_: Adjust the stack level of some internal pytest warnings.


pytest 4.0.0 (2018-11-13)
=========================

Removals
--------

- `#3737 <https://github.com/pytest-dev/pytest/issues/3737>`_: **RemovedInPytest4Warnings are now errors by default.**

  Following our plan to remove deprecated features with as little disruption as
  possible, all warnings of type ``RemovedInPytest4Warnings`` now generate errors
  instead of warning messages.

  **The affected features will be effectively removed in pytest 4.1**, so please consult the
  `Deprecations and Removals <https://docs.pytest.org/en/latest/deprecations.html>`__
  section in the docs for directions on how to update existing code.

  In the pytest ``4.0.X`` series, it is possible to change the errors back into warnings as a stop
  gap measure by adding this to your ``pytest.ini`` file:

  .. code-block:: ini

      [pytest]
      filterwarnings =
          ignore::pytest.RemovedInPytest4Warning

  But this will stop working when pytest ``4.1`` is released.

  **If you have concerns** about the removal of a specific feature, please add a
  comment to `#4348 <https://github.com/pytest-dev/pytest/issues/4348>`__.


- `#4358 <https://github.com/pytest-dev/pytest/issues/4358>`_: Remove the ``::()`` notation to denote a test class instance in node ids.

  Previously, node ids that contain test instances would use ``::()`` to denote the instance like this::

      test_foo.py::Test::()::test_bar

  The extra ``::()`` was puzzling to most users and has been removed, so that the test id becomes now::

      test_foo.py::Test::test_bar

  This change could not accompany a deprecation period as is usual when user-facing functionality changes because
  it was not really possible to detect when the functionality was being used explicitly.

  The extra ``::()`` might have been removed in some places internally already,
  which then led to confusion in places where it was expected, e.g. with
  ``--deselect`` (`#4127 <https://github.com/pytest-dev/pytest/issues/4127>`_).

  Test class instances are also not listed with ``--collect-only`` anymore.



Features
--------

- `#4270 <https://github.com/pytest-dev/pytest/issues/4270>`_: The ``cache_dir`` option uses ``$TOX_ENV_DIR`` as prefix (if set in the environment).

  This uses a different cache per tox environment by default.



Bug Fixes
---------

- `#3554 <https://github.com/pytest-dev/pytest/issues/3554>`_: Fix ``CallInfo.__repr__`` for when the call is not finished yet.


pytest 3.10.1 (2018-11-11)
==========================

Bug Fixes
---------

- `#4287 <https://github.com/pytest-dev/pytest/issues/4287>`_: Fix nested usage of debugging plugin (pdb), e.g. with pytester's ``testdir.runpytest``.


- `#4304 <https://github.com/pytest-dev/pytest/issues/4304>`_: Block the ``stepwise`` plugin if ``cacheprovider`` is also blocked, as one depends on the other.


- `#4306 <https://github.com/pytest-dev/pytest/issues/4306>`_: Parse ``minversion`` as an actual version and not as dot-separated strings.


- `#4310 <https://github.com/pytest-dev/pytest/issues/4310>`_: Fix duplicate collection due to multiple args matching the same packages.


- `#4321 <https://github.com/pytest-dev/pytest/issues/4321>`_: Fix ``item.nodeid`` with resolved symlinks.


- `#4325 <https://github.com/pytest-dev/pytest/issues/4325>`_: Fix collection of direct symlinked files, where the target does not match ``python_files``.


- `#4329 <https://github.com/pytest-dev/pytest/issues/4329>`_: Fix TypeError in report_collect with _collect_report_last_write.



Trivial/Internal Changes
------------------------

- `#4305 <https://github.com/pytest-dev/pytest/issues/4305>`_: Replace byte/unicode helpers in test_capture with python level syntax.


pytest 3.10.0 (2018-11-03)
==========================

Features
--------

- `#2619 <https://github.com/pytest-dev/pytest/issues/2619>`_: Resume capturing output after ``continue`` with ``__import__("pdb").set_trace()``.

  This also adds a new ``pytest_leave_pdb`` hook, and passes in ``pdb`` to the
  existing ``pytest_enter_pdb`` hook.


- `#4147 <https://github.com/pytest-dev/pytest/issues/4147>`_: Add ``--sw``, ``--stepwise`` as an alternative to ``--lf -x`` for stopping at the first failure, but starting the next test invocation from that test.  See `the documentation <https://docs.pytest.org/en/latest/cache.html#stepwise>`__ for more info.


- `#4188 <https://github.com/pytest-dev/pytest/issues/4188>`_: Make ``--color`` emit colorful dots when not running in verbose mode. Earlier, it would only colorize the test-by-test output if ``--verbose`` was also passed.


- `#4225 <https://github.com/pytest-dev/pytest/issues/4225>`_: Improve performance with collection reporting in non-quiet mode with terminals.

  The "collecting …" message is only printed/updated every 0.5s.



Bug Fixes
---------

- `#2701 <https://github.com/pytest-dev/pytest/issues/2701>`_: Fix false ``RemovedInPytest4Warning: usage of Session... is deprecated, please use pytest`` warnings.


- `#4046 <https://github.com/pytest-dev/pytest/issues/4046>`_: Fix problems with running tests in package ``__init__.py`` files.


- `#4260 <https://github.com/pytest-dev/pytest/issues/4260>`_: Swallow warnings during anonymous compilation of source.


- `#4262 <https://github.com/pytest-dev/pytest/issues/4262>`_: Fix access denied error when deleting stale directories created by ``tmpdir`` / ``tmp_path``.


- `#611 <https://github.com/pytest-dev/pytest/issues/611>`_: Naming a fixture ``request`` will now raise a warning: the ``request`` fixture is internal and
  should not be overwritten as it will lead to internal errors.

- `#4266 <https://github.com/pytest-dev/pytest/issues/4266>`_: Handle (ignore) exceptions raised during collection, e.g. with Django's LazySettings proxy class.



Improved Documentation
----------------------

- `#4255 <https://github.com/pytest-dev/pytest/issues/4255>`_: Added missing documentation about the fact that module names passed to filter warnings are not regex-escaped.



Trivial/Internal Changes
------------------------

- `#4272 <https://github.com/pytest-dev/pytest/issues/4272>`_: Display cachedir also in non-verbose mode if non-default.


- `#4277 <https://github.com/pytest-dev/pytest/issues/4277>`_: pdb: improve message about output capturing with ``set_trace``.

  Do not display "IO-capturing turned off/on" when ``-s`` is used to avoid
  confusion.


- `#4279 <https://github.com/pytest-dev/pytest/issues/4279>`_: Improve message and stack level of warnings issued by ``monkeypatch.setenv`` when the value of the environment variable is not a ``str``.


pytest 3.9.3 (2018-10-27)
=========================

Bug Fixes
---------

- `#4174 <https://github.com/pytest-dev/pytest/issues/4174>`_: Fix "ValueError: Plugin already registered" with conftest plugins via symlink.


- `#4181 <https://github.com/pytest-dev/pytest/issues/4181>`_: Handle race condition between creation and deletion of temporary folders.


- `#4221 <https://github.com/pytest-dev/pytest/issues/4221>`_: Fix bug where the warning summary at the end of the test session was not showing the test where the warning was originated.


- `#4243 <https://github.com/pytest-dev/pytest/issues/4243>`_: Fix regression when ``stacklevel`` for warnings was passed as positional argument on python2.



Improved Documentation
----------------------

- `#3851 <https://github.com/pytest-dev/pytest/issues/3851>`_: Add reference to ``empty_parameter_set_mark`` ini option in documentation of ``@pytest.mark.parametrize``



Trivial/Internal Changes
------------------------

- `#4028 <https://github.com/pytest-dev/pytest/issues/4028>`_: Revert patching of ``sys.breakpointhook`` since it appears to do nothing.


- `#4233 <https://github.com/pytest-dev/pytest/issues/4233>`_: Apply an import sorter (``reorder-python-imports``) to the codebase.


- `#4248 <https://github.com/pytest-dev/pytest/issues/4248>`_: Remove use of unnecessary compat shim, six.binary_type


pytest 3.9.2 (2018-10-22)
=========================

Bug Fixes
---------

- `#2909 <https://github.com/pytest-dev/pytest/issues/2909>`_: Improve error message when a recursive dependency between fixtures is detected.


- `#3340 <https://github.com/pytest-dev/pytest/issues/3340>`_: Fix logging messages not shown in hooks ``pytest_sessionstart()`` and ``pytest_sessionfinish()``.


- `#3533 <https://github.com/pytest-dev/pytest/issues/3533>`_: Fix unescaped XML raw objects in JUnit report for skipped tests


- `#3691 <https://github.com/pytest-dev/pytest/issues/3691>`_: Python 2: safely format warning message about passing unicode strings to ``warnings.warn``, which may cause
  surprising ``MemoryError`` exception when monkey patching ``warnings.warn`` itself.


- `#4026 <https://github.com/pytest-dev/pytest/issues/4026>`_: Improve error message when it is not possible to determine a function's signature.


- `#4177 <https://github.com/pytest-dev/pytest/issues/4177>`_: Pin ``setuptools>=40.0`` to support ``py_modules`` in ``setup.cfg``


- `#4179 <https://github.com/pytest-dev/pytest/issues/4179>`_: Restore the tmpdir behaviour of symlinking the current test run.


- `#4192 <https://github.com/pytest-dev/pytest/issues/4192>`_: Fix filename reported by ``warnings.warn`` when using ``recwarn`` under python2.


pytest 3.9.1 (2018-10-16)
=========================

Features
--------

- `#4159 <https://github.com/pytest-dev/pytest/issues/4159>`_: For test-suites containing test classes, the information about the subclassed
  module is now output only if a higher verbosity level is specified (at least
  "-vv").


pytest 3.9.0 (2018-10-15 - not published due to a release automation bug)
=========================================================================

Deprecations
------------

- `#3616 <https://github.com/pytest-dev/pytest/issues/3616>`_: The following accesses have been documented as deprecated for years, but are now actually emitting deprecation warnings.

  * Access of ``Module``, ``Function``, ``Class``, ``Instance``, ``File`` and ``Item`` through ``Node`` instances. Now
    users will this warning::

          usage of Function.Module is deprecated, please use pytest.Module instead

    Users should just ``import pytest`` and access those objects using the ``pytest`` module.

  * ``request.cached_setup``, this was the precursor of the setup/teardown mechanism available to fixtures. You can
    consult `funcarg comparison section in the docs <https://docs.pytest.org/en/latest/funcarg_compare.html>`_.

  * Using objects named ``"Class"`` as a way to customize the type of nodes that are collected in ``Collector``
    subclasses has been deprecated. Users instead should use ``pytest_collect_make_item`` to customize node types during
    collection.

    This issue should affect only advanced plugins who create new collection types, so if you see this warning
    message please contact the authors so they can change the code.

  * The warning that produces the message below has changed to ``RemovedInPytest4Warning``::

          getfuncargvalue is deprecated, use getfixturevalue


- `#3988 <https://github.com/pytest-dev/pytest/issues/3988>`_: Add a Deprecation warning for pytest.ensuretemp as it was deprecated since a while.



Features
--------

- `#2293 <https://github.com/pytest-dev/pytest/issues/2293>`_: Improve usage errors messages by hiding internal details which can be distracting and noisy.

  This has the side effect that some error conditions that previously raised generic errors (such as
  ``ValueError`` for unregistered marks) are now raising ``Failed`` exceptions.


- `#3332 <https://github.com/pytest-dev/pytest/issues/3332>`_: Improve the error displayed when a ``conftest.py`` file could not be imported.

  In order to implement this, a new ``chain`` parameter was added to ``ExceptionInfo.getrepr``
  to show or hide chained tracebacks in Python 3 (defaults to ``True``).


- `#3849 <https://github.com/pytest-dev/pytest/issues/3849>`_: Add ``empty_parameter_set_mark=fail_at_collect`` ini option for raising an exception when parametrize collects an empty set.


- `#3964 <https://github.com/pytest-dev/pytest/issues/3964>`_: Log messages generated in the collection phase are shown when
  live-logging is enabled and/or when they are logged to a file.


- `#3985 <https://github.com/pytest-dev/pytest/issues/3985>`_: Introduce ``tmp_path`` as a fixture providing a Path object.


- `#4013 <https://github.com/pytest-dev/pytest/issues/4013>`_: Deprecation warnings are now shown even if you customize the warnings filters yourself. In the previous version
  any customization would override pytest's filters and deprecation warnings would fall back to being hidden by default.


- `#4073 <https://github.com/pytest-dev/pytest/issues/4073>`_: Allow specification of timeout for ``Testdir.runpytest_subprocess()`` and ``Testdir.run()``.


- `#4098 <https://github.com/pytest-dev/pytest/issues/4098>`_: Add returncode argument to pytest.exit() to exit pytest with a specific return code.


- `#4102 <https://github.com/pytest-dev/pytest/issues/4102>`_: Reimplement ``pytest.deprecated_call`` using ``pytest.warns`` so it supports the ``match='...'`` keyword argument.

  This has the side effect that ``pytest.deprecated_call`` now raises ``pytest.fail.Exception`` instead
  of ``AssertionError``.


- `#4149 <https://github.com/pytest-dev/pytest/issues/4149>`_: Require setuptools>=30.3 and move most of the metadata to ``setup.cfg``.



Bug Fixes
---------

- `#2535 <https://github.com/pytest-dev/pytest/issues/2535>`_: Improve error message when test functions of ``unittest.TestCase`` subclasses use a parametrized fixture.


- `#3057 <https://github.com/pytest-dev/pytest/issues/3057>`_: ``request.fixturenames`` now correctly returns the name of fixtures created by ``request.getfixturevalue()``.


- `#3946 <https://github.com/pytest-dev/pytest/issues/3946>`_: Warning filters passed as command line options using ``-W`` now take precedence over filters defined in ``ini``
  configuration files.


- `#4066 <https://github.com/pytest-dev/pytest/issues/4066>`_: Fix source reindenting by using ``textwrap.dedent`` directly.


- `#4102 <https://github.com/pytest-dev/pytest/issues/4102>`_: ``pytest.warn`` will capture previously-warned warnings in Python 2. Previously they were never raised.


- `#4108 <https://github.com/pytest-dev/pytest/issues/4108>`_: Resolve symbolic links for args.

  This fixes running ``pytest tests/test_foo.py::test_bar``, where ``tests``
  is a symlink to ``project/app/tests``:
  previously ``project/app/conftest.py`` would be ignored for fixtures then.


- `#4132 <https://github.com/pytest-dev/pytest/issues/4132>`_: Fix duplicate printing of internal errors when using ``--pdb``.


- `#4135 <https://github.com/pytest-dev/pytest/issues/4135>`_: pathlib based tmpdir cleanup now correctly handles symlinks in the folder.


- `#4152 <https://github.com/pytest-dev/pytest/issues/4152>`_: Display the filename when encountering ``SyntaxWarning``.



Improved Documentation
----------------------

- `#3713 <https://github.com/pytest-dev/pytest/issues/3713>`_: Update usefixtures documentation to clarify that it can't be used with fixture functions.


- `#4058 <https://github.com/pytest-dev/pytest/issues/4058>`_: Update fixture documentation to specify that a fixture can be invoked twice in the scope it's defined for.


- `#4064 <https://github.com/pytest-dev/pytest/issues/4064>`_: According to unittest.rst, setUpModule and tearDownModule were not implemented, but it turns out they are. So updated the documentation for unittest.


- `#4151 <https://github.com/pytest-dev/pytest/issues/4151>`_: Add tempir testing example to CONTRIBUTING.rst guide



Trivial/Internal Changes
------------------------

- `#2293 <https://github.com/pytest-dev/pytest/issues/2293>`_: The internal ``MarkerError`` exception has been removed.


- `#3988 <https://github.com/pytest-dev/pytest/issues/3988>`_: Port the implementation of tmpdir to pathlib.


- `#4063 <https://github.com/pytest-dev/pytest/issues/4063>`_: Exclude 0.00 second entries from ``--duration`` output unless ``-vv`` is passed on the command-line.


- `#4093 <https://github.com/pytest-dev/pytest/issues/4093>`_: Fixed formatting of string literals in internal tests.


pytest 3.8.2 (2018-10-02)
=========================

Deprecations and Removals
-------------------------

- `#4036 <https://github.com/pytest-dev/pytest/issues/4036>`_: The ``item`` parameter of ``pytest_warning_captured`` hook is now documented as deprecated. We realized only after
  the ``3.8`` release that this parameter is incompatible with ``pytest-xdist``.

  Our policy is to not deprecate features during bugfix releases, but in this case we believe it makes sense as we are
  only documenting it as deprecated, without issuing warnings which might potentially break test suites. This will get
  the word out that hook implementers should not use this parameter at all.

  In a future release ``item`` will always be ``None`` and will emit a proper warning when a hook implementation
  makes use of it.



Bug Fixes
---------

- `#3539 <https://github.com/pytest-dev/pytest/issues/3539>`_: Fix reload on assertion rewritten modules.


- `#4034 <https://github.com/pytest-dev/pytest/issues/4034>`_: The ``.user_properties`` attribute of ``TestReport`` objects is a list
  of (name, value) tuples, but could sometimes be instantiated as a tuple
  of tuples.  It is now always a list.


- `#4039 <https://github.com/pytest-dev/pytest/issues/4039>`_: No longer issue warnings about using ``pytest_plugins`` in non-top-level directories when using ``--pyargs``: the
  current ``--pyargs`` mechanism is not reliable and might give false negatives.


- `#4040 <https://github.com/pytest-dev/pytest/issues/4040>`_: Exclude empty reports for passed tests when ``-rP`` option is used.


- `#4051 <https://github.com/pytest-dev/pytest/issues/4051>`_: Improve error message when an invalid Python expression is passed to the ``-m`` option.


- `#4056 <https://github.com/pytest-dev/pytest/issues/4056>`_: ``MonkeyPatch.setenv`` and ``MonkeyPatch.delenv`` issue a warning if the environment variable name is not ``str`` on Python 2.

  In Python 2, adding ``unicode`` keys to ``os.environ`` causes problems with ``subprocess`` (and possible other modules),
  making this a subtle bug specially susceptible when used with ``from __future__ import unicode_literals``.



Improved Documentation
----------------------

- `#3928 <https://github.com/pytest-dev/pytest/issues/3928>`_: Add possible values for fixture scope to docs.


pytest 3.8.1 (2018-09-22)
=========================

Bug Fixes
---------

- `#3286 <https://github.com/pytest-dev/pytest/issues/3286>`_: ``.pytest_cache`` directory is now automatically ignored by Git. Users who would like to contribute a solution for other SCMs please consult/comment on this issue.


- `#3749 <https://github.com/pytest-dev/pytest/issues/3749>`_: Fix the following error during collection of tests inside packages::

      TypeError: object of type 'Package' has no len()


- `#3941 <https://github.com/pytest-dev/pytest/issues/3941>`_: Fix bug where indirect parametrization would consider the scope of all fixtures used by the test function to determine the parametrization scope, and not only the scope of the fixtures being parametrized.


- `#3973 <https://github.com/pytest-dev/pytest/issues/3973>`_: Fix crash of the assertion rewriter if a test changed the current working directory without restoring it afterwards.


- `#3998 <https://github.com/pytest-dev/pytest/issues/3998>`_: Fix issue that prevented some caplog properties (for example ``record_tuples``) from being available when entering the debugger with ``--pdb``.


- `#3999 <https://github.com/pytest-dev/pytest/issues/3999>`_: Fix ``UnicodeDecodeError`` in python2.x when a class returns a non-ascii binary ``__repr__`` in an assertion which also contains non-ascii text.



Improved Documentation
----------------------

- `#3996 <https://github.com/pytest-dev/pytest/issues/3996>`_: New `Deprecations and Removals <https://docs.pytest.org/en/latest/deprecations.html>`_ page shows all currently
  deprecated features, the rationale to do so, and alternatives to update your code. It also list features removed
  from pytest in past major releases to help those with ancient pytest versions to upgrade.



Trivial/Internal Changes
------------------------

- `#3955 <https://github.com/pytest-dev/pytest/issues/3955>`_: Improve pre-commit detection for changelog filenames


- `#3975 <https://github.com/pytest-dev/pytest/issues/3975>`_: Remove legacy code around im_func as that was python2 only


pytest 3.8.0 (2018-09-05)
=========================

Deprecations and Removals
-------------------------

- `#2452 <https://github.com/pytest-dev/pytest/issues/2452>`_: ``Config.warn`` and ``Node.warn`` have been
  deprecated, see `<https://docs.pytest.org/en/latest/deprecations.html#config-warn-and-node-warn>`_ for rationale and
  examples.

- `#3936 <https://github.com/pytest-dev/pytest/issues/3936>`_: ``@pytest.mark.filterwarnings`` second parameter is no longer regex-escaped,
  making it possible to actually use regular expressions to check the warning message.

  **Note**: regex-escaping the match string was an implementation oversight that might break test suites which depend
  on the old behavior.



Features
--------

- `#2452 <https://github.com/pytest-dev/pytest/issues/2452>`_: Internal pytest warnings are now issued using the standard ``warnings`` module, making it possible to use
  the standard warnings filters to manage those warnings. This introduces ``PytestWarning``,
  ``PytestDeprecationWarning`` and ``RemovedInPytest4Warning`` warning types as part of the public API.

  Consult `the documentation <https://docs.pytest.org/en/latest/warnings.html#internal-pytest-warnings>`__ for more info.


- `#2908 <https://github.com/pytest-dev/pytest/issues/2908>`_: ``DeprecationWarning`` and ``PendingDeprecationWarning`` are now shown by default if no other warning filter is
  configured. This makes pytest more compliant with
  `PEP-0506 <https://www.python.org/dev/peps/pep-0565/#recommended-filter-settings-for-test-runners>`_. See
  `the docs <https://docs.pytest.org/en/latest/warnings.html#deprecationwarning-and-pendingdeprecationwarning>`_ for
  more info.


- `#3251 <https://github.com/pytest-dev/pytest/issues/3251>`_: Warnings are now captured and displayed during test collection.


- `#3784 <https://github.com/pytest-dev/pytest/issues/3784>`_: ``PYTEST_DISABLE_PLUGIN_AUTOLOAD`` environment variable disables plugin auto-loading when set.


- `#3829 <https://github.com/pytest-dev/pytest/issues/3829>`_: Added the ``count`` option to ``console_output_style`` to enable displaying the progress as a count instead of a percentage.


- `#3837 <https://github.com/pytest-dev/pytest/issues/3837>`_: Added support for 'xfailed' and 'xpassed' outcomes to the ``pytester.RunResult.assert_outcomes`` signature.



Bug Fixes
---------

- `#3911 <https://github.com/pytest-dev/pytest/issues/3911>`_: Terminal writer now takes into account unicode character width when writing out progress.


- `#3913 <https://github.com/pytest-dev/pytest/issues/3913>`_: Pytest now returns with correct exit code (EXIT_USAGEERROR, 4) when called with unknown arguments.


- `#3918 <https://github.com/pytest-dev/pytest/issues/3918>`_: Improve performance of assertion rewriting.



Improved Documentation
----------------------

- `#3566 <https://github.com/pytest-dev/pytest/issues/3566>`_: Added a blurb in usage.rst for the usage of -r flag which is used to show an extra test summary info.


- `#3907 <https://github.com/pytest-dev/pytest/issues/3907>`_: Corrected type of the exceptions collection passed to ``xfail``: ``raises`` argument accepts a ``tuple`` instead of ``list``.



Trivial/Internal Changes
------------------------

- `#3853 <https://github.com/pytest-dev/pytest/issues/3853>`_: Removed ``"run all (no recorded failures)"`` message printed with ``--failed-first`` and ``--last-failed`` when there are no failed tests.


pytest 3.7.4 (2018-08-29)
=========================

Bug Fixes
---------

- `#3506 <https://github.com/pytest-dev/pytest/issues/3506>`_: Fix possible infinite recursion when writing ``.pyc`` files.


- `#3853 <https://github.com/pytest-dev/pytest/issues/3853>`_: Cache plugin now obeys the ``-q`` flag when ``--last-failed`` and ``--failed-first`` flags are used.


- `#3883 <https://github.com/pytest-dev/pytest/issues/3883>`_: Fix bad console output when using ``console_output_style=classic``.


- `#3888 <https://github.com/pytest-dev/pytest/issues/3888>`_: Fix macOS specific code using ``capturemanager`` plugin in doctests.



Improved Documentation
----------------------

- `#3902 <https://github.com/pytest-dev/pytest/issues/3902>`_: Fix pytest.org links


pytest 3.7.3 (2018-08-26)
=========================

Bug Fixes
---------

- `#3033 <https://github.com/pytest-dev/pytest/issues/3033>`_: Fixtures during teardown can again use ``capsys`` and ``capfd`` to inspect output captured during tests.


- `#3773 <https://github.com/pytest-dev/pytest/issues/3773>`_: Fix collection of tests from ``__init__.py`` files if they match the ``python_files`` configuration option.


- `#3796 <https://github.com/pytest-dev/pytest/issues/3796>`_: Fix issue where teardown of fixtures of consecutive sub-packages were executed once, at the end of the outer
  package.


- `#3816 <https://github.com/pytest-dev/pytest/issues/3816>`_: Fix bug where ``--show-capture=no`` option would still show logs printed during fixture teardown.


- `#3819 <https://github.com/pytest-dev/pytest/issues/3819>`_: Fix ``stdout/stderr`` not getting captured when real-time cli logging is active.


- `#3843 <https://github.com/pytest-dev/pytest/issues/3843>`_: Fix collection error when specifying test functions directly in the command line using ``test.py::test`` syntax together with ``--doctest-modules``.


- `#3848 <https://github.com/pytest-dev/pytest/issues/3848>`_: Fix bugs where unicode arguments could not be passed to ``testdir.runpytest`` on Python 2.


- `#3854 <https://github.com/pytest-dev/pytest/issues/3854>`_: Fix double collection of tests within packages when the filename starts with a capital letter.



Improved Documentation
----------------------

- `#3824 <https://github.com/pytest-dev/pytest/issues/3824>`_: Added example for multiple glob pattern matches in ``python_files``.


- `#3833 <https://github.com/pytest-dev/pytest/issues/3833>`_: Added missing docs for ``pytester.Testdir``.


- `#3870 <https://github.com/pytest-dev/pytest/issues/3870>`_: Correct documentation for setuptools integration.



Trivial/Internal Changes
------------------------

- `#3826 <https://github.com/pytest-dev/pytest/issues/3826>`_: Replace broken type annotations with type comments.


- `#3845 <https://github.com/pytest-dev/pytest/issues/3845>`_: Remove a reference to issue `#568 <https://github.com/pytest-dev/pytest/issues/568>`_ from the documentation, which has since been
  fixed.


pytest 3.7.2 (2018-08-16)
=========================

Bug Fixes
---------

- `#3671 <https://github.com/pytest-dev/pytest/issues/3671>`_: Fix ``filterwarnings`` not being registered as a builtin mark.


- `#3768 <https://github.com/pytest-dev/pytest/issues/3768>`_, `#3789 <https://github.com/pytest-dev/pytest/issues/3789>`_: Fix test collection from packages mixed with normal directories.


- `#3771 <https://github.com/pytest-dev/pytest/issues/3771>`_: Fix infinite recursion during collection if a ``pytest_ignore_collect`` hook returns ``False`` instead of ``None``.


- `#3774 <https://github.com/pytest-dev/pytest/issues/3774>`_: Fix bug where decorated fixtures would lose functionality (for example ``@mock.patch``).


- `#3775 <https://github.com/pytest-dev/pytest/issues/3775>`_: Fix bug where importing modules or other objects with prefix ``pytest_`` prefix would raise a ``PluginValidationError``.


- `#3788 <https://github.com/pytest-dev/pytest/issues/3788>`_: Fix ``AttributeError`` during teardown of ``TestCase`` subclasses which raise an exception during ``__init__``.


- `#3804 <https://github.com/pytest-dev/pytest/issues/3804>`_: Fix traceback reporting for exceptions with ``__cause__`` cycles.



Improved Documentation
----------------------

- `#3746 <https://github.com/pytest-dev/pytest/issues/3746>`_: Add documentation for ``metafunc.config`` that had been mistakenly hidden.


pytest 3.7.1 (2018-08-02)
=========================

Bug Fixes
---------

- `#3473 <https://github.com/pytest-dev/pytest/issues/3473>`_: Raise immediately if ``approx()`` is given an expected value of a type it doesn't understand (e.g. strings, nested dicts, etc.).


- `#3712 <https://github.com/pytest-dev/pytest/issues/3712>`_: Correctly represent the dimensions of a numpy array when calling ``repr()`` on ``approx()``.

- `#3742 <https://github.com/pytest-dev/pytest/issues/3742>`_: Fix incompatibility with third party plugins during collection, which produced the error ``object has no attribute '_collectfile'``.

- `#3745 <https://github.com/pytest-dev/pytest/issues/3745>`_: Display the absolute path if ``cache_dir`` is not relative to the ``rootdir`` instead of failing.


- `#3747 <https://github.com/pytest-dev/pytest/issues/3747>`_: Fix compatibility problem with plugins and the warning code issued by fixture functions when they are called directly.


- `#3748 <https://github.com/pytest-dev/pytest/issues/3748>`_: Fix infinite recursion in ``pytest.approx`` with arrays in ``numpy<1.13``.


- `#3757 <https://github.com/pytest-dev/pytest/issues/3757>`_: Pin pathlib2 to ``>=2.2.0`` as we require ``__fspath__`` support.


- `#3763 <https://github.com/pytest-dev/pytest/issues/3763>`_: Fix ``TypeError`` when the assertion message is ``bytes`` in python 3.


pytest 3.7.0 (2018-07-30)
=========================

Deprecations and Removals
-------------------------

- `#2639 <https://github.com/pytest-dev/pytest/issues/2639>`_: ``pytest_namespace`` has been `deprecated <https://docs.pytest.org/en/latest/deprecations.html#pytest-namespace>`_.


- `#3661 <https://github.com/pytest-dev/pytest/issues/3661>`_: Calling a fixture function directly, as opposed to request them in a test function, now issues a ``RemovedInPytest4Warning``. See `the documentation for rationale and examples <https://docs.pytest.org/en/latest/deprecations.html#calling-fixtures-directly>`_.



Features
--------

- `#2283 <https://github.com/pytest-dev/pytest/issues/2283>`_: New ``package`` fixture scope: fixtures are finalized when the last test of a *package* finishes. This feature is considered **experimental**, so use it sparingly.


- `#3576 <https://github.com/pytest-dev/pytest/issues/3576>`_: ``Node.add_marker`` now supports an ``append=True/False`` parameter to determine whether the mark comes last (default) or first.


- `#3579 <https://github.com/pytest-dev/pytest/issues/3579>`_: Fixture ``caplog`` now has a ``messages`` property, providing convenient access to the format-interpolated log messages without the extra data provided by the formatter/handler.


- `#3610 <https://github.com/pytest-dev/pytest/issues/3610>`_: New ``--trace`` option to enter the debugger at the start of a test.


- `#3623 <https://github.com/pytest-dev/pytest/issues/3623>`_: Introduce ``pytester.copy_example`` as helper to do acceptance tests against examples from the project.



Bug Fixes
---------

- `#2220 <https://github.com/pytest-dev/pytest/issues/2220>`_: Fix a bug where fixtures overridden by direct parameters (for example parametrization) were being instantiated even if they were not being used by a test.


- `#3695 <https://github.com/pytest-dev/pytest/issues/3695>`_: Fix ``ApproxNumpy`` initialisation argument mixup, ``abs`` and ``rel`` tolerances were flipped causing strange comparison results.
  Add tests to check ``abs`` and ``rel`` tolerances for ``np.array`` and test for expecting ``nan`` with ``np.array()``


- `#980 <https://github.com/pytest-dev/pytest/issues/980>`_: Fix truncated locals output in verbose mode.



Improved Documentation
----------------------

- `#3295 <https://github.com/pytest-dev/pytest/issues/3295>`_: Correct the usage documentation of ``--last-failed-no-failures`` by adding the missing ``--last-failed`` argument in the presented examples, because they are misleading and lead to think that the missing argument is not needed.



Trivial/Internal Changes
------------------------

- `#3519 <https://github.com/pytest-dev/pytest/issues/3519>`_: Now a ``README.md`` file is created in ``.pytest_cache`` to make it clear why the directory exists.


pytest 3.6.4 (2018-07-28)
=========================

Bug Fixes
---------

- Invoke pytest using ``-mpytest`` so ``sys.path`` does not get polluted by packages installed in ``site-packages``. (`#742 <https://github.com/pytest-dev/pytest/issues/742>`_)


Improved Documentation
----------------------

- Use ``smtp_connection`` instead of ``smtp`` in fixtures documentation to avoid possible confusion. (`#3592 <https://github.com/pytest-dev/pytest/issues/3592>`_)


Trivial/Internal Changes
------------------------

- Remove obsolete ``__future__`` imports. (`#2319 <https://github.com/pytest-dev/pytest/issues/2319>`_)

- Add CITATION to provide information on how to formally cite pytest. (`#3402 <https://github.com/pytest-dev/pytest/issues/3402>`_)

- Replace broken type annotations with type comments. (`#3635 <https://github.com/pytest-dev/pytest/issues/3635>`_)

- Pin ``pluggy`` to ``<0.8``. (`#3727 <https://github.com/pytest-dev/pytest/issues/3727>`_)


pytest 3.6.3 (2018-07-04)
=========================

Bug Fixes
---------

- Fix ``ImportWarning`` triggered by explicit relative imports in
  assertion-rewritten package modules. (`#3061
  <https://github.com/pytest-dev/pytest/issues/3061>`_)

- Fix error in ``pytest.approx`` when dealing with 0-dimension numpy
  arrays. (`#3593 <https://github.com/pytest-dev/pytest/issues/3593>`_)

- No longer raise ``ValueError`` when using the ``get_marker`` API. (`#3605
  <https://github.com/pytest-dev/pytest/issues/3605>`_)

- Fix problem where log messages with non-ascii characters would not
  appear in the output log file.
  (`#3630 <https://github.com/pytest-dev/pytest/issues/3630>`_)

- No longer raise ``AttributeError`` when legacy marks can't be stored in
  functions. (`#3631 <https://github.com/pytest-dev/pytest/issues/3631>`_)


Improved Documentation
----------------------

- The description above the example for ``@pytest.mark.skipif`` now better
  matches the code. (`#3611
  <https://github.com/pytest-dev/pytest/issues/3611>`_)


Trivial/Internal Changes
------------------------

- Internal refactoring: removed unused ``CallSpec2tox ._globalid_args``
  attribute and ``metafunc`` parameter from ``CallSpec2.copy()``. (`#3598
  <https://github.com/pytest-dev/pytest/issues/3598>`_)

- Silence usage of ``reduce`` warning in Python 2 (`#3609
  <https://github.com/pytest-dev/pytest/issues/3609>`_)

- Fix usage of ``attr.ib`` deprecated ``convert`` parameter. (`#3653
  <https://github.com/pytest-dev/pytest/issues/3653>`_)


pytest 3.6.2 (2018-06-20)
=========================

Bug Fixes
---------

- Fix regression in ``Node.add_marker`` by extracting the mark object of a
  ``MarkDecorator``. (`#3555
  <https://github.com/pytest-dev/pytest/issues/3555>`_)

- Warnings without ``location`` were reported as ``None``. This is corrected to
  now report ``<undetermined location>``. (`#3563
  <https://github.com/pytest-dev/pytest/issues/3563>`_)

- Continue to call finalizers in the stack when a finalizer in a former scope
  raises an exception. (`#3569
  <https://github.com/pytest-dev/pytest/issues/3569>`_)

- Fix encoding error with ``print`` statements in doctests (`#3583
  <https://github.com/pytest-dev/pytest/issues/3583>`_)


Improved Documentation
----------------------

- Add documentation for the ``--strict`` flag. (`#3549
  <https://github.com/pytest-dev/pytest/issues/3549>`_)


Trivial/Internal Changes
------------------------

- Update old quotation style to parens in fixture.rst documentation. (`#3525
  <https://github.com/pytest-dev/pytest/issues/3525>`_)

- Improve display of hint about ``--fulltrace`` with ``KeyboardInterrupt``.
  (`#3545 <https://github.com/pytest-dev/pytest/issues/3545>`_)

- pytest's testsuite is no longer runnable through ``python setup.py test`` --
  instead invoke ``pytest`` or ``tox`` directly. (`#3552
  <https://github.com/pytest-dev/pytest/issues/3552>`_)

- Fix typo in documentation (`#3567
  <https://github.com/pytest-dev/pytest/issues/3567>`_)


pytest 3.6.1 (2018-06-05)
=========================

Bug Fixes
---------

- Fixed a bug where stdout and stderr were logged twice by junitxml when a test
  was marked xfail. (`#3491
  <https://github.com/pytest-dev/pytest/issues/3491>`_)

- Fix ``usefixtures`` mark applyed to unittest tests by correctly instantiating
  ``FixtureInfo``. (`#3498
  <https://github.com/pytest-dev/pytest/issues/3498>`_)

- Fix assertion rewriter compatibility with libraries that monkey patch
  ``file`` objects. (`#3503
  <https://github.com/pytest-dev/pytest/issues/3503>`_)


Improved Documentation
----------------------

- Added a section on how to use fixtures as factories to the fixture
  documentation. (`#3461 <https://github.com/pytest-dev/pytest/issues/3461>`_)


Trivial/Internal Changes
------------------------

- Enable caching for pip/pre-commit in order to reduce build time on
  travis/appveyor. (`#3502
  <https://github.com/pytest-dev/pytest/issues/3502>`_)

- Switch pytest to the src/ layout as we already suggested it for good practice
  - now we implement it as well. (`#3513
  <https://github.com/pytest-dev/pytest/issues/3513>`_)

- Fix if in tests to support 3.7.0b5, where a docstring handling in AST got
  reverted. (`#3530 <https://github.com/pytest-dev/pytest/issues/3530>`_)

- Remove some python2.5 compatibility code. (`#3529
  <https://github.com/pytest-dev/pytest/issues/3529>`_)


pytest 3.6.0 (2018-05-23)
=========================

Features
--------

- Revamp the internals of the ``pytest.mark`` implementation with correct per
  node handling which fixes a number of long standing bugs caused by the old
  design. This introduces new ``Node.iter_markers(name)`` and
  ``Node.get_closest_marker(name)`` APIs. Users are **strongly encouraged** to
  read the `reasons for the revamp in the docs
  <https://docs.pytest.org/en/latest/mark.html#marker-revamp-and-iteration>`_,
  or jump over to details about `updating existing code to use the new APIs
  <https://docs.pytest.org/en/latest/mark.html#updating-code>`_. (`#3317
  <https://github.com/pytest-dev/pytest/issues/3317>`_)

- Now when ``@pytest.fixture`` is applied more than once to the same function a
  ``ValueError`` is raised. This buggy behavior would cause surprising problems
  and if was working for a test suite it was mostly by accident. (`#2334
  <https://github.com/pytest-dev/pytest/issues/2334>`_)

- Support for Python 3.7's builtin ``breakpoint()`` method, see `Using the
  builtin breakpoint function
  <https://docs.pytest.org/en/latest/usage.html#breakpoint-builtin>`_ for
  details. (`#3180 <https://github.com/pytest-dev/pytest/issues/3180>`_)

- ``monkeypatch`` now supports a ``context()`` function which acts as a context
  manager which undoes all patching done within the ``with`` block. (`#3290
  <https://github.com/pytest-dev/pytest/issues/3290>`_)

- The ``--pdb`` option now causes KeyboardInterrupt to enter the debugger,
  instead of stopping the test session. On python 2.7, hitting CTRL+C again
  exits the debugger. On python 3.2 and higher, use CTRL+D. (`#3299
  <https://github.com/pytest-dev/pytest/issues/3299>`_)

- pytest no longer changes the log level of the root logger when the
  ``log-level`` parameter has greater numeric value than that of the level of
  the root logger, which makes it play better with custom logging configuration
  in user code. (`#3307 <https://github.com/pytest-dev/pytest/issues/3307>`_)


Bug Fixes
---------

- A rare race-condition which might result in corrupted ``.pyc`` files on
  Windows has been hopefully solved. (`#3008
  <https://github.com/pytest-dev/pytest/issues/3008>`_)

- Also use iter_marker for discovering the marks applying for marker
  expressions from the cli to avoid the bad data from the legacy mark storage.
  (`#3441 <https://github.com/pytest-dev/pytest/issues/3441>`_)

- When showing diffs of failed assertions where the contents contain only
  whitespace, escape them using ``repr()`` first to make it easy to spot the
  differences. (`#3443 <https://github.com/pytest-dev/pytest/issues/3443>`_)


Improved Documentation
----------------------

- Change documentation copyright year to a range which auto-updates itself each
  time it is published. (`#3303
  <https://github.com/pytest-dev/pytest/issues/3303>`_)


Trivial/Internal Changes
------------------------

- ``pytest`` now depends on the `python-atomicwrites
  <https://github.com/untitaker/python-atomicwrites>`_ library. (`#3008
  <https://github.com/pytest-dev/pytest/issues/3008>`_)

- Update all pypi.python.org URLs to pypi.org. (`#3431
  <https://github.com/pytest-dev/pytest/issues/3431>`_)

- Detect `pytest_` prefixed hooks using the internal plugin manager since
  ``pluggy`` is deprecating the ``implprefix`` argument to ``PluginManager``.
  (`#3487 <https://github.com/pytest-dev/pytest/issues/3487>`_)

- Import ``Mapping`` and ``Sequence`` from ``_pytest.compat`` instead of
  directly from ``collections`` in ``python_api.py::approx``. Add ``Mapping``
  to ``_pytest.compat``, import it from ``collections`` on python 2, but from
  ``collections.abc`` on Python 3 to avoid a ``DeprecationWarning`` on Python
  3.7 or newer. (`#3497 <https://github.com/pytest-dev/pytest/issues/3497>`_)


pytest 3.5.1 (2018-04-23)
=========================


Bug Fixes
---------

- Reset ``sys.last_type``, ``sys.last_value`` and ``sys.last_traceback`` before
  each test executes. Those attributes are added by pytest during the test run
  to aid debugging, but were never reset so they would create a leaking
  reference to the last failing test's frame which in turn could never be
  reclaimed by the garbage collector. (`#2798
  <https://github.com/pytest-dev/pytest/issues/2798>`_)

- ``pytest.raises`` now raises ``TypeError`` when receiving an unknown keyword
  argument. (`#3348 <https://github.com/pytest-dev/pytest/issues/3348>`_)

- ``pytest.raises`` now works with exception classes that look like iterables.
  (`#3372 <https://github.com/pytest-dev/pytest/issues/3372>`_)


Improved Documentation
----------------------

- Fix typo in ``caplog`` fixture documentation, which incorrectly identified
  certain attributes as methods. (`#3406
  <https://github.com/pytest-dev/pytest/issues/3406>`_)


Trivial/Internal Changes
------------------------

- Added a more indicative error message when parametrizing a function whose
  argument takes a default value. (`#3221
  <https://github.com/pytest-dev/pytest/issues/3221>`_)

- Remove internal ``_pytest.terminal.flatten`` function in favor of
  ``more_itertools.collapse``. (`#3330
  <https://github.com/pytest-dev/pytest/issues/3330>`_)

- Import some modules from ``collections.abc`` instead of ``collections`` as
  the former modules trigger ``DeprecationWarning`` in Python 3.7. (`#3339
  <https://github.com/pytest-dev/pytest/issues/3339>`_)

- record_property is no longer experimental, removing the warnings was
  forgotten. (`#3360 <https://github.com/pytest-dev/pytest/issues/3360>`_)

- Mention in documentation and CLI help that fixtures with leading ``_`` are
  printed by ``pytest --fixtures`` only if the ``-v`` option is added. (`#3398
  <https://github.com/pytest-dev/pytest/issues/3398>`_)


pytest 3.5.0 (2018-03-21)
=========================

Deprecations and Removals
-------------------------

- ``record_xml_property`` fixture is now deprecated in favor of the more
  generic ``record_property``. (`#2770
  <https://github.com/pytest-dev/pytest/issues/2770>`_)

- Defining ``pytest_plugins`` is now deprecated in non-top-level conftest.py
  files, because they "leak" to the entire directory tree. `See the docs <https://docs.pytest.org/en/latest/deprecations.html#pytest-plugins-in-non-top-level-conftest-files>`_ for the rationale behind this decision (`#3084
  <https://github.com/pytest-dev/pytest/issues/3084>`_)


Features
--------

- New ``--show-capture`` command-line option that allows to specify how to
  display captured output when tests fail: ``no``, ``stdout``, ``stderr``,
  ``log`` or ``all`` (the default). (`#1478
  <https://github.com/pytest-dev/pytest/issues/1478>`_)

- New ``--rootdir`` command-line option to override the rules for discovering
  the root directory. See `customize
  <https://docs.pytest.org/en/latest/customize.html>`_ in the documentation for
  details. (`#1642 <https://github.com/pytest-dev/pytest/issues/1642>`_)

- Fixtures are now instantiated based on their scopes, with higher-scoped
  fixtures (such as ``session``) being instantiated first than lower-scoped
  fixtures (such as ``function``). The relative order of fixtures of the same
  scope is kept unchanged, based in their declaration order and their
  dependencies. (`#2405 <https://github.com/pytest-dev/pytest/issues/2405>`_)

- ``record_xml_property`` renamed to ``record_property`` and is now compatible
  with xdist, markers and any reporter. ``record_xml_property`` name is now
  deprecated. (`#2770 <https://github.com/pytest-dev/pytest/issues/2770>`_)

- New ``--nf``, ``--new-first`` options: run new tests first followed by the
  rest of the tests, in both cases tests are also sorted by the file modified
  time, with more recent files coming first. (`#3034
  <https://github.com/pytest-dev/pytest/issues/3034>`_)

- New ``--last-failed-no-failures`` command-line option that allows to specify
  the behavior of the cache plugin's ```--last-failed`` feature when no tests
  failed in the last run (or no cache was found): ``none`` or ``all`` (the
  default). (`#3139 <https://github.com/pytest-dev/pytest/issues/3139>`_)

- New ``--doctest-continue-on-failure`` command-line option to enable doctests
  to show multiple failures for each snippet, instead of stopping at the first
  failure. (`#3149 <https://github.com/pytest-dev/pytest/issues/3149>`_)

- Captured log messages are added to the ``<system-out>`` tag in the generated
  junit xml file if the ``junit_logging`` ini option is set to ``system-out``.
  If the value of this ini option is ``system-err``, the logs are written to
  ``<system-err>``. The default value for ``junit_logging`` is ``no``, meaning
  captured logs are not written to the output file. (`#3156
  <https://github.com/pytest-dev/pytest/issues/3156>`_)

- Allow the logging plugin to handle ``pytest_runtest_logstart`` and
  ``pytest_runtest_logfinish`` hooks when live logs are enabled. (`#3189
  <https://github.com/pytest-dev/pytest/issues/3189>`_)

- Passing ``--log-cli-level`` in the command-line now automatically activates
  live logging. (`#3190 <https://github.com/pytest-dev/pytest/issues/3190>`_)

- Add command line option ``--deselect`` to allow deselection of individual
  tests at collection time. (`#3198
  <https://github.com/pytest-dev/pytest/issues/3198>`_)

- Captured logs are printed before entering pdb. (`#3204
  <https://github.com/pytest-dev/pytest/issues/3204>`_)

- Deselected item count is now shown before tests are run, e.g. ``collected X
  items / Y deselected``. (`#3213
  <https://github.com/pytest-dev/pytest/issues/3213>`_)

- The builtin module ``platform`` is now available for use in expressions in
  ``pytest.mark``. (`#3236
  <https://github.com/pytest-dev/pytest/issues/3236>`_)

- The *short test summary info* section now is displayed after tracebacks and
  warnings in the terminal. (`#3255
  <https://github.com/pytest-dev/pytest/issues/3255>`_)

- New ``--verbosity`` flag to set verbosity level explicitly. (`#3296
  <https://github.com/pytest-dev/pytest/issues/3296>`_)

- ``pytest.approx`` now accepts comparing a numpy array with a scalar. (`#3312
  <https://github.com/pytest-dev/pytest/issues/3312>`_)


Bug Fixes
---------

- Suppress ``IOError`` when closing the temporary file used for capturing
  streams in Python 2.7. (`#2370
  <https://github.com/pytest-dev/pytest/issues/2370>`_)

- Fixed ``clear()`` method on ``caplog`` fixture which cleared ``records``, but
  not the ``text`` property. (`#3297
  <https://github.com/pytest-dev/pytest/issues/3297>`_)

- During test collection, when stdin is not allowed to be read, the
  ``DontReadFromStdin`` object still allow itself to be iterable and resolved
  to an iterator without crashing. (`#3314
  <https://github.com/pytest-dev/pytest/issues/3314>`_)


Improved Documentation
----------------------

- Added a `reference <https://docs.pytest.org/en/latest/reference.html>`_ page
  to the docs. (`#1713 <https://github.com/pytest-dev/pytest/issues/1713>`_)


Trivial/Internal Changes
------------------------

- Change minimum requirement of ``attrs`` to ``17.4.0``. (`#3228
  <https://github.com/pytest-dev/pytest/issues/3228>`_)

- Renamed example directories so all tests pass when ran from the base
  directory. (`#3245 <https://github.com/pytest-dev/pytest/issues/3245>`_)

- Internal ``mark.py`` module has been turned into a package. (`#3250
  <https://github.com/pytest-dev/pytest/issues/3250>`_)

- ``pytest`` now depends on the `more-itertools
  <https://github.com/erikrose/more-itertools>`_ package. (`#3265
  <https://github.com/pytest-dev/pytest/issues/3265>`_)

- Added warning when ``[pytest]`` section is used in a ``.cfg`` file passed
  with ``-c`` (`#3268 <https://github.com/pytest-dev/pytest/issues/3268>`_)

- ``nodeids`` can now be passed explicitly to ``FSCollector`` and ``Node``
  constructors. (`#3291 <https://github.com/pytest-dev/pytest/issues/3291>`_)

- Internal refactoring of ``FormattedExcinfo`` to use ``attrs`` facilities and
  remove old support code for legacy Python versions. (`#3292
  <https://github.com/pytest-dev/pytest/issues/3292>`_)

- Refactoring to unify how verbosity is handled internally. (`#3296
  <https://github.com/pytest-dev/pytest/issues/3296>`_)

- Internal refactoring to better integrate with argparse. (`#3304
  <https://github.com/pytest-dev/pytest/issues/3304>`_)

- Fix a python example when calling a fixture in doc/en/usage.rst (`#3308
  <https://github.com/pytest-dev/pytest/issues/3308>`_)


pytest 3.4.2 (2018-03-04)
=========================

Bug Fixes
---------

- Removed progress information when capture option is ``no``. (`#3203
  <https://github.com/pytest-dev/pytest/issues/3203>`_)

- Refactor check of bindir from ``exists`` to ``isdir``. (`#3241
  <https://github.com/pytest-dev/pytest/issues/3241>`_)

- Fix ``TypeError`` issue when using ``approx`` with a ``Decimal`` value.
  (`#3247 <https://github.com/pytest-dev/pytest/issues/3247>`_)

- Fix reference cycle generated when using the ``request`` fixture. (`#3249
  <https://github.com/pytest-dev/pytest/issues/3249>`_)

- ``[tool:pytest]`` sections in ``*.cfg`` files passed by the ``-c`` option are
  now properly recognized. (`#3260
  <https://github.com/pytest-dev/pytest/issues/3260>`_)


Improved Documentation
----------------------

- Add logging plugin to plugins list. (`#3209
  <https://github.com/pytest-dev/pytest/issues/3209>`_)


Trivial/Internal Changes
------------------------

- Fix minor typo in fixture.rst (`#3259
  <https://github.com/pytest-dev/pytest/issues/3259>`_)


pytest 3.4.1 (2018-02-20)
=========================

Bug Fixes
---------

- Move import of ``doctest.UnexpectedException`` to top-level to avoid possible
  errors when using ``--pdb``. (`#1810
  <https://github.com/pytest-dev/pytest/issues/1810>`_)

- Added printing of captured stdout/stderr before entering pdb, and improved a
  test which was giving false negatives about output capturing. (`#3052
  <https://github.com/pytest-dev/pytest/issues/3052>`_)

- Fix ordering of tests using parametrized fixtures which can lead to fixtures
  being created more than necessary. (`#3161
  <https://github.com/pytest-dev/pytest/issues/3161>`_)

- Fix bug where logging happening at hooks outside of "test run" hooks would
  cause an internal error. (`#3184
  <https://github.com/pytest-dev/pytest/issues/3184>`_)

- Detect arguments injected by ``unittest.mock.patch`` decorator correctly when
  pypi ``mock.patch`` is installed and imported. (`#3206
  <https://github.com/pytest-dev/pytest/issues/3206>`_)

- Errors shown when a ``pytest.raises()`` with ``match=`` fails are now cleaner
  on what happened: When no exception was raised, the "matching '...'" part got
  removed as it falsely implies that an exception was raised but it didn't
  match. When a wrong exception was raised, it's now thrown (like
  ``pytest.raised()`` without ``match=`` would) instead of complaining about
  the unmatched text. (`#3222
  <https://github.com/pytest-dev/pytest/issues/3222>`_)

- Fixed output capture handling in doctests on macOS. (`#985
  <https://github.com/pytest-dev/pytest/issues/985>`_)


Improved Documentation
----------------------

- Add Sphinx parameter docs for ``match`` and ``message`` args to
  ``pytest.raises``. (`#3202
  <https://github.com/pytest-dev/pytest/issues/3202>`_)


Trivial/Internal Changes
------------------------

- pytest has changed the publication procedure and is now being published to
  PyPI directly from Travis. (`#3060
  <https://github.com/pytest-dev/pytest/issues/3060>`_)

- Rename ``ParameterSet._for_parameterize()`` to ``_for_parametrize()`` in
  order to comply with the naming convention. (`#3166
  <https://github.com/pytest-dev/pytest/issues/3166>`_)

- Skip failing pdb/doctest test on mac. (`#985
  <https://github.com/pytest-dev/pytest/issues/985>`_)


pytest 3.4.0 (2018-01-30)
=========================

Deprecations and Removals
-------------------------

- All pytest classes now subclass ``object`` for better Python 2/3 compatibility.
  This should not affect user code except in very rare edge cases. (`#2147
  <https://github.com/pytest-dev/pytest/issues/2147>`_)


Features
--------

- Introduce ``empty_parameter_set_mark`` ini option to select which mark to
  apply when ``@pytest.mark.parametrize`` is given an empty set of parameters.
  Valid options are ``skip`` (default) and ``xfail``. Note that it is planned
  to change the default to ``xfail`` in future releases as this is considered
  less error prone. (`#2527
  <https://github.com/pytest-dev/pytest/issues/2527>`_)

- **Incompatible change**: after community feedback the `logging
  <https://docs.pytest.org/en/latest/logging.html>`_ functionality has
  undergone some changes. Please consult the `logging documentation
  <https://docs.pytest.org/en/latest/logging.html#incompatible-changes-in-pytest-3-4>`_
  for details. (`#3013 <https://github.com/pytest-dev/pytest/issues/3013>`_)

- Console output falls back to "classic" mode when capturing is disabled (``-s``),
  otherwise the output gets garbled to the point of being useless. (`#3038
  <https://github.com/pytest-dev/pytest/issues/3038>`_)

- New `pytest_runtest_logfinish
  <https://docs.pytest.org/en/latest/writing_plugins.html#_pytest.hookspec.pytest_runtest_logfinish>`_
  hook which is called when a test item has finished executing, analogous to
  `pytest_runtest_logstart
  <https://docs.pytest.org/en/latest/writing_plugins.html#_pytest.hookspec.pytest_runtest_start>`_.
  (`#3101 <https://github.com/pytest-dev/pytest/issues/3101>`_)

- Improve performance when collecting tests using many fixtures. (`#3107
  <https://github.com/pytest-dev/pytest/issues/3107>`_)

- New ``caplog.get_records(when)`` method which provides access to the captured
  records for the ``"setup"``, ``"call"`` and ``"teardown"``
  testing stages. (`#3117 <https://github.com/pytest-dev/pytest/issues/3117>`_)

- New fixture ``record_xml_attribute`` that allows modifying and inserting
  attributes on the ``<testcase>`` xml node in JUnit reports. (`#3130
  <https://github.com/pytest-dev/pytest/issues/3130>`_)

- The default cache directory has been renamed from ``.cache`` to
  ``.pytest_cache`` after community feedback that the name ``.cache`` did not
  make it clear that it was used by pytest. (`#3138
  <https://github.com/pytest-dev/pytest/issues/3138>`_)

- Colorize the levelname column in the live-log output. (`#3142
  <https://github.com/pytest-dev/pytest/issues/3142>`_)


Bug Fixes
---------

- Fix hanging pexpect test on MacOS by using flush() instead of wait().
  (`#2022 <https://github.com/pytest-dev/pytest/issues/2022>`_)

- Fix restoring Python state after in-process pytest runs with the
  ``pytester`` plugin; this may break tests using multiple inprocess
  pytest runs if later ones depend on earlier ones leaking global interpreter
  changes. (`#3016 <https://github.com/pytest-dev/pytest/issues/3016>`_)

- Fix skipping plugin reporting hook when test aborted before plugin setup
  hook. (`#3074 <https://github.com/pytest-dev/pytest/issues/3074>`_)

- Fix progress percentage reported when tests fail during teardown. (`#3088
  <https://github.com/pytest-dev/pytest/issues/3088>`_)

- **Incompatible change**: ``-o/--override`` option no longer eats all the
  remaining options, which can lead to surprising behavior: for example,
  ``pytest -o foo=1 /path/to/test.py`` would fail because ``/path/to/test.py``
  would be considered as part of the ``-o`` command-line argument. One
  consequence of this is that now multiple configuration overrides need
  multiple ``-o`` flags: ``pytest -o foo=1 -o bar=2``. (`#3103
  <https://github.com/pytest-dev/pytest/issues/3103>`_)


Improved Documentation
----------------------

- Document hooks (defined with ``historic=True``) which cannot be used with
  ``hookwrapper=True``. (`#2423
  <https://github.com/pytest-dev/pytest/issues/2423>`_)

- Clarify that warning capturing doesn't change the warning filter by default.
  (`#2457 <https://github.com/pytest-dev/pytest/issues/2457>`_)

- Clarify a possible confusion when using pytest_fixture_setup with fixture
  functions that return None. (`#2698
  <https://github.com/pytest-dev/pytest/issues/2698>`_)

- Fix the wording of a sentence on doctest flags used in pytest. (`#3076
  <https://github.com/pytest-dev/pytest/issues/3076>`_)

- Prefer ``https://*.readthedocs.io`` over ``http://*.rtfd.org`` for links in
  the documentation. (`#3092
  <https://github.com/pytest-dev/pytest/issues/3092>`_)

- Improve readability (wording, grammar) of Getting Started guide (`#3131
  <https://github.com/pytest-dev/pytest/issues/3131>`_)

- Added note that calling pytest.main multiple times from the same process is
  not recommended because of import caching. (`#3143
  <https://github.com/pytest-dev/pytest/issues/3143>`_)


Trivial/Internal Changes
------------------------

- Show a simple and easy error when keyword expressions trigger a syntax error
  (for example, ``"-k foo and import"`` will show an error that you can not use
  the ``import`` keyword in expressions). (`#2953
  <https://github.com/pytest-dev/pytest/issues/2953>`_)

- Change parametrized automatic test id generation to use the ``__name__``
  attribute of functions instead of the fallback argument name plus counter.
  (`#2976 <https://github.com/pytest-dev/pytest/issues/2976>`_)

- Replace py.std with stdlib imports. (`#3067
  <https://github.com/pytest-dev/pytest/issues/3067>`_)

- Corrected 'you' to 'your' in logging docs. (`#3129
  <https://github.com/pytest-dev/pytest/issues/3129>`_)


pytest 3.3.2 (2017-12-25)
=========================

Bug Fixes
---------

- pytester: ignore files used to obtain current user metadata in the fd leak
  detector. (`#2784 <https://github.com/pytest-dev/pytest/issues/2784>`_)

- Fix **memory leak** where objects returned by fixtures were never destructed
  by the garbage collector. (`#2981
  <https://github.com/pytest-dev/pytest/issues/2981>`_)

- Fix conversion of pyargs to filename to not convert symlinks on Python 2. (`#2985
  <https://github.com/pytest-dev/pytest/issues/2985>`_)

- ``PYTEST_DONT_REWRITE`` is now checked for plugins too rather than only for
  test modules. (`#2995 <https://github.com/pytest-dev/pytest/issues/2995>`_)


Improved Documentation
----------------------

- Add clarifying note about behavior of multiple parametrized arguments (`#3001
  <https://github.com/pytest-dev/pytest/issues/3001>`_)


Trivial/Internal Changes
------------------------

- Code cleanup. (`#3015 <https://github.com/pytest-dev/pytest/issues/3015>`_,
  `#3021 <https://github.com/pytest-dev/pytest/issues/3021>`_)

- Clean up code by replacing imports and references of ``_ast`` to ``ast``.
  (`#3018 <https://github.com/pytest-dev/pytest/issues/3018>`_)


pytest 3.3.1 (2017-12-05)
=========================

Bug Fixes
---------

- Fix issue about ``-p no:<plugin>`` having no effect. (`#2920
  <https://github.com/pytest-dev/pytest/issues/2920>`_)

- Fix regression with warnings that contained non-strings in their arguments in
  Python 2. (`#2956 <https://github.com/pytest-dev/pytest/issues/2956>`_)

- Always escape null bytes when setting ``PYTEST_CURRENT_TEST``. (`#2957
  <https://github.com/pytest-dev/pytest/issues/2957>`_)

- Fix ``ZeroDivisionError`` when using the ``testmon`` plugin when no tests
  were actually collected. (`#2971
  <https://github.com/pytest-dev/pytest/issues/2971>`_)

- Bring back ``TerminalReporter.writer`` as an alias to
  ``TerminalReporter._tw``. This alias was removed by accident in the ``3.3.0``
  release. (`#2984 <https://github.com/pytest-dev/pytest/issues/2984>`_)

- The ``pytest-capturelog`` plugin is now also blacklisted, avoiding errors when
  running pytest with it still installed. (`#3004
  <https://github.com/pytest-dev/pytest/issues/3004>`_)


Improved Documentation
----------------------

- Fix broken link to plugin ``pytest-localserver``. (`#2963
  <https://github.com/pytest-dev/pytest/issues/2963>`_)


Trivial/Internal Changes
------------------------

- Update github "bugs" link in ``CONTRIBUTING.rst`` (`#2949
  <https://github.com/pytest-dev/pytest/issues/2949>`_)


pytest 3.3.0 (2017-11-23)
=========================

Deprecations and Removals
-------------------------

- pytest no longer supports Python **2.6** and **3.3**. Those Python versions
  are EOL for some time now and incur maintenance and compatibility costs on
  the pytest core team, and following up with the rest of the community we
  decided that they will no longer be supported starting on this version. Users
  which still require those versions should pin pytest to ``<3.3``. (`#2812
  <https://github.com/pytest-dev/pytest/issues/2812>`_)

- Remove internal ``_preloadplugins()`` function. This removal is part of the
  ``pytest_namespace()`` hook deprecation. (`#2636
  <https://github.com/pytest-dev/pytest/issues/2636>`_)

- Internally change ``CallSpec2`` to have a list of marks instead of a broken
  mapping of keywords. This removes the keywords attribute of the internal
  ``CallSpec2`` class. (`#2672
  <https://github.com/pytest-dev/pytest/issues/2672>`_)

- Remove ParameterSet.deprecated_arg_dict - its not a public api and the lack
  of the underscore was a naming error. (`#2675
  <https://github.com/pytest-dev/pytest/issues/2675>`_)

- Remove the internal multi-typed attribute ``Node._evalskip`` and replace it
  with the boolean ``Node._skipped_by_mark``. (`#2767
  <https://github.com/pytest-dev/pytest/issues/2767>`_)

- The ``params`` list passed to ``pytest.fixture`` is now for
  all effects considered immutable and frozen at the moment of the ``pytest.fixture``
  call. Previously the list could be changed before the first invocation of the fixture
  allowing for a form of dynamic parametrization (for example, updated from command-line options),
  but this was an unwanted implementation detail which complicated the internals and prevented
  some internal cleanup. See issue `#2959 <https://github.com/pytest-dev/pytest/issues/2959>`_
  for details and a recommended workaround.

Features
--------

- ``pytest_fixture_post_finalizer`` hook can now receive a ``request``
  argument. (`#2124 <https://github.com/pytest-dev/pytest/issues/2124>`_)

- Replace the old introspection code in compat.py that determines the available
  arguments of fixtures with inspect.signature on Python 3 and
  funcsigs.signature on Python 2. This should respect ``__signature__``
  declarations on functions. (`#2267
  <https://github.com/pytest-dev/pytest/issues/2267>`_)

- Report tests with global ``pytestmark`` variable only once. (`#2549
  <https://github.com/pytest-dev/pytest/issues/2549>`_)

- Now pytest displays the total progress percentage while running tests. The
  previous output style can be set by configuring the ``console_output_style``
  setting to ``classic``. (`#2657 <https://github.com/pytest-dev/pytest/issues/2657>`_)

- Match ``warns`` signature to ``raises`` by adding ``match`` keyword. (`#2708
  <https://github.com/pytest-dev/pytest/issues/2708>`_)

- pytest now captures and displays output from the standard ``logging`` module.
  The user can control the logging level to be captured by specifying options
  in ``pytest.ini``, the command line and also during individual tests using
  markers. Also, a ``caplog`` fixture is available that enables users to test
  the captured log during specific tests (similar to ``capsys`` for example).
  For more information, please see the `logging docs
  <https://docs.pytest.org/en/latest/logging.html>`_. This feature was
  introduced by merging the popular `pytest-catchlog
  <https://pypi.org/project/pytest-catchlog/>`_ plugin, thanks to `Thomas Hisch
  <https://github.com/thisch>`_. Be advised that during the merging the
  backward compatibility interface with the defunct ``pytest-capturelog`` has
  been dropped. (`#2794 <https://github.com/pytest-dev/pytest/issues/2794>`_)

- Add ``allow_module_level`` kwarg to ``pytest.skip()``, enabling to skip the
  whole module. (`#2808 <https://github.com/pytest-dev/pytest/issues/2808>`_)

- Allow setting ``file_or_dir``, ``-c``, and ``-o`` in PYTEST_ADDOPTS. (`#2824
  <https://github.com/pytest-dev/pytest/issues/2824>`_)

- Return stdout/stderr capture results as a ``namedtuple``, so ``out`` and
  ``err`` can be accessed by attribute. (`#2879
  <https://github.com/pytest-dev/pytest/issues/2879>`_)

- Add ``capfdbinary``, a version of ``capfd`` which returns bytes from
  ``readouterr()``. (`#2923
  <https://github.com/pytest-dev/pytest/issues/2923>`_)

- Add ``capsysbinary`` a version of ``capsys`` which returns bytes from
  ``readouterr()``. (`#2934
  <https://github.com/pytest-dev/pytest/issues/2934>`_)

- Implement feature to skip ``setup.py`` files when run with
  ``--doctest-modules``. (`#502
  <https://github.com/pytest-dev/pytest/issues/502>`_)


Bug Fixes
---------

- Resume output capturing after ``capsys/capfd.disabled()`` context manager.
  (`#1993 <https://github.com/pytest-dev/pytest/issues/1993>`_)

- ``pytest_fixture_setup`` and ``pytest_fixture_post_finalizer`` hooks are now
  called for all ``conftest.py`` files. (`#2124
  <https://github.com/pytest-dev/pytest/issues/2124>`_)

- If an exception happens while loading a plugin, pytest no longer hides the
  original traceback. In Python 2 it will show the original traceback with a new
  message that explains in which plugin. In Python 3 it will show 2 canonized
  exceptions, the original exception while loading the plugin in addition to an
  exception that pytest throws about loading a plugin. (`#2491
  <https://github.com/pytest-dev/pytest/issues/2491>`_)

- ``capsys`` and ``capfd`` can now be used by other fixtures. (`#2709
  <https://github.com/pytest-dev/pytest/issues/2709>`_)

- Internal ``pytester`` plugin properly encodes ``bytes`` arguments to
  ``utf-8``. (`#2738 <https://github.com/pytest-dev/pytest/issues/2738>`_)

- ``testdir`` now uses use the same method used by ``tmpdir`` to create its
  temporary directory. This changes the final structure of the ``testdir``
  directory slightly, but should not affect usage in normal scenarios and
  avoids a number of potential problems. (`#2751
  <https://github.com/pytest-dev/pytest/issues/2751>`_)

- pytest no longer complains about warnings with unicode messages being
  non-ascii compatible even for ascii-compatible messages. As a result of this,
  warnings with unicode messages are converted first to an ascii representation
  for safety. (`#2809 <https://github.com/pytest-dev/pytest/issues/2809>`_)

- Change return value of pytest command when ``--maxfail`` is reached from
  ``2`` (interrupted) to ``1`` (failed). (`#2845
  <https://github.com/pytest-dev/pytest/issues/2845>`_)

- Fix issue in assertion rewriting which could lead it to rewrite modules which
  should not be rewritten. (`#2939
  <https://github.com/pytest-dev/pytest/issues/2939>`_)

- Handle marks without description in ``pytest.ini``. (`#2942
  <https://github.com/pytest-dev/pytest/issues/2942>`_)


Trivial/Internal Changes
------------------------

- pytest now depends on `attrs <https://pypi.org/project/attrs/>`__ for internal
  structures to ease code maintainability. (`#2641
  <https://github.com/pytest-dev/pytest/issues/2641>`_)

- Refactored internal Python 2/3 compatibility code to use ``six``. (`#2642
  <https://github.com/pytest-dev/pytest/issues/2642>`_)

- Stop vendoring ``pluggy`` - we're missing out on its latest changes for not
  much benefit (`#2719 <https://github.com/pytest-dev/pytest/issues/2719>`_)

- Internal refactor: simplify ascii string escaping by using the
  backslashreplace error handler in newer Python 3 versions. (`#2734
  <https://github.com/pytest-dev/pytest/issues/2734>`_)

- Remove unnecessary mark evaluator in unittest plugin (`#2767
  <https://github.com/pytest-dev/pytest/issues/2767>`_)

- Calls to ``Metafunc.addcall`` now emit a deprecation warning. This function
  is scheduled to be removed in ``pytest-4.0``. (`#2876
  <https://github.com/pytest-dev/pytest/issues/2876>`_)

- Internal move of the parameterset extraction to a more maintainable place.
  (`#2877 <https://github.com/pytest-dev/pytest/issues/2877>`_)

- Internal refactoring to simplify scope node lookup. (`#2910
  <https://github.com/pytest-dev/pytest/issues/2910>`_)

- Configure ``pytest`` to prevent pip from installing pytest in unsupported
  Python versions. (`#2922
  <https://github.com/pytest-dev/pytest/issues/2922>`_)


pytest 3.2.5 (2017-11-15)
=========================

Bug Fixes
---------

- Remove ``py<1.5`` restriction from ``pytest`` as this can cause version
  conflicts in some installations. (`#2926
  <https://github.com/pytest-dev/pytest/issues/2926>`_)


pytest 3.2.4 (2017-11-13)
=========================

Bug Fixes
---------

- Fix the bug where running with ``--pyargs`` will result in items with
  empty ``parent.nodeid`` if run from a different root directory. (`#2775
  <https://github.com/pytest-dev/pytest/issues/2775>`_)

- Fix issue with ``@pytest.parametrize`` if argnames was specified as keyword arguments.
  (`#2819 <https://github.com/pytest-dev/pytest/issues/2819>`_)

- Strip whitespace from marker names when reading them from INI config. (`#2856
  <https://github.com/pytest-dev/pytest/issues/2856>`_)

- Show full context of doctest source in the pytest output, if the line number of
  failed example in the docstring is < 9. (`#2882
  <https://github.com/pytest-dev/pytest/issues/2882>`_)

- Match fixture paths against actual path segments in order to avoid matching folders which share a prefix.
  (`#2836 <https://github.com/pytest-dev/pytest/issues/2836>`_)

Improved Documentation
----------------------

- Introduce a dedicated section about conftest.py. (`#1505
  <https://github.com/pytest-dev/pytest/issues/1505>`_)

- Explicitly mention ``xpass`` in the documentation of ``xfail``. (`#1997
  <https://github.com/pytest-dev/pytest/issues/1997>`_)

- Append example for pytest.param in the example/parametrize document. (`#2658
  <https://github.com/pytest-dev/pytest/issues/2658>`_)

- Clarify language of proposal for fixtures parameters (`#2893
  <https://github.com/pytest-dev/pytest/issues/2893>`_)

- List python 3.6 in the documented supported versions in the getting started
  document. (`#2903 <https://github.com/pytest-dev/pytest/issues/2903>`_)

- Clarify the documentation of available fixture scopes. (`#538
  <https://github.com/pytest-dev/pytest/issues/538>`_)

- Add documentation about the ``python -m pytest`` invocation adding the
  current directory to sys.path. (`#911
  <https://github.com/pytest-dev/pytest/issues/911>`_)


pytest 3.2.3 (2017-10-03)
=========================

Bug Fixes
---------

- Fix crash in tab completion when no prefix is given. (`#2748
  <https://github.com/pytest-dev/pytest/issues/2748>`_)

- The equality checking function (``__eq__``) of ``MarkDecorator`` returns
  ``False`` if one object is not an instance of ``MarkDecorator``. (`#2758
  <https://github.com/pytest-dev/pytest/issues/2758>`_)

- When running ``pytest --fixtures-per-test``: don't crash if an item has no
  _fixtureinfo attribute (e.g. doctests) (`#2788
  <https://github.com/pytest-dev/pytest/issues/2788>`_)


Improved Documentation
----------------------

- In help text of ``-k`` option, add example of using ``not`` to not select
  certain tests whose names match the provided expression. (`#1442
  <https://github.com/pytest-dev/pytest/issues/1442>`_)

- Add note in ``parametrize.rst`` about calling ``metafunc.parametrize``
  multiple times. (`#1548 <https://github.com/pytest-dev/pytest/issues/1548>`_)


Trivial/Internal Changes
------------------------

- Set ``xfail_strict=True`` in pytest's own test suite to catch expected
  failures as soon as they start to pass. (`#2722
  <https://github.com/pytest-dev/pytest/issues/2722>`_)

- Fix typo in example of passing a callable to markers (in example/markers.rst)
  (`#2765 <https://github.com/pytest-dev/pytest/issues/2765>`_)


pytest 3.2.2 (2017-09-06)
=========================

Bug Fixes
---------

- Calling the deprecated ``request.getfuncargvalue()`` now shows the source of
  the call. (`#2681 <https://github.com/pytest-dev/pytest/issues/2681>`_)

- Allow tests declared as ``@staticmethod`` to use fixtures. (`#2699
  <https://github.com/pytest-dev/pytest/issues/2699>`_)

- Fixed edge-case during collection: attributes which raised ``pytest.fail``
  when accessed would abort the entire collection. (`#2707
  <https://github.com/pytest-dev/pytest/issues/2707>`_)

- Fix ``ReprFuncArgs`` with mixed unicode and UTF-8 args. (`#2731
  <https://github.com/pytest-dev/pytest/issues/2731>`_)


Improved Documentation
----------------------

- In examples on working with custom markers, add examples demonstrating the
  usage of ``pytest.mark.MARKER_NAME.with_args`` in comparison with
  ``pytest.mark.MARKER_NAME.__call__`` (`#2604
  <https://github.com/pytest-dev/pytest/issues/2604>`_)

- In one of the simple examples, use ``pytest_collection_modifyitems()`` to skip
  tests based on a command-line option, allowing its sharing while preventing a
  user error when acessing ``pytest.config`` before the argument parsing.
  (`#2653 <https://github.com/pytest-dev/pytest/issues/2653>`_)


Trivial/Internal Changes
------------------------

- Fixed minor error in 'Good Practices/Manual Integration' code snippet.
  (`#2691 <https://github.com/pytest-dev/pytest/issues/2691>`_)

- Fixed typo in goodpractices.rst. (`#2721
  <https://github.com/pytest-dev/pytest/issues/2721>`_)

- Improve user guidance regarding ``--resultlog`` deprecation. (`#2739
  <https://github.com/pytest-dev/pytest/issues/2739>`_)


pytest 3.2.1 (2017-08-08)
=========================

Bug Fixes
---------

- Fixed small terminal glitch when collecting a single test item. (`#2579
  <https://github.com/pytest-dev/pytest/issues/2579>`_)

- Correctly consider ``/`` as the file separator to automatically mark plugin
  files for rewrite on Windows. (`#2591 <https://github.com/pytest-
  dev/pytest/issues/2591>`_)

- Properly escape test names when setting ``PYTEST_CURRENT_TEST`` environment
  variable. (`#2644 <https://github.com/pytest-dev/pytest/issues/2644>`_)

- Fix error on Windows and Python 3.6+ when ``sys.stdout`` has been replaced
  with a stream-like object which does not implement the full ``io`` module
  buffer protocol. In particular this affects ``pytest-xdist`` users on the
  aforementioned platform. (`#2666 <https://github.com/pytest-
  dev/pytest/issues/2666>`_)


Improved Documentation
----------------------

- Explicitly document which pytest features work with ``unittest``. (`#2626
  <https://github.com/pytest-dev/pytest/issues/2626>`_)


pytest 3.2.0 (2017-07-30)
=========================

Deprecations and Removals
-------------------------

- ``pytest.approx`` no longer supports ``>``, ``>=``, ``<`` and ``<=``
  operators to avoid surprising/inconsistent behavior. See `the approx docs
  <https://docs.pytest.org/en/latest/builtin.html#pytest.approx>`_ for more
  information. (`#2003 <https://github.com/pytest-dev/pytest/issues/2003>`_)

- All old-style specific behavior in current classes in the pytest's API is
  considered deprecated at this point and will be removed in a future release.
  This affects Python 2 users only and in rare situations. (`#2147
  <https://github.com/pytest-dev/pytest/issues/2147>`_)

- A deprecation warning is now raised when using marks for parameters
  in ``pytest.mark.parametrize``. Use ``pytest.param`` to apply marks to
  parameters instead. (`#2427 <https://github.com/pytest-dev/pytest/issues/2427>`_)


Features
--------

- Add support for numpy arrays (and dicts) to approx. (`#1994
  <https://github.com/pytest-dev/pytest/issues/1994>`_)

- Now test function objects have a ``pytestmark`` attribute containing a list
  of marks applied directly to the test function, as opposed to marks inherited
  from parent classes or modules. (`#2516 <https://github.com/pytest-
  dev/pytest/issues/2516>`_)

- Collection ignores local virtualenvs by default; ``--collect-in-virtualenv``
  overrides this behavior. (`#2518 <https://github.com/pytest-
  dev/pytest/issues/2518>`_)

- Allow class methods decorated as ``@staticmethod`` to be candidates for
  collection as a test function. (Only for Python 2.7 and above. Python 2.6
  will still ignore static methods.) (`#2528 <https://github.com/pytest-
  dev/pytest/issues/2528>`_)

- Introduce ``mark.with_args`` in order to allow passing functions/classes as
  sole argument to marks. (`#2540 <https://github.com/pytest-
  dev/pytest/issues/2540>`_)

- New ``cache_dir`` ini option: sets the directory where the contents of the
  cache plugin are stored. Directory may be relative or absolute path: if relative path, then
  directory is created relative to ``rootdir``, otherwise it is used as is.
  Additionally path may contain environment variables which are expanded during
  runtime. (`#2543 <https://github.com/pytest-dev/pytest/issues/2543>`_)

- Introduce the ``PYTEST_CURRENT_TEST`` environment variable that is set with
  the ``nodeid`` and stage (``setup``, ``call`` and ``teardown``) of the test
  being currently executed. See the `documentation
  <https://docs.pytest.org/en/latest/example/simple.html#pytest-current-test-
  environment-variable>`_ for more info. (`#2583 <https://github.com/pytest-
  dev/pytest/issues/2583>`_)

- Introduced ``@pytest.mark.filterwarnings`` mark which allows overwriting the
  warnings filter on a per test, class or module level. See the `docs
  <https://docs.pytest.org/en/latest/warnings.html#pytest-mark-
  filterwarnings>`_ for more information. (`#2598 <https://github.com/pytest-
  dev/pytest/issues/2598>`_)

- ``--last-failed`` now remembers forever when a test has failed and only
  forgets it if it passes again. This makes it easy to fix a test suite by
  selectively running files and fixing tests incrementally. (`#2621
  <https://github.com/pytest-dev/pytest/issues/2621>`_)

- New ``pytest_report_collectionfinish`` hook which allows plugins to add
  messages to the terminal reporting after collection has been finished
  successfully. (`#2622 <https://github.com/pytest-dev/pytest/issues/2622>`_)

- Added support for `PEP-415's <https://www.python.org/dev/peps/pep-0415/>`_
  ``Exception.__suppress_context__``. Now if a ``raise exception from None`` is
  caught by pytest, pytest will no longer chain the context in the test report.
  The behavior now matches Python's traceback behavior. (`#2631
  <https://github.com/pytest-dev/pytest/issues/2631>`_)

- Exceptions raised by ``pytest.fail``, ``pytest.skip`` and ``pytest.xfail``
  now subclass BaseException, making them harder to be caught unintentionally
  by normal code. (`#580 <https://github.com/pytest-dev/pytest/issues/580>`_)


Bug Fixes
---------

- Set ``stdin`` to a closed ``PIPE`` in ``pytester.py.Testdir.popen()`` for
  avoid unwanted interactive ``pdb`` (`#2023 <https://github.com/pytest-
  dev/pytest/issues/2023>`_)

- Add missing ``encoding`` attribute to ``sys.std*`` streams when using
  ``capsys`` capture mode. (`#2375 <https://github.com/pytest-
  dev/pytest/issues/2375>`_)

- Fix terminal color changing to black on Windows if ``colorama`` is imported
  in a ``conftest.py`` file. (`#2510 <https://github.com/pytest-
  dev/pytest/issues/2510>`_)

- Fix line number when reporting summary of skipped tests. (`#2548
  <https://github.com/pytest-dev/pytest/issues/2548>`_)

- capture: ensure that EncodedFile.name is a string. (`#2555
  <https://github.com/pytest-dev/pytest/issues/2555>`_)

- The options ``--fixtures`` and ``--fixtures-per-test`` will now keep
  indentation within docstrings. (`#2574 <https://github.com/pytest-
  dev/pytest/issues/2574>`_)

- doctests line numbers are now reported correctly, fixing `pytest-sugar#122
  <https://github.com/Frozenball/pytest-sugar/issues/122>`_. (`#2610
  <https://github.com/pytest-dev/pytest/issues/2610>`_)

- Fix non-determinism in order of fixture collection. Adds new dependency
  (ordereddict) for Python 2.6. (`#920 <https://github.com/pytest-
  dev/pytest/issues/920>`_)


Improved Documentation
----------------------

- Clarify ``pytest_configure`` hook call order. (`#2539
  <https://github.com/pytest-dev/pytest/issues/2539>`_)

- Extend documentation for testing plugin code with the ``pytester`` plugin.
  (`#971 <https://github.com/pytest-dev/pytest/issues/971>`_)


Trivial/Internal Changes
------------------------

- Update help message for ``--strict`` to make it clear it only deals with
  unregistered markers, not warnings. (`#2444 <https://github.com/pytest-
  dev/pytest/issues/2444>`_)

- Internal code move: move code for pytest.approx/pytest.raises to own files in
  order to cut down the size of python.py (`#2489 <https://github.com/pytest-
  dev/pytest/issues/2489>`_)

- Renamed the utility function ``_pytest.compat._escape_strings`` to
  ``_ascii_escaped`` to better communicate the function's purpose. (`#2533
  <https://github.com/pytest-dev/pytest/issues/2533>`_)

- Improve error message for CollectError with skip/skipif. (`#2546
  <https://github.com/pytest-dev/pytest/issues/2546>`_)

- Emit warning about ``yield`` tests being deprecated only once per generator.
  (`#2562 <https://github.com/pytest-dev/pytest/issues/2562>`_)

- Ensure final collected line doesn't include artifacts of previous write.
  (`#2571 <https://github.com/pytest-dev/pytest/issues/2571>`_)

- Fixed all flake8 errors and warnings. (`#2581 <https://github.com/pytest-
  dev/pytest/issues/2581>`_)

- Added ``fix-lint`` tox environment to run automatic pep8 fixes on the code.
  (`#2582 <https://github.com/pytest-dev/pytest/issues/2582>`_)

- Turn warnings into errors in pytest's own test suite in order to catch
  regressions due to deprecations more promptly. (`#2588
  <https://github.com/pytest-dev/pytest/issues/2588>`_)

- Show multiple issue links in CHANGELOG entries. (`#2620
  <https://github.com/pytest-dev/pytest/issues/2620>`_)


pytest 3.1.3 (2017-07-03)
=========================

Bug Fixes
---------

- Fix decode error in Python 2 for doctests in docstrings. (`#2434
  <https://github.com/pytest-dev/pytest/issues/2434>`_)

- Exceptions raised during teardown by finalizers are now suppressed until all
  finalizers are called, with the initial exception reraised. (`#2440
  <https://github.com/pytest-dev/pytest/issues/2440>`_)

- Fix incorrect "collected items" report when specifying tests on the command-
  line. (`#2464 <https://github.com/pytest-dev/pytest/issues/2464>`_)

- ``deprecated_call`` in context-manager form now captures deprecation warnings
  even if the same warning has already been raised. Also, ``deprecated_call``
  will always produce the same error message (previously it would produce
  different messages in context-manager vs. function-call mode). (`#2469
  <https://github.com/pytest-dev/pytest/issues/2469>`_)

- Fix issue where paths collected by pytest could have triple leading ``/``
  characters. (`#2475 <https://github.com/pytest-dev/pytest/issues/2475>`_)

- Fix internal error when trying to detect the start of a recursive traceback.
  (`#2486 <https://github.com/pytest-dev/pytest/issues/2486>`_)


Improved Documentation
----------------------

- Explicitly state for which hooks the calls stop after the first non-None
  result. (`#2493 <https://github.com/pytest-dev/pytest/issues/2493>`_)


Trivial/Internal Changes
------------------------

- Create invoke tasks for updating the vendored packages. (`#2474
  <https://github.com/pytest-dev/pytest/issues/2474>`_)

- Update copyright dates in LICENSE, README.rst and in the documentation.
  (`#2499 <https://github.com/pytest-dev/pytest/issues/2499>`_)


pytest 3.1.2 (2017-06-08)
=========================

Bug Fixes
---------

- Required options added via ``pytest_addoption`` will no longer prevent using
  --help without passing them. (#1999)

- Respect ``python_files`` in assertion rewriting. (#2121)

- Fix recursion error detection when frames in the traceback contain objects
  that can't be compared (like ``numpy`` arrays). (#2459)

- ``UnicodeWarning`` is issued from the internal pytest warnings plugin only
  when the message contains non-ascii unicode (Python 2 only). (#2463)

- Added a workaround for Python 3.6 ``WindowsConsoleIO`` breaking due to Pytests's
  ``FDCapture``. Other code using console handles might still be affected by the
  very same issue and might require further workarounds/fixes, i.e. ``colorama``.
  (#2467)


Improved Documentation
----------------------

- Fix internal API links to ``pluggy`` objects. (#2331)

- Make it clear that ``pytest.xfail`` stops test execution at the calling point
  and improve overall flow of the ``skipping`` docs. (#810)


pytest 3.1.1 (2017-05-30)
=========================

Bug Fixes
---------

- pytest warning capture no longer overrides existing warning filters. The
  previous behaviour would override all filters and caused regressions in test
  suites which configure warning filters to match their needs. Note that as a
  side-effect of this is that ``DeprecationWarning`` and
  ``PendingDeprecationWarning`` are no longer shown by default. (#2430)

- Fix issue with non-ascii contents in doctest text files. (#2434)

- Fix encoding errors for unicode warnings in Python 2. (#2436)

- ``pytest.deprecated_call`` now captures ``PendingDeprecationWarning`` in
  context manager form. (#2441)


Improved Documentation
----------------------

- Addition of towncrier for changelog management. (#2390)


3.1.0 (2017-05-22)
==================


New Features
------------

* The ``pytest-warnings`` plugin has been integrated into the core and now ``pytest`` automatically
  captures and displays warnings at the end of the test session.

  .. warning::

    This feature may disrupt test suites which apply and treat warnings themselves, and can be
    disabled in your ``pytest.ini``:

    .. code-block:: ini

      [pytest]
      addopts = -p no:warnings

    See the `warnings documentation page <https://docs.pytest.org/en/latest/warnings.html>`_ for more
    information.

  Thanks `@nicoddemus`_ for the PR.

* Added ``junit_suite_name`` ini option to specify root ``<testsuite>`` name for JUnit XML reports (`#533`_).

* Added an ini option ``doctest_encoding`` to specify which encoding to use for doctest files.
  Thanks `@wheerd`_ for the PR (`#2101`_).

* ``pytest.warns`` now checks for subclass relationship rather than
  class equality. Thanks `@lesteve`_ for the PR (`#2166`_)

* ``pytest.raises`` now asserts that the error message matches a text or regex
  with the ``match`` keyword argument. Thanks `@Kriechi`_ for the PR.

* ``pytest.param`` can be used to declare test parameter sets with marks and test ids.
  Thanks `@RonnyPfannschmidt`_ for the PR.


Changes
-------

* remove all internal uses of pytest_namespace hooks,
  this is to prepare the removal of preloadconfig in pytest 4.0
  Thanks to `@RonnyPfannschmidt`_ for the PR.

* pytest now warns when a callable ids raises in a parametrized test. Thanks `@fogo`_ for the PR.

* It is now possible to skip test classes from being collected by setting a
  ``__test__`` attribute to ``False`` in the class body (`#2007`_). Thanks
  to `@syre`_ for the report and `@lwm`_ for the PR.

* Change junitxml.py to produce reports that comply with Junitxml schema.
  If the same test fails with failure in call and then errors in teardown
  we split testcase element into two, one containing the error and the other
  the failure. (`#2228`_) Thanks to `@kkoukiou`_ for the PR.

* Testcase reports with a ``url`` attribute will now properly write this to junitxml.
  Thanks `@fushi`_ for the PR (`#1874`_).

* Remove common items from dict comparison output when verbosity=1. Also update
  the truncation message to make it clearer that pytest truncates all
  assertion messages if verbosity < 2 (`#1512`_).
  Thanks `@mattduck`_ for the PR

* ``--pdbcls`` no longer implies ``--pdb``. This makes it possible to use
  ``addopts=--pdbcls=module.SomeClass`` on ``pytest.ini``. Thanks `@davidszotten`_ for
  the PR (`#1952`_).

* fix `#2013`_: turn RecordedWarning into ``namedtuple``,
  to give it a comprehensible repr while preventing unwarranted modification.

* fix `#2208`_: ensure an iteration limit for _pytest.compat.get_real_func.
  Thanks `@RonnyPfannschmidt`_ for the report and PR.

* Hooks are now verified after collection is complete, rather than right after loading installed plugins. This
  makes it easy to write hooks for plugins which will be loaded during collection, for example using the
  ``pytest_plugins`` special variable (`#1821`_).
  Thanks `@nicoddemus`_ for the PR.

* Modify ``pytest_make_parametrize_id()`` hook to accept ``argname`` as an
  additional parameter.
  Thanks `@unsignedint`_ for the PR.

* Add ``venv`` to the default ``norecursedirs`` setting.
  Thanks `@The-Compiler`_ for the PR.

* ``PluginManager.import_plugin`` now accepts unicode plugin names in Python 2.
  Thanks `@reutsharabani`_ for the PR.

* fix `#2308`_: When using both ``--lf`` and ``--ff``, only the last failed tests are run.
  Thanks `@ojii`_ for the PR.

* Replace minor/patch level version numbers in the documentation with placeholders.
  This significantly reduces change-noise as different contributors regnerate
  the documentation on different platforms.
  Thanks `@RonnyPfannschmidt`_ for the PR.

* fix `#2391`_: consider pytest_plugins on all plugin modules
  Thanks `@RonnyPfannschmidt`_ for the PR.


Bug Fixes
---------

* Fix ``AttributeError`` on ``sys.stdout.buffer`` / ``sys.stderr.buffer``
  while using ``capsys`` fixture in python 3. (`#1407`_).
  Thanks to `@asottile`_.

* Change capture.py's ``DontReadFromInput`` class to throw ``io.UnsupportedOperation`` errors rather
  than ValueErrors in the ``fileno`` method (`#2276`_).
  Thanks `@metasyn`_ and `@vlad-dragos`_ for the PR.

* Fix exception formatting while importing modules when the exception message
  contains non-ascii characters (`#2336`_).
  Thanks `@fabioz`_ for the report and `@nicoddemus`_ for the PR.

* Added documentation related to issue (`#1937`_)
  Thanks `@skylarjhdownes`_ for the PR.

* Allow collecting files with any file extension as Python modules (`#2369`_).
  Thanks `@Kodiologist`_ for the PR.

* Show the correct error message when collect "parametrize" func with wrong args (`#2383`_).
  Thanks `@The-Compiler`_ for the report and `@robin0371`_ for the PR.


.. _@davidszotten: https://github.com/davidszotten
.. _@fabioz: https://github.com/fabioz
.. _@fogo: https://github.com/fogo
.. _@fushi: https://github.com/fushi
.. _@Kodiologist: https://github.com/Kodiologist
.. _@Kriechi: https://github.com/Kriechi
.. _@mandeep: https://github.com/mandeep
.. _@mattduck: https://github.com/mattduck
.. _@metasyn: https://github.com/metasyn
.. _@MichalTHEDUDE: https://github.com/MichalTHEDUDE
.. _@ojii: https://github.com/ojii
.. _@reutsharabani: https://github.com/reutsharabani
.. _@robin0371: https://github.com/robin0371
.. _@skylarjhdownes: https://github.com/skylarjhdownes
.. _@unsignedint: https://github.com/unsignedint
.. _@wheerd: https://github.com/wheerd


.. _#1407: https://github.com/pytest-dev/pytest/issues/1407
.. _#1512: https://github.com/pytest-dev/pytest/issues/1512
.. _#1821: https://github.com/pytest-dev/pytest/issues/1821
.. _#1874: https://github.com/pytest-dev/pytest/pull/1874
.. _#1937: https://github.com/pytest-dev/pytest/issues/1937
.. _#1952: https://github.com/pytest-dev/pytest/pull/1952
.. _#2007: https://github.com/pytest-dev/pytest/issues/2007
.. _#2013: https://github.com/pytest-dev/pytest/issues/2013
.. _#2101: https://github.com/pytest-dev/pytest/pull/2101
.. _#2166: https://github.com/pytest-dev/pytest/pull/2166
.. _#2208: https://github.com/pytest-dev/pytest/issues/2208
.. _#2228: https://github.com/pytest-dev/pytest/issues/2228
.. _#2276: https://github.com/pytest-dev/pytest/issues/2276
.. _#2308: https://github.com/pytest-dev/pytest/issues/2308
.. _#2336: https://github.com/pytest-dev/pytest/issues/2336
.. _#2369: https://github.com/pytest-dev/pytest/issues/2369
.. _#2383: https://github.com/pytest-dev/pytest/issues/2383
.. _#2391: https://github.com/pytest-dev/pytest/issues/2391
.. _#533: https://github.com/pytest-dev/pytest/issues/533



3.0.7 (2017-03-14)
==================


* Fix issue in assertion rewriting breaking due to modules silently discarding
  other modules when importing fails
  Notably, importing the ``anydbm`` module is fixed. (`#2248`_).
  Thanks `@pfhayes`_ for the PR.

* junitxml: Fix problematic case where system-out tag occurred twice per testcase
  element in the XML report. Thanks `@kkoukiou`_ for the PR.

* Fix regression, pytest now skips unittest correctly if run with ``--pdb``
  (`#2137`_). Thanks to `@gst`_ for the report and `@mbyt`_ for the PR.

* Ignore exceptions raised from descriptors (e.g. properties) during Python test collection (`#2234`_).
  Thanks to `@bluetech`_.

* ``--override-ini`` now correctly overrides some fundamental options like ``python_files`` (`#2238`_).
  Thanks `@sirex`_ for the report and `@nicoddemus`_ for the PR.

* Replace ``raise StopIteration`` usages in the code by simple ``returns`` to finish generators, in accordance to `PEP-479`_ (`#2160`_).
  Thanks `@tgoodlet`_ for the report and `@nicoddemus`_ for the PR.

* Fix internal errors when an unprintable ``AssertionError`` is raised inside a test.
  Thanks `@omerhadari`_ for the PR.

* Skipping plugin now also works with test items generated by custom collectors (`#2231`_).
  Thanks to `@vidartf`_.

* Fix trailing whitespace in console output if no .ini file presented (`#2281`_). Thanks `@fbjorn`_ for the PR.

* Conditionless ``xfail`` markers no longer rely on the underlying test item
  being an instance of ``PyobjMixin``, and can therefore apply to tests not
  collected by the built-in python test collector. Thanks `@barneygale`_ for the
  PR.


.. _@pfhayes: https://github.com/pfhayes
.. _@bluetech: https://github.com/bluetech
.. _@gst: https://github.com/gst
.. _@sirex: https://github.com/sirex
.. _@vidartf: https://github.com/vidartf
.. _@kkoukiou: https://github.com/KKoukiou
.. _@omerhadari: https://github.com/omerhadari
.. _@fbjorn: https://github.com/fbjorn

.. _#2248: https://github.com/pytest-dev/pytest/issues/2248
.. _#2137: https://github.com/pytest-dev/pytest/issues/2137
.. _#2160: https://github.com/pytest-dev/pytest/issues/2160
.. _#2231: https://github.com/pytest-dev/pytest/issues/2231
.. _#2234: https://github.com/pytest-dev/pytest/issues/2234
.. _#2238: https://github.com/pytest-dev/pytest/issues/2238
.. _#2281: https://github.com/pytest-dev/pytest/issues/2281

.. _PEP-479: https://www.python.org/dev/peps/pep-0479/


3.0.6 (2017-01-22)
==================

* pytest no longer generates ``PendingDeprecationWarning`` from its own operations, which was introduced by mistake in version ``3.0.5`` (`#2118`_).
  Thanks to `@nicoddemus`_ for the report and `@RonnyPfannschmidt`_ for the PR.


* pytest no longer recognizes coroutine functions as yield tests (`#2129`_).
  Thanks to `@malinoff`_ for the PR.

* Plugins loaded by the ``PYTEST_PLUGINS`` environment variable are now automatically
  considered for assertion rewriting (`#2185`_).
  Thanks `@nicoddemus`_ for the PR.

* Improve error message when pytest.warns fails (`#2150`_). The type(s) of the
  expected warnings and the list of caught warnings is added to the
  error message. Thanks `@lesteve`_ for the PR.

* Fix ``pytester`` internal plugin to work correctly with latest versions of
  ``zope.interface`` (`#1989`_). Thanks `@nicoddemus`_ for the PR.

* Assert statements of the ``pytester`` plugin again benefit from assertion rewriting (`#1920`_).
  Thanks `@RonnyPfannschmidt`_ for the report and `@nicoddemus`_ for the PR.

* Specifying tests with colons like ``test_foo.py::test_bar`` for tests in
  subdirectories with ini configuration files now uses the correct ini file
  (`#2148`_).  Thanks `@pelme`_.

* Fail ``testdir.runpytest().assert_outcomes()`` explicitly if the pytest
  terminal output it relies on is missing. Thanks to `@eli-b`_ for the PR.


.. _@barneygale: https://github.com/barneygale
.. _@lesteve: https://github.com/lesteve
.. _@malinoff: https://github.com/malinoff
.. _@pelme: https://github.com/pelme
.. _@eli-b: https://github.com/eli-b

.. _#2118: https://github.com/pytest-dev/pytest/issues/2118

.. _#1989: https://github.com/pytest-dev/pytest/issues/1989
.. _#1920: https://github.com/pytest-dev/pytest/issues/1920
.. _#2129: https://github.com/pytest-dev/pytest/issues/2129
.. _#2148: https://github.com/pytest-dev/pytest/issues/2148
.. _#2150: https://github.com/pytest-dev/pytest/issues/2150
.. _#2185: https://github.com/pytest-dev/pytest/issues/2185


3.0.5 (2016-12-05)
==================

* Add warning when not passing ``option=value`` correctly to ``-o/--override-ini`` (`#2105`_).
  Also improved the help documentation. Thanks to `@mbukatov`_ for the report and
  `@lwm`_ for the PR.

* Now ``--confcutdir`` and ``--junit-xml`` are properly validated if they are directories
  and filenames, respectively (`#2089`_ and `#2078`_). Thanks to `@lwm`_ for the PR.

* Add hint to error message hinting possible missing ``__init__.py`` (`#478`_). Thanks `@DuncanBetts`_.

* More accurately describe when fixture finalization occurs in documentation (`#687`_). Thanks `@DuncanBetts`_.

* Provide ``:ref:`` targets for ``recwarn.rst`` so we can use intersphinx referencing.
  Thanks to `@dupuy`_ for the report and `@lwm`_ for the PR.

* In Python 2, use a simple ``+-`` ASCII string in the string representation of ``pytest.approx`` (for example ``"4 +- 4.0e-06"``)
  because it is brittle to handle that in different contexts and representations internally in pytest
  which can result in bugs such as `#2111`_. In Python 3, the representation still uses ``±`` (for example ``4 ± 4.0e-06``).
  Thanks `@kerrick-lyft`_ for the report and `@nicoddemus`_ for the PR.

* Using ``item.Function``, ``item.Module``, etc., is now issuing deprecation warnings, prefer
  ``pytest.Function``, ``pytest.Module``, etc., instead (`#2034`_).
  Thanks `@nmundar`_ for the PR.

* Fix error message using ``approx`` with complex numbers (`#2082`_).
  Thanks `@adler-j`_ for the report and `@nicoddemus`_ for the PR.

* Fixed false-positives warnings from assertion rewrite hook for modules imported more than
  once by the ``pytest_plugins`` mechanism.
  Thanks `@nicoddemus`_ for the PR.

* Remove an internal cache which could cause hooks from ``conftest.py`` files in
  sub-directories to be called in other directories incorrectly (`#2016`_).
  Thanks `@d-b-w`_ for the report and `@nicoddemus`_ for the PR.

* Remove internal code meant to support earlier Python 3 versions that produced the side effect
  of leaving ``None`` in ``sys.modules`` when expressions were evaluated by pytest (for example passing a condition
  as a string to ``pytest.mark.skipif``)(`#2103`_).
  Thanks `@jaraco`_ for the report and `@nicoddemus`_ for the PR.

* Cope gracefully with a .pyc file with no matching .py file (`#2038`_). Thanks
  `@nedbat`_.

.. _@syre: https://github.com/syre
.. _@adler-j: https://github.com/adler-j
.. _@d-b-w: https://bitbucket.org/d-b-w/
.. _@DuncanBetts: https://github.com/DuncanBetts
.. _@dupuy: https://bitbucket.org/dupuy/
.. _@kerrick-lyft: https://github.com/kerrick-lyft
.. _@lwm: https://github.com/lwm
.. _@mbukatov: https://github.com/mbukatov
.. _@nedbat: https://github.com/nedbat
.. _@nmundar: https://github.com/nmundar

.. _#2016: https://github.com/pytest-dev/pytest/issues/2016
.. _#2034: https://github.com/pytest-dev/pytest/issues/2034
.. _#2038: https://github.com/pytest-dev/pytest/issues/2038
.. _#2078: https://github.com/pytest-dev/pytest/issues/2078
.. _#2082: https://github.com/pytest-dev/pytest/issues/2082
.. _#2089: https://github.com/pytest-dev/pytest/issues/2089
.. _#2103: https://github.com/pytest-dev/pytest/issues/2103
.. _#2105: https://github.com/pytest-dev/pytest/issues/2105
.. _#2111: https://github.com/pytest-dev/pytest/issues/2111
.. _#478: https://github.com/pytest-dev/pytest/issues/478
.. _#687: https://github.com/pytest-dev/pytest/issues/687


3.0.4 (2016-11-09)
==================

* Import errors when collecting test modules now display the full traceback (`#1976`_).
  Thanks `@cwitty`_ for the report and `@nicoddemus`_ for the PR.

* Fix confusing command-line help message for custom options with two or more ``metavar`` properties (`#2004`_).
  Thanks `@okulynyak`_ and `@davehunt`_ for the report and `@nicoddemus`_ for the PR.

* When loading plugins, import errors which contain non-ascii messages are now properly handled in Python 2 (`#1998`_).
  Thanks `@nicoddemus`_ for the PR.

* Fixed cyclic reference when ``pytest.raises`` is used in context-manager form (`#1965`_). Also as a
  result of this fix, ``sys.exc_info()`` is left empty in both context-manager and function call usages.
  Previously, ``sys.exc_info`` would contain the exception caught by the context manager,
  even when the expected exception occurred.
  Thanks `@MSeifert04`_ for the report and the PR.

* Fixed false-positives warnings from assertion rewrite hook for modules that were rewritten but
  were later marked explicitly by ``pytest.register_assert_rewrite``
  or implicitly as a plugin (`#2005`_).
  Thanks `@RonnyPfannschmidt`_ for the report and `@nicoddemus`_ for the PR.

* Report teardown output on test failure (`#442`_).
  Thanks `@matclab`_ for the PR.

* Fix teardown error message in generated xUnit XML.
  Thanks `@gdyuldin`_ for the PR.

* Properly handle exceptions in ``multiprocessing`` tasks (`#1984`_).
  Thanks `@adborden`_ for the report and `@nicoddemus`_ for the PR.

* Clean up unittest TestCase objects after tests are complete (`#1649`_).
  Thanks `@d_b_w`_ for the report and PR.


.. _@adborden: https://github.com/adborden
.. _@cwitty: https://github.com/cwitty
.. _@d_b_w: https://github.com/d_b_w
.. _@gdyuldin: https://github.com/gdyuldin
.. _@matclab: https://github.com/matclab
.. _@MSeifert04: https://github.com/MSeifert04
.. _@okulynyak: https://github.com/okulynyak

.. _#442: https://github.com/pytest-dev/pytest/issues/442
.. _#1965: https://github.com/pytest-dev/pytest/issues/1965
.. _#1976: https://github.com/pytest-dev/pytest/issues/1976
.. _#1984: https://github.com/pytest-dev/pytest/issues/1984
.. _#1998: https://github.com/pytest-dev/pytest/issues/1998
.. _#2004: https://github.com/pytest-dev/pytest/issues/2004
.. _#2005: https://github.com/pytest-dev/pytest/issues/2005
.. _#1649: https://github.com/pytest-dev/pytest/issues/1649


3.0.3 (2016-09-28)
==================

* The ``ids`` argument to ``parametrize`` again accepts ``unicode`` strings
  in Python 2 (`#1905`_).
  Thanks `@philpep`_ for the report and `@nicoddemus`_ for the PR.

* Assertions are now being rewritten for plugins in development mode
  (``pip install -e``) (`#1934`_).
  Thanks `@nicoddemus`_ for the PR.

* Fix pkg_resources import error in Jython projects (`#1853`_).
  Thanks `@raquel-ucl`_ for the PR.

* Got rid of ``AttributeError: 'Module' object has no attribute '_obj'`` exception
  in Python 3 (`#1944`_).
  Thanks `@axil`_ for the PR.

* Explain a bad scope value passed to ``@fixture`` declarations or
  a ``MetaFunc.parametrize()`` call. Thanks `@tgoodlet`_ for the PR.

* This version includes ``pluggy-0.4.0``, which correctly handles
  ``VersionConflict`` errors in plugins (`#704`_).
  Thanks `@nicoddemus`_ for the PR.


.. _@philpep: https://github.com/philpep
.. _@raquel-ucl: https://github.com/raquel-ucl
.. _@axil: https://github.com/axil
.. _@tgoodlet: https://github.com/tgoodlet
.. _@vlad-dragos: https://github.com/vlad-dragos

.. _#1853: https://github.com/pytest-dev/pytest/issues/1853
.. _#1905: https://github.com/pytest-dev/pytest/issues/1905
.. _#1934: https://github.com/pytest-dev/pytest/issues/1934
.. _#1944: https://github.com/pytest-dev/pytest/issues/1944
.. _#704: https://github.com/pytest-dev/pytest/issues/704




3.0.2 (2016-09-01)
==================

* Improve error message when passing non-string ids to ``pytest.mark.parametrize`` (`#1857`_).
  Thanks `@okken`_ for the report and `@nicoddemus`_ for the PR.

* Add ``buffer`` attribute to stdin stub class ``pytest.capture.DontReadFromInput``
  Thanks `@joguSD`_ for the PR.

* Fix ``UnicodeEncodeError`` when string comparison with unicode has failed. (`#1864`_)
  Thanks `@AiOO`_ for the PR.

* ``pytest_plugins`` is now handled correctly if defined as a string (as opposed as
  a sequence of strings) when modules are considered for assertion rewriting.
  Due to this bug, much more modules were being rewritten than necessary
  if a test suite uses ``pytest_plugins`` to load internal plugins (`#1888`_).
  Thanks `@jaraco`_ for the report and `@nicoddemus`_ for the PR (`#1891`_).

* Do not call tearDown and cleanups when running tests from
  ``unittest.TestCase`` subclasses with ``--pdb``
  enabled. This allows proper post mortem debugging for all applications
  which have significant logic in their tearDown machinery (`#1890`_). Thanks
  `@mbyt`_ for the PR.

* Fix use of deprecated ``getfuncargvalue`` method in the internal doctest plugin.
  Thanks `@ViviCoder`_ for the report (`#1898`_).

.. _@joguSD: https://github.com/joguSD
.. _@AiOO: https://github.com/AiOO
.. _@mbyt: https://github.com/mbyt
.. _@ViviCoder: https://github.com/ViviCoder

.. _#1857: https://github.com/pytest-dev/pytest/issues/1857
.. _#1864: https://github.com/pytest-dev/pytest/issues/1864
.. _#1888: https://github.com/pytest-dev/pytest/issues/1888
.. _#1891: https://github.com/pytest-dev/pytest/pull/1891
.. _#1890: https://github.com/pytest-dev/pytest/issues/1890
.. _#1898: https://github.com/pytest-dev/pytest/issues/1898


3.0.1 (2016-08-23)
==================

* Fix regression when ``importorskip`` is used at module level (`#1822`_).
  Thanks `@jaraco`_ and `@The-Compiler`_ for the report and `@nicoddemus`_ for the PR.

* Fix parametrization scope when session fixtures are used in conjunction
  with normal parameters in the same call (`#1832`_).
  Thanks `@The-Compiler`_ for the report, `@Kingdread`_ and `@nicoddemus`_ for the PR.

* Fix internal error when parametrizing tests or fixtures using an empty ``ids`` argument (`#1849`_).
  Thanks `@OPpuolitaival`_ for the report and `@nicoddemus`_ for the PR.

* Fix loader error when running ``pytest`` embedded in a zipfile.
  Thanks `@mbachry`_ for the PR.


.. _@Kingdread: https://github.com/Kingdread
.. _@mbachry: https://github.com/mbachry
.. _@OPpuolitaival: https://github.com/OPpuolitaival

.. _#1822: https://github.com/pytest-dev/pytest/issues/1822
.. _#1832: https://github.com/pytest-dev/pytest/issues/1832
.. _#1849: https://github.com/pytest-dev/pytest/issues/1849


3.0.0 (2016-08-18)
==================

**Incompatible changes**


A number of incompatible changes were made in this release, with the intent of removing features deprecated for a long
time or change existing behaviors in order to make them less surprising/more useful.

* Reinterpretation mode has now been removed.  Only plain and rewrite
  mode are available, consequently the ``--assert=reinterp`` option is
  no longer available.  This also means files imported from plugins or
  ``conftest.py`` will not benefit from improved assertions by
  default, you should use ``pytest.register_assert_rewrite()`` to
  explicitly turn on assertion rewriting for those files.  Thanks
  `@flub`_ for the PR.

* The following deprecated commandline options were removed:

  * ``--genscript``: no longer supported;
  * ``--no-assert``: use ``--assert=plain`` instead;
  * ``--nomagic``: use ``--assert=plain`` instead;
  * ``--report``: use ``-r`` instead;

  Thanks to `@RedBeardCode`_ for the PR (`#1664`_).

* ImportErrors in plugins now are a fatal error instead of issuing a
  pytest warning (`#1479`_). Thanks to `@The-Compiler`_ for the PR.

* Removed support code for Python 3 versions < 3.3 (`#1627`_).

* Removed all ``py.test-X*`` entry points. The versioned, suffixed entry points
  were never documented and a leftover from a pre-virtualenv era. These entry
  points also created broken entry points in wheels, so removing them also
  removes a source of confusion for users (`#1632`_).
  Thanks `@obestwalter`_ for the PR.

* ``pytest.skip()`` now raises an error when used to decorate a test function,
  as opposed to its original intent (to imperatively skip a test inside a test function). Previously
  this usage would cause the entire module to be skipped (`#607`_).
  Thanks `@omarkohl`_ for the complete PR (`#1519`_).

* Exit tests if a collection error occurs. A poll indicated most users will hit CTRL-C
  anyway as soon as they see collection errors, so pytest might as well make that the default behavior (`#1421`_).
  A ``--continue-on-collection-errors`` option has been added to restore the previous behaviour.
  Thanks `@olegpidsadnyi`_ and `@omarkohl`_ for the complete PR (`#1628`_).

* Renamed the pytest ``pdb`` module (plugin) into ``debugging`` to avoid clashes with the builtin ``pdb`` module.

* Raise a helpful failure message when requesting a parametrized fixture at runtime,
  e.g. with ``request.getfixturevalue``. Previously these parameters were simply
  never defined, so a fixture decorated like ``@pytest.fixture(params=[0, 1, 2])``
  only ran once (`#460`_).
  Thanks to `@nikratio`_ for the bug report, `@RedBeardCode`_ and `@tomviner`_ for the PR.

* ``_pytest.monkeypatch.monkeypatch`` class has been renamed to ``_pytest.monkeypatch.MonkeyPatch``
  so it doesn't conflict with the ``monkeypatch`` fixture.

* ``--exitfirst / -x`` can now be overridden by a following ``--maxfail=N``
  and is just a synonym for ``--maxfail=1``.


**New Features**

* Support nose-style ``__test__`` attribute on methods of classes,
  including unittest-style Classes. If set to ``False``, the test will not be
  collected.

* New ``doctest_namespace`` fixture for injecting names into the
  namespace in which doctests run.
  Thanks `@milliams`_ for the complete PR (`#1428`_).

* New ``--doctest-report`` option available to change the output format of diffs
  when running (failing) doctests (implements `#1749`_).
  Thanks `@hartym`_ for the PR.

* New ``name`` argument to ``pytest.fixture`` decorator which allows a custom name
  for a fixture (to solve the funcarg-shadowing-fixture problem).
  Thanks `@novas0x2a`_ for the complete PR (`#1444`_).

* New ``approx()`` function for easily comparing floating-point numbers in
  tests.
  Thanks `@kalekundert`_ for the complete PR (`#1441`_).

* Ability to add global properties in the final xunit output file by accessing
  the internal ``junitxml`` plugin (experimental).
  Thanks `@tareqalayan`_ for the complete PR `#1454`_).

* New ``ExceptionInfo.match()`` method to match a regular expression on the
  string representation of an exception (`#372`_).
  Thanks `@omarkohl`_ for the complete PR (`#1502`_).

* ``__tracebackhide__`` can now also be set to a callable which then can decide
  whether to filter the traceback based on the ``ExceptionInfo`` object passed
  to it. Thanks `@The-Compiler`_ for the complete PR (`#1526`_).

* New ``pytest_make_parametrize_id(config, val)`` hook which can be used by plugins to provide
  friendly strings for custom types.
  Thanks `@palaviv`_ for the PR.

* ``capsys`` and ``capfd`` now have a ``disabled()`` context-manager method, which
  can be used to temporarily disable capture within a test.
  Thanks `@nicoddemus`_ for the PR.

* New cli flag ``--fixtures-per-test``: shows which fixtures are being used
  for each selected test item. Features doc strings of fixtures by default.
  Can also show where fixtures are defined if combined with ``-v``.
  Thanks `@hackebrot`_ for the PR.

* Introduce ``pytest`` command as recommended entry point. Note that ``py.test``
  still works and is not scheduled for removal. Closes proposal
  `#1629`_. Thanks `@obestwalter`_ and `@davehunt`_ for the complete PR
  (`#1633`_).

* New cli flags:

  + ``--setup-plan``: performs normal collection and reports
    the potential setup and teardown and does not execute any fixtures and tests;
  + ``--setup-only``: performs normal collection, executes setup and teardown of
    fixtures and reports them;
  + ``--setup-show``: performs normal test execution and additionally shows
    setup and teardown of fixtures;
  + ``--keep-duplicates``: py.test now ignores duplicated paths given in the command
    line. To retain the previous behavior where the same test could be run multiple
    times by specifying it in the command-line multiple times, pass the ``--keep-duplicates``
    argument (`#1609`_);

  Thanks `@d6e`_, `@kvas-it`_, `@sallner`_, `@ioggstream`_ and `@omarkohl`_ for the PRs.

* New CLI flag ``--override-ini``/``-o``: overrides values from the ini file.
  For example: ``"-o xfail_strict=True"``'.
  Thanks `@blueyed`_ and `@fengxx`_ for the PR.

* New hooks:

  + ``pytest_fixture_setup(fixturedef, request)``: executes fixture setup;
  + ``pytest_fixture_post_finalizer(fixturedef)``: called after the fixture's
    finalizer and has access to the fixture's result cache.

  Thanks `@d6e`_, `@sallner`_.

* Issue warnings for asserts whose test is a tuple literal. Such asserts will
  never fail because tuples are always truthy and are usually a mistake
  (see `#1562`_). Thanks `@kvas-it`_, for the PR.

* Allow passing a custom debugger class (e.g. ``--pdbcls=IPython.core.debugger:Pdb``).
  Thanks to `@anntzer`_ for the PR.


**Changes**

* Plugins now benefit from assertion rewriting.  Thanks
  `@sober7`_, `@nicoddemus`_ and `@flub`_ for the PR.

* Change ``report.outcome`` for ``xpassed`` tests to ``"passed"`` in non-strict
  mode and ``"failed"`` in strict mode. Thanks to `@hackebrot`_ for the PR
  (`#1795`_) and `@gprasad84`_ for report (`#1546`_).

* Tests marked with ``xfail(strict=False)`` (the default) now appear in
  JUnitXML reports as passing tests instead of skipped.
  Thanks to `@hackebrot`_ for the PR (`#1795`_).

* Highlight path of the file location in the error report to make it easier to copy/paste.
  Thanks `@suzaku`_ for the PR (`#1778`_).

* Fixtures marked with ``@pytest.fixture`` can now use ``yield`` statements exactly like
  those marked with the ``@pytest.yield_fixture`` decorator. This change renders
  ``@pytest.yield_fixture`` deprecated and makes ``@pytest.fixture`` with ``yield`` statements
  the preferred way to write teardown code (`#1461`_).
  Thanks `@csaftoiu`_ for bringing this to attention and `@nicoddemus`_ for the PR.

* Explicitly passed parametrize ids do not get escaped to ascii (`#1351`_).
  Thanks `@ceridwen`_ for the PR.

* Fixtures are now sorted in the error message displayed when an unknown
  fixture is declared in a test function.
  Thanks `@nicoddemus`_ for the PR.

* ``pytest_terminal_summary`` hook now receives the ``exitstatus``
  of the test session as argument. Thanks `@blueyed`_ for the PR (`#1809`_).

* Parametrize ids can accept ``None`` as specific test id, in which case the
  automatically generated id for that argument will be used.
  Thanks `@palaviv`_ for the complete PR (`#1468`_).

* The parameter to xunit-style setup/teardown methods (``setup_method``,
  ``setup_module``, etc.) is now optional and may be omitted.
  Thanks `@okken`_ for bringing this to attention and `@nicoddemus`_ for the PR.

* Improved automatic id generation selection in case of duplicate ids in
  parametrize.
  Thanks `@palaviv`_ for the complete PR (`#1474`_).

* Now pytest warnings summary is shown up by default. Added a new flag
  ``--disable-pytest-warnings`` to explicitly disable the warnings summary (`#1668`_).

* Make ImportError during collection more explicit by reminding
  the user to check the name of the test module/package(s) (`#1426`_).
  Thanks `@omarkohl`_ for the complete PR (`#1520`_).

* Add ``build/`` and ``dist/`` to the default ``--norecursedirs`` list. Thanks
  `@mikofski`_ for the report and `@tomviner`_ for the PR (`#1544`_).

* ``pytest.raises`` in the context manager form accepts a custom
  ``message`` to raise when no exception occurred.
  Thanks `@palaviv`_ for the complete PR (`#1616`_).

* ``conftest.py`` files now benefit from assertion rewriting; previously it
  was only available for test modules. Thanks `@flub`_, `@sober7`_ and
  `@nicoddemus`_ for the PR (`#1619`_).

* Text documents without any doctests no longer appear as "skipped".
  Thanks `@graingert`_ for reporting and providing a full PR (`#1580`_).

* Ensure that a module within a namespace package can be found when it
  is specified on the command line together with the ``--pyargs``
  option.  Thanks to `@taschini`_ for the PR (`#1597`_).

* Always include full assertion explanation during assertion rewriting. The previous behaviour was hiding
  sub-expressions that happened to be ``False``, assuming this was redundant information.
  Thanks `@bagerard`_ for reporting (`#1503`_). Thanks to `@davehunt`_ and
  `@tomviner`_ for the PR.

* ``OptionGroup.addoption()`` now checks if option names were already
  added before, to make it easier to track down issues like `#1618`_.
  Before, you only got exceptions later from ``argparse`` library,
  giving no clue about the actual reason for double-added options.

* ``yield``-based tests are considered deprecated and will be removed in pytest-4.0.
  Thanks `@nicoddemus`_ for the PR.

* ``[pytest]`` sections in ``setup.cfg`` files should now be named ``[tool:pytest]``
  to avoid conflicts with other distutils commands (see `#567`_). ``[pytest]`` sections in
  ``pytest.ini`` or ``tox.ini`` files are supported and unchanged.
  Thanks `@nicoddemus`_ for the PR.

* Using ``pytest_funcarg__`` prefix to declare fixtures is considered deprecated and will be
  removed in pytest-4.0 (`#1684`_).
  Thanks `@nicoddemus`_ for the PR.

* Passing a command-line string to ``pytest.main()`` is considered deprecated and scheduled
  for removal in pytest-4.0. It is recommended to pass a list of arguments instead (`#1723`_).

* Rename ``getfuncargvalue`` to ``getfixturevalue``. ``getfuncargvalue`` is
  still present but is now considered deprecated. Thanks to `@RedBeardCode`_ and `@tomviner`_
  for the PR (`#1626`_).

* ``optparse`` type usage now triggers DeprecationWarnings (`#1740`_).


* ``optparse`` backward compatibility supports float/complex types (`#457`_).

* Refined logic for determining the ``rootdir``, considering only valid
  paths which fixes a number of issues: `#1594`_, `#1435`_ and `#1471`_.
  Updated the documentation according to current behavior. Thanks to
  `@blueyed`_, `@davehunt`_ and `@matthiasha`_ for the PR.

* Always include full assertion explanation. The previous behaviour was hiding
  sub-expressions that happened to be False, assuming this was redundant information.
  Thanks `@bagerard`_ for reporting (`#1503`_). Thanks to `@davehunt`_ and
  `@tomviner`_ for PR.

* Better message in case of not using parametrized variable (see `#1539`_).
  Thanks to `@tramwaj29`_ for the PR.

* Updated docstrings with a more uniform style.

* Add stderr write for ``pytest.exit(msg)`` during startup. Previously the message was never shown.
  Thanks `@BeyondEvil`_ for reporting `#1210`_. Thanks to `@JonathonSonesen`_ and
  `@tomviner`_ for the PR.

* No longer display the incorrect test deselection reason (`#1372`_).
  Thanks `@ronnypfannschmidt`_ for the PR.

* The ``--resultlog`` command line option has been deprecated: it is little used
  and there are more modern and better alternatives (see `#830`_).
  Thanks `@nicoddemus`_ for the PR.

* Improve error message with fixture lookup errors: add an 'E' to the first
  line and '>' to the rest. Fixes `#717`_. Thanks `@blueyed`_ for reporting and
  a PR, `@eolo999`_ for the initial PR and `@tomviner`_ for his guidance during
  EuroPython2016 sprint.


**Bug Fixes**

* Parametrize now correctly handles duplicated test ids.

* Fix internal error issue when the ``method`` argument is missing for
  ``teardown_method()`` (`#1605`_).

* Fix exception visualization in case the current working directory (CWD) gets
  deleted during testing (`#1235`_). Thanks `@bukzor`_ for reporting. PR by
  `@marscher`_.

* Improve test output for logical expression with brackets (`#925`_).
  Thanks `@DRMacIver`_ for reporting and `@RedBeardCode`_ for the PR.

* Create correct diff for strings ending with newlines (`#1553`_).
  Thanks `@Vogtinator`_ for reporting and `@RedBeardCode`_ and
  `@tomviner`_ for the PR.

* ``ConftestImportFailure`` now shows the traceback making it easier to
  identify bugs in ``conftest.py`` files (`#1516`_). Thanks `@txomon`_ for
  the PR.

* Text documents without any doctests no longer appear as "skipped".
  Thanks `@graingert`_ for reporting and providing a full PR (`#1580`_).

* Fixed collection of classes with custom ``__new__`` method.
  Fixes `#1579`_. Thanks to `@Stranger6667`_ for the PR.

* Fixed scope overriding inside metafunc.parametrize (`#634`_).
  Thanks to `@Stranger6667`_ for the PR.

* Fixed the total tests tally in junit xml output (`#1798`_).
  Thanks to `@cryporchild`_ for the PR.

* Fixed off-by-one error with lines from ``request.node.warn``.
  Thanks to `@blueyed`_ for the PR.


.. _#1210: https://github.com/pytest-dev/pytest/issues/1210
.. _#1235: https://github.com/pytest-dev/pytest/issues/1235
.. _#1351: https://github.com/pytest-dev/pytest/issues/1351
.. _#1372: https://github.com/pytest-dev/pytest/issues/1372
.. _#1421: https://github.com/pytest-dev/pytest/issues/1421
.. _#1426: https://github.com/pytest-dev/pytest/issues/1426
.. _#1428: https://github.com/pytest-dev/pytest/pull/1428
.. _#1435: https://github.com/pytest-dev/pytest/issues/1435
.. _#1441: https://github.com/pytest-dev/pytest/pull/1441
.. _#1444: https://github.com/pytest-dev/pytest/pull/1444
.. _#1454: https://github.com/pytest-dev/pytest/pull/1454
.. _#1461: https://github.com/pytest-dev/pytest/pull/1461
.. _#1468: https://github.com/pytest-dev/pytest/pull/1468
.. _#1471: https://github.com/pytest-dev/pytest/issues/1471
.. _#1474: https://github.com/pytest-dev/pytest/pull/1474
.. _#1479: https://github.com/pytest-dev/pytest/issues/1479
.. _#1502: https://github.com/pytest-dev/pytest/pull/1502
.. _#1503: https://github.com/pytest-dev/pytest/issues/1503
.. _#1516: https://github.com/pytest-dev/pytest/pull/1516
.. _#1519: https://github.com/pytest-dev/pytest/pull/1519
.. _#1520: https://github.com/pytest-dev/pytest/pull/1520
.. _#1526: https://github.com/pytest-dev/pytest/pull/1526
.. _#1539: https://github.com/pytest-dev/pytest/issues/1539
.. _#1544: https://github.com/pytest-dev/pytest/issues/1544
.. _#1546: https://github.com/pytest-dev/pytest/issues/1546
.. _#1553: https://github.com/pytest-dev/pytest/issues/1553
.. _#1562: https://github.com/pytest-dev/pytest/issues/1562
.. _#1579: https://github.com/pytest-dev/pytest/issues/1579
.. _#1580: https://github.com/pytest-dev/pytest/pull/1580
.. _#1594: https://github.com/pytest-dev/pytest/issues/1594
.. _#1597: https://github.com/pytest-dev/pytest/pull/1597
.. _#1605: https://github.com/pytest-dev/pytest/issues/1605
.. _#1616: https://github.com/pytest-dev/pytest/pull/1616
.. _#1618: https://github.com/pytest-dev/pytest/issues/1618
.. _#1619: https://github.com/pytest-dev/pytest/issues/1619
.. _#1626: https://github.com/pytest-dev/pytest/pull/1626
.. _#1627: https://github.com/pytest-dev/pytest/pull/1627
.. _#1628: https://github.com/pytest-dev/pytest/pull/1628
.. _#1629: https://github.com/pytest-dev/pytest/issues/1629
.. _#1632: https://github.com/pytest-dev/pytest/issues/1632
.. _#1633: https://github.com/pytest-dev/pytest/pull/1633
.. _#1664: https://github.com/pytest-dev/pytest/pull/1664
.. _#1668: https://github.com/pytest-dev/pytest/issues/1668
.. _#1684: https://github.com/pytest-dev/pytest/pull/1684
.. _#1723: https://github.com/pytest-dev/pytest/pull/1723
.. _#1740: https://github.com/pytest-dev/pytest/issues/1740
.. _#1749: https://github.com/pytest-dev/pytest/issues/1749
.. _#1778: https://github.com/pytest-dev/pytest/pull/1778
.. _#1795: https://github.com/pytest-dev/pytest/pull/1795
.. _#1798: https://github.com/pytest-dev/pytest/pull/1798
.. _#1809: https://github.com/pytest-dev/pytest/pull/1809
.. _#372: https://github.com/pytest-dev/pytest/issues/372
.. _#457: https://github.com/pytest-dev/pytest/issues/457
.. _#460: https://github.com/pytest-dev/pytest/pull/460
.. _#567: https://github.com/pytest-dev/pytest/pull/567
.. _#607: https://github.com/pytest-dev/pytest/issues/607
.. _#634: https://github.com/pytest-dev/pytest/issues/634
.. _#717: https://github.com/pytest-dev/pytest/issues/717
.. _#830: https://github.com/pytest-dev/pytest/issues/830
.. _#925: https://github.com/pytest-dev/pytest/issues/925


.. _@anntzer: https://github.com/anntzer
.. _@bagerard: https://github.com/bagerard
.. _@BeyondEvil: https://github.com/BeyondEvil
.. _@blueyed: https://github.com/blueyed
.. _@ceridwen: https://github.com/ceridwen
.. _@cryporchild: https://github.com/cryporchild
.. _@csaftoiu: https://github.com/csaftoiu
.. _@d6e: https://github.com/d6e
.. _@davehunt: https://github.com/davehunt
.. _@DRMacIver: https://github.com/DRMacIver
.. _@eolo999: https://github.com/eolo999
.. _@fengxx: https://github.com/fengxx
.. _@flub: https://github.com/flub
.. _@gprasad84: https://github.com/gprasad84
.. _@graingert: https://github.com/graingert
.. _@hartym: https://github.com/hartym
.. _@JonathonSonesen: https://github.com/JonathonSonesen
.. _@kalekundert: https://github.com/kalekundert
.. _@kvas-it: https://github.com/kvas-it
.. _@marscher: https://github.com/marscher
.. _@mikofski: https://github.com/mikofski
.. _@milliams: https://github.com/milliams
.. _@nikratio: https://github.com/nikratio
.. _@novas0x2a: https://github.com/novas0x2a
.. _@obestwalter: https://github.com/obestwalter
.. _@okken: https://github.com/okken
.. _@olegpidsadnyi: https://github.com/olegpidsadnyi
.. _@omarkohl: https://github.com/omarkohl
.. _@palaviv: https://github.com/palaviv
.. _@RedBeardCode: https://github.com/RedBeardCode
.. _@sallner: https://github.com/sallner
.. _@sober7: https://github.com/sober7
.. _@Stranger6667: https://github.com/Stranger6667
.. _@suzaku: https://github.com/suzaku
.. _@tareqalayan: https://github.com/tareqalayan
.. _@taschini: https://github.com/taschini
.. _@tramwaj29: https://github.com/tramwaj29
.. _@txomon: https://github.com/txomon
.. _@Vogtinator: https://github.com/Vogtinator
.. _@matthiasha: https://github.com/matthiasha


2.9.2 (2016-05-31)
==================

**Bug Fixes**

* fix `#510`_: skip tests where one parameterize dimension was empty
  thanks Alex Stapleton for the Report and `@RonnyPfannschmidt`_ for the PR

* Fix Xfail does not work with condition keyword argument.
  Thanks `@astraw38`_ for reporting the issue (`#1496`_) and `@tomviner`_
  for PR the (`#1524`_).

* Fix win32 path issue when putting custom config file with absolute path
  in ``pytest.main("-c your_absolute_path")``.

* Fix maximum recursion depth detection when raised error class is not aware
  of unicode/encoded bytes.
  Thanks `@prusse-martin`_ for the PR (`#1506`_).

* Fix ``pytest.mark.skip`` mark when used in strict mode.
  Thanks `@pquentin`_ for the PR and `@RonnyPfannschmidt`_ for
  showing how to fix the bug.

* Minor improvements and fixes to the documentation.
  Thanks `@omarkohl`_ for the PR.

* Fix ``--fixtures`` to show all fixture definitions as opposed to just
  one per fixture name.
  Thanks to `@hackebrot`_ for the PR.

.. _#510: https://github.com/pytest-dev/pytest/issues/510
.. _#1506: https://github.com/pytest-dev/pytest/pull/1506
.. _#1496: https://github.com/pytest-dev/pytest/issues/1496
.. _#1524: https://github.com/pytest-dev/pytest/pull/1524

.. _@prusse-martin: https://github.com/prusse-martin
.. _@astraw38: https://github.com/astraw38


2.9.1 (2016-03-17)
==================

**Bug Fixes**

* Improve error message when a plugin fails to load.
  Thanks `@nicoddemus`_ for the PR.

* Fix (`#1178 <https://github.com/pytest-dev/pytest/issues/1178>`_):
  ``pytest.fail`` with non-ascii characters raises an internal pytest error.
  Thanks `@nicoddemus`_ for the PR.

* Fix (`#469`_): junit parses report.nodeid incorrectly, when params IDs
  contain ``::``. Thanks `@tomviner`_ for the PR (`#1431`_).

* Fix (`#578 <https://github.com/pytest-dev/pytest/issues/578>`_): SyntaxErrors
  containing non-ascii lines at the point of failure generated an internal
  py.test error.
  Thanks `@asottile`_ for the report and `@nicoddemus`_ for the PR.

* Fix (`#1437`_): When passing in a bytestring regex pattern to parameterize
  attempt to decode it as utf-8 ignoring errors.

* Fix (`#649`_): parametrized test nodes cannot be specified to run on the command line.

* Fix (`#138`_): better reporting for python 3.3+ chained exceptions

.. _#1437: https://github.com/pytest-dev/pytest/issues/1437
.. _#469: https://github.com/pytest-dev/pytest/issues/469
.. _#1431: https://github.com/pytest-dev/pytest/pull/1431
.. _#649: https://github.com/pytest-dev/pytest/issues/649
.. _#138: https://github.com/pytest-dev/pytest/issues/138

.. _@asottile: https://github.com/asottile


2.9.0 (2016-02-29)
==================

**New Features**

* New ``pytest.mark.skip`` mark, which unconditionally skips marked tests.
  Thanks `@MichaelAquilina`_ for the complete PR (`#1040`_).

* ``--doctest-glob`` may now be passed multiple times in the command-line.
  Thanks `@jab`_ and `@nicoddemus`_ for the PR.

* New ``-rp`` and ``-rP`` reporting options give the summary and full output
  of passing tests, respectively. Thanks to `@codewarrior0`_ for the PR.

* ``pytest.mark.xfail`` now has a ``strict`` option, which makes ``XPASS``
  tests to fail the test suite (defaulting to ``False``). There's also a
  ``xfail_strict`` ini option that can be used to configure it project-wise.
  Thanks `@rabbbit`_ for the request and `@nicoddemus`_ for the PR (`#1355`_).

* ``Parser.addini`` now supports options of type ``bool``.
  Thanks `@nicoddemus`_ for the PR.

* New ``ALLOW_BYTES`` doctest option. This strips ``b`` prefixes from byte strings
  in doctest output (similar to ``ALLOW_UNICODE``).
  Thanks `@jaraco`_ for the request and `@nicoddemus`_ for the PR (`#1287`_).

* Give a hint on ``KeyboardInterrupt`` to use the ``--fulltrace`` option to show the errors.
  Fixes `#1366`_.
  Thanks to `@hpk42`_ for the report and `@RonnyPfannschmidt`_ for the PR.

* Catch ``IndexError`` exceptions when getting exception source location.
  Fixes a pytest internal error for dynamically generated code (fixtures and tests)
  where source lines are fake by intention.

**Changes**

* **Important**: `py.code <https://pylib.readthedocs.io/en/latest/code.html>`_ has been
  merged into the ``pytest`` repository as ``pytest._code``. This decision
  was made because ``py.code`` had very few uses outside ``pytest`` and the
  fact that it was in a different repository made it difficult to fix bugs on
  its code in a timely manner. The team hopes with this to be able to better
  refactor out and improve that code.
  This change shouldn't affect users, but it is useful to let users aware
  if they encounter any strange behavior.

  Keep in mind that the code for ``pytest._code`` is **private** and
  **experimental**, so you definitely should not import it explicitly!

  Please note that the original ``py.code`` is still available in
  `pylib <https://pylib.readthedocs.io>`_.

* ``pytest_enter_pdb`` now optionally receives the pytest config object.
  Thanks `@nicoddemus`_ for the PR.

* Removed code and documentation for Python 2.5 or lower versions,
  including removal of the obsolete ``_pytest.assertion.oldinterpret`` module.
  Thanks `@nicoddemus`_ for the PR (`#1226`_).

* Comparisons now always show up in full when ``CI`` or ``BUILD_NUMBER`` is
  found in the environment, even when ``-vv`` isn't used.
  Thanks `@The-Compiler`_ for the PR.

* ``--lf`` and ``--ff`` now support long names: ``--last-failed`` and
  ``--failed-first`` respectively.
  Thanks `@MichaelAquilina`_ for the PR.

* Added expected exceptions to ``pytest.raises`` fail message.

* Collection only displays progress ("collecting X items") when in a terminal.
  This avoids cluttering the output when using ``--color=yes`` to obtain
  colors in CI integrations systems (`#1397`_).

**Bug Fixes**

* The ``-s`` and ``-c`` options should now work under ``xdist``;
  ``Config.fromdictargs`` now represents its input much more faithfully.
  Thanks to `@bukzor`_ for the complete PR (`#680`_).

* Fix (`#1290`_): support Python 3.5's ``@`` operator in assertion rewriting.
  Thanks `@Shinkenjoe`_ for report with test case and `@tomviner`_ for the PR.

* Fix formatting utf-8 explanation messages (`#1379`_).
  Thanks `@biern`_ for the PR.

* Fix `traceback style docs`_ to describe all of the available options
  (auto/long/short/line/native/no), with ``auto`` being the default since v2.6.
  Thanks `@hackebrot`_ for the PR.

* Fix (`#1422`_): junit record_xml_property doesn't allow multiple records
  with same name.

.. _`traceback style docs`: https://pytest.org/latest/usage.html#modifying-python-traceback-printing

.. _#1609: https://github.com/pytest-dev/pytest/issues/1609
.. _#1422: https://github.com/pytest-dev/pytest/issues/1422
.. _#1379: https://github.com/pytest-dev/pytest/issues/1379
.. _#1366: https://github.com/pytest-dev/pytest/issues/1366
.. _#1040: https://github.com/pytest-dev/pytest/pull/1040
.. _#680: https://github.com/pytest-dev/pytest/issues/680
.. _#1287: https://github.com/pytest-dev/pytest/pull/1287
.. _#1226: https://github.com/pytest-dev/pytest/pull/1226
.. _#1290: https://github.com/pytest-dev/pytest/pull/1290
.. _#1355: https://github.com/pytest-dev/pytest/pull/1355
.. _#1397: https://github.com/pytest-dev/pytest/issues/1397
.. _@biern: https://github.com/biern
.. _@MichaelAquilina: https://github.com/MichaelAquilina
.. _@bukzor: https://github.com/bukzor
.. _@hpk42: https://github.com/hpk42
.. _@nicoddemus: https://github.com/nicoddemus
.. _@jab: https://github.com/jab
.. _@codewarrior0: https://github.com/codewarrior0
.. _@jaraco: https://github.com/jaraco
.. _@The-Compiler: https://github.com/The-Compiler
.. _@Shinkenjoe: https://github.com/Shinkenjoe
.. _@tomviner: https://github.com/tomviner
.. _@RonnyPfannschmidt: https://github.com/RonnyPfannschmidt
.. _@rabbbit: https://github.com/rabbbit
.. _@hackebrot: https://github.com/hackebrot
.. _@pquentin: https://github.com/pquentin
.. _@ioggstream: https://github.com/ioggstream

2.8.7 (2016-01-24)
==================

- fix #1338: use predictable object resolution for monkeypatch

2.8.6 (2016-01-21)
==================

- fix #1259: allow for double nodeids in junitxml,
  this was a regression failing plugins combinations
  like pytest-pep8 + pytest-flakes

- Workaround for exception that occurs in pyreadline when using
  ``--pdb`` with standard I/O capture enabled.
  Thanks Erik M. Bray for the PR.

- fix #900: Better error message in case the target of a ``monkeypatch`` call
  raises an ``ImportError``.

- fix #1292: monkeypatch calls (setattr, setenv, etc.) are now O(1).
  Thanks David R. MacIver for the report and Bruno Oliveira for the PR.

- fix #1223: captured stdout and stderr are now properly displayed before
  entering pdb when ``--pdb`` is used instead of being thrown away.
  Thanks Cal Leeming for the PR.

- fix #1305: pytest warnings emitted during ``pytest_terminal_summary`` are now
  properly displayed.
  Thanks Ionel Maries Cristian for the report and Bruno Oliveira for the PR.

- fix #628: fixed internal UnicodeDecodeError when doctests contain unicode.
  Thanks Jason R. Coombs for the report and Bruno Oliveira for the PR.

- fix #1334: Add captured stdout to jUnit XML report on setup error.
  Thanks Georgy Dyuldin for the PR.


2.8.5 (2015-12-11)
==================

- fix #1243: fixed issue where class attributes injected during collection could break pytest.
  PR by Alexei Kozlenok, thanks Ronny Pfannschmidt and Bruno Oliveira for the review and help.

- fix #1074: precompute junitxml chunks instead of storing the whole tree in objects
  Thanks Bruno Oliveira for the report and Ronny Pfannschmidt for the PR

- fix #1238: fix ``pytest.deprecated_call()`` receiving multiple arguments
  (Regression introduced in 2.8.4). Thanks Alex Gaynor for the report and
  Bruno Oliveira for the PR.


2.8.4 (2015-12-06)
==================

- fix #1190: ``deprecated_call()`` now works when the deprecated
  function has been already called by another test in the same
  module. Thanks Mikhail Chernykh for the report and Bruno Oliveira for the
  PR.

- fix #1198: ``--pastebin`` option now works on Python 3. Thanks
  Mehdy Khoshnoody for the PR.

- fix #1219: ``--pastebin`` now works correctly when captured output contains
  non-ascii characters. Thanks Bruno Oliveira for the PR.

- fix #1204: another error when collecting with a nasty __getattr__().
  Thanks Florian Bruhin for the PR.

- fix the summary printed when no tests did run.
  Thanks Florian Bruhin for the PR.
- fix #1185 - ensure MANIFEST.in exactly matches what should go to a sdist

- a number of documentation modernizations wrt good practices.
  Thanks Bruno Oliveira for the PR.

2.8.3 (2015-11-18)
==================

- fix #1169: add __name__ attribute to testcases in TestCaseFunction to
  support the @unittest.skip decorator on functions and methods.
  Thanks Lee Kamentsky for the PR.

- fix #1035: collecting tests if test module level obj has __getattr__().
  Thanks Suor for the report and Bruno Oliveira / Tom Viner for the PR.

- fix #331: don't collect tests if their failure cannot be reported correctly
  e.g. they are a callable instance of a class.

- fix #1133: fixed internal error when filtering tracebacks where one entry
  belongs to a file which is no longer available.
  Thanks Bruno Oliveira for the PR.

- enhancement made to highlight in red the name of the failing tests so
  they stand out in the output.
  Thanks Gabriel Reis for the PR.

- add more talks to the documentation
- extend documentation on the --ignore cli option
- use pytest-runner for setuptools integration
- minor fixes for interaction with OS X El Capitan
  system integrity protection (thanks Florian)


2.8.2 (2015-10-07)
==================

- fix #1085: proper handling of encoding errors when passing encoded byte
  strings to pytest.parametrize in Python 2.
  Thanks Themanwithoutaplan for the report and Bruno Oliveira for the PR.

- fix #1087: handling SystemError when passing empty byte strings to
  pytest.parametrize in Python 3.
  Thanks Paul Kehrer for the report and Bruno Oliveira for the PR.

- fix #995: fixed internal error when filtering tracebacks where one entry
  was generated by an exec() statement.
  Thanks Daniel Hahler, Ashley C Straw, Philippe Gauthier and Pavel Savchenko
  for contributing and Bruno Oliveira for the PR.

- fix #1100 and #1057: errors when using autouse fixtures and doctest modules.
  Thanks Sergey B Kirpichev and Vital Kudzelka for contributing and Bruno
  Oliveira for the PR.

2.8.1 (2015-09-29)
==================

- fix #1034: Add missing nodeid on pytest_logwarning call in
  addhook.  Thanks Simon Gomizelj for the PR.

- 'deprecated_call' is now only satisfied with a DeprecationWarning or
  PendingDeprecationWarning. Before 2.8.0, it accepted any warning, and 2.8.0
  made it accept only DeprecationWarning (but not PendingDeprecationWarning).
  Thanks Alex Gaynor for the issue and Eric Hunsberger for the PR.

- fix issue #1073: avoid calling __getattr__ on potential plugin objects.
  This fixes an incompatibility with pytest-django.  Thanks Andreas Pelme,
  Bruno Oliveira and Ronny Pfannschmidt for contributing and Holger Krekel
  for the fix.

- Fix issue #704: handle versionconflict during plugin loading more
  gracefully.  Thanks Bruno Oliveira for the PR.

- Fix issue #1064: ""--junitxml" regression when used with the
  "pytest-xdist" plugin, with test reports being assigned to the wrong tests.
  Thanks Daniel Grunwald for the report and Bruno Oliveira for the PR.

- (experimental) adapt more SEMVER style versioning and change meaning of
  master branch in git repo: "master" branch now keeps the bugfixes, changes
  aimed for micro releases.  "features" branch will only be released
  with minor or major pytest releases.

- Fix issue #766 by removing documentation references to distutils.
  Thanks Russel Winder.

- Fix issue #1030: now byte-strings are escaped to produce item node ids
  to make them always serializable.
  Thanks Andy Freeland for the report and Bruno Oliveira for the PR.

- Python 2: if unicode parametrized values are convertible to ascii, their
  ascii representation is used for the node id.

- Fix issue #411: Add __eq__ method to assertion comparison example.
  Thanks Ben Webb.
- Fix issue #653: deprecated_call can be used as context manager.

- fix issue 877: properly handle assertion explanations with non-ascii repr
  Thanks Mathieu Agopian for the report and Ronny Pfannschmidt for the PR.

- fix issue 1029: transform errors when writing cache values into pytest-warnings

2.8.0 (2015-09-18)
==================

- new ``--lf`` and ``-ff`` options to run only the last failing tests or
  "failing tests first" from the last run.  This functionality is provided
  through porting the formerly external pytest-cache plugin into pytest core.
  BACKWARD INCOMPAT: if you used pytest-cache's functionality to persist
  data between test runs be aware that we don't serialize sets anymore.
  Thanks Ronny Pfannschmidt for most of the merging work.

- "-r" option now accepts "a" to include all possible reports, similar
  to passing "fEsxXw" explicitly (isse960).
  Thanks Abhijeet Kasurde for the PR.

- avoid python3.5 deprecation warnings by introducing version
  specific inspection helpers, thanks Michael Droettboom.

- fix issue562: @nose.tools.istest now fully respected.

- fix issue934: when string comparison fails and a diff is too large to display
  without passing -vv, still show a few lines of the diff.
  Thanks Florian Bruhin for the report and Bruno Oliveira for the PR.

- fix issue736: Fix a bug where fixture params would be discarded when combined
  with parametrization markers.
  Thanks to Markus Unterwaditzer for the PR.

- fix issue710: introduce ALLOW_UNICODE doctest option: when enabled, the
  ``u`` prefix is stripped from unicode strings in expected doctest output. This
  allows doctests which use unicode to run in Python 2 and 3 unchanged.
  Thanks Jason R. Coombs for the report and Bruno Oliveira for the PR.

- parametrize now also generates meaningful test IDs for enum, regex and class
  objects (as opposed to class instances).
  Thanks to Florian Bruhin for the PR.

- Add 'warns' to assert that warnings are thrown (like 'raises').
  Thanks to Eric Hunsberger for the PR.

- Fix issue683: Do not apply an already applied mark.  Thanks ojake for the PR.

- Deal with capturing failures better so fewer exceptions get lost to
  /dev/null.  Thanks David Szotten for the PR.

- fix issue730: deprecate and warn about the --genscript option.
  Thanks Ronny Pfannschmidt for the report and Christian Pommranz for the PR.

- fix issue751: multiple parametrize with ids bug if it parametrizes class with
  two or more test methods. Thanks Sergey Chipiga for reporting and Jan
  Bednarik for PR.

- fix issue82: avoid loading conftest files from setup.cfg/pytest.ini/tox.ini
  files and upwards by default (--confcutdir can still be set to override this).
  Thanks Bruno Oliveira for the PR.

- fix issue768: docstrings found in python modules were not setting up session
  fixtures. Thanks Jason R. Coombs for reporting and Bruno Oliveira for the PR.

- added ``tmpdir_factory``, a session-scoped fixture that can be used to create
  directories under the base temporary directory. Previously this object was
  installed as a ``_tmpdirhandler`` attribute of the ``config`` object, but now it
  is part of the official API and using ``config._tmpdirhandler`` is
  deprecated.
  Thanks Bruno Oliveira for the PR.

- fix issue808: pytest's internal assertion rewrite hook now implements the
  optional PEP302 get_data API so tests can access data files next to them.
  Thanks xmo-odoo for request and example and Bruno Oliveira for
  the PR.

- rootdir and inifile are now displayed during usage errors to help
  users diagnose problems such as unexpected ini files which add
  unknown options being picked up by pytest. Thanks to Pavel Savchenko for
  bringing the problem to attention in #821 and Bruno Oliveira for the PR.

- Summary bar now is colored yellow for warning
  situations such as: all tests either were skipped or xpass/xfailed,
  or no tests were run at all (this is a partial fix for issue500).

- fix issue812: pytest now exits with status code 5 in situations where no
  tests were run at all, such as the directory given in the command line does
  not contain any tests or as result of a command line option filters
  all out all tests (-k for example).
  Thanks Eric Siegerman (issue812) and Bruno Oliveira for the PR.

- Summary bar now is colored yellow for warning
  situations such as: all tests either were skipped or xpass/xfailed,
  or no tests were run at all (related to issue500).
  Thanks Eric Siegerman.

- New ``testpaths`` ini option: list of directories to search for tests
  when executing pytest from the root directory. This can be used
  to speed up test collection when a project has well specified directories
  for tests, being usually more practical than configuring norecursedirs for
  all directories that do not contain tests.
  Thanks to Adrian for idea (#694) and Bruno Oliveira for the PR.

- fix issue713: JUnit XML reports for doctest failures.
  Thanks Punyashloka Biswal.

- fix issue970: internal pytest warnings now appear as "pytest-warnings" in
  the terminal instead of "warnings", so it is clear for users that those
  warnings are from pytest and not from the builtin "warnings" module.
  Thanks Bruno Oliveira.

- Include setup and teardown in junitxml test durations.
  Thanks Janne Vanhala.

- fix issue735: assertion failures on debug versions of Python 3.4+

- new option ``--import-mode`` to allow to change test module importing
  behaviour to append to sys.path instead of prepending.  This better allows
  to run test modules against installed versions of a package even if the
  package under test has the same import root.  In this example::

        testing/__init__.py
        testing/test_pkg_under_test.py
        pkg_under_test/

  the tests will run against the installed version
  of pkg_under_test when ``--import-mode=append`` is used whereas
  by default they would always pick up the local version.  Thanks Holger Krekel.

- pytester: add method ``TmpTestdir.delete_loaded_modules()``, and call it
  from ``inline_run()`` to allow temporary modules to be reloaded.
  Thanks Eduardo Schettino.

- internally refactor pluginmanager API and code so that there
  is a clear distinction between a pytest-agnostic rather simple
  pluginmanager and the PytestPluginManager which adds a lot of
  behaviour, among it handling of the local conftest files.
  In terms of documented methods this is a backward compatible
  change but it might still break 3rd party plugins which relied on
  details like especially the pluginmanager.add_shutdown() API.
  Thanks Holger Krekel.

- pluginmanagement: introduce ``pytest.hookimpl`` and
  ``pytest.hookspec`` decorators for setting impl/spec
  specific parameters.  This substitutes the previous
  now deprecated use of ``pytest.mark`` which is meant to
  contain markers for test functions only.

- write/refine docs for "writing plugins" which now have their
  own page and are separate from the "using/installing plugins`` page.

- fix issue732: properly unregister plugins from any hook calling
  sites allowing to have temporary plugins during test execution.

- deprecate and warn about ``__multicall__`` argument in hook
  implementations.  Use the ``hookwrapper`` mechanism instead already
  introduced with pytest-2.7.

- speed up pytest's own test suite considerably by using inprocess
  tests by default (testrun can be modified with --runpytest=subprocess
  to create subprocesses in many places instead).  The main
  APIs to run pytest in a test is "runpytest()" or "runpytest_subprocess"
  and "runpytest_inprocess" if you need a particular way of running
  the test.  In all cases you get back a RunResult but the inprocess
  one will also have a "reprec" attribute with the recorded events/reports.

- fix monkeypatch.setattr("x.y", raising=False) to actually not raise
  if "y" is not a pre-existing attribute. Thanks Florian Bruhin.

- fix issue741: make running output from testdir.run copy/pasteable
  Thanks Bruno Oliveira.

- add a new ``--noconftest`` argument which ignores all ``conftest.py`` files.

- add ``file`` and ``line`` attributes to JUnit-XML output.

- fix issue890: changed extension of all documentation files from ``txt`` to
  ``rst``. Thanks to Abhijeet for the PR.

- fix issue714: add ability to apply indirect=True parameter on particular argnames.
  Thanks Elizaveta239.

- fix issue890: changed extension of all documentation files from ``txt`` to
  ``rst``. Thanks to Abhijeet for the PR.

- fix issue957: "# doctest: SKIP" option will now register doctests as SKIPPED
  rather than PASSED.
  Thanks Thomas Grainger for the report and Bruno Oliveira for the PR.

- issue951: add new record_xml_property fixture, that supports logging
  additional information on xml output. Thanks David Diaz for the PR.

- issue949: paths after normal options (for example ``-s``, ``-v``, etc) are now
  properly used to discover ``rootdir`` and ``ini`` files.
  Thanks Peter Lauri for the report and Bruno Oliveira for the PR.

2.7.3 (2015-09-15)
==================

- Allow 'dev', 'rc', or other non-integer version strings in ``importorskip``.
  Thanks to Eric Hunsberger for the PR.

- fix issue856: consider --color parameter in all outputs (for example
  --fixtures). Thanks Barney Gale for the report and Bruno Oliveira for the PR.

- fix issue855: passing str objects as ``plugins`` argument to pytest.main
  is now interpreted as a module name to be imported and registered as a
  plugin, instead of silently having no effect.
  Thanks xmo-odoo for the report and Bruno Oliveira for the PR.

- fix issue744: fix for ast.Call changes in Python 3.5+.  Thanks
  Guido van Rossum, Matthias Bussonnier, Stefan Zimmermann and
  Thomas Kluyver.

- fix issue842: applying markers in classes no longer propagate this markers
  to superclasses which also have markers.
  Thanks xmo-odoo for the report and Bruno Oliveira for the PR.

- preserve warning functions after call to pytest.deprecated_call. Thanks
  Pieter Mulder for PR.

- fix issue854: autouse yield_fixtures defined as class members of
  unittest.TestCase subclasses now work as expected.
  Thannks xmo-odoo for the report and Bruno Oliveira for the PR.

- fix issue833: --fixtures now shows all fixtures of collected test files, instead of just the
  fixtures declared on the first one.
  Thanks Florian Bruhin for reporting and Bruno Oliveira for the PR.

- fix issue863: skipped tests now report the correct reason when a skip/xfail
  condition is met when using multiple markers.
  Thanks Raphael Pierzina for reporting and Bruno Oliveira for the PR.

- optimized tmpdir fixture initialization, which should make test sessions
  faster (specially when using pytest-xdist). The only visible effect
  is that now pytest uses a subdirectory in the $TEMP directory for all
  directories created by this fixture (defaults to $TEMP/pytest-$USER).
  Thanks Bruno Oliveira for the PR.

2.7.2 (2015-06-23)
==================

- fix issue767: pytest.raises value attribute does not contain the exception
  instance on Python 2.6. Thanks Eric Siegerman for providing the test
  case and Bruno Oliveira for PR.

- Automatically create directory for junitxml and results log.
  Thanks Aron Curzon.

- fix issue713: JUnit XML reports for doctest failures.
  Thanks Punyashloka Biswal.

- fix issue735: assertion failures on debug versions of Python 3.4+
  Thanks Benjamin Peterson.

- fix issue114: skipif marker reports to internal skipping plugin;
  Thanks Floris Bruynooghe for reporting and Bruno Oliveira for the PR.

- fix issue748: unittest.SkipTest reports to internal pytest unittest plugin.
  Thanks Thomas De Schampheleire for reporting and Bruno Oliveira for the PR.

- fix issue718: failed to create representation of sets containing unsortable
  elements in python 2. Thanks Edison Gustavo Muenz.

- fix issue756, fix issue752 (and similar issues): depend on py-1.4.29
  which has a refined algorithm for traceback generation.


2.7.1 (2015-05-19)
==================

- fix issue731: do not get confused by the braces which may be present
  and unbalanced in an object's repr while collapsing False
  explanations.  Thanks Carl Meyer for the report and test case.

- fix issue553: properly handling inspect.getsourcelines failures in
  FixtureLookupError which would lead to an internal error,
  obfuscating the original problem. Thanks talljosh for initial
  diagnose/patch and Bruno Oliveira for final patch.

- fix issue660: properly report scope-mismatch-access errors
  independently from ordering of fixture arguments.  Also
  avoid the pytest internal traceback which does not provide
  information to the user. Thanks Holger Krekel.

- streamlined and documented release process.  Also all versions
  (in setup.py and documentation generation) are now read
  from _pytest/__init__.py. Thanks Holger Krekel.

- fixed docs to remove the notion that yield-fixtures are experimental.
  They are here to stay :)  Thanks Bruno Oliveira.

- Support building wheels by using environment markers for the
  requirements.  Thanks Ionel Maries Cristian.

- fixed regression to 2.6.4 which surfaced e.g. in lost stdout capture printing
  when tests raised SystemExit. Thanks Holger Krekel.

- reintroduced _pytest fixture of the pytester plugin which is used
  at least by pytest-xdist.

2.7.0 (2015-03-26)
==================

- fix issue435: make reload() work when assert rewriting is active.
  Thanks Daniel Hahler.

- fix issue616: conftest.py files and their contained fixutres are now
  properly considered for visibility, independently from the exact
  current working directory and test arguments that are used.
  Many thanks to Eric Siegerman and his PR235 which contains
  systematic tests for conftest visibility and now passes.
  This change also introduces the concept of a ``rootdir`` which
  is printed as a new pytest header and documented in the pytest
  customize web page.

- change reporting of "diverted" tests, i.e. tests that are collected
  in one file but actually come from another (e.g. when tests in a test class
  come from a base class in a different file).  We now show the nodeid
  and indicate via a postfix the other file.

- add ability to set command line options by environment variable PYTEST_ADDOPTS.

- added documentation on the new pytest-dev teams on bitbucket and
  github.  See https://pytest.org/latest/contributing.html .
  Thanks to Anatoly for pushing and initial work on this.

- fix issue650: new option ``--docttest-ignore-import-errors`` which
  will turn import errors in doctests into skips.  Thanks Charles Cloud
  for the complete PR.

- fix issue655: work around different ways that cause python2/3
  to leak sys.exc_info into fixtures/tests causing failures in 3rd party code

- fix issue615: assertion rewriting did not correctly escape % signs
  when formatting boolean operations, which tripped over mixing
  booleans with modulo operators.  Thanks to Tom Viner for the report,
  triaging and fix.

- implement issue351: add ability to specify parametrize ids as a callable
  to generate custom test ids.  Thanks Brianna Laugher for the idea and
  implementation.

- introduce and document new hookwrapper mechanism useful for plugins
  which want to wrap the execution of certain hooks for their purposes.
  This supersedes the undocumented ``__multicall__`` protocol which
  pytest itself and some external plugins use.  Note that pytest-2.8
  is scheduled to drop supporting the old ``__multicall__``
  and only support the hookwrapper protocol.

- majorly speed up invocation of plugin hooks

- use hookwrapper mechanism in builtin pytest plugins.

- add a doctest ini option for doctest flags, thanks Holger Peters.

- add note to docs that if you want to mark a parameter and the
  parameter is a callable, you also need to pass in a reason to disambiguate
  it from the "decorator" case.  Thanks Tom Viner.

- "python_classes" and "python_functions" options now support glob-patterns
  for test discovery, as discussed in issue600. Thanks Ldiary Translations.

- allow to override parametrized fixtures with non-parametrized ones and vice versa (bubenkoff).

- fix issue463: raise specific error for 'parameterize' misspelling (pfctdayelise).

- On failure, the ``sys.last_value``, ``sys.last_type`` and
  ``sys.last_traceback`` are set, so that a user can inspect the error
  via postmortem debugging (almarklein).

2.6.4 (2014-10-24)
==================

- Improve assertion failure reporting on iterables, by using ndiff and
  pprint.

- removed outdated japanese docs from source tree.

- docs for "pytest_addhooks" hook.  Thanks Bruno Oliveira.

- updated plugin index docs.  Thanks Bruno Oliveira.

- fix issue557: with "-k" we only allow the old style "-" for negation
  at the beginning of strings and even that is deprecated.  Use "not" instead.
  This should allow to pick parametrized tests where "-" appeared in the parameter.

- fix issue604: Escape % character in the assertion message.

- fix issue620: add explanation in the --genscript target about what
  the binary blob means. Thanks Dinu Gherman.

- fix issue614: fixed pastebin support.


- fix issue620: add explanation in the --genscript target about what
  the binary blob means. Thanks Dinu Gherman.

- fix issue614: fixed pastebin support.

2.6.3 (2014-09-24)
==================

- fix issue575: xunit-xml was reporting collection errors as failures
  instead of errors, thanks Oleg Sinyavskiy.

- fix issue582: fix setuptools example, thanks Laszlo Papp and Ronny
  Pfannschmidt.

- Fix infinite recursion bug when pickling capture.EncodedFile, thanks
  Uwe Schmitt.

- fix issue589: fix bad interaction with numpy and others when showing
  exceptions.  Check for precise "maximum recursion depth exceed" exception
  instead of presuming any RuntimeError is that one (implemented in py
  dep).  Thanks Charles Cloud for analysing the issue.

- fix conftest related fixture visibility issue: when running with a
  CWD outside of a test package pytest would get fixture discovery wrong.
  Thanks to Wolfgang Schnerring for figuring out a reproducible example.

- Introduce pytest_enter_pdb hook (needed e.g. by pytest_timeout to cancel the
  timeout when interactively entering pdb).  Thanks Wolfgang Schnerring.

- check xfail/skip also with non-python function test items. Thanks
  Floris Bruynooghe.

2.6.2 (2014-09-05)
==================

- Added function pytest.freeze_includes(), which makes it easy to embed
  pytest into executables using tools like cx_freeze.
  See docs for examples and rationale. Thanks Bruno Oliveira.

- Improve assertion rewriting cache invalidation precision.

- fixed issue561: adapt autouse fixture example for python3.

- fixed issue453: assertion rewriting issue with __repr__ containing
  "\n{", "\n}" and "\n~".

- fix issue560: correctly display code if an "else:" or "finally:" is
  followed by statements on the same line.

- Fix example in monkeypatch documentation, thanks t-8ch.

- fix issue572: correct tmpdir doc example for python3.

- Do not mark as universal wheel because Python 2.6 is different from
  other builds due to the extra argparse dependency.  Fixes issue566.
  Thanks sontek.

- Implement issue549: user-provided assertion messages now no longer
  replace the py.test introspection message but are shown in addition
  to them.

2.6.1 (2014-08-07)
==================

- No longer show line numbers in the --verbose output, the output is now
  purely the nodeid.  The line number is still shown in failure reports.
  Thanks Floris Bruynooghe.

- fix issue437 where assertion rewriting could cause pytest-xdist slaves
  to collect different tests. Thanks Bruno Oliveira.

- fix issue555: add "errors" attribute to capture-streams to satisfy
  some distutils and possibly other code accessing sys.stdout.errors.

- fix issue547 capsys/capfd also work when output capturing ("-s") is disabled.

- address issue170: allow pytest.mark.xfail(...) to specify expected exceptions via
  an optional "raises=EXC" argument where EXC can be a single exception
  or a tuple of exception classes.  Thanks David Mohr for the complete
  PR.

- fix integration of pytest with unittest.mock.patch decorator when
  it uses the "new" argument.  Thanks Nicolas Delaby for test and PR.

- fix issue with detecting conftest files if the arguments contain
  "::" node id specifications (copy pasted from "-v" output)

- fix issue544 by only removing "@NUM" at the end of "::" separated parts
  and if the part has a ".py" extension

- don't use py.std import helper, rather import things directly.
  Thanks Bruno Oliveira.

2.6
===

- Cache exceptions from fixtures according to their scope (issue 467).

- fix issue537: Avoid importing old assertion reinterpretation code by default.

- fix issue364: shorten and enhance tracebacks representation by default.
  The new "--tb=auto" option (default) will only display long tracebacks
  for the first and last entry.  You can get the old behaviour of printing
  all entries as long entries with "--tb=long".  Also short entries by
  default are now printed very similarly to "--tb=native" ones.

- fix issue514: teach assertion reinterpretation about private class attributes

- change -v output to include full node IDs of tests.  Users can copy
  a node ID from a test run, including line number, and use it as a
  positional argument in order to run only a single test.

- fix issue 475: fail early and comprehensible if calling
  pytest.raises with wrong exception type.

- fix issue516: tell in getting-started about current dependencies.

- cleanup setup.py a bit and specify supported versions. Thanks Jurko
  Gospodnetic for the PR.

- change XPASS colour to yellow rather then red when tests are run
  with -v.

- fix issue473: work around mock putting an unbound method into a class
  dict when double-patching.

- fix issue498: if a fixture finalizer fails, make sure that
  the fixture is still invalidated.

- fix issue453: the result of the pytest_assertrepr_compare hook now gets
  it's newlines escaped so that format_exception does not blow up.

- internal new warning system: pytest will now produce warnings when
  it detects oddities in your test collection or execution.
  Warnings are ultimately sent to a new pytest_logwarning hook which is
  currently only implemented by the terminal plugin which displays
  warnings in the summary line and shows more details when -rw (report on
  warnings) is specified.

- change skips into warnings for test classes with an __init__ and
  callables in test modules which look like a test but are not functions.

- fix issue436: improved finding of initial conftest files from command
  line arguments by using the result of parse_known_args rather than
  the previous flaky heuristics.  Thanks Marc Abramowitz for tests
  and initial fixing approaches in this area.

- fix issue #479: properly handle nose/unittest(2) SkipTest exceptions
  during collection/loading of test modules.  Thanks to Marc Schlaich
  for the complete PR.

- fix issue490: include pytest_load_initial_conftests in documentation
  and improve docstring.

- fix issue472: clarify that ``pytest.config.getvalue()`` cannot work
  if it's triggered ahead of command line parsing.

- merge PR123: improved integration with mock.patch decorator on tests.

- fix issue412: messing with stdout/stderr FD-level streams is now
  captured without crashes.

- fix issue483: trial/py33 works now properly.  Thanks Daniel Grana for PR.

- improve example for pytest integration with "python setup.py test"
  which now has a generic "-a" or "--pytest-args" option where you
  can pass additional options as a quoted string.  Thanks Trevor Bekolay.

- simplified internal capturing mechanism and made it more robust
  against tests or setups changing FD1/FD2, also better integrated
  now with pytest.pdb() in single tests.

- improvements to pytest's own test-suite leakage detection, courtesy of PRs
  from Marc Abramowitz

- fix issue492: avoid leak in test_writeorg.  Thanks Marc Abramowitz.

- fix issue493: don't run tests in doc directory with ``python setup.py test``
  (use tox -e doctesting for that)

- fix issue486: better reporting and handling of early conftest loading failures

- some cleanup and simplification of internal conftest handling.

- work a bit harder to break reference cycles when catching exceptions.
  Thanks Jurko Gospodnetic.

- fix issue443: fix skip examples to use proper comparison.  Thanks Alex
  Groenholm.

- support nose-style ``__test__`` attribute on modules, classes and
  functions, including unittest-style Classes.  If set to False, the
  test will not be collected.

- fix issue512: show "<notset>" for arguments which might not be set
  in monkeypatch plugin.  Improves output in documentation.


2.5.2 (2014-01-29)
==================

- fix issue409 -- better interoperate with cx_freeze by not
  trying to import from collections.abc which causes problems
  for py27/cx_freeze.  Thanks Wolfgang L. for reporting and tracking it down.

- fixed docs and code to use "pytest" instead of "py.test" almost everywhere.
  Thanks Jurko Gospodnetic for the complete PR.

- fix issue425: mention at end of "py.test -h" that --markers
  and --fixtures work according to specified test path (or current dir)

- fix issue413: exceptions with unicode attributes are now printed
  correctly also on python2 and with pytest-xdist runs. (the fix
  requires py-1.4.20)

- copy, cleanup and integrate py.io capture
  from pylib 1.4.20.dev2 (rev 13d9af95547e)

- address issue416: clarify docs as to conftest.py loading semantics

- fix issue429: comparing byte strings with non-ascii chars in assert
  expressions now work better.  Thanks Floris Bruynooghe.

- make capfd/capsys.capture private, its unused and shouldn't be exposed


2.5.1 (2013-12-17)
==================

- merge new documentation styling PR from Tobias Bieniek.

- fix issue403: allow parametrize of multiple same-name functions within
  a collection node.  Thanks Andreas Kloeckner and Alex Gaynor for reporting
  and analysis.

- Allow parameterized fixtures to specify the ID of the parameters by
  adding an ids argument to pytest.fixture() and pytest.yield_fixture().
  Thanks Floris Bruynooghe.

- fix issue404 by always using the binary xml escape in the junitxml
  plugin.  Thanks Ronny Pfannschmidt.

- fix issue407: fix addoption docstring to point to argparse instead of
  optparse. Thanks Daniel D. Wright.



2.5.0 (2013-12-12)
==================

- dropped python2.5 from automated release testing of pytest itself
  which means it's probably going to break soon (but still works
  with this release we believe).

- simplified and fixed implementation for calling finalizers when
  parametrized fixtures or function arguments are involved.  finalization
  is now performed lazily at setup time instead of in the "teardown phase".
  While this might sound odd at first, it helps to ensure that we are
  correctly handling setup/teardown even in complex code.  User-level code
  should not be affected unless it's implementing the pytest_runtest_teardown
  hook and expecting certain fixture instances are torn down within (very
  unlikely and would have been unreliable anyway).

- PR90: add --color=yes|no|auto option to force terminal coloring
  mode ("auto" is default).  Thanks Marc Abramowitz.

- fix issue319 - correctly show unicode in assertion errors.  Many
  thanks to Floris Bruynooghe for the complete PR.  Also means
  we depend on py>=1.4.19 now.

- fix issue396 - correctly sort and finalize class-scoped parametrized
  tests independently from number of methods on the class.

- refix issue323 in a better way -- parametrization should now never
  cause Runtime Recursion errors because the underlying algorithm
  for re-ordering tests per-scope/per-fixture is not recursive
  anymore (it was tail-call recursive before which could lead
  to problems for more than >966 non-function scoped parameters).

- fix issue290 - there is preliminary support now for parametrizing
  with repeated same values (sometimes useful to test if calling
  a second time works as with the first time).

- close issue240 - document precisely how pytest module importing
  works, discuss the two common test directory layouts, and how it
  interacts with PEP420-namespace packages.

- fix issue246 fix finalizer order to be LIFO on independent fixtures
  depending on a parametrized higher-than-function scoped fixture.
  (was quite some effort so please bear with the complexity of this sentence :)
  Thanks Ralph Schmitt for the precise failure example.

- fix issue244 by implementing special index for parameters to only use
  indices for paramentrized test ids

- fix issue287 by running all finalizers but saving the exception
  from the first failing finalizer and re-raising it so teardown will
  still have failed.  We reraise the first failing exception because
  it might be the cause for other finalizers to fail.

- fix ordering when mock.patch or other standard decorator-wrappings
  are used with test methods.  This fixues issue346 and should
  help with random "xdist" collection failures.  Thanks to
  Ronny Pfannschmidt and Donald Stufft for helping to isolate it.

- fix issue357 - special case "-k" expressions to allow for
  filtering with simple strings that are not valid python expressions.
  Examples: "-k 1.3" matches all tests parametrized with 1.3.
  "-k None" filters all tests that have "None" in their name
  and conversely "-k 'not None'".
  Previously these examples would raise syntax errors.

- fix issue384 by removing the trial support code
  since the unittest compat enhancements allow
  trial to handle it on its own

- don't hide an ImportError when importing a plugin produces one.
  fixes issue375.

- fix issue275 - allow usefixtures and autouse fixtures
  for running doctest text files.

- fix issue380 by making --resultlog only rely on longrepr instead
  of the "reprcrash" attribute which only exists sometimes.

- address issue122: allow @pytest.fixture(params=iterator) by exploding
  into a list early on.

- fix pexpect-3.0 compatibility for pytest's own tests.
  (fixes issue386)

- allow nested parametrize-value markers, thanks James Lan for the PR.

- fix unicode handling with new monkeypatch.setattr(import_path, value)
  API.  Thanks Rob Dennis.  Fixes issue371.

- fix unicode handling with junitxml, fixes issue368.

- In assertion rewriting mode on Python 2, fix the detection of coding
  cookies. See issue #330.

- make "--runxfail" turn imperative pytest.xfail calls into no ops
  (it already did neutralize pytest.mark.xfail markers)

- refine pytest / pkg_resources interactions: The AssertionRewritingHook
  PEP302 compliant loader now registers itself with setuptools/pkg_resources
  properly so that the pkg_resources.resource_stream method works properly.
  Fixes issue366.  Thanks for the investigations and full PR to Jason R. Coombs.

- pytestconfig fixture is now session-scoped as it is the same object during the
  whole test run.  Fixes issue370.

- avoid one surprising case of marker malfunction/confusion::

      @pytest.mark.some(lambda arg: ...)
      def test_function():

  would not work correctly because pytest assumes @pytest.mark.some
  gets a function to be decorated already.  We now at least detect if this
  arg is a lambda and thus the example will work.  Thanks Alex Gaynor
  for bringing it up.

- xfail a test on pypy that checks wrong encoding/ascii (pypy does
  not error out). fixes issue385.

- internally make varnames() deal with classes's __init__,
  although it's not needed by pytest itself atm.  Also
  fix caching.  Fixes issue376.

- fix issue221 - handle importing of namespace-package with no
  __init__.py properly.

- refactor internal FixtureRequest handling to avoid monkeypatching.
  One of the positive user-facing effects is that the "request" object
  can now be used in closures.

- fixed version comparison in pytest.importskip(modname, minverstring)

- fix issue377 by clarifying in the nose-compat docs that pytest
  does not duplicate the unittest-API into the "plain" namespace.

- fix verbose reporting for @mock'd test functions

2.4.2 (2013-10-04)
==================

- on Windows require colorama and a newer py lib so that py.io.TerminalWriter()
  now uses colorama instead of its own ctypes hacks. (fixes issue365)
  thanks Paul Moore for bringing it up.

- fix "-k" matching of tests where "repr" and "attr" and other names would
  cause wrong matches because of an internal implementation quirk
  (don't ask) which is now properly implemented. fixes issue345.

- avoid tmpdir fixture to create too long filenames especially
  when parametrization is used (issue354)

- fix pytest-pep8 and pytest-flakes / pytest interactions
  (collection names in mark plugin was assuming an item always
  has a function which is not true for those plugins etc.)
  Thanks Andi Zeidler.

- introduce node.get_marker/node.add_marker API for plugins
  like pytest-pep8 and pytest-flakes to avoid the messy
  details of the node.keywords  pseudo-dicts.  Adapted
  docs.

- remove attempt to "dup" stdout at startup as it's icky.
  the normal capturing should catch enough possibilities
  of tests messing up standard FDs.

- add pluginmanager.do_configure(config) as a link to
  config.do_configure() for plugin-compatibility

2.4.1 (2013-10-02)
==================

- When using parser.addoption() unicode arguments to the
  "type" keyword should also be converted to the respective types.
  thanks Floris Bruynooghe, @dnozay. (fixes issue360 and issue362)

- fix dotted filename completion when using argcomplete
  thanks Anthon van der Neuth. (fixes issue361)

- fix regression when a 1-tuple ("arg",) is used for specifying
  parametrization (the values of the parametrization were passed
  nested in a tuple).  Thanks Donald Stufft.

- merge doc typo fixes, thanks Andy Dirnberger

2.4
===

known incompatibilities:

- if calling --genscript from python2.7 or above, you only get a
  standalone script which works on python2.7 or above.  Use Python2.6
  to also get a python2.5 compatible version.

- all xunit-style teardown methods (nose-style, pytest-style,
  unittest-style) will not be called if the corresponding setup method failed,
  see issue322 below.

- the pytest_plugin_unregister hook wasn't ever properly called
  and there is no known implementation of the hook - so it got removed.

- pytest.fixture-decorated functions cannot be generators (i.e. use
  yield) anymore.  This change might be reversed in 2.4.1 if it causes
  unforeseen real-life issues.  However, you can always write and return
  an inner function/generator and change the fixture consumer to iterate
  over the returned generator.  This change was done in lieu of the new
  ``pytest.yield_fixture`` decorator, see below.

new features:

- experimentally introduce a new ``pytest.yield_fixture`` decorator
  which accepts exactly the same parameters as pytest.fixture but
  mandates a ``yield`` statement instead of a ``return statement`` from
  fixture functions.  This allows direct integration with "with-style"
  context managers in fixture functions and generally avoids registering
  of finalization callbacks in favour of treating the "after-yield" as
  teardown code.  Thanks Andreas Pelme, Vladimir Keleshev, Floris
  Bruynooghe, Ronny Pfannschmidt and many others for discussions.

- allow boolean expression directly with skipif/xfail
  if a "reason" is also specified.  Rework skipping documentation
  to recommend "condition as booleans" because it prevents surprises
  when importing markers between modules.  Specifying conditions
  as strings will remain fully supported.

- reporting: color the last line red or green depending if
  failures/errors occurred or everything passed.  thanks Christian
  Theunert.

- make "import pdb ; pdb.set_trace()" work natively wrt capturing (no
  "-s" needed anymore), making ``pytest.set_trace()`` a mere shortcut.

- fix issue181: --pdb now also works on collect errors (and
  on internal errors) .  This was implemented by a slight internal
  refactoring and the introduction of a new hook
  ``pytest_exception_interact`` hook (see next item).

- fix issue341: introduce new experimental hook for IDEs/terminals to
  intercept debugging: ``pytest_exception_interact(node, call, report)``.

- new monkeypatch.setattr() variant to provide a shorter
  invocation for patching out classes/functions from modules:

     monkeypatch.setattr("requests.get", myfunc)

  will replace the "get" function of the "requests" module with ``myfunc``.

- fix issue322: tearDownClass is not run if setUpClass failed. Thanks
  Mathieu Agopian for the initial fix.  Also make all of pytest/nose
  finalizer mimic the same generic behaviour: if a setupX exists and
  fails, don't run teardownX.  This internally introduces a new method
  "node.addfinalizer()" helper which can only be called during the setup
  phase of a node.

- simplify pytest.mark.parametrize() signature: allow to pass a
  CSV-separated string to specify argnames.  For example:
  ``pytest.mark.parametrize("input,expected",  [(1,2), (2,3)])``
  works as well as the previous:
  ``pytest.mark.parametrize(("input", "expected"), ...)``.

- add support for setUpModule/tearDownModule detection, thanks Brian Okken.

- integrate tab-completion on options through use of "argcomplete".
  Thanks Anthon van der Neut for the PR.

- change option names to be hyphen-separated long options but keep the
  old spelling backward compatible.  py.test -h will only show the
  hyphenated version, for example "--collect-only" but "--collectonly"
  will remain valid as well (for backward-compat reasons).  Many thanks to
  Anthon van der Neut for the implementation and to Hynek Schlawack for
  pushing us.

- fix issue 308 - allow to mark/xfail/skip individual parameter sets
  when parametrizing.  Thanks Brianna Laugher.

- call new experimental pytest_load_initial_conftests hook to allow
  3rd party plugins to do something before a conftest is loaded.

Bug fixes:

- fix issue358 - capturing options are now parsed more properly
  by using a new parser.parse_known_args method.

- pytest now uses argparse instead of optparse (thanks Anthon) which
  means that "argparse" is added as a dependency if installing into python2.6
  environments or below.

- fix issue333: fix a case of bad unittest/pytest hook interaction.

- PR27: correctly handle nose.SkipTest during collection.  Thanks
  Antonio Cuni, Ronny Pfannschmidt.

- fix issue355: junitxml puts name="pytest" attribute to testsuite tag.

- fix issue336: autouse fixture in plugins should work again.

- fix issue279: improve object comparisons on assertion failure
  for standard datatypes and recognise collections.abc.  Thanks to
  Brianna Laugher and Mathieu Agopian.

- fix issue317: assertion rewriter support for the is_package method

- fix issue335: document py.code.ExceptionInfo() object returned
  from pytest.raises(), thanks Mathieu Agopian.

- remove implicit distribute_setup support from setup.py.

- fix issue305: ignore any problems when writing pyc files.

- SO-17664702: call fixture finalizers even if the fixture function
  partially failed (finalizers would not always be called before)

- fix issue320 - fix class scope for fixtures when mixed with
  module-level functions.  Thanks Anatloy Bubenkoff.

- you can specify "-q" or "-qq" to get different levels of "quieter"
  reporting (thanks Katarzyna Jachim)

- fix issue300 - Fix order of conftest loading when starting py.test
  in a subdirectory.

- fix issue323 - sorting of many module-scoped arg parametrizations

- make sessionfinish hooks execute with the same cwd-context as at
  session start (helps fix plugin behaviour which write output files
  with relative path such as pytest-cov)

- fix issue316 - properly reference collection hooks in docs

- fix issue 306 - cleanup of -k/-m options to only match markers/test
  names/keywords respectively.  Thanks Wouter van Ackooy.

- improved doctest counting for doctests in python modules --
  files without any doctest items will not show up anymore
  and doctest examples are counted as separate test items.
  thanks Danilo Bellini.

- fix issue245 by depending on the released py-1.4.14
  which fixes py.io.dupfile to work with files with no
  mode. Thanks Jason R. Coombs.

- fix junitxml generation when test output contains control characters,
  addressing issue267, thanks Jaap Broekhuizen

- fix issue338: honor --tb style for setup/teardown errors as well.  Thanks Maho.

- fix issue307 - use yaml.safe_load in example, thanks Mark Eichin.

- better parametrize error messages, thanks Brianna Laugher

- pytest_terminal_summary(terminalreporter) hooks can now use
  ".section(title)" and ".line(msg)" methods to print extra
  information at the end of a test run.

2.3.5 (2013-04-30)
==================

- fix issue169: respect --tb=style with setup/teardown errors as well.

- never consider a fixture function for test function collection

- allow re-running of test items / helps to fix pytest-reruntests plugin
  and also help to keep less fixture/resource references alive

- put captured stdout/stderr into junitxml output even for passing tests
  (thanks Adam Goucher)

- Issue 265 - integrate nose setup/teardown with setupstate
  so it doesn't try to teardown if it did not setup

- issue 271 - don't write junitxml on slave nodes

- Issue 274 - don't try to show full doctest example
  when doctest does not know the example location

- issue 280 - disable assertion rewriting on buggy CPython 2.6.0

- inject "getfixture()" helper to retrieve fixtures from doctests,
  thanks Andreas Zeidler

- issue 259 - when assertion rewriting, be consistent with the default
  source encoding of ASCII on Python 2

- issue 251 - report a skip instead of ignoring classes with init

- issue250 unicode/str mixes in parametrization names and values now works

- issue257, assertion-triggered compilation of source ending in a
  comment line doesn't blow up in python2.5 (fixed through py>=1.4.13.dev6)

- fix --genscript option to generate standalone scripts that also
  work with python3.3 (importer ordering)

- issue171 - in assertion rewriting, show the repr of some
  global variables

- fix option help for "-k"

- move long description of distribution into README.rst

- improve docstring for metafunc.parametrize()

- fix bug where using capsys with pytest.set_trace() in a test
  function would break when looking at capsys.readouterr()

- allow to specify prefixes starting with "_" when
  customizing python_functions test discovery. (thanks Graham Horler)

- improve PYTEST_DEBUG tracing output by putting
  extra data on a new lines with additional indent

- ensure OutcomeExceptions like skip/fail have initialized exception attributes

- issue 260 - don't use nose special setup on plain unittest cases

- fix issue134 - print the collect errors that prevent running specified test items

- fix issue266 - accept unicode in MarkEvaluator expressions

2.3.4 (2012-11-20)
==================

- yielded test functions will now have autouse-fixtures active but
  cannot accept fixtures as funcargs - it's anyway recommended to
  rather use the post-2.0 parametrize features instead of yield, see:
  http://pytest.org/latest/example/parametrize.html
- fix autouse-issue where autouse-fixtures would not be discovered
  if defined in an a/conftest.py file and tests in a/tests/test_some.py
- fix issue226 - LIFO ordering for fixture teardowns
- fix issue224 - invocations with >256 char arguments now work
- fix issue91 - add/discuss package/directory level setups in example
- allow to dynamically define markers via
  item.keywords[...]=assignment integrating with "-m" option
- make "-k" accept an expressions the same as with "-m" so that one
  can write: -k "name1 or name2" etc.  This is a slight incompatibility
  if you used special syntax like "TestClass.test_method" which you now
  need to write as -k "TestClass and test_method" to match a certain
  method in a certain test class.

2.3.3 (2012-11-06)
==================

- fix issue214 - parse modules that contain special objects like e. g.
  flask's request object which blows up on getattr access if no request
  is active. thanks Thomas Waldmann.

- fix issue213 - allow to parametrize with values like numpy arrays that
  do not support an __eq__ operator

- fix issue215 - split test_python.org into multiple files

- fix issue148 - @unittest.skip on classes is now recognized and avoids
  calling setUpClass/tearDownClass, thanks Pavel Repin

- fix issue209 - reintroduce python2.4 support by depending on newer
  pylib which re-introduced statement-finding for pre-AST interpreters

- nose support: only call setup if it's a callable, thanks Andrew
  Taumoefolau

- fix issue219 - add py2.4-3.3 classifiers to TROVE list

- in tracebacks *,** arg values are now shown next to normal arguments
  (thanks Manuel Jacob)

- fix issue217 - support mock.patch with pytest's fixtures - note that
  you need either mock-1.0.1 or the python3.3 builtin unittest.mock.

- fix issue127 - improve documentation for pytest_addoption() and
  add a ``config.getoption(name)`` helper function for consistency.

2.3.2 (2012-10-25)
==================

- fix issue208 and fix issue29 use new py version to avoid long pauses
  when printing tracebacks in long modules

- fix issue205 - conftests in subdirs customizing
  pytest_pycollect_makemodule and pytest_pycollect_makeitem
  now work properly

- fix teardown-ordering for parametrized setups

- fix issue127 - better documentation for pytest_addoption
  and related objects.

- fix unittest behaviour: TestCase.runtest only called if there are
  test methods defined

- improve trial support: don't collect its empty
  unittest.TestCase.runTest() method

- "python setup.py test" now works with pytest itself

- fix/improve internal/packaging related bits:

  - exception message check of test_nose.py now passes on python33 as well

  - issue206 - fix test_assertrewrite.py to work when a global
    PYTHONDONTWRITEBYTECODE=1 is present

  - add tox.ini to pytest distribution so that ignore-dirs and others config
    bits are properly distributed for maintainers who run pytest-own tests

2.3.1 (2012-10-20)
==================

- fix issue202 - fix regression: using "self" from fixture functions now
  works as expected (it's the same "self" instance that a test method
  which uses the fixture sees)

- skip pexpect using tests (test_pdb.py mostly) on freebsd* systems
  due to pexpect not supporting it properly (hanging)

- link to web pages from --markers output which provides help for
  pytest.mark.* usage.

2.3.0 (2012-10-19)
==================

- fix issue202 - better automatic names for parametrized test functions
- fix issue139 - introduce @pytest.fixture which allows direct scoping
  and parametrization of funcarg factories.
- fix issue198 - conftest fixtures were not found on windows32 in some
  circumstances with nested directory structures due to path manipulation issues
- fix issue193 skip test functions with were parametrized with empty
  parameter sets
- fix python3.3 compat, mostly reporting bits that previously depended
  on dict ordering
- introduce re-ordering of tests by resource and parametrization setup
  which takes precedence to the usual file-ordering
- fix issue185 monkeypatching time.time does not cause pytest to fail
- fix issue172 duplicate call of pytest.fixture decoratored setup_module
  functions
- fix junitxml=path construction so that if tests change the
  current working directory and the path is a relative path
  it is constructed correctly from the original current working dir.
- fix "python setup.py test" example to cause a proper "errno" return
- fix issue165 - fix broken doc links and mention stackoverflow for FAQ
- catch unicode-issues when writing failure representations
  to terminal to prevent the whole session from crashing
- fix xfail/skip confusion: a skip-mark or an imperative pytest.skip
  will now take precedence before xfail-markers because we
  can't determine xfail/xpass status in case of a skip. see also:
  http://stackoverflow.com/questions/11105828/in-py-test-when-i-explicitly-skip-a-test-that-is-marked-as-xfail-how-can-i-get

- always report installed 3rd party plugins in the header of a test run

- fix issue160: a failing setup of an xfail-marked tests should
  be reported as xfail (not xpass)

- fix issue128: show captured output when capsys/capfd are used

- fix issue179: properly show the dependency chain of factories

- pluginmanager.register(...) now raises ValueError if the
  plugin has been already registered or the name is taken

- fix issue159: improve http://pytest.org/latest/faq.html
  especially with respect to the "magic" history, also mention
  pytest-django, trial and unittest integration.

- make request.keywords and node.keywords writable.  All descendant
  collection nodes will see keyword values.  Keywords are dictionaries
  containing markers and other info.

- fix issue 178: xml binary escapes are now wrapped in py.xml.raw

- fix issue 176: correctly catch the builtin AssertionError
  even when we replaced AssertionError with a subclass on the
  python level

- factory discovery no longer fails with magic global callables
  that provide no sane __code__ object (mock.call for example)

- fix issue 182: testdir.inprocess_run now considers passed plugins

- fix issue 188: ensure sys.exc_info is clear on python2
                 before calling into a test

- fix issue 191: add unittest TestCase runTest method support
- fix issue 156: monkeypatch correctly handles class level descriptors

- reporting refinements:

  - pytest_report_header now receives a "startdir" so that
    you can use startdir.bestrelpath(yourpath) to show
    nice relative path

  - allow plugins to implement both pytest_report_header and
    pytest_sessionstart (sessionstart is invoked first).

  - don't show deselected reason line if there is none

  - py.test -vv will show all of assert comparisons instead of truncating

2.2.4 (2012-05-22)
==================

- fix error message for rewritten assertions involving the % operator
- fix issue 126: correctly match all invalid xml characters for junitxml
  binary escape
- fix issue with unittest: now @unittest.expectedFailure markers should
  be processed correctly (you can also use @pytest.mark markers)
- document integration with the extended distribute/setuptools test commands
- fix issue 140: properly get the real functions
  of bound classmethods for setup/teardown_class
- fix issue #141: switch from the deceased paste.pocoo.org to bpaste.net
- fix issue #143: call unconfigure/sessionfinish always when
  configure/sessionstart where called
- fix issue #144: better mangle test ids to junitxml classnames
- upgrade distribute_setup.py to 0.6.27

2.2.3 (2012-02-05)
==================

- fix uploaded package to only include necessary files

2.2.2 (2012-02-05)
==================

- fix issue101: wrong args to unittest.TestCase test function now
  produce better output
- fix issue102: report more useful errors and hints for when a
  test directory was renamed and some pyc/__pycache__ remain
- fix issue106: allow parametrize to be applied multiple times
  e.g. from module, class and at function level.
- fix issue107: actually perform session scope finalization
- don't check in parametrize if indirect parameters are funcarg names
- add chdir method to monkeypatch funcarg
- fix crash resulting from calling monkeypatch undo a second time
- fix issue115: make --collectonly robust against early failure
  (missing files/directories)
- "-qq --collectonly" now shows only files and the number of tests in them
- "-q --collectonly" now shows test ids
- allow adding of attributes to test reports such that it also works
  with distributed testing (no upgrade of pytest-xdist needed)

2.2.1 (2011-12-16)
==================

- fix issue99 (in pytest and py) internallerrors with resultlog now
  produce better output - fixed by normalizing pytest_internalerror
  input arguments.
- fix issue97 / traceback issues (in pytest and py) improve traceback output
  in conjunction with jinja2 and cython which hack tracebacks
- fix issue93 (in pytest and pytest-xdist) avoid "delayed teardowns":
  the final test in a test node will now run its teardown directly
  instead of waiting for the end of the session. Thanks Dave Hunt for
  the good reporting and feedback.  The pytest_runtest_protocol as well
  as the pytest_runtest_teardown hooks now have "nextitem" available
  which will be None indicating the end of the test run.
- fix collection crash due to unknown-source collected items, thanks
  to Ralf Schmitt (fixed by depending on a more recent pylib)

2.2.0 (2011-11-18)
==================

- fix issue90: introduce eager tearing down of test items so that
  teardown function are called earlier.
- add an all-powerful metafunc.parametrize function which allows to
  parametrize test function arguments in multiple steps and therefore
  from independent plugins and places.
- add a @pytest.mark.parametrize helper which allows to easily
  call a test function with different argument values
- Add examples to the "parametrize" example page, including a quick port
  of Test scenarios and the new parametrize function and decorator.
- introduce registration for "pytest.mark.*" helpers via ini-files
  or through plugin hooks.  Also introduce a "--strict" option which
  will treat unregistered markers as errors
  allowing to avoid typos and maintain a well described set of markers
  for your test suite.  See exaples at http://pytest.org/latest/mark.html
  and its links.
- issue50: introduce "-m marker" option to select tests based on markers
  (this is a stricter and more predictable version of '-k' in that "-m"
  only matches complete markers and has more obvious rules for and/or
  semantics.
- new feature to help optimizing the speed of your tests:
  --durations=N option for displaying N slowest test calls
  and setup/teardown methods.
- fix issue87: --pastebin now works with python3
- fix issue89: --pdb with unexpected exceptions in doctest work more sensibly
- fix and cleanup pytest's own test suite to not leak FDs
- fix issue83: link to generated funcarg list
- fix issue74: pyarg module names are now checked against imp.find_module false positives
- fix compatibility with twisted/trial-11.1.0 use cases
- simplify Node.listchain
- simplify junitxml output code by relying on py.xml
- add support for skip properties on unittest classes and functions

2.1.3 (2011-10-18)
==================

- fix issue79: assertion rewriting failed on some comparisons in boolops
- correctly handle zero length arguments (a la pytest '')
- fix issue67 / junitxml now contains correct test durations, thanks ronny
- fix issue75 / skipping test failure on jython
- fix issue77 / Allow assertrepr_compare hook to apply to a subset of tests

2.1.2 (2011-09-24)
==================

- fix assertion rewriting on files with windows newlines on some Python versions
- refine test discovery by package/module name (--pyargs), thanks Florian Mayer
- fix issue69 / assertion rewriting fixed on some boolean operations
- fix issue68 / packages now work with assertion rewriting
- fix issue66: use different assertion rewriting caches when the -O option is passed
- don't try assertion rewriting on Jython, use reinterp

2.1.1
=====

- fix issue64 / pytest.set_trace now works within pytest_generate_tests hooks
- fix issue60 / fix error conditions involving the creation of __pycache__
- fix issue63 / assertion rewriting on inserts involving strings containing '%'
- fix assertion rewriting on calls with a ** arg
- don't cache rewritten modules if bytecode generation is disabled
- fix assertion rewriting in read-only directories
- fix issue59: provide system-out/err tags for junitxml output
- fix issue61: assertion rewriting on boolean operations with 3 or more operands
- you can now build a man page with "cd doc ; make man"

2.1.0 (2011-07-09)
==================

- fix issue53 call nosestyle setup functions with correct ordering
- fix issue58 and issue59: new assertion code fixes
- merge Benjamin's assertionrewrite branch: now assertions
  for test modules on python 2.6 and above are done by rewriting
  the AST and saving the pyc file before the test module is imported.
  see doc/assert.txt for more info.
- fix issue43: improve doctests with better traceback reporting on
  unexpected exceptions
- fix issue47: timing output in junitxml for test cases is now correct
- fix issue48: typo in MarkInfo repr leading to exception
- fix issue49: avoid confusing error when initizaliation partially fails
- fix issue44: env/username expansion for junitxml file path
- show releaselevel information in test runs for pypy
- reworked doc pages for better navigation and PDF generation
- report KeyboardInterrupt even if interrupted during session startup
- fix issue 35 - provide PDF doc version and download link from index page

2.0.3 (2011-05-11)
==================

- fix issue38: nicer tracebacks on calls to hooks, particularly early
  configure/sessionstart ones

- fix missing skip reason/meta information in junitxml files, reported
  via http://lists.idyll.org/pipermail/testing-in-python/2011-March/003928.html

- fix issue34: avoid collection failure with "test" prefixed classes
  deriving from object.

- don't require zlib (and other libs) for genscript plugin without
  --genscript actually being used.

- speed up skips (by not doing a full traceback representation
  internally)

- fix issue37: avoid invalid characters in junitxml's output

2.0.2 (2011-03-09)
==================

- tackle issue32 - speed up test runs of very quick test functions
  by reducing the relative overhead

- fix issue30 - extended xfail/skipif handling and improved reporting.
  If you have a syntax error in your skip/xfail
  expressions you now get nice error reports.

  Also you can now access module globals from xfail/skipif
  expressions so that this for example works now::

    import pytest
    import mymodule
    @pytest.mark.skipif("mymodule.__version__[0] == "1")
    def test_function():
        pass

  This will not run the test function if the module's version string
  does not start with a "1".  Note that specifying a string instead
  of a boolean expressions allows py.test to report meaningful information
  when summarizing a test run as to what conditions lead to skipping
  (or xfail-ing) tests.

- fix issue28 - setup_method and pytest_generate_tests work together
  The setup_method fixture method now gets called also for
  test function invocations generated from the pytest_generate_tests
  hook.

- fix issue27 - collectonly and keyword-selection (-k) now work together
  Also, if you do "py.test --collectonly -q" you now get a flat list
  of test ids that you can use to paste to the py.test commandline
  in order to execute a particular test.

- fix issue25 avoid reported problems with --pdb and python3.2/encodings output

- fix issue23 - tmpdir argument now works on Python3.2 and WindowsXP
  Starting with Python3.2 os.symlink may be supported. By requiring
  a newer py lib version the py.path.local() implementation acknowledges
  this.

- fixed typos in the docs (thanks Victor Garcia, Brianna Laugher) and particular
  thanks to Laura Creighton who also reviewed parts of the documentation.

- fix slightly wrong output of verbose progress reporting for classes
  (thanks Amaury)

- more precise (avoiding of) deprecation warnings for node.Class|Function accesses

- avoid std unittest assertion helper code in tracebacks (thanks Ronny)

2.0.1 (2011-02-07)
==================

- refine and unify initial capturing so that it works nicely
  even if the logging module is used on an early-loaded conftest.py
  file or plugin.
- allow to omit "()" in test ids to allow for uniform test ids
  as produced by Alfredo's nice pytest.vim plugin.
- fix issue12 - show plugin versions with "--version" and
  "--traceconfig" and also document how to add extra information
  to reporting test header
- fix issue17 (import-* reporting issue on python3) by
  requiring py>1.4.0 (1.4.1 is going to include it)
- fix issue10 (numpy arrays truth checking) by refining
  assertion interpretation in py lib
- fix issue15: make nose compatibility tests compatible
  with python3 (now that nose-1.0 supports python3)
- remove somewhat surprising "same-conftest" detection because
  it ignores conftest.py when they appear in several subdirs.
- improve assertions ("not in"), thanks Floris Bruynooghe
- improve behaviour/warnings when running on top of "python -OO"
  (assertions and docstrings are turned off, leading to potential
  false positives)
- introduce a pytest_cmdline_processargs(args) hook
  to allow dynamic computation of command line arguments.
  This fixes a regression because py.test prior to 2.0
  allowed to set command line options from conftest.py
  files which so far pytest-2.0 only allowed from ini-files now.
- fix issue7: assert failures in doctest modules.
  unexpected failures in doctests will not generally
  show nicer, i.e. within the doctest failing context.
- fix issue9: setup/teardown functions for an xfail-marked
  test will report as xfail if they fail but report as normally
  passing (not xpassing) if they succeed.  This only is true
  for "direct" setup/teardown invocations because teardown_class/
  teardown_module cannot closely relate to a single test.
- fix issue14: no logging errors at process exit
- refinements to "collecting" output on non-ttys
- refine internal plugin registration and --traceconfig output
- introduce a mechanism to prevent/unregister plugins from the
  command line, see http://pytest.org/plugins.html#cmdunregister
- activate resultlog plugin by default
- fix regression wrt yielded tests which due to the
  collection-before-running semantics were not
  setup as with pytest 1.3.4.  Note, however, that
  the recommended and much cleaner way to do test
  parametraization remains the "pytest_generate_tests"
  mechanism, see the docs.

2.0.0 (2010-11-25)
==================

- pytest-2.0 is now its own package and depends on pylib-2.0
- new ability: python -m pytest / python -m pytest.main ability
- new python invocation: pytest.main(args, plugins) to load
  some custom plugins early.
- try harder to run unittest test suites in a more compatible manner
  by deferring setup/teardown semantics to the unittest package.
  also work harder to run twisted/trial and Django tests which
  should now basically work by default.
- introduce a new way to set config options via ini-style files,
  by default setup.cfg and tox.ini files are searched.  The old
  ways (certain environment variables, dynamic conftest.py reading
  is removed).
- add a new "-q" option which decreases verbosity and prints a more
  nose/unittest-style "dot" output.
- fix issue135 - marks now work with unittest test cases as well
- fix issue126 - introduce py.test.set_trace() to trace execution via
  PDB during the running of tests even if capturing is ongoing.
- fix issue123 - new "python -m py.test" invocation for py.test
  (requires Python 2.5 or above)
- fix issue124 - make reporting more resilient against tests opening
  files on filedescriptor 1 (stdout).
- fix issue109 - sibling conftest.py files will not be loaded.
  (and Directory collectors cannot be customized anymore from a Directory's
  conftest.py - this needs to happen at least one level up).
- introduce (customizable) assertion failure representations and enhance
  output on assertion failures for comparisons and other cases (Floris Bruynooghe)
- nose-plugin: pass through type-signature failures in setup/teardown
  functions instead of not calling them (Ed Singleton)
- remove py.test.collect.Directory (follows from a major refactoring
  and simplification of the collection process)
- majorly reduce py.test core code, shift function/python testing to own plugin
- fix issue88 (finding custom test nodes from command line arg)
- refine 'tmpdir' creation, will now create basenames better associated
  with test names (thanks Ronny)
- "xpass" (unexpected pass) tests don't cause exitcode!=0
- fix issue131 / issue60 - importing doctests in __init__ files used as namespace packages
- fix issue93 stdout/stderr is captured while importing conftest.py
- fix bug: unittest collected functions now also can have "pytestmark"
  applied at class/module level
- add ability to use "class" level for cached_setup helper
- fix strangeness: mark.* objects are now immutable, create new instances

1.3.4 (2010-09-14)
==================

- fix issue111: improve install documentation for windows
- fix issue119: fix custom collectability of __init__.py as a module
- fix issue116: --doctestmodules work with __init__.py files as well
- fix issue115: unify internal exception passthrough/catching/GeneratorExit
- fix issue118: new --tb=native for presenting cpython-standard exceptions

1.3.3 (2010-07-30)
==================

- fix issue113: assertion representation problem with triple-quoted strings
  (and possibly other cases)
- make conftest loading detect that a conftest file with the same
  content was already loaded, avoids surprises in nested directory structures
  which can be produced e.g. by Hudson. It probably removes the need to use
  --confcutdir in most cases.
- fix terminal coloring for win32
  (thanks Michael Foord for reporting)
- fix weirdness: make terminal width detection work on stdout instead of stdin
  (thanks Armin Ronacher for reporting)
- remove trailing whitespace in all py/text distribution files

1.3.2 (2010-07-08)
==================

**New features**

- fix issue103:  introduce py.test.raises as context manager, examples::

    with py.test.raises(ZeroDivisionError):
        x = 0
        1 / x

    with py.test.raises(RuntimeError) as excinfo:
        call_something()

    # you may do extra checks on excinfo.value|type|traceback here

  (thanks Ronny Pfannschmidt)

- Funcarg factories can now dynamically apply a marker to a
  test invocation.  This is for example useful if a factory
  provides parameters to a test which are expected-to-fail::

    def pytest_funcarg__arg(request):
        request.applymarker(py.test.mark.xfail(reason="flaky config"))
        ...

    def test_function(arg):
        ...

- improved error reporting on collection and import errors. This makes
  use of a more general mechanism, namely that for custom test item/collect
  nodes ``node.repr_failure(excinfo)`` is now uniformly called so that you can
  override it to return a string error representation of your choice
  which is going to be reported as a (red) string.

- introduce '--junitprefix=STR' option to prepend a prefix
  to all reports in the junitxml file.

**Bug fixes**

- make tests and the ``pytest_recwarn`` plugin in particular fully compatible
  to Python2.7 (if you use the ``recwarn`` funcarg warnings will be enabled so that
  you can properly check for their existence in a cross-python manner).
- refine --pdb: ignore xfailed tests, unify its TB-reporting and
  don't display failures again at the end.
- fix assertion interpretation with the ** operator (thanks Benjamin Peterson)
- fix issue105 assignment on the same line as a failing assertion (thanks Benjamin Peterson)
- fix issue104 proper escaping for test names in junitxml plugin (thanks anonymous)
- fix issue57 -f|--looponfail to work with xpassing tests (thanks Ronny)
- fix issue92 collectonly reporter and --pastebin (thanks Benjamin Peterson)
- fix py.code.compile(source) to generate unique filenames
- fix assertion re-interp problems on PyPy, by defering code
  compilation to the (overridable) Frame.eval class. (thanks Amaury Forgeot)
- fix py.path.local.pyimport() to work with directories
- streamline py.path.local.mkdtemp implementation and usage
- don't print empty lines when showing junitxml-filename
- add optional boolean ignore_errors parameter to py.path.local.remove
- fix terminal writing on win32/python2.4
- py.process.cmdexec() now tries harder to return properly encoded unicode objects
  on all python versions
- install plain py.test/py.which scripts also for Jython, this helps to
  get canonical script paths in virtualenv situations
- make path.bestrelpath(path) return ".", note that when calling
  X.bestrelpath the assumption is that X is a directory.
- make initial conftest discovery ignore "--" prefixed arguments
- fix resultlog plugin when used in a multicpu/multihost xdist situation
  (thanks Jakub Gustak)
- perform distributed testing related reporting in the xdist-plugin
  rather than having dist-related code in the generic py.test
  distribution
- fix homedir detection on Windows
- ship distribute_setup.py version 0.6.13

1.3.1 (2010-05-25)
==================

**New features**

- issue91: introduce new py.test.xfail(reason) helper
  to imperatively mark a test as expected to fail. Can
  be used from within setup and test functions. This is
  useful especially for parametrized tests when certain
  configurations are expected-to-fail.  In this case the
  declarative approach with the @py.test.mark.xfail cannot
  be used as it would mark all configurations as xfail.

- issue102: introduce new --maxfail=NUM option to stop
  test runs after NUM failures.  This is a generalization
  of the '-x' or '--exitfirst' option which is now equivalent
  to '--maxfail=1'.  Both '-x' and '--maxfail' will
  now also print a line near the end indicating the Interruption.

- issue89: allow py.test.mark decorators to be used on classes
  (class decorators were introduced with python2.6) and
  also allow to have multiple markers applied at class/module level
  by specifying a list.

- improve and refine letter reporting in the progress bar:
  .  pass
  f  failed test
  s  skipped tests (reminder: use for dependency/platform mismatch only)
  x  xfailed test (test that was expected to fail)
  X  xpassed test (test that was expected to fail but passed)

  You can use any combination of 'fsxX' with the '-r' extended
  reporting option. The xfail/xpass results will show up as
  skipped tests in the junitxml output - which also fixes
  issue99.

- make py.test.cmdline.main() return the exitstatus instead of raising
  SystemExit and also allow it to be called multiple times.  This of
  course requires that your application and tests are properly teared
  down and don't have global state.

**Bug Fixes**

- improved traceback presentation:
  - improved and unified reporting for "--tb=short" option
  - Errors during test module imports are much shorter, (using --tb=short style)
  - raises shows shorter more relevant tracebacks
  - --fulltrace now more systematically makes traces longer / inhibits cutting

- improve support for raises and other dynamically compiled code by
  manipulating python's linecache.cache instead of the previous
  rather hacky way of creating custom code objects.  This makes
  it seemlessly work on Jython and PyPy where it previously didn't.

- fix issue96: make capturing more resilient against Control-C
  interruptions (involved somewhat substantial refactoring
  to the underlying capturing functionality to avoid race
  conditions).

- fix chaining of conditional skipif/xfail decorators - so it works now
  as expected to use multiple @py.test.mark.skipif(condition) decorators,
  including specific reporting which of the conditions lead to skipping.

- fix issue95: late-import zlib so that it's not required
  for general py.test startup.

- fix issue94: make reporting more robust against bogus source code
  (and internally be more careful when presenting unexpected byte sequences)


1.3.0 (2010-05-05)
==================

- deprecate --report option in favour of a new shorter and easier to
  remember -r option: it takes a string argument consisting of any
  combination of 'xfsX' characters.  They relate to the single chars
  you see during the dotted progress printing and will print an extra line
  per test at the end of the test run.  This extra line indicates the exact
  position or test ID that you directly paste to the py.test cmdline in order
  to re-run a particular test.

- allow external plugins to register new hooks via the new
  pytest_addhooks(pluginmanager) hook.  The new release of
  the pytest-xdist plugin for distributed and looponfailing
  testing requires this feature.

- add a new pytest_ignore_collect(path, config) hook to allow projects and
  plugins to define exclusion behaviour for their directory structure -
  for example you may define in a conftest.py this method::

        def pytest_ignore_collect(path):
            return path.check(link=1)

  to prevent even a collection try of any tests in symlinked dirs.

- new pytest_pycollect_makemodule(path, parent) hook for
  allowing customization of the Module collection object for a
  matching test module.

- extend and refine xfail mechanism:
  ``@py.test.mark.xfail(run=False)`` do not run the decorated test
  ``@py.test.mark.xfail(reason="...")`` prints the reason string in xfail summaries
  specifying ``--runxfail`` on command line virtually ignores xfail markers

- expose (previously internal) commonly useful methods:
  py.io.get_terminal_with() -> return terminal width
  py.io.ansi_print(...) -> print colored/bold text on linux/win32
  py.io.saferepr(obj) -> return limited representation string

- expose test outcome related exceptions as py.test.skip.Exception,
  py.test.raises.Exception etc., useful mostly for plugins
  doing special outcome interpretation/tweaking

- (issue85) fix junitxml plugin to handle tests with non-ascii output

- fix/refine python3 compatibility (thanks Benjamin Peterson)

- fixes for making the jython/win32 combination work, note however:
  jython2.5.1/win32 does not provide a command line launcher, see
  http://bugs.jython.org/issue1491 . See pylib install documentation
  for how to work around.

- fixes for handling of unicode exception values and unprintable objects

- (issue87) fix unboundlocal error in assertionold code

- (issue86) improve documentation for looponfailing

- refine IO capturing: stdin-redirect pseudo-file now has a NOP close() method

- ship distribute_setup.py version 0.6.10

- added links to the new capturelog and coverage plugins


1.2.0 (2010-01-18)
==================

- refined usage and options for "py.cleanup"::

    py.cleanup     # remove "*.pyc" and "*$py.class" (jython) files
    py.cleanup -e .swp -e .cache # also remove files with these extensions
    py.cleanup -s  # remove "build" and "dist" directory next to setup.py files
    py.cleanup -d  # also remove empty directories
    py.cleanup -a  # synonym for "-s -d -e 'pip-log.txt'"
    py.cleanup -n  # dry run, only show what would be removed

- add a new option "py.test --funcargs" which shows available funcargs
  and their help strings (docstrings on their respective factory function)
  for a given test path

- display a short and concise traceback if a funcarg lookup fails

- early-load "conftest.py" files in non-dot first-level sub directories.
  allows to conveniently keep and access test-related options in a ``test``
  subdir and still add command line options.

- fix issue67: new super-short traceback-printing option: "--tb=line" will print a single line for each failing (python) test indicating its filename, lineno and the failure value

- fix issue78: always call python-level teardown functions even if the
  according setup failed.  This includes refinements for calling setup_module/class functions
  which will now only be called once instead of the previous behaviour where they'd be called
  multiple times if they raise an exception (including a Skipped exception).  Any exception
  will be re-corded and associated with all tests in the according module/class scope.

- fix issue63: assume <40 columns to be a bogus terminal width, default to 80

- fix pdb debugging to be in the correct frame on raises-related errors

- update apipkg.py to fix an issue where recursive imports might
  unnecessarily break importing

- fix plugin links

1.1.1 (2009-11-24)
==================

- moved dist/looponfailing from py.test core into a new
  separately released pytest-xdist plugin.

- new junitxml plugin: --junitxml=path will generate a junit style xml file
  which is processable e.g. by the Hudson CI system.

- new option: --genscript=path will generate a standalone py.test script
  which will not need any libraries installed.  thanks to Ralf Schmitt.

- new option: --ignore will prevent specified path from collection.
  Can be specified multiple times.

- new option: --confcutdir=dir will make py.test only consider conftest
  files that are relative to the specified dir.

- new funcarg: "pytestconfig" is the pytest config object for access
  to command line args and can now be easily used in a test.

- install ``py.test`` and ``py.which`` with a ``-$VERSION`` suffix to
  disambiguate between Python3, python2.X, Jython and PyPy installed versions.

- new "pytestconfig" funcarg allows access to test config object

- new "pytest_report_header" hook can return additional lines
  to be displayed at the header of a test run.

- (experimental) allow "py.test path::name1::name2::..." for pointing
  to a test within a test collection directly.  This might eventually
  evolve as a full substitute to "-k" specifications.

- streamlined plugin loading: order is now as documented in
  customize.html: setuptools, ENV, commandline, conftest.
  also setuptools entry point names are turned to canonical namees ("pytest_*")

- automatically skip tests that need 'capfd' but have no os.dup

- allow pytest_generate_tests to be defined in classes as well

- deprecate usage of 'disabled' attribute in favour of pytestmark
- deprecate definition of Directory, Module, Class and Function nodes
  in conftest.py files.  Use pytest collect hooks instead.

- collection/item node specific runtest/collect hooks are only called exactly
  on matching conftest.py files, i.e. ones which are exactly below
  the filesystem path of an item

- change: the first pytest_collect_directory hook to return something
  will now prevent further hooks to be called.

- change: figleaf plugin now requires --figleaf to run.  Also
  change its long command line options to be a bit shorter (see py.test -h).

- change: pytest doctest plugin is now enabled by default and has a
  new option --doctest-glob to set a pattern for file matches.

- change: remove internal py._* helper vars, only keep py._pydir

- robustify capturing to survive if custom pytest_runtest_setup
  code failed and prevented the capturing setup code from running.

- make py.test.* helpers provided by default plugins visible early -
  works transparently both for pydoc and for interactive sessions
  which will regularly see e.g. py.test.mark and py.test.importorskip.

- simplify internal plugin manager machinery
- simplify internal collection tree by introducing a RootCollector node

- fix assert reinterpreation that sees a call containing "keyword=..."

- fix issue66: invoke pytest_sessionstart and pytest_sessionfinish
  hooks on slaves during dist-testing, report module/session teardown
  hooks correctly.

- fix issue65: properly handle dist-testing if no
  execnet/py lib installed remotely.

- skip some install-tests if no execnet is available

- fix docs, fix internal bin/ script generation


1.1.0 (2009-11-05)
==================

- introduce automatic plugin registration via 'pytest11'
  entrypoints via setuptools' pkg_resources.iter_entry_points

- fix py.test dist-testing to work with execnet >= 1.0.0b4

- re-introduce py.test.cmdline.main() for better backward compatibility

- svn paths: fix a bug with path.check(versioned=True) for svn paths,
  allow '%' in svn paths, make svnwc.update() default to interactive mode
  like in 1.0.x and add svnwc.update(interactive=False) to inhibit interaction.

- refine distributed tarball to contain test and no pyc files

- try harder to have deprecation warnings for py.compat.* accesses
  report a correct location

1.0.3
=====

* adjust and improve docs

* remove py.rest tool and internal namespace - it was
  never really advertised and can still be used with
  the old release if needed.  If there is interest
  it could be revived into its own tool i guess.

* fix issue48 and issue59: raise an Error if the module
  from an imported test file does not seem to come from
  the filepath - avoids "same-name" confusion that has
  been reported repeatedly

* merged Ronny's nose-compatibility hacks: now
  nose-style setup_module() and setup() functions are
  supported

* introduce generalized py.test.mark function marking

* reshuffle / refine command line grouping

* deprecate parser.addgroup in favour of getgroup which creates option group

* add --report command line option that allows to control showing of skipped/xfailed sections

* generalized skipping: a new way to mark python functions with skipif or xfail
  at function, class and modules level based on platform or sys-module attributes.

* extend py.test.mark decorator to allow for positional args

* introduce and test "py.cleanup -d" to remove empty directories

* fix issue #59 - robustify unittest test collection

* make bpython/help interaction work by adding an __all__ attribute
  to ApiModule, cleanup initpkg

* use MIT license for pylib, add some contributors

* remove py.execnet code and substitute all usages with 'execnet' proper

* fix issue50 - cached_setup now caches more to expectations
  for test functions with multiple arguments.

* merge Jarko's fixes, issue #45 and #46

* add the ability to specify a path for py.lookup to search in

* fix a funcarg cached_setup bug probably only occurring
  in distributed testing and "module" scope with teardown.

* many fixes and changes for making the code base python3 compatible,
  many thanks to Benjamin Peterson for helping with this.

* consolidate builtins implementation to be compatible with >=2.3,
  add helpers to ease keeping 2 and 3k compatible code

* deprecate py.compat.doctest|subprocess|textwrap|optparse

* deprecate py.magic.autopath, remove py/magic directory

* move pytest assertion handling to py/code and a pytest_assertion
  plugin, add "--no-assert" option, deprecate py.magic namespaces
  in favour of (less) py.code ones.

* consolidate and cleanup py/code classes and files

* cleanup py/misc, move tests to bin-for-dist

* introduce delattr/delitem/delenv methods to py.test's monkeypatch funcarg

* consolidate py.log implementation, remove old approach.

* introduce py.io.TextIO and py.io.BytesIO for distinguishing between
  text/unicode and byte-streams (uses underlying standard lib io.*
  if available)

* make py.unittest_convert helper script available which converts "unittest.py"
  style files into the simpler assert/direct-test-classes py.test/nosetests
  style.  The script was written by Laura Creighton.

* simplified internal localpath implementation

1.0.2 (2009-08-27)
==================

* fixing packaging issues, triggered by fedora redhat packaging,
  also added doc, examples and contrib dirs to the tarball.

* added a documentation link to the new django plugin.

1.0.1 (2009-08-19)
==================

* added a 'pytest_nose' plugin which handles nose.SkipTest,
  nose-style function/method/generator setup/teardown and
  tries to report functions correctly.

* capturing of unicode writes or encoded strings to sys.stdout/err
  work better, also terminalwriting was adapted and somewhat
  unified between windows and linux.

* improved documentation layout and content a lot

* added a "--help-config" option to show conftest.py / ENV-var names for
  all longopt cmdline options, and some special conftest.py variables.
  renamed 'conf_capture' conftest setting to 'option_capture' accordingly.

* fix issue #27: better reporting on non-collectable items given on commandline
  (e.g. pyc files)

* fix issue #33: added --version flag (thanks Benjamin Peterson)

* fix issue #32: adding support for "incomplete" paths to wcpath.status()

* "Test" prefixed classes are *not* collected by default anymore if they
  have an __init__ method

* monkeypatch setenv() now accepts a "prepend" parameter

* improved reporting of collection error tracebacks

* simplified multicall mechanism and plugin architecture,
  renamed some internal methods and argnames

1.0.0 (2009-08-04)
==================

* more terse reporting try to show filesystem path relatively to current dir
* improve xfail output a bit

1.0.0b9 (2009-07-31)
====================

* cleanly handle and report final teardown of test setup

* fix svn-1.6 compat issue with py.path.svnwc().versioned()
  (thanks Wouter Vanden Hove)

* setup/teardown or collection problems now show as ERRORs
  or with big "E"'s in the progress lines.  they are reported
  and counted separately.

* dist-testing: properly handle test items that get locally
  collected but cannot be collected on the remote side - often
  due to platform/dependency reasons

* simplified py.test.mark API - see keyword plugin documentation

* integrate better with logging: capturing now by default captures
  test functions and their immediate setup/teardown in a single stream

* capsys and capfd funcargs now have a readouterr() and a close() method
  (underlyingly py.io.StdCapture/FD objects are used which grew a
  readouterr() method as well to return snapshots of captured out/err)

* make assert-reinterpretation work better with comparisons not
  returning bools (reported with numpy from thanks maciej fijalkowski)

* reworked per-test output capturing into the pytest_iocapture.py plugin
  and thus removed capturing code from config object

* item.repr_failure(excinfo) instead of item.repr_failure(excinfo, outerr)


1.0.0b8 (2009-07-22)
====================

* pytest_unittest-plugin is now enabled by default

* introduced pytest_keyboardinterrupt hook and
  refined pytest_sessionfinish hooked, added tests.

* workaround a buggy logging module interaction ("closing already closed
  files").  Thanks to Sridhar Ratnakumar for triggering.

* if plugins use "py.test.importorskip" for importing
  a dependency only a warning will be issued instead
  of exiting the testing process.

* many improvements to docs:
  - refined funcargs doc , use the term "factory" instead of "provider"
  - added a new talk/tutorial doc page
  - better download page
  - better plugin docstrings
  - added new plugins page and automatic doc generation script

* fixed teardown problem related to partially failing funcarg setups
  (thanks MrTopf for reporting), "pytest_runtest_teardown" is now
  always invoked even if the "pytest_runtest_setup" failed.

* tweaked doctest output for docstrings in py modules,
  thanks Radomir.

1.0.0b7
=======

* renamed py.test.xfail back to py.test.mark.xfail to avoid
  two ways to decorate for xfail

* re-added py.test.mark decorator for setting keywords on functions
  (it was actually documented so removing it was not nice)

* remove scope-argument from request.addfinalizer() because
  request.cached_setup has the scope arg. TOOWTDI.

* perform setup finalization before reporting failures

* apply modified patches from Andreas Kloeckner to allow
  test functions to have no func_code (#22) and to make
  "-k" and function keywords work  (#20)

* apply patch from Daniel Peolzleithner (issue #23)

* resolve issue #18, multiprocessing.Manager() and
  redirection clash

* make __name__ == "__channelexec__" for remote_exec code

1.0.0b3 (2009-06-19)
====================

* plugin classes are removed: one now defines
  hooks directly in conftest.py or global pytest_*.py
  files.

* added new pytest_namespace(config) hook that allows
  to inject helpers directly to the py.test.* namespace.

* documented and refined many hooks

* added new style of generative tests via
  pytest_generate_tests hook that integrates
  well with function arguments.


1.0.0b1
=======

* introduced new "funcarg" setup method,
  see doc/test/funcarg.txt

* introduced plugin architecture and many
  new py.test plugins, see
  doc/test/plugins.txt

* teardown_method is now guaranteed to get
  called after a test method has run.

* new method: py.test.importorskip(mod,minversion)
  will either import or call py.test.skip()

* completely revised internal py.test architecture

* new py.process.ForkedFunc object allowing to
  fork execution of a function to a sub process
  and getting a result back.

XXX lots of things missing here XXX

0.9.2
=====

* refined installation and metadata, created new setup.py,
  now based on setuptools/ez_setup (thanks to Ralf Schmitt
  for his support).

* improved the way of making py.* scripts available in
  windows environments, they are now added to the
  Scripts directory as ".cmd" files.

* py.path.svnwc.status() now is more complete and
  uses xml output from the 'svn' command if available
  (Guido Wesdorp)

* fix for py.path.svn* to work with svn 1.5
  (Chris Lamb)

* fix path.relto(otherpath) method on windows to
  use normcase for checking if a path is relative.

* py.test's traceback is better parseable from editors
  (follows the filenames:LINENO: MSG convention)
  (thanks to Osmo Salomaa)

* fix to javascript-generation, "py.test --runbrowser"
  should work more reliably now

* removed previously accidentally added
  py.test.broken and py.test.notimplemented helpers.

* there now is a py.__version__ attribute

0.9.1
=====

This is a fairly complete list of v0.9.1, which can
serve as a reference for developers.

* allowing + signs in py.path.svn urls [39106]
* fixed support for Failed exceptions without excinfo in py.test [39340]
* added support for killing processes for Windows (as well as platforms that
  support os.kill) in py.misc.killproc [39655]
* added setup/teardown for generative tests to py.test [40702]
* added detection of FAILED TO LOAD MODULE to py.test [40703, 40738, 40739]
* fixed problem with calling .remove() on wcpaths of non-versioned files in
  py.path [44248]
* fixed some import and inheritance issues in py.test [41480, 44648, 44655]
* fail to run greenlet tests when pypy is available, but without stackless
  [45294]
* small fixes in rsession tests [45295]
* fixed issue with 2.5 type representations in py.test [45483, 45484]
* made that internal reporting issues displaying is done atomically in py.test
  [45518]
* made that non-existing files are ignored by the py.lookup script [45519]
* improved exception name creation in py.test [45535]
* made that less threads are used in execnet [merge in 45539]
* removed lock required for atomic reporting issue displaying in py.test
  [45545]
* removed globals from execnet [45541, 45547]
* refactored cleanup mechanics, made that setDaemon is set to 1 to make atexit
  get called in 2.5 (py.execnet) [45548]
* fixed bug in joining threads in py.execnet's servemain [45549]
* refactored py.test.rsession tests to not rely on exact output format anymore
  [45646]
* using repr() on test outcome [45647]
* added 'Reason' classes for py.test.skip() [45648, 45649]
* killed some unnecessary sanity check in py.test.collect [45655]
* avoid using os.tmpfile() in py.io.fdcapture because on Windows it's only
  usable by Administrators [45901]
* added support for locking and non-recursive commits to py.path.svnwc [45994]
* locking files in py.execnet to prevent CPython from segfaulting [46010]
* added export() method to py.path.svnurl
* fixed -d -x in py.test [47277]
* fixed argument concatenation problem in py.path.svnwc [49423]
* restore py.test behaviour that it exits with code 1 when there are failures
  [49974]
* don't fail on html files that don't have an accompanying .txt file [50606]
* fixed 'utestconvert.py < input' [50645]
* small fix for code indentation in py.code.source [50755]
* fix _docgen.py documentation building [51285]
* improved checks for source representation of code blocks in py.test [51292]
* added support for passing authentication to py.path.svn* objects [52000,
  52001]
* removed sorted() call for py.apigen tests in favour of [].sort() to support
  Python 2.3 [52481]
