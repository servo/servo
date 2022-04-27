import os.path
from inspect import isabstract
from urllib.parse import urljoin, urlparse, parse_qs
from abc import ABCMeta, abstractproperty

from .utils import to_os_path

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any, Dict, Hashable, List, Optional, Sequence, Text, Tuple, Type, Union, cast
    from .manifest import Manifest
    Fuzzy = Dict[Optional[Tuple[str, str, str]], List[int]]
    PageRanges = Dict[str, List[int]]

item_types = {}  # type: Dict[str, Type[ManifestItem]]


class ManifestItemMeta(ABCMeta):
    """Custom metaclass that registers all the subclasses in the
    item_types dictionary according to the value of their item_type
    attribute, and otherwise behaves like an ABCMeta."""

    def __new__(cls, name, bases, attrs):
        # type: (Type[ManifestItemMeta], str, Tuple[type], Dict[str, Any]) -> ManifestItemMeta
        inst = super().__new__(cls, name, bases, attrs)
        if isabstract(inst):
            return inst

        assert issubclass(inst, ManifestItem)
        if MYPY:
            inst_ = cast(Type[ManifestItem], inst)
            item_type = cast(str, inst_.item_type)
        else:
            inst_ = inst
            assert isinstance(inst_.item_type, str)
            item_type = inst_.item_type

        item_types[item_type] = inst_

        return inst_


class ManifestItem(metaclass=ManifestItemMeta):
    __slots__ = ("_tests_root", "path")

    def __init__(self, tests_root, path):
        # type: (Text, Text) -> None
        self._tests_root = tests_root
        self.path = path

    @abstractproperty
    def id(self):
        # type: () -> Text
        """The test's id (usually its url)"""
        pass

    @abstractproperty
    def item_type(self):
        # type: () -> str
        """The item's type"""
        pass

    @property
    def path_parts(self):
        # type: () -> Tuple[Text, ...]
        return tuple(self.path.split(os.path.sep))

    def key(self):
        # type: () -> Hashable
        """A unique identifier for the test"""
        return (self.item_type, self.id)

    def __eq__(self, other):
        # type: (Any) -> bool
        if not hasattr(other, "key"):
            return False
        return bool(self.key() == other.key())

    def __hash__(self):
        # type: () -> int
        return hash(self.key())

    def __repr__(self):
        # type: () -> str
        return f"<{self.__module__}.{self.__class__.__name__} id={self.id!r}, path={self.path!r}>"

    def to_json(self):
        # type: () -> Tuple[Any, ...]
        return ()

    @classmethod
    def from_json(cls,
                  manifest,  # type: Manifest
                  path,  # type: Text
                  obj  # type: Any
                  ):
        # type: (...) -> ManifestItem
        path = to_os_path(path)
        tests_root = manifest.tests_root
        assert tests_root is not None
        return cls(tests_root, path)


class URLManifestItem(ManifestItem):
    __slots__ = ("url_base", "_url", "_extras", "_flags")

    def __init__(self,
                 tests_root,  # type: Text
                 path,  # type: Text
                 url_base,  # type: Text
                 url,  # type: Optional[Text]
                 **extras  # type: Any
                 ):
        # type: (...) -> None
        super().__init__(tests_root, path)
        assert url_base[0] == "/"
        self.url_base = url_base
        assert url is None or url[0] != "/"
        self._url = url
        self._extras = extras
        parsed_url = urlparse(self.url)
        self._flags = (set(parsed_url.path.rsplit("/", 1)[1].split(".")[1:-1]) |
                       set(parse_qs(parsed_url.query).get("wpt_flags", [])))

    @property
    def id(self):
        # type: () -> Text
        return self.url

    @property
    def url(self):
        # type: () -> Text
        rel_url = self._url or self.path.replace(os.path.sep, "/")
        # we can outperform urljoin, because we know we just have path relative URLs
        if self.url_base == "/":
            return "/" + rel_url
        return urljoin(self.url_base, rel_url)

    @property
    def https(self):
        # type: () -> bool
        return "https" in self._flags or "serviceworker" in self._flags or "serviceworker-module" in self._flags

    @property
    def h2(self):
        # type: () -> bool
        return "h2" in self._flags

    @property
    def subdomain(self):
        # type: () -> bool
        # Note: this is currently hard-coded to check for `www`, rather than
        # all possible valid subdomains. It can be extended if needed.
        return "www" in self._flags

    def to_json(self):
        # type: () -> Tuple[Optional[Text], Dict[Any, Any]]
        rel_url = None if self._url == self.path.replace(os.path.sep, "/") else self._url
        rv = (rel_url, {})  # type: Tuple[Optional[Text], Dict[Any, Any]]
        return rv

    @classmethod
    def from_json(cls,
                  manifest,  # type: Manifest
                  path,  # type: Text
                  obj  # type: Tuple[Text, Dict[Any, Any]]
                  ):
        # type: (...) -> URLManifestItem
        path = to_os_path(path)
        url, extras = obj
        tests_root = manifest.tests_root
        assert tests_root is not None
        return cls(tests_root,
                   path,
                   manifest.url_base,
                   url,
                   **extras)


class TestharnessTest(URLManifestItem):
    __slots__ = ()

    item_type = "testharness"

    @property
    def timeout(self):
        # type: () -> Optional[Text]
        return self._extras.get("timeout")

    @property
    def testdriver(self):
        # type: () -> Optional[Text]
        return self._extras.get("testdriver")

    @property
    def jsshell(self):
        # type: () -> Optional[Text]
        return self._extras.get("jsshell")

    @property
    def script_metadata(self):
        # type: () -> Optional[List[Tuple[Text, Text]]]
        return self._extras.get("script_metadata")

    def to_json(self):
        # type: () -> Tuple[Optional[Text], Dict[Text, Any]]
        rv = super().to_json()
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        if self.testdriver:
            rv[-1]["testdriver"] = self.testdriver
        if self.jsshell:
            rv[-1]["jsshell"] = True
        if self.script_metadata:
            rv[-1]["script_metadata"] = [(k, v) for (k,v) in self.script_metadata]
        return rv


class RefTest(URLManifestItem):
    __slots__ = ("references",)

    item_type = "reftest"

    def __init__(self,
                 tests_root,  # type: Text
                 path,  # type: Text
                 url_base,  # type: Text
                 url,  # type: Optional[Text]
                 references=None,  # type: Optional[List[Tuple[Text, Text]]]
                 **extras  # type: Any
                 ):
        super().__init__(tests_root, path, url_base, url, **extras)
        if references is None:
            self.references = []  # type: List[Tuple[Text, Text]]
        else:
            self.references = references

    @property
    def timeout(self):
        # type: () -> Optional[Text]
        return self._extras.get("timeout")

    @property
    def viewport_size(self):
        # type: () -> Optional[Text]
        return self._extras.get("viewport_size")

    @property
    def dpi(self):
        # type: () -> Optional[Text]
        return self._extras.get("dpi")

    @property
    def fuzzy(self):
        # type: () -> Fuzzy
        fuzzy = self._extras.get("fuzzy", {})  # type: Union[Fuzzy, List[Tuple[Optional[Sequence[Text]], List[int]]]]
        if not isinstance(fuzzy, list):
            return fuzzy

        rv = {}  # type: Fuzzy
        for k, v in fuzzy:  # type: Tuple[Optional[Sequence[Text]], List[int]]
            if k is None:
                key = None  # type: Optional[Tuple[Text, Text, Text]]
            else:
                # mypy types this as Tuple[Text, ...]
                assert len(k) == 3
                key = tuple(k)  # type: ignore
            rv[key] = v
        return rv

    def to_json(self):  # type: ignore
        # type: () -> Tuple[Optional[Text], List[Tuple[Text, Text]], Dict[Text, Any]]
        rel_url = None if self._url == self.path else self._url
        rv = (rel_url, self.references, {})  # type: Tuple[Optional[Text], List[Tuple[Text, Text]], Dict[Text, Any]]
        extras = rv[-1]
        if self.timeout is not None:
            extras["timeout"] = self.timeout
        if self.viewport_size is not None:
            extras["viewport_size"] = self.viewport_size
        if self.dpi is not None:
            extras["dpi"] = self.dpi
        if self.fuzzy:
            extras["fuzzy"] = list(self.fuzzy.items())
        return rv

    @classmethod
    def from_json(cls,  # type: ignore
                  manifest,  # type: Manifest
                  path,  # type: Text
                  obj  # type: Tuple[Text, List[Tuple[Text, Text]], Dict[Any, Any]]
                  ):
        # type: (...) -> RefTest
        tests_root = manifest.tests_root
        assert tests_root is not None
        path = to_os_path(path)
        url, references, extras = obj
        return cls(tests_root,
                   path,
                   manifest.url_base,
                   url,
                   references,
                   **extras)


class PrintRefTest(RefTest):
    __slots__ = ("references",)

    item_type = "print-reftest"

    @property
    def page_ranges(self):
        # type: () -> PageRanges
        return self._extras.get("page_ranges", {})

    def to_json(self):  # type: ignore
        rv = super().to_json()
        if self.page_ranges:
            rv[-1]["page_ranges"] = self.page_ranges
        return rv


class ManualTest(URLManifestItem):
    __slots__ = ()

    item_type = "manual"


class ConformanceCheckerTest(URLManifestItem):
    __slots__ = ()

    item_type = "conformancechecker"


class VisualTest(URLManifestItem):
    __slots__ = ()

    item_type = "visual"


class CrashTest(URLManifestItem):
    __slots__ = ()

    item_type = "crashtest"

    @property
    def timeout(self):
        # type: () -> Optional[Text]
        return None


class WebDriverSpecTest(URLManifestItem):
    __slots__ = ()

    item_type = "wdspec"

    @property
    def timeout(self):
        # type: () -> Optional[Text]
        return self._extras.get("timeout")

    def to_json(self):
        # type: () -> Tuple[Optional[Text], Dict[Text, Any]]
        rv = super().to_json()
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        return rv


class SupportFile(ManifestItem):
    __slots__ = ()

    item_type = "support"

    @property
    def id(self):
        # type: () -> Text
        return self.path
