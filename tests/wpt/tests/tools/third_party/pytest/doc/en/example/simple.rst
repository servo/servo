

Basic patterns and examples
==========================================================

How to change command line options defaults
-------------------------------------------

It can be tedious to type the same series of command line options
every time you use ``pytest``.  For example, if you always want to see
detailed info on skipped and xfailed tests, as well as have terser "dot"
progress output, you can write it into a configuration file:

.. code-block:: ini

    # content of pytest.ini
    [pytest]
    addopts = -ra -q


Alternatively, you can set a ``PYTEST_ADDOPTS`` environment variable to add command
line options while the environment is in use:

.. code-block:: bash

    export PYTEST_ADDOPTS="-v"

Here's how the command-line is built in the presence of ``addopts`` or the environment variable:

.. code-block:: text

    <pytest.ini:addopts> $PYTEST_ADDOPTS <extra command-line arguments>

So if the user executes in the command-line:

.. code-block:: bash

    pytest -m slow

The actual command line executed is:

.. code-block:: bash

    pytest -ra -q -v -m slow

Note that as usual for other command-line applications, in case of conflicting options the last one wins, so the example
above will show verbose output because ``-v`` overwrites ``-q``.


.. _request example:

Pass different values to a test function, depending on command line options
----------------------------------------------------------------------------

.. regendoc:wipe

Suppose we want to write a test that depends on a command line option.
Here is a basic pattern to achieve this:

.. code-block:: python

    # content of test_sample.py
    def test_answer(cmdopt):
        if cmdopt == "type1":
            print("first")
        elif cmdopt == "type2":
            print("second")
        assert 0  # to see what was printed


For this to work we need to add a command line option and
provide the ``cmdopt`` through a :ref:`fixture function <fixture>`:

.. code-block:: python

    # content of conftest.py
    import pytest


    def pytest_addoption(parser):
        parser.addoption(
            "--cmdopt", action="store", default="type1", help="my option: type1 or type2"
        )


    @pytest.fixture
    def cmdopt(request):
        return request.config.getoption("--cmdopt")

Let's run this without supplying our new option:

.. code-block:: pytest

    $ pytest -q test_sample.py
    F                                                                    [100%]
    ================================= FAILURES =================================
    _______________________________ test_answer ________________________________

    cmdopt = 'type1'

        def test_answer(cmdopt):
            if cmdopt == "type1":
                print("first")
            elif cmdopt == "type2":
                print("second")
    >       assert 0  # to see what was printed
    E       assert 0

    test_sample.py:6: AssertionError
    --------------------------- Captured stdout call ---------------------------
    first
    ========================= short test summary info ==========================
    FAILED test_sample.py::test_answer - assert 0
    1 failed in 0.12s

And now with supplying a command line option:

.. code-block:: pytest

    $ pytest -q --cmdopt=type2
    F                                                                    [100%]
    ================================= FAILURES =================================
    _______________________________ test_answer ________________________________

    cmdopt = 'type2'

        def test_answer(cmdopt):
            if cmdopt == "type1":
                print("first")
            elif cmdopt == "type2":
                print("second")
    >       assert 0  # to see what was printed
    E       assert 0

    test_sample.py:6: AssertionError
    --------------------------- Captured stdout call ---------------------------
    second
    ========================= short test summary info ==========================
    FAILED test_sample.py::test_answer - assert 0
    1 failed in 0.12s

You can see that the command line option arrived in our test.

We could add simple validation for the input by listing the choices:

.. code-block:: python

    # content of conftest.py
    import pytest


    def pytest_addoption(parser):
        parser.addoption(
            "--cmdopt",
            action="store",
            default="type1",
            help="my option: type1 or type2",
            choices=("type1", "type2"),
        )

Now we'll get feedback on a bad argument:

.. code-block:: pytest

    $ pytest -q --cmdopt=type3
    ERROR: usage: pytest [options] [file_or_dir] [file_or_dir] [...]
    pytest: error: argument --cmdopt: invalid choice: 'type3' (choose from 'type1', 'type2')


If you need to provide more detailed error messages, you can use the
``type`` parameter and raise :exc:`pytest.UsageError`:

.. code-block:: python

    # content of conftest.py
    import pytest


    def type_checker(value):
        msg = "cmdopt must specify a numeric type as typeNNN"
        if not value.startswith("type"):
            raise pytest.UsageError(msg)
        try:
            int(value[4:])
        except ValueError:
            raise pytest.UsageError(msg)

        return value


    def pytest_addoption(parser):
        parser.addoption(
            "--cmdopt",
            action="store",
            default="type1",
            help="my option: type1 or type2",
            type=type_checker,
        )

This completes the basic pattern.  However, one often rather wants to
process command line options outside of the test and rather pass in
different or more complex objects.

Dynamically adding command line options
--------------------------------------------------------------

.. regendoc:wipe

Through :confval:`addopts` you can statically add command line
options for your project.  You can also dynamically modify
the command line arguments before they get processed:

.. code-block:: python

    # setuptools plugin
    import sys


    def pytest_load_initial_conftests(args):
        if "xdist" in sys.modules:  # pytest-xdist plugin
            import multiprocessing

            num = max(multiprocessing.cpu_count() / 2, 1)
            args[:] = ["-n", str(num)] + args

If you have the :pypi:`xdist plugin <pytest-xdist>` installed
you will now always perform test runs using a number
of subprocesses close to your CPU. Running in an empty
directory with the above conftest.py:

.. code-block:: pytest

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 0 items

    ========================== no tests ran in 0.12s ===========================

.. _`excontrolskip`:

Control skipping of tests according to command line option
--------------------------------------------------------------

.. regendoc:wipe

Here is a ``conftest.py`` file adding a ``--runslow`` command
line option to control skipping of ``pytest.mark.slow`` marked tests:

.. code-block:: python

    # content of conftest.py

    import pytest


    def pytest_addoption(parser):
        parser.addoption(
            "--runslow", action="store_true", default=False, help="run slow tests"
        )


    def pytest_configure(config):
        config.addinivalue_line("markers", "slow: mark test as slow to run")


    def pytest_collection_modifyitems(config, items):
        if config.getoption("--runslow"):
            # --runslow given in cli: do not skip slow tests
            return
        skip_slow = pytest.mark.skip(reason="need --runslow option to run")
        for item in items:
            if "slow" in item.keywords:
                item.add_marker(skip_slow)

We can now write a test module like this:

.. code-block:: python

    # content of test_module.py
    import pytest


    def test_func_fast():
        pass


    @pytest.mark.slow
    def test_func_slow():
        pass

and when running it will see a skipped "slow" test:

.. code-block:: pytest

    $ pytest -rs    # "-rs" means report details on the little 's'
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 2 items

    test_module.py .s                                                    [100%]

    ========================= short test summary info ==========================
    SKIPPED [1] test_module.py:8: need --runslow option to run
    ======================= 1 passed, 1 skipped in 0.12s =======================

Or run it including the ``slow`` marked test:

.. code-block:: pytest

    $ pytest --runslow
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 2 items

    test_module.py ..                                                    [100%]

    ============================ 2 passed in 0.12s =============================

.. _`__tracebackhide__`:

Writing well integrated assertion helpers
-----------------------------------------

.. regendoc:wipe

If you have a test helper function called from a test you can
use the ``pytest.fail`` marker to fail a test with a certain message.
The test support function will not show up in the traceback if you
set the ``__tracebackhide__`` option somewhere in the helper function.
Example:

.. code-block:: python

    # content of test_checkconfig.py
    import pytest


    def checkconfig(x):
        __tracebackhide__ = True
        if not hasattr(x, "config"):
            pytest.fail(f"not configured: {x}")


    def test_something():
        checkconfig(42)

The ``__tracebackhide__`` setting influences ``pytest`` showing
of tracebacks: the ``checkconfig`` function will not be shown
unless the ``--full-trace`` command line option is specified.
Let's run our little function:

.. code-block:: pytest

    $ pytest -q test_checkconfig.py
    F                                                                    [100%]
    ================================= FAILURES =================================
    ______________________________ test_something ______________________________

        def test_something():
    >       checkconfig(42)
    E       Failed: not configured: 42

    test_checkconfig.py:11: Failed
    ========================= short test summary info ==========================
    FAILED test_checkconfig.py::test_something - Failed: not configured: 42
    1 failed in 0.12s

If you only want to hide certain exceptions, you can set ``__tracebackhide__``
to a callable which gets the ``ExceptionInfo`` object. You can for example use
this to make sure unexpected exception types aren't hidden:

.. code-block:: python

    import operator

    import pytest


    class ConfigException(Exception):
        pass


    def checkconfig(x):
        __tracebackhide__ = operator.methodcaller("errisinstance", ConfigException)
        if not hasattr(x, "config"):
            raise ConfigException(f"not configured: {x}")


    def test_something():
        checkconfig(42)

This will avoid hiding the exception traceback on unrelated exceptions (i.e.
bugs in assertion helpers).


Detect if running from within a pytest run
--------------------------------------------------------------

.. regendoc:wipe

Usually it is a bad idea to make application code
behave differently if called from a test.  But if you
absolutely must find out if your application code is
running from a test you can do something like this:

.. code-block:: python

    # content of your_module.py


    _called_from_test = False

.. code-block:: python

    # content of conftest.py


    def pytest_configure(config):
        your_module._called_from_test = True

and then check for the ``your_module._called_from_test`` flag:

.. code-block:: python

    if your_module._called_from_test:
        # called from within a test run
        ...
    else:
        # called "normally"
        ...

accordingly in your application.

Adding info to test report header
--------------------------------------------------------------

.. regendoc:wipe

It's easy to present extra information in a ``pytest`` run:

.. code-block:: python

    # content of conftest.py


    def pytest_report_header(config):
        return "project deps: mylib-1.1"

which will add the string to the test header accordingly:

.. code-block:: pytest

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    project deps: mylib-1.1
    rootdir: /home/sweet/project
    collected 0 items

    ========================== no tests ran in 0.12s ===========================

.. regendoc:wipe

It is also possible to return a list of strings which will be considered as several
lines of information. You may consider ``config.getoption('verbose')`` in order to
display more information if applicable:

.. code-block:: python

    # content of conftest.py


    def pytest_report_header(config):
        if config.getoption("verbose") > 0:
            return ["info1: did you know that ...", "did you?"]

which will add info only when run with "--v":

.. code-block:: pytest

    $ pytest -v
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y -- $PYTHON_PREFIX/bin/python
    cachedir: .pytest_cache
    info1: did you know that ...
    did you?
    rootdir: /home/sweet/project
    collecting ... collected 0 items

    ========================== no tests ran in 0.12s ===========================

and nothing when run plainly:

.. code-block:: pytest

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 0 items

    ========================== no tests ran in 0.12s ===========================

Profiling test duration
--------------------------

.. regendoc:wipe

.. versionadded: 2.2

If you have a slow running large test suite you might want to find
out which tests are the slowest. Let's make an artificial test suite:

.. code-block:: python

    # content of test_some_are_slow.py
    import time


    def test_funcfast():
        time.sleep(0.1)


    def test_funcslow1():
        time.sleep(0.2)


    def test_funcslow2():
        time.sleep(0.3)

Now we can profile which test functions execute the slowest:

.. code-block:: pytest

    $ pytest --durations=3
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 3 items

    test_some_are_slow.py ...                                            [100%]

    =========================== slowest 3 durations ============================
    0.30s call     test_some_are_slow.py::test_funcslow2
    0.20s call     test_some_are_slow.py::test_funcslow1
    0.10s call     test_some_are_slow.py::test_funcfast
    ============================ 3 passed in 0.12s =============================

Incremental testing - test steps
---------------------------------------------------

.. regendoc:wipe

Sometimes you may have a testing situation which consists of a series
of test steps.  If one step fails it makes no sense to execute further
steps as they are all expected to fail anyway and their tracebacks
add no insight.  Here is a simple ``conftest.py`` file which introduces
an ``incremental`` marker which is to be used on classes:

.. code-block:: python

    # content of conftest.py

    from typing import Dict, Tuple

    import pytest

    # store history of failures per test class name and per index in parametrize (if parametrize used)
    _test_failed_incremental: Dict[str, Dict[Tuple[int, ...], str]] = {}


    def pytest_runtest_makereport(item, call):
        if "incremental" in item.keywords:
            # incremental marker is used
            if call.excinfo is not None:
                # the test has failed
                # retrieve the class name of the test
                cls_name = str(item.cls)
                # retrieve the index of the test (if parametrize is used in combination with incremental)
                parametrize_index = (
                    tuple(item.callspec.indices.values())
                    if hasattr(item, "callspec")
                    else ()
                )
                # retrieve the name of the test function
                test_name = item.originalname or item.name
                # store in _test_failed_incremental the original name of the failed test
                _test_failed_incremental.setdefault(cls_name, {}).setdefault(
                    parametrize_index, test_name
                )


    def pytest_runtest_setup(item):
        if "incremental" in item.keywords:
            # retrieve the class name of the test
            cls_name = str(item.cls)
            # check if a previous test has failed for this class
            if cls_name in _test_failed_incremental:
                # retrieve the index of the test (if parametrize is used in combination with incremental)
                parametrize_index = (
                    tuple(item.callspec.indices.values())
                    if hasattr(item, "callspec")
                    else ()
                )
                # retrieve the name of the first test function to fail for this class name and index
                test_name = _test_failed_incremental[cls_name].get(parametrize_index, None)
                # if name found, test has failed for the combination of class name & test name
                if test_name is not None:
                    pytest.xfail(f"previous test failed ({test_name})")


These two hook implementations work together to abort incremental-marked
tests in a class.  Here is a test module example:

.. code-block:: python

    # content of test_step.py

    import pytest


    @pytest.mark.incremental
    class TestUserHandling:
        def test_login(self):
            pass

        def test_modification(self):
            assert 0

        def test_deletion(self):
            pass


    def test_normal():
        pass

If we run this:

.. code-block:: pytest

    $ pytest -rx
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 4 items

    test_step.py .Fx.                                                    [100%]

    ================================= FAILURES =================================
    ____________________ TestUserHandling.test_modification ____________________

    self = <test_step.TestUserHandling object at 0xdeadbeef0001>

        def test_modification(self):
    >       assert 0
    E       assert 0

    test_step.py:11: AssertionError
    ================================ XFAILURES =================================
    ______________________ TestUserHandling.test_deletion ______________________

    item = <Function test_deletion>

        def pytest_runtest_setup(item):
            if "incremental" in item.keywords:
                # retrieve the class name of the test
                cls_name = str(item.cls)
                # check if a previous test has failed for this class
                if cls_name in _test_failed_incremental:
                    # retrieve the index of the test (if parametrize is used in combination with incremental)
                    parametrize_index = (
                        tuple(item.callspec.indices.values())
                        if hasattr(item, "callspec")
                        else ()
                    )
                    # retrieve the name of the first test function to fail for this class name and index
                    test_name = _test_failed_incremental[cls_name].get(parametrize_index, None)
                    # if name found, test has failed for the combination of class name & test name
                    if test_name is not None:
    >                   pytest.xfail(f"previous test failed ({test_name})")
    E                   _pytest.outcomes.XFailed: previous test failed (test_modification)

    conftest.py:47: XFailed
    ========================= short test summary info ==========================
    XFAIL test_step.py::TestUserHandling::test_deletion - reason: previous test failed (test_modification)
    ================== 1 failed, 2 passed, 1 xfailed in 0.12s ==================

We'll see that ``test_deletion`` was not executed because ``test_modification``
failed.  It is reported as an "expected failure".


Package/Directory-level fixtures (setups)
-------------------------------------------------------

If you have nested test directories, you can have per-directory fixture scopes
by placing fixture functions in a ``conftest.py`` file in that directory.
You can use all types of fixtures including :ref:`autouse fixtures
<autouse fixtures>` which are the equivalent of xUnit's setup/teardown
concept.  It's however recommended to have explicit fixture references in your
tests or test classes rather than relying on implicitly executing
setup/teardown functions, especially if they are far away from the actual tests.

Here is an example for making a ``db`` fixture available in a directory:

.. code-block:: python

    # content of a/conftest.py
    import pytest


    class DB:
        pass


    @pytest.fixture(scope="package")
    def db():
        return DB()

and then a test module in that directory:

.. code-block:: python

    # content of a/test_db.py
    def test_a1(db):
        assert 0, db  # to show value

another test module:

.. code-block:: python

    # content of a/test_db2.py
    def test_a2(db):
        assert 0, db  # to show value

and then a module in a sister directory which will not see
the ``db`` fixture:

.. code-block:: python

    # content of b/test_error.py
    def test_root(db):  # no db here, will error out
        pass

We can run this:

.. code-block:: pytest

    $ pytest
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 7 items

    a/test_db.py F                                                       [ 14%]
    a/test_db2.py F                                                      [ 28%]
    b/test_error.py E                                                    [ 42%]
    test_step.py .Fx.                                                    [100%]

    ================================== ERRORS ==================================
    _______________________ ERROR at setup of test_root ________________________
    file /home/sweet/project/b/test_error.py, line 1
      def test_root(db):  # no db here, will error out
    E       fixture 'db' not found
    >       available fixtures: cache, capfd, capfdbinary, caplog, capsys, capsysbinary, doctest_namespace, monkeypatch, pytestconfig, record_property, record_testsuite_property, record_xml_attribute, recwarn, tmp_path, tmp_path_factory, tmpdir, tmpdir_factory
    >       use 'pytest --fixtures [testpath]' for help on them.

    /home/sweet/project/b/test_error.py:1
    ================================= FAILURES =================================
    _________________________________ test_a1 __________________________________

    db = <conftest.DB object at 0xdeadbeef0002>

        def test_a1(db):
    >       assert 0, db  # to show value
    E       AssertionError: <conftest.DB object at 0xdeadbeef0002>
    E       assert 0

    a/test_db.py:2: AssertionError
    _________________________________ test_a2 __________________________________

    db = <conftest.DB object at 0xdeadbeef0002>

        def test_a2(db):
    >       assert 0, db  # to show value
    E       AssertionError: <conftest.DB object at 0xdeadbeef0002>
    E       assert 0

    a/test_db2.py:2: AssertionError
    ____________________ TestUserHandling.test_modification ____________________

    self = <test_step.TestUserHandling object at 0xdeadbeef0003>

        def test_modification(self):
    >       assert 0
    E       assert 0

    test_step.py:11: AssertionError
    ========================= short test summary info ==========================
    FAILED a/test_db.py::test_a1 - AssertionError: <conftest.DB object at 0x7...
    FAILED a/test_db2.py::test_a2 - AssertionError: <conftest.DB object at 0x...
    FAILED test_step.py::TestUserHandling::test_modification - assert 0
    ERROR b/test_error.py::test_root
    ============= 3 failed, 2 passed, 1 xfailed, 1 error in 0.12s ==============

The two test modules in the ``a`` directory see the same ``db`` fixture instance
while the one test in the sister-directory ``b`` doesn't see it.  We could of course
also define a ``db`` fixture in that sister directory's ``conftest.py`` file.
Note that each fixture is only instantiated if there is a test actually needing
it (unless you use "autouse" fixture which are always executed ahead of the first test
executing).


Post-process test reports / failures
---------------------------------------

If you want to postprocess test reports and need access to the executing
environment you can implement a hook that gets called when the test
"report" object is about to be created.  Here we write out all failing
test calls and also access a fixture (if it was used by the test) in
case you want to query/look at it during your post processing.  In our
case we just write some information out to a ``failures`` file:

.. code-block:: python

    # content of conftest.py

    import os.path

    import pytest


    @pytest.hookimpl(wrapper=True, tryfirst=True)
    def pytest_runtest_makereport(item, call):
        # execute all other hooks to obtain the report object
        rep = yield

        # we only look at actual failing test calls, not setup/teardown
        if rep.when == "call" and rep.failed:
            mode = "a" if os.path.exists("failures") else "w"
            with open("failures", mode, encoding="utf-8") as f:
                # let's also access a fixture for the fun of it
                if "tmp_path" in item.fixturenames:
                    extra = " ({})".format(item.funcargs["tmp_path"])
                else:
                    extra = ""

                f.write(rep.nodeid + extra + "\n")

        return rep


if you then have failing tests:

.. code-block:: python

    # content of test_module.py
    def test_fail1(tmp_path):
        assert 0


    def test_fail2():
        assert 0

and run them:

.. code-block:: pytest

    $ pytest test_module.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 2 items

    test_module.py FF                                                    [100%]

    ================================= FAILURES =================================
    ________________________________ test_fail1 ________________________________

    tmp_path = PosixPath('PYTEST_TMPDIR/test_fail10')

        def test_fail1(tmp_path):
    >       assert 0
    E       assert 0

    test_module.py:2: AssertionError
    ________________________________ test_fail2 ________________________________

        def test_fail2():
    >       assert 0
    E       assert 0

    test_module.py:6: AssertionError
    ========================= short test summary info ==========================
    FAILED test_module.py::test_fail1 - assert 0
    FAILED test_module.py::test_fail2 - assert 0
    ============================ 2 failed in 0.12s =============================

you will have a "failures" file which contains the failing test ids:

.. code-block:: bash

    $ cat failures
    test_module.py::test_fail1 (PYTEST_TMPDIR/test_fail10)
    test_module.py::test_fail2

Making test result information available in fixtures
-----------------------------------------------------------

.. regendoc:wipe

If you want to make test result reports available in fixture finalizers
here is a little example implemented via a local plugin:

.. code-block:: python

    # content of conftest.py
    from typing import Dict
    import pytest
    from pytest import StashKey, CollectReport

    phase_report_key = StashKey[Dict[str, CollectReport]]()


    @pytest.hookimpl(wrapper=True, tryfirst=True)
    def pytest_runtest_makereport(item, call):
        # execute all other hooks to obtain the report object
        rep = yield

        # store test results for each phase of a call, which can
        # be "setup", "call", "teardown"
        item.stash.setdefault(phase_report_key, {})[rep.when] = rep

        return rep


    @pytest.fixture
    def something(request):
        yield
        # request.node is an "item" because we use the default
        # "function" scope
        report = request.node.stash[phase_report_key]
        if report["setup"].failed:
            print("setting up a test failed or skipped", request.node.nodeid)
        elif ("call" not in report) or report["call"].failed:
            print("executing test failed or skipped", request.node.nodeid)


if you then have failing tests:

.. code-block:: python

    # content of test_module.py

    import pytest


    @pytest.fixture
    def other():
        assert 0


    def test_setup_fails(something, other):
        pass


    def test_call_fails(something):
        assert 0


    def test_fail2():
        assert 0

and run it:

.. code-block:: pytest

    $ pytest -s test_module.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 3 items

    test_module.py Esetting up a test failed or skipped test_module.py::test_setup_fails
    Fexecuting test failed or skipped test_module.py::test_call_fails
    F

    ================================== ERRORS ==================================
    ____________________ ERROR at setup of test_setup_fails ____________________

        @pytest.fixture
        def other():
    >       assert 0
    E       assert 0

    test_module.py:7: AssertionError
    ================================= FAILURES =================================
    _____________________________ test_call_fails ______________________________

    something = None

        def test_call_fails(something):
    >       assert 0
    E       assert 0

    test_module.py:15: AssertionError
    ________________________________ test_fail2 ________________________________

        def test_fail2():
    >       assert 0
    E       assert 0

    test_module.py:19: AssertionError
    ========================= short test summary info ==========================
    FAILED test_module.py::test_call_fails - assert 0
    FAILED test_module.py::test_fail2 - assert 0
    ERROR test_module.py::test_setup_fails - assert 0
    ======================== 2 failed, 1 error in 0.12s ========================

You'll see that the fixture finalizers could use the precise reporting
information.

.. _pytest current test env:

``PYTEST_CURRENT_TEST`` environment variable
--------------------------------------------



Sometimes a test session might get stuck and there might be no easy way to figure out
which test got stuck, for example if pytest was run in quiet mode (``-q``) or you don't have access to the console
output. This is particularly a problem if the problem happens only sporadically, the famous "flaky" kind of tests.

``pytest`` sets the :envvar:`PYTEST_CURRENT_TEST` environment variable when running tests, which can be inspected
by process monitoring utilities or libraries like :pypi:`psutil` to discover which test got stuck if necessary:

.. code-block:: python

    import psutil

    for pid in psutil.pids():
        environ = psutil.Process(pid).environ()
        if "PYTEST_CURRENT_TEST" in environ:
            print(f'pytest process {pid} running: {environ["PYTEST_CURRENT_TEST"]}')

During the test session pytest will set ``PYTEST_CURRENT_TEST`` to the current test
:ref:`nodeid <nodeids>` and the current stage, which can be ``setup``, ``call``,
or ``teardown``.

For example, when running a single test function named ``test_foo`` from ``foo_module.py``,
``PYTEST_CURRENT_TEST`` will be set to:

#. ``foo_module.py::test_foo (setup)``
#. ``foo_module.py::test_foo (call)``
#. ``foo_module.py::test_foo (teardown)``

In that order.

.. note::

    The contents of ``PYTEST_CURRENT_TEST`` is meant to be human readable and the actual format
    can be changed between releases (even bug fixes) so it shouldn't be relied on for scripting
    or automation.

.. _freezing-pytest:

Freezing pytest
---------------

If you freeze your application using a tool like
`PyInstaller <https://pyinstaller.readthedocs.io>`_
in order to distribute it to your end-users, it is a good idea to also package
your test runner and run your tests using the frozen application. This way packaging
errors such as dependencies not being included into the executable can be detected early
while also allowing you to send test files to users so they can run them in their
machines, which can be useful to obtain more information about a hard to reproduce bug.

Fortunately recent ``PyInstaller`` releases already have a custom hook
for pytest, but if you are using another tool to freeze executables
such as ``cx_freeze`` or ``py2exe``, you can use ``pytest.freeze_includes()``
to obtain the full list of internal pytest modules. How to configure the tools
to find the internal modules varies from tool to tool, however.

Instead of freezing the pytest runner as a separate executable, you can make
your frozen program work as the pytest runner by some clever
argument handling during program startup. This allows you to
have a single executable, which is usually more convenient.
Please note that the mechanism for plugin discovery used by pytest
(setuptools entry points) doesn't work with frozen executables so pytest
can't find any third party plugins automatically. To include third party plugins
like ``pytest-timeout`` they must be imported explicitly and passed on to pytest.main.

.. code-block:: python

    # contents of app_main.py
    import sys

    import pytest_timeout  # Third party plugin

    if len(sys.argv) > 1 and sys.argv[1] == "--pytest":
        import pytest

        sys.exit(pytest.main(sys.argv[2:], plugins=[pytest_timeout]))
    else:
        # normal application execution: at this point argv can be parsed
        # by your argument-parsing library of choice as usual
        ...


This allows you to execute tests using the frozen
application with standard ``pytest`` command-line options:

.. code-block:: bash

    ./app_main --pytest --verbose --tb=long --junit=xml=results.xml test-suite/
