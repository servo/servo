Version Handling
================

.. currentmodule:: packaging.version

A core requirement of dealing with packages is the ability to work with
versions. `PEP 440`_ defines the standard version scheme for Python packages
which has been implemented by this module.

Usage
-----

.. doctest::

    >>> from packaging.version import Version, parse
    >>> v1 = parse("1.0a5")
    >>> v2 = Version("1.0")
    >>> v1
    <Version('1.0a5')>
    >>> v2
    <Version('1.0')>
    >>> v1 < v2
    True
    >>> v1.epoch
    0
    >>> v1.release
    (1, 0)
    >>> v1.pre
    ('a', 5)
    >>> v1.is_prerelease
    True
    >>> v2.is_prerelease
    False
    >>> Version("french toast")
    Traceback (most recent call last):
        ...
    InvalidVersion: Invalid version: 'french toast'
    >>> Version("1.0").post
    >>> Version("1.0").is_postrelease
    False
    >>> Version("1.0.post0").post
    0
    >>> Version("1.0.post0").is_postrelease
    True


Reference
---------

.. function:: parse(version)

    This function takes a version string and will parse it as a
    :class:`Version` if the version is a valid PEP 440 version, otherwise it
    will parse it as a deprecated :class:`LegacyVersion`.


.. class:: Version(version)

    This class abstracts handling of a project's versions. It implements the
    scheme defined in `PEP 440`_. A :class:`Version` instance is comparison
    aware and can be compared and sorted using the standard Python interfaces.

    :param str version: The string representation of a version which will be
                        parsed and normalized before use.
    :raises InvalidVersion: If the ``version`` does not conform to PEP 440 in
                            any way then this exception will be raised.

    .. attribute:: public

        A string representing the public version portion of this ``Version()``.

    .. attribute:: base_version

        A string representing the base version of this :class:`Version`
        instance. The base version is the public version of the project without
        any pre or post release markers.

    .. attribute:: epoch

        An integer giving the version epoch of this :class:`Version` instance

    .. attribute:: release

        A tuple of integers giving the components of the release segment of
        this :class:`Version` instance; that is, the ``1.2.3`` part of the
        version number, including trailing zeroes but not including the epoch
        or any prerelease/development/postrelease suffixes

    .. attribute:: major

        An integer representing the first item of :attr:`release` or ``0`` if unavailable.

    .. attribute:: minor

        An integer representing the second item of :attr:`release` or ``0`` if unavailable.

    .. attribute:: micro

        An integer representing the third item of :attr:`release` or ``0`` if unavailable.

    .. attribute:: local

        A string representing the local version portion of this ``Version()``
        if it has one, or ``None`` otherwise.

    .. attribute:: pre

        If this :class:`Version` instance represents a prerelease, this
        attribute will be a pair of the prerelease phase (the string ``"a"``,
        ``"b"``, or ``"rc"``) and the prerelease number (an integer).  If this
        instance is not a prerelease, the attribute will be `None`.

    .. attribute:: is_prerelease

        A boolean value indicating whether this :class:`Version` instance
        represents a prerelease and/or development release.

    .. attribute:: dev

        If this :class:`Version` instance represents a development release,
        this attribute will be the development release number (an integer);
        otherwise, it will be `None`.

    .. attribute:: is_devrelease

        A boolean value indicating whether this :class:`Version` instance
        represents a development release.

    .. attribute:: post

        If this :class:`Version` instance represents a postrelease, this
        attribute will be the postrelease number (an integer); otherwise, it
        will be `None`.

    .. attribute:: is_postrelease

        A boolean value indicating whether this :class:`Version` instance
        represents a post-release.


.. class:: LegacyVersion(version)

    .. deprecated:: 20.5

        Use :class:`Version` instead.

    This class abstracts handling of a project's versions if they are not
    compatible with the scheme defined in `PEP 440`_. It implements a similar
    interface to that of :class:`Version`.

    This class implements the previous de facto sorting algorithm used by
    setuptools, however it will always sort as less than a :class:`Version`
    instance.

    :param str version: The string representation of a version which will be
                        used as is.

    .. note::

        :class:`LegacyVersion` instances are always ordered lower than :class:`Version` instances.

        >>> from packaging.version import Version, LegacyVersion
        >>> v1 = Version("1.0")
        >>> v2 = LegacyVersion("1.0")
        >>> v1 > v2
        True
        >>> v3 = LegacyVersion("1.3")
        >>> v1 > v3
        True

        Also note that some strings are still valid PEP 440 strings (:class:`Version`), even if they look very similar to
        other versions that are not (:class:`LegacyVersion`). Examples include versions with `Pre-release spelling`_ and
        `Post-release spelling`_.

        >>> from packaging.version import parse
        >>> v1 = parse('0.9.8a')
        >>> v2 = parse('0.9.8beta')
        >>> v3 = parse('0.9.8r')
        >>> v4 = parse('0.9.8rev')
        >>> v5 = parse('0.9.8t')
        >>> v1
        <Version('0.9.8a0')>
        >>> v1.is_prerelease
        True
        >>> v2
        <Version('0.9.8b0')>
        >>> v2.is_prerelease
        True
        >>> v3
        <Version('0.9.8.post0')>
        >>> v3.is_postrelease
        True
        >>> v4
        <Version('0.9.8.post0')>
        >>> v4.is_postrelease
        True
        >>> v5
        <LegacyVersion('0.9.8t')>
        >>> v5.is_prerelease
        False
        >>> v5.is_postrelease
        False

    .. attribute:: public

        A string representing the public version portion of this
        :class:`LegacyVersion`. This will always be the entire version string.

    .. attribute:: base_version

        A string representing the base version portion of this
        :class:`LegacyVersion` instance. This will always be the entire version
        string.

    .. attribute:: epoch

        This will always be ``-1`` since without `PEP 440`_ we do not have the
        concept of version epochs.  The value reflects the fact that
        :class:`LegacyVersion` instances always compare less than
        :class:`Version` instances.

    .. attribute:: release

        This will always be ``None`` since without `PEP 440`_ we do not have
        the concept of a release segment or its components.  It exists
        primarily to allow a :class:`LegacyVersion` to be used as a stand in
        for a :class:`Version`.

    .. attribute:: local

        This will always be ``None`` since without `PEP 440`_ we do not have
        the concept of a local version. It exists primarily to allow a
        :class:`LegacyVersion` to be used as a stand in for a :class:`Version`.

    .. attribute:: pre

        This will always be ``None`` since without `PEP 440`_ we do not have
        the concept of a prerelease. It exists primarily to allow a
        :class:`LegacyVersion` to be used as a stand in for a :class:`Version`.

    .. attribute:: is_prerelease

        A boolean value indicating whether this :class:`LegacyVersion`
        represents a prerelease and/or development release.  Since without
        `PEP 440`_ there is no concept of pre or dev releases this will
        always be `False` and exists for compatibility with :class:`Version`.

    .. attribute:: dev

        This will always be ``None`` since without `PEP 440`_ we do not have
        the concept of a development release. It exists primarily to allow a
        :class:`LegacyVersion` to be used as a stand in for a :class:`Version`.

    .. attribute:: is_devrelease

        A boolean value indicating whether this :class:`LegacyVersion`
        represents a development release.  Since without `PEP 440`_ there is
        no concept of dev releases this will always be `False` and exists for
        compatibility with :class:`Version`.

    .. attribute:: post

        This will always be ``None`` since without `PEP 440`_ we do not have
        the concept of a postrelease. It exists primarily to allow a
        :class:`LegacyVersion` to be used as a stand in for a :class:`Version`.

    .. attribute:: is_postrelease

        A boolean value indicating whether this :class:`LegacyVersion`
        represents a post-release. Since without `PEP 440`_ there is no concept
        of post-releases this will always be ``False`` and exists for
        compatibility with :class:`Version`.


.. exception:: InvalidVersion

    Raised when attempting to create a :class:`Version` with a version string
    that does not conform to `PEP 440`_.


.. data:: VERSION_PATTERN

    A string containing the regular expression used to match a valid version.
    The pattern is not anchored at either end, and is intended for embedding
    in larger expressions (for example, matching a version number as part of
    a file name). The regular expression should be compiled with the
    ``re.VERBOSE`` and ``re.IGNORECASE`` flags set.


.. _PEP 440: https://www.python.org/dev/peps/pep-0440/
.. _Pre-release spelling : https://www.python.org/dev/peps/pep-0440/#pre-release-spelling
.. _Post-release spelling : https://www.python.org/dev/peps/pep-0440/#post-release-spelling
