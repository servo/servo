from wptserve.request import InputFile
from io import BytesIO
import pytest

wptserve = pytest.importorskip("wptserve")
bstr = b'This is a test document\nWith new lines\nSeveral in fact...'
rfile = ''
test_file = ''  # This will be used to test the InputFile functions against
input_file = InputFile(None, 0)


def setup_function(function):
    global rfile, input_file, test_file
    rfile = BytesIO(bstr)
    test_file = BytesIO(bstr)
    input_file = InputFile(rfile, len(bstr))


def teardown_function(function):
    rfile.close()
    test_file.close()


def test_seek():
    input_file.seek(2)
    test_file.seek(2)
    assert input_file.read(1) == test_file.read(1)

    input_file.seek(4)
    test_file.seek(4)
    assert input_file.read(1) == test_file.read(1)


def test_seek_backwards():
    input_file.seek(2)
    test_file.seek(2)
    assert input_file.tell() == test_file.tell()
    assert input_file.read(1) == test_file.read(1)
    assert input_file.tell() == test_file.tell()

    input_file.seek(0)
    test_file.seek(0)
    assert input_file.read(1) == test_file.read(1)


def test_seek_negative_offset():
    with pytest.raises(ValueError):
        input_file.seek(-1)


def test_seek_file_bigger_than_buffer():
    old_max_buf = InputFile.max_buffer_size
    InputFile.max_buffer_size = 10

    try:
        input_file = InputFile(rfile, len(bstr))

        input_file.seek(2)
        test_file.seek(2)
        assert input_file.read(1) == test_file.read(1)

        input_file.seek(4)
        test_file.seek(4)
        assert input_file.read(1) == test_file.read(1)
    finally:
        InputFile.max_buffer_size = old_max_buf


def test_read():
    assert input_file.read() == test_file.read()


def test_read_file_bigger_than_buffer():
    old_max_buf = InputFile.max_buffer_size
    InputFile.max_buffer_size = 10

    try:
        input_file = InputFile(rfile, len(bstr))
        assert input_file.read() == test_file.read()
    finally:
        InputFile.max_buffer_size = old_max_buf


def test_readline():
    assert input_file.readline() == test_file.readline()
    assert input_file.readline() == test_file.readline()

    input_file.seek(0)
    test_file.seek(0)
    assert input_file.readline() == test_file.readline()


def test_readline_max_byte():
    line = test_file.readline()
    assert input_file.readline(max_bytes=len(line)//2) == line[:len(line)//2]
    assert input_file.readline(max_bytes=len(line)) == line[len(line)//2:]


def test_readline_max_byte_longer_than_file():
    assert input_file.readline(max_bytes=1000) == test_file.readline()
    assert input_file.readline(max_bytes=1000) == test_file.readline()


def test_readline_file_bigger_than_buffer():
    old_max_buf = InputFile.max_buffer_size
    InputFile.max_buffer_size = 10

    try:
        input_file = InputFile(rfile, len(bstr))

        assert input_file.readline() == test_file.readline()
        assert input_file.readline() == test_file.readline()
    finally:
        InputFile.max_buffer_size = old_max_buf


def test_readlines():
    assert input_file.readlines() == test_file.readlines()


def test_readlines_file_bigger_than_buffer():
    old_max_buf = InputFile.max_buffer_size
    InputFile.max_buffer_size = 10

    try:
        input_file = InputFile(rfile, len(bstr))

        assert input_file.readlines() == test_file.readlines()
    finally:
        InputFile.max_buffer_size = old_max_buf


def test_iter():
    for a, b in zip(input_file, test_file):
        assert a == b


def test_iter_file_bigger_than_buffer():
    old_max_buf = InputFile.max_buffer_size
    InputFile.max_buffer_size = 10

    try:
        input_file = InputFile(rfile, len(bstr))

        for a, b in zip(input_file, test_file):
            assert a == b
    finally:
        InputFile.max_buffer_size = old_max_buf
