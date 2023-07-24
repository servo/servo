Hashing
=======

Hash Method Generation
----------------------

.. warning::

   The overarching theme is to never set the ``@attr.s(hash=X)`` parameter yourself.
   Leave it at ``None`` which means that ``attrs`` will do the right thing for you, depending on the other parameters:

   - If you want to make objects hashable by value: use ``@attr.s(frozen=True)``.
   - If you want hashing and equality by object identity: use ``@attr.s(eq=False)``

   Setting ``hash`` yourself can have unexpected consequences so we recommend to tinker with it only if you know exactly what you're doing.

Under certain circumstances, it's necessary for objects to be *hashable*.
For example if you want to put them into a `set` or if you want to use them as keys in a `dict`.

The *hash* of an object is an integer that represents the contents of an object.
It can be obtained by calling `hash` on an object and is implemented by writing a ``__hash__`` method for your class.

``attrs`` will happily write a ``__hash__`` method for you [#fn1]_, however it will *not* do so by default.
Because according to the definition_ from the official Python docs, the returned hash has to fulfill certain constraints:

#. Two objects that are equal, **must** have the same hash.
   This means that if ``x == y``, it *must* follow that ``hash(x) == hash(y)``.

   By default, Python classes are compared *and* hashed by their `id`.
   That means that every instance of a class has a different hash, no matter what attributes it carries.

   It follows that the moment you (or ``attrs``) change the way equality is handled by implementing ``__eq__`` which is based on attribute values, this constraint is broken.
   For that reason Python 3 will make a class that has customized equality unhashable.
   Python 2 on the other hand will happily let you shoot your foot off.
   Unfortunately ``attrs`` currently mimics Python 2's behavior for backward compatibility reasons if you set ``hash=False``.

   The *correct way* to achieve hashing by id is to set ``@attr.s(eq=False)``.
   Setting ``@attr.s(hash=False)`` (which implies ``eq=True``) is almost certainly a *bug*.

   .. warning::

      Be careful when subclassing!
      Setting ``eq=False`` on a class whose base class has a non-default ``__hash__`` method will *not* make ``attrs`` remove that ``__hash__`` for you.

      It is part of ``attrs``'s philosophy to only *add* to classes so you have the freedom to customize your classes as you wish.
      So if you want to *get rid* of methods, you'll have to do it by hand.

      The easiest way to reset ``__hash__`` on a class is adding ``__hash__ = object.__hash__`` in the class body.

#. If two object are not equal, their hash **should** be different.

   While this isn't a requirement from a standpoint of correctness, sets and dicts become less effective if there are a lot of identical hashes.
   The worst case is when all objects have the same hash which turns a set into a list.

#. The hash of an object **must not** change.

   If you create a class with ``@attr.s(frozen=True)`` this is fullfilled by definition, therefore ``attrs`` will write a ``__hash__`` function for you automatically.
   You can also force it to write one with ``hash=True`` but then it's *your* responsibility to make sure that the object is not mutated.

   This point is the reason why mutable structures like lists, dictionaries, or sets aren't hashable while immutable ones like tuples or frozensets are:
   point 1 and 2 require that the hash changes with the contents but point 3 forbids it.

For a more thorough explanation of this topic, please refer to this blog post: `Python Hashes and Equality`_.


Hashing and Mutability
----------------------

Changing any field involved in hash code computation after the first call to ``__hash__`` (typically this would be after its insertion into a hash-based collection) can result in silent bugs.
Therefore, it is strongly recommended that hashable classes be ``frozen``.
Beware, however, that this is not a complete guarantee of safety:
if a field points to an object and that object is mutated, the hash code may change, but ``frozen`` will not protect you.


Hash Code Caching
-----------------

Some objects have hash codes which are expensive to compute.
If such objects are to be stored in hash-based collections, it can be useful to compute the hash codes only once and then store the result on the object to make future hash code requests fast.
To enable caching of hash codes, pass ``cache_hash=True`` to ``@attrs``.
This may only be done if ``attrs`` is already generating a hash function for the object.

.. [#fn1] The hash is computed by hashing a tuple that consists of an unique id for the class plus all attribute values.

.. _definition: https://docs.python.org/3/glossary.html#term-hashable
.. _`Python Hashes and Equality`: https://hynek.me/articles/hashes-and-equality/
