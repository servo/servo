.. _`skip and xfail`:

.. _skipping:

Skip and xfail: dealing with tests that can not succeed
=====================================================================

If you have test functions that cannot be run on certain platforms
or that you expect to fail you can mark them accordingly or you
may call helper functions during execution of setup or test functions.

A *skip* means that you expect your test to pass unless the environment
(e.g. wrong Python interpreter, missing dependency) prevents it to run.
And *xfail* means that your test can run but you expect it to fail
because there is an implementation problem.

``pytest`` counts and lists *skip* and *xfail* tests separately. Detailed
information about skipped/xfailed tests is not shown by default to avoid
cluttering the output.  You can use the ``-r`` option to see details
corresponding to the "short" letters shown in the test progress::

    py.test -rxs  # show extra info on skips and xfails

(See :ref:`how to change command line options defaults`)

.. _skipif:
.. _`condition booleans`:

Marking a test function to be skipped
-------------------------------------------

.. versionadded:: 2.9

The simplest way to skip a test function is to mark it with the ``skip`` decorator
which may be passed an optional ``reason``:

.. code-block:: python

    @pytest.mark.skip(reason="no way of currently testing this")
    def test_the_unknown():
        ...

``skipif``
~~~~~~~~~~

.. versionadded:: 2.0, 2.4

If you wish to skip something conditionally then you can use ``skipif`` instead.
Here is an example of marking a test function to be skipped
when run on a Python3.3 interpreter::

    import sys
    @pytest.mark.skipif(sys.version_info < (3,3),
                        reason="requires python3.3")
    def test_function():
        ...

During test function setup the condition ("sys.version_info >= (3,3)") is
checked.  If it evaluates to True, the test function will be skipped
with the specified reason.  Note that pytest enforces specifying a reason
in order to report meaningful "skip reasons" (e.g. when using ``-rs``).
If the condition is a string, it will be evaluated as python expression.

You can share skipif markers between modules.  Consider this test module::

    # content of test_mymodule.py

    import mymodule
    minversion = pytest.mark.skipif(mymodule.__versioninfo__ < (1,1),
                                    reason="at least mymodule-1.1 required")
    @minversion
    def test_function():
        ...

You can import it from another test module::

    # test_myothermodule.py
    from test_mymodule import minversion

    @minversion
    def test_anotherfunction():
        ...

For larger test suites it's usually a good idea to have one file
where you define the markers which you then consistently apply
throughout your test suite.

Alternatively, the pre pytest-2.4 way to specify :ref:`condition strings
<string conditions>` instead of booleans will remain fully supported in future
versions of pytest.  It couldn't be easily used for importing markers
between test modules so it's no longer advertised as the primary method.


Skip all test functions of a class or module
---------------------------------------------

You can use the ``skipif`` decorator (and any other marker) on classes::

    @pytest.mark.skipif(sys.platform == 'win32',
                        reason="does not run on windows")
    class TestPosixCalls:

        def test_function(self):
            "will not be setup or run under 'win32' platform"

If the condition is true, this marker will produce a skip result for
each of the test methods.

If you want to skip all test functions of a module, you must use
the ``pytestmark`` name on the global level:

.. code-block:: python

    # test_module.py
    pytestmark = pytest.mark.skipif(...)

If multiple "skipif" decorators are applied to a test function, it
will be skipped if any of the skip conditions is true.

.. _`whole class- or module level`: mark.html#scoped-marking

.. _xfail:

Mark a test function as expected to fail
-------------------------------------------------------

You can use the ``xfail`` marker to indicate that you
expect a test to fail::

    @pytest.mark.xfail
    def test_function():
        ...

This test will be run but no traceback will be reported
when it fails. Instead terminal reporting will list it in the
"expected to fail" (``XFAIL``) or "unexpectedly passing" (``XPASS``) sections.

``strict`` parameter
~~~~~~~~~~~~~~~~~~~~

.. versionadded:: 2.9

Both ``XFAIL`` and ``XPASS`` don't fail the test suite, unless the ``strict`` keyword-only
parameter is passed as ``True``:

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


``reason`` parameter
~~~~~~~~~~~~~~~~~~~~

As with skipif_ you can also mark your expectation of a failure
on a particular platform::

    @pytest.mark.xfail(sys.version_info >= (3,3),
                       reason="python3.3 api changes")
    def test_function():
        ...


``raises`` parameter
~~~~~~~~~~~~~~~~~~~~

If you want to be more specific as to why the test is failing, you can specify
a single exception, or a list of exceptions, in the ``raises`` argument.

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

This is specially useful for marking crashing tests for later inspection.


Ignoring xfail marks
~~~~~~~~~~~~~~~~~~~~

By specifying on the commandline::

    pytest --runxfail

you can force the running and reporting of an ``xfail`` marked test
as if it weren't marked at all.

Examples
~~~~~~~~

Here is a simple test file with the several usages:

.. literalinclude:: example/xfail_demo.py

Running it with the report-on-xfail option gives this output::

    example $ py.test -rx xfail_demo.py
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR/example, inifile: 
    collected 7 items
    
    xfail_demo.py xxxxxxx
    ======= short test summary info ========
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
    
    ======= 7 xfailed in 0.12 seconds ========

xfail signature summary
~~~~~~~~~~~~~~~~~~~~~~~

Here's the signature of the ``xfail`` marker, using Python 3 keyword-only
arguments syntax:

.. code-block:: python

    def xfail(condition=None, *, reason=None, raises=None, run=True, strict=False):



.. _`skip/xfail with parametrize`:

Skip/xfail with parametrize
---------------------------

It is possible to apply markers like skip and xfail to individual
test instances when using parametrize::

    import pytest

    @pytest.mark.parametrize(("n", "expected"), [
        (1, 2),
    pytest.mark.xfail((1, 0)),
	pytest.mark.xfail(reason="some bug")((1, 3)),
        (2, 3),
        (3, 4),
        (4, 5),
        pytest.mark.skipif("sys.version_info >= (3,0)")((10, 11)),
    ])
    def test_increment(n, expected):
        assert n + 1 == expected


Imperative xfail from within a test or setup function
------------------------------------------------------

If you cannot declare xfail- of skipif conditions at import
time you can also imperatively produce an according outcome
imperatively, in test or setup code::

    def test_function():
        if not valid_config():
            pytest.xfail("failing configuration (but should work)")
            # or
            pytest.skip("unsupported configuration")


Skipping on a missing import dependency
--------------------------------------------------

You can use the following import helper at module level
or within a test or test setup function::

    docutils = pytest.importorskip("docutils")

If ``docutils`` cannot be imported here, this will lead to a
skip outcome of the test.  You can also skip based on the
version number of a library::

    docutils = pytest.importorskip("docutils", minversion="0.3")

The version will be read from the specified
module's ``__version__`` attribute.


.. _string conditions:

specifying conditions as strings versus booleans
----------------------------------------------------------

Prior to pytest-2.4 the only way to specify skipif/xfail conditions was
to use strings::

    import sys
    @pytest.mark.skipif("sys.version_info >= (3,3)")
    def test_function():
        ...

During test function setup the skipif condition is evaluated by calling
``eval('sys.version_info >= (3,0)', namespace)``.  The namespace contains
all the module globals, and ``os`` and ``sys`` as a minimum.

Since pytest-2.4 `condition booleans`_ are considered preferable
because markers can then be freely imported between test modules.
With strings you need to import not only the marker but all variables
everything used by the marker, which violates encapsulation.

The reason for specifying the condition as a string was that ``pytest`` can
report a summary of skip conditions based purely on the condition string.
With conditions as booleans you are required to specify a ``reason`` string.

Note that string conditions will remain fully supported and you are free
to use them if you have no need for cross-importing markers.

The evaluation of a condition string in ``pytest.mark.skipif(conditionstring)``
or ``pytest.mark.xfail(conditionstring)`` takes place in a namespace
dictionary which is constructed as follows:

* the namespace is initialized by putting the ``sys`` and ``os`` modules
  and the pytest ``config`` object into it.

* updated with the module globals of the test function for which the
  expression is applied.

The pytest ``config`` object allows you to skip based on a test
configuration value which you might have added::

    @pytest.mark.skipif("not config.getvalue('db')")
    def test_function(...):
        ...

The equivalent with "boolean conditions" is::

    @pytest.mark.skipif(not pytest.config.getvalue("db"),
                        reason="--db was not specified")
    def test_function(...):
        pass

.. note::

    You cannot use ``pytest.config.getvalue()`` in code
    imported before py.test's argument parsing takes place.  For example,
    ``conftest.py`` files are imported before command line parsing and thus
    ``config.getvalue()`` will not execute correctly.
