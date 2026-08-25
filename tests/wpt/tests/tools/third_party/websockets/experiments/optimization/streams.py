"""
Benchmark two possible implementations of a stream reader.

The difference lies in the data structure that buffers incoming data:

* ``ByteArrayStreamReader`` uses a ``bytearray``;
* ``BytesDequeStreamReader`` uses a ``deque[bytes]``.

``ByteArrayStreamReader`` is faster for streaming small frames, which is the
standard use case of websockets, likely due to its simple implementation and
to ``bytearray`` being fast at appending data and removing data at the front
(https://hg.python.org/cpython/rev/499a96611baa).

``BytesDequeStreamReader`` is faster for large frames and for bursts, likely
because it copies payloads only once, while ``ByteArrayStreamReader`` copies
them twice.

"""


import collections
import os
import timeit


# Implementations


class ByteArrayStreamReader:
    def __init__(self):
        self.buffer = bytearray()
        self.eof = False

    def readline(self):
        n = 0  # number of bytes to read
        p = 0  # number of bytes without a newline
        while True:
            n = self.buffer.find(b"\n", p) + 1
            if n > 0:
                break
            p = len(self.buffer)
            yield
        r = self.buffer[:n]
        del self.buffer[:n]
        return r

    def readexactly(self, n):
        assert n >= 0
        while len(self.buffer) < n:
            yield
        r = self.buffer[:n]
        del self.buffer[:n]
        return r

    def feed_data(self, data):
        self.buffer += data

    def feed_eof(self):
        self.eof = True

    def at_eof(self):
        return self.eof and not self.buffer


class BytesDequeStreamReader:
    def __init__(self):
        self.buffer = collections.deque()
        self.eof = False

    def readline(self):
        b = []
        while True:
            # Read next chunk
            while True:
                try:
                    c = self.buffer.popleft()
                except IndexError:
                    yield
                else:
                    break
            # Handle chunk
            n = c.find(b"\n") + 1
            if n == len(c):
                # Read exactly enough data
                b.append(c)
                break
            elif n > 0:
                # Read too much data
                b.append(c[:n])
                self.buffer.appendleft(c[n:])
                break
            else:  # n == 0
                # Need to read more data
                b.append(c)
        return b"".join(b)

    def readexactly(self, n):
        if n == 0:
            return b""
        b = []
        while True:
            # Read next chunk
            while True:
                try:
                    c = self.buffer.popleft()
                except IndexError:
                    yield
                else:
                    break
            # Handle chunk
            n -= len(c)
            if n == 0:
                # Read exactly enough data
                b.append(c)
                break
            elif n < 0:
                # Read too much data
                b.append(c[:n])
                self.buffer.appendleft(c[n:])
                break
            else:  # n >= 0
                # Need to read more data
                b.append(c)
        return b"".join(b)

    def feed_data(self, data):
        self.buffer.append(data)

    def feed_eof(self):
        self.eof = True

    def at_eof(self):
        return self.eof and not self.buffer


# Tests


class Protocol:
    def __init__(self, StreamReader):
        self.reader = StreamReader()
        self.events = []
        # Start parser coroutine
        self.parser = self.run_parser()
        next(self.parser)

    def run_parser(self):
        while True:
            frame = yield from self.reader.readexactly(2)
            self.events.append(frame)
            frame = yield from self.reader.readline()
            self.events.append(frame)

    def data_received(self, data):
        self.reader.feed_data(data)
        next(self.parser)  # run parser until more data is needed
        events, self.events = self.events, []
        return events


def run_test(StreamReader):
    proto = Protocol(StreamReader)

    actual = proto.data_received(b"a")
    expected = []
    assert actual == expected, f"{actual} != {expected}"

    actual = proto.data_received(b"b")
    expected = [b"ab"]
    assert actual == expected, f"{actual} != {expected}"

    actual = proto.data_received(b"c")
    expected = []
    assert actual == expected, f"{actual} != {expected}"

    actual = proto.data_received(b"\n")
    expected = [b"c\n"]
    assert actual == expected, f"{actual} != {expected}"

    actual = proto.data_received(b"efghi\njklmn")
    expected = [b"ef", b"ghi\n", b"jk"]
    assert actual == expected, f"{actual} != {expected}"


# Benchmarks


def get_frame_packets(size, packet_size=None):
    if size < 126:
        frame = bytes([138, size])
    elif size < 65536:
        frame = bytes([138, 126]) + bytes(divmod(size, 256))
    else:
        size1, size2 = divmod(size, 65536)
        frame = (
            bytes([138, 127]) + bytes(divmod(size1, 256)) + bytes(divmod(size2, 256))
        )
    frame += os.urandom(size)
    if packet_size is None:
        return [frame]
    else:
        packets = []
        while frame:
            packets.append(frame[:packet_size])
            frame = frame[packet_size:]
        return packets


def benchmark_stream(StreamReader, packets, size, count):
    reader = StreamReader()
    for _ in range(count):
        for packet in packets:
            reader.feed_data(packet)
        yield from reader.readexactly(2)
        if size >= 65536:
            yield from reader.readexactly(4)
        elif size >= 126:
            yield from reader.readexactly(2)
        yield from reader.readexactly(size)
    reader.feed_eof()
    assert reader.at_eof()


def benchmark_burst(StreamReader, packets, size, count):
    reader = StreamReader()
    for _ in range(count):
        for packet in packets:
            reader.feed_data(packet)
    reader.feed_eof()
    for _ in range(count):
        yield from reader.readexactly(2)
        if size >= 65536:
            yield from reader.readexactly(4)
        elif size >= 126:
            yield from reader.readexactly(2)
        yield from reader.readexactly(size)
    assert reader.at_eof()


def run_benchmark(size, count, packet_size=None, number=1000):
    stmt = f"list(benchmark(StreamReader, packets, {size}, {count}))"
    setup = f"packets = get_frame_packets({size}, {packet_size})"
    context = globals()

    context["StreamReader"] = context["ByteArrayStreamReader"]
    context["benchmark"] = context["benchmark_stream"]
    bas = min(timeit.repeat(stmt, setup, number=number, globals=context))
    context["benchmark"] = context["benchmark_burst"]
    bab = min(timeit.repeat(stmt, setup, number=number, globals=context))

    context["StreamReader"] = context["BytesDequeStreamReader"]
    context["benchmark"] = context["benchmark_stream"]
    bds = min(timeit.repeat(stmt, setup, number=number, globals=context))
    context["benchmark"] = context["benchmark_burst"]
    bdb = min(timeit.repeat(stmt, setup, number=number, globals=context))

    print(
        f"Frame size = {size} bytes, "
        f"frame count = {count}, "
        f"packet size = {packet_size}"
    )
    print(f"* ByteArrayStreamReader  (stream): {bas / number * 1_000_000:.1f}µs")
    print(
        f"* BytesDequeStreamReader (stream): "
        f"{bds / number * 1_000_000:.1f}µs ({(bds / bas - 1) * 100:+.1f}%)"
    )
    print(f"* ByteArrayStreamReader  (burst):  {bab / number * 1_000_000:.1f}µs")
    print(
        f"* BytesDequeStreamReader (burst):  "
        f"{bdb / number * 1_000_000:.1f}µs ({(bdb / bab - 1) * 100:+.1f}%)"
    )
    print()


if __name__ == "__main__":
    run_test(ByteArrayStreamReader)
    run_test(BytesDequeStreamReader)

    run_benchmark(size=8, count=1000)
    run_benchmark(size=60, count=1000)
    run_benchmark(size=500, count=500)
    run_benchmark(size=4_000, count=200)
    run_benchmark(size=30_000, count=100)
    run_benchmark(size=250_000, count=50)
    run_benchmark(size=2_000_000, count=20)

    run_benchmark(size=4_000, count=200, packet_size=1024)
    run_benchmark(size=30_000, count=100, packet_size=1024)
    run_benchmark(size=250_000, count=50, packet_size=1024)
    run_benchmark(size=2_000_000, count=20, packet_size=1024)

    run_benchmark(size=30_000, count=100, packet_size=4096)
    run_benchmark(size=250_000, count=50, packet_size=4096)
    run_benchmark(size=2_000_000, count=20, packet_size=4096)

    run_benchmark(size=30_000, count=100, packet_size=16384)
    run_benchmark(size=250_000, count=50, packet_size=16384)
    run_benchmark(size=2_000_000, count=20, packet_size=16384)

    run_benchmark(size=250_000, count=50, packet_size=65536)
    run_benchmark(size=2_000_000, count=20, packet_size=65536)
