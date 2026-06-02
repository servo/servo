(how)=

# How Does It Work?

## Boilerplate

*attrs* isn't the first library that aims to simplify class definition in Python.
But its **declarative** approach combined with **no runtime overhead** lets it stand out.

Once you apply the `@attrs.define` (or `@attr.s`) decorator to a class, *attrs* searches the class object for instances of `attr.ib`s.
Internally they're a representation of the data passed into `attr.ib` along with a counter to preserve the order of the attributes.
Alternatively, it's possible to define them using {doc}`types`.

In order to ensure that subclassing works as you'd expect it to work, *attrs* also walks the class hierarchy and collects the attributes of all base classes.
Please note that *attrs* does *not* call `super()` *ever*.
It will write {term}`dunder methods` to work on *all* of those attributes which also has performance benefits due to fewer function calls.

Once *attrs* knows what attributes it has to work on, it writes the requested {term}`dunder methods` and -- depending on whether you wish to have a {term}`dict <dict classes>` or {term}`slotted <slotted classes>` class -- creates a new class for you (`slots=True`) or attaches them to the original class (`slots=False`).
While creating new classes is more elegant, we've run into several edge cases surrounding metaclasses that make it impossible to go this route unconditionally.

To be very clear: if you define a class with a single attribute without a default value, the generated `__init__` will look *exactly* how you'd expect:

```{doctest}
>>> import inspect
>>> from attrs import define
>>> @define
... class C:
...     x: int
>>> print(inspect.getsource(C.__init__))
def __init__(self, x):
    self.x = x
<BLANKLINE>
```

No magic, no meta programming, no expensive introspection at runtime.

---

Everything until this point happens exactly *once* when the class is defined.
As soon as a class is done, it's done.
And it's just a regular Python class like any other, except for a single `__attrs_attrs__` attribute that *attrs* uses internally.
Much of the information is accessible via {func}`attrs.fields` and other functions which can be used for introspection or for writing your own tools and decorators on top of *attrs* (like {func}`attrs.asdict`).

And once you start instantiating your classes, *attrs* is out of your way completely.

This **static** approach was very much a design goal of *attrs* and what I strongly believe makes it distinct.

(how-frozen)=

## Immutability

In order to give you immutability, *attrs* will attach a `__setattr__` method to your class that raises an {class}`attrs.exceptions.FrozenInstanceError` whenever anyone tries to set an attribute.

The same is true if you choose to freeze individual attributes using the {obj}`attrs.setters.frozen` *on_setattr* hook -- except that the exception becomes {class}`attrs.exceptions.FrozenAttributeError`.

Both exceptions subclass {class}`attrs.exceptions.FrozenError`.

---

Depending on whether a class is a dict class or a slotted class, *attrs* uses a different technique to circumvent that limitation in the `__init__` method.

Once constructed, frozen instances don't differ in any way from regular ones except that you cannot change its attributes.

### Dict Classes

Dict classes -- i.e. regular classes -- simply assign the value directly into the class' eponymous `__dict__` (and there's nothing we can do to stop the user to do the same).

The performance impact is negligible.

### Slotted Classes

Slotted classes are more complicated.
Here it uses (an aggressively cached) {meth}`object.__setattr__` to set your attributes.
This is (still) slower than a plain assignment:

```none
$ pyperf timeit --rigorous \
      -s "import attr; C = attr.make_class('C', ['x', 'y', 'z'], slots=True)" \
      "C(1, 2, 3)"
.........................................
Mean +- std dev: 228 ns +- 18 ns

$ pyperf timeit --rigorous \
      -s "import attr; C = attr.make_class('C', ['x', 'y', 'z'], slots=True, frozen=True)" \
      "C(1, 2, 3)"
.........................................
Mean +- std dev: 425 ns +- 16 ns
```

So on a laptop computer the difference is about 200 nanoseconds (1 second is 1,000,000,000 nanoseconds).
It's certainly something you'll feel in a hot loop but shouldn't matter in normal code.
Pick what's more important to you.

### Summary

You should avoid instantiating lots of frozen slotted classes (i.e. `@frozen`) in performance-critical code.

Frozen dict classes have barely a performance impact, unfrozen slotted classes are even *faster* than unfrozen dict classes (i.e. regular classes).


(how-slotted-cached_property)=

## Cached Properties on Slotted Classes.

By default, the standard library `functools.cached_property` decorator does not work on slotted classes,
because it requires a `__dict__` to store the cached value.
This could be surprising when uses *attrs*, as makes using slotted classes so easy,
so attrs will convert `functools.cached_property` decorated methods, when constructing slotted classes.

Getting this working is achieved by:
* Adding names to `__slots__` for the wrapped methods.
* Adding a `__getattr__` method to set values on the wrapped methods.

For most users this should mean that it works transparently.

Note that the implementation does not guarantee that the wrapped method is called
only once in multi-threaded usage.  This matches the implementation of `cached_property`
in python v3.12.
