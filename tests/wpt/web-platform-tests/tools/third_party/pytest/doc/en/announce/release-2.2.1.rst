pytest-2.2.1: bug fixes, perfect teardowns
===========================================================================


pytest-2.2.1 is a minor backward-compatible release of the py.test
testing tool.   It contains bug fixes and little improvements, including
documentation fixes.  If you are using the distributed testing
pluginmake sure to upgrade it to pytest-xdist-1.8.

For general information see here:

     http://pytest.org/

To install or upgrade pytest:

    pip install -U pytest # or
    easy_install -U pytest

Special thanks for helping on this release to Ronny Pfannschmidt, Jurko
Gospodnetic and Ralf Schmitt.

best,
holger krekel


Changes between 2.2.0 and 2.2.1
----------------------------------------

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
