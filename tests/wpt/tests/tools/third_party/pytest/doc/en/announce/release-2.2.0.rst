py.test 2.2.0: test marking++, parametrization++ and duration profiling
===========================================================================

pytest-2.2.0 is a test-suite compatible release of the popular
py.test testing tool.  Plugins might need upgrades. It comes
with these improvements:

* easier and more powerful parametrization of tests:

  - new @pytest.mark.parametrize decorator to run tests with different arguments
  - new metafunc.parametrize() API for parametrizing arguments independently
  - see examples at http://pytest.org/en/stable/example/how-to/parametrize.html
  - NOTE that parametrize() related APIs are still a bit experimental
    and might change in future releases.

* improved handling of test markers and refined marking mechanism:

  - "-m markexpr" option for selecting tests according to their mark
  - a new "markers" ini-variable for registering test markers for your project
  - the new "--strict" bails out with an error if using unregistered markers.
  - see examples at http://pytest.org/en/stable/example/markers.html

* duration profiling: new "--duration=N" option showing the N slowest test
  execution or setup/teardown calls. This is most useful if you want to
  find out where your slowest test code is.

* also 2.2.0 performs more eager calling of teardown/finalizers functions
  resulting in better and more accurate reporting when they fail

Besides there is the usual set of bug fixes along with a cleanup of
pytest's own test suite allowing it to run on a wider range of environments.

For general information, see extensive docs with examples here:

     http://pytest.org/

If you want to install or upgrade pytest you might just type::

    pip install -U pytest # or
    easy_install -U pytest

Thanks to Ronny Pfannschmidt, David Burns, Jeff Donner, Daniel Nouri, Alfredo Deza and all who gave feedback or sent bug reports.

best,
holger krekel


notes on incompatibility
------------------------------

While test suites should work unchanged you might need to upgrade plugins:

* You need a new version of the pytest-xdist plugin (1.7) for distributing
  test runs.

* Other plugins might need an upgrade if they implement
  the ``pytest_runtest_logreport`` hook which now is called unconditionally
  for the setup/teardown fixture phases of a test. You may choose to
  ignore setup/teardown failures by inserting "if rep.when != 'call': return"
  or something similar. Note that most code probably "just" works because
  the hook was already called for failing setup/teardown phases of a test
  so a plugin should have been ready to grok such reports already.


Changes between 2.1.3 and 2.2.0
----------------------------------------

- fix issue90: introduce eager tearing down of test items so that
  teardown function are called earlier.
- add an all-powerful metafunc.parametrize function which allows to
  parametrize test function arguments in multiple steps and therefore
  from independent plugins and places.
- add a @pytest.mark.parametrize helper which allows to easily
  call a test function with different argument values.
- Add examples to the "parametrize" example page, including a quick port
  of Test scenarios and the new parametrize function and decorator.
- introduce registration for "pytest.mark.*" helpers via ini-files
  or through plugin hooks.  Also introduce a "--strict" option which
  will treat unregistered markers as errors
  allowing to avoid typos and maintain a well described set of markers
  for your test suite.  See examples at http://pytest.org/en/stable/how-to/mark.html
  and its links.
- issue50: introduce "-m marker" option to select tests based on markers
  (this is a stricter and more predictable version of "-k" in that "-m"
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
