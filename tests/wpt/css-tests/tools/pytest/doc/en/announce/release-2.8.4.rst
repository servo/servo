pytest-2.8.4
============

pytest is a mature Python testing tool with more than a 1100 tests
against itself, passing on many different interpreters and platforms.
This release is supposed to be drop-in compatible to 2.8.2.

See below for the changes and see docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed to this release, among them:

  Bruno Oliveira
  Florian Bruhin
  Jeff Widman
  Mehdy Khoshnoody
  Nicholas Chammas
  Ronny Pfannschmidt
  Tim Chan


Happy testing,
The py.test Development Team


2.8.4 (compared to 2.8.3)
-----------------------------

- fix #1190: ``deprecated_call()`` now works when the deprecated
  function has been already called by another test in the same
  module. Thanks Mikhail Chernykh for the report and Bruno Oliveira for the
  PR.

- fix #1198: ``--pastebin`` option now works on Python 3. Thanks
  Mehdy Khoshnoody for the PR.

- fix #1219: ``--pastebin`` now works correctly when captured output contains
  non-ascii characters. Thanks Bruno Oliveira for the PR.

- fix #1204: another error when collecting with a nasty __getattr__().
  Thanks Florian Bruhin for the PR.

- fix the summary printed when no tests did run.
  Thanks Florian Bruhin for the PR.

- a number of documentation modernizations wrt good practices.
  Thanks Bruno Oliveira for the PR.
