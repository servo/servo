import sys

import pytest

from wptserve.pipes import ReplacementTokenizer

@pytest.mark.parametrize(
    "content,expected",
    [
        ["aaa", [('ident', 'aaa')]],
        ["bbb()", [('ident', 'bbb'), ('arguments', [])]],
        ["$ccc:ddd", [('var', '$ccc'), ('ident', 'ddd')]],
        ["$eee", [('ident', '$eee')]],
        ["fff[0]", [('ident', 'fff'), ('index', 0)]],
        pytest.param(
            "ggg[hhh]", [('ident', 'ggg'), ('index', u'hhh')],
            marks=pytest.mark.xfail(sys.version_info >= (3,),
                                    reason="wptserve only works on Py2")
        ),
        pytest.param(
            "[iii]", [('index', u'iii')],
            marks=pytest.mark.xfail(sys.version_info >= (3,),
                                    reason="wptserve only works on Py2")
        ),
        pytest.param(
            "jjj['kkk']", [('ident', 'jjj'), ('index', u"'kkk'")],
            marks=pytest.mark.xfail(sys.version_info >= (3,),
                                    reason="wptserve only works on Py2")
        ),
        pytest.param(
            "lll[]", [('ident', 'lll'), ('index', u"")],
            marks=pytest.mark.xfail(sys.version_info >= (3,),
                                    reason="wptserve only works on Py2")
        ),
        ["111", [('ident', u'111')]],
        ["$111", [('ident', u'$111')]],
    ]
)
def test_tokenizer(content, expected):
    tokenizer = ReplacementTokenizer()
    tokens = tokenizer.tokenize(content)
    assert expected == tokens


@pytest.mark.parametrize(
    "content,expected",
    [
        ["/", []],
        ["$aaa: BBB", [('var', '$aaa')]],
    ]
)
def test_tokenizer_errors(content, expected):
    tokenizer = ReplacementTokenizer()
    tokens = tokenizer.tokenize(content)
    assert expected == tokens
