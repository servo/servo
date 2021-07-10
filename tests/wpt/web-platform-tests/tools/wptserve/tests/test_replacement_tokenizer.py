from __future__ import unicode_literals

import pytest

from wptserve.pipes import ReplacementTokenizer

@pytest.mark.parametrize(
    "content,expected",
    [
        [b"aaa", [('ident', 'aaa')]],
        [b"bbb()", [('ident', 'bbb'), ('arguments', [])]],
        [b"bcd(uvw, xyz)", [('ident', 'bcd'), ('arguments', ['uvw', 'xyz'])]],
        [b"$ccc:ddd", [('var', '$ccc'), ('ident', 'ddd')]],
        [b"$eee", [('ident', '$eee')]],
        [b"fff[0]", [('ident', 'fff'), ('index', 0)]],
        [b"ggg[hhh]", [('ident', 'ggg'), ('index', 'hhh')]],
        [b"[iii]", [('index', 'iii')]],
        [b"jjj['kkk']", [('ident', 'jjj'), ('index', "'kkk'")]],
        [b"lll[]", [('ident', 'lll'), ('index', "")]],
        [b"111", [('ident', '111')]],
        [b"$111", [('ident', '$111')]],
    ]
)
def test_tokenizer(content, expected):
    tokenizer = ReplacementTokenizer()
    tokens = tokenizer.tokenize(content)
    assert expected == tokens


@pytest.mark.parametrize(
    "content,expected",
    [
        [b"/", []],
        [b"$aaa: BBB", [('var', '$aaa')]],
    ]
)
def test_tokenizer_errors(content, expected):
    tokenizer = ReplacementTokenizer()
    tokens = tokenizer.tokenize(content)
    assert expected == tokens
