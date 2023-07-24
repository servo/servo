Tornado Example Server
======================

This example is a basic HTTP/2 server written using the `Tornado`_ asynchronous
networking library.

The server returns the request headers as a JSON document to the caller, just
like the example from the :doc:`basic-usage` document.

.. literalinclude:: ../../examples/tornado/tornado-server.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _Tornado: http://www.tornadoweb.org/
