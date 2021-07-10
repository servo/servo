Changelog
=========

Versions follow `CalVer <http://calver.org>`_ with a strict backwards compatibility policy.
The third digit is only for regressions.

.. towncrier release notes start

18.1.0 (2018-05-03)
-------------------

Changes
^^^^^^^

- ``x=X(); x.cycle = x; repr(x)`` will no longer raise a ``RecursionError``, and will instead show as ``X(x=...)``.

  `#95 <https://github.com/python-attrs/attrs/issues/95>`_
- ``attr.ib(factory=f)`` is now syntactic sugar for the common case of ``attr.ib(default=attr.Factory(f))``.

  `#178 <https://github.com/python-attrs/attrs/issues/178>`_,
  `#356 <https://github.com/python-attrs/attrs/issues/356>`_
- Added ``attr.field_dict()`` to return an ordered dictionary of ``attrs`` attributes for a class, whose keys are the attribute names.

  `#290 <https://github.com/python-attrs/attrs/issues/290>`_,
  `#349 <https://github.com/python-attrs/attrs/issues/349>`_
- The order of attributes that are passed into ``attr.make_class()`` or the ``these`` argument of ``@attr.s()`` is now retained if the dictionary is ordered (i.e. ``dict`` on Python 3.6 and later, ``collections.OrderedDict`` otherwise).

  Before, the order was always determined by the order in which the attributes have been defined which may not be desirable when creating classes programatically.

  `#300 <https://github.com/python-attrs/attrs/issues/300>`_,
  `#339 <https://github.com/python-attrs/attrs/issues/339>`_,
  `#343 <https://github.com/python-attrs/attrs/issues/343>`_
- In slotted classes, ``__getstate__`` and ``__setstate__`` now ignore the ``__weakref__`` attribute.

  `#311 <https://github.com/python-attrs/attrs/issues/311>`_,
  `#326 <https://github.com/python-attrs/attrs/issues/326>`_
- Setting the cell type is now completely best effort.
  This fixes ``attrs`` on Jython.

  We cannot make any guarantees regarding Jython though, because our test suite cannot run due to dependency incompatabilities.

  `#321 <https://github.com/python-attrs/attrs/issues/321>`_,
  `#334 <https://github.com/python-attrs/attrs/issues/334>`_
- If ``attr.s`` is passed a *these* argument, it will not attempt to remove attributes with the same name from the class body anymore.

  `#322 <https://github.com/python-attrs/attrs/issues/322>`_,
  `#323 <https://github.com/python-attrs/attrs/issues/323>`_
- The hash of ``attr.NOTHING`` is now vegan and faster on 32bit Python builds.

  `#331 <https://github.com/python-attrs/attrs/issues/331>`_,
  `#332 <https://github.com/python-attrs/attrs/issues/332>`_
- The overhead of instantiating frozen dict classes is virtually eliminated.
  `#336 <https://github.com/python-attrs/attrs/issues/336>`_
- Generated ``__init__`` methods now have an ``__annotations__`` attribute derived from the types of the fields.

  `#363 <https://github.com/python-attrs/attrs/issues/363>`_
- We have restructured the documentation a bit to account for ``attrs``' growth in scope.
  Instead of putting everything into the `examples <http://www.attrs.org/en/stable/examples.html>`_ page, we have started to extract narrative chapters.

  So far, we've added chapters on `initialization <http://www.attrs.org/en/stable/init.html>`_ and `hashing <http://www.attrs.org/en/stable/hashing.html>`_.

  Expect more to come!

  `#369 <https://github.com/python-attrs/attrs/issues/369>`_,
  `#370 <https://github.com/python-attrs/attrs/issues/370>`_


----


17.4.0 (2017-12-30)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- The traversal of MROs when using multiple inheritance was backward:
  If you defined a class ``C`` that subclasses ``A`` and ``B`` like ``C(A, B)``, ``attrs`` would have collected the attributes from ``B`` *before* those of ``A``.

  This is now fixed and means that in classes that employ multiple inheritance, the output of ``__repr__`` and the order of positional arguments in ``__init__`` changes.
  Due to the nature of this bug, a proper deprecation cycle was unfortunately impossible.

  Generally speaking, it's advisable to prefer ``kwargs``-based initialization anyways – *especially* if you employ multiple inheritance and diamond-shaped hierarchies.

  `#298 <https://github.com/python-attrs/attrs/issues/298>`_,
  `#299 <https://github.com/python-attrs/attrs/issues/299>`_,
  `#304 <https://github.com/python-attrs/attrs/issues/304>`_
- The ``__repr__`` set by ``attrs``
  no longer produces an ``AttributeError``
  when the instance is missing some of the specified attributes
  (either through deleting
  or after using ``init=False`` on some attributes).

  This can break code
  that relied on ``repr(attr_cls_instance)`` raising ``AttributeError``
  to check if any attr-specified members were unset.

  If you were using this,
  you can implement a custom method for checking this::

      def has_unset_members(self):
          for field in attr.fields(type(self)):
              try:
                  getattr(self, field.name)
              except AttributeError:
                  return True
          return False

  `#308 <https://github.com/python-attrs/attrs/issues/308>`_


Deprecations
^^^^^^^^^^^^

- The ``attr.ib(convert=callable)`` option is now deprecated in favor of ``attr.ib(converter=callable)``.

  This is done to achieve consistency with other noun-based arguments like *validator*.

  *convert* will keep working until at least January 2019 while raising a ``DeprecationWarning``.

  `#307 <https://github.com/python-attrs/attrs/issues/307>`_


Changes
^^^^^^^

- Generated ``__hash__`` methods now hash the class type along with the attribute values.
  Until now the hashes of two classes with the same values were identical which was a bug.

  The generated method is also *much* faster now.

  `#261 <https://github.com/python-attrs/attrs/issues/261>`_,
  `#295 <https://github.com/python-attrs/attrs/issues/295>`_,
  `#296 <https://github.com/python-attrs/attrs/issues/296>`_
- ``attr.ib``\ ’s ``metadata`` argument now defaults to a unique empty ``dict`` instance instead of sharing a common empty ``dict`` for all.
  The singleton empty ``dict`` is still enforced.

  `#280 <https://github.com/python-attrs/attrs/issues/280>`_
- ``ctypes`` is optional now however if it's missing, a bare ``super()`` will not work in slotted classes.
  This should only happen in special environments like Google App Engine.

  `#284 <https://github.com/python-attrs/attrs/issues/284>`_,
  `#286 <https://github.com/python-attrs/attrs/issues/286>`_
- The attribute redefinition feature introduced in 17.3.0 now takes into account if an attribute is redefined via multiple inheritance.
  In that case, the definition that is closer to the base of the class hierarchy wins.

  `#285 <https://github.com/python-attrs/attrs/issues/285>`_,
  `#287 <https://github.com/python-attrs/attrs/issues/287>`_
- Subclasses of ``auto_attribs=True`` can be empty now.

  `#291 <https://github.com/python-attrs/attrs/issues/291>`_,
  `#292 <https://github.com/python-attrs/attrs/issues/292>`_
- Equality tests are *much* faster now.

  `#306 <https://github.com/python-attrs/attrs/issues/306>`_
- All generated methods now have correct ``__module__``, ``__name__``, and (on Python 3) ``__qualname__`` attributes.

  `#309 <https://github.com/python-attrs/attrs/issues/309>`_


----


17.3.0 (2017-11-08)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- Attributes are not defined on the class body anymore.

  This means that if you define a class ``C`` with an attribute ``x``, the class will *not* have an attribute ``x`` for introspection anymore.
  Instead of ``C.x``, use ``attr.fields(C).x`` or look at ``C.__attrs_attrs__``.
  The old behavior has been deprecated since version 16.1.
  (`#253 <https://github.com/python-attrs/attrs/issues/253>`_)


Changes
^^^^^^^

- ``super()`` and ``__class__`` now work with slotted classes on Python 3.
  (`#102 <https://github.com/python-attrs/attrs/issues/102>`_, `#226 <https://github.com/python-attrs/attrs/issues/226>`_, `#269 <https://github.com/python-attrs/attrs/issues/269>`_, `#270 <https://github.com/python-attrs/attrs/issues/270>`_, `#272 <https://github.com/python-attrs/attrs/issues/272>`_)
- Added ``type`` argument to ``attr.ib()`` and corresponding ``type`` attribute to ``attr.Attribute``.

  This change paves the way for automatic type checking and serialization (though as of this release ``attrs`` does not make use of it).
  In Python 3.6 or higher, the value of ``attr.Attribute.type`` can alternately be set using variable type annotations
  (see `PEP 526 <https://www.python.org/dev/peps/pep-0526/>`_). (`#151 <https://github.com/python-attrs/attrs/issues/151>`_, `#214 <https://github.com/python-attrs/attrs/issues/214>`_, `#215 <https://github.com/python-attrs/attrs/issues/215>`_, `#239 <https://github.com/python-attrs/attrs/issues/239>`_)
- The combination of ``str=True`` and ``slots=True`` now works on Python 2.
  (`#198 <https://github.com/python-attrs/attrs/issues/198>`_)
- ``attr.Factory`` is hashable again. (`#204
  <https://github.com/python-attrs/attrs/issues/204>`_)
- Subclasses now can overwrite attribute definitions of their superclass.

  That means that you can -- for example -- change the default value for an attribute by redefining it.
  (`#221 <https://github.com/python-attrs/attrs/issues/221>`_, `#229 <https://github.com/python-attrs/attrs/issues/229>`_)
- Added new option ``auto_attribs`` to ``@attr.s`` that allows to collect annotated fields without setting them to ``attr.ib()``.

  Setting a field to an ``attr.ib()`` is still possible to supply options like validators.
  Setting it to any other value is treated like it was passed as ``attr.ib(default=value)`` -- passing an instance of ``attr.Factory`` also works as expected.
  (`#262 <https://github.com/python-attrs/attrs/issues/262>`_, `#277 <https://github.com/python-attrs/attrs/issues/277>`_)
- Instances of classes created using ``attr.make_class()`` can now be pickled.
  (`#282 <https://github.com/python-attrs/attrs/issues/282>`_)


----


17.2.0 (2017-05-24)
-------------------


Changes:
^^^^^^^^

- Validators are hashable again.
  Note that validators may become frozen in the future, pending availability of no-overhead frozen classes.
  `#192 <https://github.com/python-attrs/attrs/issues/192>`_


----


17.1.0 (2017-05-16)
-------------------

To encourage more participation, the project has also been moved into a `dedicated GitHub organization <https://github.com/python-attrs/>`_ and everyone is most welcome to join!

``attrs`` also has a logo now!

.. image:: http://www.attrs.org/en/latest/_static/attrs_logo.png
   :alt: attrs logo


Backward-incompatible Changes:
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- ``attrs`` will set the ``__hash__()`` method to ``None`` by default now.
  The way hashes were handled before was in conflict with `Python's specification <https://docs.python.org/3/reference/datamodel.html#object.__hash__>`_.
  This *may* break some software although this breakage is most likely just surfacing of latent bugs.
  You can always make ``attrs`` create the ``__hash__()`` method using ``@attr.s(hash=True)``.
  See `#136`_ for the rationale of this change.

  .. warning::

    Please *do not* upgrade blindly and *do* test your software!
    *Especially* if you use instances as dict keys or put them into sets!

- Correspondingly, ``attr.ib``'s ``hash`` argument is ``None`` by default too and mirrors the ``cmp`` argument as it should.


Deprecations:
^^^^^^^^^^^^^

- ``attr.assoc()`` is now deprecated in favor of ``attr.evolve()`` and will stop working in 2018.


Changes:
^^^^^^^^

- Fix default hashing behavior.
  Now *hash* mirrors the value of *cmp* and classes are unhashable by default.
  `#136`_
  `#142 <https://github.com/python-attrs/attrs/issues/142>`_
- Added ``attr.evolve()`` that, given an instance of an ``attrs`` class and field changes as keyword arguments, will instantiate a copy of the given instance with the changes applied.
  ``evolve()`` replaces ``assoc()``, which is now deprecated.
  ``evolve()`` is significantly faster than ``assoc()``, and requires the class have an initializer that can take the field values as keyword arguments (like ``attrs`` itself can generate).
  `#116 <https://github.com/python-attrs/attrs/issues/116>`_
  `#124 <https://github.com/python-attrs/attrs/pull/124>`_
  `#135 <https://github.com/python-attrs/attrs/pull/135>`_
- ``FrozenInstanceError`` is now raised when trying to delete an attribute from a frozen class.
  `#118 <https://github.com/python-attrs/attrs/pull/118>`_
- Frozen-ness of classes is now inherited.
  `#128 <https://github.com/python-attrs/attrs/pull/128>`_
- ``__attrs_post_init__()`` is now run if validation is disabled.
  `#130 <https://github.com/python-attrs/attrs/pull/130>`_
- Added ``attr.validators.in_(options)`` that, given the allowed `options`, checks whether the attribute value is in it.
  This can be used to check constants, enums, mappings, etc.
  `#181 <https://github.com/python-attrs/attrs/pull/181>`_
- Added ``attr.validators.and_()`` that composes multiple validators into one.
  `#161 <https://github.com/python-attrs/attrs/issues/161>`_
- For convenience, the ``validator`` argument of ``@attr.s`` now can take a ``list`` of validators that are wrapped using ``and_()``.
  `#138 <https://github.com/python-attrs/attrs/issues/138>`_
- Accordingly, ``attr.validators.optional()`` now can take a ``list`` of validators too.
  `#161 <https://github.com/python-attrs/attrs/issues/161>`_
- Validators can now be defined conveniently inline by using the attribute as a decorator.
  Check out the `examples <http://www.attrs.org/en/stable/examples.html#validators>`_ to see it in action!
  `#143 <https://github.com/python-attrs/attrs/issues/143>`_
- ``attr.Factory()`` now has a ``takes_self`` argument that makes the initializer to pass the partially initialized instance into the factory.
  In other words you can define attribute defaults based on other attributes.
  `#165`_
  `#189 <https://github.com/python-attrs/attrs/issues/189>`_
- Default factories can now also be defined inline using decorators.
  They are *always* passed the partially initialized instance.
  `#165`_
- Conversion can now be made optional using ``attr.converters.optional()``.
  `#105 <https://github.com/python-attrs/attrs/issues/105>`_
  `#173 <https://github.com/python-attrs/attrs/pull/173>`_
- ``attr.make_class()`` now accepts the keyword argument ``bases`` which allows for subclassing.
  `#152 <https://github.com/python-attrs/attrs/pull/152>`_
- Metaclasses are now preserved with ``slots=True``.
  `#155 <https://github.com/python-attrs/attrs/pull/155>`_

.. _`#136`: https://github.com/python-attrs/attrs/issues/136
.. _`#165`: https://github.com/python-attrs/attrs/issues/165


----


16.3.0 (2016-11-24)
-------------------

Changes:
^^^^^^^^

- Attributes now can have user-defined metadata which greatly improves ``attrs``'s extensibility.
  `#96 <https://github.com/python-attrs/attrs/pull/96>`_
- Allow for a ``__attrs_post_init__()`` method that -- if defined -- will get called at the end of the ``attrs``-generated ``__init__()`` method.
  `#111 <https://github.com/python-attrs/attrs/pull/111>`_
- Added ``@attr.s(str=True)`` that will optionally create a ``__str__()`` method that is identical to ``__repr__()``.
  This is mainly useful with ``Exception``\ s and other classes that rely on a useful ``__str__()`` implementation but overwrite the default one through a poor own one.
  Default Python class behavior is to use ``__repr__()`` as ``__str__()`` anyways.

  If you tried using ``attrs`` with ``Exception``\ s and were puzzled by the tracebacks: this option is for you.
- ``__name__`` is not overwritten with ``__qualname__`` for ``attr.s(slots=True)`` classes anymore.
  `#99 <https://github.com/python-attrs/attrs/issues/99>`_


----


16.2.0 (2016-09-17)
-------------------

Changes:
^^^^^^^^

- Added ``attr.astuple()`` that -- similarly to ``attr.asdict()`` -- returns the instance as a tuple.
  `#77 <https://github.com/python-attrs/attrs/issues/77>`_
- Converts now work with frozen classes.
  `#76 <https://github.com/python-attrs/attrs/issues/76>`_
- Instantiation of ``attrs`` classes with converters is now significantly faster.
  `#80 <https://github.com/python-attrs/attrs/pull/80>`_
- Pickling now works with slotted classes.
  `#81 <https://github.com/python-attrs/attrs/issues/81>`_
- ``attr.assoc()`` now works with slotted classes.
  `#84 <https://github.com/python-attrs/attrs/issues/84>`_
- The tuple returned by ``attr.fields()`` now also allows to access the ``Attribute`` instances by name.
  Yes, we've subclassed ``tuple`` so you don't have to!
  Therefore ``attr.fields(C).x`` is equivalent to the deprecated ``C.x`` and works with slotted classes.
  `#88 <https://github.com/python-attrs/attrs/issues/88>`_


----


16.1.0 (2016-08-30)
-------------------

Backward-incompatible Changes:
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- All instances where function arguments were called ``cl`` have been changed to the more Pythonic ``cls``.
  Since it was always the first argument, it's doubtful anyone ever called those function with in the keyword form.
  If so, sorry for any breakage but there's no practical deprecation path to solve this ugly wart.


Deprecations:
^^^^^^^^^^^^^

- Accessing ``Attribute`` instances on class objects is now deprecated and will stop working in 2017.
  If you need introspection please use the ``__attrs_attrs__`` attribute or the ``attr.fields()`` function that carry them too.
  In the future, the attributes that are defined on the class body and are usually overwritten in your ``__init__`` method are simply removed after ``@attr.s`` has been applied.

  This will remove the confusing error message if you write your own ``__init__`` and forget to initialize some attribute.
  Instead you will get a straightforward ``AttributeError``.
  In other words: decorated classes will work more like plain Python classes which was always ``attrs``'s goal.
- The serious business aliases ``attr.attributes`` and ``attr.attr`` have been deprecated in favor of ``attr.attrs`` and ``attr.attrib`` which are much more consistent and frankly obvious in hindsight.
  They will be purged from documentation immediately but there are no plans to actually remove them.


Changes:
^^^^^^^^

- ``attr.asdict()``\ 's ``dict_factory`` arguments is now propagated on recursion.
  `#45 <https://github.com/python-attrs/attrs/issues/45>`_
- ``attr.asdict()``, ``attr.has()`` and ``attr.fields()`` are significantly faster.
  `#48 <https://github.com/python-attrs/attrs/issues/48>`_
  `#51 <https://github.com/python-attrs/attrs/issues/51>`_
- Add ``attr.attrs`` and ``attr.attrib`` as a more consistent aliases for ``attr.s`` and ``attr.ib``.
- Add ``frozen`` option to ``attr.s`` that will make instances best-effort immutable.
  `#60 <https://github.com/python-attrs/attrs/issues/60>`_
- ``attr.asdict()`` now takes ``retain_collection_types`` as an argument.
  If ``True``, it does not convert attributes of type ``tuple`` or ``set`` to ``list``.
  `#69 <https://github.com/python-attrs/attrs/issues/69>`_


----


16.0.0 (2016-05-23)
-------------------

Backward-incompatible Changes:
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- Python 3.3 and 2.6 aren't supported anymore.
  They may work by chance but any effort to keep them working has ceased.

  The last Python 2.6 release was on October 29, 2013 and isn't supported by the CPython core team anymore.
  Major Python packages like Django and Twisted dropped Python 2.6 a while ago already.

  Python 3.3 never had a significant user base and wasn't part of any distribution's LTS release.

Changes:
^^^^^^^^

- ``__slots__`` have arrived!
  Classes now can automatically be `slotted <https://docs.python.org/3/reference/datamodel.html#slots>`_-style (and save your precious memory) just by passing ``slots=True``.
  `#35 <https://github.com/python-attrs/attrs/issues/35>`_
- Allow the case of initializing attributes that are set to ``init=False``.
  This allows for clean initializer parameter lists while being able to initialize attributes to default values.
  `#32 <https://github.com/python-attrs/attrs/issues/32>`_
- ``attr.asdict()`` can now produce arbitrary mappings instead of Python ``dict``\ s when provided with a ``dict_factory`` argument.
  `#40 <https://github.com/python-attrs/attrs/issues/40>`_
- Multiple performance improvements.


----


15.2.0 (2015-12-08)
-------------------

Changes:
^^^^^^^^

- Added a ``convert`` argument to ``attr.ib``, which allows specifying a function to run on arguments.
  This allows for simple type conversions, e.g. with ``attr.ib(convert=int)``.
  `#26 <https://github.com/python-attrs/attrs/issues/26>`_
- Speed up object creation when attribute validators are used.
  `#28 <https://github.com/python-attrs/attrs/issues/28>`_


----


15.1.0 (2015-08-20)
-------------------

Changes:
^^^^^^^^

- Added ``attr.validators.optional()`` that wraps other validators allowing attributes to be ``None``.
  `#16 <https://github.com/python-attrs/attrs/issues/16>`_
- Multi-level inheritance now works.
  `#24 <https://github.com/python-attrs/attrs/issues/24>`_
- ``__repr__()`` now works with non-redecorated subclasses.
  `#20 <https://github.com/python-attrs/attrs/issues/20>`_


----


15.0.0 (2015-04-15)
-------------------

Changes:
^^^^^^^^

Initial release.
