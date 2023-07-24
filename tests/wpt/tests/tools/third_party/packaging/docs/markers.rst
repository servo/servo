Markers
=======

.. currentmodule:: packaging.markers

One extra requirement of dealing with dependencies is the ability to specify
if it is required depending on the operating system or Python version in use.
`PEP 508`_ defines the scheme which has been implemented by this module.

Usage
-----

.. doctest::

    >>> from packaging.markers import Marker, UndefinedEnvironmentName
    >>> marker = Marker("python_version>'2'")
    >>> marker
    <Marker('python_version > "2"')>
    >>> # We can evaluate the marker to see if it is satisfied
    >>> marker.evaluate()
    True
    >>> # We can also override the environment
    >>> env = {'python_version': '1.5.4'}
    >>> marker.evaluate(environment=env)
    False
    >>> # Multiple markers can be ANDed
    >>> and_marker = Marker("os_name=='a' and os_name=='b'")
    >>> and_marker
    <Marker('os_name == "a" and os_name == "b"')>
    >>> # Multiple markers can be ORed
    >>> or_marker = Marker("os_name=='a' or os_name=='b'")
    >>> or_marker
    <Marker('os_name == "a" or os_name == "b"')>
    >>> # Markers can be also used with extras, to pull in dependencies if
    >>> # a certain extra is being installed
    >>> extra = Marker('extra == "bar"')
    >>> # Evaluating an extra marker with no environment is an error
    >>> try:
    ...     extra.evaluate()
    ... except UndefinedEnvironmentName:
    ...     pass
    >>> extra_environment = {'extra': ''}
    >>> extra.evaluate(environment=extra_environment)
    False
    >>> extra_environment['extra'] = 'bar'
    >>> extra.evaluate(environment=extra_environment)
    True


Reference
---------

.. class:: Marker(markers)

    This class abstracts handling markers for dependencies of a project. It can
    be passed a single marker or multiple markers that are ANDed or ORed
    together. Each marker will be parsed according to PEP 508.

    :param str markers: The string representation of a marker or markers.
    :raises InvalidMarker: If the given ``markers`` are not parseable, then
                           this exception will be raised.

    .. method:: evaluate(environment=None)

    Evaluate the marker given the context of the current Python process.

    :param dict environment: A dictionary containing keys and values to
                             override the detected environment.
    :raises: UndefinedComparison: If the marker uses a PEP 440 comparison on
                                  strings which are not valid PEP 440 versions.
    :raises: UndefinedEnvironmentName: If the marker accesses a value that
                                       isn't present inside of the environment
                                       dictionary.

.. exception:: InvalidMarker

    Raised when attempting to create a :class:`Marker` with a string that
    does not conform to PEP 508.


.. exception:: UndefinedComparison

    Raised when attempting to evaluate a :class:`Marker` with a PEP 440
    comparison operator against values that are not valid PEP 440 versions.


.. exception:: UndefinedEnvironmentName

    Raised when attempting to evaluate a :class:`Marker` with a value that is
    missing from the evaluation environment.


.. _`PEP 508`: https://www.python.org/dev/peps/pep-0508/
