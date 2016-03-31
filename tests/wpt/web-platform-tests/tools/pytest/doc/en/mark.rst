
.. _mark:

Marking test functions with attributes
=================================================================

.. currentmodule:: _pytest.mark

By using the ``pytest.mark`` helper you can easily set
metadata on your test functions. There are
some builtin markers, for example:

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


API reference for mark related objects
------------------------------------------------

.. autoclass:: MarkGenerator
    :members:

.. autoclass:: MarkDecorator
    :members:

.. autoclass:: MarkInfo
    :members:

