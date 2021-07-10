QUIC API
========

The QUIC API performs no I/O on its own, leaving this to the API user.
This allows you to integrate QUIC in any Python application, regardless of
the concurrency model you are using.

Connection
----------

.. automodule:: aioquic.quic.connection

    .. autoclass:: QuicConnection
        :members:


Configuration
-------------

.. automodule:: aioquic.quic.configuration

    .. autoclass:: QuicConfiguration
        :members:

.. automodule:: aioquic.quic.logger

    .. autoclass:: QuicLogger
        :members:

Events
------

.. automodule:: aioquic.quic.events

    .. autoclass:: QuicEvent
        :members:

    .. autoclass:: ConnectionTerminated
        :members:

    .. autoclass:: HandshakeCompleted
        :members:

    .. autoclass:: PingAcknowledged
        :members:

    .. autoclass:: StreamDataReceived
        :members:

    .. autoclass:: StreamReset
        :members:
