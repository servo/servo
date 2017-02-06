pytest-2.8.6
============

pytest is a mature Python testing tool with more than a 1100 tests
against itself, passing on many different interpreters and platforms.
This release is supposed to be drop-in compatible to 2.8.5.

See below for the changes and see docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed to this release, among them:

    AMiT Kumar
    Bruno Oliveira
    Erik M. Bray
    Florian Bruhin
    Georgy Dyuldin
    Jeff Widman
    Kartik Singhal
    Loïc Estève
    Manu Phatak
    Peter Demin
    Rick van Hattem
    Ronny Pfannschmidt
    Ulrich Petri
    foxx


Happy testing,
The py.test Development Team


2.8.6 (compared to 2.8.5)
-------------------------

- fix #1259: allow for double nodeids in junitxml,
  this was a regression failing plugins combinations
  like pytest-pep8 + pytest-flakes

- Workaround for exception that occurs in pyreadline when using
  ``--pdb`` with standard I/O capture enabled.
  Thanks Erik M. Bray for the PR.

- fix #900: Better error message in case the target of a ``monkeypatch`` call
  raises an ``ImportError``.

- fix #1292: monkeypatch calls (setattr, setenv, etc.) are now O(1).
  Thanks David R. MacIver for the report and Bruno Oliveira for the PR.

- fix #1223: captured stdout and stderr are now properly displayed before
  entering pdb when ``--pdb`` is used instead of being thrown away.
  Thanks Cal Leeming for the PR.

- fix #1305: pytest warnings emitted during ``pytest_terminal_summary`` are now
  properly displayed.
  Thanks Ionel Maries Cristian for the report and Bruno Oliveira for the PR.

- fix #628: fixed internal UnicodeDecodeError when doctests contain unicode.
  Thanks Jason R. Coombs for the report and Bruno Oliveira for the PR.

- fix #1334: Add captured stdout to jUnit XML report on setup error.
  Thanks Georgy Dyuldin for the PR.
