Eventlet Example Server
=======================

This example is a basic HTTP/2 server written using the `eventlet`_ concurrent
networking framework. This example is notable for demonstrating how to
configure `PyOpenSSL`_, which `eventlet`_ uses for its TLS layer.

In terms of HTTP/2 functionality, this example is very simple: it returns the
request headers as a JSON document to the caller. It does not obey HTTP/2 flow
control, which is a flaw, but it is otherwise functional.

.. literalinclude:: ../../examples/eventlet/eventlet-server.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _eventlet: http://eventlet.net/
.. _PyOpenSSL: https://pyopenssl.readthedocs.org/en/stable/
