from copy import copy
from six import iteritems
from six.moves.urllib.parse import urljoin, urlparse
from abc import ABCMeta, abstractproperty

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

    item_type = None

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
        return cls(manifest.tests_root, path)


class URLManifestItem(ManifestItem):
    __slots__ = ("url_base", "_url", "_extras")

    def __init__(self, tests_root, path, url_base, url, **extras):
        super(URLManifestItem, self).__init__(tests_root, path)
        self.url_base = url_base
        self._url = url
        self._extras = extras

    @property
    def _source_file(self):
        """create a SourceFile for the item"""
        from .sourcefile import SourceFile
        return SourceFile(self._tests_root, self.path, self.url_base)

    @property
    def id(self):
        return self.url

    @property
    def url(self):
        # we can outperform urljoin, because we know we just have path relative URLs
        if self._url[0] == "/":
            # TODO: MANIFEST6
            # this is actually a bug in older generated manifests, _url shouldn't
            # be an absolute path
            return self._url
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
        url, extras = obj
        return cls(manifest.tests_root,
                   path,
                   manifest.url_base,
                   url,
                   **extras)


class TestharnessTest(URLManifestItem):
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
        if "script_metadata" in self._extras:
            return self._extras["script_metadata"]
        else:
            # TODO: MANIFEST6
            # this branch should go when the manifest version is bumped
            return self._source_file.script_metadata

    def to_json(self):
        rv = super(TestharnessTest, self).to_json()
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        if self.testdriver:
            rv[-1]["testdriver"] = self.testdriver
        if self.jsshell:
            rv[-1]["jsshell"] = True
        if self.script_metadata is not None:
            # we store this even if it is [] to avoid having to read the source file
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
        rv = self._extras.get("fuzzy", [])
        if isinstance(rv, list):
            return {tuple(item[0]): item[1]
                    for item in self._extras.get("fuzzy", [])}
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
    item_type = "reftest_node"


class RefTest(RefTestBase):
    item_type = "reftest"


class ManualTest(URLManifestItem):
    item_type = "manual"


class ConformanceCheckerTest(URLManifestItem):
    item_type = "conformancechecker"


class VisualTest(URLManifestItem):
    item_type = "visual"


class Stub(URLManifestItem):
    item_type = "stub"


class WebDriverSpecTest(URLManifestItem):
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
    item_type = "support"

    @property
    def id(self):
        return self.path
