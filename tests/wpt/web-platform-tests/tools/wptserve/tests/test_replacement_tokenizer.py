import pytest

from wptserve.pipes import ReplacementTokenizer

@pytest.mark.parametrize(
    "content,expected",
    [
        ["aaa", [('ident', 'aaa')]],
        ["bbb()", [('ident', 'bbb()')]],
        ["$ccc:ddd", [('var', '$ccc'), ('ident', 'ddd')]],
        ["$eee", [('ident', '$eee')]],
        ["fff[0]", [('ident', 'fff'), ('index', 0)]],
        ["ggg[hhh]", [('ident', 'ggg'), ('index', u'hhh')]],
        ["[iii]", [('index', u'iii')]],
        ["jjj['kkk']", [('ident', 'jjj'), ('index', u"'kkk'")]],
        ["lll[]", [('ident', 'lll'), ('index', u"")]],
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
