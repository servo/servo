Comparison
==========

By default, two instances of ``attrs`` classes are equal if all their fields are equal.
For that, ``attrs`` writes ``__eq__`` and ``__ne__`` methods for you.

Additionally, if you pass ``order=True`` (which is the default if you use the `attr.s` decorator), ``attrs`` will also create a full set of ordering methods that are based on the defined fields: ``__le__``, ``__lt__``, ``__ge__``, and ``__gt__``.


.. _custom-comparison:

Customization
-------------

As with other features, you can exclude fields from being involved in comparison operations:

.. doctest::

   >>> from attr import define, field

   >>> @define
   ... class C:
   ...     x: int
   ...     y: int = field(eq=False)

   >>> C(1, 2) == C(1, 3)
   True

Additionally you can also pass a *callable* instead of a bool to both *eq* and *order*.
It is then used as a key function like you may know from `sorted`:

.. doctest::

   >>> from attr import define, field

   >>> @define
   ... class S:
   ...     x: str = field(eq=str.lower)

   >>> S("foo") == S("FOO")
   True

   >>> @define(order=True)
   ... class C:
   ...     x: str = field(order=int)

   >>> C("10") > C("2")
   True

This is especially useful when you have fields with objects that have atypical comparison properties.
Common examples of such objects are `NumPy arrays <https://github.com/python-attrs/attrs/issues/435>`_.

To save you unnecessary boilerplate, ``attrs`` comes with the `attr.cmp_using` helper to create such functions.
For NumPy arrays it would look like this::

   import numpy

   @define(order=False)
   class C:
      an_array = field(eq=attr.cmp_using(eq=numpy.array_equal))


.. warning::

   Please note that *eq* and *order* are set *independently*, because *order* is `False` by default in `attrs.define` (but not in `attr.s`).
   You can set both at once by using the *cmp* argument that we've undeprecated just for this use-case.
