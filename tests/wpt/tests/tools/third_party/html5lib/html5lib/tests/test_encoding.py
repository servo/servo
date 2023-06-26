from __future__ import absolute_import, division, unicode_literals

import os

import pytest

from .support import get_data_files, test_dir, errorMessage, TestData as _TestData
from html5lib import HTMLParser, _inputstream


def test_basic_prescan_length():
    data = "<title>Caf\u00E9</title><!--a--><meta charset='utf-8'>".encode('utf-8')
    pad = 1024 - len(data) + 1
    data = data.replace(b"-a-", b"-" + (b"a" * pad) + b"-")
    assert len(data) == 1024  # Sanity
    stream = _inputstream.HTMLBinaryInputStream(data, useChardet=False)
    assert 'utf-8' == stream.charEncoding[0].name


def test_parser_reparse():
    data = "<title>Caf\u00E9</title><!--a--><meta charset='utf-8'>".encode('utf-8')
    pad = 10240 - len(data) + 1
    data = data.replace(b"-a-", b"-" + (b"a" * pad) + b"-")
    assert len(data) == 10240  # Sanity
    stream = _inputstream.HTMLBinaryInputStream(data, useChardet=False)
    assert 'windows-1252' == stream.charEncoding[0].name
    p = HTMLParser(namespaceHTMLElements=False)
    doc = p.parse(data, useChardet=False)
    assert 'utf-8' == p.documentEncoding
    assert doc.find(".//title").text == "Caf\u00E9"


@pytest.mark.parametrize("expected,data,kwargs", [
    ("utf-16le", b"\xFF\xFE", {"override_encoding": "iso-8859-2"}),
    ("utf-16be", b"\xFE\xFF", {"override_encoding": "iso-8859-2"}),
    ("utf-8", b"\xEF\xBB\xBF", {"override_encoding": "iso-8859-2"}),
    ("iso-8859-2", b"", {"override_encoding": "iso-8859-2", "transport_encoding": "iso-8859-3"}),
    ("iso-8859-2", b"<meta charset=iso-8859-3>", {"transport_encoding": "iso-8859-2"}),
    ("iso-8859-2", b"<meta charset=iso-8859-2>", {"same_origin_parent_encoding": "iso-8859-3"}),
    ("iso-8859-2", b"", {"same_origin_parent_encoding": "iso-8859-2", "likely_encoding": "iso-8859-3"}),
    ("iso-8859-2", b"", {"same_origin_parent_encoding": "utf-16", "likely_encoding": "iso-8859-2"}),
    ("iso-8859-2", b"", {"same_origin_parent_encoding": "utf-16be", "likely_encoding": "iso-8859-2"}),
    ("iso-8859-2", b"", {"same_origin_parent_encoding": "utf-16le", "likely_encoding": "iso-8859-2"}),
    ("iso-8859-2", b"", {"likely_encoding": "iso-8859-2", "default_encoding": "iso-8859-3"}),
    ("iso-8859-2", b"", {"default_encoding": "iso-8859-2"}),
    ("windows-1252", b"", {"default_encoding": "totally-bogus-string"}),
    ("windows-1252", b"", {}),
])
def test_parser_args(expected, data, kwargs):
    stream = _inputstream.HTMLBinaryInputStream(data, useChardet=False, **kwargs)
    assert expected == stream.charEncoding[0].name
    p = HTMLParser()
    p.parse(data, useChardet=False, **kwargs)
    assert expected == p.documentEncoding


@pytest.mark.parametrize("kwargs", [
    {"override_encoding": "iso-8859-2"},
    {"override_encoding": None},
    {"transport_encoding": "iso-8859-2"},
    {"transport_encoding": None},
    {"same_origin_parent_encoding": "iso-8859-2"},
    {"same_origin_parent_encoding": None},
    {"likely_encoding": "iso-8859-2"},
    {"likely_encoding": None},
    {"default_encoding": "iso-8859-2"},
    {"default_encoding": None},
    {"foo_encoding": "iso-8859-2"},
    {"foo_encoding": None},
])
def test_parser_args_raises(kwargs):
    with pytest.raises(TypeError) as exc_info:
        p = HTMLParser()
        p.parse("", useChardet=False, **kwargs)
    assert exc_info.value.args[0].startswith("Cannot set an encoding with a unicode input")


def param_encoding():
    for filename in get_data_files("encoding"):
        tests = _TestData(filename, b"data", encoding=None)
        for test in tests:
            yield test[b'data'], test[b'encoding']


@pytest.mark.parametrize("data, encoding", param_encoding())
def test_parser_encoding(data, encoding):
    p = HTMLParser()
    assert p.documentEncoding is None
    p.parse(data, useChardet=False)
    encoding = encoding.lower().decode("ascii")

    assert encoding == p.documentEncoding, errorMessage(data, encoding, p.documentEncoding)


@pytest.mark.parametrize("data, encoding", param_encoding())
def test_prescan_encoding(data, encoding):
    stream = _inputstream.HTMLBinaryInputStream(data, useChardet=False)
    encoding = encoding.lower().decode("ascii")

    # Very crude way to ignore irrelevant tests
    if len(data) > stream.numBytesMeta:
        return

    assert encoding == stream.charEncoding[0].name, errorMessage(data, encoding, stream.charEncoding[0].name)


# pylint:disable=wrong-import-position
try:
    import chardet  # noqa
except ImportError:
    print("chardet not found, skipping chardet tests")
else:
    def test_chardet():
        with open(os.path.join(test_dir, "encoding", "chardet", "test_big5.txt"), "rb") as fp:
            encoding = _inputstream.HTMLInputStream(fp.read()).charEncoding
            assert encoding[0].name == "big5"
# pylint:enable=wrong-import-position
