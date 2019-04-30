import json

import pytest

from ..manifest import Manifest
from ..item import URLManifestItem, RefTest


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
    m = URLManifestItem("/foo", "bar/" + path, "/", "bar/" + path)

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
    m = URLManifestItem("/foo", "bar/" + path, "/", "bar/" + path)

    assert m.https is False


@pytest.mark.parametrize("fuzzy", [
    {('/foo/test.html', u'/foo/ref.html', '=='): [[1, 1], [200, 200]]},
    {('/foo/test.html', u'/foo/ref.html', '=='): [[0, 1], [100, 200]]},
    {None: [[0, 1], [100, 200]]},
    {None: [[1, 1], [200, 200]]},
])
def test_reftest_fuzzy(fuzzy):
    t = RefTest('/',
                'foo/test.html',
                '/',
                'foo/test.html',
                [('/foo/ref.html', '==')],
                fuzzy=fuzzy)
    assert fuzzy == t.fuzzy

    json_obj = t.to_json()

    m = Manifest("/", "/")
    t2 = RefTest.from_json(m, t.path, json_obj)
    assert fuzzy == t2.fuzzy

    # test the roundtrip case, given tuples become lists
    roundtrip = json.loads(json.dumps(json_obj))
    t3 = RefTest.from_json(m, t.path, roundtrip)
    assert fuzzy == t3.fuzzy


@pytest.mark.parametrize("fuzzy", [
    {('/foo/test.html', u'/foo/ref-2.html', '=='): [[0, 1], [100, 200]]},
    {None: [[1, 1], [200, 200]], ('/foo/test.html', u'/foo/ref-2.html', '=='): [[0, 1], [100, 200]]},
])
def test_reftest_fuzzy_multi(fuzzy):
    t = RefTest('/',
                'foo/test.html',
                '/',
                'foo/test.html',
                [('/foo/ref-1.html', '=='), ('/foo/ref-2.html', '==')],
                fuzzy=fuzzy)
    assert fuzzy == t.fuzzy

    json_obj = t.to_json()

    m = Manifest("/", "/")
    t2 = RefTest.from_json(m, t.path, json_obj)
    assert fuzzy == t2.fuzzy

    # test the roundtrip case, given tuples become lists
    roundtrip = json.loads(json.dumps(json_obj))
    t3 = RefTest.from_json(m, t.path, roundtrip)
    assert fuzzy == t3.fuzzy
