py.test 2.1.0: perfected assertions and bug fixes
===========================================================================

Welcome to the release of pytest-2.1, a mature testing tool for Python,
supporting CPython 2.4-3.2, Jython and latest PyPy interpreters.  See
the improved extensive docs (now also as PDF!) with tested examples here:

     http://pytest.org/

The single biggest news about this release are **perfected assertions**
courtesy of Benjamin Peterson.  You can now safely use ``assert``
statements in test modules without having to worry about side effects
or python optimization ("-OO") options.  This is achieved by rewriting
assert statements in test modules upon import, using a PEP302 hook.
See https://docs.pytest.org/en/stable/how-to/assert.html for
detailed information.  The work has been partly sponsored by my company,
merlinux GmbH.

For further details on bug fixes and smaller enhancements see below.

If you want to install or upgrade pytest, just type one of::

    pip install -U pytest # or
    easy_install -U pytest

best,
holger krekel / https://merlinux.eu/

Changes between 2.0.3 and 2.1.0
----------------------------------------------

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
- fix issue49: avoid confusing error when initialization partially fails
- fix issue44: env/username expansion for junitxml file path
- show releaselevel information in test runs for pypy
- reworked doc pages for better navigation and PDF generation
- report KeyboardInterrupt even if interrupted during session startup
- fix issue 35 - provide PDF doc version and download link from index page
