from lint.lint import filter_whitelist_errors

def test_lint():
    filtered = filter_whitelist_errors({}, '', [])
    assert filtered == []
