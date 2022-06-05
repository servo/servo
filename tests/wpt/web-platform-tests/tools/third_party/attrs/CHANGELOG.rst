Changelog
=========

Versions follow `CalVer <https://calver.org>`_ with a strict backwards-compatibility policy.

The **first number** of the version is the year.
The **second number** is incremented with each release, starting at 1 for each year.
The **third number** is when we need to start branches for older releases (only for emergencies).

Put simply, you shouldn't ever be afraid to upgrade ``attrs`` if you're only using its public APIs.
Whenever there is a need to break compatibility, it is announced here in the changelog, and raises a ``DeprecationWarning`` for a year (if possible) before it's finally really broken.

.. warning::

   The structure of the `attrs.Attribute` class is exempt from this rule.
   It *will* change in the future, but since it should be considered read-only, that shouldn't matter.

   However if you intend to build extensions on top of ``attrs`` you have to anticipate that.

.. towncrier release notes start

21.4.0 (2021-12-29)
-------------------

Changes
^^^^^^^

- Fixed the test suite on PyPy3.8 where ``cloudpickle`` does not work.
  `#892 <https://github.com/python-attrs/attrs/issues/892>`_
- Fixed ``coverage report`` for projects that use ``attrs`` and don't set a ``--source``.
  `#895 <https://github.com/python-attrs/attrs/issues/895>`_,
  `#896 <https://github.com/python-attrs/attrs/issues/896>`_


----


21.3.0 (2021-12-28)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- When using ``@define``, converters are now run by default when setting an attribute on an instance -- additionally to validators.
  I.e. the new default is ``on_setattr=[attrs.setters.convert, attrs.setters.validate]``.

  This is unfortunately a breaking change, but it was an oversight, impossible to raise a ``DeprecationWarning`` about, and it's better to fix it now while the APIs are very fresh with few users.
  `#835 <https://github.com/python-attrs/attrs/issues/835>`_,
  `#886 <https://github.com/python-attrs/attrs/issues/886>`_
- ``import attrs`` has finally landed!
  As of this release, you can finally import ``attrs`` using its proper name.

  Not all names from the ``attr`` namespace have been transferred; most notably ``attr.s`` and ``attr.ib`` are missing.
  See ``attrs.define`` and ``attrs.field`` if you haven't seen our next-generation APIs yet.
  A more elaborate explanation can be found `On The Core API Names <https://www.attrs.org/en/latest/names.html>`_

  This feature is at least for one release **provisional**.
  We don't *plan* on changing anything, but such a big change is unlikely to go perfectly on the first strike.

  The API docs have been mostly updated, but it will be an ongoing effort to change everything to the new APIs.
  Please note that we have **not** moved -- or even removed -- anything from ``attr``!

  Please do report any bugs or documentation inconsistencies!
  `#887 <https://github.com/python-attrs/attrs/issues/887>`_


Changes
^^^^^^^

- ``attr.asdict(retain_collection_types=False)`` (default) dumps collection-esque keys as tuples.
  `#646 <https://github.com/python-attrs/attrs/issues/646>`_,
  `#888 <https://github.com/python-attrs/attrs/issues/888>`_
- ``__match_args__`` are now generated to support Python 3.10's
  `Structural Pattern Matching <https://docs.python.org/3.10/whatsnew/3.10.html#pep-634-structural-pattern-matching>`_.
  This can be controlled by the ``match_args`` argument to the class decorators on Python 3.10 and later.
  On older versions, it is never added and the argument is ignored.
  `#815 <https://github.com/python-attrs/attrs/issues/815>`_
- If the class-level *on_setattr* is set to ``attrs.setters.validate`` (default in ``@define`` and ``@mutable``) but no field defines a validator, pretend that it's not set.
  `#817 <https://github.com/python-attrs/attrs/issues/817>`_
- The generated ``__repr__`` is significantly faster on Pythons with f-strings.
  `#819 <https://github.com/python-attrs/attrs/issues/819>`_
- Attributes transformed via ``field_transformer`` are wrapped with ``AttrsClass`` again.
  `#824 <https://github.com/python-attrs/attrs/issues/824>`_
- Generated source code is now cached more efficiently for identical classes.
  `#828 <https://github.com/python-attrs/attrs/issues/828>`_
- Added ``attrs.converters.to_bool()``.
  `#830 <https://github.com/python-attrs/attrs/issues/830>`_
- ``attrs.resolve_types()`` now resolves types of subclasses after the parents are resolved.
  `#842 <https://github.com/python-attrs/attrs/issues/842>`_
  `#843 <https://github.com/python-attrs/attrs/issues/843>`_
- Added new validators: ``lt(val)`` (< val), ``le(va)`` (≤ val), ``ge(val)`` (≥ val), ``gt(val)`` (> val), and ``maxlen(n)``.
  `#845 <https://github.com/python-attrs/attrs/issues/845>`_
- ``attrs`` classes are now fully compatible with `cloudpickle <https://github.com/cloudpipe/cloudpickle>`_ (no need to disable ``repr`` anymore).
  `#857 <https://github.com/python-attrs/attrs/issues/857>`_
- Added new context manager ``attrs.validators.disabled()`` and functions ``attrs.validators.(set|get)_disabled()``.
  They deprecate ``attrs.(set|get)_run_validators()``.
  All functions are interoperable and modify the same internal state.
  They are not – and never were – thread-safe, though.
  `#859 <https://github.com/python-attrs/attrs/issues/859>`_
- ``attrs.validators.matches_re()`` now accepts pre-compiled regular expressions in addition to pattern strings.
  `#877 <https://github.com/python-attrs/attrs/issues/877>`_


----


21.2.0 (2021-05-07)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- We had to revert the recursive feature for ``attr.evolve()`` because it broke some use-cases -- sorry!
  `#806 <https://github.com/python-attrs/attrs/issues/806>`_
- Python 3.4 is now blocked using packaging metadata because ``attrs`` can't be imported on it anymore.
  To ensure that 3.4 users can keep installing  ``attrs`` easily, we will `yank <https://pypi.org/help/#yanked>`_ 21.1.0 from PyPI.
  This has **no** consequences if you pin ``attrs`` to 21.1.0.
  `#807 <https://github.com/python-attrs/attrs/issues/807>`_


----


21.1.0 (2021-05-06)
-------------------

Deprecations
^^^^^^^^^^^^

- The long-awaited, much-talked-about, little-delivered ``import attrs`` is finally upon us!

  Since the NG APIs have now been proclaimed stable, the **next** release of ``attrs`` will allow you to actually ``import attrs``.
  We're taking this opportunity to replace some defaults in our APIs that made sense in 2015, but don't in 2021.

  So please, if you have any pet peeves about defaults in ``attrs``'s APIs, *now* is the time to air your grievances in #487!
  We're not gonna get such a chance for a second time, without breaking our backward-compatibility guarantees, or long deprecation cycles.
  Therefore, speak now or forever hold you peace!
  `#487 <https://github.com/python-attrs/attrs/issues/487>`_
- The *cmp* argument to ``attr.s()`` and `attr.ib()` has been **undeprecated**
  It will continue to be supported as syntactic sugar to set *eq* and *order* in one go.

  I'm terribly sorry for the hassle around this argument!
  The reason we're bringing it back is it's usefulness regarding customization of equality/ordering.

  The ``cmp`` attribute and argument on ``attr.Attribute`` remains deprecated and will be removed later this year.
  `#773 <https://github.com/python-attrs/attrs/issues/773>`_


Changes
^^^^^^^

- It's now possible to customize the behavior of ``eq`` and ``order`` by passing in a callable.
  `#435 <https://github.com/python-attrs/attrs/issues/435>`_,
  `#627 <https://github.com/python-attrs/attrs/issues/627>`_
- The instant favorite next-generation APIs are not provisional anymore!

  They are also officially supported by Mypy as of their `0.800 release <https://mypy-lang.blogspot.com/2021/01/mypy-0800-released.html>`_.

  We hope the next release will already contain an (additional) importable package called ``attrs``.
  `#668 <https://github.com/python-attrs/attrs/issues/668>`_,
  `#786 <https://github.com/python-attrs/attrs/issues/786>`_
- If an attribute defines a converter, the type of its parameter is used as type annotation for its corresponding ``__init__`` parameter.

  If an ``attr.converters.pipe`` is used, the first one's is used.
  `#710 <https://github.com/python-attrs/attrs/issues/710>`_
- Fixed the creation of an extra slot for an ``attr.ib`` when the parent class already has a slot with the same name.
  `#718 <https://github.com/python-attrs/attrs/issues/718>`_
- ``__attrs__init__()`` will now be injected if ``init=False``, or if ``auto_detect=True`` and a user-defined ``__init__()`` exists.

  This enables users to do "pre-init" work in their ``__init__()`` (such as ``super().__init__()``).

  ``__init__()`` can then delegate constructor argument processing to ``self.__attrs_init__(*args, **kwargs)``.
  `#731 <https://github.com/python-attrs/attrs/issues/731>`_
- ``bool(attr.NOTHING)`` is now ``False``.
  `#732 <https://github.com/python-attrs/attrs/issues/732>`_
- It's now possible to use ``super()`` inside of properties of slotted classes.
  `#747 <https://github.com/python-attrs/attrs/issues/747>`_
- Allow for a ``__attrs_pre_init__()`` method that -- if defined -- will get called at the beginning of the ``attrs``-generated ``__init__()`` method.
  `#750 <https://github.com/python-attrs/attrs/issues/750>`_
- Added forgotten ``attr.Attribute.evolve()`` to type stubs.
  `#752 <https://github.com/python-attrs/attrs/issues/752>`_
- ``attrs.evolve()`` now works recursively with nested ``attrs`` classes.
  `#759 <https://github.com/python-attrs/attrs/issues/759>`_
- Python 3.10 is now officially supported.
  `#763 <https://github.com/python-attrs/attrs/issues/763>`_
- ``attr.resolve_types()`` now takes an optional *attrib* argument to work inside a ``field_transformer``.
  `#774 <https://github.com/python-attrs/attrs/issues/774>`_
- ``ClassVar``\ s are now also detected if they come from `typing-extensions <https://pypi.org/project/typing-extensions/>`_.
  `#782 <https://github.com/python-attrs/attrs/issues/782>`_
- To make it easier to customize attribute comparison (#435), we have added the ``attr.cmp_with()`` helper.

  See the `new docs on comparison <https://www.attrs.org/en/stable/comparison.html>`_ for more details.
  `#787 <https://github.com/python-attrs/attrs/issues/787>`_
- Added **provisional** support for static typing in ``pyright`` via the `dataclass_transforms specification <https://github.com/microsoft/pyright/blob/main/specs/dataclass_transforms.md>`_.
  Both the ``pyright`` specification and ``attrs`` implementation may change in future versions of both projects.

  Your constructive feedback is welcome in both `attrs#795 <https://github.com/python-attrs/attrs/issues/795>`_ and `pyright#1782 <https://github.com/microsoft/pyright/discussions/1782>`_.
  `#796 <https://github.com/python-attrs/attrs/issues/796>`_


----


20.3.0 (2020-11-05)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- ``attr.define()``, ``attr.frozen()``, ``attr.mutable()``, and ``attr.field()`` remain **provisional**.

  This release does **not** change anything about them and they are already used widely in production though.

  If you wish to use them together with mypy, you can simply drop `this plugin <https://gist.github.com/hynek/1e3844d0c99e479e716169034b5fa963#file-attrs_ng_plugin-py>`_ into your project.

  Feel free to provide feedback to them in the linked issue #668.

  We will release the ``attrs`` namespace once we have the feeling that the APIs have properly settled.
  `#668 <https://github.com/python-attrs/attrs/issues/668>`_


Changes
^^^^^^^

- ``attr.s()`` now has a *field_transformer* hook that is called for all ``Attribute``\ s and returns a (modified or updated) list of ``Attribute`` instances.
  ``attr.asdict()`` has a *value_serializer* hook that can change the way values are converted.
  Both hooks are meant to help with data (de-)serialization workflows.
  `#653 <https://github.com/python-attrs/attrs/issues/653>`_
- ``kw_only=True`` now works on Python 2.
  `#700 <https://github.com/python-attrs/attrs/issues/700>`_
- ``raise from`` now works on frozen classes on PyPy.
  `#703 <https://github.com/python-attrs/attrs/issues/703>`_,
  `#712 <https://github.com/python-attrs/attrs/issues/712>`_
- ``attr.asdict()`` and ``attr.astuple()`` now treat ``frozenset``\ s like ``set``\ s with regards to the *retain_collection_types* argument.
  `#704 <https://github.com/python-attrs/attrs/issues/704>`_
- The type stubs for ``attr.s()`` and ``attr.make_class()`` are not missing the *collect_by_mro* argument anymore.
  `#711 <https://github.com/python-attrs/attrs/issues/711>`_


----


20.2.0 (2020-09-05)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- ``attr.define()``, ``attr.frozen()``, ``attr.mutable()``, and ``attr.field()`` remain **provisional**.

  This release fixes a bunch of bugs and ergonomics but they remain mostly unchanged.

  If you wish to use them together with mypy, you can simply drop `this plugin <https://gist.github.com/hynek/1e3844d0c99e479e716169034b5fa963#file-attrs_ng_plugin-py>`_ into your project.

  Feel free to provide feedback to them in the linked issue #668.

  We will release the ``attrs`` namespace once we have the feeling that the APIs have properly settled.
  `#668 <https://github.com/python-attrs/attrs/issues/668>`_


Changes
^^^^^^^

- ``attr.define()`` et al now correct detect ``__eq__`` and ``__ne__``.
  `#671 <https://github.com/python-attrs/attrs/issues/671>`_
- ``attr.define()`` et al's hybrid behavior now also works correctly when arguments are passed.
  `#675 <https://github.com/python-attrs/attrs/issues/675>`_
- It's possible to define custom ``__setattr__`` methods on slotted classes again.
  `#681 <https://github.com/python-attrs/attrs/issues/681>`_
- In 20.1.0 we introduced the ``inherited`` attribute on the ``attr.Attribute`` class to differentiate attributes that have been inherited and those that have been defined directly on the class.

  It has shown to be problematic to involve that attribute when comparing instances of ``attr.Attribute`` though, because when sub-classing, attributes from base classes are suddenly not equal to themselves in a super class.

  Therefore the ``inherited`` attribute will now be ignored when hashing and comparing instances of ``attr.Attribute``.
  `#684 <https://github.com/python-attrs/attrs/issues/684>`_
- ``zope.interface`` is now a "soft dependency" when running the test suite; if ``zope.interface`` is not installed when running the test suite, the interface-related tests will be automatically skipped.
  `#685 <https://github.com/python-attrs/attrs/issues/685>`_
- The ergonomics of creating frozen classes using ``@define(frozen=True)`` and sub-classing frozen classes has been improved:
  you don't have to set ``on_setattr=None`` anymore.
  `#687 <https://github.com/python-attrs/attrs/issues/687>`_


----


20.1.0 (2020-08-20)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- Python 3.4 is not supported anymore.
  It has been unsupported by the Python core team for a while now, its PyPI downloads are negligible, and our CI provider removed it as a supported option.

  It's very unlikely that ``attrs`` will break under 3.4 anytime soon, which is why we do *not* block its installation on Python 3.4.
  But we don't test it anymore and will block it once someone reports breakage.
  `#608 <https://github.com/python-attrs/attrs/issues/608>`_


Deprecations
^^^^^^^^^^^^

- Less of a deprecation and more of a heads up: the next release of ``attrs`` will introduce an ``attrs`` namespace.
  That means that you'll finally be able to run ``import attrs`` with new functions that aren't cute abbreviations and that will carry better defaults.

  This should not break any of your code, because project-local packages have priority before installed ones.
  If this is a problem for you for some reason, please report it to our bug tracker and we'll figure something out.

  The old ``attr`` namespace isn't going anywhere and its defaults are not changing – this is a purely additive measure.
  Please check out the linked issue for more details.

  These new APIs have been added *provisionally* as part of #666 so you can try them out today and provide feedback.
  Learn more in the `API docs <https://www.attrs.org/en/stable/api.html>`_.
  `#408 <https://github.com/python-attrs/attrs/issues/408>`_


Changes
^^^^^^^

- Added ``attr.resolve_types()``.
  It ensures that all forward-references and types in string form are resolved into concrete types.

  You need this only if you need concrete types at runtime.
  That means that if you only use types for static type checking, you do **not** need this function.
  `#288 <https://github.com/python-attrs/attrs/issues/288>`_,
  `#302 <https://github.com/python-attrs/attrs/issues/302>`_
- Added ``@attr.s(collect_by_mro=False)`` argument that if set to ``True`` fixes the collection of attributes from base classes.

  It's only necessary for certain cases of multiple-inheritance but is kept off for now for backward-compatibility reasons.
  It will be turned on by default in the future.

  As a side-effect, ``attr.Attribute`` now *always* has an ``inherited`` attribute indicating whether an attribute on a class was directly defined or inherited.
  `#428 <https://github.com/python-attrs/attrs/issues/428>`_,
  `#635 <https://github.com/python-attrs/attrs/issues/635>`_
- On Python 3, all generated methods now have a docstring explaining that they have been created by ``attrs``.
  `#506 <https://github.com/python-attrs/attrs/issues/506>`_
- It is now possible to prevent ``attrs`` from auto-generating the ``__setstate__`` and ``__getstate__`` methods that are required for pickling of slotted classes.

  Either pass ``@attr.s(getstate_setstate=False)`` or pass ``@attr.s(auto_detect=True)`` and implement them yourself:
  if ``attrs`` finds either of the two methods directly on the decorated class, it assumes implicitly ``getstate_setstate=False`` (and implements neither).

  This option works with dict classes but should never be necessary.
  `#512 <https://github.com/python-attrs/attrs/issues/512>`_,
  `#513 <https://github.com/python-attrs/attrs/issues/513>`_,
  `#642 <https://github.com/python-attrs/attrs/issues/642>`_
- Fixed a ``ValueError: Cell is empty`` bug that could happen in some rare edge cases.
  `#590 <https://github.com/python-attrs/attrs/issues/590>`_
- ``attrs`` can now automatically detect your own implementations and infer ``init=False``, ``repr=False``, ``eq=False``, ``order=False``, and ``hash=False`` if you set ``@attr.s(auto_detect=True)``.
  ``attrs`` will ignore inherited methods.
  If the argument implies more than one method (e.g. ``eq=True`` creates both ``__eq__`` and ``__ne__``), it's enough for *one* of them to exist and ``attrs`` will create *neither*.

  This feature requires Python 3.
  `#607 <https://github.com/python-attrs/attrs/issues/607>`_
- Added ``attr.converters.pipe()``.
  The feature allows combining multiple conversion callbacks into one by piping the value through all of them, and retuning the last result.

  As part of this feature, we had to relax the type information for converter callables.
  `#618 <https://github.com/python-attrs/attrs/issues/618>`_
- Fixed serialization behavior of non-slots classes with ``cache_hash=True``.
  The hash cache will be cleared on operations which make "deep copies" of instances of classes with hash caching,
  though the cache will not be cleared with shallow copies like those made by ``copy.copy()``.

  Previously, ``copy.deepcopy()`` or serialization and deserialization with ``pickle`` would result in an un-initialized object.

  This change also allows the creation of ``cache_hash=True`` classes with a custom ``__setstate__``,
  which was previously forbidden (`#494 <https://github.com/python-attrs/attrs/issues/494>`_).
  `#620 <https://github.com/python-attrs/attrs/issues/620>`_
- It is now possible to specify hooks that are called whenever an attribute is set **after** a class has been instantiated.

  You can pass ``on_setattr`` both to ``@attr.s()`` to set the default for all attributes on a class, and to ``@attr.ib()`` to overwrite it for individual attributes.

  ``attrs`` also comes with a new module ``attr.setters`` that brings helpers that run validators, converters, or allow to freeze a subset of attributes.
  `#645 <https://github.com/python-attrs/attrs/issues/645>`_,
  `#660 <https://github.com/python-attrs/attrs/issues/660>`_
- **Provisional** APIs called ``attr.define()``, ``attr.mutable()``, and ``attr.frozen()`` have been added.

  They are only available on Python 3.6 and later, and call ``attr.s()`` with different default values.

  If nothing comes up, they will become the official way for creating classes in 20.2.0 (see above).

  **Please note** that it may take some time until mypy – and other tools that have dedicated support for ``attrs`` – recognize these new APIs.
  Please **do not** open issues on our bug tracker, there is nothing we can do about it.
  `#666 <https://github.com/python-attrs/attrs/issues/666>`_
- We have also provisionally added ``attr.field()`` that supplants ``attr.ib()``.
  It also requires at least Python 3.6 and is keyword-only.
  Other than that, it only dropped a few arguments, but changed no defaults.

  As with ``attr.s()``: ``attr.ib()`` is not going anywhere.
  `#669 <https://github.com/python-attrs/attrs/issues/669>`_


----


19.3.0 (2019-10-15)
-------------------

Changes
^^^^^^^

- Fixed ``auto_attribs`` usage when default values cannot be compared directly with ``==``, such as ``numpy`` arrays.
  `#585 <https://github.com/python-attrs/attrs/issues/585>`_


----


19.2.0 (2019-10-01)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- Removed deprecated ``Attribute`` attribute ``convert`` per scheduled removal on 2019/1.
  This planned deprecation is tracked in issue `#307 <https://github.com/python-attrs/attrs/issues/307>`_.
  `#504 <https://github.com/python-attrs/attrs/issues/504>`_
- ``__lt__``, ``__le__``, ``__gt__``, and ``__ge__`` do not consider subclasses comparable anymore.

  This has been deprecated since 18.2.0 and was raising a ``DeprecationWarning`` for over a year.
  `#570 <https://github.com/python-attrs/attrs/issues/570>`_


Deprecations
^^^^^^^^^^^^

- The ``cmp`` argument to ``attr.s()`` and ``attr.ib()`` is now deprecated.

  Please use ``eq`` to add equality methods (``__eq__`` and ``__ne__``) and ``order`` to add ordering methods (``__lt__``, ``__le__``, ``__gt__``, and ``__ge__``) instead – just like with `dataclasses <https://docs.python.org/3/library/dataclasses.html>`_.

  Both are effectively ``True`` by default but it's enough to set ``eq=False`` to disable both at once.
  Passing ``eq=False, order=True`` explicitly will raise a ``ValueError`` though.

  Since this is arguably a deeper backward-compatibility break, it will have an extended deprecation period until 2021-06-01.
  After that day, the ``cmp`` argument will be removed.

  ``attr.Attribute`` also isn't orderable anymore.
  `#574 <https://github.com/python-attrs/attrs/issues/574>`_


Changes
^^^^^^^

- Updated ``attr.validators.__all__`` to include new validators added in `#425`_.
  `#517 <https://github.com/python-attrs/attrs/issues/517>`_
- Slotted classes now use a pure Python mechanism to rewrite the ``__class__`` cell when rebuilding the class, so ``super()`` works even on environments where ``ctypes`` is not installed.
  `#522 <https://github.com/python-attrs/attrs/issues/522>`_
- When collecting attributes using ``@attr.s(auto_attribs=True)``, attributes with a default of ``None`` are now deleted too.
  `#523 <https://github.com/python-attrs/attrs/issues/523>`_,
  `#556 <https://github.com/python-attrs/attrs/issues/556>`_
- Fixed ``attr.validators.deep_iterable()`` and ``attr.validators.deep_mapping()`` type stubs.
  `#533 <https://github.com/python-attrs/attrs/issues/533>`_
- ``attr.validators.is_callable()`` validator now raises an exception ``attr.exceptions.NotCallableError``, a subclass of ``TypeError``, informing the received value.
  `#536 <https://github.com/python-attrs/attrs/issues/536>`_
- ``@attr.s(auto_exc=True)`` now generates classes that are hashable by ID, as the documentation always claimed it would.
  `#543 <https://github.com/python-attrs/attrs/issues/543>`_,
  `#563 <https://github.com/python-attrs/attrs/issues/563>`_
- Added ``attr.validators.matches_re()`` that checks string attributes whether they match a regular expression.
  `#552 <https://github.com/python-attrs/attrs/issues/552>`_
- Keyword-only attributes (``kw_only=True``) and attributes that are excluded from the ``attrs``'s ``__init__`` (``init=False``) now can appear before mandatory attributes.
  `#559 <https://github.com/python-attrs/attrs/issues/559>`_
- The fake filename for generated methods is now more stable.
  It won't change when you restart the process.
  `#560 <https://github.com/python-attrs/attrs/issues/560>`_
- The value passed to ``@attr.ib(repr=…)`` can now be either a boolean (as before) or a callable.
  That callable must return a string and is then used for formatting the attribute by the generated ``__repr__()`` method.
  `#568 <https://github.com/python-attrs/attrs/issues/568>`_
- Added ``attr.__version_info__`` that can be used to reliably check the version of ``attrs`` and write forward- and backward-compatible code.
  Please check out the `section on deprecated APIs <http://www.attrs.org/en/stable/api.html#deprecated-apis>`_ on how to use it.
  `#580 <https://github.com/python-attrs/attrs/issues/580>`_

 .. _`#425`: https://github.com/python-attrs/attrs/issues/425


----


19.1.0 (2019-03-03)
-------------------

Backward-incompatible Changes
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- Fixed a bug where deserialized objects with ``cache_hash=True`` could have incorrect hash code values.
  This change breaks classes with ``cache_hash=True`` when a custom ``__setstate__`` is present.
  An exception will be thrown when applying the ``attrs`` annotation to such a class.
  This limitation is tracked in issue `#494 <https://github.com/python-attrs/attrs/issues/494>`_.
  `#482 <https://github.com/python-attrs/attrs/issues/482>`_


Changes
^^^^^^^

- Add ``is_callable``, ``deep_iterable``, and ``deep_mapping`` validators.

  * ``is_callable``: validates that a value is callable
  * ``deep_iterable``: Allows recursion down into an iterable,
    applying another validator to every member in the iterable
    as well as applying an optional validator to the iterable itself.
  * ``deep_mapping``: Allows recursion down into the items in a mapping object,
    applying a key validator and a value validator to the key and value in every item.
    Also applies an optional validator to the mapping object itself.

  You can find them in the ``attr.validators`` package.
  `#425`_
- Fixed stub files to prevent errors raised by mypy's ``disallow_any_generics = True`` option.
  `#443 <https://github.com/python-attrs/attrs/issues/443>`_
- Attributes with ``init=False`` now can follow after ``kw_only=True`` attributes.
  `#450 <https://github.com/python-attrs/attrs/issues/450>`_
- ``attrs`` now has first class support for defining exception classes.

  If you define a class using ``@attr.s(auto_exc=True)`` and subclass an exception, the class will behave like a well-behaved exception class including an appropriate ``__str__`` method, and all attributes additionally available in an ``args`` attribute.
  `#500 <https://github.com/python-attrs/attrs/issues/500>`_
- Clarified documentation for hashing to warn that hashable objects should be deeply immutable (in their usage, even if this is not enforced).
  `#503 <https://github.com/python-attrs/attrs/issues/503>`_


----


18.2.0 (2018-09-01)
-------------------

Deprecations
^^^^^^^^^^^^

- Comparing subclasses using ``<``, ``>``, ``<=``, and ``>=`` is now deprecated.
  The docs always claimed that instances are only compared if the types are identical, so this is a first step to conform to the docs.

  Equality operators (``==`` and ``!=``) were always strict in this regard.
  `#394 <https://github.com/python-attrs/attrs/issues/394>`_


Changes
^^^^^^^

- ``attrs`` now ships its own `PEP 484 <https://www.python.org/dev/peps/pep-0484/>`_ type hints.
  Together with `mypy <http://mypy-lang.org>`_'s ``attrs`` plugin, you've got all you need for writing statically typed code in both Python 2 and 3!

  At that occasion, we've also added `narrative docs <https://www.attrs.org/en/stable/types.html>`_ about type annotations in ``attrs``.
  `#238 <https://github.com/python-attrs/attrs/issues/238>`_
- Added *kw_only* arguments to ``attr.ib`` and ``attr.s``, and a corresponding *kw_only* attribute to ``attr.Attribute``.
  This change makes it possible to have a generated ``__init__`` with keyword-only arguments on Python 3, relaxing the required ordering of default and non-default valued attributes.
  `#281 <https://github.com/python-attrs/attrs/issues/281>`_,
  `#411 <https://github.com/python-attrs/attrs/issues/411>`_
- The test suite now runs with ``hypothesis.HealthCheck.too_slow`` disabled to prevent CI breakage on slower computers.
  `#364 <https://github.com/python-attrs/attrs/issues/364>`_,
  `#396 <https://github.com/python-attrs/attrs/issues/396>`_
- ``attr.validators.in_()`` now raises a ``ValueError`` with a useful message even if the options are a string and the value is not a string.
  `#383 <https://github.com/python-attrs/attrs/issues/383>`_
- ``attr.asdict()`` now properly handles deeply nested lists and dictionaries.
  `#395 <https://github.com/python-attrs/attrs/issues/395>`_
- Added ``attr.converters.default_if_none()`` that allows to replace ``None`` values in attributes.
  For example ``attr.ib(converter=default_if_none(""))`` replaces ``None`` by empty strings.
  `#400 <https://github.com/python-attrs/attrs/issues/400>`_,
  `#414 <https://github.com/python-attrs/attrs/issues/414>`_
- Fixed a reference leak where the original class would remain live after being replaced when ``slots=True`` is set.
  `#407 <https://github.com/python-attrs/attrs/issues/407>`_
- Slotted classes can now be made weakly referenceable by passing ``@attr.s(weakref_slot=True)``.
  `#420 <https://github.com/python-attrs/attrs/issues/420>`_
- Added *cache_hash* option to ``@attr.s`` which causes the hash code to be computed once and stored on the object.
  `#426 <https://github.com/python-attrs/attrs/issues/426>`_
- Attributes can be named ``property`` and ``itemgetter`` now.
  `#430 <https://github.com/python-attrs/attrs/issues/430>`_
- It is now possible to override a base class' class variable using only class annotations.
  `#431 <https://github.com/python-attrs/attrs/issues/431>`_


----


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
- The order of attributes that are passed into ``attr.make_class()`` or the *these* argument of ``@attr.s()`` is now retained if the dictionary is ordered (i.e. ``dict`` on Python 3.6 and later, ``collections.OrderedDict`` otherwise).

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
- If ``attr.s`` is passed a *these* argument, it will no longer attempt to remove attributes with the same name from the class body.

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
  Instead of putting everything into the `examples <https://www.attrs.org/en/stable/examples.html>`_ page, we have started to extract narrative chapters.

  So far, we've added chapters on `initialization <https://www.attrs.org/en/stable/init.html>`_ and `hashing <https://www.attrs.org/en/stable/hashing.html>`_.

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
  Because of the nature of this bug, a proper deprecation cycle was unfortunately impossible.

  Generally speaking, it's advisable to prefer ``kwargs``-based initialization anyways – *especially* if you employ multiple inheritance and diamond-shaped hierarchies.

  `#298 <https://github.com/python-attrs/attrs/issues/298>`_,
  `#299 <https://github.com/python-attrs/attrs/issues/299>`_,
  `#304 <https://github.com/python-attrs/attrs/issues/304>`_
- The ``__repr__`` set by ``attrs`` no longer produces an ``AttributeError`` when the instance is missing some of the specified attributes (either through deleting or after using ``init=False`` on some attributes).

  This can break code that relied on ``repr(attr_cls_instance)`` raising ``AttributeError`` to check if any ``attrs``-specified members were unset.

  If you were using this, you can implement a custom method for checking this::

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
- ``attr.ib``\ ’s *metadata* argument now defaults to a unique empty ``dict`` instance instead of sharing a common empty ``dict`` for all.
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

- Attributes are no longer defined on the class body.

  This means that if you define a class ``C`` with an attribute ``x``, the class will *not* have an attribute ``x`` for introspection.
  Instead of ``C.x``, use ``attr.fields(C).x`` or look at ``C.__attrs_attrs__``.
  The old behavior has been deprecated since version 16.1.
  (`#253 <https://github.com/python-attrs/attrs/issues/253>`_)


Changes
^^^^^^^

- ``super()`` and ``__class__`` now work with slotted classes on Python 3.
  (`#102 <https://github.com/python-attrs/attrs/issues/102>`_, `#226 <https://github.com/python-attrs/attrs/issues/226>`_, `#269 <https://github.com/python-attrs/attrs/issues/269>`_, `#270 <https://github.com/python-attrs/attrs/issues/270>`_, `#272 <https://github.com/python-attrs/attrs/issues/272>`_)
- Added *type* argument to ``attr.ib()`` and corresponding ``type`` attribute to ``attr.Attribute``.

  This change paves the way for automatic type checking and serialization (though as of this release ``attrs`` does not make use of it).
  In Python 3.6 or higher, the value of ``attr.Attribute.type`` can alternately be set using variable type annotations
  (see `PEP 526 <https://www.python.org/dev/peps/pep-0526/>`_).
  (`#151 <https://github.com/python-attrs/attrs/issues/151>`_, `#214 <https://github.com/python-attrs/attrs/issues/214>`_, `#215 <https://github.com/python-attrs/attrs/issues/215>`_, `#239 <https://github.com/python-attrs/attrs/issues/239>`_)
- The combination of ``str=True`` and ``slots=True`` now works on Python 2.
  (`#198 <https://github.com/python-attrs/attrs/issues/198>`_)
- ``attr.Factory`` is hashable again.
  (`#204 <https://github.com/python-attrs/attrs/issues/204>`_)
- Subclasses now can overwrite attribute definitions of their base classes.

  That means that you can -- for example -- change the default value for an attribute by redefining it.
  (`#221 <https://github.com/python-attrs/attrs/issues/221>`_, `#229 <https://github.com/python-attrs/attrs/issues/229>`_)
- Added new option *auto_attribs* to ``@attr.s`` that allows to collect annotated fields without setting them to ``attr.ib()``.

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

.. image:: https://www.attrs.org/en/latest/_static/attrs_logo.png
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

- Correspondingly, ``attr.ib``'s *hash* argument is ``None`` by default too and mirrors the *cmp* argument as it should.


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
- Added ``attr.validators.in_(options)`` that, given the allowed ``options``, checks whether the attribute value is in it.
  This can be used to check constants, enums, mappings, etc.
  `#181 <https://github.com/python-attrs/attrs/pull/181>`_
- Added ``attr.validators.and_()`` that composes multiple validators into one.
  `#161 <https://github.com/python-attrs/attrs/issues/161>`_
- For convenience, the *validator* argument of ``@attr.s`` now can take a list of validators that are wrapped using ``and_()``.
  `#138 <https://github.com/python-attrs/attrs/issues/138>`_
- Accordingly, ``attr.validators.optional()`` now can take a list of validators too.
  `#161 <https://github.com/python-attrs/attrs/issues/161>`_
- Validators can now be defined conveniently inline by using the attribute as a decorator.
  Check out the `validator examples <http://www.attrs.org/en/stable/init.html#decorator>`_ to see it in action!
  `#143 <https://github.com/python-attrs/attrs/issues/143>`_
- ``attr.Factory()`` now has a *takes_self* argument that makes the initializer to pass the partially initialized instance into the factory.
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
- ``__name__`` is no longer overwritten with ``__qualname__`` for ``attr.s(slots=True)`` classes.
  `#99 <https://github.com/python-attrs/attrs/issues/99>`_


----


16.2.0 (2016-09-17)
-------------------

Changes:
^^^^^^^^

- Added ``attr.astuple()`` that -- similarly to ``attr.asdict()`` -- returns the instance as a tuple.
  `#77 <https://github.com/python-attrs/attrs/issues/77>`_
- Converters now work with frozen classes.
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
- Add *frozen* option to ``attr.s`` that will make instances best-effort immutable.
  `#60 <https://github.com/python-attrs/attrs/issues/60>`_
- ``attr.asdict()`` now takes ``retain_collection_types`` as an argument.
  If ``True``, it does not convert attributes of type ``tuple`` or ``set`` to ``list``.
  `#69 <https://github.com/python-attrs/attrs/issues/69>`_


----


16.0.0 (2016-05-23)
-------------------

Backward-incompatible Changes:
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

- Python 3.3 and 2.6 are no longer supported.
  They may work by chance but any effort to keep them working has ceased.

  The last Python 2.6 release was on October 29, 2013 and is no longer supported by the CPython core team.
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
