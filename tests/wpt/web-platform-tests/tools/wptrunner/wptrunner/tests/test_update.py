import json
import mock
import os
import pytest
import sys
from io import BytesIO

from .. import metadata, manifestupdate
from ..update import WPTUpdate
from ..update.base import StepRunner, Step
from mozlog import structuredlog, handlers, formatters

sys.path.insert(0, os.path.join(os.path.dirname(__file__), os.pardir, os.pardir, os.pardir))
from manifest import manifest, item as manifest_item


def rel_path_to_test_url(rel_path):
    assert not os.path.isabs(rel_path)
    return rel_path.replace(os.sep, "/")


def SourceFileWithTest(path, hash, cls, *args):
    s = mock.Mock(rel_path=path, hash=hash)
    test = cls("/foobar", path, "/", rel_path_to_test_url(path), *args)
    s.manifest_items = mock.Mock(return_value=(cls.item_type, [test]))
    return s


item_classes = {"testharness": manifest_item.TestharnessTest,
                "reftest": manifest_item.RefTest,
                "reftest_node": manifest_item.RefTestNode,
                "manual": manifest_item.ManualTest,
                "stub": manifest_item.Stub,
                "wdspec": manifest_item.WebDriverSpecTest,
                "conformancechecker": manifest_item.ConformanceCheckerTest,
                "visual": manifest_item.VisualTest,
                "support": manifest_item.SupportFile}


def update(tests, *logs):
    id_test_map, updater = create_updater(tests)
    for log in logs:
        log = create_log(log)
        updater.update_from_log(log)

    return list(metadata.update_results(id_test_map,
                                        ["debug", "os", "version", "processor", "bits"],
                                        ["debug"],
                                        False))


def create_updater(tests, url_base="/", **kwargs):
    id_test_map = {}
    m = create_test_manifest(tests, url_base)
    expected_data = {}
    metadata.load_expected = lambda _, __, test_path, *args: expected_data[test_path]

    id_test_map = metadata.create_test_tree(None, m)

    for test_path, test_ids, test_type, manifest_str in tests:
        expected_data[test_path] = manifestupdate.compile(BytesIO(manifest_str),
                                                          test_path,
                                                          url_base)

    return id_test_map, metadata.ExpectedUpdater(id_test_map, **kwargs)


def create_log(entries):
    data = BytesIO()
    if isinstance(entries, list):
        logger = structuredlog.StructuredLogger("expected_test")
        handler = handlers.StreamHandler(data, formatters.JSONFormatter())
        logger.add_handler(handler)

        for item in entries:
            action, kwargs = item
            getattr(logger, action)(**kwargs)
        logger.remove_handler(handler)
    else:
        json.dump(entries, data)
    data.seek(0)
    return data


def suite_log(entries, run_info=None):
    return ([("suite_start", {"tests": [], "run_info": run_info or {}})] +
            entries +
            [("suite_end", {})])


def create_test_manifest(tests, url_base="/"):
    source_files = []
    for i, (test, _, test_type, _) in enumerate(tests):
        if test_type:
            source_files.append((SourceFileWithTest(test, str(i) * 40, item_classes[test_type]), True))
    m = manifest.Manifest()
    m.update(source_files)
    return m


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_0():
    tests = [("path/to/test.htm", ["/path/to/test.htm"], "testharness",
              """[test.htm]
  [test1]
    expected: FAIL""")]

    log = suite_log([("test_start", {"test": "/path/to/test.htm"}),
                     ("test_status", {"test": "/path/to/test.htm",
                                      "subtest": "test1",
                                      "status": "PASS",
                                      "expected": "FAIL"}),
                     ("test_end", {"test": "/path/to/test.htm",
                                   "status": "OK"})])

    updated = update(tests, log)

    assert len(updated) == 1
    assert updated[0][1].is_empty


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_1():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness",
              """[test.htm]
  [test1]
    expected: ERROR""")]

    log = suite_log([("test_start", {"test": test_id}),
                     ("test_status", {"test": test_id,
                                      "subtest": "test1",
                                      "status": "FAIL",
                                      "expected": "ERROR"}),
                     ("test_end", {"test": test_id,
                                   "status": "OK"})])

    updated = update(tests, log)

    new_manifest = updated[0][1]
    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).children[0].get("expected") == "FAIL"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_skip_0():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness",
              """[test.htm]
  [test1]
    expected: FAIL""")]

    log = suite_log([("test_start", {"test": test_id}),
                     ("test_status", {"test": test_id,
                                      "subtest": "test1",
                                      "status": "FAIL",
                                      "expected": "FAIL"}),
                     ("test_end", {"test": test_id,
                                   "status": "OK"})])

    updated = update(tests, log)
    assert not updated


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_new_subtest():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  [test1]
    expected: FAIL""")]

    log = suite_log([("test_start", {"test": test_id}),
                     ("test_status", {"test": test_id,
                                      "subtest": "test1",
                                      "status": "FAIL",
                                      "expected": "FAIL"}),
                     ("test_status", {"test": test_id,
                                      "subtest": "test2",
                                      "status": "FAIL",
                                      "expected": "PASS"}),
                     ("test_end", {"test": test_id,
                                   "status": "OK"})])
    updated = update(tests, log)
    new_manifest = updated[0][1]
    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).children[0].get("expected") == "FAIL"
    assert new_manifest.get_test(test_id).children[1].get("expected") == "FAIL"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_multiple_0():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  [test1]
    expected: FAIL""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "FAIL",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": False, "os": "osx"})

    log_1 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "TIMEOUT",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": False, "os": "linux"})

    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "osx"}) == "FAIL"
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "linux"}) == "TIMEOUT"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_multiple_1():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  [test1]
    expected: FAIL""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "FAIL",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": False, "os": "osx"})

    log_1 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "TIMEOUT",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": False, "os": "linux"})

    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "osx"}) == "FAIL"
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "linux"}) == "TIMEOUT"
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "windows"}) == "FAIL"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_multiple_2():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  [test1]
    expected: FAIL""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "FAIL",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": False, "os": "osx"})

    log_1 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "TIMEOUT",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": True, "os": "osx"})

    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "osx"}) == "FAIL"
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": True, "os": "osx"}) == "TIMEOUT"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_multiple_3():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  [test1]
    expected:
      if debug: FAIL
      if not debug and os == "osx": TIMEOUT""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "FAIL",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": False, "os": "osx"})

    log_1 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "TIMEOUT",
                                        "expected": "FAIL"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": True, "os": "osx"})

    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "osx"}) == "FAIL"
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": True, "os": "osx"}) == "TIMEOUT"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_ignore_existing():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  [test1]
    expected:
      if debug: TIMEOUT
      if not debug and os == "osx": NOTRUN""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "FAIL",
                                        "expected": "PASS"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": False, "os": "linux"})

    log_1 = suite_log([("test_start", {"test": test_id}),
                       ("test_status", {"test": test_id,
                                        "subtest": "test1",
                                        "status": "FAIL",
                                        "expected": "PASS"}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"debug": True, "os": "windows"})

    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": True, "os": "osx"}) == "FAIL"
    assert new_manifest.get_test(test_id).children[0].get(
        "expected", {"debug": False, "os": "osx"}) == "NOTRUN"


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_assertion_count_0():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  max-asserts: 4
  min-asserts: 2
""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("assertion_count", {"test": test_id,
                                            "count": 6,
                                            "min_expected": 2,
                                            "max_expected": 4}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})])

    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).get("max-asserts") == 7
    assert new_manifest.get_test(test_id).get("min-asserts") == 2


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_assertion_count_1():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  max-asserts: 4
  min-asserts: 2
""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("assertion_count", {"test": test_id,
                                            "count": 1,
                                            "min_expected": 2,
                                            "max_expected": 4}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})])

    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).get("max-asserts") == 4
    assert new_manifest.get_test(test_id).has_key("min-asserts") is False


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_assertion_count_2():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  max-asserts: 4
  min-asserts: 2
""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("assertion_count", {"test": test_id,
                                            "count": 3,
                                            "min_expected": 2,
                                            "max_expected": 4}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})])

    updated = update(tests, log_0)
    assert not updated


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_assertion_count_3():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]
  max-asserts: 4
  min-asserts: 2
""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("assertion_count", {"test": test_id,
                                            "count": 6,
                                            "min_expected": 2,
                                            "max_expected": 4}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"os": "windows"})

    log_1 = suite_log([("test_start", {"test": test_id}),
                       ("assertion_count", {"test": test_id,
                                            "count": 7,
                                            "min_expected": 2,
                                            "max_expected": 4}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"os": "linux"})

    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).get("max-asserts") == 8
    assert new_manifest.get_test(test_id).get("min-asserts") == 2


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_assertion_count_4():
    test_id = "/path/to/test.htm"
    tests = [("path/to/test.htm", [test_id], "testharness", """[test.htm]""")]

    log_0 = suite_log([("test_start", {"test": test_id}),
                       ("assertion_count", {"test": test_id,
                                            "count": 6,
                                            "min_expected": 0,
                                            "max_expected": 0}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"os": "windows"})

    log_1 = suite_log([("test_start", {"test": test_id}),
                       ("assertion_count", {"test": test_id,
                                            "count": 7,
                                            "min_expected": 0,
                                            "max_expected": 0}),
                       ("test_end", {"test": test_id,
                                     "status": "OK"})],
                      run_info={"os": "linux"})

    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get_test(test_id).get("max-asserts") == "8"
    assert new_manifest.get_test(test_id).has_key("min-asserts") is False


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_lsan_0():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, "")]

    log_0 = suite_log([("lsan_leak", {"scope": "path/to/",
                                      "frames": ["foo", "bar"]})])


    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get("lsan-allowed") == ["foo"]


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_lsan_1():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, """
lsan-allowed: [foo]""")]

    log_0 = suite_log([("lsan_leak", {"scope": "path/to/",
                                      "frames": ["foo", "bar"]}),
                       ("lsan_leak", {"scope": "path/to/",
                                      "frames": ["baz", "foobar"]})])


    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get("lsan-allowed") == ["baz", "foo"]


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_lsan_2():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/__dir__", ["path/__dir__"], None, """
lsan-allowed: [foo]"""),
             ("path/to/__dir__", [dir_id], None, "")]

    log_0 = suite_log([("lsan_leak", {"scope": "path/to/",
                                      "frames": ["foo", "bar"],
                                      "allowed_match": ["foo"]}),
                       ("lsan_leak", {"scope": "path/to/",
                                      "frames": ["baz", "foobar"]})])


    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get("lsan-allowed") == ["baz"]


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_lsan_3():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, "")]

    log_0 = suite_log([("lsan_leak", {"scope": "path/to/",
                                      "frames": ["foo", "bar"]})],
                      run_info={"os": "win"})

    log_1 = suite_log([("lsan_leak", {"scope": "path/to/",
                                      "frames": ["baz", "foobar"]})],
                      run_info={"os": "linux"})


    updated = update(tests, log_0, log_1)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get("lsan-allowed") == ["baz", "foo"]


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_wptreport_0():
    tests = [("path/to/test.htm", ["/path/to/test.htm"], "testharness",
              """[test.htm]
  [test1]
    expected: FAIL""")]

    log = {"run_info": {},
           "results": [
               {"test": "/path/to/test.htm",
                "subtests": [{"name": "test1",
                              "status": "PASS",
                              "expected": "FAIL"}],
                "status": "OK"}]}

    updated = update(tests, log)

    assert len(updated) == 1
    assert updated[0][1].is_empty


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_wptreport_1():
    tests = [("path/to/test.htm", ["/path/to/test.htm"], "testharness", ""),
             ("path/to/__dir__", ["path/to/__dir__"], None, "")]

    log = {"run_info": {},
           "results": [],
           "lsan_leaks": [{"scope": "path/to/",
                           "frames": ["baz", "foobar"]}]}

    updated = update(tests, log)

    assert len(updated) == 1
    assert updated[0][1].get("lsan-allowed") == ["baz"]


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_leak_total_0():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, "")]

    log_0 = suite_log([("mozleak_total", {"scope": "path/to/",
                                          "process": "default",
                                          "bytes": 100,
                                          "threshold": 0,
                                          "objects": []})])

    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get("leak-threshold") == ['default:51200']


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_leak_total_1():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, "")]

    log_0 = suite_log([("mozleak_total", {"scope": "path/to/",
                                          "process": "default",
                                          "bytes": 100,
                                          "threshold": 1000,
                                          "objects": []})])

    updated = update(tests, log_0)
    assert not updated


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_leak_total_2():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, """
leak-total: 110""")]

    log_0 = suite_log([("mozleak_total", {"scope": "path/to/",
                                          "process": "default",
                                          "bytes": 100,
                                          "threshold": 110,
                                          "objects": []})])

    updated = update(tests, log_0)
    assert not updated


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_leak_total_3():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, """
leak-total: 100""")]

    log_0 = suite_log([("mozleak_total", {"scope": "path/to/",
                                          "process": "default",
                                          "bytes": 1000,
                                          "threshold": 100,
                                          "objects": []})])

    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.get("leak-threshold") == ['default:51200']


@pytest.mark.xfail(sys.version[0] == "3",
                   reason="metadata doesn't support py3")
def test_update_leak_total_4():
    test_id = "/path/to/test.htm"
    dir_id = "path/to/__dir__"
    tests = [("path/to/test.htm", [test_id], "testharness", ""),
             ("path/to/__dir__", [dir_id], None, """
leak-total: 110""")]

    log_0 = suite_log([
        ("lsan_leak", {"scope": "path/to/",
                       "frames": ["foo", "bar"]}),
        ("mozleak_total", {"scope": "path/to/",
                           "process": "default",
                           "bytes": 100,
                           "threshold": 110,
                           "objects": []})])

    updated = update(tests, log_0)
    new_manifest = updated[0][1]

    assert not new_manifest.is_empty
    assert new_manifest.has_key("leak-threshold") is False


class TestStep(Step):
    def create(self, state):
        test_id = "/path/to/test.htm"
        tests = [("path/to/test.htm", [test_id], "testharness", "")]
        state.foo = create_test_manifest(tests)

class UpdateRunner(StepRunner):
    steps = [TestStep]

@pytest.mark.xfail(sys.version[0] == "3",
                   reason="update.state doesn't support py3")
def test_update_pickle():
    logger = structuredlog.StructuredLogger("expected_test")
    args = {
        "test_paths": {
            "/": {"tests_path": ""},
        },
        "abort": False,
        "continue": False,
        "sync": False,
    }
    args2 = args.copy()
    args2["abort"] = True
    wptupdate = WPTUpdate(logger, **args2)
    wptupdate = WPTUpdate(logger, runner_cls=UpdateRunner, **args)
    wptupdate.run()
