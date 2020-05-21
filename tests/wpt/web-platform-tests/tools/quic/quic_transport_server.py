#!/usr/bin/env python3
import argparse
import asyncio
import io
import logging
import os
import re
import struct
import urllib.parse
from typing import Dict, Optional

from aioquic.asyncio import QuicConnectionProtocol, serve
from aioquic.quic.configuration import QuicConfiguration
from aioquic.quic.connection import END_STATES
from aioquic.quic.events import StreamDataReceived, QuicEvent
from aioquic.tls import SessionTicket

SERVER_NAME = 'aioquic-transport'

handlers_path = None


class EventHandler:
    def __init__(self, connection: QuicConnectionProtocol, global_dict: Dict):
        self.connection = connection
        self.global_dict = global_dict

    def handle_client_indication(
            self,
            origin: str,
            query: Dict[str, str]) -> None:
        name = 'handle_client_indication'
        if name in self.global_dict:
            self.global_dict[name](self.connection, origin, query)

    def handle_event(self, event: QuicEvent) -> None:
        name = 'handle_event'
        if name in self.global_dict:
            self.global_dict[name](self.connection, event)


class QuicTransportProtocol(QuicConnectionProtocol):
    def __init__(self, *args, **kwargs) -> None:
        super().__init__(*args, **kwargs)
        self.streams = dict()
        self.pending_events = []
        self.client_indication_finished = False
        self.client_indication_data = b''
        self.handler = None

    def quic_event_received(self, event: QuicEvent) -> None:
        prefix = '!!'
        logging.log(logging.INFO, 'QUIC event: %s' % type(event))
        try:
            if (not self.client_indication_finished and
                    isinstance(event, StreamDataReceived) and
                    event.stream_id == 2):
                # client indication process
                prefix = 'Client indication error: '
                self.client_indication_data += event.data
                if len(self.client_indication_data) > 65535:
                    raise Exception('too large data for client indication')
                if event.end_stream:
                    self.process_client_indication()
                    if self.is_closing_or_closed():
                        return
                    prefix = 'Event handling Error: '
                    for e in self.pending_events:
                        self.handler.handle_event(e)
                    self.pending_events.clear()
            elif not self.client_indication_finished:
                self.pending_events.append(event)
            elif self.handler is not None:
                prefix = 'Event handling Error: '
                self.handler.handle_event(event)
        except Exception as e:
            self.handler = None
            logging.log(logging.WARN, prefix + str(e))
            self.close()

    def parse_client_indication(self, bs):
        while True:
            key_b = bs.read(2)
            if len(key_b) == 0:
                return
            length_b = bs.read(2)
            if len(key_b) != 2:
                raise Exception('failed to get "Key" field')
            if len(length_b) != 2:
                raise Exception('failed to get "Length" field')
            key = struct.unpack('!H', key_b)[0]
            length = struct.unpack('!H', length_b)[0]
            value = bs.read(length)
            if len(value) != length:
                raise Exception('truncated "Value" field')
            yield (key, value)

    def process_client_indication(self) -> None:
        origin = None
        origin_string = None
        path = None
        path_string = None
        KEY_ORIGIN = 0
        KEY_PATH = 1
        for (key, value) in self.parse_client_indication(
                io.BytesIO(self.client_indication_data)):
            if key == KEY_ORIGIN:
                origin_string = value.decode()
                origin = urllib.parse.urlparse(origin_string)
            elif key == KEY_PATH:
                path_string = value.decode()
                path = urllib.parse.urlparse(path_string)
            else:
                # We must ignore unrecognized fields.
                pass
        logging.log(logging.INFO,
                    'origin = %s, path = %s' % (origin_string, path_string))
        if origin is None:
            raise Exception('No origin is given')
        if path is None:
            raise Exception('No path is given')
        if origin.scheme != 'https' and origin.scheme != 'http':
            raise Exception('Invalid origin: %s' % origin_string)
        if origin.netloc == '':
            raise Exception('Invalid origin: %s' % origin_string)

        # To make the situation simple we accept only simple path strings.
        m = re.compile('^/([a-zA-Z0-9\._\-]+)$').match(path.path)
        if m is None:
            raise Exception('Invalid path: %s' % path_string)

        handler_name = m.group(1)
        query = dict(urllib.parse.parse_qsl(path.query))
        self.handler = self.create_event_handler(handler_name)
        self.handler.handle_client_indication(origin_string, query)
        if self.is_closing_or_closed():
            return
        self.client_indication_finished = True
        logging.log(logging.INFO, 'Client indication finished')

    def create_event_handler(self, handler_name: str) -> None:
        global_dict = {}
        with open(handlers_path + '/' + handler_name) as f:
            exec(f.read(), global_dict)
        return EventHandler(self, global_dict)

    def is_closing_or_closed(self) -> bool:
        if self._quic._close_pending:
            return True
        if self._quic._state in END_STATES:
            return True
        return False


class SessionTicketStore:
    '''
    Simple in-memory store for session tickets.
    '''

    def __init__(self) -> None:
        self.tickets: Dict[bytes, SessionTicket] = {}

    def add(self, ticket: SessionTicket) -> None:
        self.tickets[ticket.ticket] = ticket

    def pop(self, label: bytes) -> Optional[SessionTicket]:
        return self.tickets.pop(label, None)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='QUIC server')
    parser.add_argument(
        '-c',
        '--certificate',
        type=str,
        required=True,
        help='load the TLS certificate from the specified file',
    )
    parser.add_argument(
        '--host',
        type=str,
        default='::',
        help='listen on the specified address (defaults to ::)',
    )
    parser.add_argument(
        '--port',
        type=int,
        default=4433,
        help='listen on the specified port (defaults to 4433)',
    )
    parser.add_argument(
        '-k',
        '--private-key',
        type=str,
        required=True,
        help='load the TLS private key from the specified file',
    )
    parser.add_argument(
        '--handlers-path',
        type=str,
        required=True,
        help='the directory path of QuicTransport event handlers',
    )
    parser.add_argument(
        '-v',
        '--verbose',
        action='store_true',
        help='increase logging verbosity'
    )
    args = parser.parse_args()

    logging.basicConfig(
        format='%(asctime)s %(levelname)s %(name)s %(message)s',
        level=logging.DEBUG if args.verbose else logging.INFO,
    )

    configuration = QuicConfiguration(
        alpn_protocols=['wq-vvv-01'] + ['siduck'],
        is_client=False,
        max_datagram_frame_size=65536,
    )

    handlers_path = os.path.abspath(os.path.expanduser(args.handlers_path))
    logging.log(logging.INFO, 'port = %s' % args.port)
    logging.log(logging.INFO, 'handlers path = %s' % handlers_path)

    # load SSL certificate and key
    configuration.load_cert_chain(args.certificate, args.private_key)

    ticket_store = SessionTicketStore()

    loop = asyncio.get_event_loop()
    loop.run_until_complete(
        serve(
            args.host,
            args.port,
            configuration=configuration,
            create_protocol=QuicTransportProtocol,
            session_ticket_fetcher=ticket_store.pop,
            session_ticket_handler=ticket_store.add,
        )
    )
    try:
        loop.run_forever()
    except KeyboardInterrupt:
        pass
