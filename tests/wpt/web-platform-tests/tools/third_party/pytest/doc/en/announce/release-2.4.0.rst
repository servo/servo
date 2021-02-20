pytest-2.4.0: new fixture features/hooks and bug fixes
===========================================================================

The just released pytest-2.4.0 brings many improvements and numerous
bug fixes while remaining plugin- and test-suite compatible apart
from a few supposedly very minor incompatibilities.  See below for
a full list of details.  A few feature highlights:

- new yield-style fixtures `pytest.yield_fixture
  <http://pytest.org/en/stable/yieldfixture.html>`_, allowing to use
  existing with-style context managers in fixture functions.

- improved pdb support: ``import pdb ; pdb.set_trace()`` now works
  without requiring prior disabling of stdout/stderr capturing.
  Also the ``--pdb`` options works now on collection and internal errors
  and we introduced a new experimental hook for IDEs/plugins to
  intercept debugging: ``pytest_exception_interact(node, call, report)``.

- shorter monkeypatch variant to allow specifying an import path as
  a target, for example: ``monkeypatch.setattr("requests.get", myfunc)``

- better unittest/nose compatibility: all teardown methods are now only
  called if the corresponding setup method succeeded.

- integrate tab-completion on command line options if you
  have `argcomplete <https://pypi.org/project/argcomplete/>`_
  configured.

- allow boolean expression directly with skipif/xfail
  if a "reason" is also specified.

- a new hook ``pytest_load_initial_conftests`` allows plugins like
  `pytest-django <https://pypi.org/project/pytest-django/>`_ to
  influence the environment before conftest files import ``django``.

- reporting: color the last line red or green depending if
  failures/errors occurred or everything passed.

The documentation has been updated to accommodate the changes,
see `http://pytest.org <http://pytest.org>`_

To install or upgrade pytest::

    pip install -U pytest # or
    easy_install -U pytest


**Many thanks to all who helped, including Floris Bruynooghe,
Brianna Laugher, Andreas Pelme, Anthon van der Neut, Anatoly Bubenkoff,
Vladimir Keleshev, Mathieu Agopian, Ronny Pfannschmidt, Christian
Theunert and many others.**

may passing tests be with you,

holger krekel

Changes between 2.3.5 and 2.4
-----------------------------------

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
