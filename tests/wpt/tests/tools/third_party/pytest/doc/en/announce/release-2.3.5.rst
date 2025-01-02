pytest-2.3.5: bug fixes and little improvements
===========================================================================

pytest-2.3.5 is a maintenance release with many bug fixes and little
improvements.  See the changelog below for details.  No backward
compatibility issues are foreseen and all plugins which worked with the
prior version are expected to work unmodified.   Speaking of which, a
few interesting new plugins saw the light last month:

- pytest-instafail: show failure information while tests are running
- pytest-qt: testing of GUI applications written with QT/Pyside
- pytest-xprocess: managing external processes across test runs
- pytest-random: randomize test ordering

And several others like pytest-django saw maintenance releases.
For a more complete list, check out
https://pypi.org/search/?q=pytest

For general information see:

     http://pytest.org/

To install or upgrade pytest:

    pip install -U pytest # or
    easy_install -U pytest

Particular thanks to Floris, Ronny, Benjamin and the many bug reporters
and fix providers.

may the fixtures be with you,
holger krekel


Changes between 2.3.4 and 2.3.5
-----------------------------------

- never consider a fixture function for test function collection

- allow re-running of test items / helps to fix pytest-reruntests plugin
  and also help to keep less fixture/resource references alive

- put captured stdout/stderr into junitxml output even for passing tests
  (thanks Adam Goucher)

- Issue 265 - integrate nose setup/teardown with setupstate
  so it doesn't try to teardown if it did not setup

- issue 271 - don't write junitxml on worker nodes

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
