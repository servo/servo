Twisted Example Server
======================

This example is a basic HTTP/2 server written for the `Twisted`_ asynchronous
networking framework. This is a relatively fleshed out example, and in
particular it makes sure to obey HTTP/2 flow control rules.

This server differs from some of the other example servers by serving files,
rather than simply sending JSON responses. This makes the example lengthier,
but also brings it closer to a real-world use-case.

.. literalinclude:: ../../examples/twisted/twisted-server.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _Twisted: https://twistedmatrix.com/