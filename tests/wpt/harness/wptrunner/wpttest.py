# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

DEFAULT_TIMEOUT = 10  # seconds
LONG_TIMEOUT = 60  # seconds

import os

import mozinfo


class Result(object):
    def __init__(self, status, message, expected=None, extra=None):
        if status not in self.statuses:
            raise ValueError("Unrecognised status %s" % status)
        self.status = status
        self.message = message
        self.expected = expected
        self.extra = extra


class SubtestResult(object):
    def __init__(self, name, status, message, stack=None, expected=None):
        self.name = name
        if status not in self.statuses:
            raise ValueError("Unrecognised status %s" % status)
        self.status = status
        self.message = message
        self.stack = stack
        self.expected = expected


class TestharnessResult(Result):
    default_expected = "OK"
    statuses = set(["OK", "ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT", "CRASH"])


class ReftestResult(Result):
    default_expected = "PASS"
    statuses = set(["PASS", "FAIL", "ERROR", "TIMEOUT", "EXTERNAL-TIMEOUT", "CRASH"])


class TestharnessSubtestResult(SubtestResult):
    default_expected = "PASS"
    statuses = set(["PASS", "FAIL", "TIMEOUT", "NOTRUN"])


def get_run_info(metadata_root, product, **kwargs):
    if product == "b2g":
        return B2GRunInfo(metadata_root, product, **kwargs)
    else:
        return RunInfo(metadata_root, product, **kwargs)


class RunInfo(dict):
    def __init__(self, metadata_root, product, debug):
        self._update_mozinfo(metadata_root)
        self.update(mozinfo.info)
        self["product"] = product
        if not "debug" in self:
            self["debug"] = debug

    def _update_mozinfo(self, metadata_root):
        """Add extra build information from a mozinfo.json file in a parent
        directory"""
        path = metadata_root
        dirs = set()
        while path != os.path.expanduser('~'):
            if path in dirs:
                break
            dirs.add(str(path))
            path = os.path.split(path)[0]

        mozinfo.find_and_update_from_json(*dirs)

class B2GRunInfo(RunInfo):
    def __init__(self, *args, **kwargs):
        RunInfo.__init__(self, *args, **kwargs)
        self["os"] = "b2g"


class Test(object):
    result_cls = None
    subtest_result_cls = None

    def __init__(self, url, expected_metadata, timeout=DEFAULT_TIMEOUT, path=None,
                 protocol="http"):
        self.url = url
        self._expected_metadata = expected_metadata
        self.timeout = timeout
        self.path = path
        if expected_metadata:
            prefs = expected_metadata.prefs()
        else:
            prefs = []
        self.environment = {"protocol": protocol, "prefs": prefs}

    def __eq__(self, other):
        return self.id == other.id

    @classmethod
    def from_manifest(cls, manifest_item, expected_metadata):
        timeout = LONG_TIMEOUT if manifest_item.timeout == "long" else DEFAULT_TIMEOUT
        return cls(manifest_item.url,
                   expected_metadata,
                   timeout=timeout,
                   path=manifest_item.path,
                   protocol="https" if hasattr(manifest_item, "https") and manifest_item.https else "http")


    @property
    def id(self):
        return self.url

    @property
    def keys(self):
        return tuple()

    def _get_metadata(self, subtest):
        if self._expected_metadata is None:
            return None

        if subtest is not None:
            metadata = self._expected_metadata.get_subtest(subtest)
        else:
            metadata = self._expected_metadata
        return metadata

    def disabled(self, subtest=None):
        metadata = self._get_metadata(subtest)
        if metadata is None:
            return False

        return metadata.disabled()

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


class TestharnessTest(Test):
    result_cls = TestharnessResult
    subtest_result_cls = TestharnessSubtestResult

    @property
    def id(self):
        return self.url


class ManualTest(Test):
    @property
    def id(self):
        return self.url


class ReftestTest(Test):
    result_cls = ReftestResult

    def __init__(self, url, expected, references, timeout=DEFAULT_TIMEOUT, path=None, protocol="http"):
        Test.__init__(self, url, expected, timeout, path, protocol)

        for _, ref_type in references:
            if ref_type not in ("==", "!="):
                raise ValueError

        self.references = references

    @classmethod
    def from_manifest(cls,
                      manifest_test,
                      expected_metadata,
                      nodes=None,
                      references_seen=None):

        timeout = LONG_TIMEOUT if manifest_test.timeout == "long" else DEFAULT_TIMEOUT

        if nodes is None:
            nodes = {}
        if references_seen is None:
            references_seen = set()

        url = manifest_test.url

        node = cls(manifest_test.url,
                   expected_metadata,
                   [],
                   timeout=timeout,
                   path=manifest_test.path,
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
                                                      None,
                                                      nodes,
                                                      references_seen)
            else:
                reference = ReftestTest(ref_url, None, [])

            node.references.append((reference, ref_type))

        return node

    @property
    def id(self):
        return self.url

    @property
    def keys(self):
        return ("reftype", "refurl")


manifest_test_cls = {"reftest": ReftestTest,
                     "testharness": TestharnessTest,
                     "manual": ManualTest}


def from_manifest(manifest_test, expected_metadata):
    test_cls = manifest_test_cls[manifest_test.item_type]

    return test_cls.from_manifest(manifest_test, expected_metadata)
