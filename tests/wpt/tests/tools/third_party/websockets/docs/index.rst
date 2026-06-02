websockets
==========

|licence| |version| |pyversions| |tests| |docs| |openssf|

.. |licence| image:: https://img.shields.io/pypi/l/websockets.svg
    :target: https://pypi.python.org/pypi/websockets

.. |version| image:: https://img.shields.io/pypi/v/websockets.svg
    :target: https://pypi.python.org/pypi/websockets

.. |pyversions| image:: https://img.shields.io/pypi/pyversions/websockets.svg
    :target: https://pypi.python.org/pypi/websockets

.. |tests| image:: https://img.shields.io/github/checks-status/python-websockets/websockets/main?label=tests
   :target: https://github.com/python-websockets/websockets/actions/workflows/tests.yml

.. |docs| image:: https://img.shields.io/readthedocs/websockets.svg
   :target: https://websockets.readthedocs.io/

.. |openssf| image:: https://bestpractices.coreinfrastructure.org/projects/6475/badge
   :target: https://bestpractices.coreinfrastructure.org/projects/6475

websockets is a library for building WebSocket_ servers and clients in Python
with a focus on correctness, simplicity, robustness, and performance.

.. _WebSocket: https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API

It supports several network I/O and control flow paradigms:

1. The default implementation builds upon :mod:`asyncio`, Python's standard
   asynchronous I/O framework. It provides an elegant coroutine-based API. It's
   ideal for servers that handle many clients concurrently.
2. The :mod:`threading` implementation is a good alternative for clients,
   especially if you aren't familiar with :mod:`asyncio`. It may also be used
   for servers that don't need to serve many clients.
3. The `Sans-I/O`_ implementation is designed for integrating in third-party
   libraries, typically application servers, in addition being used internally
   by websockets.

.. _Sans-I/O: https://sans-io.readthedocs.io/

Here's an echo server with the :mod:`asyncio` API:

.. literalinclude:: ../example/echo.py

Here's how a client sends and receives messages with the :mod:`threading` API:

.. literalinclude:: ../example/hello.py

Don't worry about the opening and closing handshakes, pings and pongs, or any
other behavior described in the WebSocket specification. websockets takes care
of this under the hood so you can focus on your application!

Also, websockets provides an interactive client:

.. code-block:: console

    $ python -m websockets ws://localhost:8765/
    Connected to ws://localhost:8765/.
    > Hello world!
    < Hello world!
    Connection closed: 1000 (OK).

Do you like it? :doc:`Let's dive in! <intro/index>`

.. toctree::
   :hidden:

   intro/index
   howto/index
   faq/index
   reference/index
   topics/index
   project/index
