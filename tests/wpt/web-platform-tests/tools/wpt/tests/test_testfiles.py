from mock import patch

from tools.wpt import testfiles


def test_getrevish_kwarg():
    assert testfiles.get_revish(revish=u"abcdef") == b"abcdef"
    assert testfiles.get_revish(revish=b"abcdef") == b"abcdef"


def test_getrevish_implicit():
    with patch("tools.wpt.testfiles.branch_point", return_value=u"base"):
        assert testfiles.get_revish() == b"base..HEAD"
