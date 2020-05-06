import socket
import sys

from six import binary_type, text_type


def isomorphic_decode(s):
    """Decodes a binary string into a text string using iso-8859-1.

    Returns `unicode` in Python 2 and `str` in Python 3. The function is a
    no-op if the argument already has a text type. iso-8859-1 is chosen because
    it is an 8-bit encoding whose code points range from 0x0 to 0xFF and the
    values are the same as the binary representations, so any binary string can
    be decoded into and encoded from iso-8859-1 without any errors or data
    loss. Python 3 also uses iso-8859-1 (or latin-1) extensively in http:
    https://github.com/python/cpython/blob/273fc220b25933e443c82af6888eb1871d032fb8/Lib/http/client.py#L213
    """
    if isinstance(s, text_type):
        return s

    if isinstance(s, binary_type):
        return s.decode("iso-8859-1")

    raise TypeError("Unexpected value (expecting string-like): %r" % s)


def isomorphic_encode(s):
    """Encodes a text-type string into binary data using iso-8859-1.

    Returns `str` in Python 2 and `bytes` in Python 3. The function is a no-op
    if the argument already has a binary type. This is the counterpart of
    isomorphic_decode.
    """
    if isinstance(s, binary_type):
        return s

    if isinstance(s, text_type):
        return s.encode("iso-8859-1")

    raise TypeError("Unexpected value (expecting string-like): %r" % s)


def invert_dict(dict):
    rv = {}
    for key, values in dict.items():
        for value in values:
            if value in rv:
                raise ValueError
            rv[value] = key
    return rv


class HTTPException(Exception):
    def __init__(self, code, message=""):
        self.code = code
        self.message = message


def _open_socket(host, port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    if port != 0:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind((host, port))
    sock.listen(5)
    return sock


def is_bad_port(port):
    """
    Bad port as per https://fetch.spec.whatwg.org/#port-blocking
    """
    return port in [
        1,     # tcpmux
        7,     # echo
        9,     # discard
        11,    # systat
        13,    # daytime
        15,    # netstat
        17,    # qotd
        19,    # chargen
        20,    # ftp-data
        21,    # ftp
        22,    # ssh
        23,    # telnet
        25,    # smtp
        37,    # time
        42,    # name
        43,    # nicname
        53,    # domain
        77,    # priv-rjs
        79,    # finger
        87,    # ttylink
        95,    # supdup
        101,   # hostriame
        102,   # iso-tsap
        103,   # gppitnp
        104,   # acr-nema
        109,   # pop2
        110,   # pop3
        111,   # sunrpc
        113,   # auth
        115,   # sftp
        117,   # uucp-path
        119,   # nntp
        123,   # ntp
        135,   # loc-srv / epmap
        139,   # netbios
        143,   # imap2
        179,   # bgp
        389,   # ldap
        427,   # afp (alternate)
        465,   # smtp (alternate)
        512,   # print / exec
        513,   # login
        514,   # shell
        515,   # printer
        526,   # tempo
        530,   # courier
        531,   # chat
        532,   # netnews
        540,   # uucp
        548,   # afp
        556,   # remotefs
        563,   # nntp+ssl
        587,   # smtp (outgoing)
        601,   # syslog-conn
        636,   # ldap+ssl
        993,   # ldap+ssl
        995,   # pop3+ssl
        2049,  # nfs
        3659,  # apple-sasl
        4045,  # lockd
        6000,  # x11
        6665,  # irc (alternate)
        6666,  # irc (alternate)
        6667,  # irc (default)
        6668,  # irc (alternate)
        6669,  # irc (alternate)
        6697,  # irc+tls
    ]


def get_port(host=''):
    host = host or '127.0.0.1'
    port = 0
    while True:
        free_socket = _open_socket(host, 0)
        port = free_socket.getsockname()[1]
        free_socket.close()
        if not is_bad_port(port):
            break
    return port

def http2_compatible():
    # Currently, the HTTP/2.0 server is only working in python 2.7.10+ and OpenSSL 1.0.2+
    import ssl
    ssl_v = ssl.OPENSSL_VERSION_INFO
    return ((sys.version_info[0] == 2 and sys.version_info[1] == 7 and sys.version_info[2] >= 10) and
            (ssl_v[0] == 1 and (ssl_v[1] == 1 or (ssl_v[1] == 0 and ssl_v[2] >= 2))))
