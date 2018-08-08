Twisted Example Client: Head Requests
=====================================

This example is a basic HTTP/2 client written for the `Twisted`_ asynchronous
networking framework.

This client is fairly simple: it makes a hard-coded HEAD request to
http2bin.org and prints out the response data. Its purpose is to demonstrate
how to write a very basic HTTP/2 client implementation.

.. literalinclude:: ../../examples/twisted/head_request.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _Twisted: https://twistedmatrix.com/