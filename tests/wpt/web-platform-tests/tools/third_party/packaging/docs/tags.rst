Tags
====

.. currentmodule:: packaging.tags

Wheels encode the Python interpreter, ABI, and platform that they support in
their filenames using `platform compatibility tags`_. This module provides
support for both parsing these tags as well as discovering what tags the
running Python interpreter supports.

Usage
-----

.. doctest::

    >>> from packaging.tags import Tag, sys_tags
    >>> import sys
    >>> looking_for = Tag("py{major}".format(major=sys.version_info.major), "none", "any")
    >>> supported_tags = list(sys_tags())
    >>> looking_for in supported_tags
    True
    >>> really_old = Tag("py1", "none", "any")
    >>> wheels = {really_old, looking_for}
    >>> best_wheel = None
    >>> for supported_tag in supported_tags:
    ...     for wheel_tag in wheels:
    ...         if supported_tag == wheel_tag:
    ...             best_wheel = wheel_tag
    ...             break
    >>> best_wheel == looking_for
    True

Reference
---------

High Level Interface
''''''''''''''''''''

The following functions are the main interface to the library, and are typically the only
items that applications should need to reference, in order to parse and check tags.

.. class:: Tag(interpreter, abi, platform)

    A representation of the tag triple for a wheel. Instances are considered
    immutable and thus are hashable. Equality checking is also supported.

    :param str interpreter: The interpreter name, e.g. ``"py"``
                            (see :attr:`INTERPRETER_SHORT_NAMES` for mapping
                            well-known interpreter names to their short names).
    :param str abi: The ABI that a wheel supports, e.g. ``"cp37m"``.
    :param str platform: The OS/platform the wheel supports,
                         e.g. ``"win_amd64"``.

    .. attribute:: interpreter

        The interpreter name.

    .. attribute:: abi

        The supported ABI.

    .. attribute:: platform

        The OS/platform.


.. function:: parse_tag(tag)

    Parses the provided ``tag`` into a set of :class:`Tag` instances.

    Returning a set is required due to the possibility that the tag is a
    `compressed tag set`_, e.g. ``"py2.py3-none-any"`` which supports both
    Python 2 and Python 3.

    :param str tag: The tag to parse, e.g. ``"py3-none-any"``.


.. function:: sys_tags(*, warn=False)

    Yields the tags that the running interpreter supports.

    The iterable is ordered so that the best-matching tag is first in the
    sequence. The exact preferential order to tags is interpreter-specific, but
    in general the tag importance is in the order of:

    1. Interpreter
    2. Platform
    3. ABI

    This order is due to the fact that an ABI is inherently tied to the
    platform, but platform-specific code is not necessarily tied to the ABI. The
    interpreter is the most important tag as it dictates basic support for any
    wheel.

    The function returns an iterable in order to allow for the possible
    short-circuiting of tag generation if the entire sequence is not necessary
    and tag calculation happens to be expensive.

    :param bool warn: Whether warnings should be logged. Defaults to ``False``.


Low Level Interface
'''''''''''''''''''

The following functions are low-level implementation details. They should typically not
be needed in application code, unless the application has specialised requirements (for
example, constructing sets of supported tags for environments other than the running
interpreter).

These functions capture the precise details of which environments support which tags. That
information is not defined in the compatibility tag standards but is noted as being up
to the implementation to provide.


.. attribute:: INTERPRETER_SHORT_NAMES

    A dictionary mapping interpreter names to their `abbreviation codes`_
    (e.g. ``"cpython"`` is ``"cp"``). All interpreter names are lower-case.


.. function:: interpreter_name()

    Returns the running interpreter's name.

    This typically acts as the prefix to the :attr:`~Tag.interpreter` tag.


.. function:: interpreter_version(*, warn=False)

    Returns the running interpreter's version.

    This typically acts as the suffix to the :attr:`~Tag.interpreter` tag.


.. function:: mac_platforms(version=None, arch=None)

    Yields the :attr:`~Tag.platform` tags for macOS.

    :param tuple version: A two-item tuple presenting the version of macOS.
                          Defaults to the current system's version.
    :param str arch: The CPU architecture. Defaults to the architecture of the
                     current system, e.g. ``"x86_64"``.

    .. note::
        Equivalent support for the other major platforms is purposefully not
        provided:

        - On Windows, platform compatibility is statically specified
        - On Linux, code must be run on the system itself to determine
          compatibility


.. function:: platform_tags(version=None, arch=None)

    Yields the :attr:`~Tag.platform` tags for the running interpreter.


.. function:: compatible_tags(python_version=None, interpreter=None, platforms=None)

    Yields the tags for an interpreter compatible with the Python version
    specified by ``python_version``.

    The specific tags generated are:

    - ``py*-none-<platform>``
    - ``<interpreter>-none-any`` if ``interpreter`` is provided
    - ``py*-none-any``

    :param Sequence python_version: A one- or two-item sequence representing the
                                 compatible version of Python. Defaults to
                                 ``sys.version_info[:2]``.
    :param str interpreter: The name of the interpreter (if known), e.g.
                            ``"cp38"``. Defaults to the current interpreter.
    :param Iterable platforms: Iterable of compatible platforms. Defaults to the
                               platforms compatible with the current system.

.. function:: cpython_tags(python_version=None, abis=None, platforms=None, *, warn=False)

    Yields the tags for the CPython interpreter.

    The specific tags generated are:

    - ``cp<python_version>-<abi>-<platform>``
    - ``cp<python_version>-abi3-<platform>``
    - ``cp<python_version>-none-<platform>``
    - ``cp<older version>-abi3-<platform>`` where "older version" is all older
      minor versions down to Python 3.2 (when ``abi3`` was introduced)

    If ``python_version`` only provides a major-only version then only
    user-provided ABIs via ``abis`` and the ``none`` ABI will be used.

    :param Sequence python_version: A one- or two-item sequence representing the
                                 targeted Python version. Defaults to
                                 ``sys.version_info[:2]``.
    :param Iterable abis: Iterable of compatible ABIs. Defaults to the ABIs
                          compatible with the current system.
    :param Iterable platforms: Iterable of compatible platforms. Defaults to the
                               platforms compatible with the current system.
    :param bool warn: Whether warnings should be logged. Defaults to ``False``.

.. function:: generic_tags(interpreter=None, abis=None, platforms=None, *, warn=False)

    Yields the tags for an interpreter which requires no specialization.

    This function should be used if one of the other interpreter-specific
    functions provided by this module is not appropriate (i.e. not calculating
    tags for a CPython interpreter).

    The specific tags generated are:

    - ``<interpreter>-<abi>-<platform>``

    The ``"none"`` ABI will be added if it was not explicitly provided.

    :param str interpreter: The name of the interpreter. Defaults to being
                            calculated.
    :param Iterable abis: Iterable of compatible ABIs. Defaults to the ABIs
                          compatible with the current system.
    :param Iterable platforms: Iterable of compatible platforms. Defaults to the
                               platforms compatible with the current system.
    :param bool warn: Whether warnings should be logged. Defaults to ``False``.

.. _`abbreviation codes`: https://www.python.org/dev/peps/pep-0425/#python-tag
.. _`compressed tag set`: https://www.python.org/dev/peps/pep-0425/#compressed-tag-sets
.. _`platform compatibility tags`: https://packaging.python.org/specifications/platform-compatibility-tags/
