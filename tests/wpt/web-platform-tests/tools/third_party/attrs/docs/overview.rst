========
Overview
========

In order to fulfill its ambitious goal of bringing back the joy to writing classes, it gives you a class decorator and a way to declaratively define the attributes on that class:

.. include:: ../README.rst
   :start-after: -code-begin-
   :end-before: -testimonials-


.. _philosophy:

Philosophy
==========

**It's about regular classes.**
   ``attrs`` is for creating well-behaved classes with a type, attributes, methods, and everything that comes with a class.
   It can be used for data-only containers like ``namedtuple``\ s or ``types.SimpleNamespace`` but they're just a sub-genre of what ``attrs`` is good for.

**The class belongs to the users.**
   You define a class and ``attrs`` adds static methods to that class based on the attributes you declare.
   The end.
   It doesn't add metaclasses.
   It doesn't add classes you've never heard of to your inheritance tree.
   An ``attrs`` class in runtime is indistiguishable from a regular class: because it *is* a regular class with a few boilerplate-y methods attached.

**Be light on API impact.**
   As convenient as it seems at first, ``attrs`` will *not* tack on any methods to your classes save the dunder ones.
   Hence all the useful :ref:`tools <helpers>` that come with ``attrs`` live in functions that operate on top of instances.
   Since they take an ``attrs`` instance as their first argument, you can attach them to your classes with one line of code.

**Performance matters.**
   ``attrs`` runtime impact is very close to zero because all the work is done when the class is defined.
   Once you're instantiating it, ``attrs`` is out of the picture completely.

**No surprises.**
   ``attrs`` creates classes that arguably work the way a Python beginner would reasonably expect them to work.
   It doesn't try to guess what you mean because explicit is better than implicit.
   It doesn't try to be clever because software shouldn't be clever.

Check out :doc:`how-does-it-work` if you'd like to know how it achieves all of the above.


What ``attrs`` Is Not
=====================

``attrs`` does *not* invent some kind of magic system that pulls classes out of its hat using meta classes, runtime introspection, and shaky interdependencies.

All ``attrs`` does is:

1. take your declaration,
2. write dunder methods based on that information,
3. and attach them to your class.

It does *nothing* dynamic at runtime, hence zero runtime overhead.
It's still *your* class.
Do with it as you please.


On the ``attr.s`` and ``attr.ib`` Names
=======================================

The ``attr.s`` decorator and the ``attr.ib`` function aren't any obscure abbreviations.
They are a *concise* and highly *readable* way to write ``attrs`` and ``attrib`` with an *explicit namespace*.

At first, some people have a negative gut reaction to that; resembling the reactions to Python's significant whitespace.
And as with that, once one gets used to it, the readability and explicitness of that API prevails and delights.

For those who can't swallow that API at all, ``attrs`` comes with serious business aliases: ``attr.attrs`` and ``attr.attrib``.

Therefore, the following class definition is identical to the previous one:

.. doctest::

   >>> from attr import attrs, attrib, Factory
   >>> @attrs
   ... class SomeClass(object):
   ...     a_number = attrib(default=42)
   ...     list_of_numbers = attrib(default=Factory(list))
   ...
   ...     def hard_math(self, another_number):
   ...         return self.a_number + sum(self.list_of_numbers) * another_number
   >>> SomeClass(1, [1, 2, 3])
   SomeClass(a_number=1, list_of_numbers=[1, 2, 3])

Use whichever variant fits your taste better.
