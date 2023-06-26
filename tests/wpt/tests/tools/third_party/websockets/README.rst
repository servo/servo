.. image:: logo/horizontal.svg
   :width: 480px
   :alt: websockets

|rtd| |pypi-v| |pypi-pyversions| |pypi-l| |pypi-wheel| |circleci| |codecov|

.. |rtd| image:: https://readthedocs.org/projects/websockets/badge/?version=latest
   :target: https://websockets.readthedocs.io/

.. |pypi-v| image:: https://img.shields.io/pypi/v/websockets.svg
    :target: https://pypi.python.org/pypi/websockets

.. |pypi-pyversions| image:: https://img.shields.io/pypi/pyversions/websockets.svg
    :target: https://pypi.python.org/pypi/websockets

.. |pypi-l| image:: https://img.shields.io/pypi/l/websockets.svg
    :target: https://pypi.python.org/pypi/websockets

.. |pypi-wheel| image:: https://img.shields.io/pypi/wheel/websockets.svg
    :target: https://pypi.python.org/pypi/websockets

.. |circleci| image:: https://img.shields.io/circleci/project/github/aaugustin/websockets.svg
   :target: https://circleci.com/gh/aaugustin/websockets

.. |codecov| image:: https://codecov.io/gh/aaugustin/websockets/branch/master/graph/badge.svg
    :target: https://codecov.io/gh/aaugustin/websockets

What is ``websockets``?
-----------------------

``websockets`` is a library for building WebSocket servers_ and clients_ in
Python with a focus on correctness and simplicity.

.. _servers: https://github.com/aaugustin/websockets/blob/master/example/server.py
.. _clients: https://github.com/aaugustin/websockets/blob/master/example/client.py

Built on top of ``asyncio``, Python's standard asynchronous I/O framework, it
provides an elegant coroutine-based API.

`Documentation is available on Read the Docs. <https://websockets.readthedocs.io/>`_

Here's how a client sends and receives messages:

.. copy-pasted because GitHub doesn't support the include directive

.. code:: python

    #!/usr/bin/env python

    import asyncio
    import websockets

    async def hello(uri):
        async with websockets.connect(uri) as websocket:
            await websocket.send("Hello world!")
            await websocket.recv()

    asyncio.get_event_loop().run_until_complete(
        hello('ws://localhost:8765'))

And here's an echo server:

.. code:: python

    #!/usr/bin/env python

    import asyncio
    import websockets

    async def echo(websocket, path):
        async for message in websocket:
            await websocket.send(message)

    asyncio.get_event_loop().run_until_complete(
        websockets.serve(echo, 'localhost', 8765))
    asyncio.get_event_loop().run_forever()

Does that look good?

`Get started with the tutorial! <https://websockets.readthedocs.io/en/stable/intro.html>`_

.. raw:: html

    <hr>
    <img align="left" height="150" width="150" src="https://raw.githubusercontent.com/aaugustin/websockets/master/logo/tidelift.png">
    <h3 align="center"><i>websockets for enterprise</i></h3>
    <p align="center"><i>Available as part of the Tidelift Subscription</i></p>
    <p align="center"><i>The maintainers of websockets and thousands of other packages are working with Tidelift to deliver commercial support and maintenance for the open source dependencies you use to build your applications. Save time, reduce risk, and improve code health, while paying the maintainers of the exact dependencies you use. <a href="https://tidelift.com/subscription/pkg/pypi-websockets?utm_source=pypi-websockets&utm_medium=referral&utm_campaign=readme">Learn more.</a></i></p>
    <hr>
    <p>(If you contribute to <code>websockets</code> and would like to become an official support provider, <a href="https://fractalideas.com/">let me know</a>.)</p>

Why should I use ``websockets``?
--------------------------------

The development of ``websockets`` is shaped by four principles:

1. **Simplicity**: all you need to understand is ``msg = await ws.recv()`` and
   ``await ws.send(msg)``; ``websockets`` takes care of managing connections
   so you can focus on your application.

2. **Robustness**: ``websockets`` is built for production; for example it was
   the only library to `handle backpressure correctly`_ before the issue
   became widely known in the Python community.

3. **Quality**: ``websockets`` is heavily tested. Continuous integration fails
   under 100% branch coverage. Also it passes the industry-standard `Autobahn
   Testsuite`_.

4. **Performance**: memory use is configurable. An extension written in C
   accelerates expensive operations. It's pre-compiled for Linux, macOS and
   Windows and packaged in the wheel format for each system and Python version.

Documentation is a first class concern in the project. Head over to `Read the
Docs`_ and see for yourself.

.. _Read the Docs: https://websockets.readthedocs.io/
.. _handle backpressure correctly: https://vorpus.org/blog/some-thoughts-on-asynchronous-api-design-in-a-post-asyncawait-world/#websocket-servers
.. _Autobahn Testsuite: https://github.com/aaugustin/websockets/blob/master/compliance/README.rst

Why shouldn't I use ``websockets``?
-----------------------------------

* If you prefer callbacks over coroutines: ``websockets`` was created to
  provide the best coroutine-based API to manage WebSocket connections in
  Python. Pick another library for a callback-based API.
* If you're looking for a mixed HTTP / WebSocket library: ``websockets`` aims
  at being an excellent implementation of :rfc:`6455`: The WebSocket Protocol
  and :rfc:`7692`: Compression Extensions for WebSocket. Its support for HTTP
  is minimal — just enough for a HTTP health check.
* If you want to use Python 2: ``websockets`` builds upon ``asyncio`` which
  only works on Python 3. ``websockets`` requires Python ≥ 3.6.1.

What else?
----------

Bug reports, patches and suggestions are welcome!

To report a security vulnerability, please use the `Tidelift security
contact`_. Tidelift will coordinate the fix and disclosure.

.. _Tidelift security contact: https://tidelift.com/security

For anything else, please open an issue_ or send a `pull request`_.

.. _issue: https://github.com/aaugustin/websockets/issues/new
.. _pull request: https://github.com/aaugustin/websockets/compare/

Participants must uphold the `Contributor Covenant code of conduct`_.

.. _Contributor Covenant code of conduct: https://github.com/aaugustin/websockets/blob/master/CODE_OF_CONDUCT.md

``websockets`` is released under the `BSD license`_.

.. _BSD license: https://github.com/aaugustin/websockets/blob/master/LICENSE
