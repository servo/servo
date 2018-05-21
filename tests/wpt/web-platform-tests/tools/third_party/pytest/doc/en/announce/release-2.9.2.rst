pytest-2.9.2
============

pytest is a mature Python testing tool with more than a 1100 tests
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

* fix `#510`_: skip tests where one parameterize dimension was empty
  thanks Alex Stapleton for the Report and `@RonnyPfannschmidt`_ for the PR

* Fix Xfail does not work with condition keyword argument.
  Thanks `@astraw38`_ for reporting the issue (`#1496`_) and `@tomviner`_
  for PR the (`#1524`_).

* Fix win32 path issue when putting custom config file with absolute path
  in ``pytest.main("-c your_absolute_path")``.

* Fix maximum recursion depth detection when raised error class is not aware
  of unicode/encoded bytes.
  Thanks `@prusse-martin`_ for the PR (`#1506`_).

* Fix ``pytest.mark.skip`` mark when used in strict mode.
  Thanks `@pquentin`_ for the PR and `@RonnyPfannschmidt`_ for
  showing how to fix the bug.

* Minor improvements and fixes to the documentation.
  Thanks `@omarkohl`_ for the PR.

* Fix ``--fixtures`` to show all fixture definitions as opposed to just
  one per fixture name.
  Thanks to `@hackebrot`_ for the PR.

.. _#510: https://github.com/pytest-dev/pytest/issues/510
.. _#1506: https://github.com/pytest-dev/pytest/pull/1506
.. _#1496: https://github.com/pytest-dev/pytest/issue/1496
.. _#1524: https://github.com/pytest-dev/pytest/issue/1524

.. _@astraw38: https://github.com/astraw38
.. _@hackebrot: https://github.com/hackebrot
.. _@omarkohl: https://github.com/omarkohl
.. _@pquentin: https://github.com/pquentin
.. _@prusse-martin: https://github.com/prusse-martin
.. _@RonnyPfannschmidt: https://github.com/RonnyPfannschmidt
.. _@tomviner: https://github.com/tomviner
