Historical Notes
================

This page lists features or behavior from previous versions of pytest which have changed over the years. They are
kept here as a historical note so users looking at old code can find documentation related to them.


.. _marker-revamp:

Marker revamp and iteration
---------------------------

.. versionchanged:: 3.6

pytest's marker implementation traditionally worked by simply updating the ``__dict__`` attribute of functions to cumulatively add markers. As a result, markers would unintentionally be passed along class hierarchies in surprising ways. Further, the API for retrieving them was inconsistent, as markers from parameterization would be stored differently than markers applied using the ``@pytest.mark`` decorator and markers added via ``node.add_marker``.

This state of things made it technically next to impossible to use data from markers correctly without having a deep understanding of the internals, leading to subtle and hard to understand bugs in more advanced usages.

Depending on how a marker got declared/changed one would get either a ``MarkerInfo`` which might contain markers from sibling classes,
``MarkDecorators`` when marks came from parameterization or from a ``node.add_marker`` call, discarding prior marks. Also ``MarkerInfo`` acts like a single mark, when it in fact represents a merged view on multiple marks with the same name.

On top of that markers were not accessible in the same way for modules, classes, and functions/methods.
In fact, markers were only accessible in functions, even if they were declared on classes/modules.

A new API to access markers has been introduced in pytest 3.6 in order to solve the problems with
the initial design, providing the :func:`_pytest.nodes.Node.iter_markers` method to iterate over
markers in a consistent manner and reworking the internals, which solved a great deal of problems
with the initial design.


.. _update marker code:

Updating code
~~~~~~~~~~~~~

The old ``Node.get_marker(name)`` function is considered deprecated because it returns an internal ``MarkerInfo`` object
which contains the merged name, ``*args`` and ``**kwargs`` of all the markers which apply to that node.

In general there are two scenarios on how markers should be handled:

1. Marks overwrite each other. Order matters but you only want to think of your mark as a single item. E.g.
``log_level('info')`` at a module level can be overwritten by ``log_level('debug')`` for a specific test.

    In this case, use ``Node.get_closest_marker(name)``:

    .. code-block:: python

        # replace this:
        marker = item.get_marker("log_level")
        if marker:
            level = marker.args[0]

        # by this:
        marker = item.get_closest_marker("log_level")
        if marker:
            level = marker.args[0]

2. Marks compose in an additive manner. E.g. ``skipif(condition)`` marks mean you just want to evaluate all of them,
order doesn't even matter. You probably want to think of your marks as a set here.

   In this case iterate over each mark and handle their ``*args`` and ``**kwargs`` individually.

   .. code-block:: python

        # replace this
        skipif = item.get_marker("skipif")
        if skipif:
            for condition in skipif.args:
                # eval condition
                ...

        # by this:
        for skipif in item.iter_markers("skipif"):
            condition = skipif.args[0]
            # eval condition


If you are unsure or have any questions, please consider opening
`an issue <https://github.com/pytest-dev/pytest/issues>`_.

Related issues
~~~~~~~~~~~~~~

Here is a non-exhaustive list of issues fixed by the new implementation:

* Marks don't pick up nested classes (`#199 <https://github.com/pytest-dev/pytest/issues/199>`_).

* Markers stain on all related classes (`#568 <https://github.com/pytest-dev/pytest/issues/568>`_).

* Combining marks - args and kwargs calculation (`#2897 <https://github.com/pytest-dev/pytest/issues/2897>`_).

* ``request.node.get_marker('name')`` returns ``None`` for markers applied in classes (`#902 <https://github.com/pytest-dev/pytest/issues/902>`_).

* Marks applied in parametrize are stored as markdecorator (`#2400 <https://github.com/pytest-dev/pytest/issues/2400>`_).

* Fix marker interaction in a backward incompatible way (`#1670 <https://github.com/pytest-dev/pytest/issues/1670>`_).

* Refactor marks to get rid of the current "marks transfer" mechanism (`#2363 <https://github.com/pytest-dev/pytest/issues/2363>`_).

* Introduce FunctionDefinition node, use it in generate_tests (`#2522 <https://github.com/pytest-dev/pytest/issues/2522>`_).

* Remove named marker attributes and collect markers in items (`#891 <https://github.com/pytest-dev/pytest/issues/891>`_).

* skipif mark from parametrize hides module level skipif mark (`#1540 <https://github.com/pytest-dev/pytest/issues/1540>`_).

* skipif + parametrize not skipping tests (`#1296 <https://github.com/pytest-dev/pytest/issues/1296>`_).

* Marker transfer incompatible with inheritance (`#535 <https://github.com/pytest-dev/pytest/issues/535>`_).

More details can be found in the `original PR <https://github.com/pytest-dev/pytest/pull/3317>`_.

.. note::

    in a future major release of pytest we will introduce class based markers,
    at which point markers will no longer be limited to instances of :py:class:`~_pytest.mark.Mark`.


cache plugin integrated into the core
-------------------------------------



The functionality of the :ref:`core cache <cache>` plugin was previously distributed
as a third party plugin named ``pytest-cache``.  The core plugin
is compatible regarding command line options and API usage except that you
can only store/receive data between test runs that is json-serializable.


funcargs and ``pytest_funcarg__``
---------------------------------



In versions prior to 2.3 there was no ``@pytest.fixture`` marker
and you had to use a magic ``pytest_funcarg__NAME`` prefix
for the fixture factory.  This remains and will remain supported
but is not anymore advertised as the primary means of declaring fixture
functions.


``@pytest.yield_fixture`` decorator
-----------------------------------



Prior to version 2.10, in order to use a ``yield`` statement to execute teardown code one
had to mark a fixture using the ``yield_fixture`` marker. From 2.10 onward, normal
fixtures can use ``yield`` directly so the ``yield_fixture`` decorator is no longer needed
and considered deprecated.


``[pytest]`` header in ``setup.cfg``
------------------------------------



Prior to 3.0, the supported section name was ``[pytest]``. Due to how
this may collide with some distutils commands, the recommended
section name for ``setup.cfg`` files is now ``[tool:pytest]``.

Note that for ``pytest.ini`` and ``tox.ini`` files the section
name is ``[pytest]``.


Applying marks to ``@pytest.mark.parametrize`` parameters
---------------------------------------------------------



Prior to version 3.1 the supported mechanism for marking values
used the syntax:

.. code-block:: python

    import pytest


    @pytest.mark.parametrize(
        "test_input,expected", [("3+5", 8), ("2+4", 6), pytest.mark.xfail(("6*9", 42))]
    )
    def test_eval(test_input, expected):
        assert eval(test_input) == expected


This was an initial hack to support the feature but soon was demonstrated to be incomplete,
broken for passing functions or applying multiple marks with the same name but different parameters.

The old syntax is planned to be removed in pytest-4.0.


``@pytest.mark.parametrize`` argument names as a tuple
------------------------------------------------------



In versions prior to 2.4 one needed to specify the argument
names as a tuple.  This remains valid but the simpler ``"name1,name2,..."``
comma-separated-string syntax is now advertised first because
it's easier to write and produces less line noise.


setup: is now an "autouse fixture"
----------------------------------



During development prior to the pytest-2.3 release the name
``pytest.setup`` was used but before the release it was renamed
and moved to become part of the general fixture mechanism,
namely :ref:`autouse fixtures`


.. _string conditions:

Conditions as strings instead of booleans
-----------------------------------------



Prior to pytest-2.4 the only way to specify skipif/xfail conditions was
to use strings:

.. code-block:: python

    import sys


    @pytest.mark.skipif("sys.version_info >= (3,3)")
    def test_function():
        ...

During test function setup the skipif condition is evaluated by calling
``eval('sys.version_info >= (3,0)', namespace)``.  The namespace contains
all the module globals, and ``os`` and ``sys`` as a minimum.

Since pytest-2.4 :ref:`boolean conditions <condition booleans>` are considered preferable
because markers can then be freely imported between test modules.
With strings you need to import not only the marker but all variables
used by the marker, which violates encapsulation.

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
configuration value which you might have added:

.. code-block:: python

    @pytest.mark.skipif("not config.getvalue('db')")
    def test_function():
        ...

The equivalent with "boolean conditions" is:

.. code-block:: python

    @pytest.mark.skipif(not pytest.config.getvalue("db"), reason="--db was not specified")
    def test_function():
        pass

.. note::

    You cannot use ``pytest.config.getvalue()`` in code
    imported before pytest's argument parsing takes place.  For example,
    ``conftest.py`` files are imported before command line parsing and thus
    ``config.getvalue()`` will not execute correctly.

``pytest.set_trace()``
----------------------



Previous to version 2.4 to set a break point in code one needed to use ``pytest.set_trace()``:

.. code-block:: python

    import pytest


    def test_function():
        ...
        pytest.set_trace()  # invoke PDB debugger and tracing


This is no longer needed and one can use the native ``import pdb;pdb.set_trace()`` call directly.

For more details see :ref:`breakpoints`.

"compat" properties
-------------------



Access of ``Module``, ``Function``, ``Class``, ``Instance``, ``File`` and ``Item`` through ``Node`` instances have long
been documented as deprecated, but started to emit warnings from pytest ``3.9`` and onward.

Users should just ``import pytest`` and access those objects using the ``pytest`` module.
