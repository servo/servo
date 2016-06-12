import os
from six.moves.urllib.parse import urljoin
from abc import ABCMeta, abstractmethod, abstractproperty

from .utils import from_os_path, to_os_path

item_types = ["testharness", "reftest", "manual", "stub", "wdspec"]


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
        return {"path": from_os_path(self.path)}

    @classmethod
    def from_json(self, manifest, tests_root, obj, source_files=None):
        raise NotImplementedError


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
        rv = ManifestItem.to_json(self)
        rv["url"] = self._url
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest,
                                      to_os_path(obj["path"]))
        return cls(source_file,
                   obj["url"],
                   url_base=manifest.url_base,
                   manifest=manifest)


class TestharnessTest(URLManifestItem):
    item_type = "testharness"

    def __init__(self, source_file, url, url_base="/", timeout=None, manifest=None):
        URLManifestItem.__init__(self, source_file, url, url_base=url_base, manifest=manifest)
        self.timeout = timeout

    def meta_key(self):
        return (self.timeout,)

    def to_json(self):
        rv = URLManifestItem.to_json(self)
        if self.timeout is not None:
            rv["timeout"] = self.timeout
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest,
                                      to_os_path(obj["path"]))
        return cls(source_file,
                   obj["url"],
                   url_base=manifest.url_base,
                   timeout=obj.get("timeout"),
                   manifest=manifest)


class RefTest(URLManifestItem):
    item_type = "reftest"

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

    @property
    def is_reference(self):
        return self.source_file.name_is_reference

    def meta_key(self):
        return (self.timeout, self.viewport_size, self.dpi)

    def to_json(self):
        rv = URLManifestItem.to_json(self)
        rv["references"] = self.references
        if self.timeout is not None:
            rv["timeout"] = self.timeout
        if self.viewport_size is not None:
            rv["viewport_size"] = self.viewport_size
        if self.dpi is not None:
            rv["dpi"] = self.dpi
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest,
                                      to_os_path(obj["path"]))
        return cls(source_file,
                   obj["url"],
                   obj["references"],
                   url_base=manifest.url_base,
                   timeout=obj.get("timeout"),
                   viewport_size=obj.get("viewport_size"),
                   dpi=obj.get("dpi"),
                   manifest=manifest)


class ManualTest(URLManifestItem):
    item_type = "manual"


class Stub(URLManifestItem):
    item_type = "stub"


class WebdriverSpecTest(URLManifestItem):
    item_type = "wdspec"

    def __init__(self, source_file, url, url_base="/", timeout=None, manifest=None):
        URLManifestItem.__init__(self, source_file, url, url_base=url_base, manifest=manifest)
        self.timeout = timeout
