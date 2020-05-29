import argparse
import asyncio
import json
import logging
import pickle
import ssl
from typing import Optional, cast

from dnslib.dns import QTYPE, DNSQuestion, DNSRecord

from aioquic.asyncio.client import connect
from aioquic.asyncio.protocol import QuicConnectionProtocol
from aioquic.quic.configuration import QuicConfiguration
from aioquic.quic.events import QuicEvent, StreamDataReceived
from aioquic.quic.logger import QuicLogger

logger = logging.getLogger("client")


class DoQClient(QuicConnectionProtocol):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self._ack_waiter: Optional[asyncio.Future[None]] = None

    async def query(self, query_type: str, dns_query: str) -> None:
        query = DNSRecord(q=DNSQuestion(dns_query, getattr(QTYPE, query_type)))
        stream_id = self._quic.get_next_available_stream_id()
        logger.debug(f"Stream ID: {stream_id}")
        end_stream = False
        self._quic.send_stream_data(stream_id, bytes(query.pack()), end_stream)
        waiter = self._loop.create_future()
        self._ack_waiter = waiter
        self.transmit()

        return await asyncio.shield(waiter)

    def quic_event_received(self, event: QuicEvent) -> None:
        if self._ack_waiter is not None:
            if isinstance(event, StreamDataReceived):
                answer = DNSRecord.parse(event.data)
                logger.info(answer)
                waiter = self._ack_waiter
                self._ack_waiter = None
                waiter.set_result(None)


def save_session_ticket(ticket):
    """
    Callback which is invoked by the TLS engine when a new session ticket
    is received.
    """
    logger.info("New session ticket received")
    if args.session_ticket:
        with open(args.session_ticket, "wb") as fp:
            pickle.dump(ticket, fp)


async def run(
    configuration: QuicConfiguration,
    host: str,
    port: int,
    query_type: str,
    dns_query: str,
) -> None:
    logger.debug(f"Connecting to {host}:{port}")
    async with connect(
        host,
        port,
        configuration=configuration,
        session_ticket_handler=save_session_ticket,
        create_protocol=DoQClient,
    ) as client:
        client = cast(DoQClient, client)
        logger.debug("Sending DNS query")
        await client.query(query_type, dns_query)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="DNS over QUIC client")
    parser.add_argument("-t", "--type", type=str, help="Type of record to ")
    parser.add_argument(
        "--host",
        type=str,
        default="localhost",
        help="The remote peer's host name or IP address",
    )
    parser.add_argument(
        "--port", type=int, default=784, help="The remote peer's port number"
    )
    parser.add_argument(
        "-k",
        "--insecure",
        action="store_true",
        help="do not validate server certificate",
    )
    parser.add_argument(
        "--ca-certs", type=str, help="load CA certificates from the specified file"
    )
    parser.add_argument("--dns_type", help="The DNS query type to send")
    parser.add_argument("--query", help="Domain to query")
    parser.add_argument(
        "-q", "--quic-log", type=str, help="log QUIC events to a file in QLOG format"
    )
    parser.add_argument(
        "-l",
        "--secrets-log",
        type=str,
        help="log secrets to a file, for use with Wireshark",
    )
    parser.add_argument(
        "-s",
        "--session-ticket",
        type=str,
        help="read and write session ticket from the specified file",
    )
    parser.add_argument(
        "-v", "--verbose", action="store_true", help="increase logging verbosity"
    )

    args = parser.parse_args()

    logging.basicConfig(
        format="%(asctime)s %(levelname)s %(name)s %(message)s",
        level=logging.DEBUG if args.verbose else logging.INFO,
    )

    configuration = QuicConfiguration(
        alpn_protocols=["dq"], is_client=True, max_datagram_frame_size=65536
    )
    if args.ca_certs:
        configuration.load_verify_locations(args.ca_certs)
    if args.insecure:
        configuration.verify_mode = ssl.CERT_NONE
    if args.quic_log:
        configuration.quic_logger = QuicLogger()
    if args.secrets_log:
        configuration.secrets_log_file = open(args.secrets_log, "a")
    if args.session_ticket:
        try:
            with open(args.session_ticket, "rb") as fp:
                configuration.session_ticket = pickle.load(fp)
        except FileNotFoundError:
            logger.debug(f"Unable to read {args.session_ticket}")
            pass
    else:
        logger.debug("No session ticket defined...")

    loop = asyncio.get_event_loop()
    try:
        loop.run_until_complete(
            run(
                configuration=configuration,
                host=args.host,
                port=args.port,
                query_type=args.dns_type,
                dns_query=args.query,
            )
        )
    finally:
        if configuration.quic_logger is not None:
            with open(args.quic_log, "w") as logger_fp:
                json.dump(configuration.quic_logger.to_dict(), logger_fp, indent=4)
