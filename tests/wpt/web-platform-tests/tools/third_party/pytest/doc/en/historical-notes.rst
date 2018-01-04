Historical Notes
================

This page lists features or behavior from previous versions of pytest which have changed over the years. They are
kept here as a historical note so users looking at old code can find documentation related to them.

cache plugin integrated into the core
-------------------------------------

.. versionadded:: 2.8

The functionality of the :ref:`core cache <cache>` plugin was previously distributed
as a third party plugin named ``pytest-cache``.  The core plugin
is compatible regarding command line options and API usage except that you
can only store/receive data between test runs that is json-serializable.


funcargs and ``pytest_funcarg__``
---------------------------------

.. versionchanged:: 2.3

In versions prior to 2.3 there was no ``@pytest.fixture`` marker
and you had to use a magic ``pytest_funcarg__NAME`` prefix
for the fixture factory.  This remains and will remain supported
but is not anymore advertised as the primary means of declaring fixture
functions.


``@pytest.yield_fixture`` decorator
-----------------------------------

.. versionchanged:: 2.10

Prior to version 2.10, in order to use a ``yield`` statement to execute teardown code one
had to mark a fixture using the ``yield_fixture`` marker. From 2.10 onward, normal
fixtures can use ``yield`` directly so the ``yield_fixture`` decorator is no longer needed
and considered deprecated.


``[pytest]`` header in ``setup.cfg``
------------------------------------

.. versionchanged:: 3.0

Prior to 3.0, the supported section name was ``[pytest]``. Due to how
this may collide with some distutils commands, the recommended
section name for ``setup.cfg`` files is now ``[tool:pytest]``.

Note that for ``pytest.ini`` and ``tox.ini`` files the section
name is ``[pytest]``.


Applying marks to ``@pytest.mark.parametrize`` parameters
---------------------------------------------------------

.. versionchanged:: 3.1

Prior to version 3.1 the supported mechanism for marking values
used the syntax::

    import pytest
    @pytest.mark.parametrize("test_input,expected", [
        ("3+5", 8),
        ("2+4", 6),
        pytest.mark.xfail(("6*9", 42),),
    ])
    def test_eval(test_input, expected):
        assert eval(test_input) == expected


This was an initial hack to support the feature but soon was demonstrated to be incomplete,
broken for passing functions or applying multiple marks with the same name but different parameters.

The old syntax is planned to be removed in pytest-4.0.


``@pytest.mark.parametrize`` argument names as a tuple
------------------------------------------------------

.. versionchanged:: 2.4

In versions prior to 2.4 one needed to specify the argument
names as a tuple.  This remains valid but the simpler ``"name1,name2,..."``
comma-separated-string syntax is now advertised first because
it's easier to write and produces less line noise.


setup: is now an "autouse fixture"
----------------------------------

.. versionchanged:: 2.3

During development prior to the pytest-2.3 release the name
``pytest.setup`` was used but before the release it was renamed
and moved to become part of the general fixture mechanism,
namely :ref:`autouse fixtures`


.. _string conditions:

Conditions as strings instead of booleans
-----------------------------------------

.. versionchanged:: 2.4

Prior to pytest-2.4 the only way to specify skipif/xfail conditions was
to use strings::

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
    imported before pytest's argument parsing takes place.  For example,
    ``conftest.py`` files are imported before command line parsing and thus
    ``config.getvalue()`` will not execute correctly.

``pytest.set_trace()``
----------------------

.. versionchanged:: 2.4

Previous to version 2.4 to set a break point in code one needed to use ``pytest.set_trace()``::

    import pytest
    def test_function():
        ...
        pytest.set_trace()    # invoke PDB debugger and tracing


This is no longer needed and one can use the native ``import pdb;pdb.set_trace()`` call directly.

For more details see :ref:`breakpoints`.
