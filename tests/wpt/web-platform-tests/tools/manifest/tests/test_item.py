import pytest

from ..item import URLManifestItem


@pytest.mark.parametrize("path", [
    "a.https.c",
    "a.b.https.c",
    "a.https.b.c",
    "a.b.https.c.d",
    "a.serviceworker.c",
    "a.b.serviceworker.c",
    "a.serviceworker.b.c",
    "a.b.serviceworker.c.d",
])
def test_url_https(path):
    m = URLManifestItem("/foobar", "/" + path, "/", "/foo.bar/" + path)

    assert m.https is True


@pytest.mark.parametrize("path", [
    "https",
    "a.https",
    "a.b.https",
    "https.a",
    "https.a.b",
    "a.bhttps.c",
    "a.httpsb.c",
    "serviceworker",
    "a.serviceworker",
    "a.b.serviceworker",
    "serviceworker.a",
    "serviceworker.a.b",
    "a.bserviceworker.c",
    "a.serviceworkerb.c",
])
def test_url_not_https(path):
    m = URLManifestItem("/foobar", "/" + path, "/", "/foo.bar/" + path)

    assert m.https is False
