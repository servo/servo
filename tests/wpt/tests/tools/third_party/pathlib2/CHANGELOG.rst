History
-------

Version 2.3.5
^^^^^^^^^^^^^

- Fall back to ascii when getfilesystemencoding returns None (see
  issue #59).

Version 2.3.4
^^^^^^^^^^^^^

- Do not raise windows error when calling resolve on a non-existing
  path in Python 2.7, to match behaviour on Python 3.x (see issue #54).

- Use the new collections.abc when possible (see issue #53).

- Sync with upstream pathlib (see issues #47 and #51).

Version 2.3.3
^^^^^^^^^^^^^

- Bring back old deprecated dependency syntax to ensure compatibility
  with older systems (see issue #46).

- Drop Python 3.3 support, as scandir no longer supports it.

- Add Python 3.7 support.

Version 2.3.2
^^^^^^^^^^^^^

- Hotfix for broken setup.py.

Version 2.3.1
^^^^^^^^^^^^^

- Fix tests for systems where filesystem encoding only supports ascii
  (reported by yurivict, fixed with help of honnibal, see issue #30).

- Use modern setuptools syntax for specifying conditional scandir
  dependency (see issue #31).

- Remove legacy use of support module from old pathlib module (see
  issue #39). This fixes the tests for Python 3.6.

- Drop the "from __future__ import unicode_literals" and -Qnew tests
  as it introduced subtle bugs in the tests, and maintaining separate
  test modules for these legacy features seems not worth the effort.

- Drop Python 3.2 support, as scandir no longer supports it.

Version 2.3.0
^^^^^^^^^^^^^

- Sync with upstream pathlib from CPython 3.6.1 (7d1017d).

Version 2.2.1
^^^^^^^^^^^^^

- Fix conditional scandir dependency in wheel (reported by AvdN, see
  issue #20 and pull request #21).

Version 2.2.0
^^^^^^^^^^^^^

- Sync with upstream pathlib from CPython 3.5.2 and 3.6.0: fix various
  exceptions, empty glob pattern, scandir, __fspath__.

- Support unicode strings to be used to construct paths in Python 2
  (reported by native-api, see issue #13 and pull request #15).

Version 2.1.0
^^^^^^^^^^^^^

- Sync with upstream pathlib from CPython 3.5.0: gethomedir, home,
  expanduser.

Version 2.0.1
^^^^^^^^^^^^^

- Fix TypeError exceptions in write_bytes and write_text (contributed
  by Emanuele Gaifas, see pull request #2).

Version 2.0
^^^^^^^^^^^

- Sync with upstream pathlib from CPython: read_text, write_text,
  read_bytes, write_bytes, __enter__, __exit__, samefile.
- Use travis and appveyor for continuous integration.
- Fixed some bugs in test code.

Version 1.0.1
^^^^^^^^^^^^^

- Pull request #4: Python 2.6 compatibility by eevee.

Version 1.0
^^^^^^^^^^^

This version brings ``pathlib`` up to date with the official Python 3.4
release, and also fixes a couple of 2.7-specific issues.

- Python issue #20765: Add missing documentation for PurePath.with_name()
  and PurePath.with_suffix().
- Fix test_mkdir_parents when the working directory has additional bits
  set (such as the setgid or sticky bits).
- Python issue #20111: pathlib.Path.with_suffix() now sanity checks the
  given suffix.
- Python issue #19918: Fix PurePath.relative_to() under Windows.
- Python issue #19921: When Path.mkdir() is called with parents=True, any
  missing parent is created with the default permissions, ignoring the mode
  argument (mimicking the POSIX "mkdir -p" command).
- Python issue #19887: Improve the Path.resolve() algorithm to support
  certain symlink chains.
- Make pathlib usable under Python 2.7 with unicode pathnames (only pure
  ASCII, though).
- Issue #21: fix TypeError under Python 2.7 when using new division.
- Add tox support for easier testing.

Version 0.97
^^^^^^^^^^^^

This version brings ``pathlib`` up to date with the final API specified
in :pep:`428`.  The changes are too long to list here, it is recommended
to read the `documentation <https://pathlib.readthedocs.org/>`_.

.. warning::
   The API in this version is partially incompatible with pathlib 0.8 and
   earlier.  Be sure to check your code for possible breakage!

Version 0.8
^^^^^^^^^^^

- Add PurePath.name and PurePath.anchor.
- Add Path.owner and Path.group.
- Add Path.replace().
- Add Path.as_uri().
- Issue #10: when creating a file with Path.open(), don't set the executable
  bit.
- Issue #11: fix comparisons with non-Path objects.

Version 0.7
^^^^^^^^^^^

- Add '**' (recursive) patterns to Path.glob().
- Fix openat() support after the API refactoring in Python 3.3 beta1.
- Add a *target_is_directory* argument to Path.symlink_to()

Version 0.6
^^^^^^^^^^^

- Add Path.is_file() and Path.is_symlink()
- Add Path.glob() and Path.rglob()
- Add PurePath.match()

Version 0.5
^^^^^^^^^^^

- Add Path.mkdir().
- Add Python 2.7 compatibility by Michele Lacchia.
- Make parent() raise ValueError when the level is greater than the path
  length.
