#!/usr/bin/env python3

import argparse
import sys


def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("--verbose", action="store_true", default=False,
                        help="turn on verbose logging")
    return parser


def run(venv, **kwargs):
    # TODO(Hexcles): Replace this with actual implementation.
    print(sys.version)
    assert sys.version_info.major == 3
    import aioquic
    print('aioquic: ' + aioquic.__version__)


def main():
    kwargs = vars(get_parser().parse_args())
    return run(None, **kwargs)


if __name__ == '__main__':
    main()
