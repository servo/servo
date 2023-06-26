API Reference
=============

.. currentmodule:: attr

``attrs`` works by decorating a class using `attrs.define` or `attr.s` and then optionally defining attributes on the class using `attrs.field`, `attr.ib`, or a type annotation.

If you're confused by the many names, please check out `names` for clarification.

What follows is the API explanation, if you'd like a more hands-on introduction, have a look at `examples`.

As of version 21.3.0, ``attrs`` consists of **two** to-level package names:

- The classic ``attr`` that powered the venerable `attr.s` and `attr.ib`
- The modern ``attrs`` that only contains most modern APIs and relies on `attrs.define` and `attrs.field` to define your classes.
  Additionally it offers some ``attr`` APIs with nicer defaults (e.g. `attrs.asdict`).
  Using this namespace requires Python 3.6 or later.

The ``attrs`` namespace is built *on top of* ``attr`` which will *never* go away.


Core
----

.. note::

  Please note that the ``attrs`` namespace has been added in version 21.3.0.
  Most of the objects are simply re-imported from ``attr``.
  Therefore if a class, method, or function claims that it has been added in an older version, it is only available in the ``attr`` namespace.

.. autodata:: attrs.NOTHING

.. autofunction:: attrs.define

.. function:: attrs.mutable(same_as_define)

   Alias for `attrs.define`.

   .. versionadded:: 20.1.0

.. function:: attrs.frozen(same_as_define)

   Behaves the same as `attrs.define` but sets *frozen=True* and *on_setattr=None*.

   .. versionadded:: 20.1.0

.. autofunction:: attrs.field

.. function:: define

   Old import path for `attrs.define`.

.. function:: mutable

   Old import path for `attrs.mutable`.

.. function:: frozen

   Old import path for `attrs.frozen`.

.. function:: field

   Old import path for `attrs.field`.

.. autoclass:: attrs.Attribute
   :members: evolve

   For example:

   .. doctest::

      >>> import attr
      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      >>> attr.fields(C).x
      Attribute(name='x', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None)


.. autofunction:: attrs.make_class

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


.. autoclass:: attrs.Factory

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


Classic
~~~~~~~

.. data:: attr.NOTHING

   Same as `attrs.NOTHING`.

.. autofunction:: attr.s(these=None, repr_ns=None, repr=None, cmp=None, hash=None, init=None, slots=False, frozen=False, weakref_slot=True, str=False, auto_attribs=False, kw_only=False, cache_hash=False, auto_exc=False, eq=None, order=None, auto_detect=False, collect_by_mro=False, getstate_setstate=None, on_setattr=None, field_transformer=None, match_args=True)

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

      ``attrs`` also comes with a serious business alias ``attr.attrib``.

   The object returned by `attr.ib` also allows for setting the default and the validator using decorators:

   .. doctest::

      >>> @attr.s
      ... class C(object):
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



Exceptions
----------

All exceptions are available from both ``attr.exceptions`` and ``attrs.exceptions`` and are the same thing.
That means that it doesn't matter from from which namespace they've been raised and/or caught:

.. doctest::

   >>> import attrs, attr
   >>> try:
   ...     raise attrs.exceptions.FrozenError()
   ... except attr.exceptions.FrozenError:
   ...     print("this works!")
   this works!

.. autoexception:: attrs.exceptions.PythonTooOldError
.. autoexception:: attrs.exceptions.FrozenError
.. autoexception:: attrs.exceptions.FrozenInstanceError
.. autoexception:: attrs.exceptions.FrozenAttributeError
.. autoexception:: attrs.exceptions.AttrsAttributeNotFoundError
.. autoexception:: attrs.exceptions.NotAnAttrsClassError
.. autoexception:: attrs.exceptions.DefaultAlreadySetError
.. autoexception:: attrs.exceptions.UnannotatedAttributeError
.. autoexception:: attrs.exceptions.NotCallableError

   For example::

       @attr.s(auto_attribs=True)
       class C:
           x: int
           y = attr.ib()  # <- ERROR!


.. _helpers:

Helpers
-------

``attrs`` comes with a bunch of helper methods that make working with it easier:

.. autofunction:: attrs.cmp_using
.. function:: attr.cmp_using

   Same as `attrs.cmp_using`.

.. autofunction:: attrs.fields

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      ...     y = attr.ib()
      >>> attrs.fields(C)
      (Attribute(name='x', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None), Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None))
      >>> attrs.fields(C)[1]
      Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None)
      >>> attrs.fields(C).y is attrs.fields(C)[1]
      True

.. function:: attr.fields

   Same as `attrs.fields`.

.. autofunction:: attrs.fields_dict

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib()
      ...     y = attr.ib()
      >>> attrs.fields_dict(C)
      {'x': Attribute(name='x', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None), 'y': Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None)}
      >>> attr.fields_dict(C)['y']
      Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None)
      >>> attrs.fields_dict(C)['y'] is attrs.fields(C).y
      True

.. function:: attr.fields_dict

   Same as `attrs.fields_dict`.

.. autofunction:: attrs.has

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     pass
      >>> attr.has(C)
      True
      >>> attr.has(object)
      False

.. function:: attr.has

   Same as `attrs.has`.

.. autofunction:: attrs.resolve_types

    For example:

    .. doctest::

        >>> import typing
        >>> @attrs.define
        ... class A:
        ...     a: typing.List['A']
        ...     b: 'B'
        ...
        >>> @attrs.define
        ... class B:
        ...     a: A
        ...
        >>> attrs.fields(A).a.type
        typing.List[ForwardRef('A')]
        >>> attrs.fields(A).b.type
        'B'
        >>> attrs.resolve_types(A, globals(), locals())
        <class 'A'>
        >>> attrs.fields(A).a.type
        typing.List[A]
        >>> attrs.fields(A).b.type
        <class 'B'>

.. function:: attr.resolve_types

   Same as `attrs.resolve_types`.

.. autofunction:: attrs.asdict

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x: int
      ...     y: int
      >>> attrs.asdict(C(1, C(2, 3)))
      {'x': 1, 'y': {'x': 2, 'y': 3}}

.. autofunction:: attr.asdict

.. autofunction:: attrs.astuple

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x = attr.field()
      ...     y = attr.field()
      >>> attrs.astuple(C(1,2))
      (1, 2)

.. autofunction:: attr.astuple


``attrs`` includes some handy helpers for filtering the attributes in `attrs.asdict` and `attrs.astuple`:

.. autofunction:: attrs.filters.include

.. autofunction:: attrs.filters.exclude

.. function:: attr.filters.include

   Same as `attrs.filters.include`.

.. function:: attr.filters.exclude

   Same as `attrs.filters.exclude`.

See :func:`attrs.asdict` for examples.

All objects from ``attrs.filters`` are also available from ``attr.filters``.

----

.. autofunction:: attrs.evolve

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x: int
      ...     y: int
      >>> i1 = C(1, 2)
      >>> i1
      C(x=1, y=2)
      >>> i2 = attrs.evolve(i1, y=3)
      >>> i2
      C(x=1, y=3)
      >>> i1 == i2
      False

   ``evolve`` creates a new instance using ``__init__``.
   This fact has several implications:

   * private attributes should be specified without the leading underscore, just like in ``__init__``.
   * attributes with ``init=False`` can't be set with ``evolve``.
   * the usual ``__init__`` validators will validate the new values.

.. function:: attr.evolve

   Same as `attrs.evolve`.

.. autofunction:: attrs.validate

   For example:

   .. doctest::

      >>> @attrs.define(on_setattr=attrs.setters.NO_OP)
      ... class C:
      ...     x = attrs.field(validator=attrs.validators.instance_of(int))
      >>> i = C(1)
      >>> i.x = "1"
      >>> attrs.validate(i)
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <class 'int'> (got '1' that is a <class 'str'>).", ...)

.. function:: attr.validate

   Same as `attrs.validate`.


Validators can be globally disabled if you want to run them only in development and tests but not in production because you fear their performance impact:

.. autofunction:: set_run_validators

.. autofunction:: get_run_validators


.. _api_validators:

Validators
----------

``attrs`` comes with some common validators in the ``attrs.validators`` module.
All objects from ``attrs.converters`` are also available from ``attr.converters``.


.. autofunction:: attrs.validators.lt

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x = attrs.field(validator=attrs.validators.lt(42))
      >>> C(41)
      C(x=41)
      >>> C(42)
      Traceback (most recent call last):
         ...
      ValueError: ("'x' must be < 42: 42")

.. autofunction:: attrs.validators.le

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C(object):
      ...     x = attrs.field(validator=attr.validators.le(42))
      >>> C(42)
      C(x=42)
      >>> C(43)
      Traceback (most recent call last):
         ...
      ValueError: ("'x' must be <= 42: 43")

.. autofunction:: attrs.validators.ge

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x = attrs.field(validator=attrs.validators.ge(42))
      >>> C(42)
      C(x=42)
      >>> C(41)
      Traceback (most recent call last):
         ...
      ValueError: ("'x' must be => 42: 41")

.. autofunction:: attrs.validators.gt

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x = attr.field(validator=attrs.validators.gt(42))
      >>> C(43)
      C(x=43)
      >>> C(42)
      Traceback (most recent call last):
         ...
      ValueError: ("'x' must be > 42: 42")

.. autofunction:: attrs.validators.max_len

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x = attrs.field(validator=attrs.validators.max_len(4))
      >>> C("spam")
      C(x='spam')
      >>> C("bacon")
      Traceback (most recent call last):
         ...
      ValueError: ("Length of 'x' must be <= 4: 5")

.. autofunction:: attrs.validators.instance_of

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x = attrs.field(validator=attrs.validators.instance_of(int))
      >>> C(42)
      C(x=42)
      >>> C("42")
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <type 'int'> (got '42' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, validator=<instance_of validator for type <type 'int'>>, type=None, kw_only=False), <type 'int'>, '42')
      >>> C(None)
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <type 'int'> (got None that is a <type 'NoneType'>).", Attribute(name='x', default=NOTHING, validator=<instance_of validator for type <type 'int'>>, repr=True, cmp=True, hash=None, init=True, type=None, kw_only=False), <type 'int'>, None)

.. autofunction:: attrs.validators.in_

   For example:

   .. doctest::

       >>> import enum
       >>> class State(enum.Enum):
       ...     ON = "on"
       ...     OFF = "off"
       >>> @attrs.define
       ... class C:
       ...     state = attrs.field(validator=attrs.validators.in_(State))
       ...     val = attrs.field(validator=attrs.validators.in_([1, 2, 3]))
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

.. autofunction:: attrs.validators.provides

.. autofunction:: attrs.validators.and_

   For convenience, it's also possible to pass a list to `attrs.field`'s validator argument.

   Thus the following two statements are equivalent::

      x = attrs.field(validator=attrs.validators.and_(v1, v2, v3))
      x = attrs.field(validator=[v1, v2, v3])

.. autofunction:: attrs.validators.optional

   For example:

   .. doctest::

      >>> @attrs.define
      ... class C:
      ...     x = attrs.field(validator=attrs.validators.optional(attr.validators.instance_of(int)))
      >>> C(42)
      C(x=42)
      >>> C("42")
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <type 'int'> (got '42' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, validator=<instance_of validator for type <type 'int'>>, type=None, kw_only=False), <type 'int'>, '42')
      >>> C(None)
      C(x=None)


.. autofunction:: attrs.validators.is_callable

    For example:

    .. doctest::

        >>> @attrs.define
        ... class C:
        ...     x = attrs.field(validator=attrs.validators.is_callable())
        >>> C(isinstance)
        C(x=<built-in function isinstance>)
        >>> C("not a callable")
        Traceback (most recent call last):
            ...
        attr.exceptions.NotCallableError: 'x' must be callable (got 'not a callable' that is a <class 'str'>).


.. autofunction:: attrs.validators.matches_re

    For example:

    .. doctest::

        >>> @attrs.define
        ... class User:
        ...     email = attrs.field(validator=attrs.validators.matches_re(
        ...         "(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)"))
        >>> User(email="user@example.com")
        User(email='user@example.com')
        >>> User(email="user@example.com@test.com")
        Traceback (most recent call last):
            ...
        ValueError: ("'email' must match regex '(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\\\.[a-zA-Z0-9-.]+$)' ('user@example.com@test.com' doesn't)", Attribute(name='email', default=NOTHING, validator=<matches_re validator for pattern re.compile('(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\.[a-zA-Z0-9-.]+$)')>, repr=True, cmp=True, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False), re.compile('(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\.[a-zA-Z0-9-.]+$)'), 'user@example.com@test.com')


.. autofunction:: attrs.validators.deep_iterable

    For example:

    .. doctest::

        >>> @attrs.define
        ... class C:
        ...     x = attrs.field(validator=attrs.validators.deep_iterable(
        ...     member_validator=attrs.validators.instance_of(int),
        ...     iterable_validator=attrs.validators.instance_of(list)
        ...     ))
        >>> C(x=[1, 2, 3])
        C(x=[1, 2, 3])
        >>> C(x=set([1, 2, 3]))
        Traceback (most recent call last):
            ...
        TypeError: ("'x' must be <class 'list'> (got {1, 2, 3} that is a <class 'set'>).", Attribute(name='x', default=NOTHING, validator=<deep_iterable validator for <instance_of validator for type <class 'list'>> iterables of <instance_of validator for type <class 'int'>>>, repr=True, cmp=True, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False), <class 'list'>, {1, 2, 3})
        >>> C(x=[1, 2, "3"])
        Traceback (most recent call last):
            ...
        TypeError: ("'x' must be <class 'int'> (got '3' that is a <class 'str'>).", Attribute(name='x', default=NOTHING, validator=<deep_iterable validator for <instance_of validator for type <class 'list'>> iterables of <instance_of validator for type <class 'int'>>>, repr=True, cmp=True, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False), <class 'int'>, '3')


.. autofunction:: attrs.validators.deep_mapping

    For example:

    .. doctest::

        >>> @attrs.define
        ... class C:
        ...     x = attrs.field(validator=attrs.validators.deep_mapping(
        ...         key_validator=attrs.validators.instance_of(str),
        ...         value_validator=attrs.validators.instance_of(int),
        ...         mapping_validator=attrs.validators.instance_of(dict)
        ...     ))
        >>> C(x={"a": 1, "b": 2})
        C(x={'a': 1, 'b': 2})
        >>> C(x=None)
        Traceback (most recent call last):
            ...
        TypeError: ("'x' must be <class 'dict'> (got None that is a <class 'NoneType'>).", Attribute(name='x', default=NOTHING, validator=<deep_mapping validator for objects mapping <instance_of validator for type <class 'str'>> to <instance_of validator for type <class 'int'>>>, repr=True, cmp=True, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False), <class 'dict'>, None)
        >>> C(x={"a": 1.0, "b": 2})
        Traceback (most recent call last):
            ...
        TypeError: ("'x' must be <class 'int'> (got 1.0 that is a <class 'float'>).", Attribute(name='x', default=NOTHING, validator=<deep_mapping validator for objects mapping <instance_of validator for type <class 'str'>> to <instance_of validator for type <class 'int'>>>, repr=True, cmp=True, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False), <class 'int'>, 1.0)
        >>> C(x={"a": 1, 7: 2})
        Traceback (most recent call last):
            ...
        TypeError: ("'x' must be <class 'str'> (got 7 that is a <class 'int'>).", Attribute(name='x', default=NOTHING, validator=<deep_mapping validator for objects mapping <instance_of validator for type <class 'str'>> to <instance_of validator for type <class 'int'>>>, repr=True, cmp=True, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False), <class 'str'>, 7)

Validators can be both globally and locally disabled:

.. autofunction:: attrs.validators.set_disabled

.. autofunction:: attrs.validators.get_disabled

.. autofunction:: attrs.validators.disabled


Converters
----------

All objects from ``attrs.converters`` are also available from ``attr.converters``.

.. autofunction:: attrs.converters.pipe

   For convenience, it's also possible to pass a list to `attr.ib`'s converter argument.

   Thus the following two statements are equivalent::

      x = attr.ib(converter=attr.converter.pipe(c1, c2, c3))
      x = attr.ib(converter=[c1, c2, c3])

.. autofunction:: attrs.converters.optional

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(converter=attr.converters.optional(int))
      >>> C(None)
      C(x=None)
      >>> C(42)
      C(x=42)


.. autofunction:: attrs.converters.default_if_none

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(
      ...         converter=attr.converters.default_if_none("")
      ...     )
      >>> C(None)
      C(x='')


.. autofunction:: attrs.converters.to_bool

   For example:

   .. doctest::

      >>> @attr.s
      ... class C(object):
      ...     x = attr.ib(
      ...         converter=attr.converters.to_bool
      ...     )
      >>> C("yes")
      C(x=True)
      >>> C(0)
      C(x=False)
      >>> C("foo")
      Traceback (most recent call last):
         File "<stdin>", line 1, in <module>
      ValueError: Cannot convert value to bool: foo



.. _api_setters:

Setters
-------

These are helpers that you can use together with `attrs.define`'s and `attrs.fields`'s ``on_setattr`` arguments.
All setters in ``attrs.setters`` are also available from ``attr.setters``.

.. autofunction:: attrs.setters.frozen
.. autofunction:: attrs.setters.validate
.. autofunction:: attrs.setters.convert
.. autofunction:: attrs.setters.pipe
.. autodata:: attrs.setters.NO_OP

   For example, only ``x`` is frozen here:

   .. doctest::

     >>> @attrs.define(on_setattr=attr.setters.frozen)
     ... class C:
     ...     x = attr.field()
     ...     y = attr.field(on_setattr=attr.setters.NO_OP)
     >>> c = C(1, 2)
     >>> c.y = 3
     >>> c.y
     3
     >>> c.x = 4
     Traceback (most recent call last):
         ...
     attrs.exceptions.FrozenAttributeError: ()

   N.B. Please use `attrs.define`'s *frozen* argument (or `attrs.frozen`) to freeze whole classes; it is more efficient.


Deprecated APIs
---------------

.. _version-info:

To help you write backward compatible code that doesn't throw warnings on modern releases, the ``attr`` module has an ``__version_info__`` attribute as of version 19.2.0.
It behaves similarly to `sys.version_info` and is an instance of `VersionInfo`:

.. autoclass:: VersionInfo

   With its help you can write code like this:

   >>> if getattr(attr, "__version_info__", (0,)) >= (19, 2):
   ...     cmp_off = {"eq": False}
   ... else:
   ...     cmp_off = {"cmp": False}
   >>> cmp_off == {"eq":  False}
   True
   >>> @attr.s(**cmp_off)
   ... class C(object):
   ...     pass


----

The serious business aliases used to be called ``attr.attributes`` and ``attr.attr``.
There are no plans to remove them but they shouldn't be used in new code.

.. autofunction:: assoc
