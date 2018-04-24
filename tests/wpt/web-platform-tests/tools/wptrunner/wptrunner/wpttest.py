import os
from collections import defaultdict

from wptmanifest.parser import atoms

atom_reset = atoms["Reset"]
enabled_tests = set(["testharness", "reftest", "wdspec"])


class Result(object):
    def __init__(self, status, message, expected=None, extra=None):
        if status not in self.statuses:
            raise ValueError("Unrecognised status %s" % status)
        self.status = status
        self.message = message
        self.expected = expected
        self.extra = extra

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
    statuses = set(["OK", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT", "CRASH"])


class TestharnessSubtestResult(SubtestResult):
    default_expected = "PASS"
    statuses = set(["PASS", "FAIL", "TIMEOUT", "NOTRUN"])


class ReftestResult(Result):
    default_expected = "PASS"
    statuses = set(["PASS", "FAIL", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT",
                    "CRASH"])


class WdspecResult(Result):
    default_expected = "OK"
    statuses = set(["OK", "ERROR", "INTERNAL-ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT", "CRASH"])


class WdspecSubtestResult(SubtestResult):
    default_expected = "PASS"
    statuses = set(["PASS", "FAIL", "ERROR"])


def get_run_info(metadata_root, product, **kwargs):
    return RunInfo(metadata_root, product, **kwargs)


class RunInfo(dict):
    def __init__(self, metadata_root, product, debug, extras=None):
        import mozinfo

        self._update_mozinfo(metadata_root)
        self.update(mozinfo.info)
        self["product"] = product
        if debug is not None:
            self["debug"] = debug
        elif "debug" not in self:
            # Default to release
            self["debug"] = False
        if product == "firefox" and "stylo" not in self:
            self["stylo"] = False
        if "STYLO_FORCE_ENABLED" in os.environ:
            self["stylo"] = True
        if "STYLO_FORCE_DISABLED" in os.environ:
            self["stylo"] = False
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
    def from_manifest(cls, manifest_item, inherit_metadata, test_metadata):
        timeout = cls.long_timeout if manifest_item.timeout == "long" else cls.default_timeout
        protocol = "https" if hasattr(manifest_item, "https") and manifest_item.https else "http"
        return cls(manifest_item.source_file.tests_root,
                   manifest_item.url,
                   inherit_metadata,
                   test_metadata,
                   timeout=timeout,
                   path=manifest_item.source_file.path,
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
        for metadata in self._inherit_metadata:
            yield metadata

        if self._test_metadata is not None:
            yield self._get_metadata()
            if subtest is not None:
                subtest_meta = self._get_metadata(subtest)
                if subtest_meta is not None:
                    yield subtest_meta

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
    def tags(self):
        tags = set()
        for meta in self.itermeta():
            meta_tags = meta.tags
            if atom_reset in meta_tags:
                tags = meta_tags.copy()
                tags.remove(atom_reset)
            else:
                tags |= meta_tags

        tags.add("dir:%s" % self.id.lstrip("/").split("/")[0])

        return tags

    @property
    def prefs(self):
        prefs = {}
        for meta in self.itermeta():
            meta_prefs = meta.prefs
            if atom_reset in prefs:
                prefs = meta_prefs.copy()
                del prefs[atom_reset]
            else:
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
                 timeout=None, path=None, protocol="http", testdriver=False):
        Test.__init__(self, tests_root, url, inherit_metadata, test_metadata, timeout,
                      path, protocol)

        self.testdriver = testdriver

    @classmethod
    def from_manifest(cls, manifest_item, inherit_metadata, test_metadata):
        timeout = cls.long_timeout if manifest_item.timeout == "long" else cls.default_timeout
        protocol = "https" if hasattr(manifest_item, "https") and manifest_item.https else "http"
        testdriver = manifest_item.testdriver if hasattr(manifest_item, "testdriver") else False
        return cls(manifest_item.source_file.tests_root,
                   manifest_item.url,
                   inherit_metadata,
                   test_metadata,
                   timeout=timeout,
                   path=manifest_item.source_file.path,
                   protocol=protocol,
                   testdriver=testdriver)

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
                 timeout=None, path=None, viewport_size=None, dpi=None, protocol="http"):
        Test.__init__(self, tests_root, url, inherit_metadata, test_metadata, timeout,
                      path, protocol)

        for _, ref_type in references:
            if ref_type not in ("==", "!="):
                raise ValueError

        self.references = references
        self.viewport_size = viewport_size
        self.dpi = dpi

    @classmethod
    def from_manifest(cls,
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

        node = cls(manifest_test.source_file.tests_root,
                   manifest_test.url,
                   inherit_metadata,
                   test_metadata,
                   [],
                   timeout=timeout,
                   path=manifest_test.path,
                   viewport_size=manifest_test.viewport_size,
                   dpi=manifest_test.dpi,
                   protocol="https" if hasattr(manifest_test, "https") and manifest_test.https else "http")

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

            manifest_node = manifest_test.manifest.get_reference(ref_url)
            if manifest_node:
                reference = ReftestTest.from_manifest(manifest_node,
                                                      [],
                                                      None,
                                                      nodes,
                                                      references_seen)
            else:
                reference = ReftestTest(manifest_test.source_file.tests_root,
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


class WdspecTest(Test):

    result_cls = WdspecResult
    subtest_result_cls = WdspecSubtestResult
    test_type = "wdspec"

    default_timeout = 25
    long_timeout = 120


manifest_test_cls = {"reftest": ReftestTest,
                     "testharness": TestharnessTest,
                     "manual": ManualTest,
                     "wdspec": WdspecTest}


def from_manifest(manifest_test, inherit_metadata, test_metadata):
    test_cls = manifest_test_cls[manifest_test.item_type]
    return test_cls.from_manifest(manifest_test, inherit_metadata, test_metadata)
