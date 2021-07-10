import asyncio
import contextlib
import sys
import unittest
import unittest.mock
import warnings

from websockets.exceptions import ConnectionClosed, InvalidState
from websockets.framing import *
from websockets.protocol import State, WebSocketCommonProtocol

from .utils import MS, AsyncioTestCase


async def async_iterable(iterable):
    for item in iterable:
        yield item


class TransportMock(unittest.mock.Mock):
    """
    Transport mock to control the protocol's inputs and outputs in tests.

    It calls the protocol's connection_made and connection_lost methods like
    actual transports.

    It also calls the protocol's connection_open method to bypass the
    WebSocket handshake.

    To simulate incoming data, tests call the protocol's data_received and
    eof_received methods directly.

    They could also pause_writing and resume_writing to test flow control.

    """

    # This should happen in __init__ but overriding Mock.__init__ is hard.
    def setup_mock(self, loop, protocol):
        self.loop = loop
        self.protocol = protocol
        self._eof = False
        self._closing = False
        # Simulate a successful TCP handshake.
        self.protocol.connection_made(self)
        # Simulate a successful WebSocket handshake.
        self.protocol.connection_open()

    def can_write_eof(self):
        return True

    def write_eof(self):
        # When the protocol half-closes the TCP connection, it expects the
        # other end to close it. Simulate that.
        if not self._eof:
            self.loop.call_soon(self.close)
        self._eof = True

    def close(self):
        # Simulate how actual transports drop the connection.
        if not self._closing:
            self.loop.call_soon(self.protocol.connection_lost, None)
        self._closing = True

    def abort(self):
        # Change this to an `if` if tests call abort() multiple times.
        assert self.protocol.state is not State.CLOSED
        self.loop.call_soon(self.protocol.connection_lost, None)


class CommonTests:
    """
    Mixin that defines most tests but doesn't inherit unittest.TestCase.

    Tests are run by the ServerTests and ClientTests subclasses.

    """

    def setUp(self):
        super().setUp()
        # Disable pings to make it easier to test what frames are sent exactly.
        self.protocol = WebSocketCommonProtocol(ping_interval=None)
        self.transport = TransportMock()
        self.transport.setup_mock(self.loop, self.protocol)

    def tearDown(self):
        self.transport.close()
        self.loop.run_until_complete(self.protocol.close())
        super().tearDown()

    # Utilities for writing tests.

    def make_drain_slow(self, delay=MS):
        # Process connection_made in order to initialize self.protocol.transport.
        self.run_loop_once()

        original_drain = self.protocol._drain

        async def delayed_drain():
            await asyncio.sleep(
                delay, loop=self.loop if sys.version_info[:2] < (3, 8) else None
            )
            await original_drain()

        self.protocol._drain = delayed_drain

    close_frame = Frame(True, OP_CLOSE, serialize_close(1000, "close"))
    local_close = Frame(True, OP_CLOSE, serialize_close(1000, "local"))
    remote_close = Frame(True, OP_CLOSE, serialize_close(1000, "remote"))

    def receive_frame(self, frame):
        """
        Make the protocol receive a frame.

        """
        write = self.protocol.data_received
        mask = not self.protocol.is_client
        frame.write(write, mask=mask)

    def receive_eof(self):
        """
        Make the protocol receive the end of the data stream.

        Since ``WebSocketCommonProtocol.eof_received`` returns ``None``, an
        actual transport would close itself after calling it. This function
        emulates that behavior.

        """
        self.protocol.eof_received()
        self.loop.call_soon(self.transport.close)

    def receive_eof_if_client(self):
        """
        Like receive_eof, but only if this is the client side.

        Since the server is supposed to initiate the termination of the TCP
        connection, this method helps making tests work for both sides.

        """
        if self.protocol.is_client:
            self.receive_eof()

    def close_connection(self, code=1000, reason="close"):
        """
        Execute a closing handshake.

        This puts the connection in the CLOSED state.

        """
        close_frame_data = serialize_close(code, reason)
        # Prepare the response to the closing handshake from the remote side.
        self.receive_frame(Frame(True, OP_CLOSE, close_frame_data))
        self.receive_eof_if_client()
        # Trigger the closing handshake from the local side and complete it.
        self.loop.run_until_complete(self.protocol.close(code, reason))
        # Empty the outgoing data stream so we can make assertions later on.
        self.assertOneFrameSent(True, OP_CLOSE, close_frame_data)

        assert self.protocol.state is State.CLOSED

    def half_close_connection_local(self, code=1000, reason="close"):
        """
        Start a closing handshake but do not complete it.

        The main difference with `close_connection` is that the connection is
        left in the CLOSING state until the event loop runs again.

        The current implementation returns a task that must be awaited or
        canceled, else asyncio complains about destroying a pending task.

        """
        close_frame_data = serialize_close(code, reason)
        # Trigger the closing handshake from the local endpoint.
        close_task = self.loop.create_task(self.protocol.close(code, reason))
        self.run_loop_once()  # wait_for executes
        self.run_loop_once()  # write_frame executes
        # Empty the outgoing data stream so we can make assertions later on.
        self.assertOneFrameSent(True, OP_CLOSE, close_frame_data)

        assert self.protocol.state is State.CLOSING

        # Complete the closing sequence at 1ms intervals so the test can run
        # at each point even it goes back to the event loop several times.
        self.loop.call_later(
            MS, self.receive_frame, Frame(True, OP_CLOSE, close_frame_data)
        )
        self.loop.call_later(2 * MS, self.receive_eof_if_client)

        # This task must be awaited or canceled by the caller.
        return close_task

    def half_close_connection_remote(self, code=1000, reason="close"):
        """
        Receive a closing handshake but do not complete it.

        The main difference with `close_connection` is that the connection is
        left in the CLOSING state until the event loop runs again.

        """
        # On the server side, websockets completes the closing handshake and
        # closes the TCP connection immediately. Yield to the event loop after
        # sending the close frame to run the test while the connection is in
        # the CLOSING state.
        if not self.protocol.is_client:
            self.make_drain_slow()

        close_frame_data = serialize_close(code, reason)
        # Trigger the closing handshake from the remote endpoint.
        self.receive_frame(Frame(True, OP_CLOSE, close_frame_data))
        self.run_loop_once()  # read_frame executes
        # Empty the outgoing data stream so we can make assertions later on.
        self.assertOneFrameSent(True, OP_CLOSE, close_frame_data)

        assert self.protocol.state is State.CLOSING

        # Complete the closing sequence at 1ms intervals so the test can run
        # at each point even it goes back to the event loop several times.
        self.loop.call_later(2 * MS, self.receive_eof_if_client)

    def process_invalid_frames(self):
        """
        Make the protocol fail quickly after simulating invalid data.

        To achieve this, this function triggers the protocol's eof_received,
        which interrupts pending reads waiting for more data.

        """
        self.run_loop_once()
        self.receive_eof()
        self.loop.run_until_complete(self.protocol.close_connection_task)

    def sent_frames(self):
        """
        Read all frames sent to the transport.

        """
        stream = asyncio.StreamReader(loop=self.loop)

        for (data,), kw in self.transport.write.call_args_list:
            stream.feed_data(data)
        self.transport.write.call_args_list = []
        stream.feed_eof()

        frames = []
        while not stream.at_eof():
            frames.append(
                self.loop.run_until_complete(
                    Frame.read(stream.readexactly, mask=self.protocol.is_client)
                )
            )
        return frames

    def last_sent_frame(self):
        """
        Read the last frame sent to the transport.

        This method assumes that at most one frame was sent. It raises an
        AssertionError otherwise.

        """
        frames = self.sent_frames()
        if frames:
            assert len(frames) == 1
            return frames[0]

    def assertFramesSent(self, *frames):
        self.assertEqual(self.sent_frames(), [Frame(*args) for args in frames])

    def assertOneFrameSent(self, *args):
        self.assertEqual(self.last_sent_frame(), Frame(*args))

    def assertNoFrameSent(self):
        self.assertIsNone(self.last_sent_frame())

    def assertConnectionClosed(self, code, message):
        # The following line guarantees that connection_lost was called.
        self.assertEqual(self.protocol.state, State.CLOSED)
        # A close frame was received.
        self.assertEqual(self.protocol.close_code, code)
        self.assertEqual(self.protocol.close_reason, message)

    def assertConnectionFailed(self, code, message):
        # The following line guarantees that connection_lost was called.
        self.assertEqual(self.protocol.state, State.CLOSED)
        # No close frame was received.
        self.assertEqual(self.protocol.close_code, 1006)
        self.assertEqual(self.protocol.close_reason, "")
        # A close frame was sent -- unless the connection was already lost.
        if code == 1006:
            self.assertNoFrameSent()
        else:
            self.assertOneFrameSent(True, OP_CLOSE, serialize_close(code, message))

    @contextlib.contextmanager
    def assertCompletesWithin(self, min_time, max_time):
        t0 = self.loop.time()
        yield
        t1 = self.loop.time()
        dt = t1 - t0
        self.assertGreaterEqual(dt, min_time, f"Too fast: {dt} < {min_time}")
        self.assertLess(dt, max_time, f"Too slow: {dt} >= {max_time}")

    # Test constructor.

    def test_timeout_backwards_compatibility(self):
        with warnings.catch_warnings(record=True) as recorded_warnings:
            protocol = WebSocketCommonProtocol(timeout=5)

        self.assertEqual(protocol.close_timeout, 5)

        self.assertEqual(len(recorded_warnings), 1)
        warning = recorded_warnings[0].message
        self.assertEqual(str(warning), "rename timeout to close_timeout")
        self.assertEqual(type(warning), DeprecationWarning)

    # Test public attributes.

    def test_local_address(self):
        get_extra_info = unittest.mock.Mock(return_value=("host", 4312))
        self.transport.get_extra_info = get_extra_info

        self.assertEqual(self.protocol.local_address, ("host", 4312))
        get_extra_info.assert_called_with("sockname")

    def test_local_address_before_connection(self):
        # Emulate the situation before connection_open() runs.
        _transport = self.protocol.transport
        del self.protocol.transport
        try:
            self.assertEqual(self.protocol.local_address, None)
        finally:
            self.protocol.transport = _transport

    def test_remote_address(self):
        get_extra_info = unittest.mock.Mock(return_value=("host", 4312))
        self.transport.get_extra_info = get_extra_info

        self.assertEqual(self.protocol.remote_address, ("host", 4312))
        get_extra_info.assert_called_with("peername")

    def test_remote_address_before_connection(self):
        # Emulate the situation before connection_open() runs.
        _transport = self.protocol.transport
        del self.protocol.transport
        try:
            self.assertEqual(self.protocol.remote_address, None)
        finally:
            self.protocol.transport = _transport

    def test_open(self):
        self.assertTrue(self.protocol.open)
        self.close_connection()
        self.assertFalse(self.protocol.open)

    def test_closed(self):
        self.assertFalse(self.protocol.closed)
        self.close_connection()
        self.assertTrue(self.protocol.closed)

    def test_wait_closed(self):
        wait_closed = self.loop.create_task(self.protocol.wait_closed())
        self.assertFalse(wait_closed.done())
        self.close_connection()
        self.assertTrue(wait_closed.done())

    # Test the recv coroutine.

    def test_recv_text(self):
        self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8")))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, "café")

    def test_recv_binary(self):
        self.receive_frame(Frame(True, OP_BINARY, b"tea"))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, b"tea")

    def test_recv_on_closing_connection_local(self):
        close_task = self.half_close_connection_local()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.recv())

        self.loop.run_until_complete(close_task)  # cleanup

    def test_recv_on_closing_connection_remote(self):
        self.half_close_connection_remote()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.recv())

    def test_recv_on_closed_connection(self):
        self.close_connection()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.recv())

    def test_recv_protocol_error(self):
        self.receive_frame(Frame(True, OP_CONT, "café".encode("utf-8")))
        self.process_invalid_frames()
        self.assertConnectionFailed(1002, "")

    def test_recv_unicode_error(self):
        self.receive_frame(Frame(True, OP_TEXT, "café".encode("latin-1")))
        self.process_invalid_frames()
        self.assertConnectionFailed(1007, "")

    def test_recv_text_payload_too_big(self):
        self.protocol.max_size = 1024
        self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8") * 205))
        self.process_invalid_frames()
        self.assertConnectionFailed(1009, "")

    def test_recv_binary_payload_too_big(self):
        self.protocol.max_size = 1024
        self.receive_frame(Frame(True, OP_BINARY, b"tea" * 342))
        self.process_invalid_frames()
        self.assertConnectionFailed(1009, "")

    def test_recv_text_no_max_size(self):
        self.protocol.max_size = None  # for test coverage
        self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8") * 205))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, "café" * 205)

    def test_recv_binary_no_max_size(self):
        self.protocol.max_size = None  # for test coverage
        self.receive_frame(Frame(True, OP_BINARY, b"tea" * 342))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, b"tea" * 342)

    def test_recv_queue_empty(self):
        recv = self.loop.create_task(self.protocol.recv())
        with self.assertRaises(asyncio.TimeoutError):
            self.loop.run_until_complete(
                asyncio.wait_for(asyncio.shield(recv), timeout=MS)
            )

        self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8")))
        data = self.loop.run_until_complete(recv)
        self.assertEqual(data, "café")

    def test_recv_queue_full(self):
        self.protocol.max_queue = 2
        # Test internals because it's hard to verify buffers from the outside.
        self.assertEqual(list(self.protocol.messages), [])

        self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8")))
        self.run_loop_once()
        self.assertEqual(list(self.protocol.messages), ["café"])

        self.receive_frame(Frame(True, OP_BINARY, b"tea"))
        self.run_loop_once()
        self.assertEqual(list(self.protocol.messages), ["café", b"tea"])

        self.receive_frame(Frame(True, OP_BINARY, b"milk"))
        self.run_loop_once()
        self.assertEqual(list(self.protocol.messages), ["café", b"tea"])

        self.loop.run_until_complete(self.protocol.recv())
        self.run_loop_once()
        self.assertEqual(list(self.protocol.messages), [b"tea", b"milk"])

        self.loop.run_until_complete(self.protocol.recv())
        self.run_loop_once()
        self.assertEqual(list(self.protocol.messages), [b"milk"])

        self.loop.run_until_complete(self.protocol.recv())
        self.run_loop_once()
        self.assertEqual(list(self.protocol.messages), [])

    def test_recv_queue_no_limit(self):
        self.protocol.max_queue = None

        for _ in range(100):
            self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8")))
            self.run_loop_once()

        # Incoming message queue can contain at least 100 messages.
        self.assertEqual(list(self.protocol.messages), ["café"] * 100)

        for _ in range(100):
            self.loop.run_until_complete(self.protocol.recv())

        self.assertEqual(list(self.protocol.messages), [])

    def test_recv_other_error(self):
        async def read_message():
            raise Exception("BOOM")

        self.protocol.read_message = read_message
        self.process_invalid_frames()
        self.assertConnectionFailed(1011, "")

    def test_recv_canceled(self):
        recv = self.loop.create_task(self.protocol.recv())
        self.loop.call_soon(recv.cancel)

        with self.assertRaises(asyncio.CancelledError):
            self.loop.run_until_complete(recv)

        # The next frame doesn't disappear in a vacuum (it used to).
        self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8")))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, "café")

    def test_recv_canceled_race_condition(self):
        recv = self.loop.create_task(
            asyncio.wait_for(self.protocol.recv(), timeout=0.000_001)
        )
        self.loop.call_soon(
            self.receive_frame, Frame(True, OP_TEXT, "café".encode("utf-8"))
        )

        with self.assertRaises(asyncio.TimeoutError):
            self.loop.run_until_complete(recv)

        # The previous frame doesn't disappear in a vacuum (it used to).
        self.receive_frame(Frame(True, OP_TEXT, "tea".encode("utf-8")))
        data = self.loop.run_until_complete(self.protocol.recv())
        # If we're getting "tea" there, it means "café" was swallowed (ha, ha).
        self.assertEqual(data, "café")

    def test_recv_when_transfer_data_cancelled(self):
        # Clog incoming queue.
        self.protocol.max_queue = 1
        self.receive_frame(Frame(True, OP_TEXT, "café".encode("utf-8")))
        self.receive_frame(Frame(True, OP_BINARY, b"tea"))
        self.run_loop_once()

        # Flow control kicks in (check with an implementation detail).
        self.assertFalse(self.protocol._put_message_waiter.done())

        # Schedule recv().
        recv = self.loop.create_task(self.protocol.recv())

        # Cancel transfer_data_task (again, implementation detail).
        self.protocol.fail_connection()
        self.run_loop_once()
        self.assertTrue(self.protocol.transfer_data_task.cancelled())

        # recv() completes properly.
        self.assertEqual(self.loop.run_until_complete(recv), "café")

    def test_recv_prevents_concurrent_calls(self):
        recv = self.loop.create_task(self.protocol.recv())

        with self.assertRaisesRegex(
            RuntimeError,
            "cannot call recv while another coroutine "
            "is already waiting for the next message",
        ):
            self.loop.run_until_complete(self.protocol.recv())

        recv.cancel()

    # Test the send coroutine.

    def test_send_text(self):
        self.loop.run_until_complete(self.protocol.send("café"))
        self.assertOneFrameSent(True, OP_TEXT, "café".encode("utf-8"))

    def test_send_binary(self):
        self.loop.run_until_complete(self.protocol.send(b"tea"))
        self.assertOneFrameSent(True, OP_BINARY, b"tea")

    def test_send_binary_from_bytearray(self):
        self.loop.run_until_complete(self.protocol.send(bytearray(b"tea")))
        self.assertOneFrameSent(True, OP_BINARY, b"tea")

    def test_send_binary_from_memoryview(self):
        self.loop.run_until_complete(self.protocol.send(memoryview(b"tea")))
        self.assertOneFrameSent(True, OP_BINARY, b"tea")

    def test_send_binary_from_non_contiguous_memoryview(self):
        self.loop.run_until_complete(self.protocol.send(memoryview(b"tteeaa")[::2]))
        self.assertOneFrameSent(True, OP_BINARY, b"tea")

    def test_send_type_error(self):
        with self.assertRaises(TypeError):
            self.loop.run_until_complete(self.protocol.send(42))
        self.assertNoFrameSent()

    def test_send_iterable_text(self):
        self.loop.run_until_complete(self.protocol.send(["ca", "fé"]))
        self.assertFramesSent(
            (False, OP_TEXT, "ca".encode("utf-8")),
            (False, OP_CONT, "fé".encode("utf-8")),
            (True, OP_CONT, "".encode("utf-8")),
        )

    def test_send_iterable_binary(self):
        self.loop.run_until_complete(self.protocol.send([b"te", b"a"]))
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_iterable_binary_from_bytearray(self):
        self.loop.run_until_complete(
            self.protocol.send([bytearray(b"te"), bytearray(b"a")])
        )
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_iterable_binary_from_memoryview(self):
        self.loop.run_until_complete(
            self.protocol.send([memoryview(b"te"), memoryview(b"a")])
        )
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_iterable_binary_from_non_contiguous_memoryview(self):
        self.loop.run_until_complete(
            self.protocol.send([memoryview(b"ttee")[::2], memoryview(b"aa")[::2]])
        )
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_empty_iterable(self):
        self.loop.run_until_complete(self.protocol.send([]))
        self.assertNoFrameSent()

    def test_send_iterable_type_error(self):
        with self.assertRaises(TypeError):
            self.loop.run_until_complete(self.protocol.send([42]))
        self.assertNoFrameSent()

    def test_send_iterable_mixed_type_error(self):
        with self.assertRaises(TypeError):
            self.loop.run_until_complete(self.protocol.send(["café", b"tea"]))
        self.assertFramesSent(
            (False, OP_TEXT, "café".encode("utf-8")),
            (True, OP_CLOSE, serialize_close(1011, "")),
        )

    def test_send_iterable_prevents_concurrent_send(self):
        self.make_drain_slow(2 * MS)

        async def send_iterable():
            await self.protocol.send(["ca", "fé"])

        async def send_concurrent():
            await asyncio.sleep(MS)
            await self.protocol.send(b"tea")

        self.loop.run_until_complete(asyncio.gather(send_iterable(), send_concurrent()))
        self.assertFramesSent(
            (False, OP_TEXT, "ca".encode("utf-8")),
            (False, OP_CONT, "fé".encode("utf-8")),
            (True, OP_CONT, "".encode("utf-8")),
            (True, OP_BINARY, b"tea"),
        )

    def test_send_async_iterable_text(self):
        self.loop.run_until_complete(self.protocol.send(async_iterable(["ca", "fé"])))
        self.assertFramesSent(
            (False, OP_TEXT, "ca".encode("utf-8")),
            (False, OP_CONT, "fé".encode("utf-8")),
            (True, OP_CONT, "".encode("utf-8")),
        )

    def test_send_async_iterable_binary(self):
        self.loop.run_until_complete(self.protocol.send(async_iterable([b"te", b"a"])))
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_async_iterable_binary_from_bytearray(self):
        self.loop.run_until_complete(
            self.protocol.send(async_iterable([bytearray(b"te"), bytearray(b"a")]))
        )
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_async_iterable_binary_from_memoryview(self):
        self.loop.run_until_complete(
            self.protocol.send(async_iterable([memoryview(b"te"), memoryview(b"a")]))
        )
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_async_iterable_binary_from_non_contiguous_memoryview(self):
        self.loop.run_until_complete(
            self.protocol.send(
                async_iterable([memoryview(b"ttee")[::2], memoryview(b"aa")[::2]])
            )
        )
        self.assertFramesSent(
            (False, OP_BINARY, b"te"), (False, OP_CONT, b"a"), (True, OP_CONT, b"")
        )

    def test_send_empty_async_iterable(self):
        self.loop.run_until_complete(self.protocol.send(async_iterable([])))
        self.assertNoFrameSent()

    def test_send_async_iterable_type_error(self):
        with self.assertRaises(TypeError):
            self.loop.run_until_complete(self.protocol.send(async_iterable([42])))
        self.assertNoFrameSent()

    def test_send_async_iterable_mixed_type_error(self):
        with self.assertRaises(TypeError):
            self.loop.run_until_complete(
                self.protocol.send(async_iterable(["café", b"tea"]))
            )
        self.assertFramesSent(
            (False, OP_TEXT, "café".encode("utf-8")),
            (True, OP_CLOSE, serialize_close(1011, "")),
        )

    def test_send_async_iterable_prevents_concurrent_send(self):
        self.make_drain_slow(2 * MS)

        async def send_async_iterable():
            await self.protocol.send(async_iterable(["ca", "fé"]))

        async def send_concurrent():
            await asyncio.sleep(MS)
            await self.protocol.send(b"tea")

        self.loop.run_until_complete(
            asyncio.gather(send_async_iterable(), send_concurrent())
        )
        self.assertFramesSent(
            (False, OP_TEXT, "ca".encode("utf-8")),
            (False, OP_CONT, "fé".encode("utf-8")),
            (True, OP_CONT, "".encode("utf-8")),
            (True, OP_BINARY, b"tea"),
        )

    def test_send_on_closing_connection_local(self):
        close_task = self.half_close_connection_local()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.send("foobar"))

        self.assertNoFrameSent()

        self.loop.run_until_complete(close_task)  # cleanup

    def test_send_on_closing_connection_remote(self):
        self.half_close_connection_remote()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.send("foobar"))

        self.assertNoFrameSent()

    def test_send_on_closed_connection(self):
        self.close_connection()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.send("foobar"))

        self.assertNoFrameSent()

    # Test the ping coroutine.

    def test_ping_default(self):
        self.loop.run_until_complete(self.protocol.ping())
        # With our testing tools, it's more convenient to extract the expected
        # ping data from the library's internals than from the frame sent.
        ping_data = next(iter(self.protocol.pings))
        self.assertIsInstance(ping_data, bytes)
        self.assertEqual(len(ping_data), 4)
        self.assertOneFrameSent(True, OP_PING, ping_data)

    def test_ping_text(self):
        self.loop.run_until_complete(self.protocol.ping("café"))
        self.assertOneFrameSent(True, OP_PING, "café".encode("utf-8"))

    def test_ping_binary(self):
        self.loop.run_until_complete(self.protocol.ping(b"tea"))
        self.assertOneFrameSent(True, OP_PING, b"tea")

    def test_ping_binary_from_bytearray(self):
        self.loop.run_until_complete(self.protocol.ping(bytearray(b"tea")))
        self.assertOneFrameSent(True, OP_PING, b"tea")

    def test_ping_binary_from_memoryview(self):
        self.loop.run_until_complete(self.protocol.ping(memoryview(b"tea")))
        self.assertOneFrameSent(True, OP_PING, b"tea")

    def test_ping_binary_from_non_contiguous_memoryview(self):
        self.loop.run_until_complete(self.protocol.ping(memoryview(b"tteeaa")[::2]))
        self.assertOneFrameSent(True, OP_PING, b"tea")

    def test_ping_type_error(self):
        with self.assertRaises(TypeError):
            self.loop.run_until_complete(self.protocol.ping(42))
        self.assertNoFrameSent()

    def test_ping_on_closing_connection_local(self):
        close_task = self.half_close_connection_local()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.ping())

        self.assertNoFrameSent()

        self.loop.run_until_complete(close_task)  # cleanup

    def test_ping_on_closing_connection_remote(self):
        self.half_close_connection_remote()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.ping())

        self.assertNoFrameSent()

    def test_ping_on_closed_connection(self):
        self.close_connection()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.ping())

        self.assertNoFrameSent()

    # Test the pong coroutine.

    def test_pong_default(self):
        self.loop.run_until_complete(self.protocol.pong())
        self.assertOneFrameSent(True, OP_PONG, b"")

    def test_pong_text(self):
        self.loop.run_until_complete(self.protocol.pong("café"))
        self.assertOneFrameSent(True, OP_PONG, "café".encode("utf-8"))

    def test_pong_binary(self):
        self.loop.run_until_complete(self.protocol.pong(b"tea"))
        self.assertOneFrameSent(True, OP_PONG, b"tea")

    def test_pong_binary_from_bytearray(self):
        self.loop.run_until_complete(self.protocol.pong(bytearray(b"tea")))
        self.assertOneFrameSent(True, OP_PONG, b"tea")

    def test_pong_binary_from_memoryview(self):
        self.loop.run_until_complete(self.protocol.pong(memoryview(b"tea")))
        self.assertOneFrameSent(True, OP_PONG, b"tea")

    def test_pong_binary_from_non_contiguous_memoryview(self):
        self.loop.run_until_complete(self.protocol.pong(memoryview(b"tteeaa")[::2]))
        self.assertOneFrameSent(True, OP_PONG, b"tea")

    def test_pong_type_error(self):
        with self.assertRaises(TypeError):
            self.loop.run_until_complete(self.protocol.pong(42))
        self.assertNoFrameSent()

    def test_pong_on_closing_connection_local(self):
        close_task = self.half_close_connection_local()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.pong())

        self.assertNoFrameSent()

        self.loop.run_until_complete(close_task)  # cleanup

    def test_pong_on_closing_connection_remote(self):
        self.half_close_connection_remote()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.pong())

        self.assertNoFrameSent()

    def test_pong_on_closed_connection(self):
        self.close_connection()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.pong())

        self.assertNoFrameSent()

    # Test the protocol's logic for acknowledging pings with pongs.

    def test_answer_ping(self):
        self.receive_frame(Frame(True, OP_PING, b"test"))
        self.run_loop_once()
        self.assertOneFrameSent(True, OP_PONG, b"test")

    def test_ignore_pong(self):
        self.receive_frame(Frame(True, OP_PONG, b"test"))
        self.run_loop_once()
        self.assertNoFrameSent()

    def test_acknowledge_ping(self):
        ping = self.loop.run_until_complete(self.protocol.ping())
        self.assertFalse(ping.done())
        ping_frame = self.last_sent_frame()
        pong_frame = Frame(True, OP_PONG, ping_frame.data)
        self.receive_frame(pong_frame)
        self.run_loop_once()
        self.run_loop_once()
        self.assertTrue(ping.done())

    def test_abort_ping(self):
        ping = self.loop.run_until_complete(self.protocol.ping())
        # Remove the frame from the buffer, else close_connection() complains.
        self.last_sent_frame()
        self.assertFalse(ping.done())
        self.close_connection()
        self.assertTrue(ping.done())
        self.assertIsInstance(ping.exception(), ConnectionClosed)

    def test_abort_ping_does_not_log_exception_if_not_retreived(self):
        self.loop.run_until_complete(self.protocol.ping())
        # Get the internal Future, which isn't directly returned by ping().
        (ping,) = self.protocol.pings.values()
        # Remove the frame from the buffer, else close_connection() complains.
        self.last_sent_frame()
        self.close_connection()
        # Check a private attribute, for lack of a better solution.
        self.assertFalse(ping._log_traceback)

    def test_acknowledge_previous_pings(self):
        pings = [
            (self.loop.run_until_complete(self.protocol.ping()), self.last_sent_frame())
            for i in range(3)
        ]
        # Unsolicited pong doesn't acknowledge pings
        self.receive_frame(Frame(True, OP_PONG, b""))
        self.run_loop_once()
        self.run_loop_once()
        self.assertFalse(pings[0][0].done())
        self.assertFalse(pings[1][0].done())
        self.assertFalse(pings[2][0].done())
        # Pong acknowledges all previous pings
        self.receive_frame(Frame(True, OP_PONG, pings[1][1].data))
        self.run_loop_once()
        self.run_loop_once()
        self.assertTrue(pings[0][0].done())
        self.assertTrue(pings[1][0].done())
        self.assertFalse(pings[2][0].done())

    def test_acknowledge_aborted_ping(self):
        ping = self.loop.run_until_complete(self.protocol.ping())
        ping_frame = self.last_sent_frame()
        # Clog incoming queue. This lets connection_lost() abort pending pings
        # with a ConnectionClosed exception before transfer_data_task
        # terminates and close_connection cancels keepalive_ping_task.
        self.protocol.max_queue = 1
        self.receive_frame(Frame(True, OP_TEXT, b"1"))
        self.receive_frame(Frame(True, OP_TEXT, b"2"))
        # Add pong frame to the queue.
        pong_frame = Frame(True, OP_PONG, ping_frame.data)
        self.receive_frame(pong_frame)
        # Connection drops.
        self.receive_eof()
        self.loop.run_until_complete(self.protocol.wait_closed())
        # Ping receives a ConnectionClosed exception.
        with self.assertRaises(ConnectionClosed):
            ping.result()

        # transfer_data doesn't crash, which would be logged.
        with self.assertNoLogs():
            # Unclog incoming queue.
            self.loop.run_until_complete(self.protocol.recv())
            self.loop.run_until_complete(self.protocol.recv())

    def test_canceled_ping(self):
        ping = self.loop.run_until_complete(self.protocol.ping())
        ping_frame = self.last_sent_frame()
        ping.cancel()
        pong_frame = Frame(True, OP_PONG, ping_frame.data)
        self.receive_frame(pong_frame)
        self.run_loop_once()
        self.run_loop_once()
        self.assertTrue(ping.cancelled())

    def test_duplicate_ping(self):
        self.loop.run_until_complete(self.protocol.ping(b"foobar"))
        self.assertOneFrameSent(True, OP_PING, b"foobar")
        with self.assertRaises(ValueError):
            self.loop.run_until_complete(self.protocol.ping(b"foobar"))
        self.assertNoFrameSent()

    # Test the protocol's logic for rebuilding fragmented messages.

    def test_fragmented_text(self):
        self.receive_frame(Frame(False, OP_TEXT, "ca".encode("utf-8")))
        self.receive_frame(Frame(True, OP_CONT, "fé".encode("utf-8")))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, "café")

    def test_fragmented_binary(self):
        self.receive_frame(Frame(False, OP_BINARY, b"t"))
        self.receive_frame(Frame(False, OP_CONT, b"e"))
        self.receive_frame(Frame(True, OP_CONT, b"a"))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, b"tea")

    def test_fragmented_text_payload_too_big(self):
        self.protocol.max_size = 1024
        self.receive_frame(Frame(False, OP_TEXT, "café".encode("utf-8") * 100))
        self.receive_frame(Frame(True, OP_CONT, "café".encode("utf-8") * 105))
        self.process_invalid_frames()
        self.assertConnectionFailed(1009, "")

    def test_fragmented_binary_payload_too_big(self):
        self.protocol.max_size = 1024
        self.receive_frame(Frame(False, OP_BINARY, b"tea" * 171))
        self.receive_frame(Frame(True, OP_CONT, b"tea" * 171))
        self.process_invalid_frames()
        self.assertConnectionFailed(1009, "")

    def test_fragmented_text_no_max_size(self):
        self.protocol.max_size = None  # for test coverage
        self.receive_frame(Frame(False, OP_TEXT, "café".encode("utf-8") * 100))
        self.receive_frame(Frame(True, OP_CONT, "café".encode("utf-8") * 105))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, "café" * 205)

    def test_fragmented_binary_no_max_size(self):
        self.protocol.max_size = None  # for test coverage
        self.receive_frame(Frame(False, OP_BINARY, b"tea" * 171))
        self.receive_frame(Frame(True, OP_CONT, b"tea" * 171))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, b"tea" * 342)

    def test_control_frame_within_fragmented_text(self):
        self.receive_frame(Frame(False, OP_TEXT, "ca".encode("utf-8")))
        self.receive_frame(Frame(True, OP_PING, b""))
        self.receive_frame(Frame(True, OP_CONT, "fé".encode("utf-8")))
        data = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(data, "café")
        self.assertOneFrameSent(True, OP_PONG, b"")

    def test_unterminated_fragmented_text(self):
        self.receive_frame(Frame(False, OP_TEXT, "ca".encode("utf-8")))
        # Missing the second part of the fragmented frame.
        self.receive_frame(Frame(True, OP_BINARY, b"tea"))
        self.process_invalid_frames()
        self.assertConnectionFailed(1002, "")

    def test_close_handshake_in_fragmented_text(self):
        self.receive_frame(Frame(False, OP_TEXT, "ca".encode("utf-8")))
        self.receive_frame(Frame(True, OP_CLOSE, b""))
        self.process_invalid_frames()
        # The RFC may have overlooked this case: it says that control frames
        # can be interjected in the middle of a fragmented message and that a
        # close frame must be echoed. Even though there's an unterminated
        # message, technically, the closing handshake was successful.
        self.assertConnectionClosed(1005, "")

    def test_connection_close_in_fragmented_text(self):
        self.receive_frame(Frame(False, OP_TEXT, "ca".encode("utf-8")))
        self.process_invalid_frames()
        self.assertConnectionFailed(1006, "")

    # Test miscellaneous code paths to ensure full coverage.

    def test_connection_lost(self):
        # Test calling connection_lost without going through close_connection.
        self.protocol.connection_lost(None)

        self.assertConnectionFailed(1006, "")

    def test_ensure_open_before_opening_handshake(self):
        # Simulate a bug by forcibly reverting the protocol state.
        self.protocol.state = State.CONNECTING

        with self.assertRaises(InvalidState):
            self.loop.run_until_complete(self.protocol.ensure_open())

    def test_ensure_open_during_unclean_close(self):
        # Process connection_made in order to start transfer_data_task.
        self.run_loop_once()

        # Ensure the test terminates quickly.
        self.loop.call_later(MS, self.receive_eof_if_client)

        # Simulate the case when close() times out sending a close frame.
        self.protocol.fail_connection()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.ensure_open())

    def test_legacy_recv(self):
        # By default legacy_recv in disabled.
        self.assertEqual(self.protocol.legacy_recv, False)

        self.close_connection()

        # Enable legacy_recv.
        self.protocol.legacy_recv = True

        # Now recv() returns None instead of raising ConnectionClosed.
        self.assertIsNone(self.loop.run_until_complete(self.protocol.recv()))

    def test_connection_closed_attributes(self):
        self.close_connection()

        with self.assertRaises(ConnectionClosed) as context:
            self.loop.run_until_complete(self.protocol.recv())

        connection_closed_exc = context.exception
        self.assertEqual(connection_closed_exc.code, 1000)
        self.assertEqual(connection_closed_exc.reason, "close")

    # Test the protocol logic for sending keepalive pings.

    def restart_protocol_with_keepalive_ping(
        self, ping_interval=3 * MS, ping_timeout=3 * MS
    ):
        initial_protocol = self.protocol
        # copied from tearDown
        self.transport.close()
        self.loop.run_until_complete(self.protocol.close())
        # copied from setUp, but enables keepalive pings
        self.protocol = WebSocketCommonProtocol(
            ping_interval=ping_interval, ping_timeout=ping_timeout
        )
        self.transport = TransportMock()
        self.transport.setup_mock(self.loop, self.protocol)
        self.protocol.is_client = initial_protocol.is_client
        self.protocol.side = initial_protocol.side

    def test_keepalive_ping(self):
        self.restart_protocol_with_keepalive_ping()

        # Ping is sent at 3ms and acknowledged at 4ms.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        (ping_1,) = tuple(self.protocol.pings)
        self.assertOneFrameSent(True, OP_PING, ping_1)
        self.receive_frame(Frame(True, OP_PONG, ping_1))

        # Next ping is sent at 7ms.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        (ping_2,) = tuple(self.protocol.pings)
        self.assertOneFrameSent(True, OP_PING, ping_2)

        # The keepalive ping task goes on.
        self.assertFalse(self.protocol.keepalive_ping_task.done())

    def test_keepalive_ping_not_acknowledged_closes_connection(self):
        self.restart_protocol_with_keepalive_ping()

        # Ping is sent at 3ms and not acknowleged.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        (ping_1,) = tuple(self.protocol.pings)
        self.assertOneFrameSent(True, OP_PING, ping_1)

        # Connection is closed at 6ms.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        self.assertOneFrameSent(True, OP_CLOSE, serialize_close(1011, ""))

        # The keepalive ping task is complete.
        self.assertEqual(self.protocol.keepalive_ping_task.result(), None)

    def test_keepalive_ping_stops_when_connection_closing(self):
        self.restart_protocol_with_keepalive_ping()
        close_task = self.half_close_connection_local()

        # No ping sent at 3ms because the closing handshake is in progress.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        self.assertNoFrameSent()

        # The keepalive ping task terminated.
        self.assertTrue(self.protocol.keepalive_ping_task.cancelled())

        self.loop.run_until_complete(close_task)  # cleanup

    def test_keepalive_ping_stops_when_connection_closed(self):
        self.restart_protocol_with_keepalive_ping()
        self.close_connection()

        # The keepalive ping task terminated.
        self.assertTrue(self.protocol.keepalive_ping_task.cancelled())

    def test_keepalive_ping_does_not_crash_when_connection_lost(self):
        self.restart_protocol_with_keepalive_ping()
        # Clog incoming queue. This lets connection_lost() abort pending pings
        # with a ConnectionClosed exception before transfer_data_task
        # terminates and close_connection cancels keepalive_ping_task.
        self.protocol.max_queue = 1
        self.receive_frame(Frame(True, OP_TEXT, b"1"))
        self.receive_frame(Frame(True, OP_TEXT, b"2"))
        # Ping is sent at 3ms.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        (ping_waiter,) = tuple(self.protocol.pings.values())
        # Connection drops.
        self.receive_eof()
        self.loop.run_until_complete(self.protocol.wait_closed())

        # The ping waiter receives a ConnectionClosed exception.
        with self.assertRaises(ConnectionClosed):
            ping_waiter.result()
        # The keepalive ping task terminated properly.
        self.assertIsNone(self.protocol.keepalive_ping_task.result())

        # Unclog incoming queue to terminate the test quickly.
        self.loop.run_until_complete(self.protocol.recv())
        self.loop.run_until_complete(self.protocol.recv())

    def test_keepalive_ping_with_no_ping_interval(self):
        self.restart_protocol_with_keepalive_ping(ping_interval=None)

        # No ping is sent at 3ms.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        self.assertNoFrameSent()

    def test_keepalive_ping_with_no_ping_timeout(self):
        self.restart_protocol_with_keepalive_ping(ping_timeout=None)

        # Ping is sent at 3ms and not acknowleged.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        (ping_1,) = tuple(self.protocol.pings)
        self.assertOneFrameSent(True, OP_PING, ping_1)

        # Next ping is sent at 7ms anyway.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))
        ping_1_again, ping_2 = tuple(self.protocol.pings)
        self.assertEqual(ping_1, ping_1_again)
        self.assertOneFrameSent(True, OP_PING, ping_2)

        # The keepalive ping task goes on.
        self.assertFalse(self.protocol.keepalive_ping_task.done())

    def test_keepalive_ping_unexpected_error(self):
        self.restart_protocol_with_keepalive_ping()

        async def ping():
            raise Exception("BOOM")

        self.protocol.ping = ping

        # The keepalive ping task fails when sending a ping at 3ms.
        self.loop.run_until_complete(asyncio.sleep(4 * MS))

        # The keepalive ping task is complete.
        # It logs and swallows the exception.
        self.assertEqual(self.protocol.keepalive_ping_task.result(), None)

    # Test the protocol logic for closing the connection.

    def test_local_close(self):
        # Emulate how the remote endpoint answers the closing handshake.
        self.loop.call_later(MS, self.receive_frame, self.close_frame)
        self.loop.call_later(MS, self.receive_eof_if_client)

        # Run the closing handshake.
        self.loop.run_until_complete(self.protocol.close(reason="close"))

        self.assertConnectionClosed(1000, "close")
        self.assertOneFrameSent(*self.close_frame)

        # Closing the connection again is a no-op.
        self.loop.run_until_complete(self.protocol.close(reason="oh noes!"))

        self.assertConnectionClosed(1000, "close")
        self.assertNoFrameSent()

    def test_remote_close(self):
        # Emulate how the remote endpoint initiates the closing handshake.
        self.loop.call_later(MS, self.receive_frame, self.close_frame)
        self.loop.call_later(MS, self.receive_eof_if_client)

        # Wait for some data in order to process the handshake.
        # After recv() raises ConnectionClosed, the connection is closed.
        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(self.protocol.recv())

        self.assertConnectionClosed(1000, "close")
        self.assertOneFrameSent(*self.close_frame)

        # Closing the connection again is a no-op.
        self.loop.run_until_complete(self.protocol.close(reason="oh noes!"))

        self.assertConnectionClosed(1000, "close")
        self.assertNoFrameSent()

    def test_remote_close_and_connection_lost(self):
        self.make_drain_slow()
        # Drop the connection right after receiving a close frame,
        # which prevents echoing the close frame properly.
        self.receive_frame(self.close_frame)
        self.receive_eof()

        with self.assertNoLogs():
            self.loop.run_until_complete(self.protocol.close(reason="oh noes!"))

        self.assertConnectionClosed(1000, "close")
        self.assertOneFrameSent(*self.close_frame)

    def test_simultaneous_close(self):
        # Receive the incoming close frame right after self.protocol.close()
        # starts executing. This reproduces the error described in:
        # https://github.com/aaugustin/websockets/issues/339
        self.loop.call_soon(self.receive_frame, self.remote_close)
        self.loop.call_soon(self.receive_eof_if_client)

        self.loop.run_until_complete(self.protocol.close(reason="local"))

        self.assertConnectionClosed(1000, "remote")
        # The current implementation sends a close frame in response to the
        # close frame received from the remote end. It skips the close frame
        # that should be sent as a result of calling close().
        self.assertOneFrameSent(*self.remote_close)

    def test_close_preserves_incoming_frames(self):
        self.receive_frame(Frame(True, OP_TEXT, b"hello"))

        self.loop.call_later(MS, self.receive_frame, self.close_frame)
        self.loop.call_later(MS, self.receive_eof_if_client)
        self.loop.run_until_complete(self.protocol.close(reason="close"))

        self.assertConnectionClosed(1000, "close")
        self.assertOneFrameSent(*self.close_frame)

        next_message = self.loop.run_until_complete(self.protocol.recv())
        self.assertEqual(next_message, "hello")

    def test_close_protocol_error(self):
        invalid_close_frame = Frame(True, OP_CLOSE, b"\x00")
        self.receive_frame(invalid_close_frame)
        self.receive_eof_if_client()
        self.run_loop_once()
        self.loop.run_until_complete(self.protocol.close(reason="close"))

        self.assertConnectionFailed(1002, "")

    def test_close_connection_lost(self):
        self.receive_eof()
        self.run_loop_once()
        self.loop.run_until_complete(self.protocol.close(reason="close"))

        self.assertConnectionFailed(1006, "")

    def test_local_close_during_recv(self):
        recv = self.loop.create_task(self.protocol.recv())

        self.loop.call_later(MS, self.receive_frame, self.close_frame)
        self.loop.call_later(MS, self.receive_eof_if_client)

        self.loop.run_until_complete(self.protocol.close(reason="close"))

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(recv)

        self.assertConnectionClosed(1000, "close")

    # There is no test_remote_close_during_recv because it would be identical
    # to test_remote_close.

    def test_remote_close_during_send(self):
        self.make_drain_slow()
        send = self.loop.create_task(self.protocol.send("hello"))

        self.receive_frame(self.close_frame)
        self.receive_eof()

        with self.assertRaises(ConnectionClosed):
            self.loop.run_until_complete(send)

        self.assertConnectionClosed(1000, "close")

    # There is no test_local_close_during_send because this cannot really
    # happen, considering that writes are serialized.


class ServerTests(CommonTests, AsyncioTestCase):
    def setUp(self):
        super().setUp()
        self.protocol.is_client = False
        self.protocol.side = "server"

    def test_local_close_send_close_frame_timeout(self):
        self.protocol.close_timeout = 10 * MS
        self.make_drain_slow(50 * MS)
        # If we can't send a close frame, time out in 10ms.
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(9 * MS, 19 * MS):
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1006, "")

    def test_local_close_receive_close_frame_timeout(self):
        self.protocol.close_timeout = 10 * MS
        # If the client doesn't send a close frame, time out in 10ms.
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(9 * MS, 19 * MS):
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1006, "")

    def test_local_close_connection_lost_timeout_after_write_eof(self):
        self.protocol.close_timeout = 10 * MS
        # If the client doesn't close its side of the TCP connection after we
        # half-close our side with write_eof(), time out in 10ms.
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(9 * MS, 19 * MS):
            # HACK: disable write_eof => other end drops connection emulation.
            self.transport._eof = True
            self.receive_frame(self.close_frame)
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1000, "close")

    def test_local_close_connection_lost_timeout_after_close(self):
        self.protocol.close_timeout = 10 * MS
        # If the client doesn't close its side of the TCP connection after we
        # half-close our side with write_eof() and close it with close(), time
        # out in 20ms.
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(19 * MS, 29 * MS):
            # HACK: disable write_eof => other end drops connection emulation.
            self.transport._eof = True
            # HACK: disable close => other end drops connection emulation.
            self.transport._closing = True
            self.receive_frame(self.close_frame)
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1000, "close")


class ClientTests(CommonTests, AsyncioTestCase):
    def setUp(self):
        super().setUp()
        self.protocol.is_client = True
        self.protocol.side = "client"

    def test_local_close_send_close_frame_timeout(self):
        self.protocol.close_timeout = 10 * MS
        self.make_drain_slow(50 * MS)
        # If we can't send a close frame, time out in 20ms.
        # - 10ms waiting for sending a close frame
        # - 10ms waiting for receiving a half-close
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(19 * MS, 29 * MS):
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1006, "")

    def test_local_close_receive_close_frame_timeout(self):
        self.protocol.close_timeout = 10 * MS
        # If the server doesn't send a close frame, time out in 20ms:
        # - 10ms waiting for receiving a close frame
        # - 10ms waiting for receiving a half-close
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(19 * MS, 29 * MS):
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1006, "")

    def test_local_close_connection_lost_timeout_after_write_eof(self):
        self.protocol.close_timeout = 10 * MS
        # If the server doesn't half-close its side of the TCP connection
        # after we send a close frame, time out in 20ms:
        # - 10ms waiting for receiving a half-close
        # - 10ms waiting for receiving a close after write_eof
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(19 * MS, 29 * MS):
            # HACK: disable write_eof => other end drops connection emulation.
            self.transport._eof = True
            self.receive_frame(self.close_frame)
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1000, "close")

    def test_local_close_connection_lost_timeout_after_close(self):
        self.protocol.close_timeout = 10 * MS
        # If the client doesn't close its side of the TCP connection after we
        # half-close our side with write_eof() and close it with close(), time
        # out in 20ms.
        # - 10ms waiting for receiving a half-close
        # - 10ms waiting for receiving a close after write_eof
        # - 10ms waiting for receiving a close after close
        # Check the timing within -1/+9ms for robustness.
        with self.assertCompletesWithin(29 * MS, 39 * MS):
            # HACK: disable write_eof => other end drops connection emulation.
            self.transport._eof = True
            # HACK: disable close => other end drops connection emulation.
            self.transport._closing = True
            self.receive_frame(self.close_frame)
            self.loop.run_until_complete(self.protocol.close(reason="close"))
        self.assertConnectionClosed(1000, "close")
