"""Benchark parsing WebSocket frames."""

import subprocess
import sys
import timeit

from websockets.extensions.permessage_deflate import PerMessageDeflate
from websockets.frames import Frame, Opcode
from websockets.streams import StreamReader


# 256kB of text, compressible by about 70%.
text = subprocess.check_output(["git", "log", "8dd8e410"], text=True)


def get_frame(size):
    repeat, remainder = divmod(size, 256 * 1024)
    payload = repeat * text + text[:remainder]
    return Frame(Opcode.TEXT, payload.encode(), True)


def parse_frame(data, count, mask, extensions):
    reader = StreamReader()
    for _ in range(count):
        reader.feed_data(data)
        parser = Frame.parse(
            reader.read_exact,
            mask=mask,
            extensions=extensions,
        )
        try:
            next(parser)
        except StopIteration:
            pass
        else:
            assert False, "parser should return frame"
    reader.feed_eof()
    assert reader.at_eof(), "parser should consume all data"


def run_benchmark(size, count, compression=False, number=100):
    if compression:
        extensions = [PerMessageDeflate(True, True, 12, 12, {"memLevel": 5})]
    else:
        extensions = []
    globals = {
        "get_frame": get_frame,
        "parse_frame": parse_frame,
        "extensions": extensions,
    }
    sppf = (
        min(
            timeit.repeat(
                f"parse_frame(data, {count}, mask=True, extensions=extensions)",
                f"data = get_frame({size})"
                f".serialize(mask=True, extensions=extensions)",
                number=number,
                globals=globals,
            )
        )
        / number
        / count
        * 1_000_000
    )
    cppf = (
        min(
            timeit.repeat(
                f"parse_frame(data, {count}, mask=False, extensions=extensions)",
                f"data = get_frame({size})"
                f".serialize(mask=False, extensions=extensions)",
                number=number,
                globals=globals,
            )
        )
        / number
        / count
        * 1_000_000
    )
    print(f"{size}\t{compression}\t{sppf:.2f}\t{cppf:.2f}")


if __name__ == "__main__":
    print("Sizes are in bytes. Times are in µs per frame.", file=sys.stderr)
    print("Run `tabs -16` for clean output. Pipe stdout to TSV for saving.")
    print(file=sys.stderr)

    print("size\tcompression\tserver\tclient")
    run_benchmark(size=8, count=1000, compression=False)
    run_benchmark(size=60, count=1000, compression=False)
    run_benchmark(size=500, count=1000, compression=False)
    run_benchmark(size=4_000, count=1000, compression=False)
    run_benchmark(size=30_000, count=200, compression=False)
    run_benchmark(size=250_000, count=100, compression=False)
    run_benchmark(size=2_000_000, count=20, compression=False)

    run_benchmark(size=8, count=1000, compression=True)
    run_benchmark(size=60, count=1000, compression=True)
    run_benchmark(size=500, count=200, compression=True)
    run_benchmark(size=4_000, count=100, compression=True)
    run_benchmark(size=30_000, count=20, compression=True)
    run_benchmark(size=250_000, count=10, compression=True)
