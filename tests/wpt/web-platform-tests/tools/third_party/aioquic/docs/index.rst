aioquic
=======

|pypi-v| |pypi-pyversions| |pypi-l| |tests| |codecov|

.. |pypi-v| image:: https://img.shields.io/pypi/v/aioquic.svg
    :target: https://pypi.python.org/pypi/aioquic

.. |pypi-pyversions| image:: https://img.shields.io/pypi/pyversions/aioquic.svg
    :target: https://pypi.python.org/pypi/aioquic

.. |pypi-l| image:: https://img.shields.io/pypi/l/aioquic.svg
    :target: https://pypi.python.org/pypi/aioquic

.. |tests| image:: https://github.com/aiortc/aioquic/workflows/tests/badge.svg
    :target: https://github.com/aiortc/aioquic/actions

.. |codecov| image:: https://img.shields.io/codecov/c/github/aiortc/aioquic.svg
    :target: https://codecov.io/gh/aiortc/aioquic

``aioquic`` is a library for the QUIC network protocol in Python. It features several
APIs:

- a QUIC API following the "bring your own I/O" pattern, suitable for
  embedding in any framework,

- an HTTP/3 API which also follows the "bring your own I/O" pattern,

- a QUIC convenience API built on top of :mod:`asyncio`, Python's standard asynchronous
  I/O framework.

.. toctree::
   :maxdepth: 2

   design
   quic
   h3
   asyncio
   license
