Utilities
=========

.. currentmodule:: packaging.utils


A set of small, helper utilities for dealing with Python packages.


Reference
---------

.. function:: canonicalize_name(name)

    This function takes a valid Python package name, and returns the normalized
    form of it.

    :param str name: The name to normalize.

    .. doctest::

        >>> from packaging.utils import canonicalize_name
        >>> canonicalize_name("Django")
        'django'
        >>> canonicalize_name("oslo.concurrency")
        'oslo-concurrency'
        >>> canonicalize_name("requests")
        'requests'

.. function:: canonicalize_version(version)

    This function takes a string representing a package version (or a
    ``Version`` instance), and returns the normalized form of it.

    :param str version: The version to normalize.

    .. doctest::

        >>> from packaging.utils import canonicalize_version
        >>> canonicalize_version('1.4.0.0.0')
        '1.4'
