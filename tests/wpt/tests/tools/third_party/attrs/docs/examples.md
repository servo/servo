# *attrs* by Example

## Basics

The simplest possible usage is:

```{doctest}
>>> from attrs import define, field
>>> @define
... class Empty:
...     pass
>>> Empty()
Empty()
>>> Empty() == Empty()
True
>>> Empty() is Empty()
False
```

So in other words: *attrs* is useful even without actual attributes!

But you'll usually want some data on your classes, so let's add some:

```{doctest}
>>> @define
... class Coordinates:
...     x: int
...     y: int
```

By default, all features are added, so you immediately have a fully functional data class with a nice `repr` string and comparison methods.

```{doctest}
>>> c1 = Coordinates(1, 2)
>>> c1
Coordinates(x=1, y=2)
>>> c2 = Coordinates(x=2, y=1)
>>> c2
Coordinates(x=2, y=1)
>>> c1 == c2
False
```

As shown, the generated `__init__` method allows for both positional and keyword arguments.

---

Unlike Data Classes, *attrs* doesn't force you to use type annotations.
So, the previous example could also have been written as:

```{doctest}
>>> @define
... class Coordinates:
...     x = field()
...     y = field()
>>> Coordinates(1, 2)
Coordinates(x=1, y=2)
```

:::{caution}
If a class body contains a field that is defined using {func}`attrs.field` (or {func}`attr.ib`), but **lacks a type annotation**, *attrs* switches to a no-typing mode and ignores fields that have type annotations but are not defined using {func}`attrs.field` (or {func}`attr.ib`).
:::

---

For private attributes, *attrs* will strip the leading underscores for keyword arguments:

```{doctest}
>>> @define
... class C:
...     _x: int
>>> C(x=1)
C(_x=1)
```

If you want to initialize your private attributes yourself, you can do that too:

```{doctest}
>>> @define
... class C:
...     _x: int = field(init=False, default=42)
>>> C()
C(_x=42)
>>> C(23)
Traceback (most recent call last):
   ...
TypeError: __init__() takes exactly 1 argument (2 given)
```

If you prefer to expose your privates, you can use keyword argument aliases:

```{doctest}
>>> @define
... class C:
...     _x: int = field(alias="_x")
>>> C(_x=1)
C(_x=1)
```

An additional way of defining attributes is supported too.
This is useful in times when you want to enhance classes that are not yours (nice `__repr__` for Django models anyone?):

```{doctest}
>>> class SomethingFromSomeoneElse:
...     def __init__(self, x):
...         self.x = x
>>> SomethingFromSomeoneElse = define(
...     these={
...         "x": field()
...     }, init=False)(SomethingFromSomeoneElse)
>>> SomethingFromSomeoneElse(1)
SomethingFromSomeoneElse(x=1)
```

[Subclassing is bad for you](https://www.youtube.com/watch?v=3MNVP9-hglc), but *attrs* will still do what you'd hope for:

```{doctest}
>>> @define(slots=False)
... class A:
...     a: int
...     def get_a(self):
...         return self.a
>>> @define(slots=False)
... class B:
...     b: int
>>> @define(slots=False)
... class C(B, A):
...     c: int
>>> i = C(1, 2, 3)
>>> i
C(a=1, b=2, c=3)
>>> i == C(1, 2, 3)
True
>>> i.get_a()
1
```

{term}`Slotted classes <slotted classes>`, which are the default for the new APIs, don't play well with multiple inheritance so we don't use them in the example.

The order of the attributes is defined by the [MRO](https://www.python.org/download/releases/2.3/mro/).


### Keyword-only Attributes

You can also add [keyword-only](https://docs.python.org/3/glossary.html#keyword-only-parameter) attributes:

```{doctest}
>>> @define
... class A:
...     a: int = field(kw_only=True)
>>> A()
Traceback (most recent call last):
...
TypeError: A() missing 1 required keyword-only argument: 'a'
>>> A(a=1)
A(a=1)
```

`kw_only` may also be specified at decorator level, and will apply to all attributes:

```{doctest}
>>> @define(kw_only=True)
... class A:
...     a: int
...     b: int
>>> A(1, 2)
Traceback (most recent call last):
...
TypeError: __init__() takes 1 positional argument but 3 were given
>>> A(a=1, b=2)
A(a=1, b=2)
```

If you create an attribute with `init=False`, the `kw_only` argument is ignored.

Keyword-only attributes allow subclasses to add attributes without default values, even if the base class defines attributes with default values:

```{doctest}
>>> @define
... class A:
...     a: int = 0
>>> @define
... class B(A):
...     b: int = field(kw_only=True)
>>> B(b=1)
B(a=0, b=1)
>>> B()
Traceback (most recent call last):
...
TypeError: B() missing 1 required keyword-only argument: 'b'
```

If you don't set `kw_only=True`, then there is no valid attribute ordering, and you'll get an error:

```{doctest}
>>> @define
... class A:
...     a: int = 0
>>> @define
... class B(A):
...     b: int
Traceback (most recent call last):
...
ValueError: No mandatory attributes allowed after an attribute with a default value or factory.  Attribute in question: Attribute(name='b', default=NOTHING, validator=None, repr=True, cmp=True, hash=None, init=True, converter=None, metadata=mappingproxy({}), type=int, kw_only=False)
```

(asdict)=

## Converting to Collections Types

When you have a class with data, it often is very convenient to transform that class into a {class}`dict` (for example if you want to serialize it to JSON):

```{doctest}
>>> from attrs import asdict
>>> asdict(Coordinates(x=1, y=2))
{'x': 1, 'y': 2}
```

Some fields cannot or should not be transformed.
For that, {func}`attrs.asdict` offers a callback that decides whether an attribute should be included:

```{doctest}
>>> @define
... class User:
...     email: str
...     password: str

>>> @define
... class UserList:
...     users: list[User]

>>> asdict(UserList([User("jane@doe.invalid", "s33kred"),
...                  User("joe@doe.invalid", "p4ssw0rd")]),
...        filter=lambda attr, value: attr.name != "password")
{'users': [{'email': 'jane@doe.invalid'}, {'email': 'joe@doe.invalid'}]}
```

For the common case where you want to [`include`](attrs.filters.include) or [`exclude`](attrs.filters.exclude) certain types, string name or attributes, *attrs* ships with a few helpers:

```{doctest}
>>> from attrs import asdict, filters, fields

>>> @define
... class User:
...     login: str
...     password: str
...     email: str
...     id: int

>>> asdict(
...     User("jane", "s33kred", "jane@example.com", 42),
...     filter=filters.exclude(fields(User).password, "email", int))
{'login': 'jane'}

>>> @define
... class C:
...     x: str
...     y: str
...     z: int

>>> asdict(C("foo", "2", 3),
...        filter=filters.include(int, fields(C).x))
{'x': 'foo', 'z': 3}

>>> asdict(C("foo", "2", 3),
...        filter=filters.include(fields(C).x, "z"))
{'x': 'foo', 'z': 3}
```

:::{note}
Though using string names directly is convenient, mistyping attribute names will silently do the wrong thing and neither Python nor your type checker can help you.
{func}`attrs.fields()` will raise an `AttributeError` when the field doesn't exist while literal string names won't.
Using {func}`attrs.fields()` to get attributes is worth being recommended in most cases.

```{doctest}
>>> asdict(
...     User("jane", "s33kred", "jane@example.com", 42),
...     filter=filters.exclude("passwd")
... )
{'login': 'jane', 'password': 's33kred', 'email': 'jane@example.com', 'id': 42}

>>> asdict(
...     User("jane", "s33kred", "jane@example.com", 42),
...     filter=fields(User).passwd
... )
Traceback (most recent call last):
...
AttributeError: 'UserAttributes' object has no attribute 'passwd'. Did you mean: 'password'?
```
:::

Other times, all you want is a tuple and *attrs* won't let you down:

```{doctest}
>>> import sqlite3
>>> from attrs import astuple

>>> @define
... class Foo:
...    a: int
...    b: int

>>> foo = Foo(2, 3)
>>> with sqlite3.connect(":memory:") as conn:
...    c = conn.cursor()
...    c.execute("CREATE TABLE foo (x INTEGER PRIMARY KEY ASC, y)") #doctest: +ELLIPSIS
...    c.execute("INSERT INTO foo VALUES (?, ?)", astuple(foo)) #doctest: +ELLIPSIS
...    foo2 = Foo(*c.execute("SELECT x, y FROM foo").fetchone())
<sqlite3.Cursor object at ...>
<sqlite3.Cursor object at ...>
>>> foo == foo2
True
```

For more advanced transformations and conversions, we recommend you look at a companion library (such as [*cattrs*](https://catt.rs/)).


## Defaults

Sometimes you want to have default values for your initializer.
And sometimes you even want mutable objects as default values (ever accidentally used `def f(arg=[])`?).
*attrs* has you covered in both cases:

```{doctest}
>>> import collections

>>> @define
... class Connection:
...     socket: int
...     @classmethod
...     def connect(cls, db_string):
...        # ... connect somehow to db_string ...
...        return cls(socket=42)

>>> @define
... class ConnectionPool:
...     db_string: str
...     pool: collections.deque = Factory(collections.deque)
...     debug: bool = False
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
```

More information on why class methods for constructing objects are awesome can be found in this insightful [blog post](https://web.archive.org/web/20210130220433/http://as.ynchrono.us/2014/12/asynchronous-object-initialization.html).

Default factories can also be set using the `factory` argument to {func}`~attrs.field`, and using a decorator.
The method receives the partially initialized instance which enables you to base a default value on other attributes:

```{doctest}
>>> @define
... class C:
...     x: int = 1
...     y: int = field()
...     @y.default
...     def _any_name_except_a_name_of_an_attribute(self):
...         return self.x + 1
...     z: list = field(factory=list)
>>> C()
C(x=1, y=2, z=[])
```

Please keep in mind that the decorator approach *only* works if the attribute in question has a {func}`~attrs.field` assigned to it.
As a result, annotating an attribute with a type is *not* enough if you use `@default`.

(examples-validators)=

## Validators

Although your initializers should do as little as possible (ideally: just initialize your instance according to the arguments!), it can come in handy to do some kind of validation on the arguments.

*attrs* offers two ways to define validators for each attribute and it's up to you to choose which one suits your style and project better.

You can use a decorator:

```{doctest}
>>> @define
... class C:
...     x: int = field()
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
```

...or a callable...

```{doctest}
>>> from attrs import validators

>>> def x_smaller_than_y(instance, attribute, value):
...     if value >= instance.y:
...         raise ValueError("'x' has to be smaller than 'y'!")
>>> @define
... class C:
...     x: int = field(validator=[validators.instance_of(int),
...                               x_smaller_than_y])
...     y: int
>>> C(x=3, y=4)
C(x=3, y=4)
>>> C(x=4, y=3)
Traceback (most recent call last):
   ...
ValueError: 'x' has to be smaller than 'y'!
```

...or both at once:

```{doctest}
>>> @define
... class C:
...     x: int = field(validator=validators.instance_of(int))
...     @x.validator
...     def fits_byte(self, attribute, value):
...         if not 0 <= value < 256:
...             raise ValueError("value out of bounds")
>>> C(128)
C(x=128)
>>> C("128")
Traceback (most recent call last):
   ...
TypeError: ("'x' must be <class 'int'> (got '128' that is a <class 'str'>).", Attribute(name='x', default=NOTHING, validator=[<instance_of validator for type <class 'int'>>, <function fits_byte at 0x10fd7a0d0>], repr=True, cmp=True, hash=True, init=True, metadata=mappingproxy({}), type=int, converter=None, kw_only=False), <class 'int'>, '128')
>>> C(256)
Traceback (most recent call last):
   ...
ValueError: value out of bounds
```

Please note that the decorator approach only works if -- and only if! -- the attribute in question has a {func}`~attrs.field` assigned.
Therefore if you use `@validator`, it is *not* enough to annotate said attribute with a type.

*attrs* ships with a bunch of validators, make sure to [check them out](api-validators) before writing your own:

```{doctest}
>>> @define
... class C:
...     x: int = field(validator=validators.instance_of(int))
>>> C(42)
C(x=42)
>>> C("42")
Traceback (most recent call last):
   ...
TypeError: ("'x' must be <type 'int'> (got '42' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, factory=NOTHING, validator=<instance_of validator for type <type 'int'>>, type=None, kw_only=False), <type 'int'>, '42')
```

If using the old-school {func}`attr.s` decorator, validators only run on initialization by default.
If using the newer {func}`attrs.define` and friends, validators run on initialization *and* on attribute setting.
This behavior can be changed using the *on_setattr* argument.

Check out {ref}`validators` for more details.


## Conversion

Attributes can have a `converter` function specified, which will be called with the attribute's passed-in value to get a new value to use.
This can be useful for doing type-conversions on values that you don't want to force your callers to do.

```{doctest}
>>> @define
... class C:
...     x: int = field(converter=int)
>>> o = C("1")
>>> o.x
1
>>> o.x = "2"
>>> o.x
2
```

If using the old-school {func}`attr.s` decorator, converters only run on initialization by default.
If using the newer {func}`attrs.define` and friends, converters run on initialization *and* on attribute setting.
This behavior can be changed using the *on_setattr* argument.

Check out {ref}`converters` for more details.

(metadata)=

## Metadata

All *attrs* attributes may include arbitrary metadata in the form of a read-only dictionary.

```{doctest}
>>> from attrs import fields

>>> @define
... class C:
...    x = field(metadata={'my_metadata': 1})
>>> fields(C).x.metadata
mappingproxy({'my_metadata': 1})
>>> fields(C).x.metadata['my_metadata']
1
```

Metadata is not used by *attrs*, and is meant to enable rich functionality in third-party libraries.
The metadata dictionary follows the normal dictionary rules:
Keys need to be hashable, and both keys and values are recommended to be immutable.

If you're the author of a third-party library with *attrs* integration, please see [*Extending Metadata*](extending-metadata).


## Types

*attrs* also allows you to associate a type with an attribute using either the *type* argument to using {pep}`526`-annotations or {func}`attrs.field`/{func}`attr.ib`:

```{doctest}
>>> @define
... class C:
...     x: int
>>> fields(C).x.type
<class 'int'>

>>> import attr
>>> @attr.s
... class C:
...     x = attr.ib(type=int)
>>> fields(C).x.type
<class 'int'>
```

If you don't mind annotating *all* attributes, you can even drop the `attrs.field` and assign default values instead:

```{doctest}
>>> import typing

>>> @define
... class AutoC:
...     cls_var: typing.ClassVar[int] = 5  # this one is ignored
...     l: list[int] = Factory(list)
...     x: int = 1
...     foo: str = "every attrib needs a type if auto_attribs=True"
...     bar: typing.Any = None
>>> fields(AutoC).l.type
list[int]
>>> fields(AutoC).x.type
<class 'int'>
>>> fields(AutoC).foo.type
<class 'str'>
>>> fields(AutoC).bar.type
typing.Any
>>> AutoC()
AutoC(l=[], x=1, foo='every attrib needs a type if auto_attribs=True', bar=None)
>>> AutoC.cls_var
5
```

The generated `__init__` method will have an attribute called `__annotations__` that contains this type information.

If your annotations contain strings (e.g. forward references),
you can resolve these after all references have been defined by using {func}`attrs.resolve_types`.
This will replace the *type* attribute in the respective fields.

```{doctest}
>>> from attrs import resolve_types

>>> @define
... class A:
...     a: 'list[A]'
...     b: 'B'
...
>>> @define
... class B:
...     a: A
...
>>> fields(A).a.type
'list[A]'
>>> fields(A).b.type
'B'
>>> resolve_types(A, globals(), locals())
<class 'A'>
>>> fields(A).a.type
list[A]
>>> fields(A).b.type
<class 'B'>
```

:::{note}
If you find yourself using string type annotations to handle forward references, wrap the entire type annotation in quotes instead of only the type you need a forward reference to (so `'list[A]'` instead of `list['A']`).
This is a limitation of the Python typing system.
:::

:::{warning}
*attrs* itself doesn't have any features that work on top of type metadata.
However it's useful for writing your own validators or serialization frameworks.
:::


## Slots

{term}`Slotted classes <slotted classes>` have several advantages on CPython.
Defining `__slots__` by hand is tedious, in *attrs* it's just a matter of using {func}`attrs.define` or passing `slots=True` to {func}`attr.s`:

```{doctest}
>>> @define
... class Coordinates:
...     x: int
...     y: int

>>> import attr

>>> @attr.s(slots=True)
... class Coordinates:
...     x: int
...     y: int
```

{func}`~attrs.define` sets `slots=True` by default.


## Immutability

Sometimes you have instances that shouldn't be changed after instantiation.
Immutability is especially popular in functional programming and is generally a very good thing.
If you'd like to enforce it, *attrs* will try to help:

```{doctest}
>>> from attrs import frozen

>>> @frozen
... class C:
...     x: int
>>> i = C(1)
>>> i.x = 2
Traceback (most recent call last):
   ...
attrs.exceptions.FrozenInstanceError: can't set attribute
>>> i.x
1
```

Please note that true immutability is impossible in Python but it will [get](how-frozen) you 99% there.
By themselves, immutable classes are useful for long-lived objects that should never change; like configurations for example.

In order to use them in regular program flow, you'll need a way to easily create new instances with changed attributes.
In Clojure that function is called [*assoc*](https://clojuredocs.org/clojure.core/assoc) and *attrs* shamelessly imitates it: {func}`attrs.evolve`:

```{doctest}
>>> from attrs import evolve, frozen

>>> @frozen
... class C:
...     x: int
...     y: int
>>> i1 = C(1, 2)
>>> i1
C(x=1, y=2)
>>> i2 = evolve(i1, y=3)
>>> i2
C(x=1, y=3)
>>> i1 == i2
False
```


## Other Goodies

Sometimes you may want to create a class programmatically.
*attrs* gives you {func}`attrs.make_class` for that:

```{doctest}
>>> from attrs import make_class
>>> @define
... class C1:
...     x = field(type=int)
...     y = field()
>>> C2 = make_class("C2", {"x": field(type=int), "y": field()})
>>> fields(C1) == fields(C2)
True
>>> fields(C1).x.type
<class 'int'>
```

You can still have power over the attributes if you pass a dictionary of name: {func}`~attrs.field` mappings and can pass the same arguments as you can to `@attrs.define`:

```{doctest}
>>> C = make_class("C", {"x": field(default=42),
...                      "y": field(default=Factory(list))},
...                repr=False)
>>> i = C()
>>> i  # no repr added!
<__main__.C object at ...>
>>> i.x
42
>>> i.y
[]
```

If you need to dynamically make a class with {func}`~attrs.make_class` and it needs to be a subclass of something else than {class}`object`, use the `bases` argument:

```{doctest}
>>> class D:
...    def __eq__(self, other):
...        return True  # arbitrary example
>>> C = make_class("C", {}, bases=(D,), cmp=False)
>>> isinstance(C(), D)
True
```

Sometimes, you want to have your class's `__init__` method do more than just
the initialization, validation, etc. that gets done for you automatically when
using `@define`.
To do this, just define a `__attrs_post_init__` method in your class.
It will get called at the end of the generated `__init__` method.

```{doctest}
>>> @define
... class C:
...     x: int
...     y: int
...     z: int = field(init=False)
...
...     def __attrs_post_init__(self):
...         self.z = self.x + self.y
>>> obj = C(x=1, y=2)
>>> obj
C(x=1, y=2, z=3)
```

You can exclude single attributes from certain methods:

```{doctest}
>>> @define
... class C:
...     user: str
...     password: str = field(repr=False)
>>> C("me", "s3kr3t")
C(user='me')
```

Alternatively, to influence how the generated `__repr__()` method formats a specific attribute, specify a custom callable to be used instead of the `repr()` built-in function:

```{doctest}
>>> @define
... class C:
...     user: str
...     password: str = field(repr=lambda value: '***')
>>> C("me", "s3kr3t")
C(user='me', password=***)
```
