=========
Changelog
=========

Versions follow `Semantic Versioning <https://semver.org/>`_ (``<major>.<minor>.<patch>``).

..
    You should *NOT* be adding new change log entries to this file, this
    file is managed by towncrier. You *may* edit previous change logs to
    fix problems like typo corrections or such.
    To add a new change log entry, please see
    https://pip.pypa.io/en/latest/development/contributing/#news-entries
    we named the news folder changelog

.. only:: changelog_towncrier_draft

    .. The 'changelog_towncrier_draft' tag is included by our 'tox -e docs',
       but not on readthedocs.

    .. include:: _changelog_towncrier_draft.rst

.. towncrier release notes start

pluggy 1.5.0 (2024-04-19)
=========================

Features
--------

- `#178 <https://github.com/pytest-dev/pluggy/issues/178>`_: Add support for deprecating specific hook parameters, or more generally, for issuing a warning whenever a hook implementation requests certain parameters.

  See :ref:`warn_on_impl` for details.



Bug Fixes
---------

- `#481 <https://github.com/pytest-dev/pluggy/issues/481>`_: ``PluginManager.get_plugins()`` no longer returns ``None`` for blocked plugins.


pluggy 1.4.0 (2024-01-24)
=========================

Features
--------

- `#463 <https://github.com/pytest-dev/pluggy/issues/463>`_: A warning :class:`~pluggy.PluggyTeardownRaisedWarning` is now issued when an old-style hookwrapper raises an exception during teardown.
  See the warning documentation for more details.

- `#471 <https://github.com/pytest-dev/pluggy/issues/471>`_: Add :func:`PluginManager.unblock <pluggy.PluginManager.unblock>` method to unblock a plugin by plugin name.

Bug Fixes
---------

- `#441 <https://github.com/pytest-dev/pluggy/issues/441>`_: Fix :func:`~pluggy.HookCaller.call_extra()` extra methods getting ordered before everything else in some circumstances. Regressed in pluggy 1.1.0.

- `#438 <https://github.com/pytest-dev/pluggy/issues/438>`_: Fix plugins registering other plugins in a hook when the other plugins implement the same hook itself. Regressed in pluggy 1.1.0.


pluggy 1.3.0 (2023-08-26)
=========================

Deprecations and Removals
-------------------------

- `#426 <https://github.com/pytest-dev/pluggy/issues/426>`_: Python 3.7 is no longer supported.



Features
--------

- `#428 <https://github.com/pytest-dev/pluggy/issues/428>`_: Pluggy now exposes its typings to static type checkers.

  As part of this, the following changes are made:

  - Renamed ``_Result`` to ``Result``, and exported as :class:`pluggy.Result`.
  - Renamed ``_HookRelay`` to ``HookRelay``, and exported as :class:`pluggy.HookRelay`.
  - Renamed ``_HookCaller`` to ``HookCaller``, and exported as :class:`pluggy.HookCaller`.
  - Exported ``HookImpl`` as :class:`pluggy.HookImpl`.
  - Renamed ``_HookImplOpts`` to ``HookimplOpts``, and exported as :class:`pluggy.HookimplOpts`.
  - Renamed ``_HookSpecOpts`` to ``HookspecOpts``, and exported as :class:`pluggy.HookspecOpts`.
  - Some fields and classes are marked ``Final`` and ``@final``.
  - The :ref:`api-reference` is updated to clearly delineate pluggy's public API.

  Compatibility aliases are put in place for the renamed types.
  We do not plan to remove the aliases, but we strongly recommend to only import from ``pluggy.*`` to ensure future compatibility.

  Please note that pluggy is currently unable to provide strong typing for hook calls, e.g. ``pm.hook.my_hook(...)``,
  nor to statically check that a hook implementation matches the hook specification's type.


pluggy 1.2.0 (2023-06-21)
=========================

Features
--------

- `#405 <https://github.com/pytest-dev/pluggy/issues/405>`_: The new-style hook wrappers, added in the yanked 1.1.0 release, now require an explicit ``wrapper=True`` designation in the ``@hookimpl()`` decorator.


pluggy 1.1.0 (YANKED)
=====================

.. note::

  This release was yanked because unfortunately the implicit new-style hook wrappers broke some downstream projects.
  See `#403 <https://github.com/pytest-dev/pluggy/issues/403>`__ for more information.
  This was rectified in the 1.2.0 release.

Deprecations and Removals
-------------------------

- `#364 <https://github.com/pytest-dev/pluggy/issues/364>`_: Python 3.6 is no longer supported.



Features
--------

- `#260 <https://github.com/pytest-dev/pluggy/issues/260>`_: Added "new-style" hook wrappers, a simpler but equally powerful alternative to the existing ``hookwrapper=True`` wrappers.

  New-style wrappers are generator functions, similarly to ``hookwrapper``, but do away with the :class:`result <pluggy.Result>` object.
  Instead, the return value is sent directly to the ``yield`` statement, or, if inner calls raised an exception, it is raised from the ``yield``.
  The wrapper is expected to return a value or raise an exception, which will become the result of the hook call.

  New-style wrappers are fully interoperable with old-style wrappers.
  We encourage users to use the new style, however we do not intend to deprecate the old style any time soon.

  See :ref:`hookwrappers` for the full documentation.


- `#364 <https://github.com/pytest-dev/pluggy/issues/364>`_: Python 3.11 and 3.12 are now officially supported.


- `#394 <https://github.com/pytest-dev/pluggy/issues/394>`_: Added the :meth:`~pluggy.Result.force_exception` method to ``_Result``.

  ``force_exception`` allows (old-style) hookwrappers to force an exception or override/adjust an existing exception of a hook invocation,
  in a properly behaving manner. Using ``force_exception`` is preferred over raising an exception from the hookwrapper,
  because raising an exception causes other hookwrappers to be skipped.


pluggy 1.0.0 (2021-08-25)
=========================

Deprecations and Removals
-------------------------

- `#116 <https://github.com/pytest-dev/pluggy/issues/116>`_: Remove deprecated ``implprefix`` support.
  Decorate hook implementations using an instance of HookimplMarker instead.
  The deprecation was announced in release ``0.7.0``.


- `#120 <https://github.com/pytest-dev/pluggy/issues/120>`_: Remove the deprecated ``proc`` argument to ``call_historic``.
  Use ``result_callback`` instead, which has the same behavior.
  The deprecation was announced in release ``0.7.0``.


- `#265 <https://github.com/pytest-dev/pluggy/issues/265>`_: Remove the ``_Result.result`` property. Use ``_Result.get_result()`` instead.
  Note that unlike ``result``, ``get_result()`` raises the exception if the hook raised.
  The deprecation was announced in release ``0.6.0``.


- `#267 <https://github.com/pytest-dev/pluggy/issues/267>`_: Remove official support for Python 3.4.


- `#272 <https://github.com/pytest-dev/pluggy/issues/272>`_: Dropped support for Python 2.
  Continue to use pluggy 0.13.x for Python 2 support.


- `#308 <https://github.com/pytest-dev/pluggy/issues/308>`_: Remove official support for Python 3.5.


- `#313 <https://github.com/pytest-dev/pluggy/issues/313>`_: The internal ``pluggy.callers``, ``pluggy.manager`` and ``pluggy.hooks`` are now explicitly marked private by a ``_`` prefix (e.g. ``pluggy._callers``).
  Only API exported by the top-level ``pluggy`` module is considered public.


- `#59 <https://github.com/pytest-dev/pluggy/issues/59>`_: Remove legacy ``__multicall__`` recursive hook calling system.
  The deprecation was announced in release ``0.5.0``.



Features
--------

- `#282 <https://github.com/pytest-dev/pluggy/issues/282>`_: When registering a hookimpl which is declared as ``hookwrapper=True`` but whose
  function is not a generator function, a :class:`~pluggy.PluginValidationError` exception is
  now raised.

  Previously this problem would cause an error only later, when calling the hook.

  In the unlikely case that you have a hookwrapper that *returns* a generator
  instead of yielding directly, for example:

  .. code-block:: python

      def my_hook_implementation(arg):
          print("before")
          yield
          print("after")


      @hookimpl(hookwrapper=True)
      def my_hook(arg):
          return my_hook_implementation(arg)

  change it to use ``yield from`` instead:

  .. code-block:: python

      @hookimpl(hookwrapper=True)
      def my_hook(arg):
          yield from my_hook_implementation(arg)


- `#309 <https://github.com/pytest-dev/pluggy/issues/309>`_: Add official support for Python 3.9.

- `#251 <https://github.com/pytest-dev/pluggy/issues/251>`_: Add ``specname`` option to ``@hookimpl``. If ``specname`` is provided, it will be used
  instead of the function name when matching this hook implementation to a hook specification during registration (allowing a plugin to register
  a hook implementation that was not named the same thing as the corresponding ``@hookspec``).


pluggy 0.13.1 (2019-11-21)
==========================

Trivial/Internal Changes
------------------------

- `#236 <https://github.com/pytest-dev/pluggy/pull/236>`_: Improved documentation, especially with regard to references.


pluggy 0.13.0 (2019-09-10)
==========================

Trivial/Internal Changes
------------------------

- `#222 <https://github.com/pytest-dev/pluggy/issues/222>`_: Replace ``importlib_metadata`` backport with ``importlib.metadata`` from the
  standard library on Python 3.8+.


pluggy 0.12.0 (2019-05-27)
==========================

Features
--------

- `#215 <https://github.com/pytest-dev/pluggy/issues/215>`_: Switch from ``pkg_resources`` to ``importlib-metadata`` for entrypoint detection for improved performance and import time.  This time with ``.egg`` support.


pluggy 0.11.0 (2019-05-07)
==========================

Bug Fixes
---------

- `#205 <https://github.com/pytest-dev/pluggy/issues/205>`_: Revert changes made in 0.10.0 release breaking ``.egg`` installs.


pluggy 0.10.0 (2019-05-07)
==========================

Features
--------

- `#199 <https://github.com/pytest-dev/pluggy/issues/199>`_: Switch from ``pkg_resources`` to ``importlib-metadata`` for entrypoint detection for improved performance and import time.


pluggy 0.9.0 (2019-02-21)
=========================

Features
--------

- `#189 <https://github.com/pytest-dev/pluggy/issues/189>`_: ``PluginManager.load_setuptools_entrypoints`` now accepts a ``name`` parameter that when given will
  load only entry points with that name.

  ``PluginManager.load_setuptools_entrypoints`` also now returns the number of plugins loaded by the
  call, as opposed to the number of all plugins loaded by all calls to this method.



Bug Fixes
---------

- `#187 <https://github.com/pytest-dev/pluggy/issues/187>`_: Fix internal ``varnames`` function for PyPy3.


pluggy 0.8.1 (2018-11-09)
=========================

Trivial/Internal Changes
------------------------

- `#166 <https://github.com/pytest-dev/pluggy/issues/166>`_: Add ``stacklevel=2`` to implprefix warning so that the reported location of warning is the caller of PluginManager.


pluggy 0.8.0 (2018-10-15)
=========================

Features
--------

- `#177 <https://github.com/pytest-dev/pluggy/issues/177>`_: Add ``get_hookimpls()`` method to hook callers.



Trivial/Internal Changes
------------------------

- `#165 <https://github.com/pytest-dev/pluggy/issues/165>`_: Add changelog in long package description and documentation.


- `#172 <https://github.com/pytest-dev/pluggy/issues/172>`_: Add a test exemplifying the opt-in nature of spec defined args.


- `#57 <https://github.com/pytest-dev/pluggy/issues/57>`_: Encapsulate hook specifications in a type for easier introspection.


pluggy 0.7.1 (2018-07-28)
=========================

Deprecations and Removals
-------------------------

- `#116 <https://github.com/pytest-dev/pluggy/issues/116>`_: Deprecate the ``implprefix`` kwarg to ``PluginManager`` and instead
  expect users to start using explicit ``HookimplMarker`` everywhere.



Features
--------

- `#122 <https://github.com/pytest-dev/pluggy/issues/122>`_: Add ``.plugin`` member to ``PluginValidationError`` to access failing plugin during post-mortem.


- `#138 <https://github.com/pytest-dev/pluggy/issues/138>`_: Add per implementation warnings support for hookspecs allowing for both
  deprecation and future warnings of legacy and (future) experimental hooks
  respectively.



Bug Fixes
---------

- `#110 <https://github.com/pytest-dev/pluggy/issues/110>`_: Fix a bug where ``_HookCaller.call_historic()`` would call the ``proc``
  arg even when the default is ``None`` resulting in a ``TypeError``.

- `#160 <https://github.com/pytest-dev/pluggy/issues/160>`_: Fix problem when handling ``VersionConflict`` errors when loading setuptools plugins.



Improved Documentation
----------------------

- `#123 <https://github.com/pytest-dev/pluggy/issues/123>`_: Document how exceptions are handled and how the hook call loop
  terminates immediately on the first error which is then delivered
  to any surrounding wrappers.


- `#136 <https://github.com/pytest-dev/pluggy/issues/136>`_: Docs rework including a much better introduction and comprehensive example
  set for new users. A big thanks goes out to @obestwalter for the great work!



Trivial/Internal Changes
------------------------

- `#117 <https://github.com/pytest-dev/pluggy/issues/117>`_: Break up the main monolithic package modules into separate modules by concern


- `#131 <https://github.com/pytest-dev/pluggy/issues/131>`_: Automate ``setuptools`` wheels building and PyPi upload using TravisCI.


- `#153 <https://github.com/pytest-dev/pluggy/issues/153>`_: Reorganize tests more appropriately by modules relating to each
  internal component/feature. This is in an effort to avoid (future)
  duplication and better separation of concerns in the test set.


- `#156 <https://github.com/pytest-dev/pluggy/issues/156>`_: Add ``HookImpl.__repr__()`` for better debugging.


- `#66 <https://github.com/pytest-dev/pluggy/issues/66>`_: Start using ``towncrier`` and a custom ``tox`` environment to prepare releases!


pluggy 0.7.0 (Unreleased)
=========================

* `#160 <https://github.com/pytest-dev/pluggy/issues/160>`_: We discovered a deployment issue so this version was never released to PyPI, only the tag exists.

pluggy 0.6.0 (2017-11-24)
=========================

- Add CI testing for the features, release, and master
  branches of ``pytest`` (PR `#79`_).
- Document public API for ``_Result`` objects passed to wrappers
  (PR `#85`_).
- Document and test hook LIFO ordering (PR `#85`_).
- Turn warnings into errors in test suite (PR `#89`_).
- Deprecate ``_Result.result`` (PR `#88`_).
- Convert ``_Multicall`` to a simple function distinguishing it from
  the legacy version (PR `#90`_).
- Resolve E741 errors (PR `#96`_).
- Test and bug fix for unmarked hook collection (PRs `#97`_ and
  `#102`_).
- Drop support for EOL Python 2.6 and 3.3 (PR `#103`_).
- Fix ``inspect`` based arg introspection on py3.6 (PR `#94`_).

.. _#79: https://github.com/pytest-dev/pluggy/pull/79
.. _#85: https://github.com/pytest-dev/pluggy/pull/85
.. _#88: https://github.com/pytest-dev/pluggy/pull/88
.. _#89: https://github.com/pytest-dev/pluggy/pull/89
.. _#90: https://github.com/pytest-dev/pluggy/pull/90
.. _#94: https://github.com/pytest-dev/pluggy/pull/94
.. _#96: https://github.com/pytest-dev/pluggy/pull/96
.. _#97: https://github.com/pytest-dev/pluggy/pull/97
.. _#102: https://github.com/pytest-dev/pluggy/pull/102
.. _#103: https://github.com/pytest-dev/pluggy/pull/103


pluggy 0.5.2 (2017-09-06)
=========================

- fix bug where ``firstresult`` wrappers were being sent an incorrectly configured
  ``_Result`` (a list was set instead of a single value). Add tests to check for
  this as well as ``_Result.force_result()`` behaviour. Thanks to `@tgoodlet`_
  for the PR `#72`_.

- fix incorrect ``getattr``  of ``DeprecationWarning`` from the ``warnings``
  module. Thanks to `@nicoddemus`_ for the PR `#77`_.

- hide ``pytest`` tracebacks in certain core routines. Thanks to
  `@nicoddemus`_ for the PR `#80`_.

.. _#72: https://github.com/pytest-dev/pluggy/pull/72
.. _#77: https://github.com/pytest-dev/pluggy/pull/77
.. _#80: https://github.com/pytest-dev/pluggy/pull/80


pluggy 0.5.1 (2017-08-29)
=========================

- fix a bug and add tests for case where ``firstresult`` hooks return
  ``None`` results. Thanks to `@RonnyPfannschmidt`_ and `@tgoodlet`_
  for the issue (`#68`_) and PR (`#69`_) respectively.

.. _#69: https://github.com/pytest-dev/pluggy/pull/69
.. _#68: https://github.com/pytest-dev/pluggy/issues/68


pluggy 0.5.0 (2017-08-28)
=========================

- fix bug where callbacks for historic hooks would not be called for
  already registered plugins.  Thanks `@vodik`_ for the PR
  and `@hpk42`_ for further fixes.

- fix `#17`_ by considering only actual functions for hooks
  this removes the ability to register arbitrary callable objects
  which at first glance is a reasonable simplification,
  thanks `@RonnyPfannschmidt`_ for report and pr.

- fix `#19`_: allow registering hookspecs from instances.  The PR from
  `@tgoodlet`_ also modernized the varnames implementation.

- resolve `#32`_: split up the test set into multiple modules.
  Thanks to `@RonnyPfannschmidt`_ for the PR and `@tgoodlet`_ for
  the initial request.

- resolve `#14`_: add full sphinx docs. Thanks to `@tgoodlet`_ for
  PR `#39`_.

- add hook call mismatch warnings. Thanks to `@tgoodlet`_ for the
  PR `#42`_.

- resolve `#44`_: move to new-style classes. Thanks to `@MichalTHEDUDE`_
  for PR `#46`_.

- add baseline benchmarking/speed tests using ``pytest-benchmark``
  in PR `#54`_.  Thanks to `@tgoodlet`_.

- update the README to showcase the API. Thanks to `@tgoodlet`_ for the
  issue and PR `#55`_.

- deprecate ``__multicall__`` and add a faster call loop implementation.
  Thanks to `@tgoodlet`_ for PR `#58`_.

- raise a comprehensible error when a ``hookimpl`` is called with positional
  args. Thanks to `@RonnyPfannschmidt`_ for the issue and `@tgoodlet`_ for
  PR `#60`_.

- fix the ``firstresult`` test making it more complete
  and remove a duplicate of that test. Thanks to `@tgoodlet`_
  for PR `#62`_.

.. _#62: https://github.com/pytest-dev/pluggy/pull/62
.. _#60: https://github.com/pytest-dev/pluggy/pull/60
.. _#58: https://github.com/pytest-dev/pluggy/pull/58
.. _#55: https://github.com/pytest-dev/pluggy/pull/55
.. _#54: https://github.com/pytest-dev/pluggy/pull/54
.. _#46: https://github.com/pytest-dev/pluggy/pull/46
.. _#44: https://github.com/pytest-dev/pluggy/issues/44
.. _#42: https://github.com/pytest-dev/pluggy/pull/42
.. _#39: https://github.com/pytest-dev/pluggy/pull/39
.. _#32: https://github.com/pytest-dev/pluggy/pull/32
.. _#19: https://github.com/pytest-dev/pluggy/issues/19
.. _#17: https://github.com/pytest-dev/pluggy/issues/17
.. _#14: https://github.com/pytest-dev/pluggy/issues/14


pluggy 0.4.0 (2016-09-25)
=========================

- add ``has_plugin(name)`` method to pluginmanager.  thanks `@nicoddemus`_.

- fix `#11`_: make plugin parsing more resilient against exceptions
  from ``__getattr__`` functions. Thanks `@nicoddemus`_.

- fix issue `#4`_: specific ``HookCallError`` exception for when a hook call
  provides not enough arguments.

- better error message when loading setuptools entrypoints fails
  due to a ``VersionConflict``.  Thanks `@blueyed`_.

.. _#11: https://github.com/pytest-dev/pluggy/issues/11
.. _#4: https://github.com/pytest-dev/pluggy/issues/4


pluggy 0.3.1 (2015-09-17)
=========================

- avoid using deprecated-in-python3.5 getargspec method. Thanks
  `@mdboom`_.


pluggy 0.3.0 (2015-05-07)
=========================

initial release

.. contributors
.. _@hpk42: https://github.com/hpk42
.. _@tgoodlet: https://github.com/goodboy
.. _@MichalTHEDUDE: https://github.com/MichalTHEDUDE
.. _@vodik: https://github.com/vodik
.. _@RonnyPfannschmidt: https://github.com/RonnyPfannschmidt
.. _@blueyed: https://github.com/blueyed
.. _@nicoddemus: https://github.com/nicoddemus
.. _@mdboom: https://github.com/mdboom
