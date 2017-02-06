
.. _`unittest.TestCase`:

Support for unittest.TestCase / Integration of fixtures
=====================================================================

.. _`unittest.py style`: http://docs.python.org/library/unittest.html

``pytest`` has support for running Python `unittest.py style`_ tests.
It's meant for leveraging existing unittest-style projects
to use pytest features.  Concretely, pytest will automatically 
collect ``unittest.TestCase`` subclasses and their ``test`` methods in
test files.  It will invoke typical setup/teardown methods and 
generally try to make test suites written to run on unittest, to also 
run using ``pytest``.  We assume here that you are familiar with writing
``unittest.TestCase`` style tests and rather focus on 
integration aspects.

Usage
-------------------------------------------------------------------

After :ref:`installation` type::

    py.test 

and you should be able to run your unittest-style tests if they
are contained in ``test_*`` modules.  If that works for you then
you can make use of most :ref:`pytest features <features>`, for example
``--pdb`` debugging in failures, using :ref:`plain assert-statements <assert>`,
:ref:`more informative tracebacks <tbreportdemo>`, stdout-capturing or 
distributing tests to multiple CPUs via the ``-nNUM`` option if you 
installed the ``pytest-xdist`` plugin.  Please refer to
the general ``pytest`` documentation for many more examples.

Mixing pytest fixtures into unittest.TestCase style tests
-----------------------------------------------------------

Running your unittest with ``pytest`` allows you to use its
:ref:`fixture mechanism <fixture>` with ``unittest.TestCase`` style
tests.  Assuming you have at least skimmed the pytest fixture features,
let's jump-start into an example that integrates a pytest ``db_class``
fixture, setting up a class-cached database object, and then reference
it from a unittest-style test::

    # content of conftest.py

    # we define a fixture function below and it will be "used" by
    # referencing its name from tests

    import pytest

    @pytest.fixture(scope="class")
    def db_class(request):
        class DummyDB:
            pass
        # set a class attribute on the invoking test context 
        request.cls.db = DummyDB()

This defines a fixture function ``db_class`` which - if used - is 
called once for each test class and which sets the class-level 
``db`` attribute to a ``DummyDB`` instance.  The fixture function
achieves this by receiving a special ``request`` object which gives
access to :ref:`the requesting test context <request-context>` such
as the ``cls`` attribute, denoting the class from which the fixture 
is used.  This architecture de-couples fixture writing from actual test
code and allows re-use of the fixture by a minimal reference, the fixture
name.  So let's write an actual ``unittest.TestCase`` class using our 
fixture definition::

    # content of test_unittest_db.py

    import unittest
    import pytest

    @pytest.mark.usefixtures("db_class")
    class MyTest(unittest.TestCase):
        def test_method1(self):
            assert hasattr(self, "db")
            assert 0, self.db   # fail for demo purposes

        def test_method2(self):
            assert 0, self.db   # fail for demo purposes

The ``@pytest.mark.usefixtures("db_class")`` class-decorator makes sure that 
the pytest fixture function ``db_class`` is called once per class.
Due to the deliberately failing assert statements, we can take a look at
the ``self.db`` values in the traceback::

    $ py.test test_unittest_db.py
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR, inifile: 
    collected 2 items
    
    test_unittest_db.py FF
    
    ======= FAILURES ========
    _______ MyTest.test_method1 ________
    
    self = <test_unittest_db.MyTest testMethod=test_method1>
    
        def test_method1(self):
            assert hasattr(self, "db")
    >       assert 0, self.db   # fail for demo purposes
    E       AssertionError: <conftest.db_class.<locals>.DummyDB object at 0xdeadbeef>
    E       assert 0
    
    test_unittest_db.py:9: AssertionError
    _______ MyTest.test_method2 ________
    
    self = <test_unittest_db.MyTest testMethod=test_method2>
    
        def test_method2(self):
    >       assert 0, self.db   # fail for demo purposes
    E       AssertionError: <conftest.db_class.<locals>.DummyDB object at 0xdeadbeef>
    E       assert 0
    
    test_unittest_db.py:12: AssertionError
    ======= 2 failed in 0.12 seconds ========

This default pytest traceback shows that the two test methods
share the same ``self.db`` instance which was our intention
when writing the class-scoped fixture function above.


autouse fixtures and accessing other fixtures
-------------------------------------------------------------------

Although it's usually better to explicitly declare use of fixtures you need
for a given test, you may sometimes want to have fixtures that are 
automatically used in a given context.  After all, the traditional 
style of unittest-setup mandates the use of this implicit fixture writing
and chances are, you are used to it or like it.  

You can flag fixture functions with ``@pytest.fixture(autouse=True)``
and define the fixture function in the context where you want it used.
Let's look at an ``initdir`` fixture which makes all test methods of a
``TestCase`` class execute in a temporary directory with a
pre-initialized ``samplefile.ini``.  Our ``initdir`` fixture itself uses
the pytest builtin :ref:`tmpdir <tmpdir>` fixture to delegate the
creation of a per-test temporary directory::

    # content of test_unittest_cleandir.py
    import pytest
    import unittest

    class MyTest(unittest.TestCase):
        @pytest.fixture(autouse=True)
        def initdir(self, tmpdir):
            tmpdir.chdir() # change to pytest-provided temporary directory
            tmpdir.join("samplefile.ini").write("# testdata")

        def test_method(self):
            s = open("samplefile.ini").read() 
            assert "testdata" in s

Due to the ``autouse`` flag the ``initdir`` fixture function will be
used for all methods of the class where it is defined.  This is a
shortcut for using a ``@pytest.mark.usefixtures("initdir")`` marker
on the class like in the previous example.

Running this test module ...::

    $ py.test -q test_unittest_cleandir.py
    .
    1 passed in 0.12 seconds

... gives us one passed test because the ``initdir`` fixture function
was executed ahead of the ``test_method``.

.. note::

   While pytest supports receiving fixtures via :ref:`test function arguments <funcargs>` for non-unittest test methods, ``unittest.TestCase`` methods cannot directly receive fixture 
   function arguments as implementing that is likely to inflict
   on the ability to run general unittest.TestCase test suites.
   Maybe optional support would be possible, though.  If unittest finally 
   grows a plugin system that should help as well.  In the meanwhile, the 
   above ``usefixtures`` and ``autouse`` examples should help to mix in 
   pytest fixtures into unittest suites.  And of course you can also start
   to selectively leave away the ``unittest.TestCase`` subclassing, use
   plain asserts and get the unlimited pytest feature set.


Converting from unittest to pytest
---------------------------------------

If you want to convert your unittest testcases to pytest, there are
some helpers like `unittest2pytest
<https://pypi.python.org/pypi/unittest2pytest/>`__, which uses lib2to3
and introspection for the transformation.
