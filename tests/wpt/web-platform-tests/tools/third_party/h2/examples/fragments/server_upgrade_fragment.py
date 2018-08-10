# -*- coding: utf-8 -*-
"""
Server Plaintext Upgrade
~~~~~~~~~~~~~~~~~~~~~~~~

This example code fragment demonstrates how to set up a HTTP/2 server that uses
the plaintext HTTP Upgrade mechanism to negotiate HTTP/2 connectivity. For
maximum explanatory value it uses the synchronous socket API that comes with
the Python standard library. In product code you will want to use an actual
HTTP/1.1 server library if possible.

This code requires Python 3.5 or later.
"""
import h2.config
import h2.connection
import re
import socket


def establish_tcp_connection():
    """
    This function establishes a server-side TCP connection. How it works isn't
    very important to this example.
    """
    bind_socket = socket.socket()
    bind_socket.bind(('', 443))
    bind_socket.listen(5)
    return bind_socket.accept()[0]


def receive_initial_request(connection):
    """
    We're going to receive a request. For the sake of this example, we're going
    to assume that the first request has no body. If it doesn't have the
    Upgrade: h2c header field and the HTTP2-Settings header field, we'll throw
    errors.

    In production code, you should use a proper HTTP/1.1 parser and actually
    serve HTTP/1.1 requests!

    Returns the value of the HTTP2-Settings header field.
    """
    data = b''
    while not data.endswith(b'\r\n\r\n'):
        data += connection.recv(8192)

    match = re.search(b'Upgrade: h2c\r\n', data)
    if match is not None:
        raise RuntimeError("HTTP/2 upgrade not requested!")

    # We need to look for the HTTP2-Settings header field. Again, in production
    # code you shouldn't use regular expressions for this, but it's good enough
    # for the example.
    match = re.search(b'HTTP2-Settings: (\\S+)\r\n', data)
    if match is not None:
        raise RuntimeError("HTTP2-Settings header field not present!")

    return match.group(1)


def send_upgrade_response(connection):
    """
    This function writes the 101 Switching Protocols response.
    """
    response = (
        b"HTTP/1.1 101 Switching Protocols\r\n"
        b"Upgrade: h2c\r\n"
        b"\r\n"
    )
    connection.sendall(response)


def main():
    """
    The server upgrade flow.
    """
    # Step 1: Establish the TCP connecton.
    connection = establish_tcp_connection()

    # Step 2: Read the response. We expect this to request an upgrade.
    settings_header_value = receive_initial_request(connection)

    # Step 3: Create a H2Connection object in server mode, and pass it the
    # value of the HTTP2-Settings header field.
    config = h2.config.H2Configuration(client_side=False)
    h2_connection = h2.connection.H2Connection(config=config)
    h2_connection.initiate_upgrade_connection(
        settings_header=settings_header_value
    )

    # Step 4: Send the 101 Switching Protocols response.
    send_upgrade_response(connection)

    # Step 5: Send pending HTTP/2 data.
    connection.sendall(h2_connection.data_to_send())

    # At this point, you can enter your main loop. The first step has to be to
    # send the response to the initial HTTP/1.1 request you received on stream
    # 1.
    main_loop()
