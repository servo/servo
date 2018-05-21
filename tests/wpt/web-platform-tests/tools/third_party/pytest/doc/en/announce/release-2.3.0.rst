pytest-2.3: improved fixtures / better unittest integration
=============================================================================

pytest-2.3 comes with many major improvements for fixture/funcarg management 
and parametrized testing in Python.  It is now easier, more efficient and
more predicatable to re-run the same tests with different fixture
instances.  Also, you can directly declare the caching "scope" of
fixtures so that dependent tests throughout your whole test suite can
re-use database or other expensive fixture objects with ease.  Lastly,
it's possible for fixture functions (formerly known as funcarg
factories) to use other fixtures, allowing for a completely modular and
re-useable fixture design. 

For detailed info and tutorial-style examples, see:

    http://pytest.org/latest/fixture.html

Moreover, there is now support for using pytest fixtures/funcargs with
unittest-style suites, see here for examples:

    http://pytest.org/latest/unittest.html

Besides, more unittest-test suites are now expected to "simply work"
with pytest.

All changes are backward compatible and you should be able to continue
to run your test suites and 3rd party plugins that worked with
pytest-2.2.4.

If you are interested in the precise reasoning (including examples) of the 
pytest-2.3 fixture evolution, please consult
http://pytest.org/latest/funcarg_compare.html

For general info on installation and getting started:

    http://pytest.org/latest/getting-started.html

Docs and PDF access as usual at:

    http://pytest.org

and more details for those already in the knowing of pytest can be found
in the CHANGELOG below.

Particular thanks for this release go to Floris Bruynooghe, Alex Okrushko
Carl Meyer, Ronny Pfannschmidt, Benjamin Peterson and Alex Gaynor for helping 
to get the new features right and well integrated.  Ronny and Floris
also helped to fix a number of bugs and yet more people helped by
providing bug reports.

have fun,
holger krekel


Changes between 2.2.4 and 2.3.0
-----------------------------------

- fix issue202 - better automatic names for parametrized test functions
- fix issue139 - introduce @pytest.fixture which allows direct scoping
  and parametrization of funcarg factories.  Introduce new @pytest.setup
  marker to allow the writing of setup functions which accept funcargs.
- fix issue198 - conftest fixtures were not found on windows32 in some
  circumstances with nested directory structures due to path manipulation issues
- fix issue193 skip test functions with were parametrized with empty
  parameter sets
- fix python3.3 compat, mostly reporting bits that previously depended
  on dict ordering
- introduce re-ordering of tests by resource and parametrization setup
  which takes precedence to the usual file-ordering
- fix issue185 monkeypatching time.time does not cause pytest to fail
- fix issue172 duplicate call of pytest.setup-decoratored setup_module
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

