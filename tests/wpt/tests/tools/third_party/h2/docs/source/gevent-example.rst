Gevent Example Server
=====================

This example is a basic HTTP/2 server written using `gevent`_, a powerful
coroutine-based Python networking library that uses `greenlet`_
to provide a high-level synchronous API on top of the `libev`_ or `libuv`_
event loop.

This example is inspired by the curio one and also demonstrates the correct use
of HTTP/2 flow control with h2 and how gevent can be simple to use.

.. literalinclude:: ../../examples/gevent/gevent-server.py
   :language: python
   :linenos:
   :encoding: utf-8

.. _gevent: http://www.gevent.org/
.. _greenlet: https://greenlet.readthedocs.io/en/latest/
.. _libev: http://software.schmorp.de/pkg/libev.html
.. _libuv: http://libuv.org/