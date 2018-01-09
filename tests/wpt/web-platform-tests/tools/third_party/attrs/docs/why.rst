.. _why:

Why not…
========


If you'd like third party's account why ``attrs`` is great, have a look at Glyph's `The One Python Library Everyone Needs <https://glyph.twistedmatrix.com/2016/08/attrs.html>`_!


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


…namedtuples?
-------------

:func:`collections.namedtuple`\ s are tuples with names, not classes. [#history]_
Since writing classes is tiresome in Python, every now and then someone discovers all the typing they could save and gets really excited.
However that convenience comes at a price.

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
  for example if you create documentation that includes class hierarchies like `Sphinx's autodoc <http://www.sphinx-doc.org/en/stable/ext/autodoc.html>`_ with ``show-inheritance``.
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
.. [#iter] :func:`attr.astuple` can be used to get that behavior in ``attrs`` on *explicit demand*.
.. [#immutable] ``attrs`` offers *optional* immutability through the ``frozen`` keyword.
.. [#perf] Although ``attrs`` would serve you just as well!
           Since both employ the same method of writing and compiling Python code for you, the performance penalty is negligible at worst and in some cases ``attrs`` is even faster if you use ``slots=True`` (which is generally a good idea anyway).

.. _behaving like a tuple: https://docs.python.org/3/tutorial/datastructures.html#tuples-and-sequences


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

While we're fans of all things artisanal, writing the same nine methods all over again doesn't qualify for me.
I usually manage to get some typos inside and there's simply more code that can break and thus has to be tested.

To bring it into perspective, the equivalent of

.. doctest::

   >>> @attr.s
   ... class SmartClass(object):
   ...    a = attr.ib()
   ...    b = attr.ib()
   >>> SmartClass(1, 2)
   SmartClass(a=1, b=2)

is

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
   ...         return hash((self.a, self.b))
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
