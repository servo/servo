from ..item import SupportFile, URLManifestItem
from ..sourcefile import SourceFile


def test_base_meta_flags():
    s = SourceFile("/", "a.b.c.d", "/", contents="")
    m = SupportFile(s)

    assert m.meta_flags == {"b", "c"}


def test_url_meta_flags():
    s = SourceFile("/", "a.b.c", "/", contents="")
    m = URLManifestItem(s, "/foo.bar/a.b.d.e")

    assert m.meta_flags == {"b", "d"}


def test_url_empty_meta_flags():
    s = SourceFile("/", "a.b.c", "/", contents="")
    m = URLManifestItem(s, "/foo.bar/abcde")

    assert m.meta_flags == set()
