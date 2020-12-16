
.. _usage:

Usage and Invocations
==========================================


.. _cmdline:

Calling pytest through ``python -m pytest``
-----------------------------------------------------



You can invoke testing through the Python interpreter from the command line:

.. code-block:: text

    python -m pytest [...]

This is almost equivalent to invoking the command line script ``pytest [...]``
directly, except that calling via ``python`` will also add the current directory to ``sys.path``.

Possible exit codes
--------------------------------------------------------------

Running ``pytest`` can result in six different exit codes:

:Exit code 0: All tests were collected and passed successfully
:Exit code 1: Tests were collected and run but some of the tests failed
:Exit code 2: Test execution was interrupted by the user
:Exit code 3: Internal error happened while executing tests
:Exit code 4: pytest command line usage error
:Exit code 5: No tests were collected

Getting help on version, option names, environment variables
--------------------------------------------------------------

.. code-block:: bash

    pytest --version   # shows where pytest was imported from
    pytest --fixtures  # show available builtin function arguments
    pytest -h | --help # show help on command line and config file options


.. _maxfail:

Stopping after the first (or N) failures
---------------------------------------------------

To stop the testing process after the first (N) failures:

.. code-block:: bash

    pytest -x            # stop after first failure
    pytest --maxfail=2    # stop after two failures

.. _select-tests:

Specifying tests / selecting tests
---------------------------------------------------

Pytest supports several ways to run and select tests from the command-line.

**Run tests in a module**

.. code-block:: bash

    pytest test_mod.py

**Run tests in a directory**

.. code-block:: bash

    pytest testing/

**Run tests by keyword expressions**

.. code-block:: bash

    pytest -k "MyClass and not method"

This will run tests which contain names that match the given *string expression*, which can
include Python operators that use filenames, class names and function names as variables.
The example above will run ``TestMyClass.test_something``  but not ``TestMyClass.test_method_simple``.

.. _nodeids:

**Run tests by node ids**

Each collected test is assigned a unique ``nodeid`` which consist of the module filename followed
by specifiers like class names, function names and parameters from parametrization, separated by ``::`` characters.

To run a specific test within a module:

.. code-block:: bash

    pytest test_mod.py::test_func


Another example specifying a test method in the command line:

.. code-block:: bash

    pytest test_mod.py::TestClass::test_method

**Run tests by marker expressions**

.. code-block:: bash

    pytest -m slow

Will run all tests which are decorated with the ``@pytest.mark.slow`` decorator.

For more information see :ref:`marks <mark>`.

**Run tests from packages**

.. code-block:: bash

    pytest --pyargs pkg.testing

This will import ``pkg.testing`` and use its filesystem location to find and run tests from.


Modifying Python traceback printing
----------------------------------------------

Examples for modifying traceback printing:

.. code-block:: bash

    pytest --showlocals # show local variables in tracebacks
    pytest -l           # show local variables (shortcut)

    pytest --tb=auto    # (default) 'long' tracebacks for the first and last
                         # entry, but 'short' style for the other entries
    pytest --tb=long    # exhaustive, informative traceback formatting
    pytest --tb=short   # shorter traceback format
    pytest --tb=line    # only one line per failure
    pytest --tb=native  # Python standard library formatting
    pytest --tb=no      # no traceback at all

The ``--full-trace`` causes very long traces to be printed on error (longer
than ``--tb=long``). It also ensures that a stack trace is printed on
**KeyboardInterrupt** (Ctrl+C).
This is very useful if the tests are taking too long and you interrupt them
with Ctrl+C to find out where the tests are *hanging*. By default no output
will be shown (because KeyboardInterrupt is caught by pytest). By using this
option you make sure a trace is shown.


.. _`pytest.detailed_failed_tests_usage`:

Detailed summary report
-----------------------



The ``-r`` flag can be used to display a "short test summary info" at the end of the test session,
making it easy in large test suites to get a clear picture of all failures, skips, xfails, etc.

Example:

.. code-block:: python

    # content of test_example.py
    import pytest


    @pytest.fixture
    def error_fixture():
        assert 0


    def test_ok():
        print("ok")


    def test_fail():
        assert 0


    def test_error(error_fixture):
        pass


    def test_skip():
        pytest.skip("skipping this test")


    def test_xfail():
        pytest.xfail("xfailing this test")


    @pytest.mark.xfail(reason="always xfail")
    def test_xpass():
        pass


.. code-block:: pytest

    $ pytest -ra
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-4.x.y, py-1.x.y, pluggy-0.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR
    collected 6 items

    test_example.py .FEsxX                                               [100%]

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_error _______________________

        @pytest.fixture
        def error_fixture():
    >       assert 0
    E       assert 0

    test_example.py:6: AssertionError
    ================================= FAILURES =================================
    ________________________________ test_fail _________________________________

        def test_fail():
    >       assert 0
    E       assert 0

    test_example.py:14: AssertionError
    ========================= short test summary info ==========================
    SKIPPED [1] $REGENDOC_TMPDIR/test_example.py:23: skipping this test
    XFAIL test_example.py::test_xfail
      reason: xfailing this test
    XPASS test_example.py::test_xpass always xfail
    ERROR test_example.py::test_error - assert 0
    FAILED test_example.py::test_fail - assert 0
    = 1 failed, 1 passed, 1 skipped, 1 xfailed, 1 xpassed, 1 error in 0.12 seconds =

The ``-r`` options accepts a number of characters after it, with ``a`` used
above meaning "all except passes".

Here is the full list of available characters that can be used:

 - ``f`` - failed
 - ``E`` - error
 - ``s`` - skipped
 - ``x`` - xfailed
 - ``X`` - xpassed
 - ``p`` - passed
 - ``P`` - passed with output
 - ``a`` - all except ``pP``
 - ``A`` - all

More than one character can be used, so for example to only see failed and skipped tests, you can execute:

.. code-block:: pytest

    $ pytest -rfs
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-4.x.y, py-1.x.y, pluggy-0.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR
    collected 6 items

    test_example.py .FEsxX                                               [100%]

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_error _______________________

        @pytest.fixture
        def error_fixture():
    >       assert 0
    E       assert 0

    test_example.py:6: AssertionError
    ================================= FAILURES =================================
    ________________________________ test_fail _________________________________

        def test_fail():
    >       assert 0
    E       assert 0

    test_example.py:14: AssertionError
    ========================= short test summary info ==========================
    FAILED test_example.py::test_fail - assert 0
    SKIPPED [1] $REGENDOC_TMPDIR/test_example.py:23: skipping this test
    = 1 failed, 1 passed, 1 skipped, 1 xfailed, 1 xpassed, 1 error in 0.12 seconds =

Using ``p`` lists the passing tests, whilst ``P`` adds an extra section "PASSES" with those tests that passed but had
captured output:

.. code-block:: pytest

    $ pytest -rpP
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-4.x.y, py-1.x.y, pluggy-0.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR
    collected 6 items

    test_example.py .FEsxX                                               [100%]

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_error _______________________

        @pytest.fixture
        def error_fixture():
    >       assert 0
    E       assert 0

    test_example.py:6: AssertionError
    ================================= FAILURES =================================
    ________________________________ test_fail _________________________________

        def test_fail():
    >       assert 0
    E       assert 0

    test_example.py:14: AssertionError
    ================================== PASSES ==================================
    _________________________________ test_ok __________________________________
    --------------------------- Captured stdout call ---------------------------
    ok
    ========================= short test summary info ==========================
    PASSED test_example.py::test_ok
    = 1 failed, 1 passed, 1 skipped, 1 xfailed, 1 xpassed, 1 error in 0.12 seconds =

.. _pdb-option:

Dropping to PDB_ (Python Debugger) on failures
-----------------------------------------------

.. _PDB: http://docs.python.org/library/pdb.html

Python comes with a builtin Python debugger called PDB_.  ``pytest``
allows one to drop into the PDB_ prompt via a command line option:

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

Dropping to PDB_ (Python Debugger) at the start of a test
----------------------------------------------------------


``pytest`` allows one to drop into the PDB_ prompt immediately at the start of each test via a command line option:

.. code-block:: bash

    pytest --trace

This will invoke the Python debugger at the start of every test.

.. _breakpoints:

Setting breakpoints
-------------------

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
-------------------------------------

Python 3.7 introduces a builtin ``breakpoint()`` function.
Pytest supports the use of ``breakpoint()`` with the following behaviours:

 - When ``breakpoint()`` is called and ``PYTHONBREAKPOINT`` is set to the default value, pytest will use the custom internal PDB trace UI instead of the system default ``Pdb``.
 - When tests are complete, the system will default back to the system ``Pdb`` trace UI.
 - With ``--pdb`` passed to pytest, the custom internal Pdb trace UI is used with both ``breakpoint()`` and failed tests/unhandled exceptions.
 - ``--pdbcls`` can be used to specify a custom debugger class.

.. _durations:

Profiling test execution duration
-------------------------------------

.. versionadded: 2.2

To get a list of the slowest 10 test durations:

.. code-block:: bash

    pytest --durations=10

By default, pytest will not show test durations that are too small (<0.01s) unless ``-vv`` is passed on the command-line.

Creating JUnitXML format files
----------------------------------------------------

To create result files which can be read by Jenkins_ or other Continuous
integration servers, use this invocation:

.. code-block:: bash

    pytest --junitxml=path

to create an XML file at ``path``.



To set the name of the root test suite xml item, you can configure the ``junit_suite_name`` option in your config file:

.. code-block:: ini

    [pytest]
    junit_suite_name = my_suite

.. versionadded:: 4.0

JUnit XML specification seems to indicate that ``"time"`` attribute
should report total test execution times, including setup and teardown
(`1 <http://windyroad.com.au/dl/Open%20Source/JUnit.xsd>`_, `2
<https://www.ibm.com/support/knowledgecenter/en/SSQ2R2_14.1.0/com.ibm.rsar.analysis.codereview.cobol.doc/topics/cac_useresults_junit.html>`_).
It is the default pytest behavior. To report just call durations
instead, configure the ``junit_duration_report`` option like this:

.. code-block:: ini

    [pytest]
    junit_duration_report = call

.. _record_property example:

record_property
^^^^^^^^^^^^^^^

If you want to log additional information for a test, you can use the
``record_property`` fixture:

.. code-block:: python

    def test_function(record_property):
        record_property("example_key", 1)
        assert True

This will add an extra property ``example_key="1"`` to the generated
``testcase`` tag:

.. code-block:: xml

    <testcase classname="test_function" file="test_function.py" line="0" name="test_function" time="0.0009">
      <properties>
        <property name="example_key" value="1" />
      </properties>
    </testcase>

Alternatively, you can integrate this functionality with custom markers:

.. code-block:: python

    # content of conftest.py


    def pytest_collection_modifyitems(session, config, items):
        for item in items:
            for marker in item.iter_markers(name="test_id"):
                test_id = marker.args[0]
                item.user_properties.append(("test_id", test_id))

And in your tests:

.. code-block:: python

    # content of test_function.py
    import pytest


    @pytest.mark.test_id(1501)
    def test_function():
        assert True

Will result in:

.. code-block:: xml

    <testcase classname="test_function" file="test_function.py" line="0" name="test_function" time="0.0009">
      <properties>
        <property name="test_id" value="1501" />
      </properties>
    </testcase>

.. warning::

    Please note that using this feature will break schema verifications for the latest JUnitXML schema.
    This might be a problem when used with some CI servers.

record_xml_attribute
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^



To add an additional xml attribute to a testcase element, you can use
``record_xml_attribute`` fixture. This can also be used to override existing values:

.. code-block:: python

    def test_function(record_xml_attribute):
        record_xml_attribute("assertions", "REQ-1234")
        record_xml_attribute("classname", "custom_classname")
        print("hello world")
        assert True

Unlike ``record_property``, this will not add a new child element.
Instead, this will add an attribute ``assertions="REQ-1234"`` inside the generated
``testcase`` tag and override the default ``classname`` with ``"classname=custom_classname"``:

.. code-block:: xml

    <testcase classname="custom_classname" file="test_function.py" line="0" name="test_function" time="0.003" assertions="REQ-1234">
        <system-out>
            hello world
        </system-out>
    </testcase>

.. warning::

    ``record_xml_attribute`` is an experimental feature, and its interface might be replaced
    by something more powerful and general in future versions. The
    functionality per-se will be kept, however.

    Using this over ``record_xml_property`` can help when using ci tools to parse the xml report.
    However, some parsers are quite strict about the elements and attributes that are allowed.
    Many tools use an xsd schema (like the example below) to validate incoming xml.
    Make sure you are using attribute names that are allowed by your parser.

    Below is the Scheme used by Jenkins to validate the XML report:

    .. code-block:: xml

        <xs:element name="testcase">
            <xs:complexType>
                <xs:sequence>
                    <xs:element ref="skipped" minOccurs="0" maxOccurs="1"/>
                    <xs:element ref="error" minOccurs="0" maxOccurs="unbounded"/>
                    <xs:element ref="failure" minOccurs="0" maxOccurs="unbounded"/>
                    <xs:element ref="system-out" minOccurs="0" maxOccurs="unbounded"/>
                    <xs:element ref="system-err" minOccurs="0" maxOccurs="unbounded"/>
                </xs:sequence>
                <xs:attribute name="name" type="xs:string" use="required"/>
                <xs:attribute name="assertions" type="xs:string" use="optional"/>
                <xs:attribute name="time" type="xs:string" use="optional"/>
                <xs:attribute name="classname" type="xs:string" use="optional"/>
                <xs:attribute name="status" type="xs:string" use="optional"/>
            </xs:complexType>
        </xs:element>

.. warning::

    Please note that using this feature will break schema verifications for the latest JUnitXML schema.
    This might be a problem when used with some CI servers.

.. _record_testsuite_property example:

record_testsuite_property
^^^^^^^^^^^^^^^^^^^^^^^^^

.. versionadded:: 4.5

If you want to add a properties node at the test-suite level, which may contains properties
that are relevant to all tests, you can use the ``record_testsuite_property`` session-scoped fixture:

The ``record_testsuite_property`` session-scoped fixture can be used to add properties relevant
to all tests.

.. code-block:: python

    import pytest


    @pytest.fixture(scope="session", autouse=True)
    def log_global_env_facts(record_testsuite_property):
        record_testsuite_property("ARCH", "PPC")
        record_testsuite_property("STORAGE_TYPE", "CEPH")


    class TestMe(object):
        def test_foo(self):
            assert True

The fixture is a callable which receives ``name`` and ``value`` of a ``<property>`` tag
added at the test-suite level of the generated xml:

.. code-block:: xml

    <testsuite errors="0" failures="0" name="pytest" skipped="0" tests="1" time="0.006">
      <properties>
        <property name="ARCH" value="PPC"/>
        <property name="STORAGE_TYPE" value="CEPH"/>
      </properties>
      <testcase classname="test_me.TestMe" file="test_me.py" line="16" name="test_foo" time="0.000243663787842"/>
    </testsuite>

``name`` must be a string, ``value`` will be converted to a string and properly xml-escaped.

The generated XML is compatible with the latest ``xunit`` standard, contrary to `record_property`_
and `record_xml_attribute`_.


Creating resultlog format files
----------------------------------------------------



    This option is rarely used and is scheduled for removal in 5.0.

    See `the deprecation docs <https://docs.pytest.org/en/latest/deprecations.html#result-log-result-log>`__
    for more information.

To create plain-text machine-readable result files you can issue:

.. code-block:: bash

    pytest --resultlog=path

and look at the content at the ``path`` location.  Such files are used e.g.
by the `PyPy-test`_ web page to show test results over several revisions.

.. _`PyPy-test`: http://buildbot.pypy.org/summary


Sending test report to online pastebin service
-----------------------------------------------------

**Creating a URL for each test failure**:

.. code-block:: bash

    pytest --pastebin=failed

This will submit test run information to a remote Paste service and
provide a URL for each failure.  You may select tests as usual or add
for example ``-x`` if you only want to send one particular failure.

**Creating a URL for a whole test session log**:

.. code-block:: bash

    pytest --pastebin=all

Currently only pasting to the http://bpaste.net service is implemented.

Early loading plugins
---------------------

You can early-load plugins (internal and external) explicitly in the command-line with the ``-p`` option::

    pytest -p mypluginmodule

The option receives a ``name`` parameter, which can be:

* A full module dotted name, for example ``myproject.plugins``. This dotted name must be importable.
* The entry-point name of a plugin. This is the name passed to ``setuptools`` when the plugin is
  registered. For example to early-load the `pytest-cov <https://pypi.org/project/pytest-cov/>`__ plugin you can use::

    pytest -p pytest_cov


Disabling plugins
-----------------

To disable loading specific plugins at invocation time, use the ``-p`` option
together with the prefix ``no:``.

Example: to disable loading the plugin ``doctest``, which is responsible for
executing doctest tests from text files, invoke pytest like this:

.. code-block:: bash

    pytest -p no:doctest

.. _`pytest.main-usage`:

Calling pytest from Python code
----------------------------------------------------



You can invoke ``pytest`` from Python code directly::

    pytest.main()

this acts as if you would call "pytest" from the command line.
It will not raise ``SystemExit`` but return the exitcode instead.
You can pass in options and arguments::

    pytest.main(['-x', 'mytestdir'])

You can specify additional plugins to ``pytest.main``::

    # content of myinvoke.py
    import pytest
    class MyPlugin(object):
        def pytest_sessionfinish(self):
            print("*** test run reporting finishing")

    pytest.main(["-qq"], plugins=[MyPlugin()])

Running it will show that ``MyPlugin`` was added and its
hook was invoked:

.. code-block:: pytest

    $ python myinvoke.py
    .FEsxX.                                                              [100%]*** test run reporting finishing

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_error _______________________

        @pytest.fixture
        def error_fixture():
    >       assert 0
    E       assert 0

    test_example.py:6: AssertionError
    ================================= FAILURES =================================
    ________________________________ test_fail _________________________________

        def test_fail():
    >       assert 0
    E       assert 0

    test_example.py:14: AssertionError

.. note::

    Calling ``pytest.main()`` will result in importing your tests and any modules
    that they import. Due to the caching mechanism of python's import system,
    making subsequent calls to ``pytest.main()`` from the same process will not
    reflect changes to those files between the calls. For this reason, making
    multiple calls to ``pytest.main()`` from the same process (in order to re-run
    tests, for example) is not recommended.


.. include:: links.inc
