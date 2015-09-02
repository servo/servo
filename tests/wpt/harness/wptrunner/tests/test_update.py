# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import unittest
import StringIO

from .. import metadata, manifestupdate
from mozlog import structuredlog, handlers, formatters


class TestExpectedUpdater(unittest.TestCase):
    def create_manifest(self, data, test_path="path/to/test.ini"):
        f = StringIO.StringIO(data)
        return manifestupdate.compile(f, test_path)

    def create_updater(self, data, **kwargs):
        expected_tree = {}
        id_path_map = {}
        for test_path, test_ids, manifest_str in data:
            if isinstance(test_ids, (str, unicode)):
                test_ids = [test_ids]
            expected_tree[test_path] = self.create_manifest(manifest_str, test_path)
            for test_id in test_ids:
                id_path_map[test_id] = test_path

        return metadata.ExpectedUpdater(expected_tree, id_path_map, **kwargs)

    def create_log(self, *args, **kwargs):
        logger = structuredlog.StructuredLogger("expected_test")
        data = StringIO.StringIO()
        handler = handlers.StreamHandler(data, formatters.JSONFormatter())
        logger.add_handler(handler)

        log_entries = ([("suite_start", {"tests": [], "run_info": kwargs.get("run_info", {})})] +
                       list(args) +
                       [("suite_end", {})])

        for item in log_entries:
            action, kwargs = item
            getattr(logger, action)(**kwargs)
        logger.remove_handler(handler)
        data.seek(0)
        return data


    def coalesce_results(self, trees):
        for tree in trees:
            for test in tree.iterchildren():
                for subtest in test.iterchildren():
                    subtest.coalesce_expected()
                test.coalesce_expected()

    def test_update_0(self):
        prev_data = [("path/to/test.htm.ini", ["/path/to/test.htm"], """[test.htm]
  type: testharness
  [test1]
    expected: FAIL""")]

        new_data = self.create_log(("test_start", {"test": "/path/to/test.htm"}),
                                   ("test_status", {"test": "/path/to/test.htm",
                                                    "subtest": "test1",
                                                    "status": "PASS",
                                                    "expected": "FAIL"}),
                                   ("test_end", {"test": "/path/to/test.htm",
                                                 "status": "OK"}))
        updater = self.create_updater(prev_data)
        updater.update_from_log(new_data)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]
        self.coalesce_results([new_manifest])
        self.assertTrue(new_manifest.is_empty)

    def test_update_1(self):
        test_id = "/path/to/test.htm"
        prev_data = [("path/to/test.htm.ini", [test_id], """[test.htm]
  type: testharness
  [test1]
    expected: ERROR""")]

        new_data = self.create_log(("test_start", {"test": test_id}),
                                   ("test_status", {"test": test_id,
                                                    "subtest": "test1",
                                                    "status": "FAIL",
                                                    "expected": "ERROR"}),
                                   ("test_end", {"test": test_id,
                                                 "status": "OK"}))
        updater = self.create_updater(prev_data)
        updater.update_from_log(new_data)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]
        self.coalesce_results([new_manifest])
        self.assertFalse(new_manifest.is_empty)
        self.assertEquals(new_manifest.get_test(test_id).children[0].get("expected"), "FAIL")

    def test_new_subtest(self):
        test_id = "/path/to/test.htm"
        prev_data = [("path/to/test.htm.ini", [test_id], """[test.htm]
  type: testharness
  [test1]
    expected: FAIL""")]

        new_data = self.create_log(("test_start", {"test": test_id}),
                                   ("test_status", {"test": test_id,
                                                    "subtest": "test1",
                                                    "status": "FAIL",
                                                    "expected": "FAIL"}),
                                   ("test_status", {"test": test_id,
                                                    "subtest": "test2",
                                                    "status": "FAIL",
                                                    "expected": "PASS"}),
                                   ("test_end", {"test": test_id,
                                                 "status": "OK"}))
        updater = self.create_updater(prev_data)
        updater.update_from_log(new_data)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]
        self.coalesce_results([new_manifest])
        self.assertFalse(new_manifest.is_empty)
        self.assertEquals(new_manifest.get_test(test_id).children[0].get("expected"), "FAIL")
        self.assertEquals(new_manifest.get_test(test_id).children[1].get("expected"), "FAIL")

    def test_update_multiple_0(self):
        test_id = "/path/to/test.htm"
        prev_data = [("path/to/test.htm.ini", [test_id], """[test.htm]
  type: testharness
  [test1]
    expected: FAIL""")]

        new_data_0 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "FAIL",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": False, "os": "osx"})

        new_data_1 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "TIMEOUT",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": False, "os": "linux"})
        updater = self.create_updater(prev_data)

        updater.update_from_log(new_data_0)
        updater.update_from_log(new_data_1)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]

        self.coalesce_results([new_manifest])

        self.assertFalse(new_manifest.is_empty)
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "osx"}), "FAIL")
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "linux"}), "TIMEOUT")

    def test_update_multiple_1(self):
        test_id = "/path/to/test.htm"
        prev_data = [("path/to/test.htm.ini", [test_id], """[test.htm]
  type: testharness
  [test1]
    expected: FAIL""")]

        new_data_0 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "FAIL",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": False, "os": "osx"})

        new_data_1 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "TIMEOUT",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": False, "os": "linux"})
        updater = self.create_updater(prev_data)

        updater.update_from_log(new_data_0)
        updater.update_from_log(new_data_1)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]

        self.coalesce_results([new_manifest])

        self.assertFalse(new_manifest.is_empty)
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "osx"}), "FAIL")
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "linux"}), "TIMEOUT")
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "windows"}), "FAIL")

    def test_update_multiple_2(self):
        test_id = "/path/to/test.htm"
        prev_data = [("path/to/test.htm.ini", [test_id], """[test.htm]
  type: testharness
  [test1]
    expected: FAIL""")]

        new_data_0 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "FAIL",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": False, "os": "osx"})

        new_data_1 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "TIMEOUT",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": True, "os": "osx"})
        updater = self.create_updater(prev_data)

        updater.update_from_log(new_data_0)
        updater.update_from_log(new_data_1)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]

        self.coalesce_results([new_manifest])

        self.assertFalse(new_manifest.is_empty)
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "osx"}), "FAIL")
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": True, "os": "osx"}), "TIMEOUT")

    def test_update_multiple_3(self):
        test_id = "/path/to/test.htm"
        prev_data = [("path/to/test.htm.ini", [test_id], """[test.htm]
  type: testharness
  [test1]
    expected:
      if debug: FAIL
      if not debug and os == "osx": TIMEOUT""")]

        new_data_0 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "FAIL",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": False, "os": "osx"})

        new_data_1 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "TIMEOUT",
                                                      "expected": "FAIL"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": True, "os": "osx"})
        updater = self.create_updater(prev_data)

        updater.update_from_log(new_data_0)
        updater.update_from_log(new_data_1)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]

        self.coalesce_results([new_manifest])

        self.assertFalse(new_manifest.is_empty)
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "osx"}), "FAIL")
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": True, "os": "osx"}), "TIMEOUT")

    def test_update_ignore_existing(self):
        test_id = "/path/to/test.htm"
        prev_data = [("path/to/test.htm.ini", [test_id], """[test.htm]
  type: testharness
  [test1]
    expected:
      if debug: TIMEOUT
      if not debug and os == "osx": NOTRUN""")]

        new_data_0 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "FAIL",
                                                      "expected": "PASS"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": False, "os": "linux"})

        new_data_1 = self.create_log(("test_start", {"test": test_id}),
                                     ("test_status", {"test": test_id,
                                                      "subtest": "test1",
                                                      "status": "FAIL",
                                                      "expected": "PASS"}),
                                     ("test_end", {"test": test_id,
                                                   "status": "OK"}),
                                     run_info={"debug": True, "os": "windows"})
        updater = self.create_updater(prev_data, ignore_existing=True)

        updater.update_from_log(new_data_0)
        updater.update_from_log(new_data_1)

        new_manifest = updater.expected_tree["path/to/test.htm.ini"]

        self.coalesce_results([new_manifest])

        self.assertFalse(new_manifest.is_empty)
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": True, "os": "osx"}), "FAIL")
        self.assertEquals(new_manifest.get_test(test_id).children[0].get(
            "expected", {"debug": False, "os": "osx"}), "FAIL")
