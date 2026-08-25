Curio Example Server
====================

This example is a basic HTTP/2 server written using `curio`_, David Beazley's
example of how to build a concurrent networking framework using Python 3.5's
new ``async``/``await`` syntax.

This example is notable for demonstrating the correct use of HTTP/2 flow
control with h2. It is also a good example of the brand new syntax.

.. literalinclude:: ../../examples/curio/curio-server.py
   :language: python
   :linenos:
   :encoding: utf-8


.. _curio: https://curio.readthedocs.org/en/latest/
