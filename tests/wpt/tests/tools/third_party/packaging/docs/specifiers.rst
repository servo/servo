Specifiers
==========

.. currentmodule:: packaging.specifiers

A core requirement of dealing with dependencies is the ability to specify what
versions of a dependency are acceptable for you. `PEP 440`_ defines the
standard specifier scheme which has been implemented by this module.

Usage
-----

.. doctest::

    >>> from packaging.specifiers import SpecifierSet
    >>> from packaging.version import Version
    >>> spec1 = SpecifierSet("~=1.0")
    >>> spec1
    <SpecifierSet('~=1.0')>
    >>> spec2 = SpecifierSet(">=1.0")
    >>> spec2
    <SpecifierSet('>=1.0')>
    >>> # We can combine specifiers
    >>> combined_spec = spec1 & spec2
    >>> combined_spec
    <SpecifierSet('>=1.0,~=1.0')>
    >>> # We can also implicitly combine a string specifier
    >>> combined_spec &= "!=1.1"
    >>> combined_spec
    <SpecifierSet('!=1.1,>=1.0,~=1.0')>
    >>> # Create a few versions to check for contains.
    >>> v1 = Version("1.0a5")
    >>> v2 = Version("1.0")
    >>> # We can check a version object to see if it falls within a specifier
    >>> v1 in combined_spec
    False
    >>> v2 in combined_spec
    True
    >>> # We can even do the same with a string based version
    >>> "1.4" in combined_spec
    True
    >>> # Finally we can filter a list of versions to get only those which are
    >>> # contained within our specifier.
    >>> list(combined_spec.filter([v1, v2, "1.4"]))
    [<Version('1.0')>, '1.4']


Reference
---------

.. class:: SpecifierSet(specifiers="", prereleases=None)

    This class abstracts handling specifying the dependencies of a project. It
    can be passed a single specifier (``>=3.0``), a comma-separated list of
    specifiers (``>=3.0,!=3.1``), or no specifier at all. Each individual
    specifier will be attempted to be parsed as a PEP 440 specifier
    (:class:`Specifier`) or as a legacy, setuptools style specifier
    (deprecated :class:`LegacySpecifier`). You may combine
    :class:`SpecifierSet` instances using the ``&`` operator
    (``SpecifierSet(">2") & SpecifierSet("<4")``).

    Both the membership tests and the combination support using raw strings
    in place of already instantiated objects.

    :param str specifiers: The string representation of a specifier or a
                           comma-separated list of specifiers which will
                           be parsed and normalized before use.
    :param bool prereleases: This tells the SpecifierSet if it should accept
                             prerelease versions if applicable or not. The
                             default of ``None`` will autodetect it from the
                             given specifiers.
    :raises InvalidSpecifier: If the given ``specifiers`` are not parseable
                              than this exception will be raised.

    .. attribute:: prereleases

        A boolean value indicating whether this :class:`SpecifierSet`
        represents a specifier that includes a pre-release versions. This can be
        set to either ``True`` or ``False`` to explicitly enable or disable
        prereleases or it can be set to ``None`` (the default) to enable
        autodetection.

    .. method:: __contains__(version)

        This is the more Pythonic version of :meth:`contains()`, but does
        not allow you to override the ``prereleases`` argument.  If you
        need that, use :meth:`contains()`.

        See :meth:`contains()`.

    .. method:: contains(version, prereleases=None)

        Determines if ``version``, which can be either a version string, a
        :class:`Version`, or a deprecated :class:`LegacyVersion` object, is
        contained within this set of specifiers.

        This will either match or not match prereleases based on the
        ``prereleases`` parameter. When ``prereleases`` is set to ``None``
        (the default) it will use the ``Specifier().prereleases`` attribute to
        determine if to allow them. Otherwise it will use the boolean value of
        the passed in value to determine if to allow them or not.

    .. method:: __len__()

        Returns the number of specifiers in this specifier set.

    .. method:: __iter__()

        Returns an iterator over all the underlying :class:`Specifier` (or
        deprecated :class:`LegacySpecifier`) instances in this specifier set.

    .. method:: filter(iterable, prereleases=None)

        Takes an iterable that can contain version strings, :class:`~.Version`,
        and deprecated :class:`~.LegacyVersion` instances and will then filter
        it, returning an iterable that contains only items which match the
        rules of this specifier object.

        This method is smarter than just
        ``filter(Specifier().contains, [...])`` because it implements the rule
        from PEP 440 where a prerelease item SHOULD be accepted if no other
        versions match the given specifier.

        The ``prereleases`` parameter functions similarly to that of the same
        parameter in ``contains``. If the value is ``None`` (the default) then
        it will intelligently decide if to allow prereleases based on the
        specifier, the ``Specifier().prereleases`` value, and the PEP 440
        rules. Otherwise it will act as a boolean which will enable or disable
        all prerelease versions from being included.


.. class:: Specifier(specifier, prereleases=None)

    This class abstracts the handling of a single `PEP 440`_ compatible
    specifier. It is generally not required to instantiate this manually,
    preferring instead to work with :class:`SpecifierSet`.

    :param str specifier: The string representation of a specifier which will
                          be parsed and normalized before use.
    :param bool prereleases: This tells the specifier if it should accept
                             prerelease versions if applicable or not. The
                             default of ``None`` will autodetect it from the
                             given specifiers.
    :raises InvalidSpecifier: If the ``specifier`` does not conform to PEP 440
                              in any way then this exception will be raised.

    .. attribute:: operator

        The string value of the operator part of this specifier.

    .. attribute:: version

        The string version of the version part of this specifier.

    .. attribute:: prereleases

        See :attr:`SpecifierSet.prereleases`.

    .. method:: __contains__(version)

        See :meth:`SpecifierSet.__contains__()`.

    .. method:: contains(version, prereleases=None)

        See :meth:`SpecifierSet.contains()`.

    .. method:: filter(iterable, prereleases=None)

        See :meth:`SpecifierSet.filter()`.


.. class:: LegacySpecifier(specifier, prereleases=None)

    .. deprecated:: 20.5

        Use :class:`Specifier` instead.

    This class abstracts the handling of a single legacy, setuptools style
    specifier. It is generally not required to instantiate this manually,
    preferring instead to work with :class:`SpecifierSet`.

    :param str specifier: The string representation of a specifier which will
                          be parsed and normalized before use.
    :param bool prereleases: This tells the specifier if it should accept
                             prerelease versions if applicable or not. The
                             default of ``None`` will autodetect it from the
                             given specifiers.
    :raises InvalidSpecifier: If the ``specifier`` is not parseable then this
                              will be raised.

    .. attribute:: operator

        The string value of the operator part of this specifier.

    .. attribute:: version

        The string version of the version part of this specifier.

    .. attribute:: prereleases

        See :attr:`SpecifierSet.prereleases`.

    .. method:: __contains__(version)

        See :meth:`SpecifierSet.__contains__()`.

    .. method:: contains(version, prereleases=None)

        See :meth:`SpecifierSet.contains()`.

    .. method:: filter(iterable, prereleases=None)

        See :meth:`SpecifierSet.filter()`.


.. exception:: InvalidSpecifier

    Raised when attempting to create a :class:`Specifier` with a specifier
    string that does not conform to `PEP 440`_.


.. _`PEP 440`: https://www.python.org/dev/peps/pep-0440/
