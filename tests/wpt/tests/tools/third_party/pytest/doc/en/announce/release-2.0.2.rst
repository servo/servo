py.test 2.0.2: bug fixes, improved xfail/skip expressions, speed ups
===========================================================================

Welcome to pytest-2.0.2, a maintenance and bug fix release of pytest,
a mature testing tool for Python, supporting CPython 2.4-3.2, Jython
and latest PyPy interpreters.  See the extensive docs with tested examples here:

    http://pytest.org/

If you want to install or upgrade pytest, just type one of::

    pip install -U pytest # or
    easy_install -U pytest

Many thanks to all issue reporters and people asking questions
or complaining, particularly Jurko for his insistence,
Laura, Victor and Brianna for helping with improving
and Ronny for his general advise.

best,
holger krekel

Changes between 2.0.1 and 2.0.2
----------------------------------------------

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
