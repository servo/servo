pytest-2.8.3: bug fixes
=======================

pytest is a mature Python testing tool with more than 1100 tests
against itself, passing on many different interpreters and platforms.
This release is supposed to be drop-in compatible to 2.8.2.

See below for the changes and see docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed to this release, among them:

        Bruno Oliveira
        Florian Bruhin
        Gabe Hollombe
        Gabriel Reis
        Hartmut Goebel
        John Vandenberg
        Lee Kamentsky
        Michael Birtwell
        Raphael Pierzina
        Ronny Pfannschmidt
        William Martin Stewart

Happy testing,
The py.test Development Team


2.8.3 (compared to 2.8.2)
-----------------------------

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
- minor fixes for interaction with OS X El Capitan system integrity protection (thanks Florian)
