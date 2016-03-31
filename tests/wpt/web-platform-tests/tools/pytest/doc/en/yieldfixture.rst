.. _yieldfixture:

Fixture functions using "yield" / context manager integration
---------------------------------------------------------------

.. versionadded:: 2.4

.. regendoc:wipe

pytest-2.4 allows fixture functions to seamlessly use a ``yield`` instead 
of a ``return`` statement to provide a fixture value while otherwise
fully supporting all other fixture features.

Let's look at a simple standalone-example using the ``yield`` syntax::

    # content of test_yield.py
    
    import pytest

    @pytest.yield_fixture
    def passwd():
        print ("\nsetup before yield")
        f = open("/etc/passwd")
        yield f.readlines()
        print ("teardown after yield")
        f.close()

    def test_has_lines(passwd):
        print ("test called")
        assert passwd

In contrast to :ref:`finalization through registering callbacks
<finalization>`, our fixture function used a ``yield``
statement to provide the lines of the ``/etc/passwd`` file.  
The code after the ``yield`` statement serves as the teardown code, 
avoiding the indirection of registering a teardown callback function.   

Let's run it with output capturing disabled::

    $ py.test -q -s test_yield.py
    
    setup before yield
    test called
    .teardown after yield
    
    1 passed in 0.12 seconds

We can also seamlessly use the new syntax with ``with`` statements.
Let's simplify the above ``passwd`` fixture::

    # content of test_yield2.py
    
    import pytest

    @pytest.yield_fixture
    def passwd():
        with open("/etc/passwd") as f:
            yield f.readlines()

    def test_has_lines(passwd):
        assert len(passwd) >= 1

The file ``f`` will be closed after the test finished execution
because the Python ``file`` object supports finalization when
the ``with`` statement ends. 

Note that the yield fixture form supports all other fixture
features such as ``scope``, ``params``, etc., thus changing existing
fixture functions to use ``yield`` is straightforward.

.. note::

    While the ``yield`` syntax is similar to what
    :py:func:`contextlib.contextmanager` decorated functions
    provide, with pytest fixture functions the part after the
    "yield" will always be invoked, independently from the
    exception status of the test function which uses the fixture.
    This behaviour makes sense if you consider that many different
    test functions might use a module or session scoped fixture.


Discussion and future considerations / feedback
++++++++++++++++++++++++++++++++++++++++++++++++++++

There are some topics that are worth mentioning:

- usually ``yield`` is used for producing multiple values.
  But fixture functions can only yield exactly one value.
  Yielding a second fixture value will get you an error.
  It's possible we can evolve pytest to allow for producing
  multiple values as an alternative to current parametrization.
  For now, you can just use the normal
  :ref:`fixture parametrization <fixture-parametrize>`
  mechanisms together with ``yield``-style fixtures.

- lastly ``yield`` introduces more than one way to write
  fixture functions, so what's the obvious way to a newcomer?

If you want to feedback or participate in discussion of the above
topics, please join our :ref:`contact channels`, you are most welcome.
