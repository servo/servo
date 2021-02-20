.. _`reference`:

API Reference
=============

This page contains the full reference to pytest's API.

.. contents::
    :depth: 3
    :local:

Functions
---------

pytest.approx
~~~~~~~~~~~~~

.. autofunction:: pytest.approx

pytest.fail
~~~~~~~~~~~

**Tutorial**: :ref:`skipping`

.. autofunction:: pytest.fail

pytest.skip
~~~~~~~~~~~

.. autofunction:: pytest.skip(msg, [allow_module_level=False])

.. _`pytest.importorskip ref`:

pytest.importorskip
~~~~~~~~~~~~~~~~~~~

.. autofunction:: pytest.importorskip

pytest.xfail
~~~~~~~~~~~~

.. autofunction:: pytest.xfail

pytest.exit
~~~~~~~~~~~

.. autofunction:: pytest.exit

pytest.main
~~~~~~~~~~~

.. autofunction:: pytest.main

pytest.param
~~~~~~~~~~~~

.. autofunction:: pytest.param(*values, [id], [marks])

pytest.raises
~~~~~~~~~~~~~

**Tutorial**: :ref:`assertraises`.

.. autofunction:: pytest.raises(expected_exception: Exception [, *, match])
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

            @pytest.mark.filterwarnings("ignore:.*usage will be deprecated.*:DeprecationWarning")
            def test_foo():
                ...


.. _`pytest.mark.parametrize ref`:

pytest.mark.parametrize
~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :doc:`parametrize`.

This mark has the same signature as :py:meth:`_pytest.python.Metafunc.parametrize`; see there.


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

.. py:function:: pytest.mark.usefixtures(*names)

    :param args: The names of the fixture to use, as strings.

.. note::

    When using `usefixtures` in hooks, it can only load fixtures when applied to a test function before test setup
    (for example in the `pytest_collection_modifyitems` hook).

    Also not that his mark has no effect when applied to **fixtures**.



.. _`pytest.mark.xfail ref`:

pytest.mark.xfail
~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`xfail`.

Marks a test function as *expected to fail*.

.. py:function:: pytest.mark.xfail(condition=None, *, reason=None, raises=None, run=True, strict=False)

    :type condition: bool or str
    :param condition:
        Condition for marking the test function as xfail (``True/False`` or a
        :ref:`condition string <string conditions>`). If a bool, you also have
        to specify ``reason`` (see :ref:`condition string <string conditions>`).
    :keyword str reason:
        Reason why the test function is marked as xfail.
    :keyword Type[Exception] raises:
        Exception subclass expected to be raised by the test function; other exceptions will fail the test.
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


Custom marks
~~~~~~~~~~~~

Marks are created dynamically using the factory object ``pytest.mark`` and applied as a decorator.

For example:

.. code-block:: python

    @pytest.mark.timeout(10, "slow", method="thread")
    def test_function():
        ...

Will create and attach a :class:`Mark <_pytest.mark.structures.Mark>` object to the collected
:class:`Item <pytest.Item>`, which can then be accessed by fixtures or hooks with
:meth:`Node.iter_markers <_pytest.nodes.Node.iter_markers>`. The ``mark`` object will have the following attributes:

.. code-block:: python

    mark.args == (10, "slow")
    mark.kwargs == {"method": "thread"}


.. _`fixtures-api`:

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


.. _`pytest.fixture-api`:

@pytest.fixture
~~~~~~~~~~~~~~~

.. autofunction:: pytest.fixture
    :decorator:


.. fixture:: cache

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


.. fixture:: capsys

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


.. fixture:: capsysbinary

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


.. fixture:: capfd

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
            captured = capfd.readouterr()
            assert captured.out == "hello\n"


.. fixture:: capfdbinary

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


.. fixture:: doctest_namespace

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


.. fixture:: request

request
~~~~~~~

**Tutorial**: :ref:`request example`.

The ``request`` fixture is a special fixture providing information of the requesting test function.

.. autoclass:: _pytest.fixtures.FixtureRequest()
    :members:


.. fixture:: pytestconfig

pytestconfig
~~~~~~~~~~~~

.. autofunction:: _pytest.fixtures.pytestconfig()


.. fixture:: record_property

record_property
~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`record_property example`.

.. autofunction:: _pytest.junitxml.record_property()


.. fixture:: record_testsuite_property

record_testsuite_property
~~~~~~~~~~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`record_testsuite_property example`.

.. autofunction:: _pytest.junitxml.record_testsuite_property()


.. fixture:: caplog

caplog
~~~~~~

**Tutorial**: :doc:`logging`.

.. autofunction:: _pytest.logging.caplog()
    :no-auto-options:

    Returns a :class:`_pytest.logging.LogCaptureFixture` instance.

.. autoclass:: _pytest.logging.LogCaptureFixture
    :members:


.. fixture:: monkeypatch

monkeypatch
~~~~~~~~~~~

.. currentmodule:: _pytest.monkeypatch

**Tutorial**: :doc:`monkeypatch`.

.. autofunction:: _pytest.monkeypatch.monkeypatch()
    :no-auto-options:

    Returns a :class:`MonkeyPatch` instance.

.. autoclass:: _pytest.monkeypatch.MonkeyPatch
    :members:


.. fixture:: testdir

testdir
~~~~~~~

.. currentmodule:: _pytest.pytester

This fixture provides a :class:`Testdir` instance useful for black-box testing of test files, making it ideal to
test plugins.

To use it, include in your top-most ``conftest.py`` file:

.. code-block:: python

    pytest_plugins = "pytester"



.. autoclass:: Testdir()
    :members:

.. autoclass:: RunResult()
    :members:

.. autoclass:: LineMatcher()
    :members:


.. fixture:: recwarn

recwarn
~~~~~~~

**Tutorial**: :ref:`assertwarnings`

.. currentmodule:: _pytest.recwarn

.. autofunction:: recwarn()
    :no-auto-options:

.. autoclass:: WarningsRecorder()
    :members:

Each recorded warning is an instance of :class:`warnings.WarningMessage`.

.. note::
    ``DeprecationWarning`` and ``PendingDeprecationWarning`` are treated
    differently; see :ref:`ensuring_function_triggers`.


.. fixture:: tmp_path

tmp_path
~~~~~~~~

**Tutorial**: :doc:`tmpdir`

.. currentmodule:: _pytest.tmpdir

.. autofunction:: tmp_path()
    :no-auto-options:


.. fixture:: tmp_path_factory

tmp_path_factory
~~~~~~~~~~~~~~~~

**Tutorial**: :ref:`tmp_path_factory example`

.. _`tmp_path_factory factory api`:

``tmp_path_factory`` instances have the following methods:

.. currentmodule:: _pytest.tmpdir

.. automethod:: TempPathFactory.mktemp
.. automethod:: TempPathFactory.getbasetemp


.. fixture:: tmpdir

tmpdir
~~~~~~

**Tutorial**: :doc:`tmpdir`

.. currentmodule:: _pytest.tmpdir

.. autofunction:: tmpdir()
    :no-auto-options:


.. fixture:: tmpdir_factory

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

Collection hooks
~~~~~~~~~~~~~~~~

``pytest`` calls the following hooks for collecting files and directories:

.. autofunction:: pytest_collection
.. autofunction:: pytest_ignore_collect
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

Test running (runtest) hooks
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

All runtest related hooks receive a :py:class:`pytest.Item <pytest.Item>` object.

.. autofunction:: pytest_runtestloop
.. autofunction:: pytest_runtest_protocol
.. autofunction:: pytest_runtest_logstart
.. autofunction:: pytest_runtest_logfinish
.. autofunction:: pytest_runtest_setup
.. autofunction:: pytest_runtest_call
.. autofunction:: pytest_runtest_teardown
.. autofunction:: pytest_runtest_makereport

For deeper understanding you may look at the default implementation of
these hooks in ``_pytest.runner`` and maybe also
in ``_pytest.pdb`` which interacts with ``_pytest.capture``
and its input/output capturing in order to immediately drop
into interactive debugging when a test failure occurs.

.. autofunction:: pytest_pyfunc_call

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
.. autofunction:: pytest_warning_recorded

Central hook for reporting about test execution:

.. autofunction:: pytest_runtest_logreport

Assertion related hooks:

.. autofunction:: pytest_assertrepr_compare
.. autofunction:: pytest_assertion_pass


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

.. autoclass:: pytest.Class()
    :members:
    :show-inheritance:

Collector
~~~~~~~~~

.. autoclass:: pytest.Collector()
    :members:
    :show-inheritance:

CollectReport
~~~~~~~~~~~~~

.. autoclass:: _pytest.reports.CollectReport()
    :members:
    :show-inheritance:
    :inherited-members:

Config
~~~~~~

.. autoclass:: _pytest.config.Config()
    :members:

ExceptionInfo
~~~~~~~~~~~~~

.. autoclass:: _pytest._code.ExceptionInfo
    :members:


ExitCode
~~~~~~~~

.. autoclass:: pytest.ExitCode
    :members:

File
~~~~

.. autoclass:: pytest.File()
    :members:
    :show-inheritance:


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

.. autoclass:: pytest.Function()
    :members:
    :show-inheritance:

Item
~~~~

.. autoclass:: pytest.Item()
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

.. autoclass:: pytest.Module()
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


PytestPluginManager
~~~~~~~~~~~~~~~~~~~

.. autoclass:: _pytest.config.PytestPluginManager()
    :members:
    :undoc-members:
    :inherited-members:
    :show-inheritance:

Session
~~~~~~~

.. autoclass:: pytest.Session()
    :members:
    :show-inheritance:

TestReport
~~~~~~~~~~

.. autoclass:: _pytest.reports.TestReport()
    :members:
    :show-inheritance:
    :inherited-members:

_Result
~~~~~~~

Result used within :ref:`hook wrappers <hookwrapper>`.

.. autoclass:: pluggy.callers._Result
.. automethod:: pluggy.callers._Result.get_result
.. automethod:: pluggy.callers._Result.force_result

Global Variables
----------------

pytest treats some global variables in a special manner when defined in a test module or
``conftest.py`` files.


.. globalvar:: collect_ignore

**Tutorial**: :ref:`customizing-test-collection`

Can be declared in *conftest.py files* to exclude test directories or modules.
Needs to be ``list[str]``.

.. code-block:: python

  collect_ignore = ["setup.py"]


.. globalvar:: collect_ignore_glob

**Tutorial**: :ref:`customizing-test-collection`

Can be declared in *conftest.py files* to exclude test directories or modules
with Unix shell-style wildcards. Needs to be ``list[str]`` where ``str`` can
contain glob patterns.

.. code-block:: python

  collect_ignore_glob = ["*_ignore.py"]


.. globalvar:: pytest_plugins

**Tutorial**: :ref:`available installable plugins`

Can be declared at the **global** level in *test modules* and *conftest.py files* to register additional plugins.
Can be either a ``str`` or ``Sequence[str]``.

.. code-block:: python

    pytest_plugins = "myapp.testsupport.myplugin"

.. code-block:: python

    pytest_plugins = ("myapp.testsupport.tools", "myapp.testsupport.regression")


.. globalvar:: pytestmark

**Tutorial**: :ref:`scoped-marking`

Can be declared at the **global** level in *test modules* to apply one or more :ref:`marks <marks ref>` to all
test functions and methods. Can be either a single mark or a list of marks (applied in left-to-right order).

.. code-block:: python

    import pytest

    pytestmark = pytest.mark.webtest


.. code-block:: python

    import pytest

    pytestmark = [pytest.mark.integration, pytest.mark.slow]


Environment Variables
---------------------

Environment variables that can be used to change pytest's behavior.

.. envvar:: PYTEST_ADDOPTS

This contains a command-line (parsed by the py:mod:`shlex` module) that will be **prepended** to the command line given
by the user, see :ref:`adding default options` for more information.

.. envvar:: PYTEST_CURRENT_TEST

This is not meant to be set by users, but is set by pytest internally with the name of the current test so other
processes can inspect it, see :ref:`pytest current test env` for more information.

.. envvar:: PYTEST_DEBUG

When set, pytest will print tracing and debug information.

.. envvar:: PYTEST_DISABLE_PLUGIN_AUTOLOAD

When set, disables plugin auto-loading through setuptools entrypoints. Only explicitly specified plugins will be
loaded.

.. envvar:: PYTEST_PLUGINS

Contains comma-separated list of modules that should be loaded as plugins:

.. code-block:: bash

    export PYTEST_PLUGINS=mymodule.plugin,xdist

.. envvar:: PY_COLORS

When set to ``1``, pytest will use color in terminal output.
When set to ``0``, pytest will not use color.
``PY_COLORS`` takes precedence over ``NO_COLOR`` and ``FORCE_COLOR``.

.. envvar:: NO_COLOR

When set (regardless of value), pytest will not use color in terminal output.
``PY_COLORS`` takes precedence over ``NO_COLOR``, which takes precedence over ``FORCE_COLOR``.
See `no-color.org <https://no-color.org/>`__ for other libraries supporting this community standard.

.. envvar:: FORCE_COLOR

When set (regardless of value), pytest will use color in terminal output.
``PY_COLORS`` and ``NO_COLOR`` take precedence over ``FORCE_COLOR``.

Exceptions
----------

.. autoclass:: pytest.UsageError()
    :show-inheritance:

.. _`warnings ref`:

Warnings
--------

Custom warnings generated in some situations such as improper usage or deprecated features.

.. autoclass:: pytest.PytestWarning
   :show-inheritance:

.. autoclass:: pytest.PytestAssertRewriteWarning
   :show-inheritance:

.. autoclass:: pytest.PytestCacheWarning
   :show-inheritance:

.. autoclass:: pytest.PytestCollectionWarning
   :show-inheritance:

.. autoclass:: pytest.PytestConfigWarning
   :show-inheritance:

.. autoclass:: pytest.PytestDeprecationWarning
   :show-inheritance:

.. autoclass:: pytest.PytestExperimentalApiWarning
   :show-inheritance:

.. autoclass:: pytest.PytestUnhandledCoroutineWarning
   :show-inheritance:

.. autoclass:: pytest.PytestUnknownMarkWarning
   :show-inheritance:


Consult the :ref:`internal-warnings` section in the documentation for more information.


.. _`ini options ref`:

Configuration Options
---------------------

Here is a list of builtin configuration options that may be written in a ``pytest.ini``, ``pyproject.toml``, ``tox.ini`` or ``setup.cfg``
file, usually located at the root of your repository. To see each file format in details, see
:ref:`config file formats`.

.. warning::
    Usage of ``setup.cfg`` is not recommended except for very simple use cases. ``.cfg``
    files use a different parser than ``pytest.ini`` and ``tox.ini`` which might cause hard to track
    down problems.
    When possible, it is recommended to use the latter files, or ``pyproject.toml``, to hold your pytest configuration.

Configuration options may be overwritten in the command-line by using ``-o/--override-ini``, which can also be
passed multiple times. The expected format is ``name=value``. For example::

   pytest -o console_output_style=classic -o cache_dir=/tmp/mycache


.. confval:: addopts

   Add the specified ``OPTS`` to the set of command line arguments as if they
   had been specified by the user. Example: if you have this ini file content:

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        addopts = --maxfail=2 -rf  # exit after 2 failures, report fail info

   issuing ``pytest test_hello.py`` actually means:

   .. code-block:: bash

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


.. confval:: faulthandler_timeout

   Dumps the tracebacks of all threads if a test takes longer than ``X`` seconds to run (including
   fixture setup and teardown). Implemented using the `faulthandler.dump_traceback_later`_ function,
   so all caveats there apply.

   .. code-block:: ini

        # content of pytest.ini
        [pytest]
        faulthandler_timeout=5

   For more information please refer to :ref:`faulthandler`.

.. _`faulthandler.dump_traceback_later`: https://docs.python.org/3/library/faulthandler.html#faulthandler.dump_traceback_later


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
    .. versionchanged:: 5.4
        ``log``, ``all``, ``out-err`` options added.

    Configures if captured output should be written to the JUnit XML file. Valid values are:

    * ``log``: write only ``logging`` captured output.
    * ``system-out``: write captured ``stdout`` contents.
    * ``system-err``: write captured ``stderr`` contents.
    * ``out-err``: write both captured ``stdout`` and ``stderr`` contents.
    * ``all``: write captured ``logging``, ``stdout`` and ``stderr`` contents.
    * ``no`` (the default): no captured output is written.

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

.. confval:: log_auto_indent

    Allow selective auto-indentation of multiline log messages.

    Supports command line option ``--log-auto-indent [value]``
    and config option ``log_auto_indent = [value]`` to set the
    auto-indentation behavior for all logging.

    ``[value]`` can be:
        * True or "On" - Dynamically auto-indent multiline log messages
        * False or "Off" or 0 - Do not auto-indent multiline log messages (the default behavior)
        * [positive integer] - auto-indent multiline log messages by [value] spaces

    .. code-block:: ini

        [pytest]
        log_auto_indent = False

    Supports passing kwarg ``extra={"auto_indent": [value]}`` to
    calls to ``logging.log()`` to specify auto-indentation behavior for
    a specific entry in the log. ``extra`` kwarg overrides the value specified
    on the command line or in the config.

.. confval:: log_cli

    Enable log display during test run (also known as :ref:`"live logging" <live_logs>`).
    The default is ``False``.

    .. code-block:: ini

        [pytest]
        log_cli = True

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

    .. note::
        The use of ``--strict-markers`` is highly preferred. ``--strict`` was kept for
        backward compatibility only and may be confusing for others as it only applies to
        markers and not to other options.

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


.. confval:: required_plugins

   A space separated list of plugins that must be present for pytest to run.
   Plugins can be listed with or without version specifiers directly following
   their name. Whitespace between different version specifiers is not allowed.
   If any one of the plugins is not found, emit an error.

   .. code-block:: ini

       [pytest]
       required_plugins = pytest-django>=3.0.0,<4.0.0 pytest-html pytest-xdist>=1.0.0


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


.. _`command-line-flags`:

Command-line Flags
------------------

All the command-line flags can be obtained by running ``pytest --help``::

    $ pytest --help
    usage: pytest [options] [file_or_dir] [file_or_dir] [...]

    positional arguments:
      file_or_dir

    general:
      -k EXPRESSION         only run tests which match the given substring
                            expression. An expression is a python evaluatable
                            expression where all names are substring-matched
                            against test names and their parent classes.
                            Example: -k 'test_method or test_other' matches all
                            test functions and classes whose name contains
                            'test_method' or 'test_other', while -k 'not
                            test_method' matches those that don't contain
                            'test_method' in their names. -k 'not test_method
                            and not test_other' will eliminate the matches.
                            Additionally keywords are matched to classes and
                            functions containing extra names in their
                            'extra_keyword_matches' set, as well as functions
                            which have names assigned directly to them. The
                            matching is case-insensitive.
      -m MARKEXPR           only run tests matching given mark expression.
                            For example: -m 'mark1 and not mark2'.
      --markers             show markers (builtin, plugin and per-project ones).
      -x, --exitfirst       exit instantly on first error or failed test.
      --fixtures, --funcargs
                            show available fixtures, sorted by plugin appearance
                            (fixtures with leading '_' are only shown with '-v')
      --fixtures-per-test   show fixtures per test
      --pdb                 start the interactive Python debugger on errors or
                            KeyboardInterrupt.
      --pdbcls=modulename:classname
                            start a custom interactive Python debugger on
                            errors. For example:
                            --pdbcls=IPython.terminal.debugger:TerminalPdb
      --trace               Immediately break when running each test.
      --capture=method      per-test capturing method: one of fd|sys|no|tee-sys.
      -s                    shortcut for --capture=no.
      --runxfail            report the results of xfail tests as if they were
                            not marked
      --lf, --last-failed   rerun only the tests that failed at the last run (or
                            all if none failed)
      --ff, --failed-first  run all tests, but run the last failures first.
                            This may re-order tests and thus lead to repeated
                            fixture setup/teardown.
      --nf, --new-first     run tests from new files first, then the rest of the
                            tests sorted by file mtime
      --cache-show=[CACHESHOW]
                            show cache contents, don't perform collection or
                            tests. Optional argument: glob (default: '*').
      --cache-clear         remove all cache contents at start of test run.
      --lfnf={all,none}, --last-failed-no-failures={all,none}
                            which tests to run with no previously (known)
                            failures.
      --sw, --stepwise      exit on test failure and continue from last failing
                            test next time
      --stepwise-skip       ignore the first failing test but stop on the next
                            failing test

    reporting:
      --durations=N         show N slowest setup/test durations (N=0 for all).
      --durations-min=N     Minimal duration in seconds for inclusion in slowest
                            list. Default 0.005
      -v, --verbose         increase verbosity.
      --no-header           disable header
      --no-summary          disable summary
      -q, --quiet           decrease verbosity.
      --verbosity=VERBOSE   set verbosity. Default is 0.
      -r chars              show extra test summary info as specified by chars:
                            (f)ailed, (E)rror, (s)kipped, (x)failed, (X)passed,
                            (p)assed, (P)assed with output, (a)ll except passed
                            (p/P), or (A)ll. (w)arnings are enabled by default
                            (see --disable-warnings), 'N' can be used to reset
                            the list. (default: 'fE').
      --disable-warnings, --disable-pytest-warnings
                            disable warnings summary
      -l, --showlocals      show locals in tracebacks (disabled by default).
      --tb=style            traceback print mode
                            (auto/long/short/line/native/no).
      --show-capture={no,stdout,stderr,log,all}
                            Controls how captured stdout/stderr/log is shown on
                            failed tests. Default is 'all'.
      --full-trace          don't cut any tracebacks (default is to cut).
      --color=color         color terminal output (yes/no/auto).
      --code-highlight={yes,no}
                            Whether code should be highlighted (only if --color
                            is also enabled)
      --pastebin=mode       send failed|all info to bpaste.net pastebin service.
      --junit-xml=path      create junit-xml style report file at given path.
      --junit-prefix=str    prepend prefix to classnames in junit-xml output

    pytest-warnings:
      -W PYTHONWARNINGS, --pythonwarnings=PYTHONWARNINGS
                            set which warnings to report, see -W option of
                            python itself.
      --maxfail=num         exit after first num failures or errors.
      --strict-config       any warnings encountered while parsing the `pytest`
                            section of the configuration file raise errors.
      --strict-markers, --strict
                            markers not registered in the `markers` section of
                            the configuration file raise errors.
      -c file               load configuration from `file` instead of trying to
                            locate one of the implicit configuration files.
      --continue-on-collection-errors
                            Force test execution even if collection errors
                            occur.
      --rootdir=ROOTDIR     Define root directory for tests. Can be relative
                            path: 'root_dir', './root_dir',
                            'root_dir/another_dir/'; absolute path:
                            '/home/user/root_dir'; path with variables:
                            '$HOME/root_dir'.

    collection:
      --collect-only, --co  only collect tests, don't execute them.
      --pyargs              try to interpret all arguments as python packages.
      --ignore=path         ignore path during collection (multi-allowed).
      --ignore-glob=path    ignore path pattern during collection (multi-
                            allowed).
      --deselect=nodeid_prefix
                            deselect item (via node id prefix) during collection
                            (multi-allowed).
      --confcutdir=dir      only load conftest.py's relative to specified dir.
      --noconftest          Don't load any conftest.py files.
      --keep-duplicates     Keep duplicate tests.
      --collect-in-virtualenv
                            Don't ignore tests in a local virtualenv directory
      --import-mode={prepend,append,importlib}
                            prepend/append to sys.path when importing test
                            modules and conftest files, default is to prepend.
      --doctest-modules     run doctests in all .py modules
      --doctest-report={none,cdiff,ndiff,udiff,only_first_failure}
                            choose another output format for diffs on doctest
                            failure
      --doctest-glob=pat    doctests file matching pattern, default: test*.txt
      --doctest-ignore-import-errors
                            ignore doctest ImportErrors
      --doctest-continue-on-failure
                            for a given doctest, continue to run after the first
                            failure

    test session debugging and configuration:
      --basetemp=dir        base temporary directory for this test run.(warning:
                            this directory is removed if it exists)
      -V, --version         display pytest version and information about
                            plugins.When given twice, also display information
                            about plugins.
      -h, --help            show help message and configuration info
      -p name               early-load given plugin module name or entry point
                            (multi-allowed).
                            To avoid loading of plugins, use the `no:` prefix,
                            e.g. `no:doctest`.
      --trace-config        trace considerations of conftest.py files.
      --debug               store internal tracing debug information in
                            'pytestdebug.log'.
      -o OVERRIDE_INI, --override-ini=OVERRIDE_INI
                            override ini option with "option=value" style, e.g.
                            `-o xfail_strict=True -o cache_dir=cache`.
      --assert=MODE         Control assertion debugging tools.
                            'plain' performs no assertion debugging.
                            'rewrite' (the default) rewrites assert statements
                            in test modules on import to provide assert
                            expression information.
      --setup-only          only setup fixtures, do not execute tests.
      --setup-show          show setup of fixtures while executing tests.
      --setup-plan          show what fixtures and tests would be executed but
                            don't execute anything.

    logging:
      --log-level=LEVEL     level of messages to catch/display.
                            Not set by default, so it depends on the root/parent
                            log handler's effective level, where it is "WARNING"
                            by default.
      --log-format=LOG_FORMAT
                            log format as used by the logging module.
      --log-date-format=LOG_DATE_FORMAT
                            log date format as used by the logging module.
      --log-cli-level=LOG_CLI_LEVEL
                            cli logging level.
      --log-cli-format=LOG_CLI_FORMAT
                            log format as used by the logging module.
      --log-cli-date-format=LOG_CLI_DATE_FORMAT
                            log date format as used by the logging module.
      --log-file=LOG_FILE   path to a file when logging will be written to.
      --log-file-level=LOG_FILE_LEVEL
                            log file logging level.
      --log-file-format=LOG_FILE_FORMAT
                            log format as used by the logging module.
      --log-file-date-format=LOG_FILE_DATE_FORMAT
                            log date format as used by the logging module.
      --log-auto-indent=LOG_AUTO_INDENT
                            Auto-indent multiline messages passed to the logging
                            module. Accepts true|on, false|off or an integer.

    [pytest] ini-options in the first pytest.ini|tox.ini|setup.cfg file found:

      markers (linelist):   markers for test functions
      empty_parameter_set_mark (string):
                            default marker for empty parametersets
      norecursedirs (args): directory patterns to avoid for recursion
      testpaths (args):     directories to search for tests when no files or
                            directories are given in the command line.
      filterwarnings (linelist):
                            Each line specifies a pattern for
                            warnings.filterwarnings. Processed after
                            -W/--pythonwarnings.
      usefixtures (args):   list of default fixtures to be used with this
                            project
      python_files (args):  glob-style file patterns for Python test module
                            discovery
      python_classes (args):
                            prefixes or glob names for Python test class
                            discovery
      python_functions (args):
                            prefixes or glob names for Python test function and
                            method discovery
      disable_test_id_escaping_and_forfeit_all_rights_to_community_support (bool):
                            disable string escape non-ascii characters, might
                            cause unwanted side effects(use at your own risk)
      console_output_style (string):
                            console output: "classic", or with additional
                            progress information ("progress" (percentage) |
                            "count").
      xfail_strict (bool):  default for the strict parameter of xfail markers
                            when not given explicitly (default: False)
      enable_assertion_pass_hook (bool):
                            Enables the pytest_assertion_pass hook.Make sure to
                            delete any previously generated pyc cache files.
      junit_suite_name (string):
                            Test suite name for JUnit report
      junit_logging (string):
                            Write captured log messages to JUnit report: one of
                            no|log|system-out|system-err|out-err|all
      junit_log_passing_tests (bool):
                            Capture log information for passing tests to JUnit
                            report:
      junit_duration_report (string):
                            Duration time to report: one of total|call
      junit_family (string):
                            Emit XML for schema: one of legacy|xunit1|xunit2
      doctest_optionflags (args):
                            option flags for doctests
      doctest_encoding (string):
                            encoding used for doctest files
      cache_dir (string):   cache directory path.
      log_level (string):   default value for --log-level
      log_format (string):  default value for --log-format
      log_date_format (string):
                            default value for --log-date-format
      log_cli (bool):       enable log display during test run (also known as
                            "live logging").
      log_cli_level (string):
                            default value for --log-cli-level
      log_cli_format (string):
                            default value for --log-cli-format
      log_cli_date_format (string):
                            default value for --log-cli-date-format
      log_file (string):    default value for --log-file
      log_file_level (string):
                            default value for --log-file-level
      log_file_format (string):
                            default value for --log-file-format
      log_file_date_format (string):
                            default value for --log-file-date-format
      log_auto_indent (string):
                            default value for --log-auto-indent
      faulthandler_timeout (string):
                            Dump the traceback of all threads if a test takes
                            more than TIMEOUT seconds to finish.
      addopts (args):       extra command line options
      minversion (string):  minimally required pytest version
      required_plugins (args):
                            plugins that must be present for pytest to run

    environment variables:
      PYTEST_ADDOPTS           extra command line options
      PYTEST_PLUGINS           comma-separated plugins to load during startup
      PYTEST_DISABLE_PLUGIN_AUTOLOAD set to disable plugin auto-loading
      PYTEST_DEBUG             set to enable debug tracing of pytest's internals


    to see available markers type: pytest --markers
    to see available fixtures type: pytest --fixtures
    (shown according to specified file_or_dir or current dir if not specified; fixtures with leading '_' are only shown with the '-v' option
