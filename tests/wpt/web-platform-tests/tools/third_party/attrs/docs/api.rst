.. _api:

API Reference
=============

.. currentmodule:: attr

``attrs`` works by decorating a class using :func:`attr.s` and then optionally defining attributes on the class using :func:`attr.ib`.

.. note::

   When this documentation speaks about "``attrs`` attributes" it means those attributes that are defined using :func:`attr.ib` in the class body.

What follows is the API explanation, if you'd like a more hands-on introduction, have a look at :doc:`examples`.



Core
----

.. autofunction:: attr.s(these=None, repr_ns=None, repr=True, cmp=True, hash=None, init=True, slots=False, frozen=False, str=False)

   .. note::

      ``attrs`` also comes with a serious business alias ``attr.attrs``.

   For example:

   .. doctest::

      >>> import attr
      >>> @attr.s
      ... class C(object):
      ...     _private = attr.ib()
      >>> C(private=42)
      C(_private=42)
      >>> class D(object):
      ...     def __init__(self, x):
      ...         self.x = x
      >>> D(1)
      <D object at ...>
      >>> D = attr.s(these={"x": attr.ib()}, init=False)(D)
      >>> D(1)
      D(x=1)


.. autofunction:: attr.ib

   .. note::

      ``attrs`` also comes with a serious business alias ``attr.attrib``.

   The object returned by :func:`attr.ib` also allows for setting the default and the validator using decorators:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      ...     y = attr.ib()
      ...     @x.validator
      ...     def name_can_be_anything(self, attribute, value):
      ...         if value < 0:
      ...             raise ValueError("x must be positive")
      ...     @y.default
      ...     def name_does_not_matter(self):
      ...         return self.x + 1
      >>> C(1)
      C(x=1, y=2)
      >>> C(-1)
      Traceback (most recent call last):
          ...
      ValueError: x must be positive

.. autoclass:: attr.Attribute

   Instances of this class are frequently used for introspection purposes like:

   - :func:`fields` returns a tuple of them.
   - Validators get them passed as the first argument.

   .. warning::

       You should never instantiate this class yourself!

   .. doctest::

      >>> import attr
      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      >>> attr.fields(C).x
      Attribute(name='x', default=NOTHING, validator=None, repr=True, cmp=True, hash=None, init=True, convert=None, metadata=mappingproxy({}), type=None)


.. autofunction:: attr.make_class

   This is handy if you want to programmatically create classes.

   For example:

   .. doctest::

      >>> C1 = attr.make_class("C1", ["x", "y"])
      >>> C1(1, 2)
      C1(x=1, y=2)
      >>> C2 = attr.make_class("C2", {"x": attr.ib(default=42),
      ...                             "y": attr.ib(default=attr.Factory(list))})
      >>> C2()
      C2(x=42, y=[])


.. autoclass:: attr.Factory

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(default=attr.Factory(list))
      ...     y = attr.ib(default=attr.Factory(
      ...         lambda self: set(self.x),
      ...         takes_self=True)
      ...     )
      >>> C()
      C(x=[], y=set())
      >>> C([1, 2, 3])
      C(x=[1, 2, 3], y={1, 2, 3})


.. autoexception:: attr.exceptions.FrozenInstanceError
.. autoexception:: attr.exceptions.AttrsAttributeNotFoundError
.. autoexception:: attr.exceptions.NotAnAttrsClassError
.. autoexception:: attr.exceptions.DefaultAlreadySetError
.. autoexception:: attr.exceptions.UnannotatedAttributeError

   For example::

       @attr.s(auto_attribs=True)
       class C:
           x: int
           y = attr.ib()


Influencing Initialization
++++++++++++++++++++++++++

Generally speaking, it's best to keep logic out of your ``__init__``.
The moment you need a finer control over how your class is instantiated, it's usually best to use a classmethod factory or to apply the `builder pattern <https://en.wikipedia.org/wiki/Builder_pattern>`_.

However, sometimes you need to do that one quick thing after your class is initialized.
And for that ``attrs`` offers the ``__attrs_post_init__`` hook that is automatically detected and run after ``attrs`` is done initializing your instance:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib()
   ...     y = attr.ib(init=False)
   ...     def __attrs_post_init__(self):
   ...         self.y = self.x + 1
   >>> C(1)
   C(x=1, y=2)

Please note that you can't directly set attributes on frozen classes:

.. doctest::

   >>> @attr.s(frozen=True)
   ... class FrozenBroken(object):
   ...     x = attr.ib()
   ...     y = attr.ib(init=False)
   ...     def __attrs_post_init__(self):
   ...         self.y = self.x + 1
   >>> FrozenBroken(1)
   Traceback (most recent call last):
      ...
   attr.exceptions.FrozenInstanceError: can't set attribute

If you need to set attributes on a frozen class, you'll have to resort to the :ref:`same trick <how-frozen>` as ``attrs`` and use :meth:`object.__setattr__`:

.. doctest::

   >>> @attr.s(frozen=True)
   ... class Frozen(object):
   ...     x = attr.ib()
   ...     y = attr.ib(init=False)
   ...     def __attrs_post_init__(self):
   ...         object.__setattr__(self, "y", self.x + 1)
   >>> Frozen(1)
   Frozen(x=1, y=2)


.. _helpers:

Helpers
-------

``attrs`` comes with a bunch of helper methods that make working with it easier:

.. autofunction:: attr.fields

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      ...     y = attr.ib()
      >>> attr.fields(C)
      (Attribute(name='x', default=NOTHING, validator=None, repr=True, cmp=True, hash=None, init=True, convert=None, metadata=mappingproxy({}), type=None), Attribute(name='y', default=NOTHING, validator=None, repr=True, cmp=True, hash=None, init=True, convert=None, metadata=mappingproxy({}), type=None))
      >>> attr.fields(C)[1]
      Attribute(name='y', default=NOTHING, validator=None, repr=True, cmp=True, hash=None, init=True, convert=None, metadata=mappingproxy({}), type=None)
      >>> attr.fields(C).y is attr.fields(C)[1]
      True


.. autofunction:: attr.has

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     pass
      >>> attr.has(C)
      True
      >>> attr.has(object)
      False


.. autofunction:: attr.asdict

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      ...     y = attr.ib()
      >>> attr.asdict(C(1, C(2, 3)))
      {'x': 1, 'y': {'x': 2, 'y': 3}}


.. autofunction:: attr.astuple

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      ...     y = attr.ib()
      >>> attr.astuple(C(1,2))
      (1, 2)

``attrs`` includes some handy helpers for filtering:

.. autofunction:: attr.filters.include

.. autofunction:: attr.filters.exclude

See :ref:`asdict` for examples.

.. autofunction:: attr.evolve

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      ...     y = attr.ib()
      >>> i1 = C(1, 2)
      >>> i1
      C(x=1, y=2)
      >>> i2 = attr.evolve(i1, y=3)
      >>> i2
      C(x=1, y=3)
      >>> i1 == i2
      False

   ``evolve`` creates a new instance using ``__init__``.
   This fact has several implications:

   * private attributes should be specified without the leading underscore, just like in ``__init__``.
   * attributes with ``init=False`` can't be set with ``evolve``.
   * the usual ``__init__`` validators will validate the new values.

.. autofunction:: validate

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(validator=attr.validators.instance_of(int))
      >>> i = C(1)
      >>> i.x = "1"
      >>> attr.validate(i)
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <type 'int'> (got '1' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, validator=<instance_of validator for type <type 'int'>>, repr=True, cmp=True, hash=None, init=True, type=None), <type 'int'>, '1')


Validators can be globally disabled if you want to run them only in development and tests but not in production because you fear their performance impact:

.. autofunction:: set_run_validators

.. autofunction:: get_run_validators


.. _api_validators:

Validators
----------

``attrs`` comes with some common validators in the ``attrs.validators`` module:


.. autofunction:: attr.validators.instance_of


   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(validator=attr.validators.instance_of(int))
      >>> C(42)
      C(x=42)
      >>> C("42")
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <type 'int'> (got '42' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, validator=<instance_of validator for type <type 'int'>>, type=None), <type 'int'>, '42')
      >>> C(None)
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <type 'int'> (got None that is a <type 'NoneType'>).", Attribute(name='x', default=NOTHING, validator=<instance_of validator for type <type 'int'>>, repr=True, cmp=True, hash=None, init=True, type=None), <type 'int'>, None)

.. autofunction:: attr.validators.in_

   For example:

   .. doctest::

       >>> import enum
       >>> class State(enum.Enum):
       ...     ON = "on"
       ...     OFF = "off"
       >>> @attr.s
       ... class C(object):
       ...     state = attr.ib(validator=attr.validators.in_(State))
       ...     val = attr.ib(validator=attr.validators.in_([1, 2, 3]))
       >>> C(State.ON, 1)
       C(state=<State.ON: 'on'>, val=1)
       >>> C("on", 1)
       Traceback (most recent call last):
          ...
       ValueError: 'state' must be in <enum 'State'> (got 'on')
       >>> C(State.ON, 4)
       Traceback (most recent call last):
          ...
       ValueError: 'val' must be in [1, 2, 3] (got 4)

.. autofunction:: attr.validators.provides

.. autofunction:: attr.validators.and_

   For convenience, it's also possible to pass a list to :func:`attr.ib`'s validator argument.

   Thus the following two statements are equivalent::

      x = attr.ib(validator=attr.validators.and_(v1, v2, v3))
      x = attr.ib(validator=[v1, v2, v3])

.. autofunction:: attr.validators.optional

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(validator=attr.validators.optional(attr.validators.instance_of(int)))
      >>> C(42)
      C(x=42)
      >>> C("42")
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <type 'int'> (got '42' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, validator=<instance_of validator for type <type 'int'>>, type=None), <type 'int'>, '42')
      >>> C(None)
      C(x=None)


Converters
----------

.. autofunction:: attr.converters.optional

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(convert=attr.converters.optional(int))
      >>> C(None)
      C(x=None)
      >>> C(42)
      C(x=42)


Deprecated APIs
---------------

The serious business aliases used to be called ``attr.attributes`` and ``attr.attr``.
There are no plans to remove them but they shouldn't be used in new code.

.. autofunction:: assoc
