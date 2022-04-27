py.test 2.0.1: bug fixes
===========================================================================

Welcome to pytest-2.0.1, a maintenance and bug fix release of pytest,
a mature testing tool for Python, supporting CPython 2.4-3.2, Jython
and latest PyPy interpreters.  See extensive docs with tested examples here:

    http://pytest.org/

If you want to install or upgrade pytest, just type one of::

    pip install -U pytest # or
    easy_install -U pytest

Many thanks to all issue reporters and people asking questions or
complaining.  Particular thanks to Floris Bruynooghe and Ronny Pfannschmidt
for their great coding contributions and many others for feedback and help.

best,
holger krekel

Changes between 2.0.0 and 2.0.1
----------------------------------------------

- refine and unify initial capturing so that it works nicely
  even if the logging module is used on an early-loaded conftest.py
  file or plugin.
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
  command line, see http://pytest.org/en/stable/how-to/plugins.html#cmdunregister
- activate resultlog plugin by default
- fix regression wrt yielded tests which due to the
  collection-before-running semantics were not
  setup as with pytest 1.3.4.  Note, however, that
  the recommended and much cleaner way to do test
  parametrization remains the "pytest_generate_tests"
  mechanism, see the docs.
