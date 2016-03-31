
The writing and reporting of assertions in tests
==================================================

.. _`assertfeedback`:
.. _`assert with the assert statement`:
.. _`assert`:


Asserting with the ``assert`` statement
---------------------------------------------------------

``pytest`` allows you to use the standard python ``assert`` for verifying
expectations and values in Python tests.  For example, you can write the
following::

    # content of test_assert1.py
    def f():
        return 3

    def test_function():
        assert f() == 4

to assert that your function returns a certain value. If this assertion fails
you will see the return value of the function call::

    $ py.test test_assert1.py
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR, inifile: 
    collected 1 items
    
    test_assert1.py F
    
    ======= FAILURES ========
    _______ test_function ________
    
        def test_function():
    >       assert f() == 4
    E       assert 3 == 4
    E        +  where 3 = f()
    
    test_assert1.py:5: AssertionError
    ======= 1 failed in 0.12 seconds ========

``pytest`` has support for showing the values of the most common subexpressions
including calls, attributes, comparisons, and binary and unary
operators. (See :ref:`tbreportdemo`).  This allows you to use the
idiomatic python constructs without boilerplate code while not losing
introspection information.

However, if you specify a message with the assertion like this::

    assert a % 2 == 0, "value was odd, should be even"

then no assertion introspection takes places at all and the message
will be simply shown in the traceback.

See :ref:`assert-details` for more information on assertion introspection.

.. _`assertraises`:

Assertions about expected exceptions
------------------------------------------

In order to write assertions about raised exceptions, you can use
``pytest.raises`` as a context manager like this::

    import pytest

    def test_zero_division():
        with pytest.raises(ZeroDivisionError):
            1 / 0

and if you need to have access to the actual exception info you may use::

    def test_recursion_depth():
        with pytest.raises(RuntimeError) as excinfo:
            def f():
                f()
            f()
        assert 'maximum recursion' in str(excinfo.value)

``excinfo`` is a ``ExceptionInfo`` instance, which is a wrapper around
the actual exception raised.  The main attributes of interest are
``.type``, ``.value`` and ``.traceback``.

If you want to write test code that works on Python 2.4 as well,
you may also use two other ways to test for an expected exception::

    pytest.raises(ExpectedException, func, *args, **kwargs)
    pytest.raises(ExpectedException, "func(*args, **kwargs)")

both of which execute the specified function with args and kwargs and
asserts that the given ``ExpectedException`` is raised.  The reporter will
provide you with helpful output in case of failures such as *no
exception* or *wrong exception*.

Note that it is also possible to specify a "raises" argument to
``pytest.mark.xfail``, which checks that the test is failing in a more
specific way than just having any exception raised::

    @pytest.mark.xfail(raises=IndexError)
    def test_f():
        f()

Using ``pytest.raises`` is likely to be better for cases where you are testing
exceptions your own code is deliberately raising, whereas using
``@pytest.mark.xfail`` with a check function is probably better for something
like documenting unfixed bugs (where the test describes what "should" happen)
or bugs in dependencies.


.. _`assertwarns`:

Assertions about expected warnings
-----------------------------------------

.. versionadded:: 2.8

You can check that code raises a particular warning using
:ref:`pytest.warns <warns>`.


.. _newreport:

Making use of context-sensitive comparisons
-------------------------------------------------

.. versionadded:: 2.0

``pytest`` has rich support for providing context-sensitive information
when it encounters comparisons.  For example::

    # content of test_assert2.py

    def test_set_comparison():
        set1 = set("1308")
        set2 = set("8035")
        assert set1 == set2

if you run this module::

    $ py.test test_assert2.py
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR, inifile: 
    collected 1 items
    
    test_assert2.py F
    
    ======= FAILURES ========
    _______ test_set_comparison ________
    
        def test_set_comparison():
            set1 = set("1308")
            set2 = set("8035")
    >       assert set1 == set2
    E       assert set(['0', '1', '3', '8']) == set(['0', '3', '5', '8'])
    E         Extra items in the left set:
    E         '1'
    E         Extra items in the right set:
    E         '5'
    E         Use -v to get the full diff
    
    test_assert2.py:5: AssertionError
    ======= 1 failed in 0.12 seconds ========

Special comparisons are done for a number of cases:

* comparing long strings: a context diff is shown
* comparing long sequences: first failing indices
* comparing dicts: different entries

See the :ref:`reporting demo <tbreportdemo>` for many more examples.

Defining your own assertion comparison
----------------------------------------------

It is possible to add your own detailed explanations by implementing
the ``pytest_assertrepr_compare`` hook.

.. autofunction:: _pytest.hookspec.pytest_assertrepr_compare

As an example consider adding the following hook in a conftest.py which
provides an alternative explanation for ``Foo`` objects::

   # content of conftest.py
   from test_foocompare import Foo
   def pytest_assertrepr_compare(op, left, right):
       if isinstance(left, Foo) and isinstance(right, Foo) and op == "==":
           return ['Comparing Foo instances:',
                   '   vals: %s != %s' % (left.val, right.val)]

now, given this test module::

   # content of test_foocompare.py
   class Foo:
       def __init__(self, val):
           self.val = val

       def __eq__(self, other):
           return self.val == other.val

   def test_compare():
       f1 = Foo(1)
       f2 = Foo(2)
       assert f1 == f2

you can run the test module and get the custom output defined in
the conftest file::

   $ py.test -q test_foocompare.py
   F
   ======= FAILURES ========
   _______ test_compare ________
   
       def test_compare():
           f1 = Foo(1)
           f2 = Foo(2)
   >       assert f1 == f2
   E       assert Comparing Foo instances:
   E            vals: 1 != 2
   
   test_foocompare.py:11: AssertionError
   1 failed in 0.12 seconds

.. _assert-details:
.. _`assert introspection`:

Advanced assertion introspection
----------------------------------

.. versionadded:: 2.1


Reporting details about a failing assertion is achieved either by rewriting
assert statements before they are run or re-evaluating the assert expression and
recording the intermediate values. Which technique is used depends on the
location of the assert, ``pytest`` configuration, and Python version being used
to run ``pytest``.

By default, ``pytest`` rewrites assert statements in test modules.
Rewritten assert statements put introspection information into the assertion failure message.
``pytest`` only rewrites test modules directly discovered by its test collection process, so
asserts in supporting modules which are not themselves test modules will not be
rewritten.

.. note::

   ``pytest`` rewrites test modules on import. It does this by using an import
   hook to write a new pyc files. Most of the time this works transparently.
   However, if you are messing with import yourself, the import hook may
   interfere. If this is the case, simply use ``--assert=reinterp`` or
   ``--assert=plain``. Additionally, rewriting will fail silently if it cannot
   write new pycs, i.e. in a read-only filesystem or a zipfile.

If an assert statement has not been rewritten or the Python version is less than
2.6, ``pytest`` falls back on assert reinterpretation. In assert
reinterpretation, ``pytest`` walks the frame of the function containing the
assert statement to discover sub-expression results of the failing assert
statement. You can force ``pytest`` to always use assertion reinterpretation by
passing the ``--assert=reinterp`` option.

Assert reinterpretation has a caveat not present with assert rewriting: If
evaluating the assert expression has side effects you may get a warning that the
intermediate values could not be determined safely.  A common example of this
issue is an assertion which reads from a file::

        assert f.read() != '...'

If this assertion fails then the re-evaluation will probably succeed!
This is because ``f.read()`` will return an empty string when it is
called the second time during the re-evaluation.  However, it is
easy to rewrite the assertion and avoid any trouble::

        content = f.read()
        assert content != '...'

All assert introspection can be turned off by passing ``--assert=plain``.

For further information, Benjamin Peterson wrote up `Behind the scenes of pytest's new assertion rewriting <http://pybites.blogspot.com/2011/07/behind-scenes-of-pytests-new-assertion.html>`_.

.. versionadded:: 2.1
   Add assert rewriting as an alternate introspection technique.

.. versionchanged:: 2.1
   Introduce the ``--assert`` option. Deprecate ``--no-assert`` and
   ``--nomagic``.
