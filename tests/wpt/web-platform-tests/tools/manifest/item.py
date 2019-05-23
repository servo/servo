from copy import copy
from six import iteritems
from six.moves.urllib.parse import urljoin, urlparse
from abc import ABCMeta, abstractproperty

from .utils import to_os_path

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Optional

item_types = {}


class ManifestItemMeta(ABCMeta):
    """Custom metaclass that registers all the subclasses in the
    item_types dictionary according to the value of their item_type
    attribute, and otherwise behaves like an ABCMeta."""

    def __new__(cls, name, bases, attrs, **kwargs):
        rv = ABCMeta.__new__(cls, name, bases, attrs, **kwargs)
        if rv.item_type:
            item_types[rv.item_type] = rv

        return rv


class ManifestItem(object):
    __metaclass__ = ManifestItemMeta

    __slots__ = ("_tests_root", "path")

    item_type = None  # type: Optional[str]

    def __init__(self, tests_root=None, path=None):
        self._tests_root = tests_root
        self.path = path

    @abstractproperty
    def id(self):
        """The test's id (usually its url)"""
        pass

    def key(self):
        """A unique identifier for the test"""
        return (self.item_type, self.id)

    def __eq__(self, other):
        if not hasattr(other, "key"):
            return False
        return self.key() == other.key()

    def __hash__(self):
        return hash(self.key())

    def __repr__(self):
        return "<%s.%s id=%s, path=%s>" % (self.__module__, self.__class__.__name__, self.id, self.path)

    def to_json(self):
        return [{}]

    @classmethod
    def from_json(cls, manifest, path, obj):
        path = to_os_path(path)
        return cls(manifest.tests_root, path)


class URLManifestItem(ManifestItem):
    __slots__ = ("url_base", "_url", "_extras")

    def __init__(self, tests_root, path, url_base, url, **extras):
        super(URLManifestItem, self).__init__(tests_root, path)
        assert url_base[0] == "/"
        self.url_base = url_base
        assert url[0] != "/"
        self._url = url
        self._extras = extras

    @property
    def id(self):
        return self.url

    @property
    def url(self):
        # we can outperform urljoin, because we know we just have path relative URLs
        if self.url_base == "/":
            return "/" + self._url
        return urljoin(self.url_base, self._url)

    @property
    def https(self):
        flags = set(urlparse(self.url).path.rsplit("/", 1)[1].split(".")[1:-1])
        return ("https" in flags or "serviceworker" in flags)

    def to_json(self):
        rv = [self._url, {}]
        return rv

    @classmethod
    def from_json(cls, manifest, path, obj):
        path = to_os_path(path)
        url, extras = obj
        return cls(manifest.tests_root,
                   path,
                   manifest.url_base,
                   url,
                   **extras)


class TestharnessTest(URLManifestItem):
    __slots__ = ()

    item_type = "testharness"

    @property
    def timeout(self):
        return self._extras.get("timeout")

    @property
    def testdriver(self):
        return self._extras.get("testdriver")

    @property
    def jsshell(self):
        return self._extras.get("jsshell")

    @property
    def script_metadata(self):
        return self._extras.get("script_metadata")

    def to_json(self):
        rv = super(TestharnessTest, self).to_json()
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        if self.testdriver:
            rv[-1]["testdriver"] = self.testdriver
        if self.jsshell:
            rv[-1]["jsshell"] = True
        if self.script_metadata:
            rv[-1]["script_metadata"] = self.script_metadata
        return rv


class RefTestBase(URLManifestItem):
    __slots__ = ("references",)

    item_type = "reftest_base"

    def __init__(self, tests_root, path, url_base, url, references=None, **extras):
        super(RefTestBase, self).__init__(tests_root, path, url_base, url, **extras)
        if references is None:
            self.references = []
        else:
            self.references = references

    @property
    def timeout(self):
        return self._extras.get("timeout")

    @property
    def viewport_size(self):
        return self._extras.get("viewport_size")

    @property
    def dpi(self):
        return self._extras.get("dpi")

    @property
    def fuzzy(self):
        fuzzy = self._extras.get("fuzzy", {})
        if not isinstance(fuzzy, list):
            return fuzzy

        rv = {}
        for k, v in fuzzy:
            if k is not None:
                k = tuple(k)
            rv[k] = v
        return rv

    def to_json(self):
        rv = [self._url, self.references, {}]
        extras = rv[-1]
        if self.timeout is not None:
            extras["timeout"] = self.timeout
        if self.viewport_size is not None:
            extras["viewport_size"] = self.viewport_size
        if self.dpi is not None:
            extras["dpi"] = self.dpi
        if self.fuzzy:
            extras["fuzzy"] = list(iteritems(self.fuzzy))
        return rv

    @classmethod
    def from_json(cls, manifest, path, obj):
        path = to_os_path(path)
        url, references, extras = obj
        return cls(manifest.tests_root,
                   path,
                   manifest.url_base,
                   url,
                   references,
                   **extras)

    def to_RefTest(self):
        if type(self) == RefTest:
            return self
        rv = copy(self)
        rv.__class__ = RefTest
        return rv

    def to_RefTestNode(self):
        if type(self) == RefTestNode:
            return self
        rv = copy(self)
        rv.__class__ = RefTestNode
        return rv


class RefTestNode(RefTestBase):
    __slots__ = ()

    item_type = "reftest_node"


class RefTest(RefTestBase):
    __slots__ = ()

    item_type = "reftest"


class ManualTest(URLManifestItem):
    __slots__ = ()

    item_type = "manual"


class ConformanceCheckerTest(URLManifestItem):
    __slots__ = ()

    item_type = "conformancechecker"


class VisualTest(URLManifestItem):
    __slots__ = ()

    item_type = "visual"


class Stub(URLManifestItem):
    __slots__ = ()

    item_type = "stub"


class WebDriverSpecTest(URLManifestItem):
    __slots__ = ()

    item_type = "wdspec"

    @property
    def timeout(self):
        return self._extras.get("timeout")

    def to_json(self):
        rv = super(WebDriverSpecTest, self).to_json()
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        return rv


class SupportFile(ManifestItem):
    __slots__ = ()

    item_type = "support"

    @property
    def id(self):
        return self.path
