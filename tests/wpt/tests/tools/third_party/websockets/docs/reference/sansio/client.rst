Client (`Sans-I/O`_)
====================

.. _Sans-I/O: https://sans-io.readthedocs.io/

.. currentmodule:: websockets.client

.. autoclass:: ClientProtocol(wsuri, origin=None, extensions=None, subprotocols=None, state=State.CONNECTING, max_size=2 ** 20, logger=None)

    .. automethod:: receive_data

    .. automethod:: receive_eof

    .. automethod:: connect

    .. automethod:: send_request

    .. automethod:: send_continuation

    .. automethod:: send_text

    .. automethod:: send_binary

    .. automethod:: send_close

    .. automethod:: send_ping

    .. automethod:: send_pong

    .. automethod:: fail

    .. automethod:: events_received

    .. automethod:: data_to_send

    .. automethod:: close_expected

    WebSocket protocol objects also provide these attributes:

    .. autoattribute:: id

    .. autoattribute:: logger

    .. autoproperty:: state

    The following attributes are available after the opening handshake,
    once the WebSocket connection is open:

    .. autoattribute:: handshake_exc

    The following attributes are available after the closing handshake,
    once the WebSocket connection is closed:

    .. autoproperty:: close_code

    .. autoproperty:: close_reason

    .. autoproperty:: close_exc
