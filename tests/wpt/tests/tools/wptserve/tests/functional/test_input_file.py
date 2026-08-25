from io import BytesIO

import pytest

from wptserve.request import InputFile

def files_with_buffer(max_buffer_size=None):
    bstr = b"This is a test document\nWith new lines\nSeveral in fact..."

    with BytesIO(bstr) as rfile, BytesIO(bstr) as test_file:
        if max_buffer_size is not None:
            old_max_buf = InputFile.max_buffer_size
            InputFile.max_buffer_size = max_buffer_size

        try:
            with InputFile(rfile, len(bstr)) as input_file:
                yield (input_file, test_file)
        finally:
            if max_buffer_size is not None:
                InputFile.max_buffer_size = old_max_buf


@pytest.fixture
def files():
    yield from files_with_buffer()


@pytest.fixture
def files_small_buffer():
    yield from files_with_buffer(10)


def test_seek(files):
    input_file, test_file = files

    input_file.seek(2)
    test_file.seek(2)
    assert input_file.read(1) == test_file.read(1)

    input_file.seek(4)
    test_file.seek(4)
    assert input_file.read(1) == test_file.read(1)


def test_seek_backwards(files):
    input_file, test_file = files

    input_file.seek(2)
    test_file.seek(2)
    assert input_file.tell() == test_file.tell()
    assert input_file.read(1) == test_file.read(1)
    assert input_file.tell() == test_file.tell()

    input_file.seek(0)
    test_file.seek(0)
    assert input_file.read(1) == test_file.read(1)


def test_seek_negative_offset(files):
    input_file, test_file = files

    with pytest.raises(ValueError):
        input_file.seek(-1)


def test_seek_file_bigger_than_buffer(files_small_buffer):
    input_file, test_file = files_small_buffer

    input_file.seek(2)
    test_file.seek(2)
    assert input_file.read(1) == test_file.read(1)

    input_file.seek(4)
    test_file.seek(4)
    assert input_file.read(1) == test_file.read(1)


def test_read(files):
    input_file, test_file = files

    assert input_file.read() == test_file.read()


def test_read_file_bigger_than_buffer(files_small_buffer):
    input_file, test_file = files_small_buffer

    assert input_file.read() == test_file.read()


def test_readline(files):
    input_file, test_file = files

    assert input_file.readline() == test_file.readline()
    assert input_file.readline() == test_file.readline()

    input_file.seek(0)
    test_file.seek(0)
    assert input_file.readline() == test_file.readline()


def test_readline_max_byte(files):
    input_file, test_file = files

    line = test_file.readline()
    assert input_file.readline(max_bytes=len(line)//2) == line[:len(line)//2]
    assert input_file.readline(max_bytes=len(line)) == line[len(line)//2:]


def test_readline_max_byte_longer_than_file(files):
    input_file, test_file = files

    assert input_file.readline(max_bytes=1000) == test_file.readline()
    assert input_file.readline(max_bytes=1000) == test_file.readline()


def test_readline_file_bigger_than_buffer(files_small_buffer):
    input_file, test_file = files_small_buffer

    assert input_file.readline() == test_file.readline()
    assert input_file.readline() == test_file.readline()


def test_readlines(files):
    input_file, test_file = files

    assert input_file.readlines() == test_file.readlines()


def test_readlines_file_bigger_than_buffer(files_small_buffer):
    input_file, test_file = files_small_buffer

    assert input_file.readlines() == test_file.readlines()


def test_iter(files):
    input_file, test_file = files

    for a, b in zip(input_file, test_file):
        assert a == b


def test_iter_file_bigger_than_buffer(files_small_buffer):
    input_file, test_file = files_small_buffer

    for a, b in zip(input_file, test_file):
        assert a == b
