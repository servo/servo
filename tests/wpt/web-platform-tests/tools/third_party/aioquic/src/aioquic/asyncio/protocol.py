import asyncio
from typing import Any, Callable, Dict, Optional, Text, Tuple, Union, cast

from ..quic import events
from ..quic.connection import NetworkAddress, QuicConnection

QuicConnectionIdHandler = Callable[[bytes], None]
QuicStreamHandler = Callable[[asyncio.StreamReader, asyncio.StreamWriter], None]


class QuicConnectionProtocol(asyncio.DatagramProtocol):
    def __init__(
        self, quic: QuicConnection, stream_handler: Optional[QuicStreamHandler] = None
    ):
        loop = asyncio.get_event_loop()

        self._closed = asyncio.Event()
        self._connected = False
        self._connected_waiter: Optional[asyncio.Future[None]] = None
        self._loop = loop
        self._ping_waiters: Dict[int, asyncio.Future[None]] = {}
        self._quic = quic
        self._stream_readers: Dict[int, asyncio.StreamReader] = {}
        self._timer: Optional[asyncio.TimerHandle] = None
        self._timer_at: Optional[float] = None
        self._transmit_task: Optional[asyncio.Handle] = None
        self._transport: Optional[asyncio.DatagramTransport] = None

        # callbacks
        self._connection_id_issued_handler: QuicConnectionIdHandler = lambda c: None
        self._connection_id_retired_handler: QuicConnectionIdHandler = lambda c: None
        self._connection_terminated_handler: Callable[[], None] = lambda: None
        if stream_handler is not None:
            self._stream_handler = stream_handler
        else:
            self._stream_handler = lambda r, w: None

    def change_connection_id(self) -> None:
        """
        Change the connection ID used to communicate with the peer.

        The previous connection ID will be retired.
        """
        self._quic.change_connection_id()
        self.transmit()

    def close(self) -> None:
        """
        Close the connection.
        """
        self._quic.close()
        self.transmit()

    def connect(self, addr: NetworkAddress) -> None:
        """
        Initiate the TLS handshake.

        This method can only be called for clients and a single time.
        """
        self._quic.connect(addr, now=self._loop.time())
        self.transmit()

    async def create_stream(
        self, is_unidirectional: bool = False
    ) -> Tuple[asyncio.StreamReader, asyncio.StreamWriter]:
        """
        Create a QUIC stream and return a pair of (reader, writer) objects.

        The returned reader and writer objects are instances of :class:`asyncio.StreamReader`
        and :class:`asyncio.StreamWriter` classes.
        """
        stream_id = self._quic.get_next_available_stream_id(
            is_unidirectional=is_unidirectional
        )
        return self._create_stream(stream_id)

    def request_key_update(self) -> None:
        """
        Request an update of the encryption keys.
        """
        self._quic.request_key_update()
        self.transmit()

    async def ping(self) -> None:
        """
        Ping the peer and wait for the response.
        """
        waiter = self._loop.create_future()
        uid = id(waiter)
        self._ping_waiters[uid] = waiter
        self._quic.send_ping(uid)
        self.transmit()
        await asyncio.shield(waiter)

    def transmit(self) -> None:
        """
        Send pending datagrams to the peer and arm the timer if needed.
        """
        self._transmit_task = None

        # send datagrams
        for data, addr in self._quic.datagrams_to_send(now=self._loop.time()):
            self._transport.sendto(data, addr)

        # re-arm timer
        timer_at = self._quic.get_timer()
        if self._timer is not None and self._timer_at != timer_at:
            self._timer.cancel()
            self._timer = None
        if self._timer is None and timer_at is not None:
            self._timer = self._loop.call_at(timer_at, self._handle_timer)
        self._timer_at = timer_at

    async def wait_closed(self) -> None:
        """
        Wait for the connection to be closed.
        """
        await self._closed.wait()

    async def wait_connected(self) -> None:
        """
        Wait for the TLS handshake to complete.
        """
        assert self._connected_waiter is None, "already awaiting connected"
        if not self._connected:
            self._connected_waiter = self._loop.create_future()
            await asyncio.shield(self._connected_waiter)

    # asyncio.Transport

    def connection_made(self, transport: asyncio.BaseTransport) -> None:
        self._transport = cast(asyncio.DatagramTransport, transport)

    def datagram_received(self, data: Union[bytes, Text], addr: NetworkAddress) -> None:
        self._quic.receive_datagram(cast(bytes, data), addr, now=self._loop.time())
        self._process_events()
        self.transmit()

    # overridable

    def quic_event_received(self, event: events.QuicEvent) -> None:
        """
        Called when a QUIC event is received.

        Reimplement this in your subclass to handle the events.
        """
        # FIXME: move this to a subclass
        if isinstance(event, events.ConnectionTerminated):
            for reader in self._stream_readers.values():
                reader.feed_eof()
        elif isinstance(event, events.StreamDataReceived):
            reader = self._stream_readers.get(event.stream_id, None)
            if reader is None:
                reader, writer = self._create_stream(event.stream_id)
                self._stream_handler(reader, writer)
            reader.feed_data(event.data)
            if event.end_stream:
                reader.feed_eof()

    # private

    def _create_stream(
        self, stream_id: int
    ) -> Tuple[asyncio.StreamReader, asyncio.StreamWriter]:
        adapter = QuicStreamAdapter(self, stream_id)
        reader = asyncio.StreamReader()
        writer = asyncio.StreamWriter(adapter, None, reader, self._loop)
        self._stream_readers[stream_id] = reader
        return reader, writer

    def _handle_timer(self) -> None:
        now = max(self._timer_at, self._loop.time())
        self._timer = None
        self._timer_at = None
        self._quic.handle_timer(now=now)
        self._process_events()
        self.transmit()

    def _process_events(self) -> None:
        event = self._quic.next_event()
        while event is not None:
            if isinstance(event, events.ConnectionIdIssued):
                self._connection_id_issued_handler(event.connection_id)
            elif isinstance(event, events.ConnectionIdRetired):
                self._connection_id_retired_handler(event.connection_id)
            elif isinstance(event, events.ConnectionTerminated):
                self._connection_terminated_handler()

                # abort connection waiter
                if self._connected_waiter is not None:
                    waiter = self._connected_waiter
                    self._connected_waiter = None
                    waiter.set_exception(ConnectionError)

                # abort ping waiters
                for waiter in self._ping_waiters.values():
                    waiter.set_exception(ConnectionError)
                self._ping_waiters.clear()

                self._closed.set()
            elif isinstance(event, events.HandshakeCompleted):
                if self._connected_waiter is not None:
                    waiter = self._connected_waiter
                    self._connected = True
                    self._connected_waiter = None
                    waiter.set_result(None)
            elif isinstance(event, events.PingAcknowledged):
                waiter = self._ping_waiters.pop(event.uid, None)
                if waiter is not None:
                    waiter.set_result(None)
            self.quic_event_received(event)
            event = self._quic.next_event()

    def _transmit_soon(self) -> None:
        if self._transmit_task is None:
            self._transmit_task = self._loop.call_soon(self.transmit)


class QuicStreamAdapter(asyncio.Transport):
    def __init__(self, protocol: QuicConnectionProtocol, stream_id: int):
        self.protocol = protocol
        self.stream_id = stream_id

    def can_write_eof(self) -> bool:
        return True

    def get_extra_info(self, name: str, default: Any = None) -> Any:
        """
        Get information about the underlying QUIC stream.
        """
        if name == "stream_id":
            return self.stream_id

    def write(self, data):
        self.protocol._quic.send_stream_data(self.stream_id, data)
        self.protocol._transmit_soon()

    def write_eof(self):
        self.protocol._quic.send_stream_data(self.stream_id, b"", end_stream=True)
        self.protocol._transmit_soon()
