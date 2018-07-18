py.test 2.1.1: assertion fixes and improved junitxml output
===========================================================================

pytest-2.1.1 is a backward compatible maintenance release of the
popular py.test testing tool.  See extensive docs with examples here:

     http://pytest.org/

Most bug fixes address remaining issues with the perfected assertions
introduced with 2.1.0 - many thanks to the bug reporters and to Benjamin
Peterson for helping to fix them.  Also, junitxml output now produces
system-out/err tags which lead to better displays of tracebacks with Jenkins.

Also a quick note to package maintainers and others interested: there now
is a "pytest" man page which can be generated with "make man" in doc/.

If you want to install or upgrade pytest, just type one of::

    pip install -U pytest # or
    easy_install -U pytest

best,
holger krekel / http://merlinux.eu

Changes between 2.1.0 and 2.1.1
----------------------------------------------

- fix issue64 / pytest.set_trace now works within pytest_generate_tests hooks
- fix issue60 / fix error conditions involving the creation of __pycache__
- fix issue63 / assertion rewriting on inserts involving strings containing '%'
- fix assertion rewriting on calls with a ** arg
- don't cache rewritten modules if bytecode generation is disabled
- fix assertion rewriting in read-only directories
- fix issue59: provide system-out/err tags for junitxml output
- fix issue61: assertion rewriting on boolean operations with 3 or more operands
- you can now build a man page with "cd doc ; make man"
