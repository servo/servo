
.. _`unittest.TestCase`:
.. _`unittest`:

How to use ``unittest``-based tests with pytest
===============================================

``pytest`` supports running Python ``unittest``-based tests out of the box.
It's meant for leveraging existing ``unittest``-based test suites
to use pytest as a test runner and also allow to incrementally adapt
the test suite to take full advantage of pytest's features.

To run an existing ``unittest``-style test suite using ``pytest``, type:

.. code-block:: bash

    pytest tests


pytest will automatically collect ``unittest.TestCase`` subclasses and
their ``test`` methods in ``test_*.py`` or ``*_test.py`` files.

Almost all ``unittest`` features are supported:

* ``@unittest.skip`` style decorators;
* ``setUp/tearDown``;
* ``setUpClass/tearDownClass``;
* ``setUpModule/tearDownModule``;

.. _`pytest-subtests`: https://github.com/pytest-dev/pytest-subtests
.. _`load_tests protocol`: https://docs.python.org/3/library/unittest.html#load-tests-protocol

Additionally, :ref:`subtests <python:subtests>` are supported by the
`pytest-subtests`_ plugin.

Up to this point pytest does not have support for the following features:

* `load_tests protocol`_;

Benefits out of the box
-----------------------

By running your test suite with pytest you can make use of several features,
in most cases without having to modify existing code:

* Obtain :ref:`more informative tracebacks <tbreportdemo>`;
* :ref:`stdout and stderr <captures>` capturing;
* :ref:`Test selection options <select-tests>` using ``-k`` and ``-m`` flags;
* :ref:`maxfail`;
* :ref:`--pdb <pdb-option>` command-line option for debugging on test failures
  (see :ref:`note <pdb-unittest-note>` below);
* Distribute tests to multiple CPUs using the :pypi:`pytest-xdist` plugin;
* Use :ref:`plain assert-statements <assert>` instead of ``self.assert*`` functions
  (:pypi:`unittest2pytest` is immensely helpful in this);


pytest features in ``unittest.TestCase`` subclasses
---------------------------------------------------

The following pytest features work in ``unittest.TestCase`` subclasses:

* :ref:`Marks <mark>`: :ref:`skip <skip>`, :ref:`skipif <skipif>`, :ref:`xfail <xfail>`;
* :ref:`Auto-use fixtures <mixing-fixtures>`;

The following pytest features **do not** work, and probably
never will due to different design philosophies:

* :ref:`Fixtures <fixture>` (except for ``autouse`` fixtures, see :ref:`below <mixing-fixtures>`);
* :ref:`Parametrization <parametrize>`;
* :ref:`Custom hooks <writing-plugins>`;


Third party plugins may or may not work well, depending on the plugin and the test suite.

.. _mixing-fixtures:

Mixing pytest fixtures into ``unittest.TestCase`` subclasses using marks
------------------------------------------------------------------------

Running your unittest with ``pytest`` allows you to use its
:ref:`fixture mechanism <fixture>` with ``unittest.TestCase`` style
tests.  Assuming you have at least skimmed the pytest fixture features,
let's jump-start into an example that integrates a pytest ``db_class``
fixture, setting up a class-cached database object, and then reference
it from a unittest-style test:

.. code-block:: python

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
fixture definition:

.. code-block:: python

    # content of test_unittest_db.py

    import unittest

    import pytest


    @pytest.mark.usefixtures("db_class")
    class MyTest(unittest.TestCase):
        def test_method1(self):
            assert hasattr(self, "db")
            assert 0, self.db  # fail for demo purposes

        def test_method2(self):
            assert 0, self.db  # fail for demo purposes

The ``@pytest.mark.usefixtures("db_class")`` class-decorator makes sure that
the pytest fixture function ``db_class`` is called once per class.
Due to the deliberately failing assert statements, we can take a look at
the ``self.db`` values in the traceback:

.. code-block:: pytest

    $ pytest test_unittest_db.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-8.x.y, pluggy-1.x.y
    rootdir: /home/sweet/project
    collected 2 items

    test_unittest_db.py FF                                               [100%]

    ================================= FAILURES =================================
    ___________________________ MyTest.test_method1 ____________________________

    self = <test_unittest_db.MyTest testMethod=test_method1>

        def test_method1(self):
            assert hasattr(self, "db")
    >       assert 0, self.db  # fail for demo purposes
    E       AssertionError: <conftest.db_class.<locals>.DummyDB object at 0xdeadbeef0001>
    E       assert 0

    test_unittest_db.py:11: AssertionError
    ___________________________ MyTest.test_method2 ____________________________

    self = <test_unittest_db.MyTest testMethod=test_method2>

        def test_method2(self):
    >       assert 0, self.db  # fail for demo purposes
    E       AssertionError: <conftest.db_class.<locals>.DummyDB object at 0xdeadbeef0001>
    E       assert 0

    test_unittest_db.py:14: AssertionError
    ========================= short test summary info ==========================
    FAILED test_unittest_db.py::MyTest::test_method1 - AssertionError: <conft...
    FAILED test_unittest_db.py::MyTest::test_method2 - AssertionError: <conft...
    ============================ 2 failed in 0.12s =============================

This default pytest traceback shows that the two test methods
share the same ``self.db`` instance which was our intention
when writing the class-scoped fixture function above.


Using autouse fixtures and accessing other fixtures
---------------------------------------------------

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
the pytest builtin :fixture:`tmp_path` fixture to delegate the
creation of a per-test temporary directory:

.. code-block:: python

    # content of test_unittest_cleandir.py
    import unittest

    import pytest


    class MyTest(unittest.TestCase):
        @pytest.fixture(autouse=True)
        def initdir(self, tmp_path, monkeypatch):
            monkeypatch.chdir(tmp_path)  # change to pytest-provided temporary directory
            tmp_path.joinpath("samplefile.ini").write_text("# testdata", encoding="utf-8")

        def test_method(self):
            with open("samplefile.ini", encoding="utf-8") as f:
                s = f.read()
            assert "testdata" in s

Due to the ``autouse`` flag the ``initdir`` fixture function will be
used for all methods of the class where it is defined.  This is a
shortcut for using a ``@pytest.mark.usefixtures("initdir")`` marker
on the class like in the previous example.

Running this test module ...:

.. code-block:: pytest

    $ pytest -q test_unittest_cleandir.py
    .                                                                    [100%]
    1 passed in 0.12s

... gives us one passed test because the ``initdir`` fixture function
was executed ahead of the ``test_method``.

.. note::

   ``unittest.TestCase`` methods cannot directly receive fixture
   arguments as implementing that is likely to inflict
   on the ability to run general unittest.TestCase test suites.

   The above ``usefixtures`` and ``autouse`` examples should help to mix in
   pytest fixtures into unittest suites.

   You can also gradually move away from subclassing from ``unittest.TestCase`` to *plain asserts*
   and then start to benefit from the full pytest feature set step by step.

.. _pdb-unittest-note:

.. note::

    Due to architectural differences between the two frameworks, setup and
    teardown for ``unittest``-based tests is performed during the ``call`` phase
    of testing instead of in ``pytest``'s standard ``setup`` and ``teardown``
    stages. This can be important to understand in some situations, particularly
    when reasoning about errors. For example, if a ``unittest``-based suite
    exhibits errors during setup, ``pytest`` will report no errors during its
    ``setup`` phase and will instead raise the error during ``call``.
