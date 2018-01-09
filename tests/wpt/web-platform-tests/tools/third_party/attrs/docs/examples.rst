.. _examples:

``attrs`` by Example
====================


Basics
------

The simplest possible usage is:

.. doctest::

   >>> import attr
   >>> @attr.s
   ... class Empty(object):
   ...     pass
   >>> Empty()
   Empty()
   >>> Empty() == Empty()
   True
   >>> Empty() is Empty()
   False

So in other words: ``attrs`` is useful even without actual attributes!

But you'll usually want some data on your classes, so let's add some:

.. doctest::

   >>> @attr.s
   ... class Coordinates(object):
   ...     x = attr.ib()
   ...     y = attr.ib()

By default, all features are added, so you immediately have a fully functional data class with a nice ``repr`` string and comparison methods.

.. doctest::

   >>> c1 = Coordinates(1, 2)
   >>> c1
   Coordinates(x=1, y=2)
   >>> c2 = Coordinates(x=2, y=1)
   >>> c2
   Coordinates(x=2, y=1)
   >>> c1 == c2
   False

As shown, the generated ``__init__`` method allows for both positional and keyword arguments.

If playful naming turns you off, ``attrs`` comes with serious business aliases:

.. doctest::

   >>> from attr import attrs, attrib
   >>> @attrs
   ... class SeriousCoordinates(object):
   ...     x = attrib()
   ...     y = attrib()
   >>> SeriousCoordinates(1, 2)
   SeriousCoordinates(x=1, y=2)
   >>> attr.fields(Coordinates) == attr.fields(SeriousCoordinates)
   True

For private attributes, ``attrs`` will strip the leading underscores for keyword arguments:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     _x = attr.ib()
   >>> C(x=1)
   C(_x=1)

If you want to initialize your private attributes yourself, you can do that too:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     _x = attr.ib(init=False, default=42)
   >>> C()
   C(_x=42)
   >>> C(23)
   Traceback (most recent call last):
      ...
   TypeError: __init__() takes exactly 1 argument (2 given)

An additional way of defining attributes is supported too.
This is useful in times when you want to enhance classes that are not yours (nice ``__repr__`` for Django models anyone?):

.. doctest::

   >>> class SomethingFromSomeoneElse(object):
   ...     def __init__(self, x):
   ...         self.x = x
   >>> SomethingFromSomeoneElse = attr.s(
   ...     these={
   ...         "x": attr.ib()
   ...     }, init=False)(SomethingFromSomeoneElse)
   >>> SomethingFromSomeoneElse(1)
   SomethingFromSomeoneElse(x=1)


`Subclassing is bad for you <https://www.youtube.com/watch?v=3MNVP9-hglc>`_, but ``attrs`` will still do what you'd hope for:

.. doctest::

   >>> @attr.s
   ... class A(object):
   ...     a = attr.ib()
   ...     def get_a(self):
   ...         return self.a
   >>> @attr.s
   ... class B(object):
   ...     b = attr.ib()
   >>> @attr.s
   ... class C(B, A):
   ...     c = attr.ib()
   >>> i = C(1, 2, 3)
   >>> i
   C(a=1, b=2, c=3)
   >>> i == C(1, 2, 3)
   True
   >>> i.get_a()
   1

The order of the attributes is defined by the `MRO <https://www.python.org/download/releases/2.3/mro/>`_.

In Python 3, classes defined within other classes are `detected <https://www.python.org/dev/peps/pep-3155/>`_ and reflected in the ``__repr__``.
In Python 2 though, it's impossible.
Therefore ``@attr.s`` comes with the ``repr_ns`` option to set it manually:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     @attr.s(repr_ns="C")
   ...     class D(object):
   ...         pass
   >>> C.D()
   C.D()

``repr_ns`` works on both Python 2 and 3.
On Python 3 it overrides the implicit detection.


.. _asdict:

Converting to Collections Types
-------------------------------

When you have a class with data, it often is very convenient to transform that class into a :class:`dict` (for example if you want to serialize it to JSON):

.. doctest::

   >>> attr.asdict(Coordinates(x=1, y=2))
   {'x': 1, 'y': 2}

Some fields cannot or should not be transformed.
For that, :func:`attr.asdict` offers a callback that decides whether an attribute should be included:

.. doctest::

   >>> @attr.s
   ... class UserList(object):
   ...     users = attr.ib()
   >>> @attr.s
   ... class User(object):
   ...     email = attr.ib()
   ...     password = attr.ib()
   >>> attr.asdict(UserList([User("jane@doe.invalid", "s33kred"),
   ...                       User("joe@doe.invalid", "p4ssw0rd")]),
   ...             filter=lambda attr, value: attr.name != "password")
   {'users': [{'email': 'jane@doe.invalid'}, {'email': 'joe@doe.invalid'}]}

For the common case where you want to :func:`include <attr.filters.include>` or :func:`exclude <attr.filters.exclude>` certain types or attributes, ``attrs`` ships with a few helpers:

.. doctest::

   >>> @attr.s
   ... class User(object):
   ...     login = attr.ib()
   ...     password = attr.ib()
   ...     id = attr.ib()
   >>> attr.asdict(
   ...     User("jane", "s33kred", 42),
   ...     filter=attr.filters.exclude(attr.fields(User).password, int))
   {'login': 'jane'}
   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib()
   ...     y = attr.ib()
   ...     z = attr.ib()
   >>> attr.asdict(C("foo", "2", 3),
   ...             filter=attr.filters.include(int, attr.fields(C).x))
   {'x': 'foo', 'z': 3}

Other times, all you want is a tuple and ``attrs`` won't let you down:

.. doctest::

   >>> import sqlite3
   >>> import attr
   >>> @attr.s
   ... class Foo:
   ...    a = attr.ib()
   ...    b = attr.ib()
   >>> foo = Foo(2, 3)
   >>> with sqlite3.connect(":memory:") as conn:
   ...    c = conn.cursor()
   ...    c.execute("CREATE TABLE foo (x INTEGER PRIMARY KEY ASC, y)") #doctest: +ELLIPSIS
   ...    c.execute("INSERT INTO foo VALUES (?, ?)", attr.astuple(foo)) #doctest: +ELLIPSIS
   ...    foo2 = Foo(*c.execute("SELECT x, y FROM foo").fetchone())
   <sqlite3.Cursor object at ...>
   <sqlite3.Cursor object at ...>
   >>> foo == foo2
   True




Defaults
--------

Sometimes you want to have default values for your initializer.
And sometimes you even want mutable objects as default values (ever used accidentally ``def f(arg=[])``?).
``attrs`` has you covered in both cases:

.. doctest::

   >>> import collections
   >>> @attr.s
   ... class Connection(object):
   ...     socket = attr.ib()
   ...     @classmethod
   ...     def connect(cls, db_string):
   ...        # ... connect somehow to db_string ...
   ...        return cls(socket=42)
   >>> @attr.s
   ... class ConnectionPool(object):
   ...     db_string = attr.ib()
   ...     pool = attr.ib(default=attr.Factory(collections.deque))
   ...     debug = attr.ib(default=False)
   ...     def get_connection(self):
   ...         try:
   ...             return self.pool.pop()
   ...         except IndexError:
   ...             if self.debug:
   ...                 print("New connection!")
   ...             return Connection.connect(self.db_string)
   ...     def free_connection(self, conn):
   ...         if self.debug:
   ...             print("Connection returned!")
   ...         self.pool.appendleft(conn)
   ...
   >>> cp = ConnectionPool("postgres://localhost")
   >>> cp
   ConnectionPool(db_string='postgres://localhost', pool=deque([]), debug=False)
   >>> conn = cp.get_connection()
   >>> conn
   Connection(socket=42)
   >>> cp.free_connection(conn)
   >>> cp
   ConnectionPool(db_string='postgres://localhost', pool=deque([Connection(socket=42)]), debug=False)

More information on why class methods for constructing objects are awesome can be found in this insightful `blog post <http://as.ynchrono.us/2014/12/asynchronous-object-initialization.html>`_.

Default factories can also be set using a decorator.
The method receives the partially initialized instance which enables you to base a default value on other attributes:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib(default=1)
   ...     y = attr.ib()
   ...     @y.default
   ...     def name_does_not_matter(self):
   ...         return self.x + 1
   >>> C()
   C(x=1, y=2)


.. _examples_validators:

Validators
----------

Although your initializers should do as little as possible (ideally: just initialize your instance according to the arguments!), it can come in handy to do some kind of validation on the arguments.

``attrs`` offers two ways to define validators for each attribute and it's up to you to choose which one suites better your style and project.


Decorator
~~~~~~~~~

The more straightforward way is by using the attribute's ``validator`` method as a decorator.
The method has to accept three arguments:

#. the *instance* that's being validated (aka ``self``),
#. the *attribute* that it's validating, and finally
#. the *value* that is passed for it.

If the value does not pass the validator's standards, it just raises an appropriate exception.

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib()
   ...     @x.validator
   ...     def check(self, attribute, value):
   ...         if value > 42:
   ...             raise ValueError("x must be smaller or equal to 42")
   >>> C(42)
   C(x=42)
   >>> C(43)
   Traceback (most recent call last):
      ...
   ValueError: x must be smaller or equal to 42


Callables
~~~~~~~~~

If you want to re-use your validators, you should have a look at the ``validator`` argument to :func:`attr.ib()`.

It takes either a callable or a list of callables (usually functions) and treats them as validators that receive the same arguments as with the decorator approach.

Since the validators runs *after* the instance is initialized, you can refer to other attributes while validating:

.. doctest::

   >>> def x_smaller_than_y(instance, attribute, value):
   ...     if value >= instance.y:
   ...         raise ValueError("'x' has to be smaller than 'y'!")
   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib(validator=[attr.validators.instance_of(int),
   ...                            x_smaller_than_y])
   ...     y = attr.ib()
   >>> C(x=3, y=4)
   C(x=3, y=4)
   >>> C(x=4, y=3)
   Traceback (most recent call last):
      ...
   ValueError: 'x' has to be smaller than 'y'!

This example also shows of some syntactic sugar for using the :func:`attr.validators.and_` validator: if you pass a list, all validators have to pass.

``attrs`` won't intercept your changes to those attributes but you can always call :func:`attr.validate` on any instance to verify that it's still valid:

.. doctest::

   >>> i = C(4, 5)
   >>> i.x = 5  # works, no magic here
   >>> attr.validate(i)
   Traceback (most recent call last):
      ...
   ValueError: 'x' has to be smaller than 'y'!

``attrs`` ships with a bunch of validators, make sure to :ref:`check them out <api_validators>` before writing your own:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib(validator=attr.validators.instance_of(int))
   >>> C(42)
   C(x=42)
   >>> C("42")
   Traceback (most recent call last):
      ...
   TypeError: ("'x' must be <type 'int'> (got '42' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, factory=NOTHING, validator=<instance_of validator for type <type 'int'>>, type=None), <type 'int'>, '42')

Of course you can mix and match the two approaches at your convenience:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib(validator=attr.validators.instance_of(int))
   ...     @x.validator
   ...     def fits_byte(self, attribute, value):
   ...         if not 0 < value < 256:
   ...             raise ValueError("value out of bounds")
   >>> C(128)
   C(x=128)
   >>> C("128")
   Traceback (most recent call last):
      ...
   TypeError: ("'x' must be <class 'int'> (got '128' that is a <class 'str'>).", Attribute(name='x', default=NOTHING, validator=[<instance_of validator for type <class 'int'>>, <function fits_byte at 0x10fd7a0d0>], repr=True, cmp=True, hash=True, init=True, convert=None, metadata=mappingproxy({}), type=None), <class 'int'>, '128')
   >>> C(256)
   Traceback (most recent call last):
      ...
   ValueError: value out of bounds

And finally you can disable validators globally:

   >>> attr.set_run_validators(False)
   >>> C("128")
   C(x='128')
   >>> attr.set_run_validators(True)
   >>> C("128")
   Traceback (most recent call last):
      ...
   TypeError: ("'x' must be <class 'int'> (got '128' that is a <class 'str'>).", Attribute(name='x', default=NOTHING, validator=[<instance_of validator for type <class 'int'>>, <function fits_byte at 0x10fd7a0d0>], repr=True, cmp=True, hash=True, init=True, convert=None, metadata=mappingproxy({}), type=None), <class 'int'>, '128')


Conversion
----------

Attributes can have a ``convert`` function specified, which will be called with the attribute's passed-in value to get a new value to use.
This can be useful for doing type-conversions on values that you don't want to force your callers to do.

.. doctest::

    >>> @attr.s
    ... class C(object):
    ...     x = attr.ib(convert=int)
    >>> o = C("1")
    >>> o.x
    1

Converters are run *before* validators, so you can use validators to check the final form of the value.

.. doctest::

    >>> def validate_x(instance, attribute, value):
    ...     if value < 0:
    ...         raise ValueError("x must be be at least 0.")
    >>> @attr.s
    ... class C(object):
    ...     x = attr.ib(convert=int, validator=validate_x)
    >>> o = C("0")
    >>> o.x
    0
    >>> C("-1")
    Traceback (most recent call last):
        ...
    ValueError: x must be be at least 0.


.. _metadata:

Metadata
--------

All ``attrs`` attributes may include arbitrary metadata in the form of a read-only dictionary.

.. doctest::

    >>> @attr.s
    ... class C(object):
    ...    x = attr.ib(metadata={'my_metadata': 1})
    >>> attr.fields(C).x.metadata
    mappingproxy({'my_metadata': 1})
    >>> attr.fields(C).x.metadata['my_metadata']
    1

Metadata is not used by ``attrs``, and is meant to enable rich functionality in third-party libraries.
The metadata dictionary follows the normal dictionary rules: keys need to be hashable, and both keys and values are recommended to be immutable.

If you're the author of a third-party library with ``attrs`` integration, please see :ref:`Extending Metadata <extending_metadata>`.


Types
-----

``attrs`` also allows you to associate a type with an attribute using either the *type* argument to :func:`attr.ib` or -- as of Python 3.6 -- using `PEP 526 <https://www.python.org/dev/peps/pep-0526/>`_-annotations:


.. doctest::

   >>> @attr.s
   ... class C:
   ...     x = attr.ib(type=int)
   ...     y: int = attr.ib()
   >>> attr.fields(C).x.type
   <class 'int'>
   >>> attr.fields(C).y.type
   <class 'int'>

If you don't mind annotating *all* attributes, you can even drop the :func:`attr.ib` and assign default values instead:

.. doctest::

   >>> import typing
   >>> @attr.s(auto_attribs=True)
   ... class AutoC:
   ...     cls_var: typing.ClassVar[int] = 5  # this one is ignored
   ...     l: typing.List[int] = attr.Factory(list)
   ...     x: int = 1
   ...     foo: str = attr.ib(
   ...          default="every attrib needs a type if auto_attribs=True"
   ...     )
   ...     bar: typing.Any = None
   >>> attr.fields(AutoC).l.type
   typing.List[int]
   >>> attr.fields(AutoC).x.type
   <class 'int'>
   >>> attr.fields(AutoC).foo.type
   <class 'str'>
   >>> attr.fields(AutoC).bar.type
   typing.Any
   >>> AutoC()
   AutoC(l=[], x=1, foo='every attrib needs a type if auto_attribs=True', bar=None)
   >>> AutoC.cls_var
   5


.. warning::

   ``attrs`` itself doesn't have any features that work on top of type metadata *yet*.
   However it's useful for writing your own validators or serialization frameworks.


.. _slots:

Slots
-----

By default, instances of classes have a dictionary for attribute storage.
This wastes space for objects having very few data attributes.
The space consumption can become significant when creating large numbers of instances.

Normal Python classes can avoid using a separate dictionary for each instance of a class by `defining <https://docs.python.org/3/reference/datamodel.html#slots>`_ ``__slots__``.
For ``attrs`` classes it's enough to set ``slots=True``:

.. doctest::

   >>> @attr.s(slots=True)
   ... class Coordinates(object):
   ...     x = attr.ib()
   ...     y = attr.ib()


.. note::

    ``attrs`` slot classes can inherit from other classes just like non-slot classes, but some of the benefits of slot classes are lost if you do that.
    If you must inherit from other classes, try to inherit only from other slot classes.

Slot classes are a little different than ordinary, dictionary-backed classes:

- Assigning to a non-existent attribute of an instance will result in an ``AttributeError`` being raised.
  Depending on your needs, this might be a good thing since it will let you catch typos early.
  This is not the case if your class inherits from any non-slot classes.

  .. doctest::

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

- Since non-slot classes cannot be turned into slot classes after they have been created, ``attr.s(slots=True)`` will *replace* the class it is applied to with a copy.
  In almost all cases this isn't a problem, but we mention it for the sake of completeness.

  * One notable problem is that certain metaclass features like ``__init_subclass__`` do not work with slot classes.

- Using :mod:`pickle` with slot classes requires pickle protocol 2 or greater.
  Python 2 uses protocol 0 by default so the protocol needs to be specified.
  Python 3 uses protocol 3 by default.
  You can support protocol 0 and 1 by implementing :meth:`__getstate__ <object.__getstate__>` and :meth:`__setstate__ <object.__setstate__>` methods yourself.
  Those methods are created for frozen slot classes because they won't pickle otherwise.
  `Think twice <https://www.youtube.com/watch?v=7KnfGDajDQw>`_ before using :mod:`pickle` though.

- As always with slot classes, you must specify a ``__weakref__`` slot if you wish for the class to be weak-referenceable.
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

All in all, setting ``slots=True`` is usually a very good idea.


Immutability
------------

Sometimes you have instances that shouldn't be changed after instantiation.
Immutability is especially popular in functional programming and is generally a very good thing.
If you'd like to enforce it, ``attrs`` will try to help:

.. doctest::

   >>> @attr.s(frozen=True)
   ... class C(object):
   ...     x = attr.ib()
   >>> i = C(1)
   >>> i.x = 2
   Traceback (most recent call last):
      ...
   attr.exceptions.FrozenInstanceError: can't set attribute
   >>> i.x
   1

Please note that true immutability is impossible in Python but it will :ref:`get <how-frozen>` you 99% there.
By themselves, immutable classes are useful for long-lived objects that should never change; like configurations for example.

In order to use them in regular program flow, you'll need a way to easily create new instances with changed attributes.
In Clojure that function is called `assoc <https://clojuredocs.org/clojure.core/assoc>`_ and ``attrs`` shamelessly imitates it: :func:`attr.evolve`:

.. doctest::

   >>> @attr.s(frozen=True)
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


Other Goodies
-------------

Sometimes you may want to create a class programmatically.
``attrs`` won't let you down and gives you :func:`attr.make_class` :

.. doctest::

   >>> @attr.s
   ... class C1(object):
   ...     x = attr.ib()
   ...     y = attr.ib()
   >>> C2 = attr.make_class("C2", ["x", "y"])
   >>> attr.fields(C1) == attr.fields(C2)
   True

You can still have power over the attributes if you pass a dictionary of name: ``attr.ib`` mappings and can pass arguments to ``@attr.s``:

.. doctest::

   >>> C = attr.make_class("C", {"x": attr.ib(default=42),
   ...                           "y": attr.ib(default=attr.Factory(list))},
   ...                     repr=False)
   >>> i = C()
   >>> i  # no repr added!
   <__main__.C object at ...>
   >>> i.x
   42
   >>> i.y
   []

If you need to dynamically make a class with :func:`attr.make_class` and it needs to be a subclass of something else than ``object``, use the ``bases`` argument:

.. doctest::

  >>> class D(object):
  ...    def __eq__(self, other):
  ...        return True  # arbitrary example
  >>> C = attr.make_class("C", {}, bases=(D,), cmp=False)
  >>> isinstance(C(), D)
  True

Sometimes, you want to have your class's ``__init__`` method do more than just
the initialization, validation, etc. that gets done for you automatically when
using ``@attr.s``.
To do this, just define a ``__attrs_post_init__`` method in your class.
It will get called at the end of the generated ``__init__`` method.

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib()
   ...     y = attr.ib()
   ...     z = attr.ib(init=False)
   ...
   ...     def __attrs_post_init__(self):
   ...         self.z = self.x + self.y
   >>> obj = C(x=1, y=2)
   >>> obj
   C(x=1, y=2, z=3)

Finally, you can exclude single attributes from certain methods:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     user = attr.ib()
   ...     password = attr.ib(repr=False)
   >>> C("me", "s3kr3t")
   C(user='me')
