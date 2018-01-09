0.6.0
-----
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


0.5.2
-----
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


0.5.1
-----
- fix a bug and add tests for case where ``firstresult`` hooks return
  ``None`` results. Thanks to `@RonnyPfannschmidt`_ and `@tgoodlet`_
  for the issue (`#68`_) and PR (`#69`_) respectively.

.. _#69: https://github.com/pytest-dev/pluggy/pull/69
.. _#68: https://github.com/pytest-dev/pluggy/issuses/68


0.5.0
-----
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


0.4.0
-----
- add ``has_plugin(name)`` method to pluginmanager.  thanks `@nicoddemus`_.

- fix `#11`_: make plugin parsing more resilient against exceptions
  from ``__getattr__`` functions. Thanks `@nicoddemus`_.

- fix issue `#4`_: specific ``HookCallError`` exception for when a hook call
  provides not enough arguments.

- better error message when loading setuptools entrypoints fails
  due to a ``VersionConflict``.  Thanks `@blueyed`_.

.. _#11: https://github.com/pytest-dev/pluggy/issues/11
.. _#4: https://github.com/pytest-dev/pluggy/issues/4


0.3.1
-----
- avoid using deprecated-in-python3.5 getargspec method. Thanks
  `@mdboom`_.


0.3.0
-----
initial release

.. contributors
.. _@hpk42: https://github.com/hpk42
.. _@tgoodlet: https://github.com/tgoodlet
.. _@MichalTHEDUDE: https://github.com/MichalTHEDUDE
.. _@vodik: https://github.com/vodik
.. _@RonnyPfannschmidt: https://github.com/RonnyPfannschmidt
.. _@blueyed: https://github.com/blueyed
.. _@nicoddemus: https://github.com/nicoddemus
.. _@mdboom: https://github.com/mdboom
