import contextlib
import logging
import platform
import socket
import sys
import threading
import time
import unittest
import uuid
from unittest.mock import patch

from websockets.exceptions import ConnectionClosedError, ConnectionClosedOK
from websockets.frames import CloseCode, Frame, Opcode
from websockets.protocol import CLIENT, SERVER, Protocol
from websockets.sync.connection import *

from ..protocol import RecordingProtocol
from ..utils import MS
from .connection import InterceptingConnection


# Connection implements symmetrical behavior between clients and servers.
# All tests run on the client side and the server side to validate this.


class ClientConnectionTests(unittest.TestCase):
    LOCAL = CLIENT
    REMOTE = SERVER

    def setUp(self):
        socket_, remote_socket = socket.socketpair()
        protocol = Protocol(self.LOCAL)
        remote_protocol = RecordingProtocol(self.REMOTE)
        self.connection = Connection(socket_, protocol, close_timeout=2 * MS)
        self.remote_connection = InterceptingConnection(remote_socket, remote_protocol)

    def tearDown(self):
        self.remote_connection.close()
        self.connection.close()

    # Test helpers built upon RecordingProtocol and InterceptingConnection.

    def assertFrameSent(self, frame):
        """Check that a single frame was sent."""
        time.sleep(MS)  # let the remote side process messages
        self.assertEqual(self.remote_connection.protocol.get_frames_rcvd(), [frame])

    def assertNoFrameSent(self):
        """Check that no frame was sent."""
        time.sleep(MS)  # let the remote side process messages
        self.assertEqual(self.remote_connection.protocol.get_frames_rcvd(), [])

    @contextlib.contextmanager
    def delay_frames_rcvd(self, delay):
        """Delay frames before they're received by the connection."""
        with self.remote_connection.delay_frames_sent(delay):
            yield
            time.sleep(MS)  # let the remote side process messages

    @contextlib.contextmanager
    def delay_eof_rcvd(self, delay):
        """Delay EOF before it's received by the connection."""
        with self.remote_connection.delay_eof_sent(delay):
            yield
            time.sleep(MS)  # let the remote side process messages

    @contextlib.contextmanager
    def drop_frames_rcvd(self):
        """Drop frames before they're received by the connection."""
        with self.remote_connection.drop_frames_sent():
            yield
            time.sleep(MS)  # let the remote side process messages

    @contextlib.contextmanager
    def drop_eof_rcvd(self):
        """Drop EOF before it's received by the connection."""
        with self.remote_connection.drop_eof_sent():
            yield
            time.sleep(MS)  # let the remote side process messages

    # Test __enter__ and __exit__.

    def test_enter(self):
        """__enter__ returns the connection itself."""
        with self.connection as connection:
            self.assertIs(connection, self.connection)

    def test_exit(self):
        """__exit__ closes the connection with code 1000."""
        with self.connection:
            self.assertNoFrameSent()
        self.assertFrameSent(Frame(Opcode.CLOSE, b"\x03\xe8"))

    def test_exit_with_exception(self):
        """__exit__ with an exception closes the connection with code 1011."""
        with self.assertRaises(RuntimeError):
            with self.connection:
                raise RuntimeError
        self.assertFrameSent(Frame(Opcode.CLOSE, b"\x03\xf3"))

    # Test __iter__.

    def test_iter_text(self):
        """__iter__ yields text messages."""
        iterator = iter(self.connection)
        self.remote_connection.send("😀")
        self.assertEqual(next(iterator), "😀")
        self.remote_connection.send("😀")
        self.assertEqual(next(iterator), "😀")

    def test_iter_binary(self):
        """__iter__ yields binary messages."""
        iterator = iter(self.connection)
        self.remote_connection.send(b"\x01\x02\xfe\xff")
        self.assertEqual(next(iterator), b"\x01\x02\xfe\xff")
        self.remote_connection.send(b"\x01\x02\xfe\xff")
        self.assertEqual(next(iterator), b"\x01\x02\xfe\xff")

    def test_iter_mixed(self):
        """__iter__ yields a mix of text and binary messages."""
        iterator = iter(self.connection)
        self.remote_connection.send("😀")
        self.assertEqual(next(iterator), "😀")
        self.remote_connection.send(b"\x01\x02\xfe\xff")
        self.assertEqual(next(iterator), b"\x01\x02\xfe\xff")

    def test_iter_connection_closed_ok(self):
        """__iter__ terminates after a normal closure."""
        iterator = iter(self.connection)
        self.remote_connection.close()
        with self.assertRaises(StopIteration):
            next(iterator)

    def test_iter_connection_closed_error(self):
        """__iter__ raises ConnnectionClosedError after an error."""
        iterator = iter(self.connection)
        self.remote_connection.close(code=CloseCode.INTERNAL_ERROR)
        with self.assertRaises(ConnectionClosedError):
            next(iterator)

    # Test recv.

    def test_recv_text(self):
        """recv receives a text message."""
        self.remote_connection.send("😀")
        self.assertEqual(self.connection.recv(), "😀")

    def test_recv_binary(self):
        """recv receives a binary message."""
        self.remote_connection.send(b"\x01\x02\xfe\xff")
        self.assertEqual(self.connection.recv(), b"\x01\x02\xfe\xff")

    def test_recv_fragmented_text(self):
        """recv receives a fragmented text message."""
        self.remote_connection.send(["😀", "😀"])
        self.assertEqual(self.connection.recv(), "😀😀")

    def test_recv_fragmented_binary(self):
        """recv receives a fragmented binary message."""
        self.remote_connection.send([b"\x01\x02", b"\xfe\xff"])
        self.assertEqual(self.connection.recv(), b"\x01\x02\xfe\xff")

    def test_recv_connection_closed_ok(self):
        """recv raises ConnectionClosedOK after a normal closure."""
        self.remote_connection.close()
        with self.assertRaises(ConnectionClosedOK):
            self.connection.recv()

    def test_recv_connection_closed_error(self):
        """recv raises ConnectionClosedError after an error."""
        self.remote_connection.close(code=CloseCode.INTERNAL_ERROR)
        with self.assertRaises(ConnectionClosedError):
            self.connection.recv()

    def test_recv_during_recv(self):
        """recv raises RuntimeError when called concurrently with itself."""
        recv_thread = threading.Thread(target=self.connection.recv)
        recv_thread.start()

        with self.assertRaisesRegex(
            RuntimeError,
            "cannot call recv while another thread "
            "is already running recv or recv_streaming",
        ):
            self.connection.recv()

        self.remote_connection.send("")
        recv_thread.join()

    def test_recv_during_recv_streaming(self):
        """recv raises RuntimeError when called concurrently with recv_streaming."""
        recv_streaming_thread = threading.Thread(
            target=lambda: list(self.connection.recv_streaming())
        )
        recv_streaming_thread.start()

        with self.assertRaisesRegex(
            RuntimeError,
            "cannot call recv while another thread "
            "is already running recv or recv_streaming",
        ):
            self.connection.recv()

        self.remote_connection.send("")
        recv_streaming_thread.join()

    # Test recv_streaming.

    def test_recv_streaming_text(self):
        """recv_streaming receives a text message."""
        self.remote_connection.send("😀")
        self.assertEqual(
            list(self.connection.recv_streaming()),
            ["😀"],
        )

    def test_recv_streaming_binary(self):
        """recv_streaming receives a binary message."""
        self.remote_connection.send(b"\x01\x02\xfe\xff")
        self.assertEqual(
            list(self.connection.recv_streaming()),
            [b"\x01\x02\xfe\xff"],
        )

    def test_recv_streaming_fragmented_text(self):
        """recv_streaming receives a fragmented text message."""
        self.remote_connection.send(["😀", "😀"])
        # websockets sends an trailing empty fragment. That's an implementation detail.
        self.assertEqual(
            list(self.connection.recv_streaming()),
            ["😀", "😀", ""],
        )

    def test_recv_streaming_fragmented_binary(self):
        """recv_streaming receives a fragmented binary message."""
        self.remote_connection.send([b"\x01\x02", b"\xfe\xff"])
        # websockets sends an trailing empty fragment. That's an implementation detail.
        self.assertEqual(
            list(self.connection.recv_streaming()),
            [b"\x01\x02", b"\xfe\xff", b""],
        )

    def test_recv_streaming_connection_closed_ok(self):
        """recv_streaming raises ConnectionClosedOK after a normal closure."""
        self.remote_connection.close()
        with self.assertRaises(ConnectionClosedOK):
            list(self.connection.recv_streaming())

    def test_recv_streaming_connection_closed_error(self):
        """recv_streaming raises ConnectionClosedError after an error."""
        self.remote_connection.close(code=CloseCode.INTERNAL_ERROR)
        with self.assertRaises(ConnectionClosedError):
            list(self.connection.recv_streaming())

    def test_recv_streaming_during_recv(self):
        """recv_streaming raises RuntimeError when called concurrently with recv."""
        recv_thread = threading.Thread(target=self.connection.recv)
        recv_thread.start()

        with self.assertRaisesRegex(
            RuntimeError,
            "cannot call recv_streaming while another thread "
            "is already running recv or recv_streaming",
        ):
            list(self.connection.recv_streaming())

        self.remote_connection.send("")
        recv_thread.join()

    def test_recv_streaming_during_recv_streaming(self):
        """recv_streaming raises RuntimeError when called concurrently with itself."""
        recv_streaming_thread = threading.Thread(
            target=lambda: list(self.connection.recv_streaming())
        )
        recv_streaming_thread.start()

        with self.assertRaisesRegex(
            RuntimeError,
            r"cannot call recv_streaming while another thread "
            r"is already running recv or recv_streaming",
        ):
            list(self.connection.recv_streaming())

        self.remote_connection.send("")
        recv_streaming_thread.join()

    # Test send.

    def test_send_text(self):
        """send sends a text message."""
        self.connection.send("😀")
        self.assertEqual(self.remote_connection.recv(), "😀")

    def test_send_binary(self):
        """send sends a binary message."""
        self.connection.send(b"\x01\x02\xfe\xff")
        self.assertEqual(self.remote_connection.recv(), b"\x01\x02\xfe\xff")

    def test_send_fragmented_text(self):
        """send sends a fragmented text message."""
        self.connection.send(["😀", "😀"])
        # websockets sends an trailing empty fragment. That's an implementation detail.
        self.assertEqual(
            list(self.remote_connection.recv_streaming()),
            ["😀", "😀", ""],
        )

    def test_send_fragmented_binary(self):
        """send sends a fragmented binary message."""
        self.connection.send([b"\x01\x02", b"\xfe\xff"])
        # websockets sends an trailing empty fragment. That's an implementation detail.
        self.assertEqual(
            list(self.remote_connection.recv_streaming()),
            [b"\x01\x02", b"\xfe\xff", b""],
        )

    def test_send_connection_closed_ok(self):
        """send raises ConnectionClosedOK after a normal closure."""
        self.remote_connection.close()
        with self.assertRaises(ConnectionClosedOK):
            self.connection.send("😀")

    def test_send_connection_closed_error(self):
        """send raises ConnectionClosedError after an error."""
        self.remote_connection.close(code=CloseCode.INTERNAL_ERROR)
        with self.assertRaises(ConnectionClosedError):
            self.connection.send("😀")

    def test_send_during_send(self):
        """send raises RuntimeError when called concurrently with itself."""
        recv_thread = threading.Thread(target=self.remote_connection.recv)
        recv_thread.start()

        send_gate = threading.Event()
        exit_gate = threading.Event()

        def fragments():
            yield "😀"
            send_gate.set()
            exit_gate.wait()
            yield "😀"

        send_thread = threading.Thread(
            target=self.connection.send,
            args=(fragments(),),
        )
        send_thread.start()

        send_gate.wait()
        # The check happens in four code paths, depending on the argument.
        for message in [
            "😀",
            b"\x01\x02\xfe\xff",
            ["😀", "😀"],
            [b"\x01\x02", b"\xfe\xff"],
        ]:
            with self.subTest(message=message):
                with self.assertRaisesRegex(
                    RuntimeError,
                    "cannot call send while another thread is already running send",
                ):
                    self.connection.send(message)

        exit_gate.set()
        send_thread.join()
        recv_thread.join()

    def test_send_empty_iterable(self):
        """send does nothing when called with an empty iterable."""
        self.connection.send([])
        self.connection.close()
        self.assertEqual(list(iter(self.remote_connection)), [])

    def test_send_mixed_iterable(self):
        """send raises TypeError when called with an iterable of inconsistent types."""
        with self.assertRaises(TypeError):
            self.connection.send(["😀", b"\xfe\xff"])

    def test_send_unsupported_iterable(self):
        """send raises TypeError when called with an iterable of unsupported type."""
        with self.assertRaises(TypeError):
            self.connection.send([None])

    def test_send_dict(self):
        """send raises TypeError when called with a dict."""
        with self.assertRaises(TypeError):
            self.connection.send({"type": "object"})

    def test_send_unsupported_type(self):
        """send raises TypeError when called with an unsupported type."""
        with self.assertRaises(TypeError):
            self.connection.send(None)

    # Test close.

    def test_close(self):
        """close sends a close frame."""
        self.connection.close()
        self.assertFrameSent(Frame(Opcode.CLOSE, b"\x03\xe8"))

    def test_close_explicit_code_reason(self):
        """close sends a close frame with a given code and reason."""
        self.connection.close(CloseCode.GOING_AWAY, "bye!")
        self.assertFrameSent(Frame(Opcode.CLOSE, b"\x03\xe9bye!"))

    def test_close_waits_for_close_frame(self):
        """close waits for a close frame (then EOF) before returning."""
        with self.delay_frames_rcvd(MS):
            self.connection.close()

        with self.assertRaises(ConnectionClosedOK) as raised:
            self.connection.recv()

        exc = raised.exception
        self.assertEqual(str(exc), "sent 1000 (OK); then received 1000 (OK)")
        self.assertIsNone(exc.__cause__)

    def test_close_waits_for_connection_closed(self):
        """close waits for EOF before returning."""
        if self.LOCAL is SERVER:
            self.skipTest("only relevant on the client-side")

        with self.delay_eof_rcvd(MS):
            self.connection.close()

        with self.assertRaises(ConnectionClosedOK) as raised:
            self.connection.recv()

        exc = raised.exception
        self.assertEqual(str(exc), "sent 1000 (OK); then received 1000 (OK)")
        self.assertIsNone(exc.__cause__)

    def test_close_timeout_waiting_for_close_frame(self):
        """close times out if no close frame is received."""
        with self.drop_eof_rcvd(), self.drop_frames_rcvd():
            self.connection.close()

        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.recv()

        exc = raised.exception
        self.assertEqual(str(exc), "sent 1000 (OK); no close frame received")
        self.assertIsInstance(exc.__cause__, TimeoutError)

    def test_close_timeout_waiting_for_connection_closed(self):
        """close times out if EOF isn't received."""
        if self.LOCAL is SERVER:
            self.skipTest("only relevant on the client-side")

        with self.drop_eof_rcvd():
            self.connection.close()

        with self.assertRaises(ConnectionClosedOK) as raised:
            self.connection.recv()

        exc = raised.exception
        self.assertEqual(str(exc), "sent 1000 (OK); then received 1000 (OK)")
        # Remove socket.timeout when dropping Python < 3.10.
        self.assertIsInstance(exc.__cause__, (socket.timeout, TimeoutError))

    def test_close_waits_for_recv(self):
        self.remote_connection.send("😀")

        close_thread = threading.Thread(target=self.connection.close)
        close_thread.start()

        # Let close() initiate the closing handshake and send a close frame.
        time.sleep(MS)
        self.assertTrue(close_thread.is_alive())

        # Connection isn't closed yet.
        self.connection.recv()

        # Let close() receive a close frame and finish the closing handshake.
        time.sleep(MS)
        self.assertFalse(close_thread.is_alive())

        # Connection is closed now.
        with self.assertRaises(ConnectionClosedOK) as raised:
            self.connection.recv()

        exc = raised.exception
        self.assertEqual(str(exc), "sent 1000 (OK); then received 1000 (OK)")
        self.assertIsNone(exc.__cause__)

    def test_close_timeout_waiting_for_recv(self):
        self.remote_connection.send("😀")

        close_thread = threading.Thread(target=self.connection.close)
        close_thread.start()

        # Let close() time out during the closing handshake.
        time.sleep(3 * MS)
        self.assertFalse(close_thread.is_alive())

        # Connection is closed now.
        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.recv()

        exc = raised.exception
        self.assertEqual(str(exc), "sent 1000 (OK); no close frame received")
        self.assertIsInstance(exc.__cause__, TimeoutError)

    def test_close_idempotency(self):
        """close does nothing if the connection is already closed."""
        self.connection.close()
        self.assertFrameSent(Frame(Opcode.CLOSE, b"\x03\xe8"))

        self.connection.close()
        self.assertNoFrameSent()

    @unittest.skipIf(
        platform.python_implementation() == "PyPy",
        "this test fails randomly due to a bug in PyPy",  # see #1314 for details
    )
    def test_close_idempotency_race_condition(self):
        """close waits if the connection is already closing."""

        self.connection.close_timeout = 5 * MS

        def closer():
            with self.delay_frames_rcvd(3 * MS):
                self.connection.close()

        close_thread = threading.Thread(target=closer)
        close_thread.start()

        # Let closer() initiate the closing handshake and send a close frame.
        time.sleep(MS)
        self.assertFrameSent(Frame(Opcode.CLOSE, b"\x03\xe8"))

        # Connection isn't closed yet.
        with self.assertRaises(TimeoutError):
            self.connection.recv(timeout=0)

        self.connection.close()
        self.assertNoFrameSent()

        # Connection is closed now.
        with self.assertRaises(ConnectionClosedOK):
            self.connection.recv(timeout=0)

        close_thread.join()

    def test_close_during_send(self):
        """close fails the connection when called concurrently with send."""
        close_gate = threading.Event()
        exit_gate = threading.Event()

        def closer():
            close_gate.wait()
            self.connection.close()
            exit_gate.set()

        def fragments():
            yield "😀"
            close_gate.set()
            exit_gate.wait()
            yield "😀"

        close_thread = threading.Thread(target=closer)
        close_thread.start()

        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.send(fragments())

        exc = raised.exception
        self.assertEqual(
            str(exc),
            "sent 1011 (internal error) close during fragmented message; "
            "no close frame received",
        )
        self.assertIsNone(exc.__cause__)

        close_thread.join()

    # Test ping.

    @patch("random.getrandbits")
    def test_ping(self, getrandbits):
        """ping sends a ping frame with a random payload."""
        getrandbits.return_value = 1918987876
        self.connection.ping()
        getrandbits.assert_called_once_with(32)
        self.assertFrameSent(Frame(Opcode.PING, b"rand"))

    def test_ping_explicit_text(self):
        """ping sends a ping frame with a payload provided as text."""
        self.connection.ping("ping")
        self.assertFrameSent(Frame(Opcode.PING, b"ping"))

    def test_ping_explicit_binary(self):
        """ping sends a ping frame with a payload provided as binary."""
        self.connection.ping(b"ping")
        self.assertFrameSent(Frame(Opcode.PING, b"ping"))

    def test_ping_duplicate_payload(self):
        """ping rejects the same payload until receiving the pong."""
        with self.remote_connection.protocol_mutex:  # block response to ping
            pong_waiter = self.connection.ping("idem")
            with self.assertRaisesRegex(
                RuntimeError,
                "already waiting for a pong with the same data",
            ):
                self.connection.ping("idem")
        self.assertTrue(pong_waiter.wait(MS))
        self.connection.ping("idem")  # doesn't raise an exception

    def test_acknowledge_ping(self):
        """ping is acknowledged by a pong with the same payload."""
        with self.drop_frames_rcvd():
            pong_waiter = self.connection.ping("this")
            self.assertFalse(pong_waiter.wait(MS))
        self.remote_connection.pong("this")
        self.assertTrue(pong_waiter.wait(MS))

    def test_acknowledge_ping_non_matching_pong(self):
        """ping isn't acknowledged by a pong with a different payload."""
        with self.drop_frames_rcvd():
            pong_waiter = self.connection.ping("this")
        self.remote_connection.pong("that")
        self.assertFalse(pong_waiter.wait(MS))

    def test_acknowledge_previous_ping(self):
        """ping is acknowledged by a pong with the same payload as a later ping."""
        with self.drop_frames_rcvd():
            pong_waiter = self.connection.ping("this")
            self.connection.ping("that")
        self.remote_connection.pong("that")
        self.assertTrue(pong_waiter.wait(MS))

    # Test pong.

    def test_pong(self):
        """pong sends a pong frame."""
        self.connection.pong()
        self.assertFrameSent(Frame(Opcode.PONG, b""))

    def test_pong_explicit_text(self):
        """pong sends a pong frame with a payload provided as text."""
        self.connection.pong("pong")
        self.assertFrameSent(Frame(Opcode.PONG, b"pong"))

    def test_pong_explicit_binary(self):
        """pong sends a pong frame with a payload provided as binary."""
        self.connection.pong(b"pong")
        self.assertFrameSent(Frame(Opcode.PONG, b"pong"))

    # Test attributes.

    def test_id(self):
        """Connection has an id attribute."""
        self.assertIsInstance(self.connection.id, uuid.UUID)

    def test_logger(self):
        """Connection has a logger attribute."""
        self.assertIsInstance(self.connection.logger, logging.LoggerAdapter)

    def test_local_address(self):
        """Connection has a local_address attribute."""
        self.assertIsNotNone(self.connection.local_address)

    def test_remote_address(self):
        """Connection has a remote_address attribute."""
        self.assertIsNotNone(self.connection.remote_address)

    def test_request(self):
        """Connection has a request attribute."""
        self.assertIsNone(self.connection.request)

    def test_response(self):
        """Connection has a response attribute."""
        self.assertIsNone(self.connection.response)

    def test_subprotocol(self):
        """Connection has a subprotocol attribute."""
        self.assertIsNone(self.connection.subprotocol)

    # Test reporting of network errors.

    @unittest.skipUnless(sys.platform == "darwin", "works only on BSD")
    def test_reading_in_recv_events_fails(self):
        """Error when reading incoming frames is correctly reported."""
        # Inject a fault by closing the socket. This works only on BSD.
        # I cannot find a way to achieve the same effect on Linux.
        self.connection.socket.close()
        # The connection closed exception reports the injected fault.
        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.recv()
        self.assertIsInstance(raised.exception.__cause__, IOError)

    def test_writing_in_recv_events_fails(self):
        """Error when responding to incoming frames is correctly reported."""
        # Inject a fault by shutting down the socket for writing — but not by
        # closing it because that would terminate the connection.
        self.connection.socket.shutdown(socket.SHUT_WR)
        # Receive a ping. Responding with a pong will fail.
        self.remote_connection.ping()
        # The connection closed exception reports the injected fault.
        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.recv()
        self.assertIsInstance(raised.exception.__cause__, BrokenPipeError)

    def test_writing_in_send_context_fails(self):
        """Error when sending outgoing frame is correctly reported."""
        # Inject a fault by shutting down the socket for writing — but not by
        # closing it because that would terminate the connection.
        self.connection.socket.shutdown(socket.SHUT_WR)
        # Sending a pong will fail.
        # The connection closed exception reports the injected fault.
        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.pong()
        self.assertIsInstance(raised.exception.__cause__, BrokenPipeError)

    # Test safety nets — catching all exceptions in case of bugs.

    @patch("websockets.protocol.Protocol.events_received")
    def test_unexpected_failure_in_recv_events(self, events_received):
        """Unexpected internal error in recv_events() is correctly reported."""
        # Inject a fault in a random call in recv_events().
        # This test is tightly coupled to the implementation.
        events_received.side_effect = AssertionError
        # Receive a message to trigger the fault.
        self.remote_connection.send("😀")

        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.recv()

        exc = raised.exception
        self.assertEqual(str(exc), "no close frame received or sent")
        self.assertIsInstance(exc.__cause__, AssertionError)

    @patch("websockets.protocol.Protocol.send_text")
    def test_unexpected_failure_in_send_context(self, send_text):
        """Unexpected internal error in send_context() is correctly reported."""
        # Inject a fault in a random call in send_context().
        # This test is tightly coupled to the implementation.
        send_text.side_effect = AssertionError

        # Send a message to trigger the fault.
        # The connection closed exception reports the injected fault.
        with self.assertRaises(ConnectionClosedError) as raised:
            self.connection.send("😀")

        exc = raised.exception
        self.assertEqual(str(exc), "no close frame received or sent")
        self.assertIsInstance(exc.__cause__, AssertionError)


class ServerConnectionTests(ClientConnectionTests):
    LOCAL = SERVER
    REMOTE = CLIENT
