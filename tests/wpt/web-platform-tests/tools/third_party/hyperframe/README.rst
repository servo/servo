======================================
hyperframe: Pure-Python HTTP/2 framing
======================================

.. image:: https://travis-ci.org/python-hyper/hyperframe.png?branch=master
    :target: https://travis-ci.org/python-hyper/hyperframe

This library contains the HTTP/2 framing code used in the `hyper`_ project. It
provides a pure-Python codebase that is capable of decoding a binary stream
into HTTP/2 frames.

This library is used directly by `hyper`_ and a number of other projects to
provide HTTP/2 frame decoding logic.

Contributing
============

hyperframe welcomes contributions from anyone! Unlike many other projects we
are happy to accept cosmetic contributions and small contributions, in addition
to large feature requests and changes.

Before you contribute (either by opening an issue or filing a pull request),
please `read the contribution guidelines`_.

.. _read the contribution guidelines: http://hyper.readthedocs.org/en/development/contributing.html

License
=======

hyperframe is made available under the MIT License. For more details, see the
``LICENSE`` file in the repository.

Authors
=======

hyperframe is maintained by Cory Benfield, with contributions from others. For
more details about the contributors, please see ``CONTRIBUTORS.rst``.

.. _hyper: http://python-hyper.org/
