# Changelog

Versions follow [CalVer](https://calver.org) with a strict backwards-compatibility policy.

The **first number** of the version is the year.
The **second number** is incremented with each release, starting at 1 for each year.
The **third number** is when we need to start branches for older releases (only for emergencies).

You can find out backwards-compatibility policy [here](https://github.com/python-attrs/attrs/blob/main/.github/SECURITY.md).

Changes for the upcoming release can be found in the ["changelog.d" directory](https://github.com/python-attrs/attrs/tree/main/changelog.d) in our repository.

<!-- towncrier release notes start -->

## [23.2.0](https://github.com/python-attrs/attrs/tree/23.2.0) - 2023-12-31

### Changes

- The type annotation for `attrs.resolve_types()` is now correct.
  [#1141](https://github.com/python-attrs/attrs/issues/1141)
- Type stubs now use `typing.dataclass_transform` to decorate dataclass-like decorators, instead of the non-standard `__dataclass_transform__` special form, which is only supported by Pyright.
  [#1158](https://github.com/python-attrs/attrs/issues/1158)
- Fixed serialization of namedtuple fields using `attrs.asdict/astuple()` with `retain_collection_types=True`.
  [#1165](https://github.com/python-attrs/attrs/issues/1165)
- `attrs.AttrsInstance` is now a `typing.Protocol` in both type hints and code.
  This allows you to subclass it along with another `Protocol`.
  [#1172](https://github.com/python-attrs/attrs/issues/1172)
- If *attrs* detects that `__attrs_pre_init__` accepts more than just `self`, it will call it with the same arguments as `__init__` was called.
  This allows you to, for example, pass arguments to `super().__init__()`.
  [#1187](https://github.com/python-attrs/attrs/issues/1187)
- Slotted classes now transform `functools.cached_property` decorated methods to support equivalent semantics.
  [#1200](https://github.com/python-attrs/attrs/issues/1200)
- Added *class_body* argument to `attrs.make_class()` to provide additional attributes for newly created classes.
  It is, for example, now possible to attach methods.
  [#1203](https://github.com/python-attrs/attrs/issues/1203)

## [23.1.0](https://github.com/python-attrs/attrs/tree/23.1.0) - 2023-04-16

### Backwards-incompatible Changes

- Python 3.6 has been dropped and packaging switched to static package data using [Hatch](https://hatch.pypa.io/latest/).
  [#993](https://github.com/python-attrs/attrs/issues/993)


### Deprecations

- The support for *zope-interface* via the `attrs.validators.provides` validator is now deprecated and will be removed in, or after, April 2024.

  The presence of a C-based package in our developement dependencies has caused headaches and we're not under the impression it's used a lot.

  Let us know if you're using it and we might publish it as a separate package.
  [#1120](https://github.com/python-attrs/attrs/issues/1120)


### Changes

- `attrs.filters.exclude()` and `attrs.filters.include()` now support the passing of attribute names as strings.
  [#1068](https://github.com/python-attrs/attrs/issues/1068)
- `attrs.has()` and `attrs.fields()` now handle generic classes correctly.
  [#1079](https://github.com/python-attrs/attrs/issues/1079)
- Fix frozen exception classes when raised within e.g. `contextlib.contextmanager`, which mutates their `__traceback__` attributes.
  [#1081](https://github.com/python-attrs/attrs/issues/1081)
- `@frozen` now works with type checkers that implement [PEP-681](https://peps.python.org/pep-0681/) (ex. [pyright](https://github.com/microsoft/pyright/)).
  [#1084](https://github.com/python-attrs/attrs/issues/1084)
- Restored ability to unpickle instances pickled before 22.2.0.
  [#1085](https://github.com/python-attrs/attrs/issues/1085)
- `attrs.asdict()`'s and `attrs.astuple()`'s type stubs now accept the `attrs.AttrsInstance` protocol.
  [#1090](https://github.com/python-attrs/attrs/issues/1090)
- Fix slots class cellvar updating closure in CPython 3.8+ even when `__code__` introspection is unavailable.
  [#1092](https://github.com/python-attrs/attrs/issues/1092)
- `attrs.resolve_types()` can now pass `include_extras` to `typing.get_type_hints()` on Python 3.9+, and does so by default.
  [#1099](https://github.com/python-attrs/attrs/issues/1099)
- Added instructions for pull request workflow to `CONTRIBUTING.md`.
  [#1105](https://github.com/python-attrs/attrs/issues/1105)
- Added *type* parameter to `attrs.field()` function for use with `attrs.make_class()`.

  Please note that type checkers ignore type metadata passed into `make_class()`, but it can be useful if you're wrapping _attrs_.
  [#1107](https://github.com/python-attrs/attrs/issues/1107)
- It is now possible for `attrs.evolve()` (and `attr.evolve()`) to change fields named `inst` if the instance is passed as a positional argument.

  Passing the instance using the `inst` keyword argument is now deprecated and will be removed in, or after, April 2024.
  [#1117](https://github.com/python-attrs/attrs/issues/1117)
- `attrs.validators.optional()` now also accepts a tuple of validators (in addition to lists of validators).
  [#1122](https://github.com/python-attrs/attrs/issues/1122)


## [22.2.0](https://github.com/python-attrs/attrs/tree/22.2.0) - 2022-12-21

### Backwards-incompatible Changes

- Python 3.5 is not supported anymore.
  [#988](https://github.com/python-attrs/attrs/issues/988)


### Deprecations

- Python 3.6 is now deprecated and support will be removed in the next release.
  [#1017](https://github.com/python-attrs/attrs/issues/1017)


### Changes

- `attrs.field()` now supports an *alias* option for explicit `__init__` argument names.

  Get `__init__` signatures matching any taste, peculiar or plain!
  The [PEP 681 compatible](https://peps.python.org/pep-0681/#field-specifier-parameters) *alias* option can be use to override private attribute name mangling, or add other arbitrary field argument name overrides.
  [#950](https://github.com/python-attrs/attrs/issues/950)
- `attrs.NOTHING` is now an enum value, making it possible to use with e.g. [`typing.Literal`](https://docs.python.org/3/library/typing.html#typing.Literal).
  [#983](https://github.com/python-attrs/attrs/issues/983)
- Added missing re-import of `attr.AttrsInstance` to the `attrs` namespace.
  [#987](https://github.com/python-attrs/attrs/issues/987)
- Fix slight performance regression in classes with custom `__setattr__` and speedup even more.
  [#991](https://github.com/python-attrs/attrs/issues/991)
- Class-creation performance improvements by switching performance-sensitive templating operations to f-strings.

  You can expect an improvement of about 5% -- even for very simple classes.
  [#995](https://github.com/python-attrs/attrs/issues/995)
- `attrs.has()` is now a [`TypeGuard`](https://docs.python.org/3/library/typing.html#typing.TypeGuard) for `AttrsInstance`.
  That means that type checkers know a class is an instance of an `attrs` class if you check it using `attrs.has()` (or `attr.has()`) first.
  [#997](https://github.com/python-attrs/attrs/issues/997)
- Made `attrs.AttrsInstance` stub available at runtime and fixed type errors related to the usage of `attrs.AttrsInstance` in *Pyright*.
  [#999](https://github.com/python-attrs/attrs/issues/999)
- On Python 3.10 and later, call [`abc.update_abstractmethods()`](https://docs.python.org/3/library/abc.html#abc.update_abstractmethods) on dict classes after creation.
  This improves the detection of abstractness.
  [#1001](https://github.com/python-attrs/attrs/issues/1001)
- *attrs*'s pickling methods now use dicts instead of tuples.
  That is safer and more robust across different versions of a class.
  [#1009](https://github.com/python-attrs/attrs/issues/1009)
- Added `attrs.validators.not_(wrapped_validator)` to logically invert *wrapped_validator* by accepting only values where *wrapped_validator* rejects the value with a `ValueError` or `TypeError` (by default, exception types configurable).
  [#1010](https://github.com/python-attrs/attrs/issues/1010)
- The type stubs for `attrs.cmp_using()` now have default values.
  [#1027](https://github.com/python-attrs/attrs/issues/1027)
- To conform with [PEP 681](https://peps.python.org/pep-0681/), `attr.s()` and `attrs.define()` now accept *unsafe_hash* in addition to *hash*.
  [#1065](https://github.com/python-attrs/attrs/issues/1065)


## [22.1.0](https://github.com/python-attrs/attrs/tree/22.1.0) - 2022-07-28

### Backwards-incompatible Changes

- Python 2.7 is not supported anymore.

  Dealing with Python 2.7 tooling has become too difficult for a volunteer-run project.

  We have supported Python 2 more than 2 years after it was officially discontinued and feel that we have paid our dues.
  All version up to 21.4.0 from December 2021 remain fully functional, of course.
  [#936](https://github.com/python-attrs/attrs/issues/936)

- The deprecated `cmp` attribute of `attrs.Attribute` has been removed.
  This does not affect the *cmp* argument to `attr.s` that can be used as a shortcut to set *eq* and *order* at the same time.
  [#939](https://github.com/python-attrs/attrs/issues/939)


### Changes

- Instantiation of frozen slotted classes is now faster.
  [#898](https://github.com/python-attrs/attrs/issues/898)
- If an `eq` key is defined, it is also used before hashing the attribute.
  [#909](https://github.com/python-attrs/attrs/issues/909)
- Added `attrs.validators.min_len()`.
  [#916](https://github.com/python-attrs/attrs/issues/916)
- `attrs.validators.deep_iterable()`'s *member_validator* argument now also accepts a list of validators and wraps them in an `attrs.validators.and_()`.
  [#925](https://github.com/python-attrs/attrs/issues/925)
- Added missing type stub re-imports for `attrs.converters` and `attrs.filters`.
  [#931](https://github.com/python-attrs/attrs/issues/931)
- Added missing stub for `attr(s).cmp_using()`.
  [#949](https://github.com/python-attrs/attrs/issues/949)
- `attrs.validators._in()`'s `ValueError` is not missing the attribute, expected options, and the value it got anymore.
  [#951](https://github.com/python-attrs/attrs/issues/951)
- Python 3.11 is now officially supported.
  [#969](https://github.com/python-attrs/attrs/issues/969)


## [21.4.0](https://github.com/python-attrs/attrs/tree/21.4.0) - 2021-12-29

### Changes

- Fixed the test suite on PyPy3.8 where `cloudpickle` does not work.
  [#892](https://github.com/python-attrs/attrs/issues/892)
- Fixed `coverage report` for projects that use `attrs` and don't set a `--source`.
  [#895](https://github.com/python-attrs/attrs/issues/895),
  [#896](https://github.com/python-attrs/attrs/issues/896)


## [21.3.0](https://github.com/python-attrs/attrs/tree/21.3.0) - 2021-12-28

### Backward-incompatible Changes

- When using `@define`, converters are now run by default when setting an attribute on an instance -- additionally to validators.
  I.e. the new default is `on_setattr=[attrs.setters.convert, attrs.setters.validate]`.

  This is unfortunately a breaking change, but it was an oversight, impossible to raise a `DeprecationWarning` about, and it's better to fix it now while the APIs are very fresh with few users.
  [#835](https://github.com/python-attrs/attrs/issues/835),
  [#886](https://github.com/python-attrs/attrs/issues/886)

- `import attrs` has finally landed!
  As of this release, you can finally import `attrs` using its proper name.

  Not all names from the `attr` namespace have been transferred; most notably `attr.s` and `attr.ib` are missing.
  See `attrs.define` and `attrs.field` if you haven't seen our next-generation APIs yet.
  A more elaborate explanation can be found [On The Core API Names](https://www.attrs.org/en/latest/names.html)

  This feature is at least for one release **provisional**.
  We don't *plan* on changing anything, but such a big change is unlikely to go perfectly on the first strike.

  The API docs have been mostly updated, but it will be an ongoing effort to change everything to the new APIs.
  Please note that we have **not** moved -- or even removed -- anything from `attr`!

  Please do report any bugs or documentation inconsistencies!
  [#887](https://github.com/python-attrs/attrs/issues/887)


### Changes

- `attr.asdict(retain_collection_types=False)` (default) dumps collection-esque keys as tuples.
  [#646](https://github.com/python-attrs/attrs/issues/646),
  [#888](https://github.com/python-attrs/attrs/issues/888)
- `__match_args__` are now generated to support Python 3.10's
  [Structural Pattern Matching](https://docs.python.org/3.10/whatsnew/3.10.html#pep-634-structural-pattern-matching).
  This can be controlled by the `match_args` argument to the class decorators on Python 3.10 and later.
  On older versions, it is never added and the argument is ignored.
  [#815](https://github.com/python-attrs/attrs/issues/815)
- If the class-level *on_setattr* is set to `attrs.setters.validate` (default in `@define` and `@mutable`) but no field defines a validator, pretend that it's not set.
  [#817](https://github.com/python-attrs/attrs/issues/817)
- The generated `__repr__` is significantly faster on Pythons with f-strings.
  [#819](https://github.com/python-attrs/attrs/issues/819)
- Attributes transformed via `field_transformer` are wrapped with `AttrsClass` again.
  [#824](https://github.com/python-attrs/attrs/issues/824)
- Generated source code is now cached more efficiently for identical classes.
  [#828](https://github.com/python-attrs/attrs/issues/828)
- Added `attrs.converters.to_bool()`.
  [#830](https://github.com/python-attrs/attrs/issues/830)
- `attrs.resolve_types()` now resolves types of subclasses after the parents are resolved.
  [#842](https://github.com/python-attrs/attrs/issues/842)
  [#843](https://github.com/python-attrs/attrs/issues/843)
- Added new validators: `lt(val)` (\< val), `le(va)` (≤ val), `ge(val)` (≥ val), `gt(val)` (> val), and `maxlen(n)`.
  [#845](https://github.com/python-attrs/attrs/issues/845)
- `attrs` classes are now fully compatible with [cloudpickle](https://github.com/cloudpipe/cloudpickle) (no need to disable `repr` anymore).
  [#857](https://github.com/python-attrs/attrs/issues/857)
- Added new context manager `attrs.validators.disabled()` and functions `attrs.validators.(set|get)_disabled()`.
  They deprecate `attrs.(set|get)_run_validators()`.
  All functions are interoperable and modify the same internal state.
  They are not – and never were – thread-safe, though.
  [#859](https://github.com/python-attrs/attrs/issues/859)
- `attrs.validators.matches_re()` now accepts pre-compiled regular expressions in addition to pattern strings.
  [#877](https://github.com/python-attrs/attrs/issues/877)

---

## [21.2.0](https://github.com/python-attrs/attrs/tree/21.2.0) - 2021-05-07

### Backward-incompatible Changes

- We had to revert the recursive feature for `attr.evolve()` because it broke some use-cases -- sorry!
  [#806](https://github.com/python-attrs/attrs/issues/806)
- Python 3.4 is now blocked using packaging metadata because `attrs` can't be imported on it anymore.
  To ensure that 3.4 users can keep installing  `attrs` easily, we will [yank](https://pypi.org/help/#yanked) 21.1.0 from PyPI.
  This has **no** consequences if you pin `attrs` to 21.1.0.
  [#807](https://github.com/python-attrs/attrs/issues/807)


## [21.1.0](https://github.com/python-attrs/attrs/tree/21.1.0) - 2021-05-06

### Deprecations

- The long-awaited, much-talked-about, little-delivered `import attrs` is finally upon us!

  Since the NG APIs have now been proclaimed stable, the **next** release of `attrs` will allow you to actually `import attrs`.
  We're taking this opportunity to replace some defaults in our APIs that made sense in 2015, but don't in 2021.

  So please, if you have any pet peeves about defaults in `attrs`'s APIs, *now* is the time to air your grievances in #487!
  We're not gonna get such a chance for a second time, without breaking our backward-compatibility guarantees, or long deprecation cycles.
  Therefore, speak now or forever hold you peace!
  [#487](https://github.com/python-attrs/attrs/issues/487)

- The *cmp* argument to `attr.s()` and `attr.ib()` has been **undeprecated**
  It will continue to be supported as syntactic sugar to set *eq* and *order* in one go.

  I'm terribly sorry for the hassle around this argument!
  The reason we're bringing it back is it's usefulness regarding customization of equality/ordering.

  The `cmp` attribute and argument on `attr.Attribute` remains deprecated and will be removed later this year.
  [#773](https://github.com/python-attrs/attrs/issues/773)


### Changes

- It's now possible to customize the behavior of `eq` and `order` by passing in a callable.
  [#435](https://github.com/python-attrs/attrs/issues/435),
  [#627](https://github.com/python-attrs/attrs/issues/627)

- The instant favorite next-generation APIs are not provisional anymore!

  They are also officially supported by Mypy as of their [0.800 release](https://mypy-lang.blogspot.com/2021/01/mypy-0800-released.html).

  We hope the next release will already contain an (additional) importable package called `attrs`.
  [#668](https://github.com/python-attrs/attrs/issues/668),
  [#786](https://github.com/python-attrs/attrs/issues/786)

- If an attribute defines a converter, the type of its parameter is used as type annotation for its corresponding `__init__` parameter.

  If an `attr.converters.pipe` is used, the first one's is used.
  [#710](https://github.com/python-attrs/attrs/issues/710)

- Fixed the creation of an extra slot for an `attr.ib` when the parent class already has a slot with the same name.
  [#718](https://github.com/python-attrs/attrs/issues/718)

- `__attrs__init__()` will now be injected if `init=False`, or if `auto_detect=True` and a user-defined `__init__()` exists.

  This enables users to do "pre-init" work in their `__init__()` (such as `super().__init__()`).

  `__init__()` can then delegate constructor argument processing to `self.__attrs_init__(*args, **kwargs)`.
  [#731](https://github.com/python-attrs/attrs/issues/731)

- `bool(attr.NOTHING)` is now `False`.
  [#732](https://github.com/python-attrs/attrs/issues/732)

- It's now possible to use `super()` inside of properties of slotted classes.
  [#747](https://github.com/python-attrs/attrs/issues/747)

- Allow for a `__attrs_pre_init__()` method that -- if defined -- will get called at the beginning of the `attrs`-generated `__init__()` method.
  [#750](https://github.com/python-attrs/attrs/issues/750)

- Added forgotten `attr.Attribute.evolve()` to type stubs.
  [#752](https://github.com/python-attrs/attrs/issues/752)

- `attrs.evolve()` now works recursively with nested `attrs` classes.
  [#759](https://github.com/python-attrs/attrs/issues/759)

- Python 3.10 is now officially supported.
  [#763](https://github.com/python-attrs/attrs/issues/763)

- `attr.resolve_types()` now takes an optional *attrib* argument to work inside a `field_transformer`.
  [#774](https://github.com/python-attrs/attrs/issues/774)

- `ClassVar`s are now also detected if they come from [typing-extensions](https://pypi.org/project/typing-extensions/).
  [#782](https://github.com/python-attrs/attrs/issues/782)

- To make it easier to customize attribute comparison (#435), we have added the `attr.cmp_with()` helper.

  See the [new docs on comparison](https://www.attrs.org/en/stable/comparison.html) for more details.
  [#787](https://github.com/python-attrs/attrs/issues/787)

- Added **provisional** support for static typing in `pyright` via [PEP 681](https://peps.python.org/pep-0681/).
  Both the `pyright` specification and `attrs` implementation may change in future versions of both projects.

  Your constructive feedback is welcome in both [attrs#795](https://github.com/python-attrs/attrs/issues/795) and [pyright#1782](https://github.com/microsoft/pyright/discussions/1782).
  [#796](https://github.com/python-attrs/attrs/issues/796)


## [20.3.0](https://github.com/python-attrs/attrs/tree/20.3.0) - 2020-11-05

### Backward-incompatible Changes

- `attr.define()`, `attr.frozen()`, `attr.mutable()`, and `attr.field()` remain **provisional**.

  This release does **not** change anything about them and they are already used widely in production though.

  If you wish to use them together with mypy, you can simply drop [this plugin](https://gist.github.com/hynek/1e3844d0c99e479e716169034b5fa963#file-attrs_ng_plugin-py) into your project.

  Feel free to provide feedback to them in the linked issue #668.

  We will release the `attrs` namespace once we have the feeling that the APIs have properly settled.
  [#668](https://github.com/python-attrs/attrs/issues/668)

### Changes

- `attr.s()` now has a *field_transformer* hook that is called for all `Attribute`s and returns a (modified or updated) list of `Attribute` instances.
  `attr.asdict()` has a *value_serializer* hook that can change the way values are converted.
  Both hooks are meant to help with data (de-)serialization workflows.
  [#653](https://github.com/python-attrs/attrs/issues/653)
- `kw_only=True` now works on Python 2.
  [#700](https://github.com/python-attrs/attrs/issues/700)
- `raise from` now works on frozen classes on PyPy.
  [#703](https://github.com/python-attrs/attrs/issues/703),
  [#712](https://github.com/python-attrs/attrs/issues/712)
- `attr.asdict()` and `attr.astuple()` now treat `frozenset`s like `set`s with regards to the *retain_collection_types* argument.
  [#704](https://github.com/python-attrs/attrs/issues/704)
- The type stubs for `attr.s()` and `attr.make_class()` are not missing the *collect_by_mro* argument anymore.
  [#711](https://github.com/python-attrs/attrs/issues/711)

---

## [20.2.0](https://github.com/python-attrs/attrs/tree/20.2.0) - 2020-09-05

### Backward-incompatible Changes

- `attr.define()`, `attr.frozen()`, `attr.mutable()`, and `attr.field()` remain **provisional**.

  This release fixes a bunch of bugs and ergonomics but they remain mostly unchanged.

  If you wish to use them together with mypy, you can simply drop [this plugin](https://gist.github.com/hynek/1e3844d0c99e479e716169034b5fa963#file-attrs_ng_plugin-py) into your project.

  Feel free to provide feedback to them in the linked issue #668.

  We will release the `attrs` namespace once we have the feeling that the APIs have properly settled.
  [#668](https://github.com/python-attrs/attrs/issues/668)

### Changes

- `attr.define()` et al now correctly detect `__eq__` and `__ne__`.
  [#671](https://github.com/python-attrs/attrs/issues/671)

- `attr.define()` et al's hybrid behavior now also works correctly when arguments are passed.
  [#675](https://github.com/python-attrs/attrs/issues/675)

- It's possible to define custom `__setattr__` methods on slotted classes again.
  [#681](https://github.com/python-attrs/attrs/issues/681)

- In 20.1.0 we introduced the `inherited` attribute on the `attr.Attribute` class to differentiate attributes that have been inherited and those that have been defined directly on the class.

  It has shown to be problematic to involve that attribute when comparing instances of `attr.Attribute` though, because when sub-classing, attributes from base classes are suddenly not equal to themselves in a super class.

  Therefore the `inherited` attribute will now be ignored when hashing and comparing instances of `attr.Attribute`.
  [#684](https://github.com/python-attrs/attrs/issues/684)

- `zope.interface` is now a "soft dependency" when running the test suite; if `zope.interface` is not installed when running the test suite, the interface-related tests will be automatically skipped.
  [#685](https://github.com/python-attrs/attrs/issues/685)

- The ergonomics of creating frozen classes using `@define(frozen=True)` and sub-classing frozen classes has been improved:
  you don't have to set `on_setattr=None` anymore.
  [#687](https://github.com/python-attrs/attrs/issues/687)

---

## [20.1.0](https://github.com/python-attrs/attrs/tree/20.1.0) - 2020-08-20

### Backward-incompatible Changes

- Python 3.4 is not supported anymore.
  It has been unsupported by the Python core team for a while now, its PyPI downloads are negligible, and our CI provider removed it as a supported option.

  It's very unlikely that `attrs` will break under 3.4 anytime soon, which is why we do *not* block its installation on Python 3.4.
  But we don't test it anymore and will block it once someone reports breakage.
  [#608](https://github.com/python-attrs/attrs/issues/608)

### Deprecations

- Less of a deprecation and more of a heads up: the next release of `attrs` will introduce an `attrs` namespace.
  That means that you'll finally be able to run `import attrs` with new functions that aren't cute abbreviations and that will carry better defaults.

  This should not break any of your code, because project-local packages have priority before installed ones.
  If this is a problem for you for some reason, please report it to our bug tracker and we'll figure something out.

  The old `attr` namespace isn't going anywhere and its defaults are not changing – this is a purely additive measure.
  Please check out the linked issue for more details.

  These new APIs have been added *provisionally* as part of #666 so you can try them out today and provide feedback.
  Learn more in the [API docs](https://www.attrs.org/en/stable/api.html).
  [#408](https://github.com/python-attrs/attrs/issues/408)

### Changes

- Added `attr.resolve_types()`.
  It ensures that all forward-references and types in string form are resolved into concrete types.

  You need this only if you need concrete types at runtime.
  That means that if you only use types for static type checking, you do **not** need this function.
  [#288](https://github.com/python-attrs/attrs/issues/288),
  [#302](https://github.com/python-attrs/attrs/issues/302)

- Added `@attr.s(collect_by_mro=False)` argument that if set to `True` fixes the collection of attributes from base classes.

  It's only necessary for certain cases of multiple-inheritance but is kept off for now for backward-compatibility reasons.
  It will be turned on by default in the future.

  As a side-effect, `attr.Attribute` now *always* has an `inherited` attribute indicating whether an attribute on a class was directly defined or inherited.
  [#428](https://github.com/python-attrs/attrs/issues/428),
  [#635](https://github.com/python-attrs/attrs/issues/635)

- On Python 3, all generated methods now have a docstring explaining that they have been created by `attrs`.
  [#506](https://github.com/python-attrs/attrs/issues/506)

- It is now possible to prevent `attrs` from auto-generating the `__setstate__` and `__getstate__` methods that are required for pickling of slotted classes.

  Either pass `@attr.s(getstate_setstate=False)` or pass `@attr.s(auto_detect=True)` and implement them yourself:
  if `attrs` finds either of the two methods directly on the decorated class, it assumes implicitly `getstate_setstate=False` (and implements neither).

  This option works with dict classes but should never be necessary.
  [#512](https://github.com/python-attrs/attrs/issues/512),
  [#513](https://github.com/python-attrs/attrs/issues/513),
  [#642](https://github.com/python-attrs/attrs/issues/642)

- Fixed a `ValueError: Cell is empty` bug that could happen in some rare edge cases.
  [#590](https://github.com/python-attrs/attrs/issues/590)

- `attrs` can now automatically detect your own implementations and infer `init=False`, `repr=False`, `eq=False`, `order=False`, and `hash=False` if you set `@attr.s(auto_detect=True)`.
  `attrs` will ignore inherited methods.
  If the argument implies more than one method (e.g. `eq=True` creates both `__eq__` and `__ne__`), it's enough for *one* of them to exist and `attrs` will create *neither*.

  This feature requires Python 3.
  [#607](https://github.com/python-attrs/attrs/issues/607)

- Added `attr.converters.pipe()`.
  The feature allows combining multiple conversion callbacks into one by piping the value through all of them, and retuning the last result.

  As part of this feature, we had to relax the type information for converter callables.
  [#618](https://github.com/python-attrs/attrs/issues/618)

- Fixed serialization behavior of non-slots classes with `cache_hash=True`.
  The hash cache will be cleared on operations which make "deep copies" of instances of classes with hash caching,
  though the cache will not be cleared with shallow copies like those made by `copy.copy()`.

  Previously, `copy.deepcopy()` or serialization and deserialization with `pickle` would result in an un-initialized object.

  This change also allows the creation of `cache_hash=True` classes with a custom `__setstate__`,
  which was previously forbidden ([#494](https://github.com/python-attrs/attrs/issues/494)).
  [#620](https://github.com/python-attrs/attrs/issues/620)

- It is now possible to specify hooks that are called whenever an attribute is set **after** a class has been instantiated.

  You can pass `on_setattr` both to `@attr.s()` to set the default for all attributes on a class, and to `@attr.ib()` to overwrite it for individual attributes.

  `attrs` also comes with a new module `attr.setters` that brings helpers that run validators, converters, or allow to freeze a subset of attributes.
  [#645](https://github.com/python-attrs/attrs/issues/645),
  [#660](https://github.com/python-attrs/attrs/issues/660)

- **Provisional** APIs called `attr.define()`, `attr.mutable()`, and `attr.frozen()` have been added.

  They are only available on Python 3.6 and later, and call `attr.s()` with different default values.

  If nothing comes up, they will become the official way for creating classes in 20.2.0 (see above).

  **Please note** that it may take some time until mypy – and other tools that have dedicated support for `attrs` – recognize these new APIs.
  Please **do not** open issues on our bug tracker, there is nothing we can do about it.
  [#666](https://github.com/python-attrs/attrs/issues/666)

- We have also provisionally added `attr.field()` that supplants `attr.ib()`.
  It also requires at least Python 3.6 and is keyword-only.
  Other than that, it only dropped a few arguments, but changed no defaults.

  As with `attr.s()`: `attr.ib()` is not going anywhere.
  [#669](https://github.com/python-attrs/attrs/issues/669)

---

## [19.3.0](https://github.com/python-attrs/attrs/tree/19.3.0) - 2019-10-15

### Changes

- Fixed `auto_attribs` usage when default values cannot be compared directly with `==`, such as `numpy` arrays.
  [#585](https://github.com/python-attrs/attrs/issues/585)

---

## [19.2.0](https://github.com/python-attrs/attrs/tree/19.2.0) - 2019-10-01

### Backward-incompatible Changes

- Removed deprecated `Attribute` attribute `convert` per scheduled removal on 2019/1.
  This planned deprecation is tracked in issue [#307](https://github.com/python-attrs/attrs/issues/307).
  [#504](https://github.com/python-attrs/attrs/issues/504)

- `__lt__`, `__le__`, `__gt__`, and `__ge__` do not consider subclasses comparable anymore.

  This has been deprecated since 18.2.0 and was raising a `DeprecationWarning` for over a year.
  [#570](https://github.com/python-attrs/attrs/issues/570)

### Deprecations

- The `cmp` argument to `attr.s()` and `attr.ib()` is now deprecated.

  Please use `eq` to add equality methods (`__eq__` and `__ne__`) and `order` to add ordering methods (`__lt__`, `__le__`, `__gt__`, and `__ge__`) instead – just like with [dataclasses](https://docs.python.org/3/library/dataclasses.html).

  Both are effectively `True` by default but it's enough to set `eq=False` to disable both at once.
  Passing `eq=False, order=True` explicitly will raise a `ValueError` though.

  Since this is arguably a deeper backward-compatibility break, it will have an extended deprecation period until 2021-06-01.
  After that day, the `cmp` argument will be removed.

  `attr.Attribute` also isn't orderable anymore.
  [#574](https://github.com/python-attrs/attrs/issues/574)

### Changes

- Updated `attr.validators.__all__` to include new validators added in [#425].
  [#517](https://github.com/python-attrs/attrs/issues/517)
- Slotted classes now use a pure Python mechanism to rewrite the `__class__` cell when rebuilding the class, so `super()` works even on environments where `ctypes` is not installed.
  [#522](https://github.com/python-attrs/attrs/issues/522)
- When collecting attributes using `@attr.s(auto_attribs=True)`, attributes with a default of `None` are now deleted too.
  [#523](https://github.com/python-attrs/attrs/issues/523),
  [#556](https://github.com/python-attrs/attrs/issues/556)
- Fixed `attr.validators.deep_iterable()` and `attr.validators.deep_mapping()` type stubs.
  [#533](https://github.com/python-attrs/attrs/issues/533)
- `attr.validators.is_callable()` validator now raises an exception `attr.exceptions.NotCallableError`, a subclass of `TypeError`, informing the received value.
  [#536](https://github.com/python-attrs/attrs/issues/536)
- `@attr.s(auto_exc=True)` now generates classes that are hashable by ID, as the documentation always claimed it would.
  [#543](https://github.com/python-attrs/attrs/issues/543),
  [#563](https://github.com/python-attrs/attrs/issues/563)
- Added `attr.validators.matches_re()` that checks string attributes whether they match a regular expression.
  [#552](https://github.com/python-attrs/attrs/issues/552)
- Keyword-only attributes (`kw_only=True`) and attributes that are excluded from the `attrs`'s `__init__` (`init=False`) now can appear before mandatory attributes.
  [#559](https://github.com/python-attrs/attrs/issues/559)
- The fake filename for generated methods is now more stable.
  It won't change when you restart the process.
  [#560](https://github.com/python-attrs/attrs/issues/560)
- The value passed to `@attr.ib(repr=…)` can now be either a boolean (as before) or a callable.
  That callable must return a string and is then used for formatting the attribute by the generated `__repr__()` method.
  [#568](https://github.com/python-attrs/attrs/issues/568)
- Added `attr.__version_info__` that can be used to reliably check the version of `attrs` and write forward- and backward-compatible code.
  Please check out the [section on deprecated APIs](https://www.attrs.org/en/stable/api-attr.html#deprecated-apis) on how to use it.
  [#580](https://github.com/python-attrs/attrs/issues/580)

>

---

## [19.1.0](https://github.com/python-attrs/attrs/tree/19.1.0) - 2019-03-03

### Backward-incompatible Changes

- Fixed a bug where deserialized objects with `cache_hash=True` could have incorrect hash code values.
  This change breaks classes with `cache_hash=True` when a custom `__setstate__` is present.
  An exception will be thrown when applying the `attrs` annotation to such a class.
  This limitation is tracked in issue [#494](https://github.com/python-attrs/attrs/issues/494).
  [#482](https://github.com/python-attrs/attrs/issues/482)

### Changes

- Add `is_callable`, `deep_iterable`, and `deep_mapping` validators.

  - `is_callable`: validates that a value is callable
  - `deep_iterable`: Allows recursion down into an iterable,
    applying another validator to every member in the iterable
    as well as applying an optional validator to the iterable itself.
  - `deep_mapping`: Allows recursion down into the items in a mapping object,
    applying a key validator and a value validator to the key and value in every item.
    Also applies an optional validator to the mapping object itself.

  You can find them in the `attr.validators` package.
  [#425]

- Fixed stub files to prevent errors raised by mypy's `disallow_any_generics = True` option.
  [#443](https://github.com/python-attrs/attrs/issues/443)

- Attributes with `init=False` now can follow after `kw_only=True` attributes.
  [#450](https://github.com/python-attrs/attrs/issues/450)

- `attrs` now has first class support for defining exception classes.

  If you define a class using `@attr.s(auto_exc=True)` and subclass an exception, the class will behave like a well-behaved exception class including an appropriate `__str__` method, and all attributes additionally available in an `args` attribute.
  [#500](https://github.com/python-attrs/attrs/issues/500)

- Clarified documentation for hashing to warn that hashable objects should be deeply immutable (in their usage, even if this is not enforced).
  [#503](https://github.com/python-attrs/attrs/issues/503)

---

## [18.2.0](https://github.com/python-attrs/attrs/tree/18.2.0) - 2018-09-01

### Deprecations

- Comparing subclasses using `<`, `>`, `<=`, and `>=` is now deprecated.
  The docs always claimed that instances are only compared if the types are identical, so this is a first step to conform to the docs.

  Equality operators (`==` and `!=`) were always strict in this regard.
  [#394](https://github.com/python-attrs/attrs/issues/394)

### Changes

- `attrs` now ships its own [PEP 484](https://peps.python.org/pep-0484/) type hints.
  Together with [mypy](http://mypy-lang.org)'s `attrs` plugin, you've got all you need for writing statically typed code in both Python 2 and 3!

  At that occasion, we've also added [narrative docs](https://www.attrs.org/en/stable/types.html) about type annotations in `attrs`.
  [#238](https://github.com/python-attrs/attrs/issues/238)

- Added *kw_only* arguments to `attr.ib` and `attr.s`, and a corresponding *kw_only* attribute to `attr.Attribute`.
  This change makes it possible to have a generated `__init__` with keyword-only arguments on Python 3, relaxing the required ordering of default and non-default valued attributes.
  [#281](https://github.com/python-attrs/attrs/issues/281),
  [#411](https://github.com/python-attrs/attrs/issues/411)

- The test suite now runs with `hypothesis.HealthCheck.too_slow` disabled to prevent CI breakage on slower computers.
  [#364](https://github.com/python-attrs/attrs/issues/364),
  [#396](https://github.com/python-attrs/attrs/issues/396)

- `attr.validators.in_()` now raises a `ValueError` with a useful message even if the options are a string and the value is not a string.
  [#383](https://github.com/python-attrs/attrs/issues/383)

- `attr.asdict()` now properly handles deeply nested lists and dictionaries.
  [#395](https://github.com/python-attrs/attrs/issues/395)

- Added `attr.converters.default_if_none()` that allows to replace `None` values in attributes.
  For example `attr.ib(converter=default_if_none(""))` replaces `None` by empty strings.
  [#400](https://github.com/python-attrs/attrs/issues/400),
  [#414](https://github.com/python-attrs/attrs/issues/414)

- Fixed a reference leak where the original class would remain live after being replaced when `slots=True` is set.
  [#407](https://github.com/python-attrs/attrs/issues/407)

- Slotted classes can now be made weakly referenceable by passing `@attr.s(weakref_slot=True)`.
  [#420](https://github.com/python-attrs/attrs/issues/420)

- Added *cache_hash* option to `@attr.s` which causes the hash code to be computed once and stored on the object.
  [#426](https://github.com/python-attrs/attrs/issues/426)

- Attributes can be named `property` and `itemgetter` now.
  [#430](https://github.com/python-attrs/attrs/issues/430)

- It is now possible to override a base class' class variable using only class annotations.
  [#431](https://github.com/python-attrs/attrs/issues/431)

---

## [18.1.0](https://github.com/python-attrs/attrs/tree/18.1.0) - 2018-05-03

### Changes

- `x=X(); x.cycle = x; repr(x)` will no longer raise a `RecursionError`, and will instead show as `X(x=...)`.

  [#95](https://github.com/python-attrs/attrs/issues/95)

- `attr.ib(factory=f)` is now syntactic sugar for the common case of `attr.ib(default=attr.Factory(f))`.

  [#178](https://github.com/python-attrs/attrs/issues/178),
  [#356](https://github.com/python-attrs/attrs/issues/356)

- Added `attr.field_dict()` to return an ordered dictionary of `attrs` attributes for a class, whose keys are the attribute names.

  [#290](https://github.com/python-attrs/attrs/issues/290),
  [#349](https://github.com/python-attrs/attrs/issues/349)

- The order of attributes that are passed into `attr.make_class()` or the *these* argument of `@attr.s()` is now retained if the dictionary is ordered (i.e. `dict` on Python 3.6 and later, `collections.OrderedDict` otherwise).

  Before, the order was always determined by the order in which the attributes have been defined which may not be desirable when creating classes programmatically.

  [#300](https://github.com/python-attrs/attrs/issues/300),
  [#339](https://github.com/python-attrs/attrs/issues/339),
  [#343](https://github.com/python-attrs/attrs/issues/343)

- In slotted classes, `__getstate__` and `__setstate__` now ignore the `__weakref__` attribute.

  [#311](https://github.com/python-attrs/attrs/issues/311),
  [#326](https://github.com/python-attrs/attrs/issues/326)

- Setting the cell type is now completely best effort.
  This fixes `attrs` on Jython.

  We cannot make any guarantees regarding Jython though, because our test suite cannot run due to dependency incompatibilities.

  [#321](https://github.com/python-attrs/attrs/issues/321),
  [#334](https://github.com/python-attrs/attrs/issues/334)

- If `attr.s` is passed a *these* argument, it will no longer attempt to remove attributes with the same name from the class body.

  [#322](https://github.com/python-attrs/attrs/issues/322),
  [#323](https://github.com/python-attrs/attrs/issues/323)

- The hash of `attr.NOTHING` is now vegan and faster on 32bit Python builds.

  [#331](https://github.com/python-attrs/attrs/issues/331),
  [#332](https://github.com/python-attrs/attrs/issues/332)

- The overhead of instantiating frozen dict classes is virtually eliminated.
  [#336](https://github.com/python-attrs/attrs/issues/336)

- Generated `__init__` methods now have an `__annotations__` attribute derived from the types of the fields.

  [#363](https://github.com/python-attrs/attrs/issues/363)

- We have restructured the documentation a bit to account for `attrs`' growth in scope.
  Instead of putting everything into the [examples](https://www.attrs.org/en/stable/examples.html) page, we have started to extract narrative chapters.

  So far, we've added chapters on [initialization](https://www.attrs.org/en/stable/init.html) and [hashing](https://www.attrs.org/en/stable/hashing.html).

  Expect more to come!

  [#369](https://github.com/python-attrs/attrs/issues/369),
  [#370](https://github.com/python-attrs/attrs/issues/370)

---

## [17.4.0](https://github.com/python-attrs/attrs/tree/17.4.0) - 2017-12-30

### Backward-incompatible Changes

- The traversal of MROs when using multiple inheritance was backward:
  If you defined a class `C` that subclasses `A` and `B` like `C(A, B)`, `attrs` would have collected the attributes from `B` *before* those of `A`.

  This is now fixed and means that in classes that employ multiple inheritance, the output of `__repr__` and the order of positional arguments in `__init__` changes.
  Because of the nature of this bug, a proper deprecation cycle was unfortunately impossible.

  Generally speaking, it's advisable to prefer `kwargs`-based initialization anyways – *especially* if you employ multiple inheritance and diamond-shaped hierarchies.

  [#298](https://github.com/python-attrs/attrs/issues/298),
  [#299](https://github.com/python-attrs/attrs/issues/299),
  [#304](https://github.com/python-attrs/attrs/issues/304)

- The `__repr__` set by `attrs` no longer produces an `AttributeError` when the instance is missing some of the specified attributes (either through deleting or after using `init=False` on some attributes).

  This can break code that relied on `repr(attr_cls_instance)` raising `AttributeError` to check if any `attrs`-specified members were unset.

  If you were using this, you can implement a custom method for checking this:

  ```
  def has_unset_members(self):
      for field in attr.fields(type(self)):
          try:
              getattr(self, field.name)
          except AttributeError:
              return True
      return False
  ```

  [#308](https://github.com/python-attrs/attrs/issues/308)

### Deprecations

- The `attr.ib(convert=callable)` option is now deprecated in favor of `attr.ib(converter=callable)`.

  This is done to achieve consistency with other noun-based arguments like *validator*.

  *convert* will keep working until at least January 2019 while raising a `DeprecationWarning`.

  [#307](https://github.com/python-attrs/attrs/issues/307)

### Changes

- Generated `__hash__` methods now hash the class type along with the attribute values.
  Until now the hashes of two classes with the same values were identical which was a bug.

  The generated method is also *much* faster now.

  [#261](https://github.com/python-attrs/attrs/issues/261),
  [#295](https://github.com/python-attrs/attrs/issues/295),
  [#296](https://github.com/python-attrs/attrs/issues/296)

- `attr.ib`’s *metadata* argument now defaults to a unique empty `dict` instance instead of sharing a common empty `dict` for all.
  The singleton empty `dict` is still enforced.

  [#280](https://github.com/python-attrs/attrs/issues/280)

- `ctypes` is optional now however if it's missing, a bare `super()` will not work in slotted classes.
  This should only happen in special environments like Google App Engine.

  [#284](https://github.com/python-attrs/attrs/issues/284),
  [#286](https://github.com/python-attrs/attrs/issues/286)

- The attribute redefinition feature introduced in 17.3.0 now takes into account if an attribute is redefined via multiple inheritance.
  In that case, the definition that is closer to the base of the class hierarchy wins.

  [#285](https://github.com/python-attrs/attrs/issues/285),
  [#287](https://github.com/python-attrs/attrs/issues/287)

- Subclasses of `auto_attribs=True` can be empty now.

  [#291](https://github.com/python-attrs/attrs/issues/291),
  [#292](https://github.com/python-attrs/attrs/issues/292)

- Equality tests are *much* faster now.

  [#306](https://github.com/python-attrs/attrs/issues/306)

- All generated methods now have correct `__module__`, `__name__`, and (on Python 3) `__qualname__` attributes.

  [#309](https://github.com/python-attrs/attrs/issues/309)

---

## [17.3.0](https://github.com/python-attrs/attrs/tree/17.3.0) - 2017-11-08

### Backward-incompatible Changes

- Attributes are no longer defined on the class body.

  This means that if you define a class `C` with an attribute `x`, the class will *not* have an attribute `x` for introspection.
  Instead of `C.x`, use `attr.fields(C).x` or look at `C.__attrs_attrs__`.
  The old behavior has been deprecated since version 16.1.
  ([#253](https://github.com/python-attrs/attrs/issues/253))

### Changes

- `super()` and `__class__` now work with slotted classes on Python 3.
  ([#102](https://github.com/python-attrs/attrs/issues/102), [#226](https://github.com/python-attrs/attrs/issues/226), [#269](https://github.com/python-attrs/attrs/issues/269), [#270](https://github.com/python-attrs/attrs/issues/270), [#272](https://github.com/python-attrs/attrs/issues/272))

- Added *type* argument to `attr.ib()` and corresponding `type` attribute to `attr.Attribute`.

  This change paves the way for automatic type checking and serialization (though as of this release `attrs` does not make use of it).
  In Python 3.6 or higher, the value of `attr.Attribute.type` can alternately be set using variable type annotations
  (see [PEP 526](https://peps.python.org/pep-0526/)).
  ([#151](https://github.com/python-attrs/attrs/issues/151), [#214](https://github.com/python-attrs/attrs/issues/214), [#215](https://github.com/python-attrs/attrs/issues/215), [#239](https://github.com/python-attrs/attrs/issues/239))

- The combination of `str=True` and `slots=True` now works on Python 2.
  ([#198](https://github.com/python-attrs/attrs/issues/198))

- `attr.Factory` is hashable again.
  ([#204](https://github.com/python-attrs/attrs/issues/204))

- Subclasses now can overwrite attribute definitions of their base classes.

  That means that you can -- for example -- change the default value for an attribute by redefining it.
  ([#221](https://github.com/python-attrs/attrs/issues/221), [#229](https://github.com/python-attrs/attrs/issues/229))

- Added new option *auto_attribs* to `@attr.s` that allows to collect annotated fields without setting them to `attr.ib()`.

  Setting a field to an `attr.ib()` is still possible to supply options like validators.
  Setting it to any other value is treated like it was passed as `attr.ib(default=value)` -- passing an instance of `attr.Factory` also works as expected.
  ([#262](https://github.com/python-attrs/attrs/issues/262), [#277](https://github.com/python-attrs/attrs/issues/277))

- Instances of classes created using `attr.make_class()` can now be pickled.
  ([#282](https://github.com/python-attrs/attrs/issues/282))

---

## [17.2.0](https://github.com/python-attrs/attrs/tree/17.2.0) - 2017-05-24

### Changes:

- Validators are hashable again.
  Note that validators may become frozen in the future, pending availability of no-overhead frozen classes.
  [#192](https://github.com/python-attrs/attrs/issues/192)

---

## [17.1.0](https://github.com/python-attrs/attrs/tree/17.1.0) - 2017-05-16

To encourage more participation, the project has also been moved into a [dedicated GitHub organization](https://github.com/python-attrs/) and everyone is most welcome to join!

`attrs` also has a logo now!

```{image} https://www.attrs.org/en/latest/_static/attrs_logo.png
:alt: attrs logo
```

### Backward-incompatible Changes:

- `attrs` will set the `__hash__()` method to `None` by default now.
  The way hashes were handled before was in conflict with [Python's specification](https://docs.python.org/3/reference/datamodel.html#object.__hash__).
  This *may* break some software although this breakage is most likely just surfacing of latent bugs.
  You can always make `attrs` create the `__hash__()` method using `@attr.s(hash=True)`.
  See [#136] for the rationale of this change.

  :::{warning}
  Please *do not* upgrade blindly and *do* test your software!
  *Especially* if you use instances as dict keys or put them into sets!
  :::

- Correspondingly, `attr.ib`'s *hash* argument is `None` by default too and mirrors the *cmp* argument as it should.

### Deprecations:

- `attr.assoc()` is now deprecated in favor of `attr.evolve()` and will stop working in 2018.

### Changes:

- Fix default hashing behavior.
  Now *hash* mirrors the value of *cmp* and classes are unhashable by default.
  [#136]
  [#142](https://github.com/python-attrs/attrs/issues/142)
- Added `attr.evolve()` that, given an instance of an `attrs` class and field changes as keyword arguments, will instantiate a copy of the given instance with the changes applied.
  `evolve()` replaces `assoc()`, which is now deprecated.
  `evolve()` is significantly faster than `assoc()`, and requires the class have an initializer that can take the field values as keyword arguments (like `attrs` itself can generate).
  [#116](https://github.com/python-attrs/attrs/issues/116)
  [#124](https://github.com/python-attrs/attrs/pull/124)
  [#135](https://github.com/python-attrs/attrs/pull/135)
- `FrozenInstanceError` is now raised when trying to delete an attribute from a frozen class.
  [#118](https://github.com/python-attrs/attrs/pull/118)
- Frozen-ness of classes is now inherited.
  [#128](https://github.com/python-attrs/attrs/pull/128)
- `__attrs_post_init__()` is now run if validation is disabled.
  [#130](https://github.com/python-attrs/attrs/pull/130)
- Added `attr.validators.in_(options)` that, given the allowed `options`, checks whether the attribute value is in it.
  This can be used to check constants, enums, mappings, etc.
  [#181](https://github.com/python-attrs/attrs/pull/181)
- Added `attr.validators.and_()` that composes multiple validators into one.
  [#161](https://github.com/python-attrs/attrs/issues/161)
- For convenience, the *validator* argument of `@attr.s` now can take a list of validators that are wrapped using `and_()`.
  [#138](https://github.com/python-attrs/attrs/issues/138)
- Accordingly, `attr.validators.optional()` now can take a list of validators too.
  [#161](https://github.com/python-attrs/attrs/issues/161)
- Validators can now be defined conveniently inline by using the attribute as a decorator.
  Check out the [validator examples](https://www.attrs.org/en/stable/init.html#decorator) to see it in action!
  [#143](https://github.com/python-attrs/attrs/issues/143)
- `attr.Factory()` now has a *takes_self* argument that makes the initializer to pass the partially initialized instance into the factory.
  In other words you can define attribute defaults based on other attributes.
  [#165]
  [#189](https://github.com/python-attrs/attrs/issues/189)
- Default factories can now also be defined inline using decorators.
  They are *always* passed the partially initialized instance.
  [#165]
- Conversion can now be made optional using `attr.converters.optional()`.
  [#105](https://github.com/python-attrs/attrs/issues/105)
  [#173](https://github.com/python-attrs/attrs/pull/173)
- `attr.make_class()` now accepts the keyword argument `bases` which allows for subclassing.
  [#152](https://github.com/python-attrs/attrs/pull/152)
- Metaclasses are now preserved with `slots=True`.
  [#155](https://github.com/python-attrs/attrs/pull/155)

---

## [16.3.0](https://github.com/python-attrs/attrs/tree/16.3.0) - 2016-11-24

### Changes:

- Attributes now can have user-defined metadata which greatly improves `attrs`'s extensibility.
  [#96](https://github.com/python-attrs/attrs/pull/96)

- Allow for a `__attrs_post_init__()` method that -- if defined -- will get called at the end of the `attrs`-generated `__init__()` method.
  [#111](https://github.com/python-attrs/attrs/pull/111)

- Added `@attr.s(str=True)` that will optionally create a `__str__()` method that is identical to `__repr__()`.
  This is mainly useful with `Exception`s and other classes that rely on a useful `__str__()` implementation but overwrite the default one through a poor own one.
  Default Python class behavior is to use `__repr__()` as `__str__()` anyways.

  If you tried using `attrs` with `Exception`s and were puzzled by the tracebacks: this option is for you.

- `__name__` is no longer overwritten with `__qualname__` for `attr.s(slots=True)` classes.
  [#99](https://github.com/python-attrs/attrs/issues/99)

---

## [16.2.0](https://github.com/python-attrs/attrs/tree/16.2.0) - 2016-09-17

### Changes:

- Added `attr.astuple()` that -- similarly to `attr.asdict()` -- returns the instance as a tuple.
  [#77](https://github.com/python-attrs/attrs/issues/77)
- Converters now work with frozen classes.
  [#76](https://github.com/python-attrs/attrs/issues/76)
- Instantiation of `attrs` classes with converters is now significantly faster.
  [#80](https://github.com/python-attrs/attrs/pull/80)
- Pickling now works with slotted classes.
  [#81](https://github.com/python-attrs/attrs/issues/81)
- `attr.assoc()` now works with slotted classes.
  [#84](https://github.com/python-attrs/attrs/issues/84)
- The tuple returned by `attr.fields()` now also allows to access the `Attribute` instances by name.
  Yes, we've subclassed `tuple` so you don't have to!
  Therefore `attr.fields(C).x` is equivalent to the deprecated `C.x` and works with slotted classes.
  [#88](https://github.com/python-attrs/attrs/issues/88)

---

## [16.1.0](https://github.com/python-attrs/attrs/tree/16.1.0) - 2016-08-30

### Backward-incompatible Changes:

- All instances where function arguments were called `cl` have been changed to the more Pythonic `cls`.
  Since it was always the first argument, it's doubtful anyone ever called those function with in the keyword form.
  If so, sorry for any breakage but there's no practical deprecation path to solve this ugly wart.

### Deprecations:

- Accessing `Attribute` instances on class objects is now deprecated and will stop working in 2017.
  If you need introspection please use the `__attrs_attrs__` attribute or the `attr.fields()` function that carry them too.
  In the future, the attributes that are defined on the class body and are usually overwritten in your `__init__` method are simply removed after `@attr.s` has been applied.

  This will remove the confusing error message if you write your own `__init__` and forget to initialize some attribute.
  Instead you will get a straightforward `AttributeError`.
  In other words: decorated classes will work more like plain Python classes which was always `attrs`'s goal.

- The serious-business aliases `attr.attributes` and `attr.attr` have been deprecated in favor of `attr.attrs` and `attr.attrib` which are much more consistent and frankly obvious in hindsight.
  They will be purged from documentation immediately but there are no plans to actually remove them.

### Changes:

- `attr.asdict()`'s `dict_factory` arguments is now propagated on recursion.
  [#45](https://github.com/python-attrs/attrs/issues/45)
- `attr.asdict()`, `attr.has()` and `attr.fields()` are significantly faster.
  [#48](https://github.com/python-attrs/attrs/issues/48)
  [#51](https://github.com/python-attrs/attrs/issues/51)
- Add `attr.attrs` and `attr.attrib` as a more consistent aliases for `attr.s` and `attr.ib`.
- Add *frozen* option to `attr.s` that will make instances best-effort immutable.
  [#60](https://github.com/python-attrs/attrs/issues/60)
- `attr.asdict()` now takes `retain_collection_types` as an argument.
  If `True`, it does not convert attributes of type `tuple` or `set` to `list`.
  [#69](https://github.com/python-attrs/attrs/issues/69)

---

## [16.0.0](https://github.com/python-attrs/attrs/tree/16.0.0) - 2016-05-23

### Backward-incompatible Changes:

- Python 3.3 and 2.6 are no longer supported.
  They may work by chance but any effort to keep them working has ceased.

  The last Python 2.6 release was on October 29, 2013 and is no longer supported by the CPython core team.
  Major Python packages like Django and Twisted dropped Python 2.6 a while ago already.

  Python 3.3 never had a significant user base and wasn't part of any distribution's LTS release.

### Changes:

- `__slots__` have arrived!
  Classes now can automatically be [slotted](https://docs.python.org/3/reference/datamodel.html#slots)-style (and save your precious memory) just by passing `slots=True`.
  [#35](https://github.com/python-attrs/attrs/issues/35)
- Allow the case of initializing attributes that are set to `init=False`.
  This allows for clean initializer parameter lists while being able to initialize attributes to default values.
  [#32](https://github.com/python-attrs/attrs/issues/32)
- `attr.asdict()` can now produce arbitrary mappings instead of Python `dict`s when provided with a `dict_factory` argument.
  [#40](https://github.com/python-attrs/attrs/issues/40)
- Multiple performance improvements.

---

## [15.2.0](https://github.com/python-attrs/attrs/tree/15.2.0) - 2015-12-08

### Changes:

- Added a `convert` argument to `attr.ib`, which allows specifying a function to run on arguments.
  This allows for simple type conversions, e.g. with `attr.ib(convert=int)`.
  [#26](https://github.com/python-attrs/attrs/issues/26)
- Speed up object creation when attribute validators are used.
  [#28](https://github.com/python-attrs/attrs/issues/28)

---

## [15.1.0](https://github.com/python-attrs/attrs/tree/15.1.0) - 2015-08-20

### Changes:

- Added `attr.validators.optional()` that wraps other validators allowing attributes to be `None`.
  [#16](https://github.com/python-attrs/attrs/issues/16)
- Multi-level inheritance now works.
  [#24](https://github.com/python-attrs/attrs/issues/24)
- `__repr__()` now works with non-redecorated subclasses.
  [#20](https://github.com/python-attrs/attrs/issues/20)

---

## [15.0.0](https://github.com/python-attrs/attrs/tree/15.0.0) - 2015-04-15

### Changes:

Initial release.

[#136]: https://github.com/python-attrs/attrs/issues/136
[#165]: https://github.com/python-attrs/attrs/issues/165
[#425]: https://github.com/python-attrs/attrs/issues/425
