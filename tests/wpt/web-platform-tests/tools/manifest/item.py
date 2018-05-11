import os
from six.moves.urllib.parse import urljoin
from abc import ABCMeta, abstractmethod, abstractproperty


def get_source_file(source_files, tests_root, manifest, path):
    def make_new():
        from .sourcefile import SourceFile

        return SourceFile(tests_root, path, manifest.url_base)

    if source_files is None:
        return make_new()

    if path not in source_files:
        source_files[path] = make_new()

    return source_files[path]


class ManifestItem(object):
    __metaclass__ = ABCMeta

    item_type = None

    def __init__(self, source_file, manifest=None):
        self.manifest = manifest
        self.source_file = source_file

    @abstractproperty
    def id(self):
        """The test's id (usually its url)"""
        pass

    @property
    def path(self):
        """The test path relative to the test_root"""
        return self.source_file.rel_path

    @property
    def https(self):
        return "https" in self.source_file.meta_flags

    def key(self):
        """A unique identifier for the test"""
        return (self.item_type, self.id)

    def meta_key(self):
        """Extra metadata that doesn't form part of the test identity, but for
        which changes mean regenerating the manifest (e.g. the test timeout."""
        return ()

    def __eq__(self, other):
        if not hasattr(other, "key"):
            return False
        return self.key() == other.key()

    def __hash__(self):
        return hash(self.key() + self.meta_key())

    def __repr__(self):
        return "<%s.%s id=%s, path=%s>" % (self.__module__, self.__class__.__name__, self.id, self.path)

    def to_json(self):
        return [{}]

    @classmethod
    def from_json(cls, manifest, tests_root, path, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, path)
        return cls(source_file,
                   manifest=manifest)


class URLManifestItem(ManifestItem):
    def __init__(self, source_file, url, url_base="/", manifest=None):
        ManifestItem.__init__(self, source_file, manifest=manifest)
        self._url = url
        self.url_base = url_base

    @property
    def id(self):
        return self.url

    @property
    def url(self):
        return urljoin(self.url_base, self._url)

    def to_json(self):
        rv = [self._url, {}]
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, path, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, path)
        url, extras = obj
        return cls(source_file,
                   url,
                   url_base=manifest.url_base,
                   manifest=manifest)


class TestharnessTest(URLManifestItem):
    item_type = "testharness"

    def __init__(self, source_file, url, url_base="/", timeout=None, testdriver=False, jsshell=False, manifest=None):
        URLManifestItem.__init__(self, source_file, url, url_base=url_base, manifest=manifest)
        self.timeout = timeout
        self.testdriver = testdriver
        self.jsshell = jsshell

    def meta_key(self):
        return (self.timeout, self.testdriver)

    def to_json(self):
        rv = URLManifestItem.to_json(self)
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        if self.testdriver:
            rv[-1]["testdriver"] = self.testdriver
        if self.jsshell:
            rv[-1]["jsshell"] = True
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, path, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, path)

        url, extras = obj
        return cls(source_file,
                   url,
                   url_base=manifest.url_base,
                   timeout=extras.get("timeout"),
                   testdriver=bool(extras.get("testdriver")),
                   jsshell=bool(extras.get("jsshell")),
                   manifest=manifest)


class RefTestNode(URLManifestItem):
    item_type = "reftest_node"

    def __init__(self, source_file, url, references, url_base="/", timeout=None,
                 viewport_size=None, dpi=None, manifest=None):
        URLManifestItem.__init__(self, source_file, url, url_base=url_base, manifest=manifest)
        for _, ref_type in references:
            if ref_type not in ["==", "!="]:
                raise ValueError("Unrecognised ref_type %s" % ref_type)
        self.references = tuple(references)
        self.timeout = timeout
        self.viewport_size = viewport_size
        self.dpi = dpi

    def meta_key(self):
        return (self.timeout, self.viewport_size, self.dpi)

    def to_json(self):
        rv = [self.url, self.references, {}]
        extras = rv[-1]
        if self.timeout is not None:
            extras["timeout"] = self.timeout
        if self.viewport_size is not None:
            extras["viewport_size"] = self.viewport_size
        if self.dpi is not None:
            extras["dpi"] = self.dpi
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, path, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, path)
        url, references, extras = obj
        return cls(source_file,
                   url,
                   references,
                   url_base=manifest.url_base,
                   timeout=extras.get("timeout"),
                   viewport_size=extras.get("viewport_size"),
                   dpi=extras.get("dpi"),
                   manifest=manifest)

    def to_RefTest(self):
        if type(self) == RefTest:
            return self
        rv = RefTest.__new__(RefTest)
        rv.__dict__.update(self.__dict__)
        return rv

    def to_RefTestNode(self):
        if type(self) == RefTestNode:
            return self
        rv = RefTestNode.__new__(RefTestNode)
        rv.__dict__.update(self.__dict__)
        return rv


class RefTest(RefTestNode):
    item_type = "reftest"


class ManualTest(URLManifestItem):
    item_type = "manual"


class ConformanceCheckerTest(URLManifestItem):
    item_type = "conformancechecker"


class VisualTest(URLManifestItem):
    item_type = "visual"


class Stub(URLManifestItem):
    item_type = "stub"


class WebdriverSpecTest(URLManifestItem):
    item_type = "wdspec"

    def __init__(self, source_file, url, url_base="/", timeout=None, manifest=None):
        URLManifestItem.__init__(self, source_file, url, url_base=url_base, manifest=manifest)
        self.timeout = timeout

    def to_json(self):
        rv = URLManifestItem.to_json(self)
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, path, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, path)

        url, extras = obj
        return cls(source_file,
                   url,
                   url_base=manifest.url_base,
                   timeout=extras.get("timeout"),
                   manifest=manifest)


class SupportFile(ManifestItem):
    item_type = "support"

    @property
    def id(self):
        return self.source_file.rel_path
