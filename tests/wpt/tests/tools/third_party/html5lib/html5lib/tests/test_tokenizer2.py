from __future__ import absolute_import, division, unicode_literals

import io

from six import unichr, text_type

from html5lib._tokenizer import HTMLTokenizer
from html5lib.constants import tokenTypes


def ignore_parse_errors(toks):
    for tok in toks:
        if tok['type'] != tokenTypes['ParseError']:
            yield tok


def test_maintain_attribute_order():
    # generate loads to maximize the chance a hash-based mutation will occur
    attrs = [(unichr(x), text_type(i)) for i, x in enumerate(range(ord('a'), ord('z')))]
    stream = io.StringIO("<span " + " ".join("%s='%s'" % (x, i) for x, i in attrs) + ">")

    toks = HTMLTokenizer(stream)
    out = list(ignore_parse_errors(toks))

    assert len(out) == 1
    assert out[0]['type'] == tokenTypes['StartTag']

    attrs_tok = out[0]['data']
    assert len(attrs_tok) == len(attrs)

    for (in_name, in_value), (out_name, out_value) in zip(attrs, attrs_tok.items()):
        assert in_name == out_name
        assert in_value == out_value


def test_duplicate_attribute():
    stream = io.StringIO("<span a=1 a=2 a=3>")

    toks = HTMLTokenizer(stream)
    out = list(ignore_parse_errors(toks))

    assert len(out) == 1
    assert out[0]['type'] == tokenTypes['StartTag']

    attrs_tok = out[0]['data']
    assert len(attrs_tok) == 1
    assert list(attrs_tok.items()) == [('a', '1')]


def test_maintain_duplicate_attribute_order():
    # generate loads to maximize the chance a hash-based mutation will occur
    attrs = [(unichr(x), text_type(i)) for i, x in enumerate(range(ord('a'), ord('z')))]
    stream = io.StringIO("<span " + " ".join("%s='%s'" % (x, i) for x, i in attrs) + " a=100>")

    toks = HTMLTokenizer(stream)
    out = list(ignore_parse_errors(toks))

    assert len(out) == 1
    assert out[0]['type'] == tokenTypes['StartTag']

    attrs_tok = out[0]['data']
    assert len(attrs_tok) == len(attrs)

    for (in_name, in_value), (out_name, out_value) in zip(attrs, attrs_tok.items()):
        assert in_name == out_name
        assert in_value == out_value
