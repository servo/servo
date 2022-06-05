Type Annotations
================

``attrs`` comes with first class support for type annotations for both Python 3.6 (:pep:`526`) and legacy syntax.

However they will forever remain *optional*, therefore the example from the README could also be written as:

.. doctest::

   >>> from attrs import define, field

   >>> @define
   ... class SomeClass:
   ...     a_number = field(default=42)
   ...     list_of_numbers = field(factory=list)

   >>> sc = SomeClass(1, [1, 2, 3])
   >>> sc
   SomeClass(a_number=1, list_of_numbers=[1, 2, 3])

You can choose freely between the approaches, but please remember that if you choose to use type annotations, you **must** annotate **all** attributes!

----

Even when going all-in an type annotations, you will need `attr.field` for some advanced features though.

One of those features are the decorator-based features like defaults.
It's important to remember that ``attrs`` doesn't do any magic behind your back.
All the decorators are implemented using an object that is returned by the call to `attrs.field`.

Attributes that only carry a class annotation do not have that object so trying to call a method on it will inevitably fail.

*****

Please note that types -- however added -- are *only metadata* that can be queried from the class and they aren't used for anything out of the box!

Because Python does not allow references to a class object before the class is defined,
types may be defined as string literals, so-called *forward references* (:pep:`526`).
You can enable this automatically for a whole module by using ``from __future__ import annotations`` (:pep:`563`) as of Python 3.7.
In this case ``attrs`` simply puts these string literals into the ``type`` attributes.
If you need to resolve these to real types, you can call `attrs.resolve_types` which will update the attribute in place.

In practice though, types show their biggest usefulness in combination with tools like mypy_, pytype_, or pyright_ that have dedicated support for ``attrs`` classes.

The addition of static types is certainly one of the most exciting features in the Python ecosystem and helps you writing *correct* and *verified self-documenting* code.

If you don't know where to start, Carl Meyer gave a great talk on `Type-checked Python in the Real World <https://www.youtube.com/watch?v=pMgmKJyWKn8>`_ at PyCon US 2018 that will help you to get started in no time.


mypy
----

While having a nice syntax for type metadata is great, it's even greater that mypy_ as of 0.570 ships with a dedicated ``attrs`` plugin which allows you to statically check your code.

Imagine you add another line that tries to instantiate the defined class using ``SomeClass("23")``.
Mypy will catch that error for you:

.. code-block:: console

   $ mypy t.py
   t.py:12: error: Argument 1 to "SomeClass" has incompatible type "str"; expected "int"

This happens *without* running your code!

And it also works with *both* Python 2-style annotation styles.
To mypy, this code is equivalent to the one above:

.. code-block:: python

  @attr.s
  class SomeClass(object):
      a_number = attr.ib(default=42)  # type: int
      list_of_numbers = attr.ib(factory=list, type=list[int])


pyright
-------

``attrs`` provides support for pyright_ though the dataclass_transform_ specification.
This provides static type inference for a subset of ``attrs`` equivalent to standard-library ``dataclasses``,
and requires explicit type annotations using the `attrs.define` or ``@attr.s(auto_attribs=True)`` API.

Given the following definition, ``pyright`` will generate static type signatures for ``SomeClass`` attribute access, ``__init__``, ``__eq__``, and comparison methods::

  @attr.define
  class SomeClass:
      a_number: int = 42
      list_of_numbers: list[int] = attr.field(factory=list)

.. warning::

   The ``pyright`` inferred types are a subset of those supported by ``mypy``, including:

   - The generated ``__init__`` signature only includes the attribute type annotations.
     It currently does not include attribute ``converter`` types.

   - The ``attr.frozen`` decorator is not typed with frozen attributes, which are properly typed via ``attr.define(frozen=True)``.

     A `full list <https://github.com/microsoft/pyright/blob/main/specs/dataclass_transforms.md#attrs>`_ of limitations and incompatibilities can be found in pyright's repository.

   Your constructive feedback is welcome in both `attrs#795 <https://github.com/python-attrs/attrs/issues/795>`_ and `pyright#1782 <https://github.com/microsoft/pyright/discussions/1782>`_.
   Generally speaking, the decision on improving ``attrs`` support in pyright is entirely Microsoft's prerogative though.


.. _mypy: http://mypy-lang.org
.. _pytype: https://google.github.io/pytype/
.. _pyright: https://github.com/microsoft/pyright
.. _dataclass_transform: https://github.com/microsoft/pyright/blob/main/specs/dataclass_transforms.md
