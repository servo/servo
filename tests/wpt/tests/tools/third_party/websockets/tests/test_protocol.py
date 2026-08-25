import logging
import unittest.mock

from websockets.exceptions import (
    ConnectionClosedError,
    ConnectionClosedOK,
    InvalidState,
    PayloadTooBig,
    ProtocolError,
)
from websockets.frames import (
    OP_BINARY,
    OP_CLOSE,
    OP_CONT,
    OP_PING,
    OP_PONG,
    OP_TEXT,
    Close,
    CloseCode,
    Frame,
)
from websockets.protocol import *
from websockets.protocol import CLIENT, CLOSED, CLOSING, SERVER

from .extensions.utils import Rsv2Extension
from .test_frames import FramesTestCase


class ProtocolTestCase(FramesTestCase):
    def assertFrameSent(self, connection, frame, eof=False):
        """
        Outgoing data for ``connection`` contains the given frame.

        ``frame`` may be ``None`` if no frame is expected.

        When ``eof`` is ``True``, the end of the stream is also expected.

        """
        frames_sent = [
            None
            if write is SEND_EOF
            else self.parse(
                write,
                mask=connection.side is CLIENT,
                extensions=connection.extensions,
            )
            for write in connection.data_to_send()
        ]
        frames_expected = [] if frame is None else [frame]
        if eof:
            frames_expected += [None]
        self.assertEqual(frames_sent, frames_expected)

    def assertFrameReceived(self, connection, frame):
        """
        Incoming data for ``connection`` contains the given frame.

        ``frame`` may be ``None`` if no frame is expected.

        """
        frames_received = connection.events_received()
        frames_expected = [] if frame is None else [frame]
        self.assertEqual(frames_received, frames_expected)

    def assertConnectionClosing(self, connection, code=None, reason=""):
        """
        Incoming data caused the "Start the WebSocket Closing Handshake" process.

        """
        close_frame = Frame(
            OP_CLOSE,
            b"" if code is None else Close(code, reason).serialize(),
        )
        # A close frame was received.
        self.assertFrameReceived(connection, close_frame)
        # A close frame and possibly the end of stream were sent.
        self.assertFrameSent(connection, close_frame, eof=connection.side is SERVER)

    def assertConnectionFailing(self, connection, code=None, reason=""):
        """
        Incoming data caused the "Fail the WebSocket Connection" process.

        """
        close_frame = Frame(
            OP_CLOSE,
            b"" if code is None else Close(code, reason).serialize(),
        )
        # No frame was received.
        self.assertFrameReceived(connection, None)
        # A close frame and possibly the end of stream were sent.
        self.assertFrameSent(connection, close_frame, eof=connection.side is SERVER)


class MaskingTests(ProtocolTestCase):
    """
    Test frame masking.

    5.1.  Overview

    """

    unmasked_text_frame_date = b"\x81\x04Spam"
    masked_text_frame_data = b"\x81\x84\x00\xff\x00\xff\x53\x8f\x61\x92"

    def test_client_sends_masked_frame(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\xff\x00\xff"):
            client.send_text(b"Spam", True)
        self.assertEqual(client.data_to_send(), [self.masked_text_frame_data])

    def test_server_sends_unmasked_frame(self):
        server = Protocol(SERVER)
        server.send_text(b"Spam", True)
        self.assertEqual(server.data_to_send(), [self.unmasked_text_frame_date])

    def test_client_receives_unmasked_frame(self):
        client = Protocol(CLIENT)
        client.receive_data(self.unmasked_text_frame_date)
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, b"Spam"),
        )

    def test_server_receives_masked_frame(self):
        server = Protocol(SERVER)
        server.receive_data(self.masked_text_frame_data)
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, b"Spam"),
        )

    def test_client_receives_masked_frame(self):
        client = Protocol(CLIENT)
        client.receive_data(self.masked_text_frame_data)
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "incorrect masking")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "incorrect masking"
        )

    def test_server_receives_unmasked_frame(self):
        server = Protocol(SERVER)
        server.receive_data(self.unmasked_text_frame_date)
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "incorrect masking")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "incorrect masking"
        )


class ContinuationTests(ProtocolTestCase):
    """
    Test continuation frames without text or binary frames.

    """

    def test_client_sends_unexpected_continuation(self):
        client = Protocol(CLIENT)
        with self.assertRaises(ProtocolError) as raised:
            client.send_continuation(b"", fin=False)
        self.assertEqual(str(raised.exception), "unexpected continuation frame")

    def test_server_sends_unexpected_continuation(self):
        server = Protocol(SERVER)
        with self.assertRaises(ProtocolError) as raised:
            server.send_continuation(b"", fin=False)
        self.assertEqual(str(raised.exception), "unexpected continuation frame")

    def test_client_receives_unexpected_continuation(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x00\x00")
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "unexpected continuation frame")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "unexpected continuation frame"
        )

    def test_server_receives_unexpected_continuation(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x00\x80\x00\x00\x00\x00")
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "unexpected continuation frame")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "unexpected continuation frame"
        )

    def test_client_sends_continuation_after_sending_close(self):
        client = Protocol(CLIENT)
        # Since it isn't possible to send a close frame in a fragmented
        # message (see test_client_send_close_in_fragmented_message), in fact,
        # this is the same test as test_client_sends_unexpected_continuation.
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_close(CloseCode.GOING_AWAY)
        self.assertEqual(client.data_to_send(), [b"\x88\x82\x00\x00\x00\x00\x03\xe9"])
        with self.assertRaises(ProtocolError) as raised:
            client.send_continuation(b"", fin=False)
        self.assertEqual(str(raised.exception), "unexpected continuation frame")

    def test_server_sends_continuation_after_sending_close(self):
        # Since it isn't possible to send a close frame in a fragmented
        # message (see test_server_send_close_in_fragmented_message), in fact,
        # this is the same test as test_server_sends_unexpected_continuation.
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE)
        self.assertEqual(server.data_to_send(), [b"\x88\x02\x03\xe8"])
        with self.assertRaises(ProtocolError) as raised:
            server.send_continuation(b"", fin=False)
        self.assertEqual(str(raised.exception), "unexpected continuation frame")

    def test_client_receives_continuation_after_receiving_close(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x02\x03\xe8")
        self.assertConnectionClosing(client, CloseCode.NORMAL_CLOSURE)
        client.receive_data(b"\x00\x00")
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, None)

    def test_server_receives_continuation_after_receiving_close(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe9")
        self.assertConnectionClosing(server, CloseCode.GOING_AWAY)
        server.receive_data(b"\x00\x80\x00\xff\x00\xff")
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, None)


class TextTests(ProtocolTestCase):
    """
    Test text frames and continuation frames.

    """

    def test_client_sends_text(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_text("😀".encode())
        self.assertEqual(
            client.data_to_send(), [b"\x81\x84\x00\x00\x00\x00\xf0\x9f\x98\x80"]
        )

    def test_server_sends_text(self):
        server = Protocol(SERVER)
        server.send_text("😀".encode())
        self.assertEqual(server.data_to_send(), [b"\x81\x04\xf0\x9f\x98\x80"])

    def test_client_receives_text(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x81\x04\xf0\x9f\x98\x80")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, "😀".encode()),
        )

    def test_server_receives_text(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x81\x84\x00\x00\x00\x00\xf0\x9f\x98\x80")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, "😀".encode()),
        )

    def test_client_receives_text_over_size_limit(self):
        client = Protocol(CLIENT, max_size=3)
        client.receive_data(b"\x81\x04\xf0\x9f\x98\x80")
        self.assertIsInstance(client.parser_exc, PayloadTooBig)
        self.assertEqual(str(client.parser_exc), "over size limit (4 > 3 bytes)")
        self.assertConnectionFailing(
            client, CloseCode.MESSAGE_TOO_BIG, "over size limit (4 > 3 bytes)"
        )

    def test_server_receives_text_over_size_limit(self):
        server = Protocol(SERVER, max_size=3)
        server.receive_data(b"\x81\x84\x00\x00\x00\x00\xf0\x9f\x98\x80")
        self.assertIsInstance(server.parser_exc, PayloadTooBig)
        self.assertEqual(str(server.parser_exc), "over size limit (4 > 3 bytes)")
        self.assertConnectionFailing(
            server, CloseCode.MESSAGE_TOO_BIG, "over size limit (4 > 3 bytes)"
        )

    def test_client_receives_text_without_size_limit(self):
        client = Protocol(CLIENT, max_size=None)
        client.receive_data(b"\x81\x04\xf0\x9f\x98\x80")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, "😀".encode()),
        )

    def test_server_receives_text_without_size_limit(self):
        server = Protocol(SERVER, max_size=None)
        server.receive_data(b"\x81\x84\x00\x00\x00\x00\xf0\x9f\x98\x80")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, "😀".encode()),
        )

    def test_client_sends_fragmented_text(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_text("😀".encode()[:2], fin=False)
        self.assertEqual(client.data_to_send(), [b"\x01\x82\x00\x00\x00\x00\xf0\x9f"])
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_continuation("😀😀".encode()[2:6], fin=False)
        self.assertEqual(
            client.data_to_send(), [b"\x00\x84\x00\x00\x00\x00\x98\x80\xf0\x9f"]
        )
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_continuation("😀".encode()[2:], fin=True)
        self.assertEqual(client.data_to_send(), [b"\x80\x82\x00\x00\x00\x00\x98\x80"])

    def test_server_sends_fragmented_text(self):
        server = Protocol(SERVER)
        server.send_text("😀".encode()[:2], fin=False)
        self.assertEqual(server.data_to_send(), [b"\x01\x02\xf0\x9f"])
        server.send_continuation("😀😀".encode()[2:6], fin=False)
        self.assertEqual(server.data_to_send(), [b"\x00\x04\x98\x80\xf0\x9f"])
        server.send_continuation("😀".encode()[2:], fin=True)
        self.assertEqual(server.data_to_send(), [b"\x80\x02\x98\x80"])

    def test_client_receives_fragmented_text(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x01\x02\xf0\x9f")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, "😀".encode()[:2], fin=False),
        )
        client.receive_data(b"\x00\x04\x98\x80\xf0\x9f")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, "😀😀".encode()[2:6], fin=False),
        )
        client.receive_data(b"\x80\x02\x98\x80")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, "😀".encode()[2:]),
        )

    def test_server_receives_fragmented_text(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x01\x82\x00\x00\x00\x00\xf0\x9f")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, "😀".encode()[:2], fin=False),
        )
        server.receive_data(b"\x00\x84\x00\x00\x00\x00\x98\x80\xf0\x9f")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, "😀😀".encode()[2:6], fin=False),
        )
        server.receive_data(b"\x80\x82\x00\x00\x00\x00\x98\x80")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, "😀".encode()[2:]),
        )

    def test_client_receives_fragmented_text_over_size_limit(self):
        client = Protocol(CLIENT, max_size=3)
        client.receive_data(b"\x01\x02\xf0\x9f")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, "😀".encode()[:2], fin=False),
        )
        client.receive_data(b"\x80\x02\x98\x80")
        self.assertIsInstance(client.parser_exc, PayloadTooBig)
        self.assertEqual(str(client.parser_exc), "over size limit (2 > 1 bytes)")
        self.assertConnectionFailing(
            client, CloseCode.MESSAGE_TOO_BIG, "over size limit (2 > 1 bytes)"
        )

    def test_server_receives_fragmented_text_over_size_limit(self):
        server = Protocol(SERVER, max_size=3)
        server.receive_data(b"\x01\x82\x00\x00\x00\x00\xf0\x9f")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, "😀".encode()[:2], fin=False),
        )
        server.receive_data(b"\x80\x82\x00\x00\x00\x00\x98\x80")
        self.assertIsInstance(server.parser_exc, PayloadTooBig)
        self.assertEqual(str(server.parser_exc), "over size limit (2 > 1 bytes)")
        self.assertConnectionFailing(
            server, CloseCode.MESSAGE_TOO_BIG, "over size limit (2 > 1 bytes)"
        )

    def test_client_receives_fragmented_text_without_size_limit(self):
        client = Protocol(CLIENT, max_size=None)
        client.receive_data(b"\x01\x02\xf0\x9f")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, "😀".encode()[:2], fin=False),
        )
        client.receive_data(b"\x00\x04\x98\x80\xf0\x9f")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, "😀😀".encode()[2:6], fin=False),
        )
        client.receive_data(b"\x80\x02\x98\x80")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, "😀".encode()[2:]),
        )

    def test_server_receives_fragmented_text_without_size_limit(self):
        server = Protocol(SERVER, max_size=None)
        server.receive_data(b"\x01\x82\x00\x00\x00\x00\xf0\x9f")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, "😀".encode()[:2], fin=False),
        )
        server.receive_data(b"\x00\x84\x00\x00\x00\x00\x98\x80\xf0\x9f")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, "😀😀".encode()[2:6], fin=False),
        )
        server.receive_data(b"\x80\x82\x00\x00\x00\x00\x98\x80")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, "😀".encode()[2:]),
        )

    def test_client_sends_unexpected_text(self):
        client = Protocol(CLIENT)
        client.send_text(b"", fin=False)
        with self.assertRaises(ProtocolError) as raised:
            client.send_text(b"", fin=False)
        self.assertEqual(str(raised.exception), "expected a continuation frame")

    def test_server_sends_unexpected_text(self):
        server = Protocol(SERVER)
        server.send_text(b"", fin=False)
        with self.assertRaises(ProtocolError) as raised:
            server.send_text(b"", fin=False)
        self.assertEqual(str(raised.exception), "expected a continuation frame")

    def test_client_receives_unexpected_text(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x01\x00")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, b"", fin=False),
        )
        client.receive_data(b"\x01\x00")
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "expected a continuation frame")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "expected a continuation frame"
        )

    def test_server_receives_unexpected_text(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x01\x80\x00\x00\x00\x00")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, b"", fin=False),
        )
        server.receive_data(b"\x01\x80\x00\x00\x00\x00")
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "expected a continuation frame")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "expected a continuation frame"
        )

    def test_client_sends_text_after_sending_close(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_close(CloseCode.GOING_AWAY)
        self.assertEqual(client.data_to_send(), [b"\x88\x82\x00\x00\x00\x00\x03\xe9"])
        with self.assertRaises(InvalidState):
            client.send_text(b"")

    def test_server_sends_text_after_sending_close(self):
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE)
        self.assertEqual(server.data_to_send(), [b"\x88\x02\x03\xe8"])
        with self.assertRaises(InvalidState):
            server.send_text(b"")

    def test_client_receives_text_after_receiving_close(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x02\x03\xe8")
        self.assertConnectionClosing(client, CloseCode.NORMAL_CLOSURE)
        client.receive_data(b"\x81\x00")
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, None)

    def test_server_receives_text_after_receiving_close(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe9")
        self.assertConnectionClosing(server, CloseCode.GOING_AWAY)
        server.receive_data(b"\x81\x80\x00\xff\x00\xff")
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, None)


class BinaryTests(ProtocolTestCase):
    """
    Test binary frames and continuation frames.

    """

    def test_client_sends_binary(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_binary(b"\x01\x02\xfe\xff")
        self.assertEqual(
            client.data_to_send(), [b"\x82\x84\x00\x00\x00\x00\x01\x02\xfe\xff"]
        )

    def test_server_sends_binary(self):
        server = Protocol(SERVER)
        server.send_binary(b"\x01\x02\xfe\xff")
        self.assertEqual(server.data_to_send(), [b"\x82\x04\x01\x02\xfe\xff"])

    def test_client_receives_binary(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x82\x04\x01\x02\xfe\xff")
        self.assertFrameReceived(
            client,
            Frame(OP_BINARY, b"\x01\x02\xfe\xff"),
        )

    def test_server_receives_binary(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x82\x84\x00\x00\x00\x00\x01\x02\xfe\xff")
        self.assertFrameReceived(
            server,
            Frame(OP_BINARY, b"\x01\x02\xfe\xff"),
        )

    def test_client_receives_binary_over_size_limit(self):
        client = Protocol(CLIENT, max_size=3)
        client.receive_data(b"\x82\x04\x01\x02\xfe\xff")
        self.assertIsInstance(client.parser_exc, PayloadTooBig)
        self.assertEqual(str(client.parser_exc), "over size limit (4 > 3 bytes)")
        self.assertConnectionFailing(
            client, CloseCode.MESSAGE_TOO_BIG, "over size limit (4 > 3 bytes)"
        )

    def test_server_receives_binary_over_size_limit(self):
        server = Protocol(SERVER, max_size=3)
        server.receive_data(b"\x82\x84\x00\x00\x00\x00\x01\x02\xfe\xff")
        self.assertIsInstance(server.parser_exc, PayloadTooBig)
        self.assertEqual(str(server.parser_exc), "over size limit (4 > 3 bytes)")
        self.assertConnectionFailing(
            server, CloseCode.MESSAGE_TOO_BIG, "over size limit (4 > 3 bytes)"
        )

    def test_client_sends_fragmented_binary(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_binary(b"\x01\x02", fin=False)
        self.assertEqual(client.data_to_send(), [b"\x02\x82\x00\x00\x00\x00\x01\x02"])
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_continuation(b"\xee\xff\x01\x02", fin=False)
        self.assertEqual(
            client.data_to_send(), [b"\x00\x84\x00\x00\x00\x00\xee\xff\x01\x02"]
        )
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_continuation(b"\xee\xff", fin=True)
        self.assertEqual(client.data_to_send(), [b"\x80\x82\x00\x00\x00\x00\xee\xff"])

    def test_server_sends_fragmented_binary(self):
        server = Protocol(SERVER)
        server.send_binary(b"\x01\x02", fin=False)
        self.assertEqual(server.data_to_send(), [b"\x02\x02\x01\x02"])
        server.send_continuation(b"\xee\xff\x01\x02", fin=False)
        self.assertEqual(server.data_to_send(), [b"\x00\x04\xee\xff\x01\x02"])
        server.send_continuation(b"\xee\xff", fin=True)
        self.assertEqual(server.data_to_send(), [b"\x80\x02\xee\xff"])

    def test_client_receives_fragmented_binary(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x02\x02\x01\x02")
        self.assertFrameReceived(
            client,
            Frame(OP_BINARY, b"\x01\x02", fin=False),
        )
        client.receive_data(b"\x00\x04\xfe\xff\x01\x02")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, b"\xfe\xff\x01\x02", fin=False),
        )
        client.receive_data(b"\x80\x02\xfe\xff")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, b"\xfe\xff"),
        )

    def test_server_receives_fragmented_binary(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x02\x82\x00\x00\x00\x00\x01\x02")
        self.assertFrameReceived(
            server,
            Frame(OP_BINARY, b"\x01\x02", fin=False),
        )
        server.receive_data(b"\x00\x84\x00\x00\x00\x00\xee\xff\x01\x02")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, b"\xee\xff\x01\x02", fin=False),
        )
        server.receive_data(b"\x80\x82\x00\x00\x00\x00\xfe\xff")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, b"\xfe\xff"),
        )

    def test_client_receives_fragmented_binary_over_size_limit(self):
        client = Protocol(CLIENT, max_size=3)
        client.receive_data(b"\x02\x02\x01\x02")
        self.assertFrameReceived(
            client,
            Frame(OP_BINARY, b"\x01\x02", fin=False),
        )
        client.receive_data(b"\x80\x02\xfe\xff")
        self.assertIsInstance(client.parser_exc, PayloadTooBig)
        self.assertEqual(str(client.parser_exc), "over size limit (2 > 1 bytes)")
        self.assertConnectionFailing(
            client, CloseCode.MESSAGE_TOO_BIG, "over size limit (2 > 1 bytes)"
        )

    def test_server_receives_fragmented_binary_over_size_limit(self):
        server = Protocol(SERVER, max_size=3)
        server.receive_data(b"\x02\x82\x00\x00\x00\x00\x01\x02")
        self.assertFrameReceived(
            server,
            Frame(OP_BINARY, b"\x01\x02", fin=False),
        )
        server.receive_data(b"\x80\x82\x00\x00\x00\x00\xfe\xff")
        self.assertIsInstance(server.parser_exc, PayloadTooBig)
        self.assertEqual(str(server.parser_exc), "over size limit (2 > 1 bytes)")
        self.assertConnectionFailing(
            server, CloseCode.MESSAGE_TOO_BIG, "over size limit (2 > 1 bytes)"
        )

    def test_client_sends_unexpected_binary(self):
        client = Protocol(CLIENT)
        client.send_binary(b"", fin=False)
        with self.assertRaises(ProtocolError) as raised:
            client.send_binary(b"", fin=False)
        self.assertEqual(str(raised.exception), "expected a continuation frame")

    def test_server_sends_unexpected_binary(self):
        server = Protocol(SERVER)
        server.send_binary(b"", fin=False)
        with self.assertRaises(ProtocolError) as raised:
            server.send_binary(b"", fin=False)
        self.assertEqual(str(raised.exception), "expected a continuation frame")

    def test_client_receives_unexpected_binary(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x02\x00")
        self.assertFrameReceived(
            client,
            Frame(OP_BINARY, b"", fin=False),
        )
        client.receive_data(b"\x02\x00")
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "expected a continuation frame")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "expected a continuation frame"
        )

    def test_server_receives_unexpected_binary(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x02\x80\x00\x00\x00\x00")
        self.assertFrameReceived(
            server,
            Frame(OP_BINARY, b"", fin=False),
        )
        server.receive_data(b"\x02\x80\x00\x00\x00\x00")
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "expected a continuation frame")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "expected a continuation frame"
        )

    def test_client_sends_binary_after_sending_close(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_close(CloseCode.GOING_AWAY)
        self.assertEqual(client.data_to_send(), [b"\x88\x82\x00\x00\x00\x00\x03\xe9"])
        with self.assertRaises(InvalidState):
            client.send_binary(b"")

    def test_server_sends_binary_after_sending_close(self):
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE)
        self.assertEqual(server.data_to_send(), [b"\x88\x02\x03\xe8"])
        with self.assertRaises(InvalidState):
            server.send_binary(b"")

    def test_client_receives_binary_after_receiving_close(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x02\x03\xe8")
        self.assertConnectionClosing(client, CloseCode.NORMAL_CLOSURE)
        client.receive_data(b"\x82\x00")
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, None)

    def test_server_receives_binary_after_receiving_close(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe9")
        self.assertConnectionClosing(server, CloseCode.GOING_AWAY)
        server.receive_data(b"\x82\x80\x00\xff\x00\xff")
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, None)


class CloseTests(ProtocolTestCase):
    """
    Test close frames.

    See RFC 6544:

    5.5.1. Close
    7.1.6.  The WebSocket Connection Close Reason
    7.1.7.  Fail the WebSocket Connection

    """

    def test_close_code(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x04\x03\xe8OK")
        client.receive_eof()
        self.assertEqual(client.close_code, CloseCode.NORMAL_CLOSURE)

    def test_close_reason(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x84\x00\x00\x00\x00\x03\xe8OK")
        server.receive_eof()
        self.assertEqual(server.close_reason, "OK")

    def test_close_code_not_provided(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x80\x00\x00\x00\x00")
        server.receive_eof()
        self.assertEqual(server.close_code, CloseCode.NO_STATUS_RCVD)

    def test_close_reason_not_provided(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x00")
        client.receive_eof()
        self.assertEqual(client.close_reason, "")

    def test_close_code_not_available(self):
        client = Protocol(CLIENT)
        client.receive_eof()
        self.assertEqual(client.close_code, CloseCode.ABNORMAL_CLOSURE)

    def test_close_reason_not_available(self):
        server = Protocol(SERVER)
        server.receive_eof()
        self.assertEqual(server.close_reason, "")

    def test_close_code_not_available_yet(self):
        server = Protocol(SERVER)
        self.assertIsNone(server.close_code)

    def test_close_reason_not_available_yet(self):
        client = Protocol(CLIENT)
        self.assertIsNone(client.close_reason)

    def test_client_sends_close(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x3c\x3c\x3c\x3c"):
            client.send_close()
        self.assertEqual(client.data_to_send(), [b"\x88\x80\x3c\x3c\x3c\x3c"])
        self.assertIs(client.state, CLOSING)

    def test_server_sends_close(self):
        server = Protocol(SERVER)
        server.send_close()
        self.assertEqual(server.data_to_send(), [b"\x88\x00"])
        self.assertIs(server.state, CLOSING)

    def test_client_receives_close(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x3c\x3c\x3c\x3c"):
            client.receive_data(b"\x88\x00")
        self.assertEqual(client.events_received(), [Frame(OP_CLOSE, b"")])
        self.assertEqual(client.data_to_send(), [b"\x88\x80\x3c\x3c\x3c\x3c"])
        self.assertIs(client.state, CLOSING)

    def test_server_receives_close(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        self.assertEqual(server.events_received(), [Frame(OP_CLOSE, b"")])
        self.assertEqual(server.data_to_send(), [b"\x88\x00", b""])
        self.assertIs(server.state, CLOSING)

    def test_client_sends_close_then_receives_close(self):
        # Client-initiated close handshake on the client side.
        client = Protocol(CLIENT)

        client.send_close()
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, Frame(OP_CLOSE, b""))

        client.receive_data(b"\x88\x00")
        self.assertFrameReceived(client, Frame(OP_CLOSE, b""))
        self.assertFrameSent(client, None)

        client.receive_eof()
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, None, eof=True)

    def test_server_sends_close_then_receives_close(self):
        # Server-initiated close handshake on the server side.
        server = Protocol(SERVER)

        server.send_close()
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, Frame(OP_CLOSE, b""))

        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        self.assertFrameReceived(server, Frame(OP_CLOSE, b""))
        self.assertFrameSent(server, None, eof=True)

        server.receive_eof()
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, None)

    def test_client_receives_close_then_sends_close(self):
        # Server-initiated close handshake on the client side.
        client = Protocol(CLIENT)

        client.receive_data(b"\x88\x00")
        self.assertFrameReceived(client, Frame(OP_CLOSE, b""))
        self.assertFrameSent(client, Frame(OP_CLOSE, b""))

        client.receive_eof()
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, None, eof=True)

    def test_server_receives_close_then_sends_close(self):
        # Client-initiated close handshake on the server side.
        server = Protocol(SERVER)

        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        self.assertFrameReceived(server, Frame(OP_CLOSE, b""))
        self.assertFrameSent(server, Frame(OP_CLOSE, b""), eof=True)

        server.receive_eof()
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, None)

    def test_client_sends_close_with_code(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_close(CloseCode.GOING_AWAY)
        self.assertEqual(client.data_to_send(), [b"\x88\x82\x00\x00\x00\x00\x03\xe9"])
        self.assertIs(client.state, CLOSING)

    def test_server_sends_close_with_code(self):
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE)
        self.assertEqual(server.data_to_send(), [b"\x88\x02\x03\xe8"])
        self.assertIs(server.state, CLOSING)

    def test_client_receives_close_with_code(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x02\x03\xe8")
        self.assertConnectionClosing(client, CloseCode.NORMAL_CLOSURE, "")
        self.assertIs(client.state, CLOSING)

    def test_server_receives_close_with_code(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe9")
        self.assertConnectionClosing(server, CloseCode.GOING_AWAY, "")
        self.assertIs(server.state, CLOSING)

    def test_client_sends_close_with_code_and_reason(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_close(CloseCode.GOING_AWAY, "going away")
        self.assertEqual(
            client.data_to_send(), [b"\x88\x8c\x00\x00\x00\x00\x03\xe9going away"]
        )
        self.assertIs(client.state, CLOSING)

    def test_server_sends_close_with_code_and_reason(self):
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE, "OK")
        self.assertEqual(server.data_to_send(), [b"\x88\x04\x03\xe8OK"])
        self.assertIs(server.state, CLOSING)

    def test_client_receives_close_with_code_and_reason(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x04\x03\xe8OK")
        self.assertConnectionClosing(client, CloseCode.NORMAL_CLOSURE, "OK")
        self.assertIs(client.state, CLOSING)

    def test_server_receives_close_with_code_and_reason(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x8c\x00\x00\x00\x00\x03\xe9going away")
        self.assertConnectionClosing(server, CloseCode.GOING_AWAY, "going away")
        self.assertIs(server.state, CLOSING)

    def test_client_sends_close_with_reason_only(self):
        client = Protocol(CLIENT)
        with self.assertRaises(ProtocolError) as raised:
            client.send_close(reason="going away")
        self.assertEqual(str(raised.exception), "cannot send a reason without a code")

    def test_server_sends_close_with_reason_only(self):
        server = Protocol(SERVER)
        with self.assertRaises(ProtocolError) as raised:
            server.send_close(reason="OK")
        self.assertEqual(str(raised.exception), "cannot send a reason without a code")

    def test_client_receives_close_with_truncated_code(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x01\x03")
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "close frame too short")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "close frame too short"
        )
        self.assertIs(client.state, CLOSING)

    def test_server_receives_close_with_truncated_code(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x81\x00\x00\x00\x00\x03")
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "close frame too short")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "close frame too short"
        )
        self.assertIs(server.state, CLOSING)

    def test_client_receives_close_with_non_utf8_reason(self):
        client = Protocol(CLIENT)

        client.receive_data(b"\x88\x04\x03\xe8\xff\xff")
        self.assertIsInstance(client.parser_exc, UnicodeDecodeError)
        self.assertEqual(
            str(client.parser_exc),
            "'utf-8' codec can't decode byte 0xff in position 0: invalid start byte",
        )
        self.assertConnectionFailing(
            client, CloseCode.INVALID_DATA, "invalid start byte at position 0"
        )
        self.assertIs(client.state, CLOSING)

    def test_server_receives_close_with_non_utf8_reason(self):
        server = Protocol(SERVER)

        server.receive_data(b"\x88\x84\x00\x00\x00\x00\x03\xe9\xff\xff")
        self.assertIsInstance(server.parser_exc, UnicodeDecodeError)
        self.assertEqual(
            str(server.parser_exc),
            "'utf-8' codec can't decode byte 0xff in position 0: invalid start byte",
        )
        self.assertConnectionFailing(
            server, CloseCode.INVALID_DATA, "invalid start byte at position 0"
        )
        self.assertIs(server.state, CLOSING)


class PingTests(ProtocolTestCase):
    """
    Test ping. See 5.5.2. Ping in RFC 6544.

    """

    def test_client_sends_ping(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x44\x88\xcc"):
            client.send_ping(b"")
        self.assertEqual(client.data_to_send(), [b"\x89\x80\x00\x44\x88\xcc"])

    def test_server_sends_ping(self):
        server = Protocol(SERVER)
        server.send_ping(b"")
        self.assertEqual(server.data_to_send(), [b"\x89\x00"])

    def test_client_receives_ping(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x89\x00")
        self.assertFrameReceived(
            client,
            Frame(OP_PING, b""),
        )
        self.assertFrameSent(
            client,
            Frame(OP_PONG, b""),
        )

    def test_server_receives_ping(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x89\x80\x00\x44\x88\xcc")
        self.assertFrameReceived(
            server,
            Frame(OP_PING, b""),
        )
        self.assertFrameSent(
            server,
            Frame(OP_PONG, b""),
        )

    def test_client_sends_ping_with_data(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x44\x88\xcc"):
            client.send_ping(b"\x22\x66\xaa\xee")
        self.assertEqual(
            client.data_to_send(), [b"\x89\x84\x00\x44\x88\xcc\x22\x22\x22\x22"]
        )

    def test_server_sends_ping_with_data(self):
        server = Protocol(SERVER)
        server.send_ping(b"\x22\x66\xaa\xee")
        self.assertEqual(server.data_to_send(), [b"\x89\x04\x22\x66\xaa\xee"])

    def test_client_receives_ping_with_data(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x89\x04\x22\x66\xaa\xee")
        self.assertFrameReceived(
            client,
            Frame(OP_PING, b"\x22\x66\xaa\xee"),
        )
        self.assertFrameSent(
            client,
            Frame(OP_PONG, b"\x22\x66\xaa\xee"),
        )

    def test_server_receives_ping_with_data(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x89\x84\x00\x44\x88\xcc\x22\x22\x22\x22")
        self.assertFrameReceived(
            server,
            Frame(OP_PING, b"\x22\x66\xaa\xee"),
        )
        self.assertFrameSent(
            server,
            Frame(OP_PONG, b"\x22\x66\xaa\xee"),
        )

    def test_client_sends_fragmented_ping_frame(self):
        client = Protocol(CLIENT)
        # This is only possible through a private API.
        with self.assertRaises(ProtocolError) as raised:
            client.send_frame(Frame(OP_PING, b"", fin=False))
        self.assertEqual(str(raised.exception), "fragmented control frame")

    def test_server_sends_fragmented_ping_frame(self):
        server = Protocol(SERVER)
        # This is only possible through a private API.
        with self.assertRaises(ProtocolError) as raised:
            server.send_frame(Frame(OP_PING, b"", fin=False))
        self.assertEqual(str(raised.exception), "fragmented control frame")

    def test_client_receives_fragmented_ping_frame(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x09\x00")
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "fragmented control frame")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "fragmented control frame"
        )

    def test_server_receives_fragmented_ping_frame(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x09\x80\x3c\x3c\x3c\x3c")
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "fragmented control frame")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "fragmented control frame"
        )

    def test_client_sends_ping_after_sending_close(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_close(CloseCode.GOING_AWAY)
        self.assertEqual(client.data_to_send(), [b"\x88\x82\x00\x00\x00\x00\x03\xe9"])
        # The spec says: "An endpoint MAY send a Ping frame any time (...)
        # before the connection is closed" but websockets doesn't support
        # sending a Ping frame after a Close frame.
        with self.assertRaises(InvalidState) as raised:
            client.send_ping(b"")
        self.assertEqual(
            str(raised.exception),
            "cannot write to a WebSocket in the CLOSING state",
        )

    def test_server_sends_ping_after_sending_close(self):
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE)
        self.assertEqual(server.data_to_send(), [b"\x88\x02\x03\xe8"])
        # The spec says: "An endpoint MAY send a Ping frame any time (...)
        # before the connection is closed" but websockets doesn't support
        # sending a Ping frame after a Close frame.
        with self.assertRaises(InvalidState) as raised:
            server.send_ping(b"")
        self.assertEqual(
            str(raised.exception),
            "cannot write to a WebSocket in the CLOSING state",
        )

    def test_client_receives_ping_after_receiving_close(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x02\x03\xe8")
        self.assertConnectionClosing(client, CloseCode.NORMAL_CLOSURE)
        client.receive_data(b"\x89\x04\x22\x66\xaa\xee")
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, None)

    def test_server_receives_ping_after_receiving_close(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe9")
        self.assertConnectionClosing(server, CloseCode.GOING_AWAY)
        server.receive_data(b"\x89\x84\x00\x44\x88\xcc\x22\x22\x22\x22")
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, None)


class PongTests(ProtocolTestCase):
    """
    Test pong frames. See 5.5.3. Pong in RFC 6544.

    """

    def test_client_sends_pong(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x44\x88\xcc"):
            client.send_pong(b"")
        self.assertEqual(client.data_to_send(), [b"\x8a\x80\x00\x44\x88\xcc"])

    def test_server_sends_pong(self):
        server = Protocol(SERVER)
        server.send_pong(b"")
        self.assertEqual(server.data_to_send(), [b"\x8a\x00"])

    def test_client_receives_pong(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x8a\x00")
        self.assertFrameReceived(
            client,
            Frame(OP_PONG, b""),
        )

    def test_server_receives_pong(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x8a\x80\x00\x44\x88\xcc")
        self.assertFrameReceived(
            server,
            Frame(OP_PONG, b""),
        )

    def test_client_sends_pong_with_data(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x44\x88\xcc"):
            client.send_pong(b"\x22\x66\xaa\xee")
        self.assertEqual(
            client.data_to_send(), [b"\x8a\x84\x00\x44\x88\xcc\x22\x22\x22\x22"]
        )

    def test_server_sends_pong_with_data(self):
        server = Protocol(SERVER)
        server.send_pong(b"\x22\x66\xaa\xee")
        self.assertEqual(server.data_to_send(), [b"\x8a\x04\x22\x66\xaa\xee"])

    def test_client_receives_pong_with_data(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x8a\x04\x22\x66\xaa\xee")
        self.assertFrameReceived(
            client,
            Frame(OP_PONG, b"\x22\x66\xaa\xee"),
        )

    def test_server_receives_pong_with_data(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x8a\x84\x00\x44\x88\xcc\x22\x22\x22\x22")
        self.assertFrameReceived(
            server,
            Frame(OP_PONG, b"\x22\x66\xaa\xee"),
        )

    def test_client_sends_fragmented_pong_frame(self):
        client = Protocol(CLIENT)
        # This is only possible through a private API.
        with self.assertRaises(ProtocolError) as raised:
            client.send_frame(Frame(OP_PONG, b"", fin=False))
        self.assertEqual(str(raised.exception), "fragmented control frame")

    def test_server_sends_fragmented_pong_frame(self):
        server = Protocol(SERVER)
        # This is only possible through a private API.
        with self.assertRaises(ProtocolError) as raised:
            server.send_frame(Frame(OP_PONG, b"", fin=False))
        self.assertEqual(str(raised.exception), "fragmented control frame")

    def test_client_receives_fragmented_pong_frame(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x0a\x00")
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "fragmented control frame")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "fragmented control frame"
        )

    def test_server_receives_fragmented_pong_frame(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x0a\x80\x3c\x3c\x3c\x3c")
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "fragmented control frame")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "fragmented control frame"
        )

    def test_client_sends_pong_after_sending_close(self):
        client = Protocol(CLIENT)
        with self.enforce_mask(b"\x00\x00\x00\x00"):
            client.send_close(CloseCode.GOING_AWAY)
        self.assertEqual(client.data_to_send(), [b"\x88\x82\x00\x00\x00\x00\x03\xe9"])
        # websockets doesn't support sending a Pong frame after a Close frame.
        with self.assertRaises(InvalidState):
            client.send_pong(b"")

    def test_server_sends_pong_after_sending_close(self):
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE)
        self.assertEqual(server.data_to_send(), [b"\x88\x02\x03\xe8"])
        # websockets doesn't support sending a Pong frame after a Close frame.
        with self.assertRaises(InvalidState):
            server.send_pong(b"")

    def test_client_receives_pong_after_receiving_close(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x02\x03\xe8")
        self.assertConnectionClosing(client, CloseCode.NORMAL_CLOSURE)
        client.receive_data(b"\x8a\x04\x22\x66\xaa\xee")
        self.assertFrameReceived(client, None)
        self.assertFrameSent(client, None)

    def test_server_receives_pong_after_receiving_close(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe9")
        self.assertConnectionClosing(server, CloseCode.GOING_AWAY)
        server.receive_data(b"\x8a\x84\x00\x44\x88\xcc\x22\x22\x22\x22")
        self.assertFrameReceived(server, None)
        self.assertFrameSent(server, None)


class FailTests(ProtocolTestCase):
    """
    Test failing the connection.

    See 7.1.7. Fail the WebSocket Connection in RFC 6544.

    """

    def test_client_stops_processing_frames_after_fail(self):
        client = Protocol(CLIENT)
        client.fail(CloseCode.PROTOCOL_ERROR)
        self.assertConnectionFailing(client, CloseCode.PROTOCOL_ERROR)
        client.receive_data(b"\x88\x02\x03\xea")
        self.assertFrameReceived(client, None)

    def test_server_stops_processing_frames_after_fail(self):
        server = Protocol(SERVER)
        server.fail(CloseCode.PROTOCOL_ERROR)
        self.assertConnectionFailing(server, CloseCode.PROTOCOL_ERROR)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xea")
        self.assertFrameReceived(server, None)


class FragmentationTests(ProtocolTestCase):
    """
    Test message fragmentation.

    See 5.4. Fragmentation in RFC 6544.

    """

    def test_client_send_ping_pong_in_fragmented_message(self):
        client = Protocol(CLIENT)
        client.send_text(b"Spam", fin=False)
        self.assertFrameSent(client, Frame(OP_TEXT, b"Spam", fin=False))
        client.send_ping(b"Ping")
        self.assertFrameSent(client, Frame(OP_PING, b"Ping"))
        client.send_continuation(b"Ham", fin=False)
        self.assertFrameSent(client, Frame(OP_CONT, b"Ham", fin=False))
        client.send_pong(b"Pong")
        self.assertFrameSent(client, Frame(OP_PONG, b"Pong"))
        client.send_continuation(b"Eggs", fin=True)
        self.assertFrameSent(client, Frame(OP_CONT, b"Eggs"))

    def test_server_send_ping_pong_in_fragmented_message(self):
        server = Protocol(SERVER)
        server.send_text(b"Spam", fin=False)
        self.assertFrameSent(server, Frame(OP_TEXT, b"Spam", fin=False))
        server.send_ping(b"Ping")
        self.assertFrameSent(server, Frame(OP_PING, b"Ping"))
        server.send_continuation(b"Ham", fin=False)
        self.assertFrameSent(server, Frame(OP_CONT, b"Ham", fin=False))
        server.send_pong(b"Pong")
        self.assertFrameSent(server, Frame(OP_PONG, b"Pong"))
        server.send_continuation(b"Eggs", fin=True)
        self.assertFrameSent(server, Frame(OP_CONT, b"Eggs"))

    def test_client_receive_ping_pong_in_fragmented_message(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x01\x04Spam")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, b"Spam", fin=False),
        )
        client.receive_data(b"\x89\x04Ping")
        self.assertFrameReceived(
            client,
            Frame(OP_PING, b"Ping"),
        )
        self.assertFrameSent(
            client,
            Frame(OP_PONG, b"Ping"),
        )
        client.receive_data(b"\x00\x03Ham")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, b"Ham", fin=False),
        )
        client.receive_data(b"\x8a\x04Pong")
        self.assertFrameReceived(
            client,
            Frame(OP_PONG, b"Pong"),
        )
        client.receive_data(b"\x80\x04Eggs")
        self.assertFrameReceived(
            client,
            Frame(OP_CONT, b"Eggs"),
        )

    def test_server_receive_ping_pong_in_fragmented_message(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x01\x84\x00\x00\x00\x00Spam")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, b"Spam", fin=False),
        )
        server.receive_data(b"\x89\x84\x00\x00\x00\x00Ping")
        self.assertFrameReceived(
            server,
            Frame(OP_PING, b"Ping"),
        )
        self.assertFrameSent(
            server,
            Frame(OP_PONG, b"Ping"),
        )
        server.receive_data(b"\x00\x83\x00\x00\x00\x00Ham")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, b"Ham", fin=False),
        )
        server.receive_data(b"\x8a\x84\x00\x00\x00\x00Pong")
        self.assertFrameReceived(
            server,
            Frame(OP_PONG, b"Pong"),
        )
        server.receive_data(b"\x80\x84\x00\x00\x00\x00Eggs")
        self.assertFrameReceived(
            server,
            Frame(OP_CONT, b"Eggs"),
        )

    def test_client_send_close_in_fragmented_message(self):
        client = Protocol(CLIENT)
        client.send_text(b"Spam", fin=False)
        self.assertFrameSent(client, Frame(OP_TEXT, b"Spam", fin=False))
        # The spec says: "An endpoint MUST be capable of handling control
        # frames in the middle of a fragmented message." However, since the
        # endpoint must not send a data frame after a close frame, a close
        # frame can't be "in the middle" of a fragmented message.
        with self.assertRaises(ProtocolError) as raised:
            client.send_close(CloseCode.GOING_AWAY)
        self.assertEqual(str(raised.exception), "expected a continuation frame")
        client.send_continuation(b"Eggs", fin=True)

    def test_server_send_close_in_fragmented_message(self):
        server = Protocol(CLIENT)
        server.send_text(b"Spam", fin=False)
        self.assertFrameSent(server, Frame(OP_TEXT, b"Spam", fin=False))
        # The spec says: "An endpoint MUST be capable of handling control
        # frames in the middle of a fragmented message." However, since the
        # endpoint must not send a data frame after a close frame, a close
        # frame can't be "in the middle" of a fragmented message.
        with self.assertRaises(ProtocolError) as raised:
            server.send_close(CloseCode.NORMAL_CLOSURE)
        self.assertEqual(str(raised.exception), "expected a continuation frame")

    def test_client_receive_close_in_fragmented_message(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x01\x04Spam")
        self.assertFrameReceived(
            client,
            Frame(OP_TEXT, b"Spam", fin=False),
        )
        # The spec says: "An endpoint MUST be capable of handling control
        # frames in the middle of a fragmented message." However, since the
        # endpoint must not send a data frame after a close frame, a close
        # frame can't be "in the middle" of a fragmented message.
        client.receive_data(b"\x88\x02\x03\xe8")
        self.assertIsInstance(client.parser_exc, ProtocolError)
        self.assertEqual(str(client.parser_exc), "incomplete fragmented message")
        self.assertConnectionFailing(
            client, CloseCode.PROTOCOL_ERROR, "incomplete fragmented message"
        )

    def test_server_receive_close_in_fragmented_message(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x01\x84\x00\x00\x00\x00Spam")
        self.assertFrameReceived(
            server,
            Frame(OP_TEXT, b"Spam", fin=False),
        )
        # The spec says: "An endpoint MUST be capable of handling control
        # frames in the middle of a fragmented message." However, since the
        # endpoint must not send a data frame after a close frame, a close
        # frame can't be "in the middle" of a fragmented message.
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe9")
        self.assertIsInstance(server.parser_exc, ProtocolError)
        self.assertEqual(str(server.parser_exc), "incomplete fragmented message")
        self.assertConnectionFailing(
            server, CloseCode.PROTOCOL_ERROR, "incomplete fragmented message"
        )


class EOFTests(ProtocolTestCase):
    """
    Test half-closes on connection termination.

    """

    def test_client_receives_eof(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x00")
        self.assertConnectionClosing(client)
        client.receive_eof()
        self.assertIs(client.state, CLOSED)

    def test_server_receives_eof(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        self.assertConnectionClosing(server)
        server.receive_eof()
        self.assertIs(server.state, CLOSED)

    def test_client_receives_eof_between_frames(self):
        client = Protocol(CLIENT)
        client.receive_eof()
        self.assertIsInstance(client.parser_exc, EOFError)
        self.assertEqual(str(client.parser_exc), "unexpected end of stream")
        self.assertIs(client.state, CLOSED)

    def test_server_receives_eof_between_frames(self):
        server = Protocol(SERVER)
        server.receive_eof()
        self.assertIsInstance(server.parser_exc, EOFError)
        self.assertEqual(str(server.parser_exc), "unexpected end of stream")
        self.assertIs(server.state, CLOSED)

    def test_client_receives_eof_inside_frame(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x81")
        client.receive_eof()
        self.assertIsInstance(client.parser_exc, EOFError)
        self.assertEqual(
            str(client.parser_exc),
            "stream ends after 1 bytes, expected 2 bytes",
        )
        self.assertIs(client.state, CLOSED)

    def test_server_receives_eof_inside_frame(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x81")
        server.receive_eof()
        self.assertIsInstance(server.parser_exc, EOFError)
        self.assertEqual(
            str(server.parser_exc),
            "stream ends after 1 bytes, expected 2 bytes",
        )
        self.assertIs(server.state, CLOSED)

    def test_client_receives_data_after_exception(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\xff\xff")
        self.assertConnectionFailing(client, CloseCode.PROTOCOL_ERROR, "invalid opcode")
        client.receive_data(b"\x00\x00")
        self.assertFrameSent(client, None)

    def test_server_receives_data_after_exception(self):
        server = Protocol(SERVER)
        server.receive_data(b"\xff\xff")
        self.assertConnectionFailing(server, CloseCode.PROTOCOL_ERROR, "invalid opcode")
        server.receive_data(b"\x00\x00")
        self.assertFrameSent(server, None)

    def test_client_receives_eof_after_exception(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\xff\xff")
        self.assertConnectionFailing(client, CloseCode.PROTOCOL_ERROR, "invalid opcode")
        client.receive_eof()
        self.assertFrameSent(client, None, eof=True)

    def test_server_receives_eof_after_exception(self):
        server = Protocol(SERVER)
        server.receive_data(b"\xff\xff")
        self.assertConnectionFailing(server, CloseCode.PROTOCOL_ERROR, "invalid opcode")
        server.receive_eof()
        self.assertFrameSent(server, None)

    def test_client_receives_data_and_eof_after_exception(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\xff\xff")
        self.assertConnectionFailing(client, CloseCode.PROTOCOL_ERROR, "invalid opcode")
        client.receive_data(b"\x00\x00")
        client.receive_eof()
        self.assertFrameSent(client, None, eof=True)

    def test_server_receives_data_and_eof_after_exception(self):
        server = Protocol(SERVER)
        server.receive_data(b"\xff\xff")
        self.assertConnectionFailing(server, CloseCode.PROTOCOL_ERROR, "invalid opcode")
        server.receive_data(b"\x00\x00")
        server.receive_eof()
        self.assertFrameSent(server, None)

    def test_client_receives_data_after_eof(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x00")
        self.assertConnectionClosing(client)
        client.receive_eof()
        with self.assertRaises(EOFError) as raised:
            client.receive_data(b"\x88\x00")
        self.assertEqual(str(raised.exception), "stream ended")

    def test_server_receives_data_after_eof(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        self.assertConnectionClosing(server)
        server.receive_eof()
        with self.assertRaises(EOFError) as raised:
            server.receive_data(b"\x88\x80\x00\x00\x00\x00")
        self.assertEqual(str(raised.exception), "stream ended")

    def test_client_receives_eof_after_eof(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x00")
        self.assertConnectionClosing(client)
        client.receive_eof()
        with self.assertRaises(EOFError) as raised:
            client.receive_eof()
        self.assertEqual(str(raised.exception), "stream ended")

    def test_server_receives_eof_after_eof(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        self.assertConnectionClosing(server)
        server.receive_eof()
        with self.assertRaises(EOFError) as raised:
            server.receive_eof()
        self.assertEqual(str(raised.exception), "stream ended")


class TCPCloseTests(ProtocolTestCase):
    """
    Test expectation of TCP close on connection termination.

    """

    def test_client_default(self):
        client = Protocol(CLIENT)
        self.assertFalse(client.close_expected())

    def test_server_default(self):
        server = Protocol(SERVER)
        self.assertFalse(server.close_expected())

    def test_client_sends_close(self):
        client = Protocol(CLIENT)
        client.send_close()
        self.assertTrue(client.close_expected())

    def test_server_sends_close(self):
        server = Protocol(SERVER)
        server.send_close()
        self.assertTrue(server.close_expected())

    def test_client_receives_close(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x00")
        self.assertTrue(client.close_expected())

    def test_client_receives_close_then_eof(self):
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x00")
        client.receive_eof()
        self.assertFalse(client.close_expected())

    def test_server_receives_close_then_eof(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        server.receive_eof()
        self.assertFalse(server.close_expected())

    def test_server_receives_close(self):
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x80\x3c\x3c\x3c\x3c")
        self.assertTrue(server.close_expected())

    def test_client_fails_connection(self):
        client = Protocol(CLIENT)
        client.fail(CloseCode.PROTOCOL_ERROR)
        self.assertTrue(client.close_expected())

    def test_server_fails_connection(self):
        server = Protocol(SERVER)
        server.fail(CloseCode.PROTOCOL_ERROR)
        self.assertTrue(server.close_expected())


class ConnectionClosedTests(ProtocolTestCase):
    """
    Test connection closed exception.

    """

    def test_client_sends_close_then_receives_close(self):
        # Client-initiated close handshake on the client side complete.
        client = Protocol(CLIENT)
        client.send_close(CloseCode.NORMAL_CLOSURE, "")
        client.receive_data(b"\x88\x02\x03\xe8")
        client.receive_eof()
        exc = client.close_exc
        self.assertIsInstance(exc, ConnectionClosedOK)
        self.assertEqual(exc.rcvd, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertEqual(exc.sent, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertFalse(exc.rcvd_then_sent)

    def test_server_sends_close_then_receives_close(self):
        # Server-initiated close handshake on the server side complete.
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE, "")
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe8")
        server.receive_eof()
        exc = server.close_exc
        self.assertIsInstance(exc, ConnectionClosedOK)
        self.assertEqual(exc.rcvd, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertEqual(exc.sent, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertFalse(exc.rcvd_then_sent)

    def test_client_receives_close_then_sends_close(self):
        # Server-initiated close handshake on the client side complete.
        client = Protocol(CLIENT)
        client.receive_data(b"\x88\x02\x03\xe8")
        client.receive_eof()
        exc = client.close_exc
        self.assertIsInstance(exc, ConnectionClosedOK)
        self.assertEqual(exc.rcvd, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertEqual(exc.sent, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertTrue(exc.rcvd_then_sent)

    def test_server_receives_close_then_sends_close(self):
        # Client-initiated close handshake on the server side complete.
        server = Protocol(SERVER)
        server.receive_data(b"\x88\x82\x00\x00\x00\x00\x03\xe8")
        server.receive_eof()
        exc = server.close_exc
        self.assertIsInstance(exc, ConnectionClosedOK)
        self.assertEqual(exc.rcvd, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertEqual(exc.sent, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertTrue(exc.rcvd_then_sent)

    def test_client_sends_close_then_receives_eof(self):
        # Client-initiated close handshake on the client side times out.
        client = Protocol(CLIENT)
        client.send_close(CloseCode.NORMAL_CLOSURE, "")
        client.receive_eof()
        exc = client.close_exc
        self.assertIsInstance(exc, ConnectionClosedError)
        self.assertIsNone(exc.rcvd)
        self.assertEqual(exc.sent, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertIsNone(exc.rcvd_then_sent)

    def test_server_sends_close_then_receives_eof(self):
        # Server-initiated close handshake on the server side times out.
        server = Protocol(SERVER)
        server.send_close(CloseCode.NORMAL_CLOSURE, "")
        server.receive_eof()
        exc = server.close_exc
        self.assertIsInstance(exc, ConnectionClosedError)
        self.assertIsNone(exc.rcvd)
        self.assertEqual(exc.sent, Close(CloseCode.NORMAL_CLOSURE, ""))
        self.assertIsNone(exc.rcvd_then_sent)

    def test_client_receives_eof(self):
        # Server-initiated close handshake on the client side times out.
        client = Protocol(CLIENT)
        client.receive_eof()
        exc = client.close_exc
        self.assertIsInstance(exc, ConnectionClosedError)
        self.assertIsNone(exc.rcvd)
        self.assertIsNone(exc.sent)
        self.assertIsNone(exc.rcvd_then_sent)

    def test_server_receives_eof(self):
        # Client-initiated close handshake on the server side times out.
        server = Protocol(SERVER)
        server.receive_eof()
        exc = server.close_exc
        self.assertIsInstance(exc, ConnectionClosedError)
        self.assertIsNone(exc.rcvd)
        self.assertIsNone(exc.sent)
        self.assertIsNone(exc.rcvd_then_sent)


class ErrorTests(ProtocolTestCase):
    """
    Test other error cases.

    """

    def test_client_hits_internal_error_reading_frame(self):
        client = Protocol(CLIENT)
        # This isn't supposed to happen, so we're simulating it.
        with unittest.mock.patch("struct.unpack", side_effect=RuntimeError("BOOM")):
            client.receive_data(b"\x81\x00")
            self.assertIsInstance(client.parser_exc, RuntimeError)
            self.assertEqual(str(client.parser_exc), "BOOM")
        self.assertConnectionFailing(client, CloseCode.INTERNAL_ERROR, "")

    def test_server_hits_internal_error_reading_frame(self):
        server = Protocol(SERVER)
        # This isn't supposed to happen, so we're simulating it.
        with unittest.mock.patch("struct.unpack", side_effect=RuntimeError("BOOM")):
            server.receive_data(b"\x81\x80\x00\x00\x00\x00")
            self.assertIsInstance(server.parser_exc, RuntimeError)
            self.assertEqual(str(server.parser_exc), "BOOM")
        self.assertConnectionFailing(server, CloseCode.INTERNAL_ERROR, "")


class ExtensionsTests(ProtocolTestCase):
    """
    Test how extensions affect frames.

    """

    def test_client_extension_encodes_frame(self):
        client = Protocol(CLIENT)
        client.extensions = [Rsv2Extension()]
        with self.enforce_mask(b"\x00\x44\x88\xcc"):
            client.send_ping(b"")
        self.assertEqual(client.data_to_send(), [b"\xa9\x80\x00\x44\x88\xcc"])

    def test_server_extension_encodes_frame(self):
        server = Protocol(SERVER)
        server.extensions = [Rsv2Extension()]
        server.send_ping(b"")
        self.assertEqual(server.data_to_send(), [b"\xa9\x00"])

    def test_client_extension_decodes_frame(self):
        client = Protocol(CLIENT)
        client.extensions = [Rsv2Extension()]
        client.receive_data(b"\xaa\x00")
        self.assertEqual(client.events_received(), [Frame(OP_PONG, b"")])

    def test_server_extension_decodes_frame(self):
        server = Protocol(SERVER)
        server.extensions = [Rsv2Extension()]
        server.receive_data(b"\xaa\x80\x00\x44\x88\xcc")
        self.assertEqual(server.events_received(), [Frame(OP_PONG, b"")])


class MiscTests(unittest.TestCase):
    def test_client_default_logger(self):
        client = Protocol(CLIENT)
        logger = logging.getLogger("websockets.client")
        self.assertIs(client.logger, logger)

    def test_server_default_logger(self):
        server = Protocol(SERVER)
        logger = logging.getLogger("websockets.server")
        self.assertIs(server.logger, logger)

    def test_client_custom_logger(self):
        logger = logging.getLogger("test")
        client = Protocol(CLIENT, logger=logger)
        self.assertIs(client.logger, logger)

    def test_server_custom_logger(self):
        logger = logging.getLogger("test")
        server = Protocol(SERVER, logger=logger)
        self.assertIs(server.logger, logger)
