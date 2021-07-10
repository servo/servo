#!/usr/bin/env python3

import argparse
import logging
import os
import sys

_dir = os.path.dirname(__file__)


def get_parser():
    parser = argparse.ArgumentParser(description='QUIC server')
    parser.add_argument(
        '-c',
        '--certificate',
        type=str,
        default=os.path.join(_dir, 'certs', 'cert.pem'),
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
        default=os.path.join(_dir, 'certs', 'cert.key'),
        help='load the TLS private key from the specified file',
    )
    parser.add_argument(
        '--handlers-path',
        type=str,
        default=os.path.join(
            _dir, '..', '..', 'webtransport', 'quic', 'handlers'),
        help='the directory path of QuicTransport event handlers',
    )
    parser.add_argument(
        '-v',
        '--verbose',
        action='store_true',
        help='increase logging verbosity'
    )
    return parser


def run(venv, **kwargs):
    assert sys.version_info.major == 3, 'QUIC server only runs in Python 3'
    logging.basicConfig(
        format='%(asctime)s %(levelname)s %(name)s %(message)s',
        level=logging.DEBUG if kwargs.get('verbose') else logging.INFO,
    )

    # Delay import after version check to make the error easier to understand.
    from .quic_transport_server import start
    start(kwargs)


def main():
    # This is only used when executing the script directly. Users are
    # responsible for managing venv themselves. `wpt serve-quic-transport` does
    # NOT use this code path.
    kwargs = vars(get_parser().parse_args())
    return run(None, **kwargs)


if __name__ == '__main__':
    main()
