import argparse
import asyncio
import json
import logging
import pickle
import sys
import time
from collections import deque
from typing import Deque, Dict, cast
from urllib.parse import urlparse

from httpx import AsyncClient
from httpx.config import Timeout
from httpx.dispatch.base import AsyncDispatcher
from httpx.models import Request, Response

from aioquic.asyncio.client import connect
from aioquic.asyncio.protocol import QuicConnectionProtocol
from aioquic.h3.connection import H3_ALPN, H3Connection
from aioquic.h3.events import DataReceived, H3Event, HeadersReceived
from aioquic.quic.configuration import QuicConfiguration
from aioquic.quic.events import QuicEvent
from aioquic.quic.logger import QuicLogger

logger = logging.getLogger("client")


class H3Dispatcher(QuicConnectionProtocol, AsyncDispatcher):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self._http = H3Connection(self._quic)
        self._request_events: Dict[int, Deque[H3Event]] = {}
        self._request_waiter: Dict[int, asyncio.Future[Deque[H3Event]]] = {}

    async def send(self, request: Request, timeout: Timeout = None) -> Response:
        stream_id = self._quic.get_next_available_stream_id()

        # prepare request
        self._http.send_headers(
            stream_id=stream_id,
            headers=[
                (b":method", request.method.encode()),
                (b":scheme", request.url.scheme.encode()),
                (b":authority", str(request.url.authority).encode()),
                (b":path", request.url.full_path.encode()),
            ]
            + [
                (k.encode(), v.encode())
                for (k, v) in request.headers.items()
                if k not in ("connection", "host")
            ],
        )
        self._http.send_data(stream_id=stream_id, data=request.read(), end_stream=True)

        # transmit request
        waiter = self._loop.create_future()
        self._request_events[stream_id] = deque()
        self._request_waiter[stream_id] = waiter
        self.transmit()

        # process response
        events: Deque[H3Event] = await asyncio.shield(waiter)
        content = b""
        headers = []
        status_code = None
        for event in events:
            if isinstance(event, HeadersReceived):
                for header, value in event.headers:
                    if header == b":status":
                        status_code = int(value.decode())
                    elif header[0:1] != b":":
                        headers.append((header.decode(), value.decode()))
            elif isinstance(event, DataReceived):
                content += event.data

        return Response(
            status_code=status_code,
            http_version="HTTP/3",
            headers=headers,
            content=content,
            request=request,
        )

    def http_event_received(self, event: H3Event):
        if isinstance(event, (HeadersReceived, DataReceived)):
            stream_id = event.stream_id
            if stream_id in self._request_events:
                self._request_events[event.stream_id].append(event)
                if event.stream_ended:
                    request_waiter = self._request_waiter.pop(stream_id)
                    request_waiter.set_result(self._request_events.pop(stream_id))

    def quic_event_received(self, event: QuicEvent):
        #  pass event to the HTTP layer
        if self._http is not None:
            for http_event in self._http.handle_event(event):
                self.http_event_received(http_event)


def save_session_ticket(ticket):
    """
    Callback which is invoked by the TLS engine when a new session ticket
    is received.
    """
    logger.info("New session ticket received")
    if args.session_ticket:
        with open(args.session_ticket, "wb") as fp:
            pickle.dump(ticket, fp)


async def run(configuration: QuicConfiguration, url: str, data: str) -> None:
    # parse URL
    parsed = urlparse(url)
    assert parsed.scheme == "https", "Only https:// URLs are supported."
    if ":" in parsed.netloc:
        host, port_str = parsed.netloc.split(":")
        port = int(port_str)
    else:
        host = parsed.netloc
        port = 443

    async with connect(
        host,
        port,
        configuration=configuration,
        create_protocol=H3Dispatcher,
        session_ticket_handler=save_session_ticket,
    ) as dispatch:
        client = AsyncClient(dispatch=cast(AsyncDispatcher, dispatch))

        # perform request
        start = time.time()
        if data is not None:
            response = await client.post(
                url,
                data=data.encode(),
                headers={"content-type": "application/x-www-form-urlencoded"},
            )
        else:
            response = await client.get(url)

        elapsed = time.time() - start

        # print speed
        octets = len(response.content)
        logger.info(
            "Received %d bytes in %.1f s (%.3f Mbps)"
            % (octets, elapsed, octets * 8 / elapsed / 1000000)
        )

        # print response
        for header, value in response.headers.items():
            sys.stderr.write(header + ": " + value + "\r\n")
        sys.stderr.write("\r\n")
        sys.stdout.buffer.write(response.content)
        sys.stdout.buffer.flush()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="HTTP/3 client")
    parser.add_argument("url", type=str, help="the URL to query (must be HTTPS)")
    parser.add_argument(
        "-d", "--data", type=str, help="send the specified data in a POST request"
    )
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

    # prepare configuration
    configuration = QuicConfiguration(is_client=True, alpn_protocols=H3_ALPN)
    if args.quic_log:
        configuration.quic_logger = QuicLogger()
    if args.secrets_log:
        configuration.secrets_log_file = open(args.secrets_log, "a")
    if args.session_ticket:
        try:
            with open(args.session_ticket, "rb") as fp:
                configuration.session_ticket = pickle.load(fp)
        except FileNotFoundError:
            pass

    loop = asyncio.get_event_loop()
    try:
        loop.run_until_complete(
            run(configuration=configuration, url=args.url, data=args.data)
        )
    finally:
        if configuration.quic_logger is not None:
            with open(args.quic_log, "w") as logger_fp:
                json.dump(configuration.quic_logger.to_dict(), logger_fp, indent=4)
