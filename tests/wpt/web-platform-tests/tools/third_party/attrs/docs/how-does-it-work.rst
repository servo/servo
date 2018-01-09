.. _how:

How Does It Work?
=================


Boilerplate
-----------

``attrs`` certainly isn't the first library that aims to simplify class definition in Python.
But its **declarative** approach combined with **no runtime overhead** lets it stand out.

Once you apply the ``@attr.s`` decorator to a class, ``attrs`` searches the class object for instances of ``attr.ib``\ s.
Internally they're a representation of the data passed into ``attr.ib`` along with a counter to preserve the order of the attributes.

In order to ensure that sub-classing works as you'd expect it to work, ``attrs`` also walks the class hierarchy and collects the attributes of all super-classes.
Please note that ``attrs`` does *not* call ``super()`` *ever*.
It will write dunder methods to work on *all* of those attributes which also has performance benefits due to fewer function calls.

Once ``attrs`` knows what attributes it has to work on, it writes the requested dunder methods and -- depending on whether you wish to have ``__slots__`` -- creates a new class for you (``slots=True``) or attaches them to the original class (``slots=False``).
While creating new classes is more elegant, we've run into several edge cases surrounding metaclasses that make it impossible to go this route unconditionally.

To be very clear: if you define a class with a single attribute  without a default value, the generated ``__init__`` will look *exactly* how you'd expect:

.. doctest::

   >>> import attr, inspect
   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib()
   >>> print(inspect.getsource(C.__init__))
   def __init__(self, x):
       self.x = x
   <BLANKLINE>

No magic, no meta programming, no expensive introspection at runtime.

****

Everything until this point happens exactly *once* when the class is defined.
As soon as a class is done, it's done.
And it's just a regular Python class like any other, except for a single ``__attrs_attrs__`` attribute that can be used for introspection or for writing your own tools and decorators on top of ``attrs`` (like :func:`attr.asdict`).

And once you start instantiating your classes, ``attrs`` is out of your way completely.

This **static** approach was very much a design goal of ``attrs`` and what I strongly believe makes it distinct.


.. _how-frozen:

Immutability
------------

In order to give you immutability, ``attrs`` will attach a ``__setattr__`` method to your class that raises a :exc:`attr.exceptions.FrozenInstanceError` whenever anyone tries to set an attribute.

To circumvent that ourselves in ``__init__``, ``attrs`` uses (an aggressively cached) :meth:`object.__setattr__` to set your attributes.
This is (still) slower than a plain assignment:

.. code-block:: none

  $ pyperf timeit --rigorous \
        -s "import attr; C = attr.make_class('C', ['x', 'y', 'z'], slots=True)" \
        "C(1, 2, 3)"
  ........................................
  Median +- std dev: 378 ns +- 12 ns

  $ pyperf timeit --rigorous \
        -s "import attr; C = attr.make_class('C', ['x', 'y', 'z'], slots=True, frozen=True)" \
        "C(1, 2, 3)"
  ........................................
  Median +- std dev: 676 ns +- 16 ns

So on a standard notebook the difference is about 300 nanoseconds (1 second is 1,000,000,000 nanoseconds).
It's certainly something you'll feel in a hot loop but shouldn't matter in normal code.
Pick what's more important to you.

****

Once constructed, frozen instances don't differ in any way from regular ones except that you cannot change its attributes.
