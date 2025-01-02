.. _mark:

How to mark test functions with attributes
===========================================

By using the ``pytest.mark`` helper you can easily set
metadata on your test functions. You can find the full list of builtin markers
in the :ref:`API Reference<marks ref>`. Or you can list all the markers, including
builtin and custom, using the CLI - :code:`pytest --markers`.

Here are some of the builtin markers:

* :ref:`usefixtures <usefixtures>` - use fixtures on a test function or class
* :ref:`filterwarnings <filterwarnings>` - filter certain warnings of a test function
* :ref:`skip <skip>` - always skip a test function
* :ref:`skipif <skipif>` - skip a test function if a certain condition is met
* :ref:`xfail <xfail>` - produce an "expected failure" outcome if a certain
  condition is met
* :ref:`parametrize <parametrizemark>` - perform multiple calls
  to the same test function.

It's easy to create custom markers or to apply markers
to whole test classes or modules. Those markers can be used by plugins, and also
are commonly used to :ref:`select tests <mark run>` on the command-line with the ``-m`` option.

See :ref:`mark examples` for examples which also serve as documentation.

.. note::

    Marks can only be applied to tests, having no effect on
    :ref:`fixtures <fixtures>`.


Registering marks
-----------------

You can register custom marks in your ``pytest.ini`` file like this:

.. code-block:: ini

    [pytest]
    markers =
        slow: marks tests as slow (deselect with '-m "not slow"')
        serial

or in your ``pyproject.toml`` file like this:

.. code-block:: toml

    [tool.pytest.ini_options]
    markers = [
        "slow: marks tests as slow (deselect with '-m \"not slow\"')",
        "serial",
    ]

Note that everything past the ``:`` after the mark name is an optional description.

Alternatively, you can register new markers programmatically in a
:ref:`pytest_configure <initialization-hooks>` hook:

.. code-block:: python

    def pytest_configure(config):
        config.addinivalue_line(
            "markers", "env(name): mark test to run only on named environment"
        )


Registered marks appear in pytest's help text and do not emit warnings (see the next section). It
is recommended that third-party plugins always :ref:`register their markers <registering-markers>`.

.. _unknown-marks:

Raising errors on unknown marks
-------------------------------

Unregistered marks applied with the ``@pytest.mark.name_of_the_mark`` decorator
will always emit a warning in order to avoid silently doing something
surprising due to mistyped names. As described in the previous section, you can disable
the warning for custom marks by registering them in your ``pytest.ini`` file or
using a custom ``pytest_configure`` hook.

When the ``--strict-markers`` command-line flag is passed, any unknown marks applied
with the ``@pytest.mark.name_of_the_mark`` decorator will trigger an error. You can
enforce this validation in your project by adding ``--strict-markers`` to ``addopts``:

.. code-block:: ini

    [pytest]
    addopts = --strict-markers
    markers =
        slow: marks tests as slow (deselect with '-m "not slow"')
        serial
