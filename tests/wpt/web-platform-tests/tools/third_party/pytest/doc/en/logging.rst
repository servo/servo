.. _logging:

Logging
-------

.. versionadded 3.3.0

.. note::

   This feature is a drop-in replacement for the `pytest-catchlog
   <https://pypi.org/project/pytest-catchlog/>`_ plugin and they will conflict
   with each other. The backward compatibility API with ``pytest-capturelog``
   has been dropped when this feature was introduced, so if for that reason you
   still need ``pytest-catchlog`` you can disable the internal feature by
   adding to your ``pytest.ini``:

   .. code-block:: ini

       [pytest]
           addopts=-p no:logging

Log messages are captured by default and for each failed test will be shown in
the same manner as captured stdout and stderr.

Running without options::

    pytest

Shows failed tests like so::

    ----------------------- Captured stdlog call ----------------------
    test_reporting.py    26 INFO     text going to logger
    ----------------------- Captured stdout call ----------------------
    text going to stdout
    ----------------------- Captured stderr call ----------------------
    text going to stderr
    ==================== 2 failed in 0.02 seconds =====================

By default each captured log message shows the module, line number, log level
and message.  Showing the exact module and line number is useful for testing and
debugging.  If desired the log format and date format can be specified to
anything that the logging module supports.

Running pytest specifying formatting options::

    pytest --log-format="%(asctime)s %(levelname)s %(message)s" \
            --log-date-format="%Y-%m-%d %H:%M:%S"

Shows failed tests like so::

    ----------------------- Captured stdlog call ----------------------
    2010-04-10 14:48:44 INFO text going to logger
    ----------------------- Captured stdout call ----------------------
    text going to stdout
    ----------------------- Captured stderr call ----------------------
    text going to stderr
    ==================== 2 failed in 0.02 seconds =====================

These options can also be customized through a configuration file:

.. code-block:: ini

    [pytest]
    log_format = %(asctime)s %(levelname)s %(message)s
    log_date_format = %Y-%m-%d %H:%M:%S

Further it is possible to disable reporting logs on failed tests completely
with::

    pytest --no-print-logs

Or in you ``pytest.ini``:

.. code-block:: ini

  [pytest]
  log_print = False


Shows failed tests in the normal manner as no logs were captured::

    ----------------------- Captured stdout call ----------------------
    text going to stdout
    ----------------------- Captured stderr call ----------------------
    text going to stderr
    ==================== 2 failed in 0.02 seconds =====================

Inside tests it is possible to change the log level for the captured log
messages.  This is supported by the ``caplog`` fixture::

    def test_foo(caplog):
        caplog.set_level(logging.INFO)
        pass

By default the level is set on the handler used to catch the log messages,
however as a convenience it is also possible to set the log level of any
logger::

    def test_foo(caplog):
        caplog.set_level(logging.CRITICAL, logger='root.baz')
        pass

It is also possible to use a context manager to temporarily change the log
level::

    def test_bar(caplog):
        with caplog.at_level(logging.INFO):
            pass

Again, by default the level of the handler is affected but the level of any
logger can be changed instead with::

    def test_bar(caplog):
        with caplog.at_level(logging.CRITICAL, logger='root.baz'):
            pass

Lastly all the logs sent to the logger during the test run are made available on
the fixture in the form of both the LogRecord instances and the final log text.
This is useful for when you want to assert on the contents of a message::

    def test_baz(caplog):
        func_under_test()
        for record in caplog.records:
            assert record.levelname != 'CRITICAL'
        assert 'wally' not in caplog.text

For all the available attributes of the log records see the
``logging.LogRecord`` class.

You can also resort to ``record_tuples`` if all you want to do is to ensure,
that certain messages have been logged under a given logger name with a given
severity and message::

    def test_foo(caplog):
        logging.getLogger().info('boo %s', 'arg')

        assert caplog.record_tuples == [
            ('root', logging.INFO, 'boo arg'),
        ]

You can call ``caplog.clear()`` to reset the captured log records in a test::

    def test_something_with_clearing_records(caplog):
        some_method_that_creates_log_records()
        caplog.clear()
        your_test_method()
        assert ['Foo'] == [rec.message for rec in caplog.records]

Live Logs
^^^^^^^^^

By default, pytest will output any logging records with a level higher or
equal to WARNING. In order to actually see these logs in the console you have to
disable pytest output capture by passing ``-s``.

You can specify the logging level for which log records with equal or higher
level are printed to the console by passing ``--log-cli-level``. This setting
accepts the logging level names as seen in python's documentation or an integer
as the logging level num.

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
``--log-file=/path/to/log/file``. This log file is opened in write mode which
means that it will be overwritten at each run tests session.

You can also specify the logging level for the log file by passing
``--log-file-level``. This setting accepts the logging level names as seen in
python's documentation(ie, uppercased level names) or an integer as the logging
level num.

Additionally, you can also specify ``--log-file-format`` and
``--log-file-date-format`` which are equal to ``--log-format`` and
``--log-date-format`` but are applied to the log file logging handler.

All of the log file options can also be set in the configuration INI file. The
option names are:

* ``log_file``
* ``log_file_level``
* ``log_file_format``
* ``log_file_date_format``
