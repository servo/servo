Twisted Example Client: Post Requests
=====================================

This example is a basic HTTP/2 client written for the `Twisted`_ asynchronous
networking framework.

This client is fairly simple: it makes a hard-coded POST request to
http2bin.org and prints out the response data, sending a file that is provided
on the command line or the script itself. Its purpose is to demonstrate how to
write a HTTP/2 client implementation that handles flow control.

.. literalinclude:: ../../examples/twisted/post_request.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _Twisted: https://twistedmatrix.com/
