Example HTTP/2-only WSGI Server
===============================

This example is a more complex HTTP/2 server that acts as a WSGI server,
passing data to an arbitrary WSGI application. This example is written using
`asyncio`_. The server supports most of PEP-3333, and so could in principle be
used as a production WSGI server: however, that's *not recommended* as certain
shortcuts have been taken to ensure ease of implementation and understanding.

The main advantages of this example are:

1. It properly demonstrates HTTP/2 flow control management.
2. It demonstrates how to plug hyper-h2 into a larger, more complex
   application.


.. literalinclude:: ../../examples/asyncio/wsgi-server.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _asyncio: https://docs.python.org/3/library/asyncio.html
