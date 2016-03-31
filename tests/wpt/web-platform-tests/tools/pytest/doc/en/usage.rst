
.. _usage:

Usage and Invocations
==========================================


.. _cmdline:

Calling pytest through ``python -m pytest``
-----------------------------------------------------

.. versionadded:: 2.0

You can invoke testing through the Python interpreter from the command line::

    python -m pytest [...]

This is equivalent to invoking the command line script ``py.test [...]``
directly.

Getting help on version, option names, environment variables
--------------------------------------------------------------

::

    py.test --version   # shows where pytest was imported from
    py.test --fixtures  # show available builtin function arguments
    py.test -h | --help # show help on command line and config file options


Stopping after the first (or N) failures
---------------------------------------------------

To stop the testing process after the first (N) failures::

    py.test -x            # stop after first failure
    py.test --maxfail=2    # stop after two failures

Specifying tests / selecting tests
---------------------------------------------------

Several test run options::

    py.test test_mod.py   # run tests in module
    py.test somepath      # run all tests below somepath
    py.test -k stringexpr # only run tests with names that match the
                          # "string expression", e.g. "MyClass and not method"
                          # will select TestMyClass.test_something
                          # but not TestMyClass.test_method_simple
    py.test test_mod.py::test_func  # only run tests that match the "node ID",
                                    # e.g "test_mod.py::test_func" will select
                                    # only test_func in test_mod.py
    py.test test_mod.py::TestClass::test_method  # run a single method in
                                                 # a single class

Import 'pkg' and use its filesystem location to find and run tests::

    py.test --pyargs pkg # run all tests found below directory of pypkg

Modifying Python traceback printing
----------------------------------------------

Examples for modifying traceback printing::

    py.test --showlocals # show local variables in tracebacks
    py.test -l           # show local variables (shortcut)

    py.test --tb=auto    # (default) 'long' tracebacks for the first and last
                         # entry, but 'short' style for the other entries
    py.test --tb=long    # exhaustive, informative traceback formatting
    py.test --tb=short   # shorter traceback format
    py.test --tb=line    # only one line per failure
    py.test --tb=native  # Python standard library formatting
    py.test --tb=no      # no traceback at all

Dropping to PDB_ (Python Debugger) on failures
-----------------------------------------------

.. _PDB: http://docs.python.org/library/pdb.html

Python comes with a builtin Python debugger called PDB_.  ``pytest``
allows one to drop into the PDB_ prompt via a command line option::

    py.test --pdb

This will invoke the Python debugger on every failure.  Often you might
only want to do this for the first failing test to understand a certain
failure situation::

    py.test -x --pdb   # drop to PDB on first failure, then end test session
    py.test --pdb --maxfail=3  # drop to PDB for first three failures

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

Setting a breakpoint / aka ``set_trace()``
----------------------------------------------------

If you want to set a breakpoint and enter the ``pdb.set_trace()`` you
can use a helper::

    import pytest
    def test_function():
        ...
        pytest.set_trace()    # invoke PDB debugger and tracing

.. versionadded: 2.0.0

Prior to pytest version 2.0.0 you could only enter PDB_ tracing if you disabled
capturing on the command line via ``py.test -s``. In later versions, pytest
automatically disables its output capture when you enter PDB_ tracing:

* Output capture in other tests is not affected.
* Any prior test output that has already been captured and will be processed as
  such.
* Any later output produced within the same test will not be captured and will
  instead get sent directly to ``sys.stdout``. Note that this holds true even
  for test output occurring after you exit the interactive PDB_ tracing session
  and continue with the regular test run.

.. versionadded: 2.4.0

Since pytest version 2.4.0 you can also use the native Python
``import pdb;pdb.set_trace()`` call to enter PDB_ tracing without having to use
the ``pytest.set_trace()`` wrapper or explicitly disable pytest's output
capturing via ``py.test -s``.

.. _durations:

Profiling test execution duration
-------------------------------------

.. versionadded: 2.2

To get a list of the slowest 10 test durations::

    py.test --durations=10


Creating JUnitXML format files
----------------------------------------------------

To create result files which can be read by Hudson_ or other Continuous
integration servers, use this invocation::

    py.test --junitxml=path

to create an XML file at ``path``.

record_xml_property
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

.. versionadded:: 2.8

If you want to log additional information for a test, you can use the
``record_xml_property`` fixture:

.. code-block:: python

    def test_function(record_xml_property):
        record_xml_property("example_key", 1)
        assert 0

This will add an extra property ``example_key="1"`` to the generated
``testcase`` tag:

.. code-block:: xml

    <testcase classname="test_function" file="test_function.py" line="0" name="test_function" time="0.0009">
      <properties>
        <property name="example_key" value="1" />
      </properties>
    </testcase>

.. warning::

    This is an experimental feature, and its interface might be replaced
    by something more powerful and general in future versions. The
    functionality per-se will be kept, however.

    Currently it does not work when used with the ``pytest-xdist`` plugin.

    Also please note that using this feature will break any schema verification.
    This might be a problem when used with some CI servers.

Creating resultlog format files
----------------------------------------------------

To create plain-text machine-readable result files you can issue::

    py.test --resultlog=path

and look at the content at the ``path`` location.  Such files are used e.g.
by the `PyPy-test`_ web page to show test results over several revisions.

.. _`PyPy-test`: http://buildbot.pypy.org/summary


Sending test report to online pastebin service
-----------------------------------------------------

**Creating a URL for each test failure**::

    py.test --pastebin=failed

This will submit test run information to a remote Paste service and
provide a URL for each failure.  You may select tests as usual or add
for example ``-x`` if you only want to send one particular failure.

**Creating a URL for a whole test session log**::

    py.test --pastebin=all

Currently only pasting to the http://bpaste.net service is implemented.

Disabling plugins
-----------------

To disable loading specific plugins at invocation time, use the ``-p`` option
together with the prefix ``no:``.

Example: to disable loading the plugin ``doctest``, which is responsible for
executing doctest tests from text files, invoke py.test like this::

    py.test -p no:doctest

.. _`pytest.main-usage`:

Calling pytest from Python code
----------------------------------------------------

.. versionadded:: 2.0

You can invoke ``pytest`` from Python code directly::

    pytest.main()

this acts as if you would call "py.test" from the command line.
It will not raise ``SystemExit`` but return the exitcode instead.
You can pass in options and arguments::

    pytest.main(['-x', 'mytestdir'])

or pass in a string::

    pytest.main("-x mytestdir")

You can specify additional plugins to ``pytest.main``::

    # content of myinvoke.py
    import pytest
    class MyPlugin:
        def pytest_sessionfinish(self):
            print("*** test run reporting finishing")

    pytest.main("-qq", plugins=[MyPlugin()])

Running it will show that ``MyPlugin`` was added and its
hook was invoked::

    $ python myinvoke.py
    *** test run reporting finishing
    

.. include:: links.inc
