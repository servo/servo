API Reference for the ``attr`` Namespace
========================================

.. module:: attr


Core
----

.. autofunction:: attr.s(these=None, repr_ns=None, repr=None, cmp=None, hash=None, init=None, slots=False, frozen=False, weakref_slot=True, str=False, auto_attribs=False, kw_only=False, cache_hash=False, auto_exc=False, eq=None, order=None, auto_detect=False, collect_by_mro=False, getstate_setstate=None, on_setattr=None, field_transformer=None, match_args=True, unsafe_hash=None)

   .. note::

      *attrs* also comes with a serious-business alias ``attr.attrs``.

   For example:

   .. doctest::

      >>> import attr
      >>> @attr.s
      ... class C:
      ...     _private = attr.ib()
      >>> C(private=42)
      C(_private=42)
      >>> class D:
      ...     def __init__(self, x):
      ...         self.x = x
      >>> D(1)
      <D object at ...>
      >>> D = attr.s(these={"x": attr.ib()}, init=False)(D)
      >>> D(1)
      D(x=1)
      >>> @attr.s(auto_exc=True)
      ... class Error(Exception):
      ...     x = attr.ib()
      ...     y = attr.ib(default=42, init=False)
      >>> Error("foo")
      Error(x='foo', y=42)
      >>> raise Error("foo")
      Traceback (most recent call last):
         ...
      Error: ('foo', 42)
      >>> raise ValueError("foo", 42)   # for comparison
      Traceback (most recent call last):
         ...
      ValueError: ('foo', 42)


.. autofunction:: attr.ib

   .. note::

      *attrs* also comes with a serious-business alias ``attr.attrib``.

   The object returned by `attr.ib` also allows for setting the default and the validator using decorators:

   .. doctest::

      >>> @attr.s
      ... class C:
      ...     x = attr.ib()
      ...     y = attr.ib()
      ...     @x.validator
      ...     def _any_name_except_a_name_of_an_attribute(self, attribute, value):
      ...         if value < 0:
      ...             raise ValueError("x must be positive")
      ...     @y.default
      ...     def _any_name_except_a_name_of_an_attribute(self):
      ...         return self.x + 1
      >>> C(1)
      C(x=1, y=2)
      >>> C(-1)
      Traceback (most recent call last):
          ...
      ValueError: x must be positive


.. function:: define

   Same as `attrs.define`.

.. function:: mutable

   Same as `attrs.mutable`.

.. function:: frozen

   Same as `attrs.frozen`.

.. function:: field

   Same as `attrs.field`.

.. class:: Attribute

   Same as `attrs.Attribute`.

.. function:: make_class

   Same as `attrs.make_class`.

.. autoclass:: Factory
   :noindex:

   Same as `attrs.Factory`.


.. data:: NOTHING

   Same as `attrs.NOTHING`.


Exceptions
----------

.. module:: attr.exceptions

All exceptions are available from both ``attr.exceptions`` and `attrs.exceptions` (it's the same module in a different namespace).

Please refer to `attrs.exceptions` for details.


Helpers
-------

.. currentmodule:: attr

.. function:: cmp_using

   Same as `attrs.cmp_using`.

.. function:: fields

   Same as `attrs.fields`.

.. function:: fields_dict

   Same as `attr.fields_dict`.

.. function:: has

   Same as `attrs.has`.

.. function:: resolve_types

   Same as `attrs.resolve_types`.

.. autofunction:: asdict
.. autofunction:: astuple

.. module:: attr.filters

.. function:: include

   Same as `attrs.filters.include`.

.. function:: exclude

   Same as `attrs.filters.exclude`.

See :func:`attrs.asdict` for examples.

All objects from `attrs.filters` are also available in ``attr.filters``.

----

.. currentmodule:: attr

.. function:: evolve

   Same as `attrs.evolve`.

.. function:: validate

   Same as `attrs.validate`.


Validators
----------

.. module:: attr.validators

All objects from `attrs.validators` are also available in ``attr.validators``.
Please refer to the former for details.


Converters
----------

.. module:: attr.converters

All objects from `attrs.converters` are also available from ``attr.converters``.
Please refer to the former for details.


Setters
-------

.. module:: attr.setters

All objects from `attrs.setters` are also available in ``attr.setters``.
Please refer to the former for details.


Deprecated APIs
---------------

.. currentmodule:: attr

To help you write backward compatible code that doesn't throw warnings on modern releases, the ``attr`` module has an ``__version_info__`` attribute as of version 19.2.0.
It behaves similarly to `sys.version_info` and is an instance of `attr.VersionInfo`:

.. autoclass:: VersionInfo

   With its help you can write code like this:

   >>> if getattr(attr, "__version_info__", (0,)) >= (19, 2):
   ...     cmp_off = {"eq": False}
   ... else:
   ...     cmp_off = {"cmp": False}
   >>> cmp_off == {"eq":  False}
   True
   >>> @attr.s(**cmp_off)
   ... class C:
   ...     pass


----

.. autofunction:: assoc

Before *attrs* got `attrs.validators.set_disabled` and  `attrs.validators.set_disabled`, it had the following APIs to globally enable and disable validators.
They won't be removed, but are discouraged to use:

.. autofunction:: set_run_validators
.. autofunction:: get_run_validators

----

The serious-business aliases used to be called ``attr.attributes`` and ``attr.attr``.
There are no plans to remove them but they shouldn't be used in new code.
