# Comparison

By default, two instances of *attrs* classes are equal if they have the same type and all their fields are equal.
For that, *attrs* writes `__eq__` and `__ne__` methods for you.

Additionally, if you pass `order=True`, *attrs* will also create a complete set of ordering methods: `__le__`, `__lt__`, `__ge__`, and `__gt__`.

Both for equality and order, *attrs* will:

- Check if the types of the instances you're comparing are equal,
- if so, create a tuple of all field values for each instance,
- and finally perform the desired comparison operation on those tuples.

[^default]: That's the default if you use the {func}`attr.s` decorator, but not with {func}`~attrs.define`.

(custom-comparison)=

## Customization

As with other features, you can exclude fields from being involved in comparison operations:

```{doctest}
>>> from attrs import define, field
>>> @define
... class C:
...     x: int
...     y: int = field(eq=False)

>>> C(1, 2) == C(1, 3)
True
```

Additionally you can also pass a *callable* instead of a bool to both *eq* and *order*.
It is then used as a key function like you may know from {func}`sorted`:

```{doctest}
>>> @define
... class S:
...     x: str = field(eq=str.lower)

>>> S("foo") == S("FOO")
True

>>> @define(order=True)
... class C:
...     x: str = field(order=int)

>>> C("10") > C("2")
True
```

This is especially useful when you have fields with objects that have atypical comparison properties.
Common examples of such objects are [NumPy arrays](https://github.com/python-attrs/attrs/issues/435).

To save you unnecessary boilerplate, *attrs* comes with the {func}`attrs.cmp_using` helper to create such functions.
For NumPy arrays it would look like this:

```python
import numpy

@define
class C:
   an_array = field(eq=attrs.cmp_using(eq=numpy.array_equal))
```

:::{warning}
Please note that *eq* and *order* are set *independently*, because *order* is `False` by default in {func}`~attrs.define` (but not in {func}`attr.s`).
You can set both at once by using the *cmp* argument that we've undeprecated just for this use-case.
:::
