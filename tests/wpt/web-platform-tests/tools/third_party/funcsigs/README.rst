funcsigs
========

``funcsigs`` is a backport of the `PEP 362`_ function signature features from
Python 3.3's `inspect`_ module. The backport is compatible with Python 2.6, 2.7
as well as 3.2 and up.

|pypi_version|

Documentation
-------------

The reference documentation is standard library documentation for the
`inspect`_ module in Python3. This documentation has been included in the
``funcsigs`` package documentation hosted on `Read The Docs`_.

Example
-------

To obtain a signature object, pass the target function to the
``funcsigs.signature`` function. ::

    >>> from funcsigs import signature
    >>> def foo(a, b=None, *args, **kwargs):
    ...     pass

    >>> sig = signature(foo)

For the details of the signature object, refer to the either the package of
standard library documentation.

Compatability
-------------

The ``funcsigs`` backport has been tested against:

* CPython 2.6
* CPython 2.7
* CPython 3.2
* PyPy 1.9

Continuous integration testing is provided by `Travis CI`_.

Under Python 2.x there is a compatability issue when a function is assigned to
the ``__wrapped__`` property of a class after it has been constructed.
Similiarily there under PyPy directly passing the ``__call__`` method of a
builtin is also a compatability issues.  Otherwise the functionality is
believed to be uniform between both Python2 and Python3.

Issues
------

Source code for ``funcsigs`` is hosted on `GitHub`_. Any bug reports or feature
requests can be made using GitHub's `issues system`_. |build_status| |coverage|

Copyright
---------

This is a derived work of CPython under the terms of the `PSF License
Agreement`_. The original CPython inspect module, its unit tests and
documentation are the copyright of the Python Software Foundation. The derived
work is distributed under the `Apache License Version 2.0`_.

.. _Apache License Version 2.0: http://opensource.org/licenses/Apache-2.0
.. _GitHub: https://github.com/aliles/funcsigs
.. _PSF License Agreement: http://docs.python.org/3/license.html#terms-and-conditions-for-accessing-or-otherwise-using-python
.. _Travis CI: http://travis-ci.org/
.. _Read The Docs: http://funcsigs.readthedocs.org/
.. _PEP 362: http://www.python.org/dev/peps/pep-0362/
.. _inspect: http://docs.python.org/3/library/inspect.html#introspecting-callables-with-the-signature-object
.. _issues system: https://github.com/alies/funcsigs/issues

.. |build_status| image:: https://secure.travis-ci.org/aliles/funcsigs.png?branch=master
   :target: http://travis-ci.org/#!/aliles/funcsigs
   :alt: Current build status

.. |coverage| image:: https://coveralls.io/repos/aliles/funcsigs/badge.png?branch=master
   :target: https://coveralls.io/r/aliles/funcsigs?branch=master
   :alt: Coverage status

.. |pypi_version| image:: https://pypip.in/v/funcsigs/badge.png
   :target: https://crate.io/packages/funcsigs/
   :alt: Latest PyPI version
