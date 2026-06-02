Version history
===============

This library adheres to `Semantic Versioning 2.0 <http://semver.org/>`_.

**1.2.1**

- Updated the copying of ``__notes__`` to match CPython behavior (PR by CF Bolz-Tereick)
- Corrected the type annotation of the exception handler callback to accept a
  ``BaseExceptionGroup`` instead of ``BaseException``
- Fixed type errors on Python < 3.10 and the type annotation of ``suppress()``
  (PR by John Litborn)

**1.2.0**

- Added special monkeypatching if `Apport <https://github.com/canonical/apport>`_ has
  overridden ``sys.excepthook`` so it will format exception groups correctly
  (PR by John Litborn)
- Added a backport of ``contextlib.suppress()`` from Python 3.12.1 which also handles
  suppressing exceptions inside exception groups
- Fixed bare ``raise`` in a handler reraising the original naked exception rather than
  an exception group which is what is raised when you do a ``raise`` in an ``except*``
  handler

**1.1.3**

- ``catch()`` now raises a ``TypeError`` if passed an async exception handler instead of
  just giving a ``RuntimeWarning`` about the coroutine never being awaited. (#66, PR by
  John Litborn)
- Fixed plain ``raise`` statement in an exception handler callback to work like a
  ``raise`` in an ``except*`` block
- Fixed new exception group not being chained to the original exception when raising an
  exception group from exceptions raised in handler callbacks
- Fixed type annotations of the ``derive()``, ``subgroup()`` and ``split()`` methods to
  match the ones in typeshed

**1.1.2**

- Changed handling of exceptions in exception group handler callbacks to not wrap a
  single exception in an exception group, as per
  `CPython issue 103590 <https://github.com/python/cpython/issues/103590>`_

**1.1.1**

- Worked around
  `CPython issue #98778 <https://github.com/python/cpython/issues/98778>`_,
  ``urllib.error.HTTPError(..., fp=None)`` raises ``KeyError`` on unknown attribute
  access, on affected Python versions. (PR by Zac Hatfield-Dodds)

**1.1.0**

- Backported upstream fix for gh-99553 (custom subclasses of ``BaseExceptionGroup`` that
  also inherit from ``Exception`` should not be able to wrap base exceptions)
- Moved all initialization code to ``__new__()`` (thus matching Python 3.11 behavior)

**1.0.4**

- Fixed regression introduced in v1.0.3 where the code computing the suggestions would
  assume that both the ``obj`` attribute of ``AttributeError`` is always available, even
  though this is only true from Python 3.10 onwards
  (#43; PR by Carl Friedrich Bolz-Tereick)

**1.0.3**

- Fixed monkey patching breaking suggestions (on a ``NameError`` or ``AttributeError``)
  on Python 3.10 (#41; PR by Carl Friedrich Bolz-Tereick)

**1.0.2**

- Updated type annotations to match the ones in ``typeshed``

**1.0.1**

- Fixed formatted traceback missing exceptions beyond 2 nesting levels of
  ``__context__`` or ``__cause__``

**1.0.0**

- Fixed
  ``AttributeError: 'PatchedTracebackException' object has no attribute '__cause__'``
  on Python 3.10 (only) when a traceback is printed from an exception where an exception
  group is set as the cause (#33)
- Fixed a loop in exception groups being rendered incorrectly (#35)
- Fixed the patched formatting functions (``format_exception()``etc.) not passing the
  ``compact=True`` flag on Python 3.10 like the original functions do

**1.0.0rc9**

- Added custom versions of several ``traceback``  functions that work with exception
  groups even if monkey patching was disabled or blocked

**1.0.0rc8**

- Don't monkey patch anything if ``sys.excepthook`` has been altered
- Fixed formatting of ``SyntaxError`` in the monkey patched
  ``TracebackException.format_exception_only()`` method

**1.0.0rc7**

- **BACKWARDS INCOMPATIBLE** Changed ``catch()`` to not wrap an exception in an
  exception group if only one exception arrived at ``catch()`` and it was not matched
  with any handlers. This was to match the behavior of ``except*``.

**1.0.0rc6**

- **BACKWARDS INCOMPATIBLE** Changed ``catch()`` to match the behavior of ``except*``:
  each handler will be called only once per key in the ``handlers`` dictionary, and with
  an exception group as the argument. Handlers now also catch subclasses of the given
  exception types, just like ``except*``.

**1.0.0rc5**

- Patch for ``traceback.TracebackException.format_exception_only()`` (PR by Zac Hatfield-Dodds)

**1.0.0rc4**

- Update `PEP 678`_ support to use ``.add_note()`` and ``__notes__`` (PR by Zac Hatfield-Dodds)

**1.0.0rc3**

- Added message about the number of sub-exceptions

**1.0.0rc2**

- Display and copy ``__note__`` (draft `PEP 678`_) if available (PR by Zac Hatfield-Dodds)

.. _PEP 678: https://www.python.org/dev/peps/pep-0678/

**1.0.0rc1**

- Initial release
