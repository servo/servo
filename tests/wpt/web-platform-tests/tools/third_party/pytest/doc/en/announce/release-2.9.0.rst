pytest-2.9.0
============

pytest is a mature Python testing tool with more than 1100 tests
against itself, passing on many different interpreters and platforms.

See below for the changes and see docs at:

    http://pytest.org

As usual, you can upgrade from pypi via::

    pip install -U pytest

Thanks to all who contributed to this release, among them:

    Anatoly Bubenkov
    Bruno Oliveira
    Buck Golemon
    David Vierra
    Florian Bruhin
    Galaczi Endre
    Georgy Dyuldin
    Lukas Bednar
    Luke Murphy
    Marcin Biernat
    Matt Williams
    Michael Aquilina
    Raphael Pierzina
    Ronny Pfannschmidt
    Ryan Wooden
    Tiemo Kieft
    TomV
    holger krekel
    jab


Happy testing,
The py.test Development Team


2.9.0 (compared to 2.8.7)
-------------------------

**New Features**

* New ``pytest.mark.skip`` mark, which unconditionally skips marked tests.
  Thanks `@MichaelAquilina`_ for the complete PR (`#1040`_).

* ``--doctest-glob`` may now be passed multiple times in the command-line.
  Thanks `@jab`_ and `@nicoddemus`_ for the PR.

* New ``-rp`` and ``-rP`` reporting options give the summary and full output
  of passing tests, respectively. Thanks to `@codewarrior0`_ for the PR.

* ``pytest.mark.xfail`` now has a ``strict`` option which makes ``XPASS``
  tests to fail the test suite, defaulting to ``False``. There's also a
  ``xfail_strict`` ini option that can be used to configure it project-wise.
  Thanks `@rabbbit`_ for the request and `@nicoddemus`_ for the PR (`#1355`_).

* ``Parser.addini`` now supports options of type ``bool``. Thanks
  `@nicoddemus`_ for the PR.

* New ``ALLOW_BYTES`` doctest option strips ``b`` prefixes from byte strings
  in doctest output (similar to ``ALLOW_UNICODE``).
  Thanks `@jaraco`_ for the request and `@nicoddemus`_ for the PR (`#1287`_).

* give a hint on KeyboardInterrupt to use the --fulltrace option to show the errors,
  this fixes `#1366`_.
  Thanks to `@hpk42`_ for the report and `@RonnyPfannschmidt`_ for the PR.

* catch IndexError exceptions when getting exception source location. This fixes
  pytest internal error for dynamically generated code (fixtures and tests)
  where source lines are fake by intention

**Changes**

* **Important**: `py.code <https://pylib.readthedocs.io/en/stable/code.html>`_ has been
  merged into the ``pytest`` repository as ``pytest._code``. This decision
  was made because ``py.code`` had very few uses outside ``pytest`` and the
  fact that it was in a different repository made it difficult to fix bugs on
  its code in a timely manner. The team hopes with this to be able to better
  refactor out and improve that code.
  This change shouldn't affect users, but it is useful to let users aware
  if they encounter any strange behavior.

  Keep in mind that the code for ``pytest._code`` is **private** and
  **experimental**, so you definitely should not import it explicitly!

  Please note that the original ``py.code`` is still available in
  `pylib <https://pylib.readthedocs.io/en/stable/>`_.

* ``pytest_enter_pdb`` now optionally receives the pytest config object.
  Thanks `@nicoddemus`_ for the PR.

* Removed code and documentation for Python 2.5 or lower versions,
  including removal of the obsolete ``_pytest.assertion.oldinterpret`` module.
  Thanks `@nicoddemus`_ for the PR (`#1226`_).

* Comparisons now always show up in full when ``CI`` or ``BUILD_NUMBER`` is
  found in the environment, even when -vv isn't used.
  Thanks `@The-Compiler`_ for the PR.

* ``--lf`` and ``--ff`` now support long names: ``--last-failed`` and
  ``--failed-first`` respectively.
  Thanks `@MichaelAquilina`_ for the PR.

* Added expected exceptions to pytest.raises fail message

* Collection only displays progress ("collecting X items") when in a terminal.
  This avoids cluttering the output when using ``--color=yes`` to obtain
  colors in CI integrations systems (`#1397`_).

**Bug Fixes**

* The ``-s`` and ``-c`` options should now work under ``xdist``;
  ``Config.fromdictargs`` now represents its input much more faithfully.
  Thanks to `@bukzor`_ for the complete PR (`#680`_).

* Fix (`#1290`_): support Python 3.5's ``@`` operator in assertion rewriting.
  Thanks `@Shinkenjoe`_ for report with test case and `@tomviner`_ for the PR.

* Fix formatting utf-8 explanation messages (`#1379`_).
  Thanks `@biern`_ for the PR.

* Fix `traceback style docs`_ to describe all of the available options
  (auto/long/short/line/native/no), with ``auto`` being the default since v2.6.
  Thanks `@hackebrot`_ for the PR.

* Fix (`#1422`_): junit record_xml_property doesn't allow multiple records
  with same name.


.. _`traceback style docs`: https://pytest.org/en/stable/usage.html#modifying-python-traceback-printing

.. _#1422: https://github.com/pytest-dev/pytest/issues/1422
.. _#1379: https://github.com/pytest-dev/pytest/issues/1379
.. _#1366: https://github.com/pytest-dev/pytest/issues/1366
.. _#1040: https://github.com/pytest-dev/pytest/pull/1040
.. _#680: https://github.com/pytest-dev/pytest/issues/680
.. _#1287: https://github.com/pytest-dev/pytest/pull/1287
.. _#1226: https://github.com/pytest-dev/pytest/pull/1226
.. _#1290: https://github.com/pytest-dev/pytest/pull/1290
.. _#1355: https://github.com/pytest-dev/pytest/pull/1355
.. _#1397: https://github.com/pytest-dev/pytest/issues/1397
.. _@biern: https://github.com/biern
.. _@MichaelAquilina: https://github.com/MichaelAquilina
.. _@bukzor: https://github.com/bukzor
.. _@hpk42: https://github.com/hpk42
.. _@nicoddemus: https://github.com/nicoddemus
.. _@jab: https://github.com/jab
.. _@codewarrior0: https://github.com/codewarrior0
.. _@jaraco: https://github.com/jaraco
.. _@The-Compiler: https://github.com/The-Compiler
.. _@Shinkenjoe: https://github.com/Shinkenjoe
.. _@tomviner: https://github.com/tomviner
.. _@RonnyPfannschmidt: https://github.com/RonnyPfannschmidt
.. _@rabbbit: https://github.com/rabbbit
.. _@hackebrot: https://github.com/hackebrot
