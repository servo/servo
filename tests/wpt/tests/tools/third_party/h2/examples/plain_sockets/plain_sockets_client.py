#!/usr/bin/env python3

"""
plain_sockets_client.py
~~~~~~~~~~~~~~~~~~~~~~~

Just enough code to send a GET request via h2 to an HTTP/2 server and receive a response body.
This is *not* a complete production-ready HTTP/2 client!
"""

import socket
import ssl
import certifi

import h2.connection
import h2.events


SERVER_NAME = 'http2.golang.org'
SERVER_PORT = 443

# generic socket and ssl configuration
socket.setdefaulttimeout(15)
ctx = ssl.create_default_context(cafile=certifi.where())
ctx.set_alpn_protocols(['h2'])

# open a socket to the server and initiate TLS/SSL
s = socket.create_connection((SERVER_NAME, SERVER_PORT))
s = ctx.wrap_socket(s, server_hostname=SERVER_NAME)

c = h2.connection.H2Connection()
c.initiate_connection()
s.sendall(c.data_to_send())

headers = [
    (':method', 'GET'),
    (':path', '/reqinfo'),
    (':authority', SERVER_NAME),
    (':scheme', 'https'),
]
c.send_headers(1, headers, end_stream=True)
s.sendall(c.data_to_send())

body = b''
response_stream_ended = False
while not response_stream_ended:
    # read raw data from the socket
    data = s.recv(65536 * 1024)
    if not data:
        break

    # feed raw data into h2, and process resulting events
    events = c.receive_data(data)
    for event in events:
        print(event)
        if isinstance(event, h2.events.DataReceived):
            # update flow control so the server doesn't starve us
            c.acknowledge_received_data(event.flow_controlled_length, event.stream_id)
            # more response body data received
            body += event.data
        if isinstance(event, h2.events.StreamEnded):
            # response body completed, let's exit the loop
            response_stream_ended = True
            break
    # send any pending data to the server
    s.sendall(c.data_to_send())

print("Response fully received:")
print(body.decode())

# tell the server we are closing the h2 connection
c.close_connection()
s.sendall(c.data_to_send())

# close the socket
s.close()
