import pytest

from ..utils import is_blacklisted


@pytest.mark.parametrize("url", [
    "/foo",
    "/tools/foo",
    "/common/foo",
    "/conformance-checkers/foo",
    "/_certs/foo",
    "/resources/foo",
    "/support/foo",
    "/foo/resources/bar",
    "/foo/support/bar"
])
def test_is_blacklisted(url):
    assert is_blacklisted(url) is True


@pytest.mark.parametrize("url", [
    "/foo/tools/bar",
    "/foo/common/bar",
    "/foo/conformance-checkers/bar",
    "/foo/_certs/bar"
])
def test_not_is_blacklisted(url):
    assert is_blacklisted(url) is False
