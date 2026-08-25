Data structures
===============

WebSocket events
----------------

.. automodule:: websockets.frames

    .. autoclass:: Frame

    .. autoclass:: Opcode

        .. autoattribute:: CONT
        .. autoattribute:: TEXT
        .. autoattribute:: BINARY
        .. autoattribute:: CLOSE
        .. autoattribute:: PING
        .. autoattribute:: PONG

    .. autoclass:: Close

    .. autoclass:: CloseCode

        .. autoattribute:: NORMAL_CLOSURE
        .. autoattribute:: GOING_AWAY
        .. autoattribute:: PROTOCOL_ERROR
        .. autoattribute:: UNSUPPORTED_DATA
        .. autoattribute:: NO_STATUS_RCVD
        .. autoattribute:: ABNORMAL_CLOSURE
        .. autoattribute:: INVALID_DATA
        .. autoattribute:: POLICY_VIOLATION
        .. autoattribute:: MESSAGE_TOO_BIG
        .. autoattribute:: MANDATORY_EXTENSION
        .. autoattribute:: INTERNAL_ERROR
        .. autoattribute:: SERVICE_RESTART
        .. autoattribute:: TRY_AGAIN_LATER
        .. autoattribute:: BAD_GATEWAY
        .. autoattribute:: TLS_HANDSHAKE

HTTP events
-----------

.. automodule:: websockets.http11

    .. autoclass:: Request

    .. autoclass:: Response

.. automodule:: websockets.datastructures

    .. autoclass:: Headers

        .. automethod:: get_all

        .. automethod:: raw_items

    .. autoexception:: MultipleValuesError

URIs
----

.. automodule:: websockets.uri

    .. autofunction:: parse_uri

    .. autoclass:: WebSocketURI
