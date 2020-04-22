from unittest import TestCase

from aioquic.h0.connection import H0_ALPN, H0Connection
from aioquic.h3.events import DataReceived, HeadersReceived

from .test_connection import client_and_server, transfer


def h0_client_and_server():
    return client_and_server(
        client_options={"alpn_protocols": H0_ALPN},
        server_options={"alpn_protocols": H0_ALPN},
    )


def h0_transfer(quic_sender, h0_receiver):
    quic_receiver = h0_receiver._quic
    transfer(quic_sender, quic_receiver)

    # process QUIC events
    http_events = []
    event = quic_receiver.next_event()
    while event is not None:
        http_events.extend(h0_receiver.handle_event(event))
        event = quic_receiver.next_event()
    return http_events


class H0ConnectionTest(TestCase):
    def test_connect(self):
        with h0_client_and_server() as (quic_client, quic_server):
            h0_client = H0Connection(quic_client)
            h0_server = H0Connection(quic_server)

            # send request
            stream_id = quic_client.get_next_available_stream_id()
            h0_client.send_headers(
                stream_id=stream_id,
                headers=[
                    (b":method", b"GET"),
                    (b":scheme", b"https"),
                    (b":authority", b"localhost"),
                    (b":path", b"/"),
                ],
            )
            h0_client.send_data(stream_id=stream_id, data=b"", end_stream=True)

            # receive request
            events = h0_transfer(quic_client, h0_server)
            self.assertEqual(len(events), 2)

            self.assertTrue(isinstance(events[0], HeadersReceived))
            self.assertEqual(
                events[0].headers, [(b":method", b"GET"), (b":path", b"/")]
            )
            self.assertEqual(events[0].stream_id, stream_id)
            self.assertEqual(events[0].stream_ended, False)

            self.assertTrue(isinstance(events[1], DataReceived))
            self.assertEqual(events[1].data, b"")
            self.assertEqual(events[1].stream_id, stream_id)
            self.assertEqual(events[1].stream_ended, True)

            # send response
            h0_server.send_headers(
                stream_id=stream_id,
                headers=[
                    (b":status", b"200"),
                    (b"content-type", b"text/html; charset=utf-8"),
                ],
            )
            h0_server.send_data(
                stream_id=stream_id,
                data=b"<html><body>hello</body></html>",
                end_stream=True,
            )

            # receive response
            events = h0_transfer(quic_server, h0_client)
            self.assertEqual(len(events), 2)

            self.assertTrue(isinstance(events[0], HeadersReceived))
            self.assertEqual(events[0].headers, [])
            self.assertEqual(events[0].stream_id, stream_id)
            self.assertEqual(events[0].stream_ended, False)

            self.assertTrue(isinstance(events[1], DataReceived))
            self.assertEqual(events[1].data, b"<html><body>hello</body></html>")
            self.assertEqual(events[1].stream_id, stream_id)
            self.assertEqual(events[1].stream_ended, True)

    def test_headers_only(self):
        with h0_client_and_server() as (quic_client, quic_server):
            h0_client = H0Connection(quic_client)
            h0_server = H0Connection(quic_server)

            # send request
            stream_id = quic_client.get_next_available_stream_id()
            h0_client.send_headers(
                stream_id=stream_id,
                headers=[
                    (b":method", b"HEAD"),
                    (b":scheme", b"https"),
                    (b":authority", b"localhost"),
                    (b":path", b"/"),
                ],
                end_stream=True,
            )

            # receive request
            events = h0_transfer(quic_client, h0_server)
            self.assertEqual(len(events), 2)

            self.assertTrue(isinstance(events[0], HeadersReceived))
            self.assertEqual(
                events[0].headers, [(b":method", b"HEAD"), (b":path", b"/")]
            )
            self.assertEqual(events[0].stream_id, stream_id)
            self.assertEqual(events[0].stream_ended, False)

            self.assertTrue(isinstance(events[1], DataReceived))
            self.assertEqual(events[1].data, b"")
            self.assertEqual(events[1].stream_id, stream_id)
            self.assertEqual(events[1].stream_ended, True)

            # send response
            h0_server.send_headers(
                stream_id=stream_id,
                headers=[
                    (b":status", b"200"),
                    (b"content-type", b"text/html; charset=utf-8"),
                ],
                end_stream=True,
            )

            # receive response
            events = h0_transfer(quic_server, h0_client)
            self.assertEqual(len(events), 2)

            self.assertTrue(isinstance(events[0], HeadersReceived))
            self.assertEqual(events[0].headers, [])
            self.assertEqual(events[0].stream_id, stream_id)
            self.assertEqual(events[0].stream_ended, False)

            self.assertTrue(isinstance(events[1], DataReceived))
            self.assertEqual(events[1].data, b"")
            self.assertEqual(events[1].stream_id, stream_id)
            self.assertEqual(events[1].stream_ended, True)
