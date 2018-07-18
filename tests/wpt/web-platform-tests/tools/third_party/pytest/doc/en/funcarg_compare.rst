:orphan:

.. _`funcargcompare`:

pytest-2.3: reasoning for fixture/funcarg evolution
=============================================================

**Target audience**: Reading this document requires basic knowledge of
python testing, xUnit setup methods and the (previous) basic pytest
funcarg mechanism, see http://pytest.org/2.2.4/funcargs.html
If you are new to pytest, then you can simply ignore this
section and read the other sections.

.. currentmodule:: _pytest

Shortcomings of the previous ``pytest_funcarg__`` mechanism
--------------------------------------------------------------

The pre pytest-2.3 funcarg mechanism calls a factory each time a
funcarg for a test function is required.  If a factory wants to
re-use a resource across different scopes, it often used
the ``request.cached_setup()`` helper to manage caching of
resources.  Here is a basic example how we could implement
a per-session Database object::

    # content of conftest.py
    class Database(object):
        def __init__(self):
            print ("database instance created")
        def destroy(self):
            print ("database instance destroyed")

    def pytest_funcarg__db(request):
        return request.cached_setup(setup=DataBase,
                                    teardown=lambda db: db.destroy,
                                    scope="session")

There are several limitations and difficulties with this approach:

1. Scoping funcarg resource creation is not straight forward, instead one must
   understand the intricate cached_setup() method mechanics.

2. parametrizing the "db" resource is not straight forward:
   you need to apply a "parametrize" decorator or implement a
   :py:func:`~hookspec.pytest_generate_tests` hook
   calling :py:func:`~python.Metafunc.parametrize` which
   performs parametrization at the places where the resource
   is used.  Moreover, you need to modify the factory to use an
   ``extrakey`` parameter containing ``request.param`` to the
   :py:func:`~python.Request.cached_setup` call.

3. Multiple parametrized session-scoped resources will be active
   at the same time, making it hard for them to affect global state
   of the application under test.

4. there is no way how you can make use of funcarg factories
   in xUnit setup methods.

5. A non-parametrized fixture function cannot use a parametrized
   funcarg resource if it isn't stated in the test function signature.

All of these limitations are addressed with pytest-2.3 and its
improved :ref:`fixture mechanism <fixture>`.


Direct scoping of fixture/funcarg factories
--------------------------------------------------------

Instead of calling cached_setup() with a cache scope, you can use the
:ref:`@pytest.fixture <pytest.fixture>` decorator and directly state
the scope::

    @pytest.fixture(scope="session")
    def db(request):
        # factory will only be invoked once per session -
        db = DataBase()
        request.addfinalizer(db.destroy)  # destroy when session is finished
        return db

This factory implementation does not need to call ``cached_setup()`` anymore
because it will only be invoked once per session.  Moreover, the
``request.addfinalizer()`` registers a finalizer according to the specified
resource scope on which the factory function is operating.


Direct parametrization of funcarg resource factories
----------------------------------------------------------

Previously, funcarg factories could not directly cause parametrization.
You needed to specify a ``@parametrize`` decorator on your test function
or implement a ``pytest_generate_tests`` hook to perform
parametrization, i.e. calling a test multiple times with different value
sets.  pytest-2.3 introduces a decorator for use on the factory itself::

    @pytest.fixture(params=["mysql", "pg"])
    def db(request):
        ... # use request.param

Here the factory will be invoked twice (with the respective "mysql"
and "pg" values set as ``request.param`` attributes) and all of
the tests requiring "db" will run twice as well.  The "mysql" and
"pg" values will also be used for reporting the test-invocation variants.

This new way of parametrizing funcarg factories should in many cases
allow to re-use already written factories because effectively
``request.param`` was already used when test functions/classes were
parametrized via
:py:func:`~_pytest.python.Metafunc.parametrize(indirect=True)` calls.

Of course it's perfectly fine to combine parametrization and scoping::

    @pytest.fixture(scope="session", params=["mysql", "pg"])
    def db(request):
        if request.param == "mysql":
            db = MySQL()
        elif request.param == "pg":
            db = PG()
        request.addfinalizer(db.destroy)  # destroy when session is finished
        return db

This would execute all tests requiring the per-session "db" resource twice,
receiving the values created by the two respective invocations to the
factory function.


No ``pytest_funcarg__`` prefix when using @fixture decorator
-------------------------------------------------------------------

When using the ``@fixture`` decorator the name of the function
denotes the name under which the resource can be accessed as a function
argument::

    @pytest.fixture()
    def db(request):
        ...

The name under which the funcarg resource can be requested is ``db``.

You can still use the "old" non-decorator way of specifying funcarg factories
aka::

    def pytest_funcarg__db(request):
        ...


But it is then not possible to define scoping and parametrization.
It is thus recommended to use the factory decorator.


solving per-session setup / autouse fixtures
--------------------------------------------------------------

pytest for a long time offered a pytest_configure and a pytest_sessionstart
hook which are often used to setup global resources.  This suffers from
several problems:

1. in distributed testing the master process would setup test resources
   that are never needed because it only co-ordinates the test run
   activities of the slave processes.

2. if you only perform a collection (with "--collect-only")
   resource-setup will still be executed.

3. If a pytest_sessionstart is contained in some subdirectories
   conftest.py file, it will not be called.  This stems from the
   fact that this hook is actually used for reporting, in particular
   the test-header with platform/custom information.

Moreover, it was not easy to define a scoped setup from plugins or
conftest files other than to implement a ``pytest_runtest_setup()`` hook
and caring for scoping/caching yourself.  And it's virtually impossible
to do this with parametrization as ``pytest_runtest_setup()`` is called
during test execution and parametrization happens at collection time.

It follows that pytest_configure/session/runtest_setup are often not
appropriate for implementing common fixture needs.  Therefore,
pytest-2.3 introduces :ref:`autouse fixtures` which fully
integrate with the generic :ref:`fixture mechanism <fixture>`
and obsolete many prior uses of pytest hooks.

funcargs/fixture discovery now happens at collection time
---------------------------------------------------------------------

Since pytest-2.3, discovery of fixture/funcarg factories are taken care of
at collection time.  This is more efficient especially for large test suites.
Moreover, a call to "pytest --collect-only" should be able to in the future
show a lot of setup-information and thus presents a nice method to get an
overview of fixture management in your project.

.. _`compatibility notes`:

.. _`funcargscompat`:

Conclusion and compatibility notes
---------------------------------------------------------

**funcargs** were originally introduced to pytest-2.0.  In pytest-2.3
the mechanism was extended and refined and is now described as
fixtures:

* previously funcarg factories were specified with a special
  ``pytest_funcarg__NAME`` prefix instead of using the
  ``@pytest.fixture`` decorator.

* Factories received a ``request`` object which managed caching through
  ``request.cached_setup()`` calls and allowed using other funcargs via
  ``request.getfuncargvalue()`` calls.  These intricate APIs made it hard
  to do proper parametrization and implement resource caching. The
  new :py:func:`pytest.fixture` decorator allows to declare the scope
  and let pytest figure things out for you.

* if you used parametrization and funcarg factories which made use of
  ``request.cached_setup()`` it is recommended to invest a few minutes
  and simplify your fixture function code to use the :ref:`@pytest.fixture`
  decorator instead.  This will also allow to take advantage of
  the automatic per-resource grouping of tests.
