Initialization
==============

In Python, instance intialization happens in the ``__init__`` method.
Generally speaking, you should keep as little logic as possible in it, and you should think about what the class needs and not how it is going to be instantiated.

Passing complex objects into ``__init__`` and then using them to derive data for the class unnecessarily couples your new class with the old class which makes it harder to test and also will cause problems later.

So assuming you use an ORM and want to extract 2D points from a row object, do not write code like this::

    class Point(object):
        def __init__(self, database_row):
            self.x = database_row.x
            self.y = database_row.y

    pt = Point(row)

Instead, write a :func:`classmethod` that will extract it for you::

   @attr.s
   class Point(object):
       x = attr.ib()
       y = attr.ib()

       @classmethod
       def from_row(cls, row):
           return cls(row.x, row.y)

   pt = Point.from_row(row)

Now you can instantiate ``Point``\ s without creating fake row objects in your tests and you can have as many smart creation helpers as you want, in case more data sources appear.

For similar reasons, we strongly discourage from patterns like::

   pt = Point(**row.attributes)

which couples your classes to the data model.
Try to design your classes in a way that is clean and convenient to use -- not based on your database format.
The database format can change anytime and you're stuck with a bad class design that is hard to change.
Embrace classmethods as a filter between reality and what's best for you to work with.

If you look for object serialization, there's a bunch of projects listed on our ``attrs`` extensions `Wiki page`_.
Some of them even support nested schemas.


Private Attributes
------------------

One thing people tend to find confusing is the treatment of private attributes that start with an underscore.
``attrs`` follows the doctrine that `there is no such thing as a private argument`_ and strips the underscores from the name when writing the ``__init__`` method signature:

.. doctest::

   >>> import inspect, attr
   >>> @attr.s
   ... class C(object):
   ...    _x = attr.ib()
   >>> inspect.signature(C.__init__)
   <Signature (self, x) -> None>

There really isn't a right or wrong, it's a matter of taste.
But it's important to be aware of it because it can lead to surprising syntax errors:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...    _1 = attr.ib()
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

   >>> import attr
   >>> @attr.s
   ... class C(object):
   ...     a = attr.ib(default=42)
   ...     b = attr.ib(default=attr.Factory(list))
   ...     c = attr.ib(factory=list)  # syntactic sugar for above
   ...     d = attr.ib()
   ...     @d.default
   ...     def _any_name_except_a_name_of_an_attribute(self):
   ...        return {}
   >>> C()
   C(a=42, b=[], c=[], d={})

It's important that the decorated method -- or any other method or property! -- doesn't have the same name as the attribute, otherwise it would overwrite the attribute definition.


Please note that as with function and method signatures, ``default=[]`` will *not* do what you may think it might do:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib(default=[])
   >>> i = C()
   >>> j = C()
   >>> i.x.append(42)
   >>> j.x
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

Another thing that definitely *does* belong into ``__init__`` is checking the resulting instance for invariants.
This is why ``attrs`` has the concept of validators.


Decorator
~~~~~~~~~

The most straightforward way is using the attribute's ``validator`` method as a decorator.

The method has to accept three arguments:

#. the *instance* that's being validated (aka ``self``),
#. the *attribute* that it's validating, and finally
#. the *value* that is passed for it.

If the value does not pass the validator's standards, it just raises an appropriate exception.

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib()
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

Again, it's important that the decorated method doesn't have the same name as the attribute.


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

Of course you can mix and match the two approaches at your convenience.
If you define validators both ways for an attribute, they are both ran:

.. doctest::

   >>> @attr.s
   ... class C(object):
   ...     x = attr.ib(validator=attr.validators.instance_of(int))
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

   >>> attr.set_run_validators(False)
   >>> C("128")
   C(x='128')
   >>> attr.set_run_validators(True)
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

    >>> @attr.s
    ... class C(object):
    ...     x = attr.ib(converter=int)
    >>> o = C("1")
    >>> o.x
    1

Converters are run *before* validators, so you can use validators to check the final form of the value.

.. doctest::

    >>> def validate_x(instance, attribute, value):
    ...     if value < 0:
    ...         raise ValueError("x must be at least 0.")
    >>> @attr.s
    ... class C(object):
    ...     x = attr.ib(converter=int, validator=validate_x)
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


Post-Init Hook
--------------

Generally speaking, the moment you think that you need finer control over how your class is instantiated than what ``attrs`` offers, it's usually best to use a classmethod factory or to apply the `builder pattern <https://en.wikipedia.org/wiki/Builder_pattern>`_.

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


.. _`Wiki page`: https://github.com/python-attrs/attrs/wiki/Extensions-to-attrs
.. _`get confused`: https://github.com/python-attrs/attrs/issues/289
.. _`there is no such thing as a private argument`: https://github.com/hynek/characteristic/issues/6
