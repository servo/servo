# -*- coding: utf-8 -*-
"""
hyper/tls
~~~~~~~~~

Contains the TLS/SSL logic for use in hyper.
"""
import os.path as path
from .common.exceptions import MissingCertFile
from .compat import ignore_missing, ssl


NPN_PROTOCOL = 'h2'
H2_NPN_PROTOCOLS = [NPN_PROTOCOL, 'h2-16', 'h2-15', 'h2-14']
SUPPORTED_NPN_PROTOCOLS = H2_NPN_PROTOCOLS + ['http/1.1']

H2C_PROTOCOL = 'h2c'

# We have a singleton SSLContext object. There's no reason to be creating one
# per connection.
_context = None

# Work out where our certificates are.
cert_loc = path.join(path.dirname(__file__), 'certs.pem')


def wrap_socket(sock, server_hostname, ssl_context=None, force_proto=None):
    """
    A vastly simplified SSL wrapping function. We'll probably extend this to
    do more things later.
    """

    global _context

    if ssl_context:
        # if an SSLContext is provided then use it instead of default context
        _ssl_context = ssl_context
    else:
        # create the singleton SSLContext we use
        if _context is None:  # pragma: no cover
            _context = init_context()
        _ssl_context = _context

    # the spec requires SNI support
    ssl_sock = _ssl_context.wrap_socket(sock, server_hostname=server_hostname)
    # Setting SSLContext.check_hostname to True only verifies that the
    # post-handshake servername matches that of the certificate. We also need
    # to check that it matches the requested one.
    if _ssl_context.check_hostname:  # pragma: no cover
        try:
            ssl.match_hostname(ssl_sock.getpeercert(), server_hostname)
        except AttributeError:
            ssl.verify_hostname(ssl_sock, server_hostname)  # pyopenssl

    # Allow for the protocol to be forced externally.
    proto = force_proto

    # ALPN is newer, so we prefer it over NPN. The odds of us getting
    # different answers is pretty low, but let's be sure.
    with ignore_missing():
        if proto is None:
            proto = ssl_sock.selected_alpn_protocol()

    with ignore_missing():
        if proto is None:
            proto = ssl_sock.selected_npn_protocol()

    return (ssl_sock, proto)


def init_context(cert_path=None, cert=None, cert_password=None):
    """
    Create a new ``SSLContext`` that is correctly set up for an HTTP/2
    connection. This SSL context object can be customized and passed as a
    parameter to the :class:`HTTPConnection <hyper.HTTPConnection>` class.
    Provide your own certificate file in case you don’t want to use hyper’s
    default certificate. The path to the certificate can be absolute or
    relative to your working directory.

    :param cert_path: (optional) The path to the certificate file of
        “certification authority” (CA) certificates
    :param cert: (optional) if string, path to ssl client cert file (.pem).
        If tuple, ('cert', 'key') pair.
        The certfile string must be the path to a single file in PEM format
        containing the certificate as well as any number of CA certificates
        needed to establish the certificate’s authenticity. The keyfile string,
        if present, must point to a file containing the private key in.
        Otherwise the private key will be taken from certfile as well.
    :param cert_password: (optional) The password argument may be a function to
        call to get the password for decrypting the private key. It will only
        be called if the private key is encrypted and a password is necessary.
        It will be called with no arguments, and it should return a string,
        bytes, or bytearray. If the return value is a string it will be
        encoded as UTF-8 before using it to decrypt the key. Alternatively a
        string, bytes, or bytearray value may be supplied directly as the
        password argument. It will be ignored if the private key is not
        encrypted and no password is needed.
    :returns: An ``SSLContext`` correctly set up for HTTP/2.
    """
    cafile = cert_path or cert_loc
    if not cafile or not path.exists(cafile):
        err_msg = ("No certificate found at " + str(cafile) + ". Either " +
                   "ensure the default cert.pem file is included in the " +
                   "distribution or provide a custom certificate when " +
                   "creating the connection.")
        raise MissingCertFile(err_msg)

    context = ssl.SSLContext(ssl.PROTOCOL_SSLv23)
    context.set_default_verify_paths()
    context.load_verify_locations(cafile=cafile)
    context.verify_mode = ssl.CERT_REQUIRED
    context.check_hostname = True

    with ignore_missing():
        context.set_npn_protocols(SUPPORTED_NPN_PROTOCOLS)

    with ignore_missing():
        context.set_alpn_protocols(SUPPORTED_NPN_PROTOCOLS)

    # required by the spec
    context.options |= ssl.OP_NO_COMPRESSION

    if cert is not None:
        try:
            basestring
        except NameError:
            basestring = (str, bytes)
        if not isinstance(cert, basestring):
            context.load_cert_chain(cert[0], cert[1], cert_password)
        else:
            context.load_cert_chain(cert, password=cert_password)

    return context
