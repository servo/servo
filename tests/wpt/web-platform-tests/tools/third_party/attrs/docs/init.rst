Initialization
==============

In Python, instance initialization happens in the ``__init__`` method.
Generally speaking, you should keep as little logic as possible in it, and you should think about what the class needs and not how it is going to be instantiated.

Passing complex objects into ``__init__`` and then using them to derive data for the class unnecessarily couples your new class with the old class which makes it harder to test and also will cause problems later.

So assuming you use an ORM and want to extract 2D points from a row object, do not write code like this::

    class Point(object):
        def __init__(self, database_row):
            self.x = database_row.x
            self.y = database_row.y

    pt = Point(row)

Instead, write a `classmethod` that will extract it for you::

   @define
   class Point:
       x: float
       y: float

       @classmethod
       def from_row(cls, row):
           return cls(row.x, row.y)

   pt = Point.from_row(row)

Now you can instantiate ``Point``\ s without creating fake row objects in your tests and you can have as many smart creation helpers as you want, in case more data sources appear.

For similar reasons, we strongly discourage from patterns like::

   pt = Point(**row.attributes)

which couples your classes to the database data model.
Try to design your classes in a way that is clean and convenient to use -- not based on your database format.
The database format can change anytime and you're stuck with a bad class design that is hard to change.
Embrace functions and classmethods as a filter between reality and what's best for you to work with.

If you look for object serialization, there's a bunch of projects listed on our ``attrs`` extensions `Wiki page`_.
Some of them even support nested schemas.


Private Attributes
------------------

One thing people tend to find confusing is the treatment of private attributes that start with an underscore.
``attrs`` follows the doctrine that `there is no such thing as a private argument`_ and strips the underscores from the name when writing the ``__init__`` method signature:

.. doctest::

   >>> import inspect, attr, attrs
   >>> from attr import define
   >>> @define
   ... class C:
   ...    _x: int
   >>> inspect.signature(C.__init__)
   <Signature (self, x: int) -> None>

There really isn't a right or wrong, it's a matter of taste.
But it's important to be aware of it because it can lead to surprising syntax errors:

.. doctest::

   >>> @define
   ... class C:
   ...    _1: int
   Traceback (most recent call last):
      ...
   SyntaxError: invalid syntax

In this case a valid attribute name ``_1`` got transformed into an invalid argument name ``1``.


Defaults
--------

Sometimes you don't want to pass all attribute values to a class.
And sometimes, certain attributes aren't even intended to be passed but you want to allow for customization anyways for easier testing.

This is when default values come into play:

.. doctest::

   >>> from attr import define, field, Factory

   >>> @define
   ... class C:
   ...     a: int = 42
   ...     b: list = field(factory=list)
   ...     c: list = Factory(list)  # syntactic sugar for above
   ...     d: dict = field()
   ...     @d.default
   ...     def _any_name_except_a_name_of_an_attribute(self):
   ...        return {}
   >>> C()
   C(a=42, b=[], c=[], d={})

It's important that the decorated method -- or any other method or property! -- doesn't have the same name as the attribute, otherwise it would overwrite the attribute definition.

Please note that as with function and method signatures, ``default=[]`` will *not* do what you may think it might do:

.. doctest::

   >>> @define
   ... class C:
   ...     x = []
   >>> i = C()
   >>> k = C()
   >>> i.x.append(42)
   >>> k.x
   [42]


This is why ``attrs`` comes with factory options.

.. warning::

   Please note that the decorator based defaults have one gotcha:
   they are executed when the attribute is set, that means depending on the order of attributes, the ``self`` object may not be fully initialized when they're called.

   Therefore you should use ``self`` as little as possible.

   Even the smartest of us can `get confused`_ by what happens if you pass partially initialized objects around.


.. _validators:

Validators
----------

Another thing that definitely *does* belong in ``__init__`` is checking the resulting instance for invariants.
This is why ``attrs`` has the concept of validators.


Decorator
~~~~~~~~~

The most straightforward way is using the attribute's ``validator`` method as a decorator.

The method has to accept three arguments:

#. the *instance* that's being validated (aka ``self``),
#. the *attribute* that it's validating, and finally
#. the *value* that is passed for it.

If the value does not pass the validator's standards, it just raises an appropriate exception.

   >>> @define
   ... class C:
   ...     x: int = field()
   ...     @x.validator
   ...     def _check_x(self, attribute, value):
   ...         if value > 42:
   ...             raise ValueError("x must be smaller or equal to 42")
   >>> C(42)
   C(x=42)
   >>> C(43)
   Traceback (most recent call last):
      ...
   ValueError: x must be smaller or equal to 42

Again, it's important that the decorated method doesn't have the same name as the attribute and that the `attrs.field()` helper is used.


Callables
~~~~~~~~~

If you want to re-use your validators, you should have a look at the ``validator`` argument to `attrs.field`.

It takes either a callable or a list of callables (usually functions) and treats them as validators that receive the same arguments as with the decorator approach.

Since the validators run *after* the instance is initialized, you can refer to other attributes while validating:

.. doctest::

   >>> def x_smaller_than_y(instance, attribute, value):
   ...     if value >= instance.y:
   ...         raise ValueError("'x' has to be smaller than 'y'!")
   >>> @define
   ... class C:
   ...     x = field(validator=[attrs.validators.instance_of(int),
   ...                          x_smaller_than_y])
   ...     y = field()
   >>> C(x=3, y=4)
   C(x=3, y=4)
   >>> C(x=4, y=3)
   Traceback (most recent call last):
      ...
   ValueError: 'x' has to be smaller than 'y'!

This example also shows of some syntactic sugar for using the `attrs.validators.and_` validator: if you pass a list, all validators have to pass.

``attrs`` won't intercept your changes to those attributes but you can always call `attrs.validate` on any instance to verify that it's still valid:
When using `attrs.define` or `attrs.frozen`, ``attrs`` will run the validators even when setting the attribute.

.. doctest::

   >>> i = C(4, 5)
   >>> i.x = 5
   Traceback (most recent call last):
      ...
   ValueError: 'x' has to be smaller than 'y'!

``attrs`` ships with a bunch of validators, make sure to `check them out <api_validators>` before writing your own:

.. doctest::

   >>> @define
   ... class C:
   ...     x = field(validator=attrs.validators.instance_of(int))
   >>> C(42)
   C(x=42)
   >>> C("42")
   Traceback (most recent call last):
      ...
   TypeError: ("'x' must be <type 'int'> (got '42' that is a <type 'str'>).", Attribute(name='x', default=NOTHING, factory=NOTHING, validator=<instance_of validator for type <type 'int'>>, type=None), <type 'int'>, '42')

Of course you can mix and match the two approaches at your convenience.
If you define validators both ways for an attribute, they are both ran:

.. doctest::

   >>> @define
   ... class C:
   ...     x = field(validator=attrs.validators.instance_of(int))
   ...     @x.validator
   ...     def fits_byte(self, attribute, value):
   ...         if not 0 <= value < 256:
   ...             raise ValueError("value out of bounds")
   >>> C(128)
   C(x=128)
   >>> C("128")
   Traceback (most recent call last):
      ...
   TypeError: ("'x' must be <class 'int'> (got '128' that is a <class 'str'>).", Attribute(name='x', default=NOTHING, validator=[<instance_of validator for type <class 'int'>>, <function fits_byte at 0x10fd7a0d0>], repr=True, cmp=True, hash=True, init=True, metadata=mappingproxy({}), type=None, converter=one), <class 'int'>, '128')
   >>> C(256)
   Traceback (most recent call last):
      ...
   ValueError: value out of bounds

And finally you can disable validators globally:

   >>> attrs.validators.set_disabled(True)
   >>> C("128")
   C(x='128')
   >>> attrs.validators.set_disabled(False)
   >>> C("128")
   Traceback (most recent call last):
      ...
   TypeError: ("'x' must be <class 'int'> (got '128' that is a <class 'str'>).", Attribute(name='x', default=NOTHING, validator=[<instance_of validator for type <class 'int'>>, <function fits_byte at 0x10fd7a0d0>], repr=True, cmp=True, hash=True, init=True, metadata=mappingproxy({}), type=None, converter=None), <class 'int'>, '128')

You can achieve the same by using the context manager:

   >>> with attrs.validators.disabled():
   ...     C("128")
   C(x='128')
   >>> C("128")
   Traceback (most recent call last):
      ...
   TypeError: ("'x' must be <class 'int'> (got '128' that is a <class 'str'>).", Attribute(name='x', default=NOTHING, validator=[<instance_of validator for type <class 'int'>>, <function fits_byte at 0x10fd7a0d0>], repr=True, cmp=True, hash=True, init=True, metadata=mappingproxy({}), type=None, converter=None), <class 'int'>, '128')


.. _converters:

Converters
----------

Finally, sometimes you may want to normalize the values coming in.
For that ``attrs`` comes with converters.

Attributes can have a ``converter`` function specified, which will be called with the attribute's passed-in value to get a new value to use.
This can be useful for doing type-conversions on values that you don't want to force your callers to do.

.. doctest::

    >>> @define
    ... class C:
    ...     x = field(converter=int)
    >>> o = C("1")
    >>> o.x
    1

Converters are run *before* validators, so you can use validators to check the final form of the value.

.. doctest::

    >>> def validate_x(instance, attribute, value):
    ...     if value < 0:
    ...         raise ValueError("x must be at least 0.")
    >>> @define
    ... class C:
    ...     x = field(converter=int, validator=validate_x)
    >>> o = C("0")
    >>> o.x
    0
    >>> C("-1")
    Traceback (most recent call last):
        ...
    ValueError: x must be at least 0.


Arguably, you can abuse converters as one-argument validators:

.. doctest::

   >>> C("x")
   Traceback (most recent call last):
       ...
   ValueError: invalid literal for int() with base 10: 'x'


If a converter's first argument has a type annotation, that type will appear in the signature for ``__init__``.
A converter will override an explicit type annotation or ``type`` argument.

.. doctest::

   >>> def str2int(x: str) -> int:
   ...     return int(x)
   >>> @define
   ... class C:
   ...     x = field(converter=str2int)
   >>> C.__init__.__annotations__
   {'return': None, 'x': <class 'str'>}


Hooking Yourself Into Initialization
------------------------------------

Generally speaking, the moment you think that you need finer control over how your class is instantiated than what ``attrs`` offers, it's usually best to use a classmethod factory or to apply the `builder pattern <https://en.wikipedia.org/wiki/Builder_pattern>`_.

However, sometimes you need to do that one quick thing before or after your class is initialized.
And for that ``attrs`` offers three means:

- ``__attrs_pre_init__`` is automatically detected and run *before* ``attrs`` starts initializing.
  This is useful if you need to inject a call to ``super().__init__()``.
- ``__attrs_post_init__`` is automatically detected and run *after* ``attrs`` is done initializing your instance.
  This is useful if you want to derive some attribute from others or perform some kind of validation over the whole instance.
- ``__attrs_init__`` is written and attached to your class *instead* of ``__init__``, if ``attrs`` is told to not write one (i.e. ``init=False`` or a combination of ``auto_detect=True`` and a custom ``__init__``).
  This is useful if you want full control over the initialization process, but don't want to set the attributes by hand.


Pre Init
~~~~~~~~

The sole reason for the existance of ``__attrs_pre_init__`` is to give users the chance to call ``super().__init__()``, because some subclassing-based APIs require that.

.. doctest::

   >>> @define
   ... class C:
   ...     x: int
   ...     def __attrs_pre_init__(self):
   ...         super().__init__()
   >>> C(42)
   C(x=42)

If you need more control, use the custom init approach described next.


Custom Init
~~~~~~~~~~~

If you tell ``attrs`` to not write an ``__init__``, it will write an ``__attrs_init__`` instead, with the same code that it would have used for ``__init__``.
You have full control over the initialization, but also have to type out the types of your arguments etc.
Here's an example of a manual default value:

.. doctest::

   >>> from typing import Optional

   >>> @define
   ... class C:
   ...     x: int
   ...
   ...     def __init__(self, x: int = 42):
   ...         self.__attrs_init__(x)
   >>> C()
   C(x=42)


Post Init
~~~~~~~~~

.. doctest::

   >>> @define
   ... class C:
   ...     x: int
   ...     y: int = field(init=False)
   ...     def __attrs_post_init__(self):
   ...         self.y = self.x + 1
   >>> C(1)
   C(x=1, y=2)

Please note that you can't directly set attributes on frozen classes:

.. doctest::

   >>> @frozen
   ... class FrozenBroken:
   ...     x: int
   ...     y: int = field(init=False)
   ...     def __attrs_post_init__(self):
   ...         self.y = self.x + 1
   >>> FrozenBroken(1)
   Traceback (most recent call last):
      ...
   attrs.exceptions.FrozenInstanceError: can't set attribute

If you need to set attributes on a frozen class, you'll have to resort to the `same trick <how-frozen>` as ``attrs`` and use :meth:`object.__setattr__`:

.. doctest::

   >>> @define
   ... class Frozen:
   ...     x: int
   ...     y: int = field(init=False)
   ...     def __attrs_post_init__(self):
   ...         object.__setattr__(self, "y", self.x + 1)
   >>> Frozen(1)
   Frozen(x=1, y=2)

Note that you *must not* access the hash code of the object in ``__attrs_post_init__`` if ``cache_hash=True``.


Order of Execution
------------------

If present, the hooks are executed in the following order:

1. ``__attrs_pre_init__`` (if present on *current* class)
2. For each attribute, in the order it was declared:

   a. default factory
   b. converter

3. *all* validators
4. ``__attrs_post_init__`` (if present on *current* class)

Notably this means, that you can access all attributes from within your validators, but your converters have to deal with invalid values and have to return a valid value.


Derived Attributes
------------------

One of the most common ``attrs`` questions on *Stack Overflow* is how to have attributes that depend on other attributes.
For example if you have an API token and want to instantiate a web client that uses it for authentication.
Based on the previous sections, there's two approaches.

The simpler one is using ``__attrs_post_init__``::

   @define
   class APIClient:
       token: str
       client: WebClient = field(init=False)

       def __attrs_post_init__(self):
           self.client = WebClient(self.token)

The second one is using a decorator-based default::

   @define
   class APIClient:
       token: str
       client: WebClient = field()  # needed! attr.ib works too

        @client.default
        def _client_factory(self):
            return WebClient(self.token)

That said, and as pointed out in the beginning of the chapter, a better approach would be to have a factory class method::

   @define
   class APIClient:
       client: WebClient

       @classmethod
       def from_token(cls, token: str) -> SomeClass:
           return cls(client=WebClient(token))

This makes the class more testable.


.. _`Wiki page`: https://github.com/python-attrs/attrs/wiki/Extensions-to-attrs
.. _`get confused`: https://github.com/python-attrs/attrs/issues/289
.. _`there is no such thing as a private argument`: https://github.com/hynek/characteristic/issues/6
