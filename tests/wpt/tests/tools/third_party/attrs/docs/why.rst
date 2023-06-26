Why not…
========


If you'd like third party's account why ``attrs`` is great, have a look at Glyph's `The One Python Library Everyone Needs <https://glyph.twistedmatrix.com/2016/08/attrs.html>`_!


…Data Classes?
--------------

:pep:`557` added Data Classes to `Python 3.7 <https://docs.python.org/3.7/whatsnew/3.7.html#dataclasses>`_ that resemble ``attrs`` in many ways.

They are the result of the Python community's `wish <https://mail.python.org/pipermail/python-ideas/2017-May/045618.html>`_ to have an easier way to write classes in the standard library that doesn't carry the problems of ``namedtuple``\ s.
To that end, ``attrs`` and its developers were involved in the PEP process and while we may disagree with some minor decisions that have been made, it's a fine library and if it stops you from abusing ``namedtuple``\ s, they are a huge win.

Nevertheless, there are still reasons to prefer ``attrs`` over Data Classes.
Whether they're relevant to *you* depends on your circumstances:

- Data Classes are *intentionally* less powerful than ``attrs``.
  There is a long list of features that were sacrificed for the sake of simplicity and while the most obvious ones are validators, converters, :ref:`equality customization <custom-comparison>`, or :doc:`extensibility <extending>` in general, it permeates throughout all APIs.

  On the other hand, Data Classes currently do not offer any significant feature that ``attrs`` doesn't already have.
- ``attrs`` supports all mainstream Python versions, including CPython 2.7 and PyPy.
- ``attrs`` doesn't force type annotations on you if you don't like them.
- But since it **also** supports typing, it's the best way to embrace type hints *gradually*, too.
- While Data Classes are implementing features from ``attrs`` every now and then, their presence is dependent on the Python version, not the package version.
  For example, support for ``__slots__`` has only been added in Python 3.10.
  That is especially painful for PyPI packages that support multiple Python versions.
  This includes possible implementation bugs.
- ``attrs`` can and will move faster.
  We are not bound to any release schedules and we have a clear deprecation policy.

  One of the `reasons <https://www.python.org/dev/peps/pep-0557/#why-not-just-use-attrs>`_ to not vendor ``attrs`` in the standard library was to not impede ``attrs``'s future development.

One way to think about ``attrs`` vs Data Classes is that ``attrs`` is a fully-fledged toolkit to write powerful classes while Data Classes are an easy way to get a class with some attributes.
Basically what ``attrs`` was in 2015.


…pydantic?
----------

*pydantic* is first an foremost a *data validation library*.
As such, it is a capable complement to class building libraries like ``attrs`` (or Data Classes!) for parsing and validating untrusted data.

However, as convenient as it might be, using it for your business or data layer `is problematic in several ways <https://threeofwands.com/why-i-use-attrs-instead-of-pydantic/>`_:
Is it really necessary to re-validate all your objects while reading them from a trusted database?
In the parlance of `Form, Command, and Model Validation <https://verraes.net/2015/02/form-command-model-validation/>`_, *pydantic* is the right tool for *Commands*.

`Separation of concerns <https://en.wikipedia.org/wiki/Separation_of_concerns>`_ feels tedious at times, but it's one of those things that you get to appreciate once you've shot your own foot often enough.


…namedtuples?
-------------

`collections.namedtuple`\ s are tuples with names, not classes. [#history]_
Since writing classes is tiresome in Python, every now and then someone discovers all the typing they could save and gets really excited.
However, that convenience comes at a price.

The most obvious difference between ``namedtuple``\ s and ``attrs``-based classes is that the latter are type-sensitive:

.. doctest::

   >>> import attr
   >>> C1 = attr.make_class("C1", ["a"])
   >>> C2 = attr.make_class("C2", ["a"])
   >>> i1 = C1(1)
   >>> i2 = C2(1)
   >>> i1.a == i2.a
   True
   >>> i1 == i2
   False

…while a ``namedtuple`` is *intentionally* `behaving like a tuple`_ which means the type of a tuple is *ignored*:

.. doctest::

   >>> from collections import namedtuple
   >>> NT1 = namedtuple("NT1", "a")
   >>> NT2 = namedtuple("NT2", "b")
   >>> t1 = NT1(1)
   >>> t2 = NT2(1)
   >>> t1 == t2 == (1,)
   True

Other often surprising behaviors include:

- Since they are a subclass of tuples, ``namedtuple``\ s have a length and are both iterable and indexable.
  That's not what you'd expect from a class and is likely to shadow subtle typo bugs.
- Iterability also implies that it's easy to accidentally unpack a ``namedtuple`` which leads to hard-to-find bugs. [#iter]_
- ``namedtuple``\ s have their methods *on your instances* whether you like it or not. [#pollution]_
- ``namedtuple``\ s are *always* immutable.
  Not only does that mean that you can't decide for yourself whether your instances should be immutable or not, it also means that if you want to influence your class' initialization (validation?  default values?), you have to implement :meth:`__new__() <object.__new__>` which is a particularly hacky and error-prone requirement for a very common problem. [#immutable]_
- To attach methods to a ``namedtuple`` you have to subclass it.
  And if you follow the standard library documentation's recommendation of::

    class Point(namedtuple('Point', ['x', 'y'])):
        # ...

  you end up with a class that has *two* ``Point``\ s in its :attr:`__mro__ <class.__mro__>`: ``[<class 'point.Point'>, <class 'point.Point'>, <type 'tuple'>, <type 'object'>]``.

  That's not only confusing, it also has very practical consequences:
  for example if you create documentation that includes class hierarchies like `Sphinx's autodoc <https://www.sphinx-doc.org/en/stable/usage/extensions/autodoc.html>`_ with ``show-inheritance``.
  Again: common problem, hacky solution with confusing fallout.

All these things make ``namedtuple``\ s a particularly poor choice for public APIs because all your objects are irrevocably tainted.
With ``attrs`` your users won't notice a difference because it creates regular, well-behaved classes.

.. admonition:: Summary

  If you want a *tuple with names*, by all means: go for a ``namedtuple``. [#perf]_
  But if you want a class with methods, you're doing yourself a disservice by relying on a pile of hacks that requires you to employ even more hacks as your requirements expand.

  Other than that, ``attrs`` also adds nifty features like validators, converters, and (mutable!) default values.


.. rubric:: Footnotes

.. [#history] The word is that ``namedtuple``\ s were added to the Python standard library as a way to make tuples in return values more readable.
              And indeed that is something you see throughout the standard library.

              Looking at what the makers of ``namedtuple``\ s use it for themselves is a good guideline for deciding on your own use cases.
.. [#pollution] ``attrs`` only adds a single attribute: ``__attrs_attrs__`` for introspection.
                All helpers are functions in the ``attr`` package.
                Since they take the instance as first argument, you can easily attach them to your classes under a name of your own choice.
.. [#iter] `attr.astuple` can be used to get that behavior in ``attrs`` on *explicit demand*.
.. [#immutable] ``attrs`` offers *optional* immutability through the ``frozen`` keyword.
.. [#perf] Although ``attrs`` would serve you just as well!
           Since both employ the same method of writing and compiling Python code for you, the performance penalty is negligible at worst and in some cases ``attrs`` is even faster if you use ``slots=True`` (which is generally a good idea anyway).

.. _behaving like a tuple: https://docs.python.org/3/tutorial/datastructures.html#tuples-and-sequences


…tuples?
--------

Readability
^^^^^^^^^^^

What makes more sense while debugging::

   Point(x=1, y=2)

or::

   (1, 2)

?

Let's add even more ambiguity::

   Customer(id=42, reseller=23, first_name="Jane", last_name="John")

or::

   (42, 23, "Jane", "John")

?

Why would you want to write ``customer[2]`` instead of ``customer.first_name``?

Don't get me started when you add nesting.
If you've never run into mysterious tuples you had no idea what the hell they meant while debugging, you're much smarter than yours truly.

Using proper classes with names and types makes program code much more readable and comprehensible_.
Especially when trying to grok a new piece of software or returning to old code after several months.

.. _comprehensible: https://arxiv.org/pdf/1304.5257.pdf


Extendability
^^^^^^^^^^^^^

Imagine you have a function that takes or returns a tuple.
Especially if you use tuple unpacking (eg. ``x, y = get_point()``), adding additional data means that you have to change the invocation of that function *everywhere*.

Adding an attribute to a class concerns only those who actually care about that attribute.


…dicts?
-------

Dictionaries are not for fixed fields.

If you have a dict, it maps something to something else.
You should be able to add and remove values.

``attrs`` lets you be specific about those expectations; a dictionary does not.
It gives you a named entity (the class) in your code, which lets you explain in other places whether you take a parameter of that class or return a value of that class.

In other words: if your dict has a fixed and known set of keys, it is an object, not a hash.
So if you never iterate over the keys of a dict, you should use a proper class.


…hand-written classes?
----------------------

While we're fans of all things artisanal, writing the same nine methods again and again doesn't qualify.
I usually manage to get some typos inside and there's simply more code that can break and thus has to be tested.

To bring it into perspective, the equivalent of

.. doctest::

   >>> @attr.s
   ... class SmartClass(object):
   ...    a = attr.ib()
   ...    b = attr.ib()
   >>> SmartClass(1, 2)
   SmartClass(a=1, b=2)

is roughly

.. doctest::

   >>> class ArtisanalClass(object):
   ...     def __init__(self, a, b):
   ...         self.a = a
   ...         self.b = b
   ...
   ...     def __repr__(self):
   ...         return "ArtisanalClass(a={}, b={})".format(self.a, self.b)
   ...
   ...     def __eq__(self, other):
   ...         if other.__class__ is self.__class__:
   ...             return (self.a, self.b) == (other.a, other.b)
   ...         else:
   ...             return NotImplemented
   ...
   ...     def __ne__(self, other):
   ...         result = self.__eq__(other)
   ...         if result is NotImplemented:
   ...             return NotImplemented
   ...         else:
   ...             return not result
   ...
   ...     def __lt__(self, other):
   ...         if other.__class__ is self.__class__:
   ...             return (self.a, self.b) < (other.a, other.b)
   ...         else:
   ...             return NotImplemented
   ...
   ...     def __le__(self, other):
   ...         if other.__class__ is self.__class__:
   ...             return (self.a, self.b) <= (other.a, other.b)
   ...         else:
   ...             return NotImplemented
   ...
   ...     def __gt__(self, other):
   ...         if other.__class__ is self.__class__:
   ...             return (self.a, self.b) > (other.a, other.b)
   ...         else:
   ...             return NotImplemented
   ...
   ...     def __ge__(self, other):
   ...         if other.__class__ is self.__class__:
   ...             return (self.a, self.b) >= (other.a, other.b)
   ...         else:
   ...             return NotImplemented
   ...
   ...     def __hash__(self):
   ...         return hash((self.__class__, self.a, self.b))
   >>> ArtisanalClass(a=1, b=2)
   ArtisanalClass(a=1, b=2)

which is quite a mouthful and it doesn't even use any of ``attrs``'s more advanced features like validators or defaults values.
Also: no tests whatsoever.
And who will guarantee you, that you don't accidentally flip the ``<`` in your tenth implementation of ``__gt__``?

It also should be noted that ``attrs`` is not an all-or-nothing solution.
You can freely choose which features you want and disable those that you want more control over:

.. doctest::

   >>> @attr.s(repr=False)
   ... class SmartClass(object):
   ...    a = attr.ib()
   ...    b = attr.ib()
   ...
   ...    def __repr__(self):
   ...        return "<SmartClass(a=%d)>" % (self.a,)
   >>> SmartClass(1, 2)
   <SmartClass(a=1)>

.. admonition:: Summary

   If you don't care and like typing, we're not gonna stop you.

   However it takes a lot of bias and determined rationalization to claim that ``attrs`` raises the mental burden on a project given how difficult it is to find the important bits in a hand-written class and how annoying it is to ensure you've copy-pasted your code correctly over all your classes.

   In any case, if you ever get sick of the repetitiveness and drowning important code in a sea of boilerplate, ``attrs`` will be waiting for you.
