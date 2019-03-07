import pytest

from ..item import URLManifestItem, TestharnessTest


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


def test_testharness_meta_key_includes_jsshell():
    a = TestharnessTest("/foobar", "/foo", "/foo.bar", "/foo.bar/foo",
                        jsshell=False, script_metadata=[])
    b = TestharnessTest("/foobar", "/foo", "/foo.bar", "/foo.bar/foo",
                        jsshell=True, script_metadata=[])

    assert a.meta_key() != b.meta_key()


@pytest.mark.parametrize("script_metadata", [
    None,
    [],
    [('script', '/resources/WebIDLParser.js'), ('script', '/resources/idlharness.js')],
    [[u'script', u'/resources/WebIDLParser.js'], [u'script', u'/resources/idlharness.js']],
])
def test_testharness_hashable_script_metadata(script_metadata):
    a = TestharnessTest("/",
                        "BackgroundSync/interfaces.https.any.js",
                        "/",
                        "/BackgroundSync/interfaces.https.any.js",
                        script_metadata=script_metadata)

    assert hash(a) is not None
