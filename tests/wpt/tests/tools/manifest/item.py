import os.path
from abc import ABCMeta, abstractproperty
from inspect import isabstract
from typing import (Any, Dict, Hashable, List, Optional, Sequence, Text, Tuple, Type,
                    TYPE_CHECKING, Union, cast)
from urllib.parse import urljoin, urlparse, parse_qs

from .utils import to_os_path

if TYPE_CHECKING:
    from .manifest import Manifest

Fuzzy = Dict[Optional[Tuple[str, str, str]], List[int]]
PageRanges = Dict[str, List[int]]
item_types: Dict[str, Type["ManifestItem"]] = {}


class ManifestItemMeta(ABCMeta):
    """Custom metaclass that registers all the subclasses in the
    item_types dictionary according to the value of their item_type
    attribute, and otherwise behaves like an ABCMeta."""

    def __new__(cls: Type["ManifestItemMeta"], name: str, bases: Tuple[type], attrs: Dict[str, Any]) -> "ManifestItemMeta":
        inst = super().__new__(cls, name, bases, attrs)
        if isabstract(inst):
            return inst

        assert issubclass(inst, ManifestItem)
        item_type = cast(str, inst.item_type)

        item_types[item_type] = inst

        return inst


class ManifestItem(metaclass=ManifestItemMeta):
    __slots__ = ("_tests_root", "path")

    def __init__(self, tests_root: Text, path: Text) -> None:
        self._tests_root = tests_root
        self.path = path

    @abstractproperty
    def id(self) -> Text:
        """The test's id (usually its url)"""
        pass

    @abstractproperty
    def item_type(self) -> str:
        """The item's type"""
        pass

    @property
    def path_parts(self) -> Tuple[Text, ...]:
        return tuple(self.path.split(os.path.sep))

    def key(self) -> Hashable:
        """A unique identifier for the test"""
        return (self.item_type, self.id)

    def __eq__(self, other: Any) -> bool:
        if not hasattr(other, "key"):
            return False
        return bool(self.key() == other.key())

    def __hash__(self) -> int:
        return hash(self.key())

    def __repr__(self) -> str:
        return f"<{self.__module__}.{self.__class__.__name__} id={self.id!r}, path={self.path!r}>"

    def to_json(self) -> Tuple[Any, ...]:
        return ()

    @classmethod
    def from_json(cls,
                  manifest: "Manifest",
                  path: Text,
                  obj: Any
                  ) -> "ManifestItem":
        path = to_os_path(path)
        tests_root = manifest.tests_root
        assert tests_root is not None
        return cls(tests_root, path)


class URLManifestItem(ManifestItem):
    __slots__ = ("url_base", "_url", "_extras", "_flags")

    def __init__(self,
                 tests_root: Text,
                 path: Text,
                 url_base: Text,
                 url: Optional[Text],
                 **extras: Any
                 ) -> None:
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
    def id(self) -> Text:
        return self.url

    @property
    def url(self) -> Text:
        rel_url = self._url or self.path.replace(os.path.sep, "/")
        # we can outperform urljoin, because we know we just have path relative URLs
        if self.url_base == "/":
            return "/" + rel_url
        return urljoin(self.url_base, rel_url)

    @property
    def https(self) -> bool:
        return "https" in self._flags or "serviceworker" in self._flags or "serviceworker-module" in self._flags

    @property
    def h2(self) -> bool:
        return "h2" in self._flags

    @property
    def subdomain(self) -> bool:
        # Note: this is currently hard-coded to check for `www`, rather than
        # all possible valid subdomains. It can be extended if needed.
        return "www" in self._flags

    def to_json(self) -> Tuple[Optional[Text], Dict[Any, Any]]:
        rel_url = None if self._url == self.path.replace(os.path.sep, "/") else self._url
        rv: Tuple[Optional[Text], Dict[Any, Any]] = (rel_url, {})
        return rv

    @classmethod
    def from_json(cls,
                  manifest: "Manifest",
                  path: Text,
                  obj: Tuple[Text, Dict[Any, Any]]
                  ) -> "URLManifestItem":
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
    def timeout(self) -> Optional[Text]:
        return self._extras.get("timeout")

    @property
    def pac(self) -> Optional[Text]:
        return self._extras.get("pac")

    @property
    def testdriver(self) -> Optional[bool]:
        return self._extras.get("testdriver")

    @property
    def jsshell(self) -> Optional[Text]:
        return self._extras.get("jsshell")

    @property
    def script_metadata(self) -> Optional[List[Tuple[Text, Text]]]:
        return self._extras.get("script_metadata")

    def to_json(self) -> Tuple[Optional[Text], Dict[Text, Any]]:
        rv = super().to_json()
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        if self.pac is not None:
            rv[-1]["pac"] = self.pac
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
                 tests_root: Text,
                 path: Text,
                 url_base: Text,
                 url: Optional[Text],
                 references: Optional[List[Tuple[Text, Text]]] = None,
                 **extras: Any
                 ):
        super().__init__(tests_root, path, url_base, url, **extras)
        if references is None:
            self.references: List[Tuple[Text, Text]] = []
        else:
            self.references = references

    @property
    def timeout(self) -> Optional[Text]:
        return self._extras.get("timeout")

    @property
    def viewport_size(self) -> Optional[Text]:
        return self._extras.get("viewport_size")

    @property
    def dpi(self) -> Optional[Text]:
        return self._extras.get("dpi")

    @property
    def fuzzy(self) -> Fuzzy:
        fuzzy: Union[Fuzzy, List[Tuple[Optional[Sequence[Text]], List[int]]]] = self._extras.get("fuzzy", {})
        if not isinstance(fuzzy, list):
            return fuzzy

        rv: Fuzzy = {}
        for k, v in fuzzy:  # type: Tuple[Optional[Sequence[Text]], List[int]]
            if k is None:
                key: Optional[Tuple[Text, Text, Text]] = None
            else:
                # mypy types this as Tuple[Text, ...]
                assert len(k) == 3
                key = tuple(k)  # type: ignore
            rv[key] = v
        return rv

    @property
    def testdriver(self) -> Optional[bool]:
        return self._extras.get("testdriver")

    def to_json(self) -> Tuple[Optional[Text], List[Tuple[Text, Text]], Dict[Text, Any]]:  # type: ignore
        rel_url = None if self._url == self.path else self._url
        rv: Tuple[Optional[Text], List[Tuple[Text, Text]], Dict[Text, Any]] = (rel_url, self.references, {})
        extras = rv[-1]
        if self.timeout is not None:
            extras["timeout"] = self.timeout
        if self.viewport_size is not None:
            extras["viewport_size"] = self.viewport_size
        if self.dpi is not None:
            extras["dpi"] = self.dpi
        if self.fuzzy:
            extras["fuzzy"] = list(self.fuzzy.items())
        if self.testdriver:
            extras["testdriver"] = self.testdriver
        return rv

    @classmethod
    def from_json(cls,  # type: ignore
                  manifest: "Manifest",
                  path: Text,
                  obj: Tuple[Text, List[Tuple[Text, Text]], Dict[Any, Any]]
                  ) -> "RefTest":
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
    def page_ranges(self) -> PageRanges:
        return cast(PageRanges, self._extras.get("page_ranges", {}))

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
    def timeout(self) -> Optional[Text]:
        return None

    @property
    def testdriver(self) -> Optional[bool]:
        return self._extras.get("testdriver")

    def to_json(self):  # type: ignore
        rel_url, extras = super().to_json()
        if self.testdriver:
            extras["testdriver"] = self.testdriver
        return rel_url, extras


class WebDriverSpecTest(URLManifestItem):
    __slots__ = ()

    item_type = "wdspec"

    @property
    def timeout(self) -> Optional[Text]:
        return self._extras.get("timeout")

    def to_json(self) -> Tuple[Optional[Text], Dict[Text, Any]]:
        rv = super().to_json()
        if self.timeout is not None:
            rv[-1]["timeout"] = self.timeout
        return rv


class SupportFile(ManifestItem):
    __slots__ = ()

    item_type = "support"

    @property
    def id(self) -> Text:
        return self.path


class SpecItem(ManifestItem):
    __slots__ = ("specs")

    item_type = "spec"

    def __init__(self,
                 tests_root: Text,
                 path: Text,
                 specs: List[Text]
                 ) -> None:
        super().__init__(tests_root, path)
        self.specs = specs

    @property
    def id(self) -> Text:
        return self.path

    def to_json(self) -> Tuple[Optional[Text], Dict[Text, Any]]:
        rv: Tuple[Optional[Text], Dict[Any, Any]] = (None, {})
        for i in range(len(self.specs)):
            spec_key = f"spec_link{i+1}"
            rv[-1][spec_key] = self.specs[i]
        return rv

    @classmethod
    def from_json(cls,
                  manifest: "Manifest",
                  path: Text,
                  obj: Any
                  ) -> "ManifestItem":
        """Not properly implemented and is not used."""
        return cls("/", "", [])
