Glossary
========

.. glossary::

   dict classes
      A regular class whose attributes are stored in the ``__dict__`` attribute of every single instance.
      This is quite wasteful especially for objects with very few data attributes and the space consumption can become significant when creating large numbers of instances.

      This is the type of class you get by default both with and without ``attrs``.

   slotted classes
      A class that has no ``__dict__`` attribute and `defines <https://docs.python.org/3/reference/datamodel.html#slots>`_ its attributes in a ``__slots__`` attribute instead.
      In ``attrs``, they are created by passing ``slots=True`` to ``@attr.s``.

      Their main advantage is that they use less memory on CPython [#pypy]_.

      However they also come with a bunch of possibly surprising gotchas:

      - Slotted classes don't allow for any other attribute to be set except for those defined in one of the class' hierarchies ``__slots__``:

        .. doctest::

          >>> import attr
          >>> @attr.s(slots=True)
          ... class Coordinates(object):
          ...     x = attr.ib()
          ...     y = attr.ib()
          ...
          >>> c = Coordinates(x=1, y=2)
          >>> c.z = 3
          Traceback (most recent call last):
              ...
          AttributeError: 'Coordinates' object has no attribute 'z'

      - Slotted classes can inherit from other classes just like non-slotted classes, but some of the benefits of slotted classes are lost if you do that.
        If you must inherit from other classes, try to inherit only from other slot classes.

      - Using :mod:`pickle` with slotted classes requires pickle protocol 2 or greater.
        Python 2 uses protocol 0 by default so the protocol needs to be specified.
        Python 3 uses protocol 3 by default.
        You can support protocol 0 and 1 by implementing :meth:`__getstate__ <object.__getstate__>` and :meth:`__setstate__ <object.__setstate__>` methods yourself.
        Those methods are created for frozen slotted classes because they won't pickle otherwise.
        `Think twice <https://www.youtube.com/watch?v=7KnfGDajDQw>`_ before using :mod:`pickle` though.

      - As always with slotted classes, you must specify a ``__weakref__`` slot if you wish for the class to be weak-referenceable.
        Here's how it looks using ``attrs``:

        .. doctest::

          >>> import weakref
          >>> @attr.s(slots=True)
          ... class C(object):
          ...     __weakref__ = attr.ib(init=False, hash=False, repr=False, cmp=False)
          ...     x = attr.ib()
          >>> c = C(1)
          >>> weakref.ref(c)
          <weakref at 0x...; to 'C' at 0x...>
      - Since it's currently impossible to make a class slotted after it's created, ``attrs`` has to replace your class with a new one.
        While it tries to do that as graciously as possible, certain metaclass features like ``__init_subclass__`` do not work with slotted classes.


.. [#pypy] On PyPy, there is no memory advantage in using slotted classes.
