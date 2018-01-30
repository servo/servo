
.. _`mark examples`:

Working with custom markers
=================================================

Here are some example using the :ref:`mark` mechanism.

Marking test functions and selecting them for a run
----------------------------------------------------

You can "mark" a test function with custom metadata like this::

    # content of test_server.py

    import pytest
    @pytest.mark.webtest
    def test_send_http():
        pass # perform some webtest test for your app
    def test_something_quick():
        pass
    def test_another():
        pass
    class TestClass(object):
        def test_method(self):
            pass

.. versionadded:: 2.2

You can then restrict a test run to only run tests marked with ``webtest``::

    $ pytest -v -m webtest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR, inifile:
    collecting ... collected 4 items
    
    test_server.py::test_send_http PASSED                                [100%]
    
    ============================ 3 tests deselected ============================
    ================== 1 passed, 3 deselected in 0.12 seconds ==================

Or the inverse, running all tests except the webtest ones::

    $ pytest -v -m "not webtest"
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR, inifile:
    collecting ... collected 4 items
    
    test_server.py::test_something_quick PASSED                          [ 33%]
    test_server.py::test_another PASSED                                  [ 66%]
    test_server.py::TestClass::test_method PASSED                        [100%]
    
    ============================ 1 tests deselected ============================
    ================== 3 passed, 1 deselected in 0.12 seconds ==================

Selecting tests based on their node ID
--------------------------------------

You can provide one or more :ref:`node IDs <node-id>` as positional
arguments to select only specified tests. This makes it easy to select
tests based on their module, class, method, or function name::

    $ pytest -v test_server.py::TestClass::test_method
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR, inifile:
    collecting ... collected 1 item
    
    test_server.py::TestClass::test_method PASSED                        [100%]
    
    ========================= 1 passed in 0.12 seconds =========================

You can also select on the class::

    $ pytest -v test_server.py::TestClass
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR, inifile:
    collecting ... collected 1 item
    
    test_server.py::TestClass::test_method PASSED                        [100%]
    
    ========================= 1 passed in 0.12 seconds =========================

Or select multiple nodes::

  $ pytest -v test_server.py::TestClass test_server.py::test_send_http
  =========================== test session starts ============================
  platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
  cachedir: .cache
  rootdir: $REGENDOC_TMPDIR, inifile:
  collecting ... collected 2 items
  
  test_server.py::TestClass::test_method PASSED                        [ 50%]
  test_server.py::test_send_http PASSED                                [100%]
  
  ========================= 2 passed in 0.12 seconds =========================

.. _node-id:

.. note::

    Node IDs are of the form ``module.py::class::method`` or
    ``module.py::function``.  Node IDs control which tests are
    collected, so ``module.py::class`` will select all test methods
    on the class.  Nodes are also created for each parameter of a
    parametrized fixture or test, so selecting a parametrized test
    must include the parameter value, e.g.
    ``module.py::function[param]``.

    Node IDs for failing tests are displayed in the test summary info
    when running pytest with the ``-rf`` option.  You can also
    construct Node IDs from the output of ``pytest --collectonly``.

Using ``-k expr`` to select tests based on their name
-------------------------------------------------------

.. versionadded: 2.0/2.3.4

You can use the ``-k`` command line option to specify an expression
which implements a substring match on the test names instead of the
exact match on markers that ``-m`` provides.  This makes it easy to
select tests based on their names::

    $ pytest -v -k http  # running with the above defined example module
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR, inifile:
    collecting ... collected 4 items
    
    test_server.py::test_send_http PASSED                                [100%]
    
    ============================ 3 tests deselected ============================
    ================== 1 passed, 3 deselected in 0.12 seconds ==================

And you can also run all tests except the ones that match the keyword::

    $ pytest -k "not send_http" -v
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR, inifile:
    collecting ... collected 4 items
    
    test_server.py::test_something_quick PASSED                          [ 33%]
    test_server.py::test_another PASSED                                  [ 66%]
    test_server.py::TestClass::test_method PASSED                        [100%]
    
    ============================ 1 tests deselected ============================
    ================== 3 passed, 1 deselected in 0.12 seconds ==================

Or to select "http" and "quick" tests::

    $ pytest -k "http or quick" -v
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y -- $PYTHON_PREFIX/bin/python3.5
    cachedir: .cache
    rootdir: $REGENDOC_TMPDIR, inifile:
    collecting ... collected 4 items
    
    test_server.py::test_send_http PASSED                                [ 50%]
    test_server.py::test_something_quick PASSED                          [100%]
    
    ============================ 2 tests deselected ============================
    ================== 2 passed, 2 deselected in 0.12 seconds ==================

.. note::

    If you are using expressions such as ``"X and Y"`` then both ``X`` and ``Y``
    need to be simple non-keyword names. For example, ``"pass"`` or ``"from"``
    will result in SyntaxErrors because ``"-k"`` evaluates the expression using
    Python's `eval`_ function.

.. _`eval`: https://docs.python.org/3.6/library/functions.html#eval


    However, if the ``"-k"`` argument is a simple string, no such restrictions
    apply. Also ``"-k 'not STRING'"`` has no restrictions.  You can also
    specify numbers like ``"-k 1.3"`` to match tests which are parametrized
    with the float ``"1.3"``.

Registering markers
-------------------------------------

.. versionadded:: 2.2

.. ini-syntax for custom markers:

Registering markers for your test suite is simple::

    # content of pytest.ini
    [pytest]
    markers =
        webtest: mark a test as a webtest.

You can ask which markers exist for your test suite - the list includes our just defined ``webtest`` markers::

    $ pytest --markers
    @pytest.mark.webtest: mark a test as a webtest.
    
    @pytest.mark.skip(reason=None): skip the given test function with an optional reason. Example: skip(reason="no way of currently testing this") skips the test.
    
    @pytest.mark.skipif(condition): skip the given test function if eval(condition) results in a True value.  Evaluation happens within the module global context. Example: skipif('sys.platform == "win32"') skips the test if we are on the win32 platform. see http://pytest.org/latest/skipping.html
    
    @pytest.mark.xfail(condition, reason=None, run=True, raises=None, strict=False): mark the test function as an expected failure if eval(condition) has a True value. Optionally specify a reason for better reporting and run=False if you don't even want to execute the test function. If only specific exception(s) are expected, you can list them in raises, and if the test fails in other ways, it will be reported as a true failure. See http://pytest.org/latest/skipping.html
    
    @pytest.mark.parametrize(argnames, argvalues): call a test function multiple times passing in different arguments in turn. argvalues generally needs to be a list of values if argnames specifies only one name or a list of tuples of values if argnames specifies multiple names. Example: @parametrize('arg1', [1,2]) would lead to two calls of the decorated test function, one with arg1=1 and another with arg1=2.see http://pytest.org/latest/parametrize.html for more info and examples.
    
    @pytest.mark.usefixtures(fixturename1, fixturename2, ...): mark tests as needing all of the specified fixtures. see http://pytest.org/latest/fixture.html#usefixtures 
    
    @pytest.mark.tryfirst: mark a hook implementation function such that the plugin machinery will try to call it first/as early as possible.
    
    @pytest.mark.trylast: mark a hook implementation function such that the plugin machinery will try to call it last/as late as possible.
    

For an example on how to add and work with markers from a plugin, see
:ref:`adding a custom marker from a plugin`.

.. note::

    It is recommended to explicitly register markers so that:

    * There is one place in your test suite defining your markers

    * Asking for existing markers via ``pytest --markers`` gives good output

    * Typos in function markers are treated as an error if you use
      the ``--strict`` option. 

.. _`scoped-marking`:

Marking whole classes or modules
----------------------------------------------------

You may use ``pytest.mark`` decorators with classes to apply markers to all of
its test methods::

    # content of test_mark_classlevel.py
    import pytest
    @pytest.mark.webtest
    class TestClass(object):
        def test_startup(self):
            pass
        def test_startup_and_more(self):
            pass

This is equivalent to directly applying the decorator to the
two test functions.

To remain backward-compatible with Python 2.4 you can also set a
``pytestmark`` attribute on a TestClass like this::

    import pytest

    class TestClass(object):
        pytestmark = pytest.mark.webtest

or if you need to use multiple markers you can use a list::

    import pytest

    class TestClass(object):
        pytestmark = [pytest.mark.webtest, pytest.mark.slowtest]

You can also set a module level marker::

    import pytest
    pytestmark = pytest.mark.webtest

in which case it will be applied to all functions and
methods defined in the module.

.. _`marking individual tests when using parametrize`:

Marking individual tests when using parametrize
-----------------------------------------------

When using parametrize, applying a mark will make it apply
to each individual test. However it is also possible to
apply a marker to an individual test instance::

    import pytest

    @pytest.mark.foo
    @pytest.mark.parametrize(("n", "expected"), [
        (1, 2),
        pytest.mark.bar((1, 3)),
        (2, 3),
    ])
    def test_increment(n, expected):
         assert n + 1 == expected

In this example the mark "foo" will apply to each of the three
tests, whereas the "bar" mark is only applied to the second test.
Skip and xfail marks can also be applied in this way, see :ref:`skip/xfail with parametrize`.

.. note::

    If the data you are parametrizing happen to be single callables, you need to be careful
    when marking these items. `pytest.mark.xfail(my_func)` won't work because it's also the
    signature of a function being decorated. To resolve this ambiguity, you need to pass a
    reason argument:
    `pytest.mark.xfail(func_bar, reason="Issue#7")`.


.. _`adding a custom marker from a plugin`:

Custom marker and command line option to control test runs
----------------------------------------------------------

.. regendoc:wipe

Plugins can provide custom markers and implement specific behaviour
based on it. This is a self-contained example which adds a command
line option and a parametrized test function marker to run tests
specifies via named environments::

    # content of conftest.py

    import pytest
    def pytest_addoption(parser):
        parser.addoption("-E", action="store", metavar="NAME",
            help="only run tests matching the environment NAME.")

    def pytest_configure(config):
        # register an additional marker
        config.addinivalue_line("markers",
            "env(name): mark test to run only on named environment")

    def pytest_runtest_setup(item):
        envmarker = item.get_marker("env")
        if envmarker is not None:
            envname = envmarker.args[0]
            if envname != item.config.getoption("-E"):
                pytest.skip("test requires env %r" % envname)

A test file using this local plugin::

    # content of test_someenv.py

    import pytest
    @pytest.mark.env("stage1")
    def test_basic_db_operation():
        pass

and an example invocations specifying a different environment than what
the test needs::

    $ pytest -E stage2
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile:
    collected 1 item
    
    test_someenv.py s                                                    [100%]
    
    ======================== 1 skipped in 0.12 seconds =========================

and here is one that specifies exactly the environment needed::

    $ pytest -E stage1
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile:
    collected 1 item
    
    test_someenv.py .                                                    [100%]
    
    ========================= 1 passed in 0.12 seconds =========================

The ``--markers`` option always gives you a list of available markers::

    $ pytest --markers
    @pytest.mark.env(name): mark test to run only on named environment
    
    @pytest.mark.skip(reason=None): skip the given test function with an optional reason. Example: skip(reason="no way of currently testing this") skips the test.
    
    @pytest.mark.skipif(condition): skip the given test function if eval(condition) results in a True value.  Evaluation happens within the module global context. Example: skipif('sys.platform == "win32"') skips the test if we are on the win32 platform. see http://pytest.org/latest/skipping.html
    
    @pytest.mark.xfail(condition, reason=None, run=True, raises=None, strict=False): mark the test function as an expected failure if eval(condition) has a True value. Optionally specify a reason for better reporting and run=False if you don't even want to execute the test function. If only specific exception(s) are expected, you can list them in raises, and if the test fails in other ways, it will be reported as a true failure. See http://pytest.org/latest/skipping.html
    
    @pytest.mark.parametrize(argnames, argvalues): call a test function multiple times passing in different arguments in turn. argvalues generally needs to be a list of values if argnames specifies only one name or a list of tuples of values if argnames specifies multiple names. Example: @parametrize('arg1', [1,2]) would lead to two calls of the decorated test function, one with arg1=1 and another with arg1=2.see http://pytest.org/latest/parametrize.html for more info and examples.
    
    @pytest.mark.usefixtures(fixturename1, fixturename2, ...): mark tests as needing all of the specified fixtures. see http://pytest.org/latest/fixture.html#usefixtures 
    
    @pytest.mark.tryfirst: mark a hook implementation function such that the plugin machinery will try to call it first/as early as possible.
    
    @pytest.mark.trylast: mark a hook implementation function such that the plugin machinery will try to call it last/as late as possible.
    

.. _`passing callables to custom markers`:

Passing a callable to custom markers
--------------------------------------------

.. regendoc:wipe

Below is the config file that will be used in the next examples::

    # content of conftest.py
    import sys

    def pytest_runtest_setup(item):
        marker = item.get_marker('my_marker')
        if marker is not None:
            for info in marker:
                print('Marker info name={} args={} kwars={}'.format(info.name, info.args, info.kwargs))
                sys.stdout.flush()

A custom marker can have its argument set, i.e. ``args`` and ``kwargs`` properties, defined by either invoking it as a callable or using ``pytest.mark.MARKER_NAME.with_args``. These two methods achieve the same effect most of the time.

However, if there is a callable as the single positional argument with no keyword arguments, using the ``pytest.mark.MARKER_NAME(c)`` will not pass ``c`` as a positional argument but decorate ``c`` with the custom marker (see :ref:`MarkDecorator <mark>`). Fortunately, ``pytest.mark.MARKER_NAME.with_args`` comes to the rescue::

    # content of test_custom_marker.py
    import pytest

    def hello_world(*args, **kwargs):
        return 'Hello World'

    @pytest.mark.my_marker.with_args(hello_world)
    def test_with_args():
        pass

The output is as follows::

    $ pytest -q -s
    Marker info name=my_marker args=(<function hello_world at 0xdeadbeef>,) kwars={}
    .                                                                    [100%]
    1 passed in 0.12 seconds

We can see that the custom marker has its argument set extended with the function ``hello_world``. This is the key difference between creating a custom marker as a callable, which invokes ``__call__`` behind the scenes, and using ``with_args``.


Reading markers which were set from multiple places
----------------------------------------------------

.. versionadded: 2.2.2

.. regendoc:wipe

If you are heavily using markers in your test suite you may encounter the case where a marker is applied several times to a test function.  From plugin
code you can read over all such settings.  Example::

    # content of test_mark_three_times.py
    import pytest
    pytestmark = pytest.mark.glob("module", x=1)

    @pytest.mark.glob("class", x=2)
    class TestClass(object):
        @pytest.mark.glob("function", x=3)
        def test_something(self):
            pass

Here we have the marker "glob" applied three times to the same
test function.  From a conftest file we can read it like this::

    # content of conftest.py
    import sys

    def pytest_runtest_setup(item):
        g = item.get_marker("glob")
        if g is not None:
            for info in g:
                print ("glob args=%s kwargs=%s" %(info.args, info.kwargs))
                sys.stdout.flush()

Let's run this without capturing output and see what we get::

    $ pytest -q -s
    glob args=('function',) kwargs={'x': 3}
    glob args=('class',) kwargs={'x': 2}
    glob args=('module',) kwargs={'x': 1}
    .                                                                    [100%]
    1 passed in 0.12 seconds

marking platform specific tests with pytest
--------------------------------------------------------------

.. regendoc:wipe

Consider you have a test suite which marks tests for particular platforms,
namely ``pytest.mark.darwin``, ``pytest.mark.win32`` etc. and you
also have tests that run on all platforms and have no specific
marker.  If you now want to have a way to only run the tests
for your particular platform, you could use the following plugin::

    # content of conftest.py
    #
    import sys
    import pytest

    ALL = set("darwin linux win32".split())

    def pytest_runtest_setup(item):
        if isinstance(item, item.Function):
            plat = sys.platform
            if not item.get_marker(plat):
                if ALL.intersection(item.keywords):
                    pytest.skip("cannot run on platform %s" %(plat))

then tests will be skipped if they were specified for a different platform.
Let's do a little test file to show how this looks like::

    # content of test_plat.py

    import pytest

    @pytest.mark.darwin
    def test_if_apple_is_evil():
        pass

    @pytest.mark.linux
    def test_if_linux_works():
        pass

    @pytest.mark.win32
    def test_if_win32_crashes():
        pass

    def test_runs_everywhere():
        pass

then you will see two tests skipped and two executed tests as expected::

    $ pytest -rs # this option reports skip reasons
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile:
    collected 4 items
    
    test_plat.py s.s.                                                    [100%]
    ========================= short test summary info ==========================
    SKIP [2] $REGENDOC_TMPDIR/conftest.py:13: cannot run on platform linux
    
    =================== 2 passed, 2 skipped in 0.12 seconds ====================

Note that if you specify a platform via the marker-command line option like this::

    $ pytest -m linux
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR, inifile:
    collected 4 items
    
    test_plat.py .                                                       [100%]
    
    ============================ 3 tests deselected ============================
    ================== 1 passed, 3 deselected in 0.12 seconds ==================

then the unmarked-tests will not be run.  It is thus a way to restrict the run to the specific tests.

Automatically adding markers based on test names
--------------------------------------------------------

.. regendoc:wipe

If you a test suite where test function names indicate a certain
type of test, you can implement a hook that automatically defines
markers so that you can use the ``-m`` option with it. Let's look
at this test module::

    # content of test_module.py

    def test_interface_simple():
        assert 0

    def test_interface_complex():
        assert 0

    def test_event_simple():
        assert 0

    def test_something_else():
        assert 0

We want to dynamically define two markers and can do it in a
``conftest.py`` plugin::

    # content of conftest.py

    import pytest
    def pytest_collection_modifyitems(items):
        for item in items:
            if "interface" in item.nodeid:
                item.add_marker(pytest.mark.interface)
            elif "event" in item.nodeid:
                item.add_marker(pytest.mark.event)

We can now use the ``-m option`` to select one set::

  $ pytest -m interface --tb=short
  =========================== test session starts ============================
  platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
  rootdir: $REGENDOC_TMPDIR, inifile:
  collected 4 items
  
  test_module.py FF                                                    [100%]
  
  ================================= FAILURES =================================
  __________________________ test_interface_simple ___________________________
  test_module.py:3: in test_interface_simple
      assert 0
  E   assert 0
  __________________________ test_interface_complex __________________________
  test_module.py:6: in test_interface_complex
      assert 0
  E   assert 0
  ============================ 2 tests deselected ============================
  ================== 2 failed, 2 deselected in 0.12 seconds ==================

or to select both "event" and "interface" tests::

  $ pytest -m "interface or event" --tb=short
  =========================== test session starts ============================
  platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
  rootdir: $REGENDOC_TMPDIR, inifile:
  collected 4 items
  
  test_module.py FFF                                                   [100%]
  
  ================================= FAILURES =================================
  __________________________ test_interface_simple ___________________________
  test_module.py:3: in test_interface_simple
      assert 0
  E   assert 0
  __________________________ test_interface_complex __________________________
  test_module.py:6: in test_interface_complex
      assert 0
  E   assert 0
  ____________________________ test_event_simple _____________________________
  test_module.py:9: in test_event_simple
      assert 0
  E   assert 0
  ============================ 1 tests deselected ============================
  ================== 3 failed, 1 deselected in 0.12 seconds ==================
