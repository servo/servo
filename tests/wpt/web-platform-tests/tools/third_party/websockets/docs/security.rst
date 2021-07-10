Security
========

Encryption
----------

For production use, a server should require encrypted connections.

See this example of :ref:`encrypting connections with TLS
<secure-server-example>`.

Memory use
----------

.. warning::

    An attacker who can open an arbitrary number of connections will be able
    to perform a denial of service by memory exhaustion. If you're concerned
    by denial of service attacks, you must reject suspicious connections
    before they reach ``websockets``, typically in a reverse proxy.

With the default settings, opening a connection uses 325 KiB of memory.

Sending some highly compressed messages could use up to 128 MiB of memory
with an amplification factor of 1000 between network traffic and memory use.

Configuring a server to :ref:`optimize memory usage <memory-usage>` will
improve security in addition to improving performance.

Other limits
------------

``websockets`` implements additional limits on the amount of data it accepts
in order to minimize exposure to security vulnerabilities.

In the opening handshake, ``websockets`` limits the number of HTTP headers to
256 and the size of an individual header to 4096 bytes. These limits are 10 to
20 times larger than what's expected in standard use cases. They're hard-coded.
If you need to change them, monkey-patch the constants in ``websockets.http``.
