Performance
===========

Here are tips to optimize performance.

uvloop
------

You can make a websockets application faster by running it with uvloop_.

(This advice isn't specific to websockets. It applies to any :mod:`asyncio`
application.)

.. _uvloop: https://github.com/MagicStack/uvloop

broadcast
---------

:func:`~websockets.broadcast` is the most efficient way to send a message to
many clients.
