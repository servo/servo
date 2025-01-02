pytest-2.9.2
============

pytest is a mature Python testing tool with more than 1100 tests
against itself, passing on many different interpreters and platforms.

See below for the changes and see docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed to this release, among them:

      Adam Chainz
      Benjamin Dopplinger
      Bruno Oliveira
      Florian Bruhin
      John Towler
      Martin Prusse
      Meng Jue
      MengJueM
      Omar Kohl
      Quentin Pradet
      Ronny Pfannschmidt
      Thomas Güttler
      TomV
      Tyler Goodlet


Happy testing,
The py.test Development Team


2.9.2 (compared to 2.9.1)
---------------------------

**Bug Fixes**

* fix :issue:`510`: skip tests where one parameterize dimension was empty
  thanks Alex Stapleton for the Report and :user:`RonnyPfannschmidt` for the PR

* Fix Xfail does not work with condition keyword argument.
  Thanks :user:`astraw38` for reporting the issue (:issue:`1496`) and :user:`tomviner`
  for PR the (:pull:`1524`).

* Fix win32 path issue when putting custom config file with absolute path
  in ``pytest.main("-c your_absolute_path")``.

* Fix maximum recursion depth detection when raised error class is not aware
  of unicode/encoded bytes.
  Thanks :user:`prusse-martin` for the PR (:pull:`1506`).

* Fix ``pytest.mark.skip`` mark when used in strict mode.
  Thanks :user:`pquentin` for the PR and :user:`RonnyPfannschmidt` for
  showing how to fix the bug.

* Minor improvements and fixes to the documentation.
  Thanks :user:`omarkohl` for the PR.

* Fix ``--fixtures`` to show all fixture definitions as opposed to just
  one per fixture name.
  Thanks to :user:`hackebrot` for the PR.
