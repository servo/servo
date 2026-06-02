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
  Thanks :user:`MichaelAquilina` for the complete PR (:pull:`1040`).

* ``--doctest-glob`` may now be passed multiple times in the command-line.
  Thanks :user:`jab` and :user:`nicoddemus` for the PR.

* New ``-rp`` and ``-rP`` reporting options give the summary and full output
  of passing tests, respectively. Thanks to :user:`codewarrior0` for the PR.

* ``pytest.mark.xfail`` now has a ``strict`` option which makes ``XPASS``
  tests to fail the test suite, defaulting to ``False``. There's also a
  ``xfail_strict`` ini option that can be used to configure it project-wise.
  Thanks :user:`rabbbit` for the request and :user:`nicoddemus` for the PR (:issue:`1355`).

* ``Parser.addini`` now supports options of type ``bool``. Thanks
  :user:`nicoddemus` for the PR.

* New ``ALLOW_BYTES`` doctest option strips ``b`` prefixes from byte strings
  in doctest output (similar to ``ALLOW_UNICODE``).
  Thanks :user:`jaraco` for the request and :user:`nicoddemus` for the PR (:issue:`1287`).

* give a hint on KeyboardInterrupt to use the --fulltrace option to show the errors,
  this fixes :issue:`1366`.
  Thanks to :user:`hpk42` for the report and :user:`RonnyPfannschmidt` for the PR.

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
  Thanks :user:`nicoddemus` for the PR.

* Removed code and documentation for Python 2.5 or lower versions,
  including removal of the obsolete ``_pytest.assertion.oldinterpret`` module.
  Thanks :user:`nicoddemus` for the PR (:issue:`1226`).

* Comparisons now always show up in full when ``CI`` or ``BUILD_NUMBER`` is
  found in the environment, even when -vv isn't used.
  Thanks :user:`The-Compiler` for the PR.

* ``--lf`` and ``--ff`` now support long names: ``--last-failed`` and
  ``--failed-first`` respectively.
  Thanks :user:`MichaelAquilina` for the PR.

* Added expected exceptions to pytest.raises fail message

* Collection only displays progress ("collecting X items") when in a terminal.
  This avoids cluttering the output when using ``--color=yes`` to obtain
  colors in CI integrations systems (:issue:`1397`).

**Bug Fixes**

* The ``-s`` and ``-c`` options should now work under ``xdist``;
  ``Config.fromdictargs`` now represents its input much more faithfully.
  Thanks to :user:`bukzor` for the complete PR (:issue:`680`).

* Fix (:issue:`1290`): support Python 3.5's ``@`` operator in assertion rewriting.
  Thanks :user:`Shinkenjoe` for report with test case and :user:`tomviner` for the PR.

* Fix formatting utf-8 explanation messages (:issue:`1379`).
  Thanks :user:`biern` for the PR.

* Fix `traceback style docs`_ to describe all of the available options
  (auto/long/short/line/native/no), with ``auto`` being the default since v2.6.
  Thanks :user:`hackebrot` for the PR.

* Fix (:issue:`1422`): junit record_xml_property doesn't allow multiple records
  with same name.


.. _`traceback style docs`: https://pytest.org/en/stable/how-to/output.html#modifying-python-traceback-printing
