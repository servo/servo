import sys

import atheris


with atheris.instrument_imports():
    from websockets.exceptions import PayloadTooBig, ProtocolError
    from websockets.frames import Frame
    from websockets.streams import StreamReader


def test_one_input(data):
    fdp = atheris.FuzzedDataProvider(data)
    mask = fdp.ConsumeBool()
    max_size_enabled = fdp.ConsumeBool()
    max_size = fdp.ConsumeInt(4)
    payload = fdp.ConsumeBytes(atheris.ALL_REMAINING)

    reader = StreamReader()
    reader.feed_data(payload)
    reader.feed_eof()

    parser = Frame.parse(
        reader.read_exact,
        mask=mask,
        max_size=max_size if max_size_enabled else None,
    )

    try:
        next(parser)
    except StopIteration as exc:
        assert isinstance(exc.value, Frame)
        return  # input accepted
    except (
        EOFError,  # connection is closed without a full WebSocket frame
        UnicodeDecodeError,  # frame contains invalid UTF-8
        PayloadTooBig,  # frame's payload size exceeds ``max_size``
        ProtocolError,  # frame contains incorrect values
    ):
        return  # input rejected with a documented exception

    raise RuntimeError("parsing didn't complete")


def main():
    atheris.Setup(sys.argv, test_one_input)
    atheris.Fuzz()


if __name__ == "__main__":
    main()
