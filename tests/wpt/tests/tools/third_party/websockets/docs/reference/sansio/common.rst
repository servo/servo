:orphan:

Both sides  (`Sans-I/O`_)
=========================

.. _Sans-I/O: https://sans-io.readthedocs.io/

.. automodule:: websockets.protocol

.. autoclass:: Protocol(side, state=State.OPEN, max_size=2 ** 20, logger=None)

    .. automethod:: receive_data

    .. automethod:: receive_eof

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

    .. autoattribute:: id

    .. autoattribute:: logger

    .. autoproperty:: state

    .. autoproperty:: close_code

    .. autoproperty:: close_reason

    .. autoproperty:: close_exc

.. autoclass:: Side

    .. autoattribute:: SERVER

    .. autoattribute:: CLIENT

.. autoclass:: State

    .. autoattribute:: CONNECTING

    .. autoattribute:: OPEN

    .. autoattribute:: CLOSING

    .. autoattribute:: CLOSED

.. autodata:: SEND_EOF
