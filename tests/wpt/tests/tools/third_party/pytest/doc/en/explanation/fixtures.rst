.. _about-fixtures:

About fixtures
===============

.. seealso:: :ref:`how-to-fixtures`
.. seealso:: :ref:`Fixtures reference <reference-fixtures>`

pytest fixtures are designed to be explicit, modular and scalable.

What fixtures are
-----------------

In testing, a `fixture <https://en.wikipedia.org/wiki/Test_fixture#Software>`_
provides a defined, reliable and consistent context for the tests. This could
include environment (for example a database configured with known parameters)
or content (such as a dataset).

Fixtures define the steps and data that constitute the *arrange* phase of a
test (see :ref:`test-anatomy`). In pytest, they are functions you define that
serve this purpose. They can also be used to define a test's *act* phase; this
is a powerful technique for designing more complex tests.

The services, state, or other operating environments set up by fixtures are
accessed by test functions through arguments. For each fixture used by a test
function there is typically a parameter (named after the fixture) in the test
function's definition.

We can tell pytest that a particular function is a fixture by decorating it with
:py:func:`@pytest.fixture <pytest.fixture>`. Here's a simple example of
what a fixture in pytest might look like:

.. code-block:: python

    import pytest


    class Fruit:
        def __init__(self, name):
            self.name = name

        def __eq__(self, other):
            return self.name == other.name


    @pytest.fixture
    def my_fruit():
        return Fruit("apple")


    @pytest.fixture
    def fruit_basket(my_fruit):
        return [Fruit("banana"), my_fruit]


    def test_my_fruit_in_basket(my_fruit, fruit_basket):
        assert my_fruit in fruit_basket

Tests don't have to be limited to a single fixture, either. They can depend on
as many fixtures as you want, and fixtures can use other fixtures, as well. This
is where pytest's fixture system really shines.


Improvements over xUnit-style setup/teardown functions
-----------------------------------------------------------

pytest fixtures offer dramatic improvements over the classic xUnit
style of setup/teardown functions:

* fixtures have explicit names and are activated by declaring their use
  from test functions, modules, classes or whole projects.

* fixtures are implemented in a modular manner, as each fixture name
  triggers a *fixture function* which can itself use other fixtures.

* fixture management scales from simple unit to complex
  functional testing, allowing to parametrize fixtures and tests according
  to configuration and component options, or to re-use fixtures
  across function, class, module or whole test session scopes.

* teardown logic can be easily, and safely managed, no matter how many fixtures
  are used, without the need to carefully handle errors by hand or micromanage
  the order that cleanup steps are added.

In addition, pytest continues to support :ref:`xunitsetup`.  You can mix
both styles, moving incrementally from classic to new style, as you
prefer.  You can also start out from existing :ref:`unittest.TestCase
style <unittest.TestCase>` or :ref:`nose based <nosestyle>` projects.



Fixture errors
--------------

pytest does its best to put all the fixtures for a given test in a linear order
so that it can see which fixture happens first, second, third, and so on. If an
earlier fixture has a problem, though, and raises an exception, pytest will stop
executing fixtures for that test and mark the test as having an error.

When a test is marked as having an error, it doesn't mean the test failed,
though. It just means the test couldn't even be attempted because one of the
things it depends on had a problem.

This is one reason why it's a good idea to cut out as many unnecessary
dependencies as possible for a given test. That way a problem in something
unrelated isn't causing us to have an incomplete picture of what may or may not
have issues.

Here's a quick example to help explain:

.. code-block:: python

    import pytest


    @pytest.fixture
    def order():
        return []


    @pytest.fixture
    def append_first(order):
        order.append(1)


    @pytest.fixture
    def append_second(order, append_first):
        order.extend([2])


    @pytest.fixture(autouse=True)
    def append_third(order, append_second):
        order += [3]


    def test_order(order):
        assert order == [1, 2, 3]


If, for whatever reason, ``order.append(1)`` had a bug and it raises an exception,
we wouldn't be able to know if ``order.extend([2])`` or ``order += [3]`` would
also have problems. After ``append_first`` throws an exception, pytest won't run
any more fixtures for ``test_order``, and it won't even try to run
``test_order`` itself. The only things that would've run would be ``order`` and
``append_first``.


Sharing test data
-----------------

If you want to make test data from files available to your tests, a good way
to do this is by loading these data in a fixture for use by your tests.
This makes use of the automatic caching mechanisms of pytest.

Another good approach is by adding the data files in the ``tests`` folder.
There are also community plugins available to help to manage this aspect of
testing, e.g. :pypi:`pytest-datadir` and :pypi:`pytest-datafiles`.

.. _fixtures-signal-cleanup:

A note about fixture cleanup
----------------------------

pytest does not do any special processing for :data:`SIGTERM <signal.SIGTERM>` and
:data:`SIGQUIT <signal.SIGQUIT>` signals (:data:`SIGINT <signal.SIGINT>` is handled naturally
by the Python runtime via :class:`KeyboardInterrupt`), so fixtures that manage external resources which are important
to be cleared when the Python process is terminated (by those signals) might leak resources.

The reason pytest does not handle those signals to perform fixture cleanup is that signal handlers are global,
and changing them might interfere with the code under execution.

If fixtures in your suite need special care regarding termination in those scenarios,
see :issue:`this comment <5243#issuecomment-491522595>` in the issue
tracker for a possible workaround.
