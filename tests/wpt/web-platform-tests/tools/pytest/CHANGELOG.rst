2.9.1
=====

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


.. _#1437: https://github.com/pytest-dev/pytest/issues/1437
.. _#469: https://github.com/pytest-dev/pytest/issues/469
.. _#1431: https://github.com/pytest-dev/pytest/pull/1431
.. _#649: https://github.com/pytest-dev/pytest/issues/649

.. _@asottile: https://github.com/asottile


2.9.0
=====

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

* **Important**: `py.code <http://pylib.readthedocs.org/en/latest/code.html>`_ has been
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
  `pylib <http://pylib.readthedocs.org>`_.

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
  (auto/long/short/line/native/no), with `auto` being the default since v2.6.
  Thanks `@hackebrot`_ for the PR.

* Fix (`#1422`_): junit record_xml_property doesn't allow multiple records
  with same name.

.. _`traceback style docs`: https://pytest.org/latest/usage.html#modifying-python-traceback-printing

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

2.8.7
=====

- fix #1338: use predictable object resolution for monkeypatch

2.8.6
=====

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


2.8.5
=====

- fix #1243: fixed issue where class attributes injected during collection could break pytest.
  PR by Alexei Kozlenok, thanks Ronny Pfannschmidt and Bruno Oliveira for the review and help.

- fix #1074: precompute junitxml chunks instead of storing the whole tree in objects
  Thanks Bruno Oliveira for the report and Ronny Pfannschmidt for the PR

- fix #1238: fix ``pytest.deprecated_call()`` receiving multiple arguments
  (Regression introduced in 2.8.4). Thanks Alex Gaynor for the report and
  Bruno Oliveira for the PR.


2.8.4
=====

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

2.8.3
=====

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


2.8.2
=====

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

2.8.1
=====

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
  aimed for micro releases.  "features" branch will only be be released
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

2.8.0
=====

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
  to run test modules against installated versions of a package even if the
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

2.7.3 (compared to 2.7.2)
=============================

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

2.7.2 (compared to 2.7.1)
=============================

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


2.7.1 (compared to 2.7.0)
=============================

- fix issue731: do not get confused by the braces which may be present
  and unbalanced in an object's repr while collapsing False
  explanations.  Thanks Carl Meyer for the report and test case.

- fix issue553: properly handling inspect.getsourcelines failures in
  FixtureLookupError which would lead to to an internal error,
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

2.7.0 (compared to 2.6.4)
=============================

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

- fix issue615: assertion re-writing did not correctly escape % signs
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

2.6.4
=====

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

2.6.3
=====

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
  CWD outside a test package pytest would get fixture discovery wrong.
  Thanks to Wolfgang Schnerring for figuring out a reproducable example.

- Introduce pytest_enter_pdb hook (needed e.g. by pytest_timeout to cancel the
  timeout when interactively entering pdb).  Thanks Wolfgang Schnerring.

- check xfail/skip also with non-python function test items. Thanks
  Floris Bruynooghe.

2.6.2
=====

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

2.6.1
=====

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
  and if the part has an ".py" extension

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


2.5.2
=====

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

- make capfd/capsys.capture private, its unused and shouldnt be exposed


2.5.1
=====

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



2.5.0
=====

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
  with repeated same values (sometimes useful to to test if calling
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
  arg is an lambda and thus the example will work.  Thanks Alex Gaynor
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

2.4.2
=====

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

2.4.1
=====

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
  finalizer mimick the same generic behaviour: if a setupX exists and
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

2.3.5
=====

- fix issue169: respect --tb=style with setup/teardown errors as well.

- never consider a fixture function for test function collection

- allow re-running of test items / helps to fix pytest-reruntests plugin
  and also help to keep less fixture/resource references alive

- put captured stdout/stderr into junitxml output even for passing tests
  (thanks Adam Goucher)

- Issue 265 - integrate nose setup/teardown with setupstate
  so it doesnt try to teardown if it did not setup

- issue 271 - dont write junitxml on slave nodes

- Issue 274 - dont try to show full doctest example
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

- improve PYTEST_DEBUG tracing output by puting
  extra data on a new lines with additional indent

- ensure OutcomeExceptions like skip/fail have initialized exception attributes

- issue 260 - don't use nose special setup on plain unittest cases

- fix issue134 - print the collect errors that prevent running specified test items

- fix issue266 - accept unicode in MarkEvaluator expressions

2.3.4
=====

- yielded test functions will now have autouse-fixtures active but
  cannot accept fixtures as funcargs - it's anyway recommended to
  rather use the post-2.0 parametrize features instead of yield, see:
  http://pytest.org/latest/example/parametrize.html
- fix autouse-issue where autouse-fixtures would not be discovered
  if defined in a a/conftest.py file and tests in a/tests/test_some.py
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

2.3.3
=====

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

- nose support: only call setup if its a callable, thanks Andrew
  Taumoefolau

- fix issue219 - add py2.4-3.3 classifiers to TROVE list

- in tracebacks *,** arg values are now shown next to normal arguments
  (thanks Manuel Jacob)

- fix issue217 - support mock.patch with pytest's fixtures - note that
  you need either mock-1.0.1 or the python3.3 builtin unittest.mock.

- fix issue127 - improve documentation for pytest_addoption() and
  add a ``config.getoption(name)`` helper function for consistency.

2.3.2
=====

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

2.3.1
=====

- fix issue202 - fix regression: using "self" from fixture functions now
  works as expected (it's the same "self" instance that a test method
  which uses the fixture sees)

- skip pexpect using tests (test_pdb.py mostly) on freebsd* systems
  due to pexpect not supporting it properly (hanging)

- link to web pages from --markers output which provides help for
  pytest.mark.* usage.

2.3.0
=====

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

- fix issue179: propperly show the dependency chain of factories

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

  - py.test -vv will show all of assert comparisations instead of truncating

2.2.4
=====

- fix error message for rewritten assertions involving the % operator
- fix issue 126: correctly match all invalid xml characters for junitxml
  binary escape
- fix issue with unittest: now @unittest.expectedFailure markers should
  be processed correctly (you can also use @pytest.mark markers)
- document integration with the extended distribute/setuptools test commands
- fix issue 140: propperly get the real functions
  of bound classmethods for setup/teardown_class
- fix issue #141: switch from the deceased paste.pocoo.org to bpaste.net
- fix issue #143: call unconfigure/sessionfinish always when
  configure/sessionstart where called
- fix issue #144: better mangle test ids to junitxml classnames
- upgrade distribute_setup.py to 0.6.27

2.2.3
=====

- fix uploaded package to only include neccesary files

2.2.2
=====

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

2.2.1
=====

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

2.2.0
=====

- fix issue90: introduce eager tearing down of test items so that
  teardown function are called earlier.
- add an all-powerful metafunc.parametrize function which allows to
  parametrize test function arguments in multiple steps and therefore
  from indepdenent plugins and palces.
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

2.1.3
=====

- fix issue79: assertion rewriting failed on some comparisons in boolops
- correctly handle zero length arguments (a la pytest '')
- fix issue67 / junitxml now contains correct test durations, thanks ronny
- fix issue75 / skipping test failure on jython
- fix issue77 / Allow assertrepr_compare hook to apply to a subset of tests

2.1.2
=====

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

2.1.0
=====

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

2.0.3
=====

- fix issue38: nicer tracebacks on calls to hooks, particularly early
  configure/sessionstart ones

- fix missing skip reason/meta information in junitxml files, reported
  via http://lists.idyll.org/pipermail/testing-in-python/2011-March/003928.html

- fix issue34: avoid collection failure with "test" prefixed classes
  deriving from object.

- don't require zlib (and other libs) for genscript plugin without
  --genscript actually being used.

- speed up skips (by not doing a full traceback represenation
  internally)

- fix issue37: avoid invalid characters in junitxml's output

2.0.2
=====

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
  thanks to Laura Creighton who also revieved parts of the documentation.

- fix slighly wrong output of verbose progress reporting for classes
  (thanks Amaury)

- more precise (avoiding of) deprecation warnings for node.Class|Function accesses

- avoid std unittest assertion helper code in tracebacks (thanks Ronny)

2.0.1
=====

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

2.0.0
=====

- pytest-2.0 is now its own package and depends on pylib-2.0
- new ability: python -m pytest / python -m pytest.main ability
- new python invcation: pytest.main(args, plugins) to load
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

1.3.4
=====

- fix issue111: improve install documentation for windows
- fix issue119: fix custom collectability of __init__.py as a module
- fix issue116: --doctestmodules work with __init__.py files as well
- fix issue115: unify internal exception passthrough/catching/GeneratorExit
- fix issue118: new --tb=native for presenting cpython-standard exceptions

1.3.3
=====

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

1.3.2
=====

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
- fix resultlog plugin when used in an multicpu/multihost xdist situation
  (thanks Jakub Gustak)
- perform distributed testing related reporting in the xdist-plugin
  rather than having dist-related code in the generic py.test
  distribution
- fix homedir detection on Windows
- ship distribute_setup.py version 0.6.13

1.3.1
=====

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


1.3.0
=====

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
  specifiying ``--runxfail`` on command line virtually ignores xfail markers

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


1.2.0
=====

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

1.1.1
=====

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


1.1.0
=====

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

* fix a funcarg cached_setup bug probably only occuring
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

1.0.2
=====

* fixing packaging issues, triggered by fedora redhat packaging,
  also added doc, examples and contrib dirs to the tarball.

* added a documentation link to the new django plugin.

1.0.1
=====

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

1.0.0
=====

* more terse reporting try to show filesystem path relatively to current dir
* improve xfail output a bit

1.0.0b9
=======

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


1.0.0b8
=======

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

1.0.0b3
=======

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
* made that non-existing files are igored by the py.lookup script [45519]
* improved exception name creation in py.test [45535]
* made that less threads are used in execnet [merge in 45539]
* removed lock required for atomical reporting issue displaying in py.test
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
