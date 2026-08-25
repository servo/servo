# mypy: allow-untyped-defs

import inspect
import json

import pytest

from ..manifest import Manifest
# Prevent pytest from treating TestharnessTest as a test class
from ..item import TestharnessTest as HarnessTest
from ..item import RefTest, item_types


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
    m = HarnessTest("/foo", "bar/" + path, "/", "bar/" + path)

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
    m = HarnessTest("/foo", "bar/" + path, "/", "bar/" + path)

    assert m.https is False


@pytest.mark.parametrize("path", [
    "a.www.c",
    "a.b.www.c",
    "a.www.b.c",
    "a.b.www.c.d",
    "a.https.www.c",
    "a.b.https.www.c",
    "a.https.www.b.c",
    "a.b.https.www.c.d",
])
def test_url_subdomain(path):
    m = HarnessTest("/foo", "bar/" + path, "/", "bar/" + path)

    assert m.subdomain is True


@pytest.mark.parametrize("path", [
    "www",
    "a.www",
    "a.b.www",
    "www.a",
    "www.a.b",
    "a.bwwww.c",
    "a.wwwwb.c",
])
def test_url_not_subdomain(path):
    m = HarnessTest("/foo", "bar/" + path, "/", "bar/" + path)

    assert m.subdomain is False


@pytest.mark.parametrize("fuzzy", [
    {('/foo/test.html', '/foo/ref.html', '=='): [[1, 1], [200, 200]]},
    {('/foo/test.html', '/foo/ref.html', '=='): [[0, 1], [100, 200]]},
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
    {('/foo/test.html', '/foo/ref-2.html', '=='): [[0, 1], [100, 200]]},
    {None: [[1, 1], [200, 200]], ('/foo/test.html', '/foo/ref-2.html', '=='): [[0, 1], [100, 200]]},
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


def test_item_types():
    for key, value in item_types.items():
        assert isinstance(key, str)
        assert not inspect.isabstract(value)


def test_wpt_flags():
    m1 = HarnessTest("/foo", "bar", "/", "bar" + "?wpt_flags=www")
    assert m1.subdomain is True
    assert m1.https is False
    assert m1.h2 is False

    m2 = HarnessTest("/foo", "bar", "/", "bar" + "?wpt_flags=https")
    assert m2.subdomain is False
    assert m2.https is True
    assert m2.h2 is False

    m3 = HarnessTest("/foo", "bar", "/", "bar" + "?wpt_flags=h2")
    assert m3.subdomain is False
    assert m3.https is False
    assert m3.h2 is True

    m4 = HarnessTest("/foo", "bar", "/", "bar" + "?wpt_flags=https&wpt_flags=www")
    assert m4.subdomain is True
    assert m4.https is True
    assert m4.h2 is False
