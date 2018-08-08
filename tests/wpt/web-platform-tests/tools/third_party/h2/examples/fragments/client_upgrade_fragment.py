# -*- coding: utf-8 -*-
"""
Client Plaintext Upgrade
~~~~~~~~~~~~~~~~~~~~~~~~

This example code fragment demonstrates how to set up a HTTP/2 client that uses
the plaintext HTTP Upgrade mechanism to negotiate HTTP/2 connectivity. For
maximum explanatory value it uses the synchronous socket API that comes with
the Python standard library. In product code you will want to use an actual
HTTP/1.1 client if possible.

This code requires Python 3.5 or later.
"""
import h2.connection
import socket


def establish_tcp_connection():
    """
    This function establishes a client-side TCP connection. How it works isn't
    very important to this example. For the purpose of this example we connect
    to localhost.
    """
    return socket.create_connection(('localhost', 80))


def send_initial_request(connection, settings):
    """
    For the sake of this upgrade demonstration, we're going to issue a GET
    request against the root of the site. In principle the best request to
    issue for an upgrade is actually ``OPTIONS *``, but this is remarkably
    poorly supported and can break in weird ways.
    """
    # Craft our initial request per RFC 7540 Section 3.2. This requires two
    # special header fields: the Upgrade headre, and the HTTP2-Settings header.
    # The value of the HTTP2-Settings header field comes from h2.
    request = (
        b"GET / HTTP/1.1\r\n" +
        b"Host: localhost\r\n" +
        b"Upgrade: h2c\r\n" +
        b"HTTP2-Settings: " + settings + "\r\n"
        b"\r\n"
    )
    connection.sendall(request)


def get_upgrade_response(connection):
    """
    This function reads from the socket until the HTTP/1.1 end-of-headers
    sequence (CRLFCRLF) is received. It then checks what the status code of the
    response is.

    This is not a substitute for proper HTTP/1.1 parsing, but it's good enough
    for example purposes.
    """
    data = b''
    while b'\r\n\r\n' not in data:
        data += connection.recv(8192)

    headers, rest = data.split(b'\r\n\r\n', 1)

    # An upgrade response begins HTTP/1.1 101 Switching Protocols. Look for the
    # code. In production code you should also check that the upgrade is to
    # h2c, but here we know we only offered one upgrade so there's only one
    # possible upgrade in use.
    split_headers = headers.split()
    if split_headers[1] != b'101':
        raise RuntimeError("Not upgrading!")

    # We don't care about the HTTP/1.1 data anymore, but we do care about
    # any other data we read from the socket: this is going to be HTTP/2 data
    # that must be passed to the H2Connection.
    return rest


def main():
    """
    The client upgrade flow.
    """
    # Step 1: Establish the TCP connecton.
    connection = establish_tcp_connection()

    # Step 2: Create H2 Connection object, put it in upgrade mode, and get the
    # value of the HTTP2-Settings header we want to use.
    h2_connection = h2.connection.H2Connection()
    settings_header_value = h2_connection.initiate_upgrade_connection()

    # Step 3: Send the initial HTTP/1.1 request with the upgrade fields.
    send_initial_request(connection, settings_header_value)

    # Step 4: Read the HTTP/1.1 response, look for 101 response.
    extra_data = get_upgrade_response(connection)

    # Step 5: Immediately send the pending HTTP/2 data.
    connection.sendall(h2_connection.data_to_send())

    # Step 6: Feed the body data to the connection.
    events = connection.receive_data(extra_data)

    # Now you can enter your main loop, beginning by processing the first set
    # of events above. These events may include ResponseReceived, which will
    # contain the response to the request we made in Step 3.
    main_loop(events)
