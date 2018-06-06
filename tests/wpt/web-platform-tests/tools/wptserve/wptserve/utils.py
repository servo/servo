import socket

def invert_dict(dict):
    rv = {}
    for key, values in dict.iteritems():
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

def get_port(host):
    port = 0
    while True:
        free_socket = _open_socket(host, 0)
        port = free_socket.getsockname()[1]
        free_socket.close()
        if not is_bad_port(port):
            break
    return port
