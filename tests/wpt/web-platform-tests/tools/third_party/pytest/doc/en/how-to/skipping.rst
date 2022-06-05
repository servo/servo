.. _`skip and xfail`:

.. _skipping:

How to use skip and xfail to deal with tests that cannot succeed
=================================================================

You can mark test functions that cannot be run on certain platforms
or that you expect to fail so pytest can deal with them accordingly and
present a summary of the test session, while keeping the test suite *green*.

A **skip** means that you expect your test to pass only if some conditions are met,
otherwise pytest should skip running the test altogether. Common examples are skipping
windows-only tests on non-windows platforms, or skipping tests that depend on an external
resource which is not available at the moment (for example a database).

An **xfail** means that you expect a test to fail for some reason.
A common example is a test for a feature not yet implemented, or a bug not yet fixed.
When a test passes despite being expected to fail (marked with ``pytest.mark.xfail``),
it's an **xpass** and will be reported in the test summary.

``pytest`` counts and lists *skip* and *xfail* tests separately. Detailed
information about skipped/xfailed tests is not shown by default to avoid
cluttering the output.  You can use the ``-r`` option to see details
corresponding to the "short" letters shown in the test progress:

.. code-block:: bash

    pytest -rxXs  # show extra info on xfailed, xpassed, and skipped tests

More details on the ``-r`` option can be found by running ``pytest -h``.

(See :ref:`how to change command line options defaults`)

.. _skipif:
.. _skip:
.. _`condition booleans`:

Skipping test functions
-----------------------



The simplest way to skip a test function is to mark it with the ``skip`` decorator
which may be passed an optional ``reason``:

.. code-block:: python

    @pytest.mark.skip(reason="no way of currently testing this")
    def test_the_unknown():
        ...


Alternatively, it is also possible to skip imperatively during test execution or setup
by calling the ``pytest.skip(reason)`` function:

.. code-block:: python

    def test_function():
        if not valid_config():
            pytest.skip("unsupported configuration")

The imperative method is useful when it is not possible to evaluate the skip condition
during import time.

It is also possible to skip the whole module using
``pytest.skip(reason, allow_module_level=True)`` at the module level:

.. code-block:: python

    import sys
    import pytest

    if not sys.platform.startswith("win"):
        pytest.skip("skipping windows-only tests", allow_module_level=True)


**Reference**: :ref:`pytest.mark.skip ref`

``skipif``
~~~~~~~~~~



If you wish to skip something conditionally then you can use ``skipif`` instead.
Here is an example of marking a test function to be skipped
when run on an interpreter earlier than Python3.6:

.. code-block:: python

    import sys


    @pytest.mark.skipif(sys.version_info < (3, 7), reason="requires python3.7 or higher")
    def test_function():
        ...

If the condition evaluates to ``True`` during collection, the test function will be skipped,
with the specified reason appearing in the summary when using ``-rs``.

You can share ``skipif`` markers between modules.  Consider this test module:

.. code-block:: python

    # content of test_mymodule.py
    import mymodule

    minversion = pytest.mark.skipif(
        mymodule.__versioninfo__ < (1, 1), reason="at least mymodule-1.1 required"
    )


    @minversion
    def test_function():
        ...

You can import the marker and reuse it in another test module:

.. code-block:: python

    # test_myothermodule.py
    from test_mymodule import minversion


    @minversion
    def test_anotherfunction():
        ...

For larger test suites it's usually a good idea to have one file
where you define the markers which you then consistently apply
throughout your test suite.

Alternatively, you can use :ref:`condition strings
<string conditions>` instead of booleans, but they can't be shared between modules easily
so they are supported mainly for backward compatibility reasons.

**Reference**: :ref:`pytest.mark.skipif ref`


Skip all test functions of a class or module
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

You can use the ``skipif`` marker (as any other marker) on classes:

.. code-block:: python

    @pytest.mark.skipif(sys.platform == "win32", reason="does not run on windows")
    class TestPosixCalls:
        def test_function(self):
            "will not be setup or run under 'win32' platform"

If the condition is ``True``, this marker will produce a skip result for
each of the test methods of that class.

If you want to skip all test functions of a module, you may use the
:globalvar:`pytestmark` global:

.. code-block:: python

    # test_module.py
    pytestmark = pytest.mark.skipif(...)

If multiple ``skipif`` decorators are applied to a test function, it
will be skipped if any of the skip conditions is true.

.. _`whole class- or module level`: mark.html#scoped-marking


Skipping files or directories
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Sometimes you may need to skip an entire file or directory, for example if the
tests rely on Python version-specific features or contain code that you do not
wish pytest to run. In this case, you must exclude the files and directories
from collection. Refer to :ref:`customizing-test-collection` for more
information.


Skipping on a missing import dependency
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

You can skip tests on a missing import by using :ref:`pytest.importorskip ref`
at module level, within a test, or test setup function.

.. code-block:: python

    docutils = pytest.importorskip("docutils")

If ``docutils`` cannot be imported here, this will lead to a skip outcome of
the test. You can also skip based on the version number of a library:

.. code-block:: python

    docutils = pytest.importorskip("docutils", minversion="0.3")

The version will be read from the specified
module's ``__version__`` attribute.

Summary
~~~~~~~

Here's a quick guide on how to skip tests in a module in different situations:

1. Skip all tests in a module unconditionally:

  .. code-block:: python

        pytestmark = pytest.mark.skip("all tests still WIP")

2. Skip all tests in a module based on some condition:

  .. code-block:: python

        pytestmark = pytest.mark.skipif(sys.platform == "win32", reason="tests for linux only")

3. Skip all tests in a module if some import is missing:

  .. code-block:: python

        pexpect = pytest.importorskip("pexpect")


.. _xfail:

XFail: mark test functions as expected to fail
----------------------------------------------

You can use the ``xfail`` marker to indicate that you
expect a test to fail:

.. code-block:: python

    @pytest.mark.xfail
    def test_function():
        ...

This test will run but no traceback will be reported when it fails. Instead, terminal
reporting will list it in the "expected to fail" (``XFAIL``) or "unexpectedly
passing" (``XPASS``) sections.

Alternatively, you can also mark a test as ``XFAIL`` from within the test or its setup function
imperatively:

.. code-block:: python

    def test_function():
        if not valid_config():
            pytest.xfail("failing configuration (but should work)")

.. code-block:: python

    def test_function2():
        import slow_module

        if slow_module.slow_function():
            pytest.xfail("slow_module taking too long")

These two examples illustrate situations where you don't want to check for a condition
at the module level, which is when a condition would otherwise be evaluated for marks.

This will make ``test_function`` ``XFAIL``. Note that no other code is executed after
the :func:`pytest.xfail` call, differently from the marker. That's because it is implemented
internally by raising a known exception.

**Reference**: :ref:`pytest.mark.xfail ref`


``condition`` parameter
~~~~~~~~~~~~~~~~~~~~~~~

If a test is only expected to fail under a certain condition, you can pass
that condition as the first parameter:

.. code-block:: python

    @pytest.mark.xfail(sys.platform == "win32", reason="bug in a 3rd party library")
    def test_function():
        ...

Note that you have to pass a reason as well (see the parameter description at
:ref:`pytest.mark.xfail ref`).

``reason`` parameter
~~~~~~~~~~~~~~~~~~~~

You can specify the motive of an expected failure with the ``reason`` parameter:

.. code-block:: python

    @pytest.mark.xfail(reason="known parser issue")
    def test_function():
        ...


``raises`` parameter
~~~~~~~~~~~~~~~~~~~~

If you want to be more specific as to why the test is failing, you can specify
a single exception, or a tuple of exceptions, in the ``raises`` argument.

.. code-block:: python

    @pytest.mark.xfail(raises=RuntimeError)
    def test_function():
        ...

Then the test will be reported as a regular failure if it fails with an
exception not mentioned in ``raises``.

``run`` parameter
~~~~~~~~~~~~~~~~~

If a test should be marked as xfail and reported as such but should not be
even executed, use the ``run`` parameter as ``False``:

.. code-block:: python

    @pytest.mark.xfail(run=False)
    def test_function():
        ...

This is specially useful for xfailing tests that are crashing the interpreter and should be
investigated later.

.. _`xfail strict tutorial`:

``strict`` parameter
~~~~~~~~~~~~~~~~~~~~

Both ``XFAIL`` and ``XPASS`` don't fail the test suite by default.
You can change this by setting the ``strict`` keyword-only parameter to ``True``:

.. code-block:: python

    @pytest.mark.xfail(strict=True)
    def test_function():
        ...


This will make ``XPASS`` ("unexpectedly passing") results from this test to fail the test suite.

You can change the default value of the ``strict`` parameter using the
``xfail_strict`` ini option:

.. code-block:: ini

    [pytest]
    xfail_strict=true


Ignoring xfail
~~~~~~~~~~~~~~

By specifying on the commandline:

.. code-block:: bash

    pytest --runxfail

you can force the running and reporting of an ``xfail`` marked test
as if it weren't marked at all. This also causes :func:`pytest.xfail` to produce no effect.

Examples
~~~~~~~~

Here is a simple test file with the several usages:

.. literalinclude:: /example/xfail_demo.py

Running it with the report-on-xfail option gives this output:

.. FIXME: Use $ instead of ! again to re-enable regendoc once it's fixed:
   https://github.com/pytest-dev/pytest/issues/8807

.. code-block:: pytest

    ! pytest -rx xfail_demo.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-6.x.y, py-1.x.y, pluggy-1.x.y
    cachedir: $PYTHON_PREFIX/.pytest_cache
    rootdir: $REGENDOC_TMPDIR/example
    collected 7 items

    xfail_demo.py xxxxxxx                                                [100%]

    ========================= short test summary info ==========================
    XFAIL xfail_demo.py::test_hello
    XFAIL xfail_demo.py::test_hello2
      reason: [NOTRUN]
    XFAIL xfail_demo.py::test_hello3
      condition: hasattr(os, 'sep')
    XFAIL xfail_demo.py::test_hello4
      bug 110
    XFAIL xfail_demo.py::test_hello5
      condition: pytest.__version__[0] != "17"
    XFAIL xfail_demo.py::test_hello6
      reason: reason
    XFAIL xfail_demo.py::test_hello7
    ============================ 7 xfailed in 0.12s ============================

.. _`skip/xfail with parametrize`:

Skip/xfail with parametrize
---------------------------

It is possible to apply markers like skip and xfail to individual
test instances when using parametrize:

.. code-block:: python

    import sys
    import pytest


    @pytest.mark.parametrize(
        ("n", "expected"),
        [
            (1, 2),
            pytest.param(1, 0, marks=pytest.mark.xfail),
            pytest.param(1, 3, marks=pytest.mark.xfail(reason="some bug")),
            (2, 3),
            (3, 4),
            (4, 5),
            pytest.param(
                10, 11, marks=pytest.mark.skipif(sys.version_info >= (3, 0), reason="py2k")
            ),
        ],
    )
    def test_increment(n, expected):
        assert n + 1 == expected
