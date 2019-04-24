import os
import subprocess
from six.moves.urllib.parse import urljoin
from collections import defaultdict

from .wptmanifest.parser import atoms

atom_reset = atoms["Reset"]
enabled_tests = {"testharness", "reftest", "wdspec"}


class Result(object):
    def __init__(self, status, message, expected=None, extra=None, stack=None):
        if status not in self.statuses:
            raise ValueError("Unrecognised status %s" % status)
        self.status = status
        self.message = message
        self.expected = expected
        self.extra = extra if extra is not None else {}
        self.stack = stack

    def __repr__(self):
        return "<%s.%s %s>" % (self.__module__, self.__class__.__name__, self.status)


class SubtestResult(object):
    def __init__(self, name, status, message, stack=None, expected=None):
        self.name = name
        if status not in self.statuses:
            raise ValueError("Unrecognised status %s" % status)
        self.status = status
        self.message = message
        self.stack = stack
        self.expected = expected

    def __repr__(self):
        return "<%s.%s %s %s>" % (self.__module__, self.__class__.__name__, self.name, self.status)


class TestharnessResult(Result):
    default_expected = "OK"
    statuses = {"OK", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT", "CRASH"}


class TestharnessSubtestResult(SubtestResult):
    default_expected = "PASS"
    statuses = {"PASS", "FAIL", "TIMEOUT", "NOTRUN"}


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


def get_run_info(metadata_root, product, **kwargs):
    return RunInfo(metadata_root, product, **kwargs)


class RunInfo(dict):
    def __init__(self, metadata_root, product, debug,
                 browser_version=None,
                 browser_channel=None,
                 verify=None,
                 extras=None):
        import mozinfo
        self._update_mozinfo(metadata_root)
        self.update(mozinfo.info)

        from update.tree import GitTree
        try:
            # GitTree.__init__ throws if we are not in a git tree.
            rev = GitTree(log_error=False).rev
        except (OSError, subprocess.CalledProcessError):
            rev = None
        if rev:
            self["revision"] = rev

        self["product"] = product
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
            path = os.path.split(path)[0]

        mozinfo.find_and_update_from_json(*dirs)


class Test(object):

    result_cls = None
    subtest_result_cls = None
    test_type = None

    default_timeout = 10  # seconds
    long_timeout = 60  # seconds

    def __init__(self, tests_root, url, inherit_metadata, test_metadata,
                 timeout=None, path=None, protocol="http"):
        self.tests_root = tests_root
        self.url = url
        self._inherit_metadata = inherit_metadata
        self._test_metadata = test_metadata
        self.timeout = timeout if timeout is not None else self.default_timeout
        self.path = path
        self.environment = {"protocol": protocol, "prefs": self.prefs}

    def __eq__(self, other):
        return self.id == other.id

    def update_metadata(self, metadata=None):
        if metadata is None:
            metadata = {}
        return metadata

    @classmethod
    def from_manifest(cls, manifest_file, manifest_item, inherit_metadata, test_metadata):
        timeout = cls.long_timeout if manifest_item.timeout == "long" else cls.default_timeout
        protocol = "https" if hasattr(manifest_item, "https") and manifest_item.https else "http"
        return cls(manifest_file.tests_root,
                   manifest_item.url,
                   inherit_metadata,
                   test_metadata,
                   timeout=timeout,
                   path=os.path.join(manifest_file.tests_root, manifest_item.path),
                   protocol=protocol)

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
        for metadata in reversed(self._inherit_metadata):
            yield metadata

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
            for key, value in threshold.iteritems():
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

    def expected(self, subtest=None):
        if subtest is None:
            default = self.result_cls.default_expected
        else:
            default = self.subtest_result_cls.default_expected

        metadata = self._get_metadata(subtest)
        if metadata is None:
            return default

        try:
            return metadata.get("expected")
        except KeyError:
            return default

    def __repr__(self):
        return "<%s.%s %s>" % (self.__module__, self.__class__.__name__, self.id)


class TestharnessTest(Test):
    result_cls = TestharnessResult
    subtest_result_cls = TestharnessSubtestResult
    test_type = "testharness"

    def __init__(self, tests_root, url, inherit_metadata, test_metadata,
                 timeout=None, path=None, protocol="http", testdriver=False,
                 jsshell=False, scripts=None):
        Test.__init__(self, tests_root, url, inherit_metadata, test_metadata, timeout,
                      path, protocol)

        self.testdriver = testdriver
        self.jsshell = jsshell
        self.scripts = scripts or []

    @classmethod
    def from_manifest(cls, manifest_file, manifest_item, inherit_metadata, test_metadata):
        timeout = cls.long_timeout if manifest_item.timeout == "long" else cls.default_timeout
        protocol = "https" if hasattr(manifest_item, "https") and manifest_item.https else "http"
        testdriver = manifest_item.testdriver if hasattr(manifest_item, "testdriver") else False
        jsshell = manifest_item.jsshell if hasattr(manifest_item, "jsshell") else False
        script_metadata = manifest_item.script_metadata or []
        scripts = [v for (k, v) in script_metadata if k == b"script"]
        return cls(manifest_file.tests_root,
                   manifest_item.url,
                   inherit_metadata,
                   test_metadata,
                   timeout=timeout,
                   path=os.path.join(manifest_file.tests_root, manifest_item.path),
                   protocol=protocol,
                   testdriver=testdriver,
                   jsshell=jsshell,
                   scripts=scripts
                   )

    @property
    def id(self):
        return self.url


class ManualTest(Test):
    test_type = "manual"

    @property
    def id(self):
        return self.url


class ReftestTest(Test):
    result_cls = ReftestResult
    test_type = "reftest"

    def __init__(self, tests_root, url, inherit_metadata, test_metadata, references,
                 timeout=None, path=None, viewport_size=None, dpi=None, fuzzy=None, protocol="http"):
        Test.__init__(self, tests_root, url, inherit_metadata, test_metadata, timeout,
                      path, protocol)

        for _, ref_type in references:
            if ref_type not in ("==", "!="):
                raise ValueError

        self.references = references
        self.viewport_size = viewport_size
        self.dpi = dpi
        self._fuzzy = fuzzy or {}

    @classmethod
    def from_manifest(cls,
                      manifest_file,
                      manifest_test,
                      inherit_metadata,
                      test_metadata,
                      nodes=None,
                      references_seen=None):

        timeout = cls.long_timeout if manifest_test.timeout == "long" else cls.default_timeout

        if nodes is None:
            nodes = {}
        if references_seen is None:
            references_seen = set()

        url = manifest_test.url

        node = cls(manifest_file.tests_root,
                   manifest_test.url,
                   inherit_metadata,
                   test_metadata,
                   [],
                   timeout=timeout,
                   path=manifest_test.path,
                   viewport_size=manifest_test.viewport_size,
                   dpi=manifest_test.dpi,
                   protocol="https" if hasattr(manifest_test, "https") and manifest_test.https else "http",
                   fuzzy=manifest_test.fuzzy)

        nodes[url] = node

        for ref_url, ref_type in manifest_test.references:
            comparison_key = (ref_type,) + tuple(sorted([url, ref_url]))
            if ref_url in nodes:
                manifest_node = ref_url
                if comparison_key in references_seen:
                    # We have reached a cycle so stop here
                    # Note that just seeing a node for the second time is not
                    # enough to detect a cycle because
                    # A != B != C != A must include C != A
                    # but A == B == A should not include the redundant B == A.
                    continue

            references_seen.add(comparison_key)

            manifest_node = manifest_file.get_reference(ref_url)
            if manifest_node:
                reference = ReftestTest.from_manifest(manifest_file,
                                                      manifest_node,
                                                      [],
                                                      None,
                                                      nodes,
                                                      references_seen)
            else:
                reference = ReftestTest(manifest_file.tests_root,
                                        ref_url,
                                        [],
                                        None,
                                        [])

            node.references.append((reference, ref_type))

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
                if len(key) == 3:
                    key[0] = urljoin(self.url, key[0])
                    key[1] = urljoin(self.url, key[1])
                else:
                    # Key is just a relative url to a ref
                    key = urljoin(self.url, key)
                values[key] = data
        return values


class WdspecTest(Test):

    result_cls = WdspecResult
    subtest_result_cls = WdspecSubtestResult
    test_type = "wdspec"

    default_timeout = 25
    long_timeout = 180  # 3 minutes


manifest_test_cls = {"reftest": ReftestTest,
                     "testharness": TestharnessTest,
                     "manual": ManualTest,
                     "wdspec": WdspecTest}


def from_manifest(manifest_file, manifest_test, inherit_metadata, test_metadata):
    test_cls = manifest_test_cls[manifest_test.item_type]
    return test_cls.from_manifest(manifest_file, manifest_test, inherit_metadata, test_metadata)
