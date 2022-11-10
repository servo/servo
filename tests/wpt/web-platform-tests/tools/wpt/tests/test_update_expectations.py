# mypy: ignore-errors

import json
import os

import pytest

from tools.wpt import wpt
from tools.wptrunner.wptrunner import manifestexpected
from localpaths import repo_root

@pytest.fixture
def metadata_file(tmp_path):
    created_files = []

    def create_metadata(test_id, subtest_name, product, status="OK", subtest_status="PASS", channel="nightly"):
        run_info = {
            "os": "linux",
            "processor": "x86_64",
            "version": "Ubuntu 20.04",
            "os_version": "20.04",
            "bits": 64,
            "linux_distro": "Ubuntu",
            "product": product,
            "debug": False,
            "browser_version": "98.0.2",
            "browser_channel": channel,
            "verify": False,
            "headless": True,
        }

        result = {
            "test": test_id,
            "subtests": [
                {
                    "name": subtest_name,
                    "status": subtest_status,
                    "message": None,
                    "known_intermittent": []
                }
            ],
            "status": status,
            "message": None,
            "duration": 555,
            "known_intermittent": []
        }

        if status != "OK":
            result["expected"] = "OK"

        if subtest_status != "PASS":
            result["subtests"][0]["expected"] = "PASS"

        data = {
            "time_start": 1648629686379,
            "run_info": run_info,
            "results": [result],
            "time_end": 1648629698721
        }

        path = os.path.join(tmp_path, f"wptreport-{len(created_files)}.json")
        with open(path, "w") as f:
            json.dump(data, f)

        created_files.append(path)
        return run_info, path

    yield create_metadata

    for path in created_files:
        os.unlink(path)


def test_update(tmp_path, metadata_file):
    # This has to be a real test so it's in the manifest
    test_id = "/infrastructure/assumptions/cookie.html"
    subtest_name = "cookies work in default browse settings"
    test_path = os.path.join("infrastructure",
                             "assumptions",
                             "cookie.html")
    run_info_firefox, path_firefox = metadata_file(test_id,
                                                   subtest_name,
                                                   "firefox",
                                                   subtest_status="FAIL",
                                                   channel="nightly")
    run_info_chrome, path_chrome = metadata_file(test_id,
                                                 subtest_name,
                                                 "chrome",
                                                 status="ERROR",
                                                 subtest_status="NOTRUN",
                                                 channel="dev")

    metadata_path = str(os.path.join(tmp_path, "metadata"))
    os.makedirs(metadata_path)
    wptreport_paths = [path_firefox, path_chrome]

    update_properties = {"properties": ["product"]}
    with open(os.path.join(metadata_path, "update_properties.json"), "w") as f:
        json.dump(update_properties, f)

    args = ["update-expectations",
            "--manifest", os.path.join(repo_root, "MANIFEST.json"),
            "--metadata", metadata_path,
            "--log-mach-level", "debug"]
    args += wptreport_paths

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=args)

    assert excinfo.value.code == 0

    expectation_path = os.path.join(metadata_path, test_path + ".ini")

    assert os.path.exists(expectation_path)

    firefox_expected = manifestexpected.get_manifest(metadata_path,
                                                     test_path,
                                                     "/",
                                                     run_info_firefox)
    # Default expected isn't stored
    with pytest.raises(KeyError):
        assert firefox_expected.get_test(test_id).get("expected")
    assert firefox_expected.get_test(test_id).get_subtest(subtest_name).expected == "FAIL"

    chrome_expected = manifestexpected.get_manifest(metadata_path,
                                                    test_path,
                                                    "/",
                                                    run_info_chrome)
    assert chrome_expected.get_test(test_id).expected == "ERROR"
    assert chrome_expected.get_test(test_id).get_subtest(subtest_name).expected == "NOTRUN"
