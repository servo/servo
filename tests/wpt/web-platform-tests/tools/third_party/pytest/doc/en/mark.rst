
.. _mark:

Marking test functions with attributes
=================================================================


By using the ``pytest.mark`` helper you can easily set
metadata on your test functions. There are
some builtin markers, for example:

* :ref:`skip <skip>` - always skip a test function
* :ref:`skipif <skipif>` - skip a test function if a certain condition is met
* :ref:`xfail <xfail>` - produce an "expected failure" outcome if a certain
  condition is met
* :ref:`parametrize <parametrizemark>` to perform multiple calls
  to the same test function.

It's easy to create custom markers or to apply markers
to whole test classes or modules. See :ref:`mark examples` for examples
which also serve as documentation.

.. note::

    Marks can only be applied to tests, having no effect on
    :ref:`fixtures <fixtures>`.


Raising errors on unknown marks: --strict
-----------------------------------------

When the ``--strict`` command-line flag is passed, any marks not registered in the ``pytest.ini`` file will trigger an error.

Marks can be registered like this:

.. code-block:: ini

    [pytest]
    markers =
        slow
        serial

This can be used to prevent users mistyping mark names by accident. Test suites that want to enforce this
should add ``--strict`` to ``addopts``:

.. code-block:: ini

    [pytest]
    addopts = --strict
    markers =
        slow
        serial


.. `marker-iteration`

Marker revamp and iteration
---------------------------

.. versionadded:: 3.6

pytest's marker implementation traditionally worked by simply updating the ``__dict__`` attribute of functions to add markers, in a cumulative manner. As a result of the this, markers would unintendely be passed along class hierarchies in surprising ways plus the API for retriving them was inconsistent, as markers from parameterization would be stored differently than markers applied using the ``@pytest.mark`` decorator and markers added via ``node.add_marker``.

This state of things made it technically next to impossible to use data from markers correctly without having a deep understanding of the internals, leading to subtle and hard to understand bugs in more advanced usages.

Depending on how a marker got declared/changed one would get either a ``MarkerInfo`` which might contain markers from sibling classes,
``MarkDecorators`` when marks came from parameterization or from a ``node.add_marker`` call, discarding prior marks. Also ``MarkerInfo`` acts like a single mark, when it in fact represents a merged view on multiple marks with the same name.

On top of that markers where not accessible the same way for modules, classes, and functions/methods,
in fact, markers where only accessible in functions, even if they where declared on classes/modules.

A new API to access markers has been introduced in pytest 3.6 in order to solve the problems with the initial design, providing :func:`_pytest.nodes.Node.iter_markers` method to iterate over markers in a consistent manner and reworking the internals, which solved great deal of problems with the initial design.


.. _update marker code:

Updating code
~~~~~~~~~~~~~

The old ``Node.get_marker(name)`` function is considered deprecated because it returns an internal ``MarkerInfo`` object
which contains the merged name, ``*args`` and ``**kwargs`` of all the markers which apply to that node.

In general there are two scenarios on how markers should be handled:

1. Marks overwrite each other. Order matters but you only want to think of your mark as a single item. E.g.
``log_level('info')`` at a module level can be overwritten by ``log_level('debug')`` for a specific test.

    In this case replace use ``Node.get_closest_marker(name)``:

    .. code-block:: python

        # replace this:
        marker = item.get_marker("log_level")
        if marker:
            level = marker.args[0]

        # by this:
        marker = item.get_closest_marker("log_level")
        if marker:
            level = marker.args[0]

2. Marks compose additive. E.g. ``skipif(condition)`` marks means you just want to evaluate all of them,
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

* markers stains on all related classes (`#568 <https://github.com/pytest-dev/pytest/issues/568>`_).

* combining marks - args and kwargs calculation (`#2897 <https://github.com/pytest-dev/pytest/issues/2897>`_).

* ``request.node.get_marker('name')`` returns ``None`` for markers applied in classes (`#902 <https://github.com/pytest-dev/pytest/issues/902>`_).

* marks applied in parametrize are stored as markdecorator (`#2400 <https://github.com/pytest-dev/pytest/issues/2400>`_).

* fix marker interaction in a backward incompatible way (`#1670 <https://github.com/pytest-dev/pytest/issues/1670>`_).

* Refactor marks to get rid of the current "marks transfer" mechanism (`#2363 <https://github.com/pytest-dev/pytest/issues/2363>`_).

* Introduce FunctionDefinition node, use it in generate_tests (`#2522 <https://github.com/pytest-dev/pytest/issues/2522>`_).

* remove named marker attributes and collect markers in items (`#891 <https://github.com/pytest-dev/pytest/issues/891>`_).

* skipif mark from parametrize hides module level skipif mark (`#1540 <https://github.com/pytest-dev/pytest/issues/1540>`_).

* skipif + parametrize not skipping tests (`#1296 <https://github.com/pytest-dev/pytest/issues/1296>`_).

* marker transfer incompatible with inheritance (`#535 <https://github.com/pytest-dev/pytest/issues/535>`_).

More details can be found in the `original PR <https://github.com/pytest-dev/pytest/pull/3317>`_.

.. note::

    in a future major relase of pytest we will introduce class based markers,
    at which points markers will no longer be limited to instances of :py:class:`Mark`
