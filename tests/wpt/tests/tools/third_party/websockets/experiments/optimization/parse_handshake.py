"""Benchark parsing WebSocket handshake requests."""

# The parser for responses is designed similarly and should perform similarly.

import sys
import timeit

from websockets.http11 import Request
from websockets.streams import StreamReader


CHROME_HANDSHAKE = (
    b"GET / HTTP/1.1\r\n"
    b"Host: localhost:5678\r\n"
    b"Connection: Upgrade\r\n"
    b"Pragma: no-cache\r\n"
    b"Cache-Control: no-cache\r\n"
    b"User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
    b"AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36\r\n"
    b"Upgrade: websocket\r\n"
    b"Origin: null\r\n"
    b"Sec-WebSocket-Version: 13\r\n"
    b"Accept-Encoding: gzip, deflate, br\r\n"
    b"Accept-Language: en-GB,en;q=0.9,en-US;q=0.8,fr;q=0.7\r\n"
    b"Sec-WebSocket-Key: ebkySAl+8+e6l5pRKTMkyQ==\r\n"
    b"Sec-WebSocket-Extensions: permessage-deflate; client_max_window_bits\r\n"
    b"\r\n"
)

FIREFOX_HANDSHAKE = (
    b"GET / HTTP/1.1\r\n"
    b"Host: localhost:5678\r\n"
    b"User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) "
    b"Gecko/20100101 Firefox/111.0\r\n"
    b"Accept: */*\r\n"
    b"Accept-Language: en-US,en;q=0.7,fr-FR;q=0.3\r\n"
    b"Accept-Encoding: gzip, deflate, br\r\n"
    b"Sec-WebSocket-Version: 13\r\n"
    b"Origin: null\r\n"
    b"Sec-WebSocket-Extensions: permessage-deflate\r\n"
    b"Sec-WebSocket-Key: 1PuS+hnb+0AXsL7z2hNAhw==\r\n"
    b"Connection: keep-alive, Upgrade\r\n"
    b"Sec-Fetch-Dest: websocket\r\n"
    b"Sec-Fetch-Mode: websocket\r\n"
    b"Sec-Fetch-Site: cross-site\r\n"
    b"Pragma: no-cache\r\n"
    b"Cache-Control: no-cache\r\n"
    b"Upgrade: websocket\r\n"
    b"\r\n"
)

WEBSOCKETS_HANDSHAKE = (
    b"GET / HTTP/1.1\r\n"
    b"Host: localhost:8765\r\n"
    b"Upgrade: websocket\r\n"
    b"Connection: Upgrade\r\n"
    b"Sec-WebSocket-Key: 9c55e0/siQ6tJPCs/QR8ZA==\r\n"
    b"Sec-WebSocket-Version: 13\r\n"
    b"Sec-WebSocket-Extensions: permessage-deflate; client_max_window_bits\r\n"
    b"User-Agent: Python/3.11 websockets/11.0\r\n"
    b"\r\n"
)


def parse_handshake(handshake):
    reader = StreamReader()
    reader.feed_data(handshake)
    parser = Request.parse(reader.read_line)
    try:
        next(parser)
    except StopIteration:
        pass
    else:
        assert False, "parser should return request"
    reader.feed_eof()
    assert reader.at_eof(), "parser should consume all data"


def run_benchmark(name, handshake, number=10000):
    ph = (
        min(
            timeit.repeat(
                "parse_handshake(handshake)",
                number=number,
                globals={"parse_handshake": parse_handshake, "handshake": handshake},
            )
        )
        / number
        * 1_000_000
    )
    print(f"{name}\t{len(handshake)}\t{ph:.1f}")


if __name__ == "__main__":
    print("Sizes are in bytes. Times are in µs per frame.", file=sys.stderr)
    print("Run `tabs -16` for clean output. Pipe stdout to TSV for saving.")
    print(file=sys.stderr)

    print("client\tsize\ttime")
    run_benchmark("Chrome", CHROME_HANDSHAKE)
    run_benchmark("Firefox", FIREFOX_HANDSHAKE)
    run_benchmark("websockets", WEBSOCKETS_HANDSHAKE)
