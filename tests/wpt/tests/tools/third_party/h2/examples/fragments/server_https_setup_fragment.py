# -*- coding: utf-8 -*-
"""
Server HTTPS Setup
~~~~~~~~~~~~~~~~~~

This example code fragment demonstrates how to set up a HTTP/2 server that
negotiates HTTP/2 using NPN and ALPN. For the sake of maximum explanatory value
this code uses the synchronous, low-level sockets API: however, if you're not
using sockets directly (e.g. because you're using asyncio), you should focus on
the set up required for the SSLContext object. For other concurrency libraries
you may need to use other setup (e.g. for Twisted you'll need to use
IProtocolNegotiationFactory).

This code requires Python 3.5 or later.
"""
import h2.config
import h2.connection
import socket
import ssl


def establish_tcp_connection():
    """
    This function establishes a server-side TCP connection. How it works isn't
    very important to this example.
    """
    bind_socket = socket.socket()
    bind_socket.bind(('', 443))
    bind_socket.listen(5)
    return bind_socket.accept()[0]


def get_http2_ssl_context():
    """
    This function creates an SSLContext object that is suitably configured for
    HTTP/2. If you're working with Python TLS directly, you'll want to do the
    exact same setup as this function does.
    """
    # Get the basic context from the standard library.
    ctx = ssl.create_default_context(purpose=ssl.Purpose.CLIENT_AUTH)

    # RFC 7540 Section 9.2: Implementations of HTTP/2 MUST use TLS version 1.2
    # or higher. Disable TLS 1.1 and lower.
    ctx.options |= (
        ssl.OP_NO_SSLv2 | ssl.OP_NO_SSLv3 | ssl.OP_NO_TLSv1 | ssl.OP_NO_TLSv1_1
    )

    # RFC 7540 Section 9.2.1: A deployment of HTTP/2 over TLS 1.2 MUST disable
    # compression.
    ctx.options |= ssl.OP_NO_COMPRESSION

    # RFC 7540 Section 9.2.2: "deployments of HTTP/2 that use TLS 1.2 MUST
    # support TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256". In practice, the
    # blocklist defined in this section allows only the AES GCM and ChaCha20
    # cipher suites with ephemeral key negotiation.
    ctx.set_ciphers("ECDHE+AESGCM:ECDHE+CHACHA20:DHE+AESGCM:DHE+CHACHA20")

    # We want to negotiate using NPN and ALPN. ALPN is mandatory, but NPN may
    # be absent, so allow that. This setup allows for negotiation of HTTP/1.1.
    ctx.set_alpn_protocols(["h2", "http/1.1"])

    try:
        ctx.set_npn_protocols(["h2", "http/1.1"])
    except NotImplementedError:
        pass

    return ctx


def negotiate_tls(tcp_conn, context):
    """
    Given an established TCP connection and a HTTP/2-appropriate TLS context,
    this function:

    1. wraps TLS around the TCP connection.
    2. confirms that HTTP/2 was negotiated and, if it was not, throws an error.
    """
    tls_conn = context.wrap_socket(tcp_conn, server_side=True)

    # Always prefer the result from ALPN to that from NPN.
    # You can only check what protocol was negotiated once the handshake is
    # complete.
    negotiated_protocol = tls_conn.selected_alpn_protocol()
    if negotiated_protocol is None:
        negotiated_protocol = tls_conn.selected_npn_protocol()

    if negotiated_protocol != "h2":
        raise RuntimeError("Didn't negotiate HTTP/2!")

    return tls_conn


def main():
    # Step 1: Set up your TLS context.
    context = get_http2_ssl_context()

    # Step 2: Receive a TCP connection.
    connection = establish_tcp_connection()

    # Step 3: Wrap the connection in TLS and validate that we negotiated HTTP/2
    tls_connection = negotiate_tls(connection, context)

    # Step 4: Create a server-side H2 connection.
    config = h2.config.H2Configuration(client_side=False)
    http2_connection = h2.connection.H2Connection(config=config)

    # Step 5: Initiate the connection
    http2_connection.initiate_connection()
    tls_connection.sendall(http2_connection.data_to_send())

    # The TCP, TLS, and HTTP/2 handshakes are now complete. You can enter your
    # main loop now.
