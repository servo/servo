import urlparse
from abc import ABCMeta, abstractmethod, abstractproperty

item_types = ["testharness", "reftest", "manual", "stub", "wdspec"]

def get_source_file(source_files, tests_root, manifest, path):
    def make_new():
        from sourcefile import SourceFile

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

    def to_json(self):
        return {"path": self.path}

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
        return urlparse.urljoin(self.url_base, self._url)

    def to_json(self):
        rv = ManifestItem.to_json(self)
        rv["url"] = self._url
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, obj["path"])
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
        source_file = get_source_file(source_files, tests_root, manifest, obj["path"])
        return cls(source_file,
                   obj["url"],
                   url_base=manifest.url_base,
                   timeout=obj.get("timeout"),
                   manifest=manifest)


class RefTest(URLManifestItem):
    item_type = "reftest"

    def __init__(self, source_file, url, references, url_base="/", timeout=None,
                 manifest=None):
        URLManifestItem.__init__(self, source_file, url, url_base=url_base, manifest=manifest)
        for _, ref_type in references:
            if ref_type not in ["==", "!="]:
                raise ValueError, "Unrecognised ref_type %s" % ref_type
        self.references = tuple(references)
        self.timeout = timeout

    @property
    def is_reference(self):
        return self.source_file.name_is_reference

    def meta_key(self):
        return (self.timeout,)

    def to_json(self):
        rv = URLManifestItem.to_json(self)
        rv["references"] = self.references
        if self.timeout is not None:
            rv["timeout"] = self.timeout
        return rv

    @classmethod
    def from_json(cls, manifest, tests_root, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, obj["path"])
        return cls(source_file,
                   obj["url"],
                   obj["references"],
                   url_base=manifest.url_base,
                   timeout=obj.get("timeout"),
                   manifest=manifest)


class ManualTest(URLManifestItem):
    item_type = "manual"

class Stub(URLManifestItem):
    item_type = "stub"

class WebdriverSpecTest(ManifestItem):
    item_type = "wdspec"

    @property
    def id(self):
        return self.path

    @classmethod
    def from_json(cls, manifest, tests_root, obj, source_files=None):
        source_file = get_source_file(source_files, tests_root, manifest, obj["path"])
        return cls(source_file, manifest=manifest)
