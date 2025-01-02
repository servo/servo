# mypy: allow-untyped-defs
import os
import subprocess
import sys
from abc import ABC
from collections import defaultdict
from typing import Any, ClassVar, Dict, Optional, Set, Type
from urllib.parse import urljoin

from .wptmanifest.parser import atoms

atom_reset = atoms["Reset"]
enabled_tests = {"testharness", "reftest", "wdspec", "crashtest", "print-reftest"}


class Result(ABC):
    default_expected: ClassVar[str]
    statuses: Set[str]

    def __init__(self,
                 status,
                 message,
                 expected=None,
                 extra=None,
                 stack=None,
                 known_intermittent=None):
        if status not in self.statuses:
            raise ValueError("Unrecognised status %s" % status)
        self.status = status
        self.message = message
        self.expected = expected if expected is not None else self.default_expected
        self.known_intermittent = known_intermittent if known_intermittent is not None else []
        self.extra = extra if extra is not None else {}
        self.stack = stack

    def __repr__(self):
        return f"<{self.__module__}.{self.__class__.__name__} {self.status}>"


class SubtestResult(ABC):
    def __init__(self, name, status, message, stack=None, expected=None, known_intermittent=None):
        self.name = name
        if status not in self.statuses:
            raise ValueError("Unrecognised status %s" % status)
        self.status = status
        self.message = message
        self.stack = stack
        self.expected = expected
        self.known_intermittent = known_intermittent if known_intermittent is not None else []

    def __repr__(self):
        return f"<{self.__module__}.{self.__class__.__name__} {self.name} {self.status}>"


class TestharnessResult(Result):
    default_expected = "OK"
    statuses = {"OK", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT", "CRASH", "PRECONDITION_FAILED"}


class TestharnessSubtestResult(SubtestResult):
    default_expected = "PASS"
    statuses = {"PASS", "FAIL", "TIMEOUT", "NOTRUN", "PRECONDITION_FAILED"}


class ReftestResult(Result):
    default_expected = "PASS"
    statuses = {"PASS", "FAIL", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT",
                "CRASH"}


class WdspecResult(Result):
    default_expected = "OK"
    statuses = {"OK", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT", "CRASH"}


class WdspecSubtestResult(SubtestResult):
    default_expected = "PASS"
    statuses = {"PASS", "FAIL", "ERROR"}


class CrashtestResult(Result):
    default_expected = "PASS"
    statuses = {"PASS", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT",
                "CRASH"}


def get_run_info(metadata_root, product, **kwargs):
    return RunInfo(metadata_root, product, **kwargs)


class RunInfo(Dict[str, Any]):
    def __init__(self, metadata_root, product_name, debug,
                 browser_version=None,
                 browser_channel=None,
                 verify=None,
                 extras=None,
                 device_serials=None,
                 adb_binary=None):
        import mozinfo
        self._update_mozinfo(metadata_root)
        self.update(mozinfo.info)

        from .update.tree import GitTree
        try:
            # GitTree.__init__ throws if we are not in a git tree.
            rev = GitTree(log_error=False).rev
        except (OSError, subprocess.CalledProcessError):
            rev = None
        if rev:
            self["revision"] = rev.decode("utf-8")

        self["python_version"] = sys.version_info.major
        self["product"] = product_name
        if debug is not None:
            self["debug"] = debug
        elif "debug" not in self:
            # Default to release
            self["debug"] = False
        if browser_version:
            self["browser_version"] = browser_version
        if browser_channel:
            self["browser_channel"] = browser_channel

        self["verify"] = verify
        if "wasm" not in self:
            self["wasm"] = False
        if extras is not None:
            self.update(extras)
        if "headless" not in self:
            self["headless"] = False

        if adb_binary:
            self["adb_binary"] = adb_binary
        if device_serials:
            # Assume all emulators are identical, so query an arbitrary one.
            self._update_with_emulator_info(device_serials[0])
            self.pop("linux_distro", None)

    def _adb_run(self, device_serial, args, **kwargs):
        adb_binary = self.get("adb_binary", "adb")
        cmd = [adb_binary, "-s", device_serial, *args]
        return subprocess.check_output(cmd, **kwargs)

    def _adb_get_property(self, device_serial, prop, **kwargs):
        args = ["shell", "getprop", prop]
        value = self._adb_run(device_serial, args, **kwargs)
        return value.strip()

    def _update_with_emulator_info(self, device_serial):
        """Override system info taken from the host if using an Android
        emulator."""
        try:
            self._adb_run(device_serial, ["wait-for-device"])
            emulator_info = {
                "os": "android",
                "os_version": self._adb_get_property(
                    device_serial,
                    "ro.build.version.release",
                    encoding="utf-8",
                ),
            }
            emulator_info["version"] = emulator_info["os_version"]

            # Detect CPU info (https://developer.android.com/ndk/guides/abis#sa)
            abi64, *_ = self._adb_get_property(
                device_serial,
                "ro.product.cpu.abilist64",
                encoding="utf-8",
            ).split(',')
            if abi64:
                emulator_info["processor"] = abi64
                emulator_info["bits"] = 64
            else:
                emulator_info["processor"], *_ = self._adb_get_property(
                    device_serial,
                    "ro.product.cpu.abilist32",
                    encoding="utf-8",
                ).split(',')
                emulator_info["bits"] = 32

            self.update(emulator_info)
        except (OSError, subprocess.CalledProcessError):
            pass

    def _update_mozinfo(self, metadata_root):
        """Add extra build information from a mozinfo.json file in a parent
        directory"""
        import mozinfo

        path = metadata_root
        dirs = set()
        while path != os.path.expanduser('~'):
            if path in dirs:
                break
            dirs.add(str(path))
            path = os.path.dirname(path)

        mozinfo.find_and_update_from_json(*dirs)


def server_protocol(manifest_item):
    if hasattr(manifest_item, "h2") and manifest_item.h2:
        return "h2"
    if hasattr(manifest_item, "https") and manifest_item.https:
        return "https"
    return "http"


class Test(ABC):
    result_cls: ClassVar[Type[Result]]
    subtest_result_cls: ClassVar[Optional[Type[SubtestResult]]] = None
    test_type: ClassVar[str]
    pac = None

    default_timeout = 10  # seconds
    long_timeout = 60  # seconds

    def __init__(self, url_base, tests_root, url, inherit_metadata, test_metadata,
          timeout=None, path=None, protocol="http", subdomain=False, pac=None,
          testdriver_features=None):
        self.url_base = url_base
        self.tests_root = tests_root
        self.url = url
        self._inherit_metadata = inherit_metadata
        self._test_metadata = test_metadata
        self.timeout = timeout if timeout is not None else self.default_timeout
        self.path = path
        self.testdriver_features = testdriver_features
        self.subdomain = subdomain
        self.environment = {"url_base": url_base,
                            "protocol": protocol,
                            "prefs": self.prefs}

        if pac is not None:
            self.environment["pac"] = urljoin(self.url, pac)

    def __eq__(self, other):
        if not isinstance(other, Test):
            return False
        return self.id == other.id

    # Python 2 does not have this delegation, while Python 3 does.
    def __ne__(self, other):
        return not self.__eq__(other)

    def make_result(self,
                    status,
                    message,
                    expected=None,
                    extra=None,
                    stack=None,
                    known_intermittent=None):
        if expected is None:
            expected = self.expected()
            known_intermittent = self.known_intermittent()
        return self.result_cls(status, message, expected, extra, stack, known_intermittent)

    def make_subtest_result(self, name, status, message, stack=None, expected=None,
                            known_intermittent=None):
        if expected is None:
            expected = self.expected(name)
            known_intermittent = self.known_intermittent(name)
        return self.subtest_result_cls(name, status, message, stack, expected, known_intermittent)

    def update_metadata(self, metadata=None):
        if metadata is None:
            metadata = {}
        return metadata

    @classmethod
    def from_manifest(cls, manifest_file, manifest_item, inherit_metadata, test_metadata):
        timeout = cls.long_timeout if manifest_item.timeout == "long" else cls.default_timeout
        return cls(manifest_file.url_base,
                   manifest_file.tests_root,
                   manifest_item.url,
                   inherit_metadata,
                   test_metadata,
                   timeout=timeout,
                   path=os.path.join(manifest_file.tests_root, manifest_item.path),
                   protocol=server_protocol(manifest_item),
                   subdomain=manifest_item.subdomain)

    @property
    def id(self):
        return self.url

    @property
    def keys(self):
        return tuple()

    @property
    def abs_path(self):
        return os.path.join(self.tests_root, self.path)

    def _get_metadata(self, subtest=None):
        if self._test_metadata is not None and subtest is not None:
            return self._test_metadata.get_subtest(subtest)
        else:
            return self._test_metadata

    def itermeta(self, subtest=None):
        if self._test_metadata is not None:
            if subtest is not None:
                subtest_meta = self._get_metadata(subtest)
                if subtest_meta is not None:
                    yield subtest_meta
            yield self._get_metadata()
        yield from reversed(self._inherit_metadata)

    def disabled(self, subtest=None):
        for meta in self.itermeta(subtest):
            disabled = meta.disabled
            if disabled is not None:
                return disabled
        return None

    @property
    def restart_after(self):
        for meta in self.itermeta(None):
            restart_after = meta.restart_after
            if restart_after is not None:
                return True
        return False

    @property
    def leaks(self):
        for meta in self.itermeta(None):
            leaks = meta.leaks
            if leaks is not None:
                return leaks
        return False

    @property
    def min_assertion_count(self):
        for meta in self.itermeta(None):
            count = meta.min_assertion_count
            if count is not None:
                return count
        return 0

    @property
    def max_assertion_count(self):
        for meta in self.itermeta(None):
            count = meta.max_assertion_count
            if count is not None:
                return count
        return 0

    @property
    def lsan_disabled(self):
        for meta in self.itermeta():
            if meta.lsan_disabled is not None:
                return meta.lsan_disabled
        return False

    @property
    def lsan_allowed(self):
        lsan_allowed = set()
        for meta in self.itermeta():
            lsan_allowed |= meta.lsan_allowed
            if atom_reset in lsan_allowed:
                lsan_allowed.remove(atom_reset)
                break
        return lsan_allowed

    @property
    def lsan_max_stack_depth(self):
        for meta in self.itermeta(None):
            depth = meta.lsan_max_stack_depth
            if depth is not None:
                return depth
        return None

    @property
    def mozleak_allowed(self):
        mozleak_allowed = set()
        for meta in self.itermeta():
            mozleak_allowed |= meta.leak_allowed
            if atom_reset in mozleak_allowed:
                mozleak_allowed.remove(atom_reset)
                break
        return mozleak_allowed

    @property
    def mozleak_threshold(self):
        rv = {}
        for meta in self.itermeta(None):
            threshold = meta.leak_threshold
            for key, value in threshold.items():
                if key not in rv:
                    rv[key] = value
        return rv

    @property
    def tags(self):
        tags = set()
        for meta in self.itermeta():
            meta_tags = meta.tags
            tags |= meta_tags
            if atom_reset in meta_tags:
                tags.remove(atom_reset)
                break

        tags.add("dir:%s" % self.id.lstrip("/").split("/")[0])

        return tags

    @property
    def prefs(self):
        prefs = {}
        for meta in reversed(list(self.itermeta())):
            meta_prefs = meta.prefs
            if atom_reset in meta_prefs:
                del meta_prefs[atom_reset]
                prefs = {}
            prefs.update(meta_prefs)
        return prefs

    def expected_fail_message(self, subtest):
        if subtest is None:
            return None

        metadata = self._get_metadata(subtest)
        if metadata is None:
            return None

        try:
            return metadata.get("expected-fail-message")
        except KeyError:
            return None

    def expected(self, subtest=None):
        if subtest is None:
            default = self.result_cls.default_expected
        else:
            default = self.subtest_result_cls.default_expected

        metadata = self._get_metadata(subtest)
        if metadata is None:
            return default

        try:
            expected = metadata.get("expected")
            if isinstance(expected, str):
                return expected
            elif isinstance(expected, list):
                return expected[0]
            elif expected is None:
                return default
        except KeyError:
            return default

    def implementation_status(self):
        implementation_status = None
        for meta in self.itermeta():
            implementation_status = meta.implementation_status
            if implementation_status:
                return implementation_status

        # assuming no specific case, we are implementing it
        return "implementing"

    def known_intermittent(self, subtest=None):
        metadata = self._get_metadata(subtest)
        if metadata is None:
            return []

        try:
            expected = metadata.get("expected")
            if isinstance(expected, list):
                return expected[1:]
            return []
        except KeyError:
            return []

    def __repr__(self):
        return f"<{self.__module__}.{self.__class__.__name__} {self.id}>"


class TestharnessTest(Test):
    result_cls = TestharnessResult
    subtest_result_cls = TestharnessSubtestResult
    test_type = "testharness"

    def __init__(self, url_base, tests_root, url, inherit_metadata, test_metadata,
                 timeout=None, path=None, protocol="http", testdriver=False,
                 jsshell=False, scripts=None, subdomain=False, pac=None,
                 testdriver_features=None):
        Test.__init__(self, url_base, tests_root, url, inherit_metadata, test_metadata, timeout,
                      path, protocol, subdomain, pac, testdriver_features)

        self.testdriver = testdriver
        self.jsshell = jsshell
        self.scripts = scripts or []

    @classmethod
    def from_manifest(cls, manifest_file, manifest_item, inherit_metadata, test_metadata):
        timeout = cls.long_timeout if manifest_item.timeout == "long" else cls.default_timeout
        pac = manifest_item.pac
        testdriver_features = manifest_item.testdriver_features
        testdriver = manifest_item.testdriver if hasattr(manifest_item, "testdriver") else False
        jsshell = manifest_item.jsshell if hasattr(manifest_item, "jsshell") else False
        script_metadata = manifest_item.script_metadata or []
        scripts = [v for (k, v) in script_metadata
                   if k == "script"]
        return cls(manifest_file.url_base,
                   manifest_file.tests_root,
                   manifest_item.url,
                   inherit_metadata,
                   test_metadata,
                   timeout=timeout,
                   pac=pac,
                   testdriver_features=testdriver_features,
                   path=os.path.join(manifest_file.tests_root, manifest_item.path),
                   protocol=server_protocol(manifest_item),
                   testdriver=testdriver,
                   jsshell=jsshell,
                   scripts=scripts,
                   subdomain=manifest_item.subdomain)

    @property
    def id(self):
        return self.url


class ReftestTest(Test):
    """A reftest

    A reftest should be considered to pass if one of its references matches (see below) *and* the
    reference passes if it has any references recursively.

    Attributes:
        references (List[Tuple[str, str]]): a list of alternate references, where one must match for the test to pass
        viewport_size (Optional[Tuple[int, int]]): size of the viewport for this test, if not default
        dpi (Optional[int]): dpi to use when rendering this test, if not default

    """
    result_cls = ReftestResult
    test_type = "reftest"

    def __init__(self, url_base, tests_root, url, inherit_metadata, test_metadata, references,
                 timeout=None, path=None, viewport_size=None, dpi=None, fuzzy=None,
                 protocol="http", subdomain=False, testdriver=False):
        Test.__init__(self, url_base, tests_root, url, inherit_metadata, test_metadata, timeout,
                      path, protocol, subdomain)

        for _, ref_type in references:
            if ref_type not in ("==", "!="):
                raise ValueError

        self.references = references
        self.viewport_size = self.get_viewport_size(viewport_size)
        self.dpi = dpi
        self.testdriver = testdriver
        self._fuzzy = fuzzy or {}

    @classmethod
    def cls_kwargs(cls, manifest_test):
        return {"viewport_size": manifest_test.viewport_size,
                "dpi": manifest_test.dpi,
                "protocol": server_protocol(manifest_test),
                "fuzzy": manifest_test.fuzzy,
                "testdriver": bool(getattr(manifest_test, "testdriver", False))}

    @classmethod
    def from_manifest(cls,
                      manifest_file,
                      manifest_test,
                      inherit_metadata,
                      test_metadata):

        timeout = cls.long_timeout if manifest_test.timeout == "long" else cls.default_timeout

        url = manifest_test.url

        node = cls(manifest_file.url_base,
                   manifest_file.tests_root,
                   manifest_test.url,
                   inherit_metadata,
                   test_metadata,
                   [],
                   timeout=timeout,
                   path=manifest_test.path,
                   subdomain=manifest_test.subdomain,
                   **cls.cls_kwargs(manifest_test))

        refs_by_type = defaultdict(list)

        for ref_url, ref_type in manifest_test.references:
            refs_by_type[ref_type].append(ref_url)

        # Construct a list of all the mismatches, where we end up with mismatch_1 != url !=
        # mismatch_2 != url != mismatch_3 etc.
        #
        # Per the logic documented above, this means that none of the mismatches provided match,
        mismatch_walk = None
        if refs_by_type["!="]:
            mismatch_walk = ReftestTest(manifest_file.url_base,
                                        manifest_file.tests_root,
                                        refs_by_type["!="][0],
                                        [],
                                        None,
                                        [])
            cmp_ref = mismatch_walk
            for ref_url in refs_by_type["!="][1:]:
                cmp_self = ReftestTest(manifest_file.url_base,
                                       manifest_file.tests_root,
                                       url,
                                       [],
                                       None,
                                       [])
                cmp_ref.references.append((cmp_self, "!="))
                cmp_ref = ReftestTest(manifest_file.url_base,
                                      manifest_file.tests_root,
                                      ref_url,
                                      [],
                                      None,
                                      [])
                cmp_self.references.append((cmp_ref, "!="))

        if mismatch_walk is None:
            mismatch_refs = []
        else:
            mismatch_refs = [(mismatch_walk, "!=")]

        if refs_by_type["=="]:
            # For each == ref, add a reference to this node whose tail is the mismatch list.
            # Per the logic documented above, this means any one of the matches must pass plus all the mismatches.
            for ref_url in refs_by_type["=="]:
                ref = ReftestTest(manifest_file.url_base,
                                  manifest_file.tests_root,
                                  ref_url,
                                  [],
                                  None,
                                  mismatch_refs)
                node.references.append((ref, "=="))
        else:
            # Otherwise, we just add the mismatches directly as we are immediately into the
            # mismatch chain with no alternates.
            node.references.extend(mismatch_refs)

        return node

    def update_metadata(self, metadata):
        if "url_count" not in metadata:
            metadata["url_count"] = defaultdict(int)
        for reference, _ in self.references:
            # We assume a naive implementation in which a url with multiple
            # possible screenshots will need to take both the lhs and rhs screenshots
            # for each possible match
            metadata["url_count"][(self.environment["protocol"], reference.url)] += 1
            reference.update_metadata(metadata)
        return metadata

    def get_viewport_size(self, override):
        return override

    @property
    def id(self):
        return self.url

    @property
    def keys(self):
        return ("reftype", "refurl")

    @property
    def fuzzy(self):
        return self._fuzzy

    @property
    def fuzzy_override(self):
        values = {}
        for meta in reversed(list(self.itermeta(None))):
            value = meta.fuzzy
            if not value:
                continue
            if atom_reset in value:
                value.remove(atom_reset)
                values = {}
            for key, data in value:
                if isinstance(key, (tuple, list)):
                    key = list(key)
                    key[0] = urljoin(self.url, key[0])
                    key[1] = urljoin(self.url, key[1])
                    key = tuple(key)
                elif key:
                    # Key is just a relative url to a ref
                    key = urljoin(self.url, key)
                values[key] = data
        return values

    @property
    def page_ranges(self):
        return {}


class PrintReftestTest(ReftestTest):
    test_type = "print-reftest"

    def __init__(self, url_base, tests_root, url, inherit_metadata, test_metadata, references,
                 timeout=None, path=None, viewport_size=None, dpi=None, fuzzy=None,
                 page_ranges=None, protocol="http", subdomain=False, testdriver=False):
        super().__init__(url_base, tests_root, url, inherit_metadata, test_metadata,
                         references, timeout, path, viewport_size, dpi,
                         fuzzy, protocol, subdomain=subdomain, testdriver=testdriver)
        self._page_ranges = page_ranges

    @classmethod
    def cls_kwargs(cls, manifest_test):
        rv = super().cls_kwargs(manifest_test)
        rv["page_ranges"] = manifest_test.page_ranges
        return rv

    def get_viewport_size(self, override):
        assert override is None
        return (5*2.54, 3*2.54)

    @property
    def page_ranges(self):
        return self._page_ranges


class WdspecTest(Test):
    result_cls = WdspecResult
    subtest_result_cls = WdspecSubtestResult
    test_type = "wdspec"

    default_timeout = 25
    long_timeout = 180  # 3 minutes


class CrashTest(Test):
    result_cls = CrashtestResult
    test_type = "crashtest"

    def __init__(self, url_base, tests_root, url, inherit_metadata, test_metadata,
                 timeout=None, path=None, protocol="http", subdomain=False, testdriver=False):
        super().__init__(url_base, tests_root, url, inherit_metadata, test_metadata,
                         timeout, path, protocol, subdomain=subdomain)
        self.testdriver = testdriver

    @classmethod
    def from_manifest(cls, manifest_file, manifest_item, inherit_metadata, test_metadata):
        timeout = cls.long_timeout if manifest_item.timeout == "long" else cls.default_timeout
        return cls(manifest_file.url_base,
                   manifest_file.tests_root,
                   manifest_item.url,
                   inherit_metadata,
                   test_metadata,
                   timeout=timeout,
                   path=os.path.join(manifest_file.tests_root, manifest_item.path),
                   protocol=server_protocol(manifest_item),
                   subdomain=manifest_item.subdomain,
                   testdriver=bool(getattr(manifest_item, "testdriver", False)))


manifest_test_cls = {"reftest": ReftestTest,
                     "print-reftest": PrintReftestTest,
                     "testharness": TestharnessTest,
                     "wdspec": WdspecTest,
                     "crashtest": CrashTest}


def from_manifest(manifest_file, manifest_test, inherit_metadata, test_metadata):
    test_cls = manifest_test_cls[manifest_test.item_type]
    return test_cls.from_manifest(manifest_file, manifest_test, inherit_metadata, test_metadata)
