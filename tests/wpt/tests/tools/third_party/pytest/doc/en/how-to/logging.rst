.. _logging:

How to manage logging
---------------------

pytest captures log messages of level ``WARNING`` or above automatically and displays them in their own section
for each failed test in the same manner as captured stdout and stderr.

Running without options:

.. code-block:: bash

    pytest

Shows failed tests like so:

.. code-block:: pytest

    ----------------------- Captured stdlog call ----------------------
    test_reporting.py    26 WARNING  text going to logger
    ----------------------- Captured stdout call ----------------------
    text going to stdout
    ----------------------- Captured stderr call ----------------------
    text going to stderr
    ==================== 2 failed in 0.02 seconds =====================

By default each captured log message shows the module, line number, log level
and message.

If desired the log and date format can be specified to
anything that the logging module supports by passing specific formatting options:

.. code-block:: bash

    pytest --log-format="%(asctime)s %(levelname)s %(message)s" \
            --log-date-format="%Y-%m-%d %H:%M:%S"

Shows failed tests like so:

.. code-block:: pytest

    ----------------------- Captured stdlog call ----------------------
    2010-04-10 14:48:44 WARNING text going to logger
    ----------------------- Captured stdout call ----------------------
    text going to stdout
    ----------------------- Captured stderr call ----------------------
    text going to stderr
    ==================== 2 failed in 0.02 seconds =====================

These options can also be customized through ``pytest.ini`` file:

.. code-block:: ini

    [pytest]
    log_format = %(asctime)s %(levelname)s %(message)s
    log_date_format = %Y-%m-%d %H:%M:%S

Specific loggers can be disabled via ``--log-disable={logger_name}``.
This argument can be passed multiple times:

.. code-block:: bash

    pytest --log-disable=main --log-disable=testing

Further it is possible to disable reporting of captured content (stdout,
stderr and logs) on failed tests completely with:

.. code-block:: bash

    pytest --show-capture=no


caplog fixture
^^^^^^^^^^^^^^

Inside tests it is possible to change the log level for the captured log
messages.  This is supported by the ``caplog`` fixture:

.. code-block:: python

    def test_foo(caplog):
        caplog.set_level(logging.INFO)

By default the level is set on the root logger,
however as a convenience it is also possible to set the log level of any
logger:

.. code-block:: python

    def test_foo(caplog):
        caplog.set_level(logging.CRITICAL, logger="root.baz")

The log levels set are restored automatically at the end of the test.

It is also possible to use a context manager to temporarily change the log
level inside a ``with`` block:

.. code-block:: python

    def test_bar(caplog):
        with caplog.at_level(logging.INFO):
            pass

Again, by default the level of the root logger is affected but the level of any
logger can be changed instead with:

.. code-block:: python

    def test_bar(caplog):
        with caplog.at_level(logging.CRITICAL, logger="root.baz"):
            pass

Lastly all the logs sent to the logger during the test run are made available on
the fixture in the form of both the ``logging.LogRecord`` instances and the final log text.
This is useful for when you want to assert on the contents of a message:

.. code-block:: python

    def test_baz(caplog):
        func_under_test()
        for record in caplog.records:
            assert record.levelname != "CRITICAL"
        assert "wally" not in caplog.text

For all the available attributes of the log records see the
``logging.LogRecord`` class.

You can also resort to ``record_tuples`` if all you want to do is to ensure,
that certain messages have been logged under a given logger name with a given
severity and message:

.. code-block:: python

    def test_foo(caplog):
        logging.getLogger().info("boo %s", "arg")

        assert caplog.record_tuples == [("root", logging.INFO, "boo arg")]

You can call ``caplog.clear()`` to reset the captured log records in a test:

.. code-block:: python

    def test_something_with_clearing_records(caplog):
        some_method_that_creates_log_records()
        caplog.clear()
        your_test_method()
        assert ["Foo"] == [rec.message for rec in caplog.records]


The ``caplog.records`` attribute contains records from the current stage only, so
inside the ``setup`` phase it contains only setup logs, same with the ``call`` and
``teardown`` phases.

To access logs from other stages, use the ``caplog.get_records(when)`` method. As an example,
if you want to make sure that tests which use a certain fixture never log any warnings, you can inspect
the records for the ``setup`` and ``call`` stages during teardown like so:

.. code-block:: python

    @pytest.fixture
    def window(caplog):
        window = create_window()
        yield window
        for when in ("setup", "call"):
            messages = [
                x.message for x in caplog.get_records(when) if x.levelno == logging.WARNING
            ]
            if messages:
                pytest.fail(f"warning messages encountered during testing: {messages}")



The full API is available at :class:`pytest.LogCaptureFixture`.

.. warning::

    The ``caplog`` fixture adds a handler to the root logger to capture logs. If the root logger is
    modified during a test, for example with ``logging.config.dictConfig``, this handler may be
    removed and cause no logs to be captured. To avoid this, ensure that any root logger configuration
    only adds to the existing handlers.


.. _live_logs:

Live Logs
^^^^^^^^^

By setting the :confval:`log_cli` configuration option to ``true``, pytest will output
logging records as they are emitted directly into the console.

You can specify the logging level for which log records with equal or higher
level are printed to the console by passing ``--log-cli-level``. This setting
accepts the logging level names or numeric values as seen in
:ref:`logging's documentation <python:levels>`.

Additionally, you can also specify ``--log-cli-format`` and
``--log-cli-date-format`` which mirror and default to ``--log-format`` and
``--log-date-format`` if not provided, but are applied only to the console
logging handler.

All of the CLI log options can also be set in the configuration INI file. The
option names are:

* ``log_cli_level``
* ``log_cli_format``
* ``log_cli_date_format``

If you need to record the whole test suite logging calls to a file, you can pass
``--log-file=/path/to/log/file``. This log file is opened in write mode by default which
means that it will be overwritten at each run tests session.
If you'd like the file opened in append mode instead, then you can pass ``--log-file-mode=a``.
Note that relative paths for the log-file location, whether passed on the CLI or declared in a
config file, are always resolved relative to the current working directory.

You can also specify the logging level for the log file by passing
``--log-file-level``. This setting accepts the logging level names or numeric
values as seen in :ref:`logging's documentation <python:levels>`.

Additionally, you can also specify ``--log-file-format`` and
``--log-file-date-format`` which are equal to ``--log-format`` and
``--log-date-format`` but are applied to the log file logging handler.

All of the log file options can also be set in the configuration INI file. The
option names are:

* ``log_file``
* ``log_file_mode``
* ``log_file_level``
* ``log_file_format``
* ``log_file_date_format``

You can call ``set_log_path()`` to customize the log_file path dynamically. This functionality
is considered **experimental**. Note that ``set_log_path()`` respects the ``log_file_mode`` option.

.. _log_colors:

Customizing Colors
^^^^^^^^^^^^^^^^^^

Log levels are colored if colored terminal output is enabled. Changing
from default colors or putting color on custom log levels is supported
through ``add_color_level()``. Example:

.. code-block:: python

    @pytest.hookimpl(trylast=True)
    def pytest_configure(config):
        logging_plugin = config.pluginmanager.get_plugin("logging-plugin")

        # Change color on existing log level
        logging_plugin.log_cli_handler.formatter.add_color_level(logging.INFO, "cyan")

        # Add color to a custom log level (a custom log level `SPAM` is already set up)
        logging_plugin.log_cli_handler.formatter.add_color_level(logging.SPAM, "blue")
.. warning::

    This feature and its API are considered **experimental** and might change
    between releases without a deprecation notice.
.. _log_release_notes:

Release notes
^^^^^^^^^^^^^

This feature was introduced as a drop-in replacement for the
:pypi:`pytest-catchlog` plugin and they conflict
with each other. The backward compatibility API with ``pytest-capturelog``
has been dropped when this feature was introduced, so if for that reason you
still need ``pytest-catchlog`` you can disable the internal feature by
adding to your ``pytest.ini``:

.. code-block:: ini

   [pytest]
       addopts=-p no:logging


.. _log_changes_3_4:

Incompatible changes in pytest 3.4
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

This feature was introduced in ``3.3`` and some **incompatible changes** have been
made in ``3.4`` after community feedback:

* Log levels are no longer changed unless explicitly requested by the :confval:`log_level` configuration
  or ``--log-level`` command-line options. This allows users to configure logger objects themselves.
  Setting :confval:`log_level` will set the level that is captured globally so if a specific test requires
  a lower level than this, use the ``caplog.set_level()`` functionality otherwise that test will be prone to
  failure.
* :ref:`Live Logs <live_logs>` is now disabled by default and can be enabled setting the
  :confval:`log_cli` configuration option to ``true``. When enabled, the verbosity is increased so logging for each
  test is visible.
* :ref:`Live Logs <live_logs>` are now sent to ``sys.stdout`` and no longer require the ``-s`` command-line option
  to work.

If you want to partially restore the logging behavior of version ``3.3``, you can add this options to your ``ini``
file:

.. code-block:: ini

    [pytest]
    log_cli=true
    log_level=NOTSET

More details about the discussion that lead to this changes can be read in :issue:`3013`.
