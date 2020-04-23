import asyncio
import binascii
import contextlib
import io
import time
from unittest import TestCase

from aioquic import tls
from aioquic.buffer import UINT_VAR_MAX, Buffer, encode_uint_var
from aioquic.quic import events
from aioquic.quic.configuration import QuicConfiguration
from aioquic.quic.connection import (
    QuicConnection,
    QuicConnectionError,
    QuicNetworkPath,
    QuicReceiveContext,
)
from aioquic.quic.crypto import CryptoPair
from aioquic.quic.logger import QuicLogger
from aioquic.quic.packet import (
    PACKET_TYPE_INITIAL,
    QuicErrorCode,
    QuicFrameType,
    encode_quic_retry,
    encode_quic_version_negotiation,
)
from aioquic.quic.packet_builder import QuicDeliveryState, QuicPacketBuilder
from aioquic.quic.recovery import QuicPacketPacer

from .utils import (
    SERVER_CACERTFILE,
    SERVER_CERTFILE,
    SERVER_CERTFILE_WITH_CHAIN,
    SERVER_KEYFILE,
)

CLIENT_ADDR = ("1.2.3.4", 1234)

SERVER_ADDR = ("2.3.4.5", 4433)


class SessionTicketStore:
    def __init__(self):
        self.tickets = {}

    def add(self, ticket):
        self.tickets[ticket.ticket] = ticket

    def pop(self, label):
        return self.tickets.pop(label, None)


def client_receive_context(client, epoch=tls.Epoch.ONE_RTT):
    return QuicReceiveContext(
        epoch=epoch,
        host_cid=client.host_cid,
        network_path=client._network_paths[0],
        quic_logger_frames=[],
        time=asyncio.get_event_loop().time(),
    )


def consume_events(connection):
    while True:
        event = connection.next_event()
        if event is None:
            break


def create_standalone_client(self, **client_options):
    client = QuicConnection(
        configuration=QuicConfiguration(
            is_client=True, quic_logger=QuicLogger(), **client_options
        )
    )
    client._ack_delay = 0

    # kick-off handshake
    client.connect(SERVER_ADDR, now=time.time())
    self.assertEqual(drop(client), 1)

    return client


@contextlib.contextmanager
def client_and_server(
    client_kwargs={},
    client_options={},
    client_patch=lambda x: None,
    handshake=True,
    server_kwargs={},
    server_certfile=SERVER_CERTFILE,
    server_keyfile=SERVER_KEYFILE,
    server_options={},
    server_patch=lambda x: None,
):
    client_configuration = QuicConfiguration(
        is_client=True, quic_logger=QuicLogger(), **client_options
    )
    client_configuration.load_verify_locations(cafile=SERVER_CACERTFILE)

    client = QuicConnection(configuration=client_configuration, **client_kwargs)
    client._ack_delay = 0
    disable_packet_pacing(client)
    client_patch(client)

    server_configuration = QuicConfiguration(
        is_client=False, quic_logger=QuicLogger(), **server_options
    )
    server_configuration.load_cert_chain(server_certfile, server_keyfile)

    server = QuicConnection(configuration=server_configuration, **server_kwargs)
    server._ack_delay = 0
    disable_packet_pacing(server)
    server_patch(server)

    # perform handshake
    if handshake:
        client.connect(SERVER_ADDR, now=time.time())
        for i in range(3):
            roundtrip(client, server)

    yield client, server

    # close
    client.close()
    server.close()


def disable_packet_pacing(connection):
    class DummyPacketPacer(QuicPacketPacer):
        def next_send_time(self, now):
            return None

    connection._loss._pacer = DummyPacketPacer()


def sequence_numbers(connection_ids):
    return list(map(lambda x: x.sequence_number, connection_ids))


def drop(sender):
    """
    Drop datagrams from `sender`.
    """
    return len(sender.datagrams_to_send(now=time.time()))


def roundtrip(sender, receiver):
    """
    Send datagrams from `sender` to `receiver` and back.
    """
    return (transfer(sender, receiver), transfer(receiver, sender))


def transfer(sender, receiver):
    """
    Send datagrams from `sender` to `receiver`.
    """
    datagrams = 0
    from_addr = CLIENT_ADDR if sender._is_client else SERVER_ADDR
    for data, addr in sender.datagrams_to_send(now=time.time()):
        datagrams += 1
        receiver.receive_datagram(data, from_addr, now=time.time())
    return datagrams


class QuicConnectionTest(TestCase):
    def check_handshake(self, client, server, alpn_protocol=None):
        """
        Check handshake completed.
        """
        event = client.next_event()
        self.assertEqual(type(event), events.ProtocolNegotiated)
        self.assertEqual(event.alpn_protocol, alpn_protocol)
        event = client.next_event()
        self.assertEqual(type(event), events.HandshakeCompleted)
        self.assertEqual(event.alpn_protocol, alpn_protocol)
        self.assertEqual(event.early_data_accepted, False)
        self.assertEqual(event.session_resumed, False)
        for i in range(7):
            self.assertEqual(type(client.next_event()), events.ConnectionIdIssued)
        self.assertIsNone(client.next_event())

        event = server.next_event()
        self.assertEqual(type(event), events.ProtocolNegotiated)
        self.assertEqual(event.alpn_protocol, alpn_protocol)
        event = server.next_event()
        self.assertEqual(type(event), events.HandshakeCompleted)
        self.assertEqual(event.alpn_protocol, alpn_protocol)
        for i in range(7):
            self.assertEqual(type(server.next_event()), events.ConnectionIdIssued)
        self.assertIsNone(server.next_event())

    def test_connect(self):
        with client_and_server() as (client, server):
            # check handshake completed
            self.check_handshake(client=client, server=server)

            # check each endpoint has available connection IDs for the peer
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [1, 2, 3, 4, 5, 6, 7]
            )
            self.assertEqual(
                sequence_numbers(server._peer_cid_available), [1, 2, 3, 4, 5, 6, 7]
            )

            # client closes the connection
            client.close()
            self.assertEqual(transfer(client, server), 1)

            # check connection closes on the client side
            client.handle_timer(client.get_timer())
            event = client.next_event()
            self.assertEqual(type(event), events.ConnectionTerminated)
            self.assertEqual(event.error_code, QuicErrorCode.NO_ERROR)
            self.assertEqual(event.frame_type, None)
            self.assertEqual(event.reason_phrase, "")
            self.assertIsNone(client.next_event())

            # check connection closes on the server side
            server.handle_timer(server.get_timer())
            event = server.next_event()
            self.assertEqual(type(event), events.ConnectionTerminated)
            self.assertEqual(event.error_code, QuicErrorCode.NO_ERROR)
            self.assertEqual(event.frame_type, None)
            self.assertEqual(event.reason_phrase, "")
            self.assertIsNone(server.next_event())

            # check client log
            client_log = client.configuration.quic_logger.to_dict()
            self.assertGreater(len(client_log["traces"][0]["events"]), 20)

            # check server log
            server_log = server.configuration.quic_logger.to_dict()
            self.assertGreater(len(server_log["traces"][0]["events"]), 20)

    def test_connect_with_alpn(self):
        with client_and_server(
            client_options={"alpn_protocols": ["h3-25", "hq-25"]},
            server_options={"alpn_protocols": ["hq-25"]},
        ) as (client, server):
            # check handshake completed
            self.check_handshake(client=client, server=server, alpn_protocol="hq-25")

    def test_connect_with_secrets_log(self):
        client_log_file = io.StringIO()
        server_log_file = io.StringIO()
        with client_and_server(
            client_options={"secrets_log_file": client_log_file},
            server_options={"secrets_log_file": server_log_file},
        ) as (client, server):
            # check handshake completed
            self.check_handshake(client=client, server=server)

            # check secrets were logged
            client_log = client_log_file.getvalue()
            server_log = server_log_file.getvalue()
            self.assertEqual(client_log, server_log)
            labels = []
            for line in client_log.splitlines():
                labels.append(line.split()[0])
            self.assertEqual(
                labels,
                [
                    "QUIC_SERVER_HANDSHAKE_TRAFFIC_SECRET",
                    "QUIC_CLIENT_HANDSHAKE_TRAFFIC_SECRET",
                    "QUIC_SERVER_TRAFFIC_SECRET_0",
                    "QUIC_CLIENT_TRAFFIC_SECRET_0",
                ],
            )

    def test_connect_with_cert_chain(self):
        with client_and_server(server_certfile=SERVER_CERTFILE_WITH_CHAIN) as (
            client,
            server,
        ):
            # check handshake completed
            self.check_handshake(client=client, server=server)

    def test_connect_with_loss_1(self):
        """
        Check connection is established even in the client's INITIAL is lost.
        """

        def datagram_sizes(items):
            return [len(x[0]) for x in items]

        client_configuration = QuicConfiguration(is_client=True)
        client_configuration.load_verify_locations(cafile=SERVER_CACERTFILE)

        client = QuicConnection(configuration=client_configuration)
        client._ack_delay = 0

        server_configuration = QuicConfiguration(is_client=False)
        server_configuration.load_cert_chain(SERVER_CERTFILE, SERVER_KEYFILE)

        server = QuicConnection(configuration=server_configuration)
        server._ack_delay = 0

        # client sends INITIAL
        now = 0.0
        client.connect(SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280])
        self.assertEqual(client.get_timer(), 1.0)

        # INITIAL is lost
        now = 1.0
        client.handle_timer(now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280])
        self.assertEqual(client.get_timer(), 3.0)

        # server receives INITIAL, sends INITIAL + HANDSHAKE
        now = 1.1
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280, 1062])
        self.assertEqual(server.get_timer(), 2.1)
        self.assertEqual(len(server._loss.spaces[0].sent_packets), 1)
        self.assertEqual(len(server._loss.spaces[1].sent_packets), 2)
        self.assertEqual(type(server.next_event()), events.ProtocolNegotiated)
        self.assertIsNone(server.next_event())

        # handshake continues normally
        now = 1.2
        client.receive_datagram(items[0][0], SERVER_ADDR, now=now)
        client.receive_datagram(items[1][0], SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [376])
        self.assertAlmostEqual(client.get_timer(), 1.825)
        self.assertEqual(type(client.next_event()), events.ProtocolNegotiated)
        self.assertEqual(type(client.next_event()), events.HandshakeCompleted)
        self.assertEqual(type(client.next_event()), events.ConnectionIdIssued)

        now = 1.3
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [229])
        self.assertAlmostEqual(server.get_timer(), 1.825)
        self.assertEqual(len(server._loss.spaces[0].sent_packets), 0)
        self.assertEqual(len(server._loss.spaces[1].sent_packets), 0)
        self.assertEqual(type(server.next_event()), events.HandshakeCompleted)
        self.assertEqual(type(server.next_event()), events.ConnectionIdIssued)

        now = 1.4
        client.receive_datagram(items[0][0], SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [32])
        self.assertAlmostEqual(client.get_timer(), 61.4)  # idle timeout

    def test_connect_with_loss_2(self):
        def datagram_sizes(items):
            return [len(x[0]) for x in items]

        client_configuration = QuicConfiguration(is_client=True)
        client_configuration.load_verify_locations(cafile=SERVER_CACERTFILE)

        client = QuicConnection(configuration=client_configuration)
        client._ack_delay = 0

        server_configuration = QuicConfiguration(is_client=False)
        server_configuration.load_cert_chain(SERVER_CERTFILE, SERVER_KEYFILE)

        server = QuicConnection(configuration=server_configuration)
        server._ack_delay = 0

        # client sends INITIAL
        now = 0.0
        client.connect(SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280])
        self.assertEqual(client.get_timer(), 1.0)

        # server receives INITIAL, sends INITIAL + HANDSHAKE but second datagram is lost
        now = 0.1
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280, 1062])
        self.assertEqual(server.get_timer(), 1.1)
        self.assertEqual(len(server._loss.spaces[0].sent_packets), 1)
        self.assertEqual(len(server._loss.spaces[1].sent_packets), 2)

        # client only receives first datagram and sends ACKS
        now = 0.2
        client.receive_datagram(items[0][0], SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [97])
        self.assertAlmostEqual(client.get_timer(), 0.625)
        self.assertEqual(type(client.next_event()), events.ProtocolNegotiated)
        self.assertIsNone(client.next_event())

        # client PTO - HANDSHAKE PING
        now = client.get_timer()  # ~0.625
        client.handle_timer(now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [44])
        self.assertAlmostEqual(client.get_timer(), 1.875)

        # server receives PING, discards INITIAL and sends ACK
        now = 0.725
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [48])
        self.assertAlmostEqual(server.get_timer(), 1.1)
        self.assertEqual(len(server._loss.spaces[0].sent_packets), 0)
        self.assertEqual(len(server._loss.spaces[1].sent_packets), 3)
        self.assertEqual(type(server.next_event()), events.ProtocolNegotiated)
        self.assertIsNone(server.next_event())

        # ACKs are lost, server retransmits HANDSHAKE
        now = server.get_timer()
        server.handle_timer(now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280, 854])
        self.assertAlmostEqual(server.get_timer(), 3.1)
        self.assertEqual(len(server._loss.spaces[0].sent_packets), 0)
        self.assertEqual(len(server._loss.spaces[1].sent_packets), 3)
        self.assertIsNone(server.next_event())

        # handshake continues normally
        now = 1.2
        client.receive_datagram(items[0][0], SERVER_ADDR, now=now)
        client.receive_datagram(items[1][0], SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [329])
        self.assertAlmostEqual(client.get_timer(), 2.45)
        self.assertEqual(type(client.next_event()), events.HandshakeCompleted)
        self.assertEqual(type(client.next_event()), events.ConnectionIdIssued)

        now = 1.3
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [229])
        self.assertAlmostEqual(server.get_timer(), 1.925)
        self.assertEqual(type(server.next_event()), events.HandshakeCompleted)
        self.assertEqual(type(server.next_event()), events.ConnectionIdIssued)

        now = 1.4
        client.receive_datagram(items[0][0], SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [32])
        self.assertAlmostEqual(client.get_timer(), 61.4)  # idle timeout

    def test_connect_with_loss_3(self):
        def datagram_sizes(items):
            return [len(x[0]) for x in items]

        client_configuration = QuicConfiguration(is_client=True)
        client_configuration.load_verify_locations(cafile=SERVER_CACERTFILE)

        client = QuicConnection(configuration=client_configuration)
        client._ack_delay = 0

        server_configuration = QuicConfiguration(is_client=False)
        server_configuration.load_cert_chain(SERVER_CERTFILE, SERVER_KEYFILE)

        server = QuicConnection(configuration=server_configuration)
        server._ack_delay = 0

        # client sends INITIAL
        now = 0.0
        client.connect(SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280])
        self.assertEqual(client.get_timer(), 1.0)

        # server receives INITIAL, sends INITIAL + HANDSHAKE
        now = 0.1
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [1280, 1062])
        self.assertEqual(server.get_timer(), 1.1)
        self.assertEqual(len(server._loss.spaces[0].sent_packets), 1)
        self.assertEqual(len(server._loss.spaces[1].sent_packets), 2)

        # client receives INITIAL + HANDSHAKE
        now = 0.2
        client.receive_datagram(items[0][0], SERVER_ADDR, now=now)
        client.receive_datagram(items[1][0], SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [376])
        self.assertAlmostEqual(client.get_timer(), 0.825)
        self.assertEqual(type(client.next_event()), events.ProtocolNegotiated)
        self.assertEqual(type(client.next_event()), events.HandshakeCompleted)
        self.assertEqual(type(client.next_event()), events.ConnectionIdIssued)

        # server completes handshake
        now = 0.3
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [229])
        self.assertAlmostEqual(server.get_timer(), 0.825)
        self.assertEqual(len(server._loss.spaces[0].sent_packets), 0)
        self.assertEqual(len(server._loss.spaces[1].sent_packets), 0)
        self.assertEqual(type(server.next_event()), events.ProtocolNegotiated)
        self.assertEqual(type(server.next_event()), events.HandshakeCompleted)
        self.assertEqual(type(server.next_event()), events.ConnectionIdIssued)

        # server PTO - 1-RTT PING
        now = 0.825
        server.handle_timer(now=now)
        items = server.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [29])
        self.assertAlmostEqual(server.get_timer(), 1.875)

        # client receives PING, sends ACK
        now = 0.9
        client.receive_datagram(items[0][0], SERVER_ADDR, now=now)
        items = client.datagrams_to_send(now=now)
        self.assertEqual(datagram_sizes(items), [32])
        self.assertAlmostEqual(client.get_timer(), 0.825)

        # server receives ACK, retransmits HANDSHAKE_DONE
        now = 1.0
        self.assertFalse(server._handshake_done_pending)
        server.receive_datagram(items[0][0], CLIENT_ADDR, now=now)
        self.assertTrue(server._handshake_done_pending)
        items = server.datagrams_to_send(now=now)
        self.assertFalse(server._handshake_done_pending)
        self.assertEqual(datagram_sizes(items), [224])

    def test_connect_with_quantum_readiness(self):
        with client_and_server(client_options={"quantum_readiness_test": True},) as (
            client,
            server,
        ):
            stream_id = client.get_next_available_stream_id()
            client.send_stream_data(stream_id, b"hello")

            self.assertEqual(roundtrip(client, server), (1, 1))

            received = None
            while True:
                event = server.next_event()
                if isinstance(event, events.StreamDataReceived):
                    received = event.data
                elif event is None:
                    break

            self.assertEqual(received, b"hello")

    def test_connect_with_0rtt(self):
        client_ticket = None
        ticket_store = SessionTicketStore()

        def save_session_ticket(ticket):
            nonlocal client_ticket
            client_ticket = ticket

        with client_and_server(
            client_kwargs={"session_ticket_handler": save_session_ticket},
            server_kwargs={"session_ticket_handler": ticket_store.add},
        ) as (client, server):
            pass

        with client_and_server(
            client_options={"session_ticket": client_ticket},
            server_kwargs={"session_ticket_fetcher": ticket_store.pop},
            handshake=False,
        ) as (client, server):
            client.connect(SERVER_ADDR, now=time.time())
            stream_id = client.get_next_available_stream_id()
            client.send_stream_data(stream_id, b"hello")

            self.assertEqual(roundtrip(client, server), (2, 1))

            event = server.next_event()
            self.assertEqual(type(event), events.ProtocolNegotiated)

            event = server.next_event()
            self.assertEqual(type(event), events.StreamDataReceived)
            self.assertEqual(event.data, b"hello")

    def test_connect_with_0rtt_bad_max_early_data(self):
        client_ticket = None
        ticket_store = SessionTicketStore()

        def patch(server):
            """
            Patch server's TLS initialization to set an invalid
            max_early_data value.
            """
            real_initialize = server._initialize

            def patched_initialize(peer_cid: bytes):
                real_initialize(peer_cid)
                server.tls._max_early_data = 12345

            server._initialize = patched_initialize

        def save_session_ticket(ticket):
            nonlocal client_ticket
            client_ticket = ticket

        with client_and_server(
            client_kwargs={"session_ticket_handler": save_session_ticket},
            server_kwargs={"session_ticket_handler": ticket_store.add},
            server_patch=patch,
        ) as (client, server):
            # check handshake failed
            event = client.next_event()
            self.assertIsNone(event)

    def test_change_connection_id(self):
        with client_and_server() as (client, server):
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [1, 2, 3, 4, 5, 6, 7]
            )

            # the client changes connection ID
            client.change_connection_id()
            self.assertEqual(transfer(client, server), 1)
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [2, 3, 4, 5, 6, 7]
            )

            # the server provides a new connection ID
            self.assertEqual(transfer(server, client), 1)
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [2, 3, 4, 5, 6, 7, 8]
            )

    def test_change_connection_id_retransmit_new_connection_id(self):
        with client_and_server() as (client, server):
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [1, 2, 3, 4, 5, 6, 7]
            )

            # the client changes connection ID
            client.change_connection_id()
            self.assertEqual(transfer(client, server), 1)
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [2, 3, 4, 5, 6, 7]
            )

            # the server provides a new connection ID, NEW_CONNECTION_ID is lost
            self.assertEqual(drop(server), 1)
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [2, 3, 4, 5, 6, 7]
            )

            # NEW_CONNECTION_ID is retransmitted
            server._on_new_connection_id_delivery(
                QuicDeliveryState.LOST, server._host_cids[-1]
            )
            self.assertEqual(transfer(server, client), 1)
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [2, 3, 4, 5, 6, 7, 8]
            )

    def test_change_connection_id_retransmit_retire_connection_id(self):
        with client_and_server() as (client, server):
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [1, 2, 3, 4, 5, 6, 7]
            )

            # the client changes connection ID, RETIRE_CONNECTION_ID is lost
            client.change_connection_id()
            self.assertEqual(drop(client), 1)
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [2, 3, 4, 5, 6, 7]
            )

            # RETIRE_CONNECTION_ID is retransmitted
            client._on_retire_connection_id_delivery(QuicDeliveryState.LOST, 0)
            self.assertEqual(transfer(client, server), 1)

            # the server provides a new connection ID
            self.assertEqual(transfer(server, client), 1)
            self.assertEqual(
                sequence_numbers(client._peer_cid_available), [2, 3, 4, 5, 6, 7, 8]
            )

    def test_get_next_available_stream_id(self):
        with client_and_server() as (client, server):
            # client
            stream_id = client.get_next_available_stream_id()
            self.assertEqual(stream_id, 0)
            client.send_stream_data(stream_id, b"hello")

            stream_id = client.get_next_available_stream_id()
            self.assertEqual(stream_id, 4)
            client.send_stream_data(stream_id, b"hello")

            stream_id = client.get_next_available_stream_id(is_unidirectional=True)
            self.assertEqual(stream_id, 2)
            client.send_stream_data(stream_id, b"hello")

            stream_id = client.get_next_available_stream_id(is_unidirectional=True)
            self.assertEqual(stream_id, 6)
            client.send_stream_data(stream_id, b"hello")

            # server
            stream_id = server.get_next_available_stream_id()
            self.assertEqual(stream_id, 1)
            server.send_stream_data(stream_id, b"hello")

            stream_id = server.get_next_available_stream_id()
            self.assertEqual(stream_id, 5)
            server.send_stream_data(stream_id, b"hello")

            stream_id = server.get_next_available_stream_id(is_unidirectional=True)
            self.assertEqual(stream_id, 3)
            server.send_stream_data(stream_id, b"hello")

            stream_id = server.get_next_available_stream_id(is_unidirectional=True)
            self.assertEqual(stream_id, 7)
            server.send_stream_data(stream_id, b"hello")

    def test_datagram_frame(self):
        with client_and_server(
            client_options={"max_datagram_frame_size": 65536},
            server_options={"max_datagram_frame_size": 65536},
        ) as (client, server):
            # check handshake completed
            self.check_handshake(client=client, server=server, alpn_protocol=None)

            # send datagram
            client.send_datagram_frame(b"hello")
            self.assertEqual(transfer(client, server), 1)

            event = server.next_event()
            self.assertEqual(type(event), events.DatagramFrameReceived)
            self.assertEqual(event.data, b"hello")

    def test_datagram_frame_2(self):
        # payload which exactly fills an entire packet
        payload = b"Z" * 1250

        with client_and_server(
            client_options={"max_datagram_frame_size": 65536},
            server_options={"max_datagram_frame_size": 65536},
        ) as (client, server):
            # check handshake completed
            self.check_handshake(client=client, server=server, alpn_protocol=None)

            # queue 20 datagrams
            for i in range(20):
                client.send_datagram_frame(payload)

            # client can only 11 datagrams are sent due to congestion control
            self.assertEqual(transfer(client, server), 11)
            for i in range(11):
                event = server.next_event()
                self.assertEqual(type(event), events.DatagramFrameReceived)
                self.assertEqual(event.data, payload)

            # server sends ACK
            self.assertEqual(transfer(server, client), 1)

            # client sends remaining datagrams
            self.assertEqual(transfer(client, server), 9)
            for i in range(9):
                event = server.next_event()
                self.assertEqual(type(event), events.DatagramFrameReceived)
                self.assertEqual(event.data, payload)

    def test_decryption_error(self):
        with client_and_server() as (client, server):
            # mess with encryption key
            server._cryptos[tls.Epoch.ONE_RTT].send.setup(
                cipher_suite=tls.CipherSuite.AES_128_GCM_SHA256,
                secret=bytes(48),
                version=server._version,
            )

            # server sends close
            server.close(error_code=QuicErrorCode.NO_ERROR)
            for data, addr in server.datagrams_to_send(now=time.time()):
                client.receive_datagram(data, SERVER_ADDR, now=time.time())

    def test_tls_error(self):
        def patch(client):
            real_initialize = client._initialize

            def patched_initialize(peer_cid: bytes):
                real_initialize(peer_cid)
                client.tls._supported_versions = [tls.TLS_VERSION_1_3_DRAFT_28]

            client._initialize = patched_initialize

        # handshake fails
        with client_and_server(client_patch=patch) as (client, server):
            timer_at = server.get_timer()
            server.handle_timer(timer_at)

            event = server.next_event()
            self.assertEqual(type(event), events.ConnectionTerminated)
            self.assertEqual(event.error_code, 326)
            self.assertEqual(event.frame_type, QuicFrameType.CRYPTO)
            self.assertEqual(event.reason_phrase, "No supported protocol version")

    def test_receive_datagram_garbage(self):
        client = create_standalone_client(self)

        datagram = binascii.unhexlify("c00000000080")
        client.receive_datagram(datagram, SERVER_ADDR, now=time.time())

    def test_receive_datagram_reserved_bits_non_zero(self):
        client = create_standalone_client(self)

        builder = QuicPacketBuilder(
            host_cid=client._peer_cid,
            is_client=False,
            peer_cid=client.host_cid,
            version=client._version,
        )
        crypto = CryptoPair()
        crypto.setup_initial(client._peer_cid, is_client=False, version=client._version)
        crypto.encrypt_packet_real = crypto.encrypt_packet

        def encrypt_packet(plain_header, plain_payload, packet_number):
            # mess with reserved bits
            plain_header = bytes([plain_header[0] | 0x0C]) + plain_header[1:]
            return crypto.encrypt_packet_real(
                plain_header, plain_payload, packet_number
            )

        crypto.encrypt_packet = encrypt_packet

        builder.start_packet(PACKET_TYPE_INITIAL, crypto)
        buf = builder.start_frame(QuicFrameType.PADDING)
        buf.push_bytes(bytes(builder.remaining_flight_space))

        for datagram in builder.flush()[0]:
            client.receive_datagram(datagram, SERVER_ADDR, now=time.time())
        self.assertEqual(drop(client), 1)
        self.assertEqual(
            client._close_event,
            events.ConnectionTerminated(
                error_code=QuicErrorCode.PROTOCOL_VIOLATION,
                frame_type=None,
                reason_phrase="Reserved bits must be zero",
            ),
        )

    def test_receive_datagram_wrong_version(self):
        client = create_standalone_client(self)

        builder = QuicPacketBuilder(
            host_cid=client._peer_cid,
            is_client=False,
            peer_cid=client.host_cid,
            version=0xFF000011,  # DRAFT_16
        )
        crypto = CryptoPair()
        crypto.setup_initial(client._peer_cid, is_client=False, version=client._version)
        builder.start_packet(PACKET_TYPE_INITIAL, crypto)
        buf = builder.start_frame(QuicFrameType.PADDING)
        buf.push_bytes(bytes(builder.remaining_flight_space))

        for datagram in builder.flush()[0]:
            client.receive_datagram(datagram, SERVER_ADDR, now=time.time())
        self.assertEqual(drop(client), 0)

    def test_receive_datagram_retry(self):
        client = create_standalone_client(self)

        client.receive_datagram(
            encode_quic_retry(
                version=client._version,
                source_cid=binascii.unhexlify("85abb547bf28be97"),
                destination_cid=client.host_cid,
                original_destination_cid=client._peer_cid,
                retry_token=bytes(16),
            ),
            SERVER_ADDR,
            now=time.time(),
        )
        self.assertEqual(drop(client), 1)

    def test_receive_datagram_retry_wrong_destination_cid(self):
        client = create_standalone_client(self)

        client.receive_datagram(
            encode_quic_retry(
                version=client._version,
                source_cid=binascii.unhexlify("85abb547bf28be97"),
                destination_cid=binascii.unhexlify("c98343fe8f5f0ff4"),
                original_destination_cid=client._peer_cid,
                retry_token=bytes(16),
            ),
            SERVER_ADDR,
            now=time.time(),
        )
        self.assertEqual(drop(client), 0)

    def test_handle_ack_frame_ecn(self):
        client = create_standalone_client(self)

        client._handle_ack_frame(
            client_receive_context(client),
            QuicFrameType.ACK_ECN,
            Buffer(data=b"\x00\x02\x00\x00\x00\x00\x00"),
        )

    def test_handle_connection_close_frame(self):
        with client_and_server() as (client, server):
            server.close(
                error_code=QuicErrorCode.PROTOCOL_VIOLATION,
                frame_type=QuicFrameType.ACK,
                reason_phrase="illegal ACK frame",
            )
            self.assertEqual(roundtrip(server, client), (1, 0))

            self.assertEqual(
                client._close_event,
                events.ConnectionTerminated(
                    error_code=QuicErrorCode.PROTOCOL_VIOLATION,
                    frame_type=QuicFrameType.ACK,
                    reason_phrase="illegal ACK frame",
                ),
            )

    def test_handle_connection_close_frame_app(self):
        with client_and_server() as (client, server):
            server.close(error_code=QuicErrorCode.NO_ERROR, reason_phrase="goodbye")
            self.assertEqual(roundtrip(server, client), (1, 0))

            self.assertEqual(
                client._close_event,
                events.ConnectionTerminated(
                    error_code=QuicErrorCode.NO_ERROR,
                    frame_type=None,
                    reason_phrase="goodbye",
                ),
            )

    def test_handle_connection_close_frame_app_not_utf8(self):
        client = create_standalone_client(self)

        client._handle_connection_close_frame(
            client_receive_context(client),
            QuicFrameType.APPLICATION_CLOSE,
            Buffer(data=binascii.unhexlify("0008676f6f6462798200")),
        )

        self.assertEqual(
            client._close_event,
            events.ConnectionTerminated(
                error_code=QuicErrorCode.NO_ERROR, frame_type=None, reason_phrase="",
            ),
        )

    def test_handle_crypto_frame_over_largest_offset(self):
        with client_and_server() as (client, server):
            # client receives offset + length > 2^62 - 1
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_crypto_frame(
                    client_receive_context(client),
                    QuicFrameType.CRYPTO,
                    Buffer(data=encode_uint_var(UINT_VAR_MAX) + encode_uint_var(1)),
                )
            self.assertEqual(
                cm.exception.error_code, QuicErrorCode.FRAME_ENCODING_ERROR
            )
            self.assertEqual(cm.exception.frame_type, QuicFrameType.CRYPTO)
            self.assertEqual(
                cm.exception.reason_phrase, "offset + length cannot exceed 2^62 - 1"
            )

    def test_handle_data_blocked_frame(self):
        with client_and_server() as (client, server):
            # client receives DATA_BLOCKED: 12345
            client._handle_data_blocked_frame(
                client_receive_context(client),
                QuicFrameType.DATA_BLOCKED,
                Buffer(data=encode_uint_var(12345)),
            )

    def test_handle_datagram_frame(self):
        client = create_standalone_client(self, max_datagram_frame_size=6)

        client._handle_datagram_frame(
            client_receive_context(client),
            QuicFrameType.DATAGRAM,
            Buffer(data=b"hello"),
        )

        self.assertEqual(
            client.next_event(), events.DatagramFrameReceived(data=b"hello")
        )

    def test_handle_datagram_frame_not_allowed(self):
        client = create_standalone_client(self, max_datagram_frame_size=None)

        with self.assertRaises(QuicConnectionError) as cm:
            client._handle_datagram_frame(
                client_receive_context(client),
                QuicFrameType.DATAGRAM,
                Buffer(data=b"hello"),
            )
        self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
        self.assertEqual(cm.exception.frame_type, QuicFrameType.DATAGRAM)
        self.assertEqual(cm.exception.reason_phrase, "Unexpected DATAGRAM frame")

    def test_handle_datagram_frame_too_large(self):
        client = create_standalone_client(self, max_datagram_frame_size=5)

        with self.assertRaises(QuicConnectionError) as cm:
            client._handle_datagram_frame(
                client_receive_context(client),
                QuicFrameType.DATAGRAM,
                Buffer(data=b"hello"),
            )
        self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
        self.assertEqual(cm.exception.frame_type, QuicFrameType.DATAGRAM)
        self.assertEqual(cm.exception.reason_phrase, "Unexpected DATAGRAM frame")

    def test_handle_datagram_frame_with_length(self):
        client = create_standalone_client(self, max_datagram_frame_size=7)

        client._handle_datagram_frame(
            client_receive_context(client),
            QuicFrameType.DATAGRAM_WITH_LENGTH,
            Buffer(data=b"\x05hellojunk"),
        )

        self.assertEqual(
            client.next_event(), events.DatagramFrameReceived(data=b"hello")
        )

    def test_handle_datagram_frame_with_length_not_allowed(self):
        client = create_standalone_client(self, max_datagram_frame_size=None)

        with self.assertRaises(QuicConnectionError) as cm:
            client._handle_datagram_frame(
                client_receive_context(client),
                QuicFrameType.DATAGRAM_WITH_LENGTH,
                Buffer(data=b"\x05hellojunk"),
            )
        self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
        self.assertEqual(cm.exception.frame_type, QuicFrameType.DATAGRAM_WITH_LENGTH)
        self.assertEqual(cm.exception.reason_phrase, "Unexpected DATAGRAM frame")

    def test_handle_datagram_frame_with_length_too_large(self):
        client = create_standalone_client(self, max_datagram_frame_size=6)

        with self.assertRaises(QuicConnectionError) as cm:
            client._handle_datagram_frame(
                client_receive_context(client),
                QuicFrameType.DATAGRAM_WITH_LENGTH,
                Buffer(data=b"\x05hellojunk"),
            )
        self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
        self.assertEqual(cm.exception.frame_type, QuicFrameType.DATAGRAM_WITH_LENGTH)
        self.assertEqual(cm.exception.reason_phrase, "Unexpected DATAGRAM frame")

    def test_handle_handshake_done_not_allowed(self):
        with client_and_server() as (client, server):
            # server receives HANDSHAKE_DONE frame
            with self.assertRaises(QuicConnectionError) as cm:
                server._handle_handshake_done_frame(
                    client_receive_context(server),
                    QuicFrameType.HANDSHAKE_DONE,
                    Buffer(data=b""),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.HANDSHAKE_DONE)
            self.assertEqual(
                cm.exception.reason_phrase,
                "Clients must not send HANDSHAKE_DONE frames",
            )

    def test_handle_max_data_frame(self):
        with client_and_server() as (client, server):
            self.assertEqual(client._remote_max_data, 1048576)

            # client receives MAX_DATA raising limit
            client._handle_max_data_frame(
                client_receive_context(client),
                QuicFrameType.MAX_DATA,
                Buffer(data=encode_uint_var(1048577)),
            )
            self.assertEqual(client._remote_max_data, 1048577)

    def test_handle_max_stream_data_frame(self):
        with client_and_server() as (client, server):
            # client creates bidirectional stream 0
            stream = client._create_stream(stream_id=0)
            self.assertEqual(stream.max_stream_data_remote, 1048576)

            # client receives MAX_STREAM_DATA raising limit
            client._handle_max_stream_data_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAM_DATA,
                Buffer(data=b"\x00" + encode_uint_var(1048577)),
            )
            self.assertEqual(stream.max_stream_data_remote, 1048577)

            # client receives MAX_STREAM_DATA lowering limit
            client._handle_max_stream_data_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAM_DATA,
                Buffer(data=b"\x00" + encode_uint_var(1048575)),
            )
            self.assertEqual(stream.max_stream_data_remote, 1048577)

    def test_handle_max_stream_data_frame_receive_only(self):
        with client_and_server() as (client, server):
            # server creates unidirectional stream 3
            server.send_stream_data(stream_id=3, data=b"hello")

            # client receives MAX_STREAM_DATA: 3, 1
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_max_stream_data_frame(
                    client_receive_context(client),
                    QuicFrameType.MAX_STREAM_DATA,
                    Buffer(data=b"\x03\x01"),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.STREAM_STATE_ERROR)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.MAX_STREAM_DATA)
            self.assertEqual(cm.exception.reason_phrase, "Stream is receive-only")

    def test_handle_max_streams_bidi_frame(self):
        with client_and_server() as (client, server):
            self.assertEqual(client._remote_max_streams_bidi, 128)

            # client receives MAX_STREAMS_BIDI raising limit
            client._handle_max_streams_bidi_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAMS_BIDI,
                Buffer(data=encode_uint_var(129)),
            )
            self.assertEqual(client._remote_max_streams_bidi, 129)

            # client receives MAX_STREAMS_BIDI lowering limit
            client._handle_max_streams_bidi_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAMS_BIDI,
                Buffer(data=encode_uint_var(127)),
            )
            self.assertEqual(client._remote_max_streams_bidi, 129)

    def test_handle_max_streams_uni_frame(self):
        with client_and_server() as (client, server):
            self.assertEqual(client._remote_max_streams_uni, 128)

            # client receives MAX_STREAMS_UNI raising limit
            client._handle_max_streams_uni_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAMS_UNI,
                Buffer(data=encode_uint_var(129)),
            )
            self.assertEqual(client._remote_max_streams_uni, 129)

            # client receives MAX_STREAMS_UNI raising limit
            client._handle_max_streams_uni_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAMS_UNI,
                Buffer(data=encode_uint_var(127)),
            )
            self.assertEqual(client._remote_max_streams_uni, 129)

    def test_handle_new_token_frame(self):
        with client_and_server() as (client, server):
            # client receives NEW_TOKEN
            client._handle_new_token_frame(
                client_receive_context(client),
                QuicFrameType.NEW_TOKEN,
                Buffer(data=binascii.unhexlify("080102030405060708")),
            )

    def test_handle_new_token_frame_from_client(self):
        with client_and_server() as (client, server):
            # server receives NEW_TOKEN
            with self.assertRaises(QuicConnectionError) as cm:
                server._handle_new_token_frame(
                    client_receive_context(client),
                    QuicFrameType.NEW_TOKEN,
                    Buffer(data=binascii.unhexlify("080102030405060708")),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.NEW_TOKEN)
            self.assertEqual(
                cm.exception.reason_phrase, "Clients must not send NEW_TOKEN frames"
            )

    def test_handle_path_challenge_frame(self):
        with client_and_server() as (client, server):
            # client changes address and sends some data
            client.send_stream_data(0, b"01234567")
            for data, addr in client.datagrams_to_send(now=time.time()):
                server.receive_datagram(data, ("1.2.3.4", 2345), now=time.time())

            # check paths
            self.assertEqual(len(server._network_paths), 2)
            self.assertEqual(server._network_paths[0].addr, ("1.2.3.4", 2345))
            self.assertFalse(server._network_paths[0].is_validated)
            self.assertEqual(server._network_paths[1].addr, ("1.2.3.4", 1234))
            self.assertTrue(server._network_paths[1].is_validated)

            # server sends PATH_CHALLENGE and receives PATH_RESPONSE
            for data, addr in server.datagrams_to_send(now=time.time()):
                client.receive_datagram(data, SERVER_ADDR, now=time.time())
            for data, addr in client.datagrams_to_send(now=time.time()):
                server.receive_datagram(data, ("1.2.3.4", 2345), now=time.time())

            # check paths
            self.assertEqual(server._network_paths[0].addr, ("1.2.3.4", 2345))
            self.assertTrue(server._network_paths[0].is_validated)
            self.assertEqual(server._network_paths[1].addr, ("1.2.3.4", 1234))
            self.assertTrue(server._network_paths[1].is_validated)

    def test_handle_path_response_frame_bad(self):
        with client_and_server() as (client, server):
            # server receives unsollicited PATH_RESPONSE
            with self.assertRaises(QuicConnectionError) as cm:
                server._handle_path_response_frame(
                    client_receive_context(client),
                    QuicFrameType.PATH_RESPONSE,
                    Buffer(data=b"\x11\x22\x33\x44\x55\x66\x77\x88"),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.PATH_RESPONSE)

    def test_handle_padding_frame(self):
        client = create_standalone_client(self)

        # no more padding
        buf = Buffer(data=b"")
        client._handle_padding_frame(
            client_receive_context(client), QuicFrameType.PADDING, buf
        )
        self.assertEqual(buf.tell(), 0)

        # padding until end
        buf = Buffer(data=bytes(10))
        client._handle_padding_frame(
            client_receive_context(client), QuicFrameType.PADDING, buf
        )
        self.assertEqual(buf.tell(), 10)

        # padding then something else
        buf = Buffer(data=bytes(10) + b"\x01")
        client._handle_padding_frame(
            client_receive_context(client), QuicFrameType.PADDING, buf
        )
        self.assertEqual(buf.tell(), 10)

    def test_handle_reset_stream_frame(self):
        with client_and_server() as (client, server):
            # client creates bidirectional stream 0
            client.send_stream_data(stream_id=0, data=b"hello")
            consume_events(client)

            # client receives RESET_STREAM
            client._handle_reset_stream_frame(
                client_receive_context(client),
                QuicFrameType.RESET_STREAM,
                Buffer(data=binascii.unhexlify("000100")),
            )

            event = client.next_event()
            self.assertEqual(type(event), events.StreamReset)
            self.assertEqual(event.error_code, QuicErrorCode.INTERNAL_ERROR)
            self.assertEqual(event.stream_id, 0)

    def test_handle_reset_stream_frame_send_only(self):
        with client_and_server() as (client, server):
            # client creates unidirectional stream 2
            client.send_stream_data(stream_id=2, data=b"hello")

            # client receives RESET_STREAM
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_reset_stream_frame(
                    client_receive_context(client),
                    QuicFrameType.RESET_STREAM,
                    Buffer(data=binascii.unhexlify("021100")),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.STREAM_STATE_ERROR)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.RESET_STREAM)
            self.assertEqual(cm.exception.reason_phrase, "Stream is send-only")

    def test_handle_retire_connection_id_frame(self):
        with client_and_server() as (client, server):
            self.assertEqual(
                sequence_numbers(client._host_cids), [0, 1, 2, 3, 4, 5, 6, 7]
            )

            # client receives RETIRE_CONNECTION_ID
            client._handle_retire_connection_id_frame(
                client_receive_context(client),
                QuicFrameType.RETIRE_CONNECTION_ID,
                Buffer(data=b"\x02"),
            )
            self.assertEqual(
                sequence_numbers(client._host_cids), [0, 1, 3, 4, 5, 6, 7, 8]
            )

    def test_handle_retire_connection_id_frame_current_cid(self):
        with client_and_server() as (client, server):
            self.assertEqual(
                sequence_numbers(client._host_cids), [0, 1, 2, 3, 4, 5, 6, 7]
            )

            # client receives RETIRE_CONNECTION_ID for the current CID
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_retire_connection_id_frame(
                    client_receive_context(client),
                    QuicFrameType.RETIRE_CONNECTION_ID,
                    Buffer(data=b"\x00"),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
            self.assertEqual(
                cm.exception.frame_type, QuicFrameType.RETIRE_CONNECTION_ID
            )
            self.assertEqual(
                cm.exception.reason_phrase, "Cannot retire current connection ID"
            )
            self.assertEqual(
                sequence_numbers(client._host_cids), [0, 1, 2, 3, 4, 5, 6, 7]
            )

    def test_handle_stop_sending_frame(self):
        with client_and_server() as (client, server):
            # client creates bidirectional stream 0
            client.send_stream_data(stream_id=0, data=b"hello")

            # client receives STOP_SENDING
            client._handle_stop_sending_frame(
                client_receive_context(client),
                QuicFrameType.STOP_SENDING,
                Buffer(data=b"\x00\x11"),
            )

    def test_handle_stop_sending_frame_receive_only(self):
        with client_and_server() as (client, server):
            # server creates unidirectional stream 3
            server.send_stream_data(stream_id=3, data=b"hello")

            # client receives STOP_SENDING
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stop_sending_frame(
                    client_receive_context(client),
                    QuicFrameType.STOP_SENDING,
                    Buffer(data=b"\x03\x11"),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.STREAM_STATE_ERROR)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.STOP_SENDING)
            self.assertEqual(cm.exception.reason_phrase, "Stream is receive-only")

    def test_handle_stream_frame_over_largest_offset(self):
        with client_and_server() as (client, server):
            # client receives offset + length > 2^62 - 1
            frame_type = QuicFrameType.STREAM_BASE | 6
            stream_id = 1
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stream_frame(
                    client_receive_context(client),
                    frame_type,
                    Buffer(
                        data=encode_uint_var(stream_id)
                        + encode_uint_var(UINT_VAR_MAX)
                        + encode_uint_var(1)
                    ),
                )
            self.assertEqual(
                cm.exception.error_code, QuicErrorCode.FRAME_ENCODING_ERROR
            )
            self.assertEqual(cm.exception.frame_type, frame_type)
            self.assertEqual(
                cm.exception.reason_phrase, "offset + length cannot exceed 2^62 - 1"
            )

    def test_handle_stream_frame_over_max_data(self):
        with client_and_server() as (client, server):
            # artificially raise received data counter
            client._local_max_data_used = client._local_max_data

            # client receives STREAM frame
            frame_type = QuicFrameType.STREAM_BASE | 4
            stream_id = 1
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stream_frame(
                    client_receive_context(client),
                    frame_type,
                    Buffer(data=encode_uint_var(stream_id) + encode_uint_var(1)),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.FLOW_CONTROL_ERROR)
            self.assertEqual(cm.exception.frame_type, frame_type)
            self.assertEqual(cm.exception.reason_phrase, "Over connection data limit")

    def test_handle_stream_frame_over_max_stream_data(self):
        with client_and_server() as (client, server):
            # client receives STREAM frame
            frame_type = QuicFrameType.STREAM_BASE | 4
            stream_id = 1
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stream_frame(
                    client_receive_context(client),
                    frame_type,
                    Buffer(
                        data=encode_uint_var(stream_id)
                        + encode_uint_var(client._local_max_stream_data_bidi_remote + 1)
                    ),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.FLOW_CONTROL_ERROR)
            self.assertEqual(cm.exception.frame_type, frame_type)
            self.assertEqual(cm.exception.reason_phrase, "Over stream data limit")

    def test_handle_stream_frame_over_max_streams(self):
        with client_and_server() as (client, server):
            # client receives STREAM frame
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stream_frame(
                    client_receive_context(client),
                    QuicFrameType.STREAM_BASE,
                    Buffer(
                        data=encode_uint_var(client._local_max_stream_data_uni * 4 + 3)
                    ),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.STREAM_LIMIT_ERROR)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.STREAM_BASE)
            self.assertEqual(cm.exception.reason_phrase, "Too many streams open")

    def test_handle_stream_frame_send_only(self):
        with client_and_server() as (client, server):
            # client creates unidirectional stream 2
            client.send_stream_data(stream_id=2, data=b"hello")

            # client receives STREAM frame
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stream_frame(
                    client_receive_context(client),
                    QuicFrameType.STREAM_BASE,
                    Buffer(data=b"\x02"),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.STREAM_STATE_ERROR)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.STREAM_BASE)
            self.assertEqual(cm.exception.reason_phrase, "Stream is send-only")

    def test_handle_stream_frame_wrong_initiator(self):
        with client_and_server() as (client, server):
            # client receives STREAM frame
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stream_frame(
                    client_receive_context(client),
                    QuicFrameType.STREAM_BASE,
                    Buffer(data=b"\x00"),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.STREAM_STATE_ERROR)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.STREAM_BASE)
            self.assertEqual(cm.exception.reason_phrase, "Wrong stream initiator")

    def test_handle_stream_data_blocked_frame(self):
        with client_and_server() as (client, server):
            # client creates bidirectional stream 0
            client.send_stream_data(stream_id=0, data=b"hello")

            # client receives STREAM_DATA_BLOCKED
            client._handle_stream_data_blocked_frame(
                client_receive_context(client),
                QuicFrameType.STREAM_DATA_BLOCKED,
                Buffer(data=b"\x00\x01"),
            )

    def test_handle_stream_data_blocked_frame_send_only(self):
        with client_and_server() as (client, server):
            # client creates unidirectional stream 2
            client.send_stream_data(stream_id=2, data=b"hello")

            # client receives STREAM_DATA_BLOCKED
            with self.assertRaises(QuicConnectionError) as cm:
                client._handle_stream_data_blocked_frame(
                    client_receive_context(client),
                    QuicFrameType.STREAM_DATA_BLOCKED,
                    Buffer(data=b"\x02\x01"),
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.STREAM_STATE_ERROR)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.STREAM_DATA_BLOCKED)
            self.assertEqual(cm.exception.reason_phrase, "Stream is send-only")

    def test_handle_streams_blocked_uni_frame(self):
        with client_and_server() as (client, server):
            # client receives STREAMS_BLOCKED_UNI: 0
            client._handle_streams_blocked_frame(
                client_receive_context(client),
                QuicFrameType.STREAMS_BLOCKED_UNI,
                Buffer(data=b"\x00"),
            )

    def test_payload_received_padding_only(self):
        with client_and_server() as (client, server):
            # client receives padding only
            is_ack_eliciting, is_probing = client._payload_received(
                client_receive_context(client), b"\x00" * 1200
            )
            self.assertFalse(is_ack_eliciting)
            self.assertTrue(is_probing)

    def test_payload_received_unknown_frame(self):
        with client_and_server() as (client, server):
            # client receives unknown frame
            with self.assertRaises(QuicConnectionError) as cm:
                client._payload_received(client_receive_context(client), b"\x1f")
            self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
            self.assertEqual(cm.exception.frame_type, 0x1F)
            self.assertEqual(cm.exception.reason_phrase, "Unknown frame type")

    def test_payload_received_unexpected_frame(self):
        with client_and_server() as (client, server):
            # client receives CRYPTO frame in 0-RTT
            with self.assertRaises(QuicConnectionError) as cm:
                client._payload_received(
                    client_receive_context(client, epoch=tls.Epoch.ZERO_RTT), b"\x06"
                )
            self.assertEqual(cm.exception.error_code, QuicErrorCode.PROTOCOL_VIOLATION)
            self.assertEqual(cm.exception.frame_type, QuicFrameType.CRYPTO)
            self.assertEqual(cm.exception.reason_phrase, "Unexpected frame type")

    def test_payload_received_malformed_frame(self):
        with client_and_server() as (client, server):
            # client receives malformed TRANSPORT_CLOSE frame
            with self.assertRaises(QuicConnectionError) as cm:
                client._payload_received(
                    client_receive_context(client), b"\x1c\x00\x01"
                )
            self.assertEqual(
                cm.exception.error_code, QuicErrorCode.FRAME_ENCODING_ERROR
            )
            self.assertEqual(cm.exception.frame_type, 0x1C)
            self.assertEqual(cm.exception.reason_phrase, "Failed to parse frame")

    def test_send_max_data_blocked_by_cc(self):
        with client_and_server() as (client, server):
            # check congestion control
            self.assertEqual(client._loss.bytes_in_flight, 0)
            self.assertEqual(client._loss.congestion_window, 14303)

            # artificially raise received data counter
            client._local_max_data_used = client._local_max_data
            self.assertEqual(server._remote_max_data, 1048576)

            # artificially raise bytes in flight
            client._loss._cc.bytes_in_flight = 14303

            # MAX_DATA is not sent due to congestion control
            self.assertEqual(drop(client), 0)

    def test_send_max_data_retransmit(self):
        with client_and_server() as (client, server):
            # artificially raise received data counter
            client._local_max_data_used = client._local_max_data
            self.assertEqual(server._remote_max_data, 1048576)

            # MAX_DATA is sent and lost
            self.assertEqual(drop(client), 1)
            self.assertEqual(client._local_max_data_sent, 2097152)
            self.assertEqual(server._remote_max_data, 1048576)

            # MAX_DATA is retransmitted and acked
            client._on_max_data_delivery(QuicDeliveryState.LOST)
            self.assertEqual(client._local_max_data_sent, 0)
            self.assertEqual(roundtrip(client, server), (1, 1))
            self.assertEqual(server._remote_max_data, 2097152)

    def test_send_max_stream_data_retransmit(self):
        with client_and_server() as (client, server):
            # client creates bidirectional stream 0
            stream = client._create_stream(stream_id=0)
            client.send_stream_data(0, b"hello")
            self.assertEqual(stream.max_stream_data_local, 1048576)
            self.assertEqual(stream.max_stream_data_local_sent, 1048576)
            self.assertEqual(roundtrip(client, server), (1, 1))

            # server sends data, just before raising MAX_STREAM_DATA
            server.send_stream_data(0, b"Z" * 524288)  # 1048576 // 2
            for i in range(10):
                roundtrip(server, client)
            self.assertEqual(stream.max_stream_data_local, 1048576)
            self.assertEqual(stream.max_stream_data_local_sent, 1048576)

            # server sends one more byte
            server.send_stream_data(0, b"Z")
            self.assertEqual(transfer(server, client), 1)

            # MAX_STREAM_DATA is sent and lost
            self.assertEqual(drop(client), 1)
            self.assertEqual(stream.max_stream_data_local, 2097152)
            self.assertEqual(stream.max_stream_data_local_sent, 2097152)
            client._on_max_stream_data_delivery(QuicDeliveryState.LOST, stream)
            self.assertEqual(stream.max_stream_data_local, 2097152)
            self.assertEqual(stream.max_stream_data_local_sent, 0)

            # MAX_DATA is retransmitted and acked
            self.assertEqual(roundtrip(client, server), (1, 1))
            self.assertEqual(stream.max_stream_data_local, 2097152)
            self.assertEqual(stream.max_stream_data_local_sent, 2097152)

    def test_send_ping(self):
        with client_and_server() as (client, server):
            consume_events(client)

            # client sends ping, server ACKs it
            client.send_ping(uid=12345)
            self.assertEqual(roundtrip(client, server), (1, 1))

            # check event
            event = client.next_event()
            self.assertEqual(type(event), events.PingAcknowledged)
            self.assertEqual(event.uid, 12345)

    def test_send_ping_retransmit(self):
        with client_and_server() as (client, server):
            consume_events(client)

            # client sends another ping, PING is lost
            client.send_ping(uid=12345)
            self.assertEqual(drop(client), 1)

            # PING is retransmitted and acked
            client._on_ping_delivery(QuicDeliveryState.LOST, (12345,))
            self.assertEqual(roundtrip(client, server), (1, 1))

            # check event
            event = client.next_event()
            self.assertEqual(type(event), events.PingAcknowledged)
            self.assertEqual(event.uid, 12345)

    def test_send_stream_data_over_max_streams_bidi(self):
        with client_and_server() as (client, server):
            # create streams
            for i in range(128):
                stream_id = i * 4
                client.send_stream_data(stream_id, b"")
                self.assertFalse(client._streams[stream_id].is_blocked)
            self.assertEqual(len(client._streams_blocked_bidi), 0)
            self.assertEqual(len(client._streams_blocked_uni), 0)
            self.assertEqual(roundtrip(client, server), (0, 0))

            # create one too many -> STREAMS_BLOCKED
            stream_id = 128 * 4
            client.send_stream_data(stream_id, b"")
            self.assertTrue(client._streams[stream_id].is_blocked)
            self.assertEqual(len(client._streams_blocked_bidi), 1)
            self.assertEqual(len(client._streams_blocked_uni), 0)
            self.assertEqual(roundtrip(client, server), (1, 1))

            # peer raises max streams
            client._handle_max_streams_bidi_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAMS_BIDI,
                Buffer(data=encode_uint_var(129)),
            )
            self.assertFalse(client._streams[stream_id].is_blocked)

    def test_send_stream_data_over_max_streams_uni(self):
        with client_and_server() as (client, server):
            # create streams
            for i in range(128):
                stream_id = i * 4 + 2
                client.send_stream_data(stream_id, b"")
                self.assertFalse(client._streams[stream_id].is_blocked)
            self.assertEqual(len(client._streams_blocked_bidi), 0)
            self.assertEqual(len(client._streams_blocked_uni), 0)
            self.assertEqual(roundtrip(client, server), (0, 0))

            # create one too many -> STREAMS_BLOCKED
            stream_id = 128 * 4 + 2
            client.send_stream_data(stream_id, b"")
            self.assertTrue(client._streams[stream_id].is_blocked)
            self.assertEqual(len(client._streams_blocked_bidi), 0)
            self.assertEqual(len(client._streams_blocked_uni), 1)
            self.assertEqual(roundtrip(client, server), (1, 1))

            # peer raises max streams
            client._handle_max_streams_uni_frame(
                client_receive_context(client),
                QuicFrameType.MAX_STREAMS_UNI,
                Buffer(data=encode_uint_var(129)),
            )
            self.assertFalse(client._streams[stream_id].is_blocked)

    def test_send_stream_data_peer_initiated(self):
        with client_and_server() as (client, server):
            # server creates bidirectional stream
            server.send_stream_data(1, b"hello")
            self.assertEqual(roundtrip(server, client), (1, 1))

            # server creates unidirectional stream
            server.send_stream_data(3, b"hello")
            self.assertEqual(roundtrip(server, client), (1, 1))

            # client creates bidirectional stream
            client.send_stream_data(0, b"hello")
            self.assertEqual(roundtrip(client, server), (1, 1))

            # client sends data on server-initiated bidirectional stream
            client.send_stream_data(1, b"hello")
            self.assertEqual(roundtrip(client, server), (1, 1))

            # client create unidirectional stream
            client.send_stream_data(2, b"hello")
            self.assertEqual(roundtrip(client, server), (1, 1))

            # client tries to send data on server-initial unidirectional stream
            with self.assertRaises(ValueError) as cm:
                client.send_stream_data(3, b"hello")
            self.assertEqual(
                str(cm.exception),
                "Cannot send data on peer-initiated unidirectional stream",
            )

            # client tries to send data on unknown server-initiated bidirectional stream
            with self.assertRaises(ValueError) as cm:
                client.send_stream_data(5, b"hello")
            self.assertEqual(
                str(cm.exception), "Cannot send data on unknown peer-initiated stream"
            )

    def test_stream_direction(self):
        with client_and_server() as (client, server):
            for off in [0, 4, 8]:
                # Client-Initiated, Bidirectional
                self.assertTrue(client._stream_can_receive(off))
                self.assertTrue(client._stream_can_send(off))
                self.assertTrue(server._stream_can_receive(off))
                self.assertTrue(server._stream_can_send(off))

                # Server-Initiated, Bidirectional
                self.assertTrue(client._stream_can_receive(off + 1))
                self.assertTrue(client._stream_can_send(off + 1))
                self.assertTrue(server._stream_can_receive(off + 1))
                self.assertTrue(server._stream_can_send(off + 1))

                # Client-Initiated, Unidirectional
                self.assertFalse(client._stream_can_receive(off + 2))
                self.assertTrue(client._stream_can_send(off + 2))
                self.assertTrue(server._stream_can_receive(off + 2))
                self.assertFalse(server._stream_can_send(off + 2))

                # Server-Initiated, Unidirectional
                self.assertTrue(client._stream_can_receive(off + 3))
                self.assertFalse(client._stream_can_send(off + 3))
                self.assertFalse(server._stream_can_receive(off + 3))
                self.assertTrue(server._stream_can_send(off + 3))

    def test_version_negotiation_fail(self):
        client = create_standalone_client(self)

        # no common version, no retry
        client.receive_datagram(
            encode_quic_version_negotiation(
                source_cid=client._peer_cid,
                destination_cid=client.host_cid,
                supported_versions=[0xFF000011],  # DRAFT_16
            ),
            SERVER_ADDR,
            now=time.time(),
        )
        self.assertEqual(drop(client), 0)

        event = client.next_event()
        self.assertEqual(type(event), events.ConnectionTerminated)
        self.assertEqual(event.error_code, QuicErrorCode.INTERNAL_ERROR)
        self.assertEqual(event.frame_type, None)
        self.assertEqual(
            event.reason_phrase, "Could not find a common protocol version"
        )

    def test_version_negotiation_ok(self):
        client = create_standalone_client(self)

        # found a common version, retry
        client.receive_datagram(
            encode_quic_version_negotiation(
                source_cid=client._peer_cid,
                destination_cid=client.host_cid,
                supported_versions=[client._version],
            ),
            SERVER_ADDR,
            now=time.time(),
        )
        self.assertEqual(drop(client), 1)


class QuicNetworkPathTest(TestCase):
    def test_can_send(self):
        path = QuicNetworkPath(("1.2.3.4", 1234))
        self.assertFalse(path.is_validated)

        # initially, cannot send any data
        self.assertTrue(path.can_send(0))
        self.assertFalse(path.can_send(1))

        # receive some data
        path.bytes_received += 1
        self.assertTrue(path.can_send(0))
        self.assertTrue(path.can_send(1))
        self.assertTrue(path.can_send(2))
        self.assertTrue(path.can_send(3))
        self.assertFalse(path.can_send(4))

        # send some data
        path.bytes_sent += 3
        self.assertTrue(path.can_send(0))
        self.assertFalse(path.can_send(1))
