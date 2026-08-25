API Reference
=============

.. module:: attrs

*attrs* works by decorating a class using `attrs.define` or `attr.s` and then defining attributes on the class using `attrs.field`, `attr.ib`, or type annotations.

What follows is the API explanation, if you'd like a more hands-on tutorial, have a look at `examples`.

If you're confused by the many names, please check out `names` for clarification, but the `TL;DR <https://en.wikipedia.org/wiki/TL;DR>`_ is that as of version 21.3.0, *attrs* consists of **two** top-level package names:

- The classic ``attr`` that powers the venerable `attr.s` and `attr.ib`.
- The newer ``attrs`` that only contains most modern APIs and relies on `attrs.define` and `attrs.field` to define your classes.
  Additionally it offers some ``attr`` APIs with nicer defaults (e.g. `attrs.asdict`).

The ``attrs`` namespace is built *on top of* ``attr`` -- which will *never* go away -- and is just as stable, since it doesn't constitute a rewrite.
To keep repetition low and this document at a reasonable size, the ``attr`` namespace is `documented on a separate page <api-attr>`, though.


Core
----

.. autodata:: attrs.NOTHING
   :no-value:

.. autofunction:: attrs.define

.. function:: mutable(same_as_define)

   Same as `attrs.define`.

   .. versionadded:: 20.1.0

.. function:: frozen(same_as_define)

   Behaves the same as `attrs.define` but sets *frozen=True* and *on_setattr=None*.

   .. versionadded:: 20.1.0

.. autofunction:: field

.. autoclass:: Attribute
   :members: evolve

   For example:

   .. doctest::

      >>> import attrs
      >>> from attrs import define, field

      >>> @define
      ... class C:
      ...     x = field()
      >>> attrs.fields(C).x
      Attribute(name='x', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None, alias='x')


.. autofunction:: make_class

   This is handy if you want to programmatically create classes.

   For example:

   .. doctest::

      >>> C1 = attrs.make_class("C1", ["x", "y"])
      >>> C1(1, 2)
      C1(x=1, y=2)
      >>> C2 = attrs.make_class("C2", {
      ...     "x": field(default=42),
      ...     "y": field(factory=list)
      ... })
      >>> C2()
      C2(x=42, y=[])


.. autoclass:: Factory

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(default=attrs.Factory(list))
      ...     y = field(default=attrs.Factory(
      ...         lambda self: set(self.x),
      ...         takes_self=True)
      ...     )
      >>> C()
      C(x=[], y=set())
      >>> C([1, 2, 3])
      C(x=[1, 2, 3], y={1, 2, 3})


Exceptions
----------

.. module:: attrs.exceptions

All exceptions are available from both ``attr.exceptions`` and ``attrs.exceptions`` and are the same thing.
That means that it doesn't matter from from which namespace they've been raised and/or caught:

.. doctest::

   >>> import attrs, attr
   >>> try:
   ...     raise attrs.exceptions.FrozenError()
   ... except attr.exceptions.FrozenError:
   ...     print("this works!")
   this works!

.. autoexception:: PythonTooOldError
.. autoexception:: FrozenError
.. autoexception:: FrozenInstanceError
.. autoexception:: FrozenAttributeError
.. autoexception:: AttrsAttributeNotFoundError
.. autoexception:: NotAnAttrsClassError
.. autoexception:: DefaultAlreadySetError
.. autoexception:: NotCallableError
.. autoexception:: UnannotatedAttributeError

   For example::

       @attr.s(auto_attribs=True)
       class C:
           x: int
           y = attr.ib()  # <- ERROR!


.. _helpers:

Helpers
-------

*attrs* comes with a bunch of helper methods that make working with it easier:

.. currentmodule:: attrs

.. autofunction:: attrs.cmp_using

.. autofunction:: attrs.fields

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field()
      ...     y = field()
      >>> attrs.fields(C)
      (Attribute(name='x', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None, alias='x'), Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None, alias='y'))
      >>> attrs.fields(C)[1]
      Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None, alias='y')
      >>> attrs.fields(C).y is attrs.fields(C)[1]
      True

.. autofunction:: attrs.fields_dict

   For example:

   .. doctest::

      >>> @attr.s
      ... class C:
      ...     x = attr.ib()
      ...     y = attr.ib()
      >>> attrs.fields_dict(C)
      {'x': Attribute(name='x', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None, alias='x'), 'y': Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None, alias='y')}
      >>> attr.fields_dict(C)['y']
      Attribute(name='y', default=NOTHING, validator=None, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None, alias='y')
      >>> attrs.fields_dict(C)['y'] is attrs.fields(C).y
      True

.. autofunction:: attrs.has

   For example:

   .. doctest::

      >>> @attr.s
      ... class C:
      ...     pass
      >>> attr.has(C)
      True
      >>> attr.has(object)
      False

.. autofunction:: attrs.resolve_types

    For example:

    .. doctest::

        >>> import typing
        >>> @define
        ... class A:
        ...     a: typing.List['A']
        ...     b: 'B'
        ...
        >>> @define
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

.. autofunction:: attrs.asdict

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x: int
      ...     y: int
      >>> attrs.asdict(C(1, C(2, 3)))
      {'x': 1, 'y': {'x': 2, 'y': 3}}

.. autofunction:: attrs.astuple

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field()
      ...     y = field()
      >>> attrs.astuple(C(1,2))
      (1, 2)

.. module:: attrs.filters

*attrs* includes helpers for filtering the attributes in `attrs.asdict` and `attrs.astuple`:

.. autofunction:: include

.. autofunction:: exclude

See :func:`attrs.asdict` for examples.

All objects from ``attrs.filters`` are also available from ``attr.filters`` (it's the same module in a different namespace).

----

.. currentmodule:: attrs

.. autofunction:: attrs.evolve

   For example:

   .. doctest::

      >>> @define
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

.. autofunction:: attrs.validate

   For example:

   .. doctest::

      >>> @define(on_setattr=attrs.setters.NO_OP)
      ... class C:
      ...     x = field(validator=attrs.validators.instance_of(int))
      >>> i = C(1)
      >>> i.x = "1"
      >>> attrs.validate(i)
      Traceback (most recent call last):
         ...
      TypeError: ("'x' must be <class 'int'> (got '1' that is a <class 'str'>).", ...)


.. _api-validators:

Validators
----------

.. module:: attrs.validators

*attrs* comes with some common validators in the ``attrs.validators`` module.
All objects from ``attrs.validators`` are also available from ``attr.validators`` (it's the same module in a different namespace).

.. autofunction:: attrs.validators.lt

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(validator=attrs.validators.lt(42))
      >>> C(41)
      C(x=41)
      >>> C(42)
      Traceback (most recent call last):
         ...
      ValueError: ("'x' must be < 42: 42")

.. autofunction:: attrs.validators.le

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(validator=attrs.validators.le(42))
      >>> C(42)
      C(x=42)
      >>> C(43)
      Traceback (most recent call last):
         ...
      ValueError: ("'x' must be <= 42: 43")

.. autofunction:: attrs.validators.ge

   For example:

   .. doctest::

      >>> @define
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

      >>> @define
      ... class C:
      ...     x = field(validator=attrs.validators.gt(42))
      >>> C(43)
      C(x=43)
      >>> C(42)
      Traceback (most recent call last):
         ...
      ValueError: ("'x' must be > 42: 42")

.. autofunction:: attrs.validators.max_len

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(validator=attrs.validators.max_len(4))
      >>> C("spam")
      C(x='spam')
      >>> C("bacon")
      Traceback (most recent call last):
         ...
      ValueError: ("Length of 'x' must be <= 4: 5")

.. autofunction:: attrs.validators.min_len

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(validator=attrs.validators.min_len(1))
      >>> C("bacon")
      C(x='bacon')
      >>> C("")
      Traceback (most recent call last):
         ...
      ValueError: ("Length of 'x' must be => 1: 0")

.. autofunction:: attrs.validators.instance_of

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(validator=attrs.validators.instance_of(int))
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
      >>> @define
      ... class C:
      ...     state = field(validator=attrs.validators.in_(State))
      ...     val = field(validator=attrs.validators.in_([1, 2, 3]))
      >>> C(State.ON, 1)
      C(state=<State.ON: 'on'>, val=1)
      >>> C("On", 1)
      Traceback (most recent call last):
         ...
      ValueError: 'state' must be in <enum 'State'> (got 'On'), Attribute(name='state', default=NOTHING, validator=<in_ validator with options <enum 'State'>>, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None), <enum 'State'>, 'on')
      >>> C(State.ON, 4)
      Traceback (most recent call last):
      ...
      ValueError: 'val' must be in [1, 2, 3] (got 4), Attribute(name='val', default=NOTHING, validator=<in_ validator with options [1, 2, 3]>, repr=True, eq=True, eq_key=None, order=True, order_key=None, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False, inherited=False, on_setattr=None), [1, 2, 3], 4)

.. autofunction:: attrs.validators.provides

.. autofunction:: attrs.validators.and_

   For convenience, it's also possible to pass a list to `attrs.field`'s validator argument.

   Thus the following two statements are equivalent::

      x = field(validator=attrs.validators.and_(v1, v2, v3))
      x = field(validator=[v1, v2, v3])

.. autofunction:: attrs.validators.not_

   For example:

   .. doctest::

      >>> reserved_names = {"id", "time", "source"}
      >>> @define
      ... class Measurement:
      ...     tags = field(
      ...         validator=attrs.validators.deep_mapping(
      ...             key_validator=attrs.validators.not_(
      ...                 attrs.validators.in_(reserved_names),
      ...                 msg="reserved tag key",
      ...             ),
      ...             value_validator=attrs.validators.instance_of((str, int)),
      ...         )
      ...     )
      >>> Measurement(tags={"source": "universe"})
      Traceback (most recent call last):
         ...
      ValueError: ("reserved tag key", Attribute(name='tags', default=NOTHING, validator=<not_ validator wrapping <in_ validator with options {'id', 'time', 'source'}>, capturing (<class 'ValueError'>, <class 'TypeError'>)>, type=None, kw_only=False), <in_ validator with options {'id', 'time', 'source'}>, {'source_': 'universe'}, (<class 'ValueError'>, <class 'TypeError'>))
      >>> Measurement(tags={"source_": "universe"})
      Measurement(tags={'source_': 'universe'})


.. autofunction:: attrs.validators.optional

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(
      ...         validator=attrs.validators.optional(
      ...             attrs.validators.instance_of(int)
      ...         ))
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

        >>> @define
        ... class C:
        ...     x = field(validator=attrs.validators.is_callable())
        >>> C(isinstance)
        C(x=<built-in function isinstance>)
        >>> C("not a callable")
        Traceback (most recent call last):
            ...
        attr.exceptions.NotCallableError: 'x' must be callable (got 'not a callable' that is a <class 'str'>).


.. autofunction:: attrs.validators.matches_re

    For example:

    .. doctest::

        >>> @define
        ... class User:
        ...     email = field(validator=attrs.validators.matches_re(
        ...         r"(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$)"))
        >>> User(email="user@example.com")
        User(email='user@example.com')
        >>> User(email="user@example.com@test.com")
        Traceback (most recent call last):
            ...
        ValueError: ("'email' must match regex '(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\\\.[a-zA-Z0-9-.]+$)' ('user@example.com@test.com' doesn't)", Attribute(name='email', default=NOTHING, validator=<matches_re validator for pattern re.compile('(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\.[a-zA-Z0-9-.]+$)')>, repr=True, cmp=True, hash=None, init=True, metadata=mappingproxy({}), type=None, converter=None, kw_only=False), re.compile('(^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\\.[a-zA-Z0-9-.]+$)'), 'user@example.com@test.com')


.. autofunction:: attrs.validators.deep_iterable

    For example:

    .. doctest::

        >>> @define
        ... class C:
        ...     x = field(validator=attrs.validators.deep_iterable(
        ...             member_validator=attrs.validators.instance_of(int),
        ...             iterable_validator=attrs.validators.instance_of(list)
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

        >>> @define
        ... class C:
        ...     x = field(validator=attrs.validators.deep_mapping(
        ...             key_validator=attrs.validators.instance_of(str),
        ...             value_validator=attrs.validators.instance_of(int),
        ...             mapping_validator=attrs.validators.instance_of(dict)
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

.. module:: attrs.converters

All objects from ``attrs.converters`` are also available from ``attr.converters`` (it's the same module in a different namespace).

.. autofunction:: attrs.converters.pipe

   For convenience, it's also possible to pass a list to `attrs.field` / `attr.ib`'s converter arguments.

   Thus the following two statements are equivalent::

      x = attrs.field(converter=attrs.converter.pipe(c1, c2, c3))
      x = attrs.field(converter=[c1, c2, c3])

.. autofunction:: attrs.converters.optional

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(converter=attrs.converters.optional(int))
      >>> C(None)
      C(x=None)
      >>> C(42)
      C(x=42)


.. autofunction:: attrs.converters.default_if_none

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(
      ...         converter=attrs.converters.default_if_none("")
      ...     )
      >>> C(None)
      C(x='')


.. autofunction:: attrs.converters.to_bool

   For example:

   .. doctest::

      >>> @define
      ... class C:
      ...     x = field(
      ...         converter=attrs.converters.to_bool
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

.. module:: attrs.setters

These are helpers that you can use together with `attrs.define`'s and `attrs.fields`'s ``on_setattr`` arguments.
All setters in ``attrs.setters`` are also available from ``attr.setters`` (it's the same module in a different namespace).

.. autofunction:: frozen
.. autofunction:: validate
.. autofunction:: convert
.. autofunction:: pipe

.. data:: NO_OP

   Sentinel for disabling class-wide *on_setattr* hooks for certain attributes.

   Does not work in `attrs.setters.pipe` or within lists.

   .. versionadded:: 20.1.0

   For example, only ``x`` is frozen here:

   .. doctest::

     >>> @define(on_setattr=attr.setters.frozen)
     ... class C:
     ...     x = field()
     ...     y = field(on_setattr=attr.setters.NO_OP)
     >>> c = C(1, 2)
     >>> c.y = 3
     >>> c.y
     3
     >>> c.x = 4
     Traceback (most recent call last):
         ...
     attrs.exceptions.FrozenAttributeError: ()

   N.B. Please use `attrs.define`'s *frozen* argument (or `attrs.frozen`) to freeze whole classes; it is more efficient.
