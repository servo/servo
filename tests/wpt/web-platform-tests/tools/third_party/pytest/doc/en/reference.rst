Reference
=========

This page contains the full reference to pytest's API.

.. contents::
    :depth: 3
    :local:

Functions
---------

pytest.approx
~~~~~~~~~~~~~

.. autofunction:: _pytest.python_api.approx

pytest.fail
~~~~~~~~~~~

**Tutorial**: :ref:`skipping`

.. autofunction:: _pytest.outcomes.fail

pytest.skip
~~~~~~~~~~~

.. autofunction:: _pytest.outcomes.skip(msg, [allow_module_level=False])

pytest.importorskip
~~~~~~~~~~~~~~~~~~~

.. autofunction:: _pytest.outcomes.importorskip

pytest.xfail
~~~~~~~~~~~~

.. autofunction:: _pytest.outcomes.xfail

pytest.exit
~~~~~~~~~~~

.. autofunction:: _pytest.outcomes.exit

pytest.main
~~~~~~~~~~~

.. autofunction:: _pytest.config.main

pytest.param
~~~~~~~~~~~~

.. autofunction:: pytest.param(*values, [id], [marks])

pytest.raises
~~~~~~~~~~~~~

**Tutorial**: :ref:`assertraises`.

.. autofunction:: pytest.raises(expected_exception: Exception, [match], [message])
    :with: excinfo

pytest.deprecated_call
~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`ensuring_function_triggers`.

.. autofunction:: pytest.deprecated_call()
    :with:

pytest.register_assert_rewrite
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`assertion-rewriting`.

.. autofunction:: pytest.register_assert_rewrite

pytest.warns
~~~~~~~~~~~~

**Tutorial**: :ref:`assertwarnings`

.. autofunction:: pytest.warns(expected_warning: Exception, [match])
    :with:

pytest.freeze_includes
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`freezing-pytest`.

.. autofunction:: pytest.freeze_includes

.. _`marks ref`:

Marks
-----

Marks can be used apply meta data to *test functions* (but not fixtures), which can then be accessed by
fixtures or plugins.




.. _`pytest.mark.filterwarnings ref`:

pytest.mark.filterwarnings
~~~~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`filterwarnings`.

Add warning filters to marked test items.

.. py:function:: pytest.mark.filterwarnings(filter)

    :keyword str filter:
        A *warning specification string*, which is composed of contents of the tuple ``(action, message, category, module, lineno)``
        as specified in `The Warnings filter <https://docs.python.org/3/library/warnings.html#warning-filter>`_ section of
        the Python documentation, separated by ``":"``. Optional fields can be omitted.
        Module names passed for filtering are not regex-escaped.

        For example:

        .. code-block:: python

            @pytest.mark.warnings("ignore:.*usage will be deprecated.*:DeprecationWarning")
            def test_foo():
                ...


.. _`pytest.mark.parametrize ref`:

pytest.mark.parametrize
~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :doc:`parametrize`.

.. automethod:: _pytest.python.Metafunc.parametrize


.. _`pytest.mark.skip ref`:

pytest.mark.skip
~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`skip`.

Unconditionally skip a test function.

.. py:function:: pytest.mark.skip(*, reason=None)

    :keyword str reason: Reason why the test function is being skipped.


.. _`pytest.mark.skipif ref`:

pytest.mark.skipif
~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`skipif`.

Skip a test function if a condition is ``True``.

.. py:function:: pytest.mark.skipif(condition, *, reason=None)

    :type condition: bool or str
    :param condition: ``True/False`` if the condition should be skipped or a :ref:`condition string <string conditions>`.
    :keyword str reason: Reason why the test function is being skipped.


.. _`pytest.mark.usefixtures ref`:

pytest.mark.usefixtures
~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`usefixtures`.

Mark a test function as using the given fixture names.

.. warning::

    This mark has no effect when applied
    to a **fixture** function.

.. py:function:: pytest.mark.usefixtures(*names)

    :param args: the names of the fixture to use, as strings


.. _`pytest.mark.xfail ref`:

pytest.mark.xfail
~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`xfail`.

Marks a test function as *expected to fail*.

.. py:function:: pytest.mark.xfail(condition=None, *, reason=None, raises=None, run=True, strict=False)

    :type condition: bool or str
    :param condition:
        Condition for marking the test function as xfail (``True/False`` or a
        :ref:`condition string <string conditions>`).
    :keyword str reason: Reason why the test function is marked as xfail.
    :keyword Exception raises: Exception subclass expected to be raised by the test function; other exceptions will fail the test.
    :keyword bool run:
        If the test function should actually be executed. If ``False``, the function will always xfail and will
        not be executed (useful if a function is segfaulting).
    :keyword bool strict:
        * If ``False`` (the default) the function will be shown in the terminal output as ``xfailed`` if it fails
          and as ``xpass`` if it passes. In both cases this will not cause the test suite to fail as a whole. This
          is particularly useful to mark *flaky* tests (tests that fail at random) to be tackled later.
        * If ``True``, the function will be shown in the terminal output as ``xfailed`` if it fails, but if it
          unexpectedly passes then it will **fail** the test suite. This is particularly useful to mark functions
          that are always failing and there should be a clear indication if they unexpectedly start to pass (for example
          a new release of a library fixes a known bug).


custom marks
~~~~~~~~~~~~

Marks are created dynamically using the factory object ``pytest.mark`` and applied as a decorator.

For example:

.. code-block:: python

    @pytest.mark.timeout(10, "slow", method="thread")
    def test_function():
        ...

Will create and attach a :class:`Mark <_pytest.mark.structures.Mark>` object to the collected
:class:`Item <_pytest.nodes.Item>`, which can then be accessed by fixtures or hooks with
:meth:`Node.iter_markers <_pytest.nodes.Node.iter_markers>`. The ``mark`` object will have the following attributes:

.. code-block:: python

    mark.args == (10, "slow")
    mark.kwargs == {"method": "thread"}


Fixtures
--------

**Tutorial**: :ref:`fixture`.

Fixtures are requested by test functions or other fixtures by declaring them as argument names.


Example of a test requiring a fixture:

.. code-block:: python

    def test_output(capsys):
        print("hello")
        out, err = capsys.readouterr()
        assert out == "hello\n"


Example of a fixture requiring another fixture:

.. code-block:: python

    @pytest.fixture
    def db_session(tmpdir):
        fn = tmpdir / "db.file"
        return connect(str(fn))

For more details, consult the full :ref:`fixtures docs <fixture>`.


@pytest.fixture
~~~~~~~~~~~~~~~

.. autofunction:: pytest.fixture
    :decorator:


.. _`cache-api`:

config.cache
~~~~~~~~~~~~

**Tutorial**: :ref:`cache`.

The ``config.cache`` object allows other plugins and fixtures
to store and retrieve values across test runs. To access it from fixtures
request ``pytestconfig`` into your fixture and get it with ``pytestconfig.cache``.

Under the hood, the cache plugin uses the simple
``dumps``/``loads`` API of the :py:mod:`json` stdlib module.

.. currentmodule:: _pytest.cacheprovider

.. automethod:: Cache.get
.. automethod:: Cache.set
.. automethod:: Cache.makedir


capsys
~~~~~~

**Tutorial**: :doc:`capture`.

.. currentmodule:: _pytest.capture

.. autofunction:: capsys()
    :no-auto-options:

    Returns an instance of :py:class:`CaptureFixture`.

    Example:

    .. code-block:: python

        def test_output(capsys):
            print("hello")
            captured = capsys.readouterr()
            assert captured.out == "hello\n"

.. autoclass:: CaptureFixture()
    :members:


capsysbinary
~~~~~~~~~~~~

**Tutorial**: :doc:`capture`.

.. autofunction:: capsysbinary()
    :no-auto-options:

    Returns an instance of :py:class:`CaptureFixture`.

    Example:

    .. code-block:: python

        def test_output(capsysbinary):
            print("hello")
            captured = capsysbinary.readouterr()
            assert captured.out == b"hello\n"


capfd
~~~~~~

**Tutorial**: :doc:`capture`.

.. autofunction:: capfd()
    :no-auto-options:

    Returns an instance of :py:class:`CaptureFixture`.

    Example:

    .. code-block:: python

        def test_system_echo(capfd):
            os.system('echo "hello"')
            captured = capsys.readouterr()
            assert captured.out == "hello\n"


capfdbinary
~~~~~~~~~~~~

**Tutorial**: :doc:`capture`.

.. autofunction:: capfdbinary()
    :no-auto-options:

    Returns an instance of :py:class:`CaptureFixture`.

    Example:

    .. code-block:: python

        def test_system_echo(capfdbinary):
            os.system('echo "hello"')
            captured = capfdbinary.readouterr()
            assert captured.out == b"hello\n"


doctest_namespace
~~~~~~~~~~~~~~~~~

**Tutorial**: :doc:`doctest`.

.. autofunction:: _pytest.doctest.doctest_namespace()

    Usually this fixture is used in conjunction with another ``autouse`` fixture:

    .. code-block:: python

        @pytest.fixture(autouse=True)
        def add_np(doctest_namespace):
            doctest_namespace["np"] = numpy

    For more details: :ref:`doctest_namespace`.


request
~~~~~~~

**Tutorial**: :ref:`request example`.

The ``request`` fixture is a special fixture providing information of the requesting test function.

.. autoclass:: _pytest.fixtures.FixtureRequest()
    :members:


pytestconfig
~~~~~~~~~~~~

.. autofunction:: _pytest.fixtures.pytestconfig()


record_property
~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`record_property example`.

.. autofunction:: _pytest.junitxml.record_property()


record_testsuite_property
~~~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`record_testsuite_property example`.

.. autofunction:: _pytest.junitxml.record_testsuite_property()

caplog
~~~~~~

**Tutorial**: :doc:`logging`.

.. autofunction:: _pytest.logging.caplog()
    :no-auto-options:

    This returns a :class:`_pytest.logging.LogCaptureFixture` instance.

.. autoclass:: _pytest.logging.LogCaptureFixture
    :members:


monkeypatch
~~~~~~~~~~~

.. currentmodule:: _pytest.monkeypatch

**Tutorial**: :doc:`monkeypatch`.

.. autofunction:: _pytest.monkeypatch.monkeypatch()
    :no-auto-options:

    This returns a :class:`MonkeyPatch` instance.

.. autoclass:: _pytest.monkeypatch.MonkeyPatch
    :members:

testdir
~~~~~~~

.. currentmodule:: _pytest.pytester

This fixture provides a :class:`Testdir` instance useful for black-box testing of test files, making it ideal to
test plugins.

To use it, include in your top-most ``conftest.py`` file::

    pytest_plugins = 'pytester'



.. autoclass:: Testdir()
    :members:

.. autoclass:: RunResult()
    :members:

.. autoclass:: LineMatcher()
    :members:


recwarn
~~~~~~~

**Tutorial**: :ref:`assertwarnings`

.. currentmodule:: _pytest.recwarn

.. autofunction:: recwarn()
    :no-auto-options:

.. autoclass:: _pytest.recwarn.WarningsRecorder()
    :members:

Each recorded warning is an instance of :class:`warnings.WarningMessage`.

.. note::
    :class:`RecordedWarning` was changed from a plain class to a namedtuple in pytest 3.1

.. note::
    ``DeprecationWarning`` and ``PendingDeprecationWarning`` are treated
    differently; see :ref:`ensuring_function_triggers`.


tmp_path
~~~~~~~~

**Tutorial**: :doc:`tmpdir`

.. currentmodule:: _pytest.tmpdir

.. autofunction:: tmp_path()
    :no-auto-options:


tmp_path_factory
~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`tmp_path_factory example`

.. _`tmp_path_factory factory api`:

``tmp_path_factory`` instances have the following methods:

.. currentmodule:: _pytest.tmpdir

.. automethod:: TempPathFactory.mktemp
.. automethod:: TempPathFactory.getbasetemp


tmpdir
~~~~~~

**Tutorial**: :doc:`tmpdir`

.. currentmodule:: _pytest.tmpdir

.. autofunction:: tmpdir()
    :no-auto-options:


tmpdir_factory
~~~~~~~~~~~~~~

**Tutorial**: :ref:`tmpdir factory example`

.. _`tmpdir factory api`:

``tmpdir_factory`` instances have the following methods:

.. currentmodule:: _pytest.tmpdir

.. automethod:: TempdirFactory.mktemp
.. automethod:: TempdirFactory.getbasetemp


.. _`hook-reference`:

Hooks
-----

**Tutorial**: :doc:`writing_plugins`.

.. currentmodule:: _pytest.hookspec

Reference to all hooks which can be implemented by :ref:`conftest.py files <localplugin>` and :ref:`plugins <plugins>`.

Bootstrapping hooks
~~~~~~~~~~~~~~~~~~~

Bootstrapping hooks called for plugins registered early enough (internal and setuptools plugins).

.. autofunction:: pytest_load_initial_conftests
.. autofunction:: pytest_cmdline_preparse
.. autofunction:: pytest_cmdline_parse
.. autofunction:: pytest_cmdline_main

.. _`initialization-hooks`:

Initialization hooks
~~~~~~~~~~~~~~~~~~~~

Initialization hooks called for plugins and ``conftest.py`` files.

.. autofunction:: pytest_addoption
.. autofunction:: pytest_addhooks
.. autofunction:: pytest_configure
.. autofunction:: pytest_unconfigure
.. autofunction:: pytest_sessionstart
.. autofunction:: pytest_sessionfinish

.. autofunction:: pytest_plugin_registered

Test running hooks
~~~~~~~~~~~~~~~~~~

All runtest related hooks receive a :py:class:`pytest.Item <_pytest.main.Item>` object.

.. autofunction:: pytest_runtestloop
.. autofunction:: pytest_runtest_protocol
.. autofunction:: pytest_runtest_logstart
.. autofunction:: pytest_runtest_logfinish
.. autofunction:: pytest_runtest_setup
.. autofunction:: pytest_runtest_call
.. autofunction:: pytest_runtest_teardown
.. autofunction:: pytest_runtest_makereport

For deeper understanding you may look at the default implementation of
these hooks in :py:mod:`_pytest.runner` and maybe also
in :py:mod:`_pytest.pdb` which interacts with :py:mod:`_pytest.capture`
and its input/output capturing in order to immediately drop
into interactive debugging when a test failure occurs.

The :py:mod:`_pytest.terminal` reported specifically uses
the reporting hook to print information about a test run.

.. autofunction:: pytest_pyfunc_call

Collection hooks
~~~~~~~~~~~~~~~~

``pytest`` calls the following hooks for collecting files and directories:

.. autofunction:: pytest_collection
.. autofunction:: pytest_ignore_collect
.. autofunction:: pytest_collect_directory
.. autofunction:: pytest_collect_file
.. autofunction:: pytest_pycollect_makemodule

For influencing the collection of objects in Python modules
you can use the following hook:

.. autofunction:: pytest_pycollect_makeitem
.. autofunction:: pytest_generate_tests
.. autofunction:: pytest_make_parametrize_id

After collection is complete, you can modify the order of
items, delete or otherwise amend the test items:

.. autofunction:: pytest_collection_modifyitems

.. autofunction:: pytest_collection_finish

Reporting hooks
~~~~~~~~~~~~~~~

Session related reporting hooks:

.. autofunction:: pytest_collectstart
.. autofunction:: pytest_make_collect_report
.. autofunction:: pytest_itemcollected
.. autofunction:: pytest_collectreport
.. autofunction:: pytest_deselected
.. autofunction:: pytest_report_header
.. autofunction:: pytest_report_collectionfinish
.. autofunction:: pytest_report_teststatus
.. autofunction:: pytest_terminal_summary
.. autofunction:: pytest_fixture_setup
.. autofunction:: pytest_fixture_post_finalizer
.. autofunction:: pytest_warning_captured

And here is the central hook for reporting about
test execution:

.. autofunction:: pytest_runtest_logreport

You can also use this hook to customize assertion representation for some
types:

.. autofunction:: pytest_assertrepr_compare


Debugging/Interaction hooks
~~~~~~~~~~~~~~~~~~~~~~~~~~~

There are few hooks which can be used for special
reporting or interaction with exceptions:

.. autofunction:: pytest_internalerror
.. autofunction:: pytest_keyboard_interrupt
.. autofunction:: pytest_exception_interact
.. autofunction:: pytest_enter_pdb


Objects
-------

Full reference to objects accessible from :ref:`fixtures <fixture>` or :ref:`hooks <hook-reference>`.


CallInfo
~~~~~~~~

.. autoclass:: _pytest.runner.CallInfo()
    :members:


Class
~~~~~

.. autoclass:: _pytest.python.Class()
    :members:
    :show-inheritance:

Collector
~~~~~~~~~

.. autoclass:: _pytest.nodes.Collector()
    :members:
    :show-inheritance:

Config
~~~~~~

.. autoclass:: _pytest.config.Config()
    :members:

ExceptionInfo
~~~~~~~~~~~~~

.. autoclass:: _pytest._code.ExceptionInfo
    :members:

FixtureDef
~~~~~~~~~~

.. autoclass:: _pytest.fixtures.FixtureDef()
    :members:
    :show-inheritance:

FSCollector
~~~~~~~~~~~

.. autoclass:: _pytest.nodes.FSCollector()
    :members:
    :show-inheritance:

Function
~~~~~~~~

.. autoclass:: _pytest.python.Function()
    :members:
    :show-inheritance:

Item
~~~~

.. autoclass:: _pytest.nodes.Item()
    :members:
    :show-inheritance:

MarkDecorator
~~~~~~~~~~~~~

.. autoclass:: _pytest.mark.MarkDecorator
    :members:


MarkGenerator
~~~~~~~~~~~~~

.. autoclass:: _pytest.mark.MarkGenerator
    :members:


Mark
~~~~

.. autoclass:: _pytest.mark.structures.Mark
    :members:


Metafunc
~~~~~~~~

.. autoclass:: _pytest.python.Metafunc
    :members:

Module
~~~~~~

.. autoclass:: _pytest.python.Module()
    :members:
    :show-inheritance:

Node
~~~~

.. autoclass:: _pytest.nodes.Node()
    :members:

Parser
~~~~~~

.. autoclass:: _pytest.config.argparsing.Parser()
    :members:

PluginManager
~~~~~~~~~~~~~

.. autoclass:: pluggy.PluginManager()
    :members:


PytestPluginManager
~~~~~~~~~~~~~~~~~~~

.. autoclass:: _pytest.config.PytestPluginManager()
    :members:
    :undoc-members:
    :show-inheritance:

Session
~~~~~~~

.. autoclass:: _pytest.main.Session()
    :members:
    :show-inheritance:

TestReport
~~~~~~~~~~

.. autoclass:: _pytest.runner.TestReport()
    :members:
    :inherited-members:

_Result
~~~~~~~

.. autoclass:: pluggy.callers._Result
    :members:

Special Variables
-----------------

pytest treats some global variables in a special manner when defined in a test module.


collect_ignore
~~~~~~~~~~~~~~

**Tutorial**: :ref:`customizing-test-collection`

Can be declared in *conftest.py files* to exclude test directories or modules.
Needs to be ``list[str]``.

.. code-block:: python

  collect_ignore = ["setup.py"]


collect_ignore_glob
~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`customizing-test-collection`

Can be declared in *conftest.py files* to exclude test directories or modules
with Unix shell-style wildcards. Needs to be ``list[str]`` where ``str`` can
contain glob patterns.

.. code-block:: python

  collect_ignore_glob = ["*_ignore.py"]


pytest_plugins
~~~~~~~~~~~~~~

**Tutorial**: :ref:`available installable plugins`

Can be declared at the **global** level in *test modules* and *conftest.py files* to register additional plugins.
Can be either a ``str`` or ``Sequence[str]``.

.. code-block:: python

    pytest_plugins = "myapp.testsupport.myplugin"

.. code-block:: python

    pytest_plugins = ("myapp.testsupport.tools", "myapp.testsupport.regression")


pytest_mark
~~~~~~~~~~~

**Tutorial**: :ref:`scoped-marking`

Can be declared at the **global** level in *test modules* to apply one or more :ref:`marks <marks ref>` to all
test functions and methods. Can be either a single mark or a list of marks.

.. code-block:: python

    import pytest

    pytestmark = pytest.mark.webtest


.. code-block:: python

    import pytest

    pytestmark = [pytest.mark.integration, pytest.mark.slow]

PYTEST_DONT_REWRITE (module docstring)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The text ``PYTEST_DONT_REWRITE`` can be add to any **module docstring** to disable
:ref:`assertion rewriting <assert introspection>` for that module.


Environment Variables
---------------------

Environment variables that can be used to change pytest's behavior.

PYTEST_ADDOPTS
~~~~~~~~~~~~~~

This contains a command-line (parsed by the py:mod:`shlex` module) that will be **prepended** to the command line given
by the user, see :ref:`adding default options` for more information.

PYTEST_DEBUG
~~~~~~~~~~~~

When set, pytest will print tracing and debug information.

PYTEST_PLUGINS
~~~~~~~~~~~~~~

Contains comma-separated list of modules that should be loaded as plugins:

.. code-block:: bash

    export PYTEST_PLUGINS=mymodule.plugin,xdist

PYTEST_DISABLE_PLUGIN_AUTOLOAD
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

When set, disables plugin auto-loading through setuptools entrypoints. Only explicitly specified plugins will be
loaded.

PYTEST_CURRENT_TEST
~~~~~~~~~~~~~~~~~~~

This is not meant to be set by users, but is set by pytest internally with the name of the current test so other
processes can inspect it, see :ref:`pytest current test env` for more information.


.. _`ini options ref`:

Configuration Options
---------------------

Here is a list of builtin configuration options that may be written in a ``pytest.ini``, ``tox.ini`` or ``setup.cfg``
file, usually located at the root of your repository. All options must be under a ``[pytest]`` section
(``[tool:pytest]`` for ``setup.cfg`` files).

.. warning::
    Usage of ``setup.cfg`` is not recommended unless for very simple use cases. ``.cfg``
    files use a different parser than ``pytest.ini`` and ``tox.ini`` which might cause hard to track
    down problems.
    When possible, it is recommended to use the latter files to hold your pytest configuration.

Configuration file options may be overwritten in the command-line by using ``-o/--override``, which can also be
passed multiple times. The expected format is ``name=value``. For example::

   pytest -o console_output_style=classic -o cache_dir=/tmp/mycache


.. confval:: addopts

   Add the specified ``OPTS`` to the set of command line arguments as if they
   had been specified by the user. Example: if you have this ini file content:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        addopts = --maxfail=2 -rf  # exit after 2 failures, report fail info

   issuing ``pytest test_hello.py`` actually means::

        pytest --maxfail=2 -rf test_hello.py

   Default is to add no options.


.. confval:: cache_dir



   Sets a directory where stores content of cache plugin. Default directory is
   ``.pytest_cache`` which is created in :ref:`rootdir <rootdir>`. Directory may be
   relative or absolute path. If setting relative path, then directory is created
   relative to :ref:`rootdir <rootdir>`. Additionally path may contain environment
   variables, that will be expanded. For more information about cache plugin
   please refer to :ref:`cache_provider`.


.. confval:: confcutdir

   Sets a directory where search upwards for ``conftest.py`` files stops.
   By default, pytest will stop searching for ``conftest.py`` files upwards
   from ``pytest.ini``/``tox.ini``/``setup.cfg`` of the project if any,
   or up to the file-system root.


.. confval:: console_output_style



   Sets the console output style while running tests:

   * ``classic``: classic pytest output.
   * ``progress``: like classic pytest output, but with a progress indicator.
   * ``count``: like progress, but shows progress as the number of tests completed instead of a percent.

   The default is ``progress``, but you can fallback to ``classic`` if you prefer or
   the new mode is causing unexpected problems:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        console_output_style = classic


.. confval:: doctest_encoding



   Default encoding to use to decode text files with docstrings.
   :doc:`See how pytest handles doctests <doctest>`.


.. confval:: doctest_optionflags

   One or more doctest flag names from the standard ``doctest`` module.
   :doc:`See how pytest handles doctests <doctest>`.


.. confval:: empty_parameter_set_mark



    Allows to pick the action for empty parametersets in parameterization

    * ``skip`` skips tests with an empty parameterset (default)
    * ``xfail`` marks tests with an empty parameterset as xfail(run=False)
    * ``fail_at_collect`` raises an exception if parametrize collects an empty parameter set

    .. code-block:: ini

      # content of pytest.ini
      [pytest]
      empty_parameter_set_mark = xfail

    .. note::

      The default value of this option is planned to change to ``xfail`` in future releases
      as this is considered less error prone, see `#3155 <https://github.com/pytest-dev/pytest/issues/3155>`_
      for more details.


.. confval:: filterwarnings



   Sets a list of filters and actions that should be taken for matched
   warnings. By default all warnings emitted during the test session
   will be displayed in a summary at the end of the test session.

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        filterwarnings =
            error
            ignore::DeprecationWarning

   This tells pytest to ignore deprecation warnings and turn all other warnings
   into errors. For more information please refer to :ref:`warnings`.


.. confval:: junit_duration_report

    .. versionadded:: 4.1

    Configures how durations are recorded into the JUnit XML report:

    * ``total`` (the default): duration times reported include setup, call, and teardown times.
    * ``call``: duration times reported include only call times, excluding setup and teardown.

    .. code-block:: ini

        [pytest]
        junit_duration_report = call


.. confval:: junit_family

    .. versionadded:: 4.2

    Configures the format of the generated JUnit XML file. The possible options are:

    * ``xunit1`` (or ``legacy``): produces old style output, compatible with the xunit 1.0 format. **This is the default**.
    * ``xunit2``: produces `xunit 2.0 style output <https://github.com/jenkinsci/xunit-plugin/blob/xunit-2.3.2/src/main/resources/org/jenkinsci/plugins/xunit/types/model/xsd/junit-10.xsd>`__,
        which should be more compatible with latest Jenkins versions.

    .. code-block:: ini

        [pytest]
        junit_family = xunit2


.. confval:: junit_logging

    .. versionadded:: 3.5

    Configures if stdout/stderr should be written to the JUnit XML file. Valid values are
    ``system-out``, ``system-err``, and ``no`` (the default).

    .. code-block:: ini

        [pytest]
        junit_logging = system-out


.. confval:: junit_log_passing_tests

    .. versionadded:: 4.6

    If ``junit_logging != "no"``, configures if the captured output should be written
    to the JUnit XML file for **passing** tests. Default is ``True``.

    .. code-block:: ini

        [pytest]
        junit_log_passing_tests = False


.. confval:: junit_suite_name

    To set the name of the root test suite xml item, you can configure the ``junit_suite_name`` option in your config file:

    .. code-block:: ini

        [pytest]
        junit_suite_name = my_suite


.. confval:: log_cli_date_format



    Sets a :py:func:`time.strftime`-compatible string that will be used when formatting dates for live logging.

    .. code-block:: ini

        [pytest]
        log_cli_date_format = %Y-%m-%d %H:%M:%S

    For more information, see :ref:`live_logs`.

.. confval:: log_cli_format



    Sets a :py:mod:`logging`-compatible string used to format live logging messages.

    .. code-block:: ini

        [pytest]
        log_cli_format = %(asctime)s %(levelname)s %(message)s

    For more information, see :ref:`live_logs`.


.. confval:: log_cli_level



    Sets the minimum log message level that should be captured for live logging. The integer value or
    the names of the levels can be used.

    .. code-block:: ini

        [pytest]
        log_cli_level = INFO

    For more information, see :ref:`live_logs`.


.. confval:: log_date_format



    Sets a :py:func:`time.strftime`-compatible string that will be used when formatting dates for logging capture.

    .. code-block:: ini

        [pytest]
        log_date_format = %Y-%m-%d %H:%M:%S

    For more information, see :ref:`logging`.


.. confval:: log_file



    Sets a file name relative to the ``pytest.ini`` file where log messages should be written to, in addition
    to the other logging facilities that are active.

    .. code-block:: ini

        [pytest]
        log_file = logs/pytest-logs.txt

    For more information, see :ref:`logging`.


.. confval:: log_file_date_format



    Sets a :py:func:`time.strftime`-compatible string that will be used when formatting dates for the logging file.

    .. code-block:: ini

        [pytest]
        log_file_date_format = %Y-%m-%d %H:%M:%S

    For more information, see :ref:`logging`.

.. confval:: log_file_format



    Sets a :py:mod:`logging`-compatible string used to format logging messages redirected to the logging file.

    .. code-block:: ini

        [pytest]
        log_file_format = %(asctime)s %(levelname)s %(message)s

    For more information, see :ref:`logging`.

.. confval:: log_file_level



    Sets the minimum log message level that should be captured for the logging file. The integer value or
    the names of the levels can be used.

    .. code-block:: ini

        [pytest]
        log_file_level = INFO

    For more information, see :ref:`logging`.


.. confval:: log_format



    Sets a :py:mod:`logging`-compatible string used to format captured logging messages.

    .. code-block:: ini

        [pytest]
        log_format = %(asctime)s %(levelname)s %(message)s

    For more information, see :ref:`logging`.


.. confval:: log_level



    Sets the minimum log message level that should be captured for logging capture. The integer value or
    the names of the levels can be used.

    .. code-block:: ini

        [pytest]
        log_level = INFO

    For more information, see :ref:`logging`.


.. confval:: log_print



    If set to ``False``, will disable displaying captured logging messages for failed tests.

    .. code-block:: ini

        [pytest]
        log_print = False

    For more information, see :ref:`logging`.


.. confval:: markers

    When the ``--strict-markers`` or ``--strict`` command-line arguments are used,
    only known markers - defined in code by core pytest or some plugin - are allowed.

    You can list additional markers in this setting to add them to the whitelist,
    in which case you probably want to add ``--strict-markers`` to ``addopts``
    to avoid future regressions:

    .. code-block:: ini

        [pytest]
        addopts = --strict-markers
        markers =
            slow
            serial

.. confval:: minversion

   Specifies a minimal pytest version required for running tests.

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        minversion = 3.0  # will fail if we run with pytest-2.8


.. confval:: norecursedirs

   Set the directory basename patterns to avoid when recursing
   for test discovery.  The individual (fnmatch-style) patterns are
   applied to the basename of a directory to decide if to recurse into it.
   Pattern matching characters::

        *       matches everything
        ?       matches any single character
        [seq]   matches any character in seq
        [!seq]  matches any char not in seq

   Default patterns are ``'.*', 'build', 'dist', 'CVS', '_darcs', '{arch}', '*.egg', 'venv'``.
   Setting a ``norecursedirs`` replaces the default.  Here is an example of
   how to avoid certain directories:

   .. code-block:: ini

        [pytest]
        norecursedirs = .svn _build tmp*

   This would tell ``pytest`` to not look into typical subversion or
   sphinx-build directories or into any ``tmp`` prefixed directory.

   Additionally, ``pytest`` will attempt to intelligently identify and ignore a
   virtualenv by the presence of an activation script.  Any directory deemed to
   be the root of a virtual environment will not be considered during test
   collection unless ``‑‑collect‑in‑virtualenv`` is given.  Note also that
   ``norecursedirs`` takes precedence over ``‑‑collect‑in‑virtualenv``; e.g. if
   you intend to run tests in a virtualenv with a base directory that matches
   ``'.*'`` you *must* override ``norecursedirs`` in addition to using the
   ``‑‑collect‑in‑virtualenv`` flag.


.. confval:: python_classes

   One or more name prefixes or glob-style patterns determining which classes
   are considered for test collection. Search for multiple glob patterns by
   adding a space between patterns. By default, pytest will consider any
   class prefixed with ``Test`` as a test collection.  Here is an example of how
   to collect tests from classes that end in ``Suite``:

   .. code-block:: ini

        [pytest]
        python_classes = *Suite

   Note that ``unittest.TestCase`` derived classes are always collected
   regardless of this option, as ``unittest``'s own collection framework is used
   to collect those tests.


.. confval:: python_files

   One or more Glob-style file patterns determining which python files
   are considered as test modules. Search for multiple glob patterns by
   adding a space between patterns:

   .. code-block:: ini

        [pytest]
        python_files = test_*.py check_*.py example_*.py

   Or one per line:

   .. code-block:: ini

        [pytest]
        python_files =
            test_*.py
            check_*.py
            example_*.py

   By default, files matching ``test_*.py`` and ``*_test.py`` will be considered
   test modules.


.. confval:: python_functions

   One or more name prefixes or glob-patterns determining which test functions
   and methods are considered tests. Search for multiple glob patterns by
   adding a space between patterns. By default, pytest will consider any
   function prefixed with ``test`` as a test.  Here is an example of how
   to collect test functions and methods that end in ``_test``:

   .. code-block:: ini

        [pytest]
        python_functions = *_test

   Note that this has no effect on methods that live on a ``unittest
   .TestCase`` derived class, as ``unittest``'s own collection framework is used
   to collect those tests.

   See :ref:`change naming conventions` for more detailed examples.


.. confval:: testpaths



   Sets list of directories that should be searched for tests when
   no specific directories, files or test ids are given in the command line when
   executing pytest from the :ref:`rootdir <rootdir>` directory.
   Useful when all project tests are in a known location to speed up
   test collection and to avoid picking up undesired tests by accident.

   .. code-block:: ini

        [pytest]
        testpaths = testing doc

   This tells pytest to only look for tests in ``testing`` and ``doc``
   directories when executing from the root directory.


.. confval:: usefixtures

    List of fixtures that will be applied to all test functions; this is semantically the same to apply
    the ``@pytest.mark.usefixtures`` marker to all test functions.


    .. code-block:: ini

        [pytest]
        usefixtures =
            clean_db


.. confval:: xfail_strict

    If set to ``True``, tests marked with ``@pytest.mark.xfail`` that actually succeed will by default fail the
    test suite.
    For more information, see :ref:`xfail strict tutorial`.


    .. code-block:: ini

        [pytest]
        xfail_strict = True
