Changelog
---------

21.3 - 2021-11-17
~~~~~~~~~~~~~~~~~

* Add a ``pp3-none-any`` tag (:issue:`311`)
* Replace the blank pyparsing 3 exclusion with a 3.0.5 exclusion (:issue:`481`, :issue:`486`)
* Fix a spelling mistake (:issue:`479`)

21.2 - 2021-10-29
~~~~~~~~~~~~~~~~~

* Update documentation entry for 21.1.

21.1 - 2021-10-29
~~~~~~~~~~~~~~~~~

* Update pin to pyparsing to exclude 3.0.0.

21.0 - 2021-07-03
~~~~~~~~~~~~~~~~~

* PEP 656: musllinux support (:issue:`411`)
* Drop support for Python 2.7, Python 3.4 and Python 3.5.
* Replace distutils usage with sysconfig (:issue:`396`)
* Add support for zip files in ``parse_sdist_filename`` (:issue:`429`)
* Use cached ``_hash`` attribute to short-circuit tag equality comparisons (:issue:`417`)
* Specify the default value for the ``specifier`` argument to ``SpecifierSet`` (:issue:`437`)
* Proper keyword-only "warn" argument in packaging.tags (:issue:`403`)
* Correctly remove prerelease suffixes from ~= check (:issue:`366`)
* Fix type hints for ``Version.post`` and ``Version.dev`` (:issue:`393`)
* Use typing alias ``UnparsedVersion`` (:issue:`398`)
* Improve type inference for ``packaging.specifiers.filter()`` (:issue:`430`)
* Tighten the return type of ``canonicalize_version()`` (:issue:`402`)

20.9 - 2021-01-29
~~~~~~~~~~~~~~~~~

* Run `isort <https://pypi.org/project/isort/>`_ over the code base (:issue:`377`)
* Add support for the ``macosx_10_*_universal2`` platform tags (:issue:`379`)
* Introduce ``packaging.utils.parse_wheel_filename()`` and ``parse_sdist_filename()``
  (:issue:`387` and :issue:`389`)

20.8 - 2020-12-11
~~~~~~~~~~~~~~~~~

* Revert back to setuptools for compatibility purposes for some Linux distros (:issue:`363`)
* Do not insert an underscore in wheel tags when the interpreter version number
  is more than 2 digits (:issue:`372`)

20.7 - 2020-11-28
~~~~~~~~~~~~~~~~~

No unreleased changes.

20.6 - 2020-11-28
~~~~~~~~~~~~~~~~~

.. note:: This release was subsequently yanked, and these changes were included in 20.7.

* Fix flit configuration, to include LICENSE files (:issue:`357`)
* Make `intel` a recognized CPU architecture for the `universal` macOS platform tag (:issue:`361`)
* Add some missing type hints to `packaging.requirements` (issue:`350`)

20.5 - 2020-11-27
~~~~~~~~~~~~~~~~~

* Officially support Python 3.9 (:issue:`343`)
* Deprecate the ``LegacyVersion`` and ``LegacySpecifier`` classes (:issue:`321`)
* Handle ``OSError`` on non-dynamic executables when attempting to resolve
  the glibc version string.

20.4 - 2020-05-19
~~~~~~~~~~~~~~~~~

* Canonicalize version before comparing specifiers. (:issue:`282`)
* Change type hint for ``canonicalize_name`` to return
  ``packaging.utils.NormalizedName``.
  This enables the use of static typing tools (like mypy) to detect mixing of
  normalized and un-normalized names.

20.3 - 2020-03-05
~~~~~~~~~~~~~~~~~

* Fix changelog for 20.2.

20.2 - 2020-03-05
~~~~~~~~~~~~~~~~~

* Fix a bug that caused a 32-bit OS that runs on a 64-bit ARM CPU (e.g. ARM-v8,
  aarch64), to report the wrong bitness.

20.1 - 2020-01-24
~~~~~~~~~~~~~~~~~~~

* Fix a bug caused by reuse of an exhausted iterator. (:issue:`257`)

20.0 - 2020-01-06
~~~~~~~~~~~~~~~~~

* Add type hints (:issue:`191`)

* Add proper trove classifiers for PyPy support (:issue:`198`)

* Scale back depending on ``ctypes`` for manylinux support detection (:issue:`171`)

* Use ``sys.implementation.name`` where appropriate for ``packaging.tags`` (:issue:`193`)

* Expand upon the API provided by ``packaging.tags``: ``interpreter_name()``, ``mac_platforms()``, ``compatible_tags()``, ``cpython_tags()``, ``generic_tags()`` (:issue:`187`)

* Officially support Python 3.8 (:issue:`232`)

* Add ``major``, ``minor``, and ``micro`` aliases to ``packaging.version.Version`` (:issue:`226`)

* Properly mark ``packaging`` has being fully typed by adding a `py.typed` file (:issue:`226`)

19.2 - 2019-09-18
~~~~~~~~~~~~~~~~~

* Remove dependency on ``attrs`` (:issue:`178`, :issue:`179`)

* Use appropriate fallbacks for CPython ABI tag (:issue:`181`, :issue:`185`)

* Add manylinux2014 support (:issue:`186`)

* Improve ABI detection (:issue:`181`)

* Properly handle debug wheels for Python 3.8 (:issue:`172`)

* Improve detection of debug builds on Windows (:issue:`194`)

19.1 - 2019-07-30
~~~~~~~~~~~~~~~~~

* Add the ``packaging.tags`` module. (:issue:`156`)

* Correctly handle two-digit versions in ``python_version`` (:issue:`119`)


19.0 - 2019-01-20
~~~~~~~~~~~~~~~~~

* Fix string representation of PEP 508 direct URL requirements with markers.

* Better handling of file URLs

  This allows for using ``file:///absolute/path``, which was previously
  prevented due to the missing ``netloc``.

  This allows for all file URLs that ``urlunparse`` turns back into the
  original URL to be valid.


18.0 - 2018-09-26
~~~~~~~~~~~~~~~~~

* Improve error messages when invalid requirements are given. (:issue:`129`)


17.1 - 2017-02-28
~~~~~~~~~~~~~~~~~

* Fix ``utils.canonicalize_version`` when supplying non PEP 440 versions.


17.0 - 2017-02-28
~~~~~~~~~~~~~~~~~

* Drop support for python 2.6, 3.2, and 3.3.

* Define minimal pyparsing version to 2.0.2 (:issue:`91`).

* Add ``epoch``, ``release``, ``pre``, ``dev``, and ``post`` attributes to
  ``Version`` and ``LegacyVersion`` (:issue:`34`).

* Add ``Version().is_devrelease`` and ``LegacyVersion().is_devrelease`` to
  make it easy to determine if a release is a development release.

* Add ``utils.canonicalize_version`` to canonicalize version strings or
  ``Version`` instances (:issue:`121`).


16.8 - 2016-10-29
~~~~~~~~~~~~~~~~~

* Fix markers that utilize ``in`` so that they render correctly.

* Fix an erroneous test on Python RC releases.


16.7 - 2016-04-23
~~~~~~~~~~~~~~~~~

* Add support for the deprecated ``python_implementation`` marker which was
  an undocumented setuptools marker in addition to the newer markers.


16.6 - 2016-03-29
~~~~~~~~~~~~~~~~~

* Add support for the deprecated, PEP 345 environment markers in addition to
  the newer markers.


16.5 - 2016-02-26
~~~~~~~~~~~~~~~~~

* Fix a regression in parsing requirements with whitespaces between the comma
  separators.


16.4 - 2016-02-22
~~~~~~~~~~~~~~~~~

* Fix a regression in parsing requirements like ``foo (==4)``.


16.3 - 2016-02-21
~~~~~~~~~~~~~~~~~

* Fix a bug where ``packaging.requirements:Requirement`` was overly strict when
  matching legacy requirements.


16.2 - 2016-02-09
~~~~~~~~~~~~~~~~~

* Add a function that implements the name canonicalization from PEP 503.


16.1 - 2016-02-07
~~~~~~~~~~~~~~~~~

* Implement requirement specifiers from PEP 508.


16.0 - 2016-01-19
~~~~~~~~~~~~~~~~~

* Relicense so that packaging is available under *either* the Apache License,
  Version 2.0 or a 2 Clause BSD license.

* Support installation of packaging when only distutils is available.

* Fix ``==`` comparison when there is a prefix and a local version in play.
  (:issue:`41`).

* Implement environment markers from PEP 508.


15.3 - 2015-08-01
~~~~~~~~~~~~~~~~~

* Normalize post-release spellings for rev/r prefixes. :issue:`35`


15.2 - 2015-05-13
~~~~~~~~~~~~~~~~~

* Fix an error where the arbitrary specifier (``===``) was not correctly
  allowing pre-releases when it was being used.

* Expose the specifier and version parts through properties on the
  ``Specifier`` classes.

* Allow iterating over the ``SpecifierSet`` to get access to all of the
  ``Specifier`` instances.

* Allow testing if a version is contained within a specifier via the ``in``
  operator.


15.1 - 2015-04-13
~~~~~~~~~~~~~~~~~

* Fix a logic error that was causing inconsistent answers about whether or not
  a pre-release was contained within a ``SpecifierSet`` or not.


15.0 - 2015-01-02
~~~~~~~~~~~~~~~~~

* Add ``Version().is_postrelease`` and ``LegacyVersion().is_postrelease`` to
  make it easy to determine if a release is a post release.

* Add ``Version().base_version`` and ``LegacyVersion().base_version`` to make
  it easy to get the public version without any pre or post release markers.

* Support the update to PEP 440 which removed the implied ``!=V.*`` when using
  either ``>V`` or ``<V`` and which instead special cased the handling of
  pre-releases, post-releases, and local versions when using ``>V`` or ``<V``.


14.5 - 2014-12-17
~~~~~~~~~~~~~~~~~

* Normalize release candidates as ``rc`` instead of ``c``.

* Expose the ``VERSION_PATTERN`` constant, a regular expression matching
  a valid version.


14.4 - 2014-12-15
~~~~~~~~~~~~~~~~~

* Ensure that versions are normalized before comparison when used in a
  specifier with a less than (``<``) or greater than (``>``) operator.


14.3 - 2014-11-19
~~~~~~~~~~~~~~~~~

* **BACKWARDS INCOMPATIBLE** Refactor specifier support so that it can sanely
  handle legacy specifiers as well as PEP 440 specifiers.

* **BACKWARDS INCOMPATIBLE** Move the specifier support out of
  ``packaging.version`` into ``packaging.specifiers``.


14.2 - 2014-09-10
~~~~~~~~~~~~~~~~~

* Add prerelease support to ``Specifier``.
* Remove the ability to do ``item in Specifier()`` and replace it with
  ``Specifier().contains(item)`` in order to allow flags that signal if a
  prerelease should be accepted or not.
* Add a method ``Specifier().filter()`` which will take an iterable and returns
  an iterable with items that do not match the specifier filtered out.


14.1 - 2014-09-08
~~~~~~~~~~~~~~~~~

* Allow ``LegacyVersion`` and ``Version`` to be sorted together.
* Add ``packaging.version.parse()`` to enable easily parsing a version string
  as either a ``Version`` or a ``LegacyVersion`` depending on it's PEP 440
  validity.


14.0 - 2014-09-05
~~~~~~~~~~~~~~~~~

* Initial release.


.. _`master`: https://github.com/pypa/packaging/
