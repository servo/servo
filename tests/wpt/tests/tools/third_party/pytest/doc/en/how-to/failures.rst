.. _how-to-handle-failures:

How to handle test failures
=============================

.. _maxfail:

Stopping after the first (or N) failures
---------------------------------------------------

To stop the testing process after the first (N) failures:

.. code-block:: bash

    pytest -x           # stop after first failure
    pytest --maxfail=2  # stop after two failures


.. _pdb-option:

Using :doc:`python:library/pdb` with pytest
-------------------------------------------

Dropping to :doc:`pdb <python:library/pdb>` on failures
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Python comes with a builtin Python debugger called :doc:`pdb <python:library/pdb>`.  ``pytest``
allows one to drop into the :doc:`pdb <python:library/pdb>` prompt via a command line option:

.. code-block:: bash

    pytest --pdb

This will invoke the Python debugger on every failure (or KeyboardInterrupt).
Often you might only want to do this for the first failing test to understand
a certain failure situation:

.. code-block:: bash

    pytest -x --pdb   # drop to PDB on first failure, then end test session
    pytest --pdb --maxfail=3  # drop to PDB for first three failures

Note that on any failure the exception information is stored on
``sys.last_value``, ``sys.last_type`` and ``sys.last_traceback``. In
interactive use, this allows one to drop into postmortem debugging with
any debug tool. One can also manually access the exception information,
for example::

    >>> import sys
    >>> sys.last_traceback.tb_lineno
    42
    >>> sys.last_value
    AssertionError('assert result == "ok"',)


.. _trace-option:

Dropping to :doc:`pdb <python:library/pdb>` at the start of a test
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``pytest`` allows one to drop into the :doc:`pdb <python:library/pdb>` prompt immediately at the start of each test via a command line option:

.. code-block:: bash

    pytest --trace

This will invoke the Python debugger at the start of every test.

.. _breakpoints:

Setting breakpoints
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. versionadded: 2.4.0

To set a breakpoint in your code use the native Python ``import pdb;pdb.set_trace()`` call
in your code and pytest automatically disables its output capture for that test:

* Output capture in other tests is not affected.
* Any prior test output that has already been captured and will be processed as
  such.
* Output capture gets resumed when ending the debugger session (via the
  ``continue`` command).


.. _`breakpoint-builtin`:

Using the builtin breakpoint function
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Python 3.7 introduces a builtin ``breakpoint()`` function.
Pytest supports the use of ``breakpoint()`` with the following behaviours:

 - When ``breakpoint()`` is called and ``PYTHONBREAKPOINT`` is set to the default value, pytest will use the custom internal PDB trace UI instead of the system default ``Pdb``.
 - When tests are complete, the system will default back to the system ``Pdb`` trace UI.
 - With ``--pdb`` passed to pytest, the custom internal Pdb trace UI is used with both ``breakpoint()`` and failed tests/unhandled exceptions.
 - ``--pdbcls`` can be used to specify a custom debugger class.


.. _faulthandler:

Fault Handler
-------------

.. versionadded:: 5.0

The :mod:`faulthandler` standard module
can be used to dump Python tracebacks on a segfault or after a timeout.

The module is automatically enabled for pytest runs, unless the ``-p no:faulthandler`` is given
on the command-line.

Also the :confval:`faulthandler_timeout=X<faulthandler_timeout>` configuration option can be used
to dump the traceback of all threads if a test takes longer than ``X``
seconds to finish (not available on Windows).

.. note::

    This functionality has been integrated from the external
    `pytest-faulthandler <https://github.com/pytest-dev/pytest-faulthandler>`__ plugin, with two
    small differences:

    * To disable it, use ``-p no:faulthandler`` instead of ``--no-faulthandler``: the former
      can be used with any plugin, so it saves one option.

    * The ``--faulthandler-timeout`` command-line option has become the
      :confval:`faulthandler_timeout` configuration option. It can still be configured from
      the command-line using ``-o faulthandler_timeout=X``.


.. _unraisable:

Warning about unraisable exceptions and unhandled thread exceptions
-------------------------------------------------------------------

.. versionadded:: 6.2

Unhandled exceptions are exceptions that are raised in a situation in which
they cannot propagate to a caller. The most common case is an exception raised
in a :meth:`__del__ <object.__del__>` implementation.

Unhandled thread exceptions are exceptions raised in a :class:`~threading.Thread`
but not handled, causing the thread to terminate uncleanly.

Both types of exceptions are normally considered bugs, but may go unnoticed
because they don't cause the program itself to crash. Pytest detects these
conditions and issues a warning that is visible in the test run summary.

The plugins are automatically enabled for pytest runs, unless the
``-p no:unraisableexception`` (for unraisable exceptions) and
``-p no:threadexception`` (for thread exceptions) options are given on the
command-line.

The warnings may be silenced selectively using the :ref:`pytest.mark.filterwarnings ref`
mark. The warning categories are :class:`pytest.PytestUnraisableExceptionWarning` and
:class:`pytest.PytestUnhandledThreadExceptionWarning`.
