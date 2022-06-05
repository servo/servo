On The Core API Names
=====================

You may be surprised seeing ``attrs`` classes being created using `attrs.define` and with type annotated fields, instead of `attr.s` and `attr.ib()`.

Or, you wonder why the web and talks are full of this weird `attr.s` and `attr.ib` -- including people having strong opinions about it and using ``attr.attrs`` and ``attr.attrib`` instead.

And what even is ``attr.dataclass`` that's not documented but commonly used!?


TL;DR
-----

We recommend our modern APIs for new code:

- `attrs.define()` to define a new class,
- `attrs.mutable()` is an alias for `attrs.define()`,
- `attrs.frozen()` is an alias for ``define(frozen=True)``
- and `attrs.field()` to define an attribute.

They have been added in ``attrs`` 20.1.0, they are expressive, and they have modern defaults like slots and type annotation awareness switched on by default.
They are only available in Python 3.6 and later.
Sometimes they're referred to as *next-generation* or *NG* APIs.
As of ``attrs`` 21.3.0 you can also import them from the ``attrs`` package namespace.

The traditional APIs `attr.s` / `attr.ib`, their serious business aliases ``attr.attrs`` / ``attr.attrib``, and the never-documented, but popular ``attr.dataclass`` easter egg will stay **forever**.

``attrs`` will **never** force you to use type annotations.


A Short History Lesson
----------------------

At this point, ``attrs`` is an old project.
It had its first release in April 2015 -- back when most Python code was on Python 2.7 and Python 3.4 was the first Python 3 release that showed promise.
``attrs`` was always Python 3-first, but `type annotations <https://www.python.org/dev/peps/pep-0484/>`_ came only into Python 3.5 that was released in September 2015 and were largely ignored until years later.

At this time, if you didn't want to implement all the :term:`dunder methods`, the most common way to create a class with some attributes on it was to subclass a `collections.namedtuple`, or one of the many hacks that allowed you to access dictionary keys using attribute lookup.

But ``attrs`` history goes even a bit further back, to the now-forgotten `characteristic <https://github.com/hynek/characteristic>`_ that came out in May 2014 and already used a class decorator, but was overall too unergonomic.

In the wake of all of that, `glyph <https://twitter.com/glyph>`_ and `Hynek <https://twitter.com/hynek>`_ came together on IRC and brainstormed how to take the good ideas of ``characteristic``, but make them easier to use and read.
At this point the plan was not to make ``attrs`` what it is now -- a flexible class building kit.
All we wanted was an ergonomic little library to succinctly define classes with attributes.

Under the impression of of the unwieldy ``characteristic`` name, we went to the other side and decided to make the package name part of the API, and keep the API functions very short.
This led to the infamous `attr.s` and `attr.ib` which some found confusing and pronounced it as "attr dot s" or used a singular ``@s`` as the decorator.
But it was really just a way to say ``attrs`` and ``attrib``\ [#attr]_.

Some people hated this cutey API from day one, which is why we added aliases for them that we called *serious business*: ``@attr.attrs`` and ``attr.attrib()``.
Fans of them usually imported the names and didn't use the package name in the first place.
Unfortunately, the ``attr`` package name started creaking the moment we added ``attr.Factory``, since it couldn’t be morphed into something meaningful in any way.
A problem that grew worse over time, as more APIs and even modules were added.

But overall, ``attrs`` in this shape was a **huge** success -- especially after glyph's blog post `The One Python Library Everyone Needs <https://glyph.twistedmatrix.com/2016/08/attrs.html>`_ in August 2016 and `pytest <https://docs.pytest.org/>`_ adopting it.

Being able to just write::

   @attr.s
   class Point(object):
       x = attr.ib()
       y = attr.ib()

was a big step for those who wanted to write small, focused classes.


Dataclasses Enter The Arena
^^^^^^^^^^^^^^^^^^^^^^^^^^^

A big change happened in May 2017 when Hynek sat down with `Guido van Rossum <https://en.wikipedia.org/wiki/Guido_van_Rossum>`_ and `Eric V. Smith <https://github.com/ericvsmith>`_ at PyCon US 2017.

Type annotations for class attributes have `just landed <https://www.python.org/dev/peps/pep-0526/>`_ in Python 3.6 and Guido felt like it would be a good mechanic to introduce something similar to ``attrs`` to the Python standard library.
The result, of course, was `PEP 557 <https://www.python.org/dev/peps/pep-0557/>`_\ [#stdlib]_ which eventually became the `dataclasses` module in Python 3.7.

``attrs`` at this point was lucky to have several people on board who were also very excited about type annotations and helped implementing it; including a `Mypy plugin <https://medium.com/@Pilot-EPD-Blog/mypy-and-attrs-e1b0225e9ac6>`_.
And so it happened that ``attrs`` `shipped <https://www.attrs.org/en/17.3.0.post2/changelog.html>`_ the new method of defining classes more than half a year before Python 3.7 -- and thus `dataclasses` -- were released.

-----

Due to backward-compatibility concerns, this feature is off by default in the `attr.s` decorator and has to be activated using ``@attr.s(auto_attribs=True)``, though.
As a little easter egg and to save ourselves some typing, we've also `added <https://github.com/python-attrs/attrs/commit/88aa1c897dfe2ee4aa987e4a56f2ba1344a17238#diff-4fc63db1f2fcb7c6e464ee9a77c3c74e90dd191d1c9ffc3bdd1234d3a6663dc0R48>`_ an alias called ``attr.dataclasses`` that just set ``auto_attribs=True``.
It was never documented, but people found it and used it and loved it.

Over the next months and years it became clear that type annotations have become the popular way to define classes and their attributes.
However, it has also become clear that some people viscerally hate type annotations.
We're determined to serve both.


``attrs`` TNG
^^^^^^^^^^^^^

Over its existence, ``attrs`` never stood still.
But since we also greatly care about backward compatibility and not breaking our users's code, many features and niceties have to be manually activated.

That is not only annoying, it also leads to the problem that many of ``attrs``'s users don't even know what it can do for them.
We've spent years alone explaining that defining attributes using type annotations is in no way unique to `dataclasses`.

Finally we've decided to take the `Go route <https://go.dev/blog/module-compatibility>`_:
instead of fiddling with the old APIs -- whose names felt anachronistic anyway -- we'd define new ones, with better defaults.
So in July 2018, we `looked for better names <https://github.com/python-attrs/attrs/issues/408>`_ and came up with `attr.define`, `attr.field`, and friends.
Then in January 2019, we `started looking for inconvenient defaults <https://github.com/python-attrs/attrs/issues/487>`_ that we now could fix without any repercussions.

These APIs proved to be very popular, so we've finally changed the documentation to them in November of 2021.

All of this took way too long, of course.
One reason is the COVID-19 pandemic, but also our fear to fumble this historic chance to fix our APIs.

Finally, in December 2021, we've added the ``attrs`` package namespace.

We hope you like the result::

   from attrs import define

   @define
   class Point:
       x: int
       y: int


.. [#attr] We considered calling the PyPI package just ``attr`` too, but the name was already taken by an *ostensibly* inactive `package on PyPI <https://pypi.org/project/attr/#history>`_.
.. [#stdlib] The highly readable PEP also explains why ``attrs`` wasn't just added to the standard library.
   Don't believe the myths and rumors.
