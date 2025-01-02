import sys

import atheris


with atheris.instrument_imports():
    from websockets.exceptions import SecurityError
    from websockets.http11 import Response
    from websockets.streams import StreamReader


def test_one_input(data):
    reader = StreamReader()
    reader.feed_data(data)
    reader.feed_eof()

    parser = Response.parse(
        reader.read_line,
        reader.read_exact,
        reader.read_to_eof,
    )
    try:
        next(parser)
    except StopIteration as exc:
        assert isinstance(exc.value, Response)
        return  # input accepted
    except (
        EOFError,  # connection is closed without a full HTTP response
        SecurityError,  # response exceeds a security limit
        LookupError,  # response isn't well formatted
        ValueError,  # response isn't well formatted
    ):
        return  # input rejected with a documented exception

    raise RuntimeError("parsing didn't complete")


def main():
    atheris.Setup(sys.argv, test_one_input)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
