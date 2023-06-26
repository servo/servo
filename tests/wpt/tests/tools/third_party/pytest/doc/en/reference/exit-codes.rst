.. _exit-codes:

Exit codes
========================================================

Running ``pytest`` can result in six different exit codes:

:Exit code 0: All tests were collected and passed successfully
:Exit code 1: Tests were collected and run but some of the tests failed
:Exit code 2: Test execution was interrupted by the user
:Exit code 3: Internal error happened while executing tests
:Exit code 4: pytest command line usage error
:Exit code 5: No tests were collected

They are represented by the :class:`pytest.ExitCode` enum. The exit codes being a part of the public API can be imported and accessed directly using:

.. code-block:: python

    from pytest import ExitCode

.. note::

    If you would like to customize the exit code in some scenarios, specially when
    no tests are collected, consider using the
    `pytest-custom_exit_code <https://github.com/yashtodi94/pytest-custom_exit_code>`__
    plugin.
