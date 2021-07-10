import argparse
import asyncio
import json
import logging
from typing import Dict, Optional

from dnslib.dns import DNSRecord

from aioquic.asyncio import QuicConnectionProtocol, serve
from aioquic.quic.configuration import QuicConfiguration
from aioquic.quic.connection import QuicConnection
from aioquic.quic.events import ProtocolNegotiated, QuicEvent, StreamDataReceived
from aioquic.quic.logger import QuicLogger
from aioquic.tls import SessionTicket

try:
    import uvloop
except ImportError:
    uvloop = None


class DnsConnection:
    def __init__(self, quic: QuicConnection):
        self._quic = quic

    def do_query(self, payload) -> bytes:
        q = DNSRecord.parse(payload)
        return q.send(self.resolver(), 53)

    def resolver(self) -> str:
        return args.resolver

    def handle_event(self, event: QuicEvent) -> None:
        if isinstance(event, StreamDataReceived):
            data = self.do_query(event.data)
            end_stream = False
            self._quic.send_stream_data(event.stream_id, data, end_stream)


class DnsServerProtocol(QuicConnectionProtocol):

    # -00 specifies 'dq', 'doq', and 'doq-h00' (the latter obviously tying to
    # the version of the draft it matches). This is confusing, so we'll just
    # support them all, until future drafts define conflicting behaviour.
    SUPPORTED_ALPNS = ["dq", "doq", "doq-h00"]

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._dns: Optional[DnsConnection] = None

    def quic_event_received(self, event: QuicEvent):
        if isinstance(event, ProtocolNegotiated):
            if event.alpn_protocol in DnsServerProtocol.SUPPORTED_ALPNS:
                self._dns = DnsConnection(self._quic)
        if self._dns is not None:
            self._dns.handle_event(event)


class SessionTicketStore:
    """
    Simple in-memory store for session tickets.
    """

    def __init__(self) -> None:
        self.tickets: Dict[bytes, SessionTicket] = {}

    def add(self, ticket: SessionTicket) -> None:
        self.tickets[ticket.ticket] = ticket

    def pop(self, label: bytes) -> Optional[SessionTicket]:
        return self.tickets.pop(label, None)


if __name__ == "__main__":

    parser = argparse.ArgumentParser(description="DNS over QUIC server")
    parser.add_argument(
        "--host",
        type=str,
        default="::",
        help="listen on the specified address (defaults to ::)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=4784,
        help="listen on the specified port (defaults to 4784)",
    )
    parser.add_argument(
        "-k",
        "--private-key",
        type=str,
        required=True,
        help="load the TLS private key from the specified file",
    )
    parser.add_argument(
        "-c",
        "--certificate",
        type=str,
        required=True,
        help="load the TLS certificate from the specified file",
    )
    parser.add_argument(
        "-r",
        "--resolver",
        type=str,
        default="8.8.8.8",
        help="Upstream Classic DNS resolver to use",
    )
    parser.add_argument(
        "-s",
        "--stateless-retry",
        action="store_true",
        help="send a stateless retry for new connections",
    )
    parser.add_argument(
        "-q", "--quic-log", type=str, help="log QUIC events to a file in QLOG format"
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="increase logging verbosity"
    )

    args = parser.parse_args()

    logging.basicConfig(
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
        level=logging.DEBUG if args.verbose else logging.INFO,
    )

    if args.quic_log:
        quic_logger = QuicLogger()
    else:
        quic_logger = None

    configuration = QuicConfiguration(
        alpn_protocols=["dq"],
        is_client=False,
        max_datagram_frame_size=65536,
        quic_logger=quic_logger,
    )

    configuration.load_cert_chain(args.certificate, args.private_key)

    ticket_store = SessionTicketStore()

    if uvloop is not None:
        uvloop.install()
    loop = asyncio.get_event_loop()
    loop.run_until_complete(
        serve(
            args.host,
            args.port,
            configuration=configuration,
            create_protocol=DnsServerProtocol,
            session_ticket_fetcher=ticket_store.pop,
            session_ticket_handler=ticket_store.add,
            stateless_retry=args.stateless_retry,
        )
    )
    try:
        loop.run_forever()
    except KeyboardInterrupt:
        pass
    finally:
        if configuration.quic_logger is not None:
            with open(args.quic_log, "w") as logger_fp:
                json.dump(configuration.quic_logger.to_dict(), logger_fp, indent=4)
