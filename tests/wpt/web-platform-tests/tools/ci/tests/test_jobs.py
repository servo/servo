import sys

import pytest

from tools.ci import jobs

default_jobs = set(["lint", "manifest_upload"])


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_testharness():
    assert jobs.get_jobs(["resources/testharness.js"]) == default_jobs | set(["resources_unittest"])
    assert jobs.get_jobs(["resources/testharness.js"],
                         includes=["resources_unittest"]) == set(["resources_unittest"])
    assert jobs.get_jobs(["tools/wptserve/wptserve/config.py"],
                         includes=["resources_unittest"]) == set(["resources_unittest"])
    assert jobs.get_jobs(["foo/resources/testharness.js"],
                         includes=["resources_unittest"]) == set()


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_stability():
    assert jobs.get_jobs(["dom/historical.html"],
                         includes=["stability"]) == set(["stability"])
    assert jobs.get_jobs(["tools/pytest.ini"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["serve"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["resources/testharness.js"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["docs/.gitignore"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["dom/tools/example.py"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["conformance-checkers/test.html"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["dom/README.md"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["css/build-css-testsuite.sh"],
                         includes=["stability"]) == set()
    assert jobs.get_jobs(["css/CSS21/test-001.html"],
                         includes=["stability"]) == set(["stability"])
    assert jobs.get_jobs(["css/build-css-testsuite.sh",
                          "css/CSS21/test-001.html"],
                         includes=["stability"]) == set(["stability"])


def test_default():
    assert jobs.get_jobs(["README.md"]) == default_jobs


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_tools_unittest():
    assert jobs.get_jobs(["tools/ci/test/test_jobs.py"],
                         includes=["tools_unittest"]) == set(["tools_unittest"])
    assert jobs.get_jobs(["dom/tools/example.py"],
                         includes=["tools_unittest"]) == set()
    assert jobs.get_jobs(["dom/historical.html"],
                         includes=["tools_unittest"]) == set()


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_wptrunner_unittest():
    assert jobs.get_jobs(["tools/wptrunner/wptrunner/wptrunner.py"],
                         includes=["wptrunner_unittest"]) == set(["wptrunner_unittest"])
    assert jobs.get_jobs(["tools/example.py"],
                         includes=["wptrunner_unittest"]) == set()


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_build_css():
    assert jobs.get_jobs(["css/css-build-testsuites.sh"],
                         includes=["build_css"]) == set(["build_css"])
    assert jobs.get_jobs(["css/CSS21/test.html"],
                         includes=["build_css"]) == set(["build_css"])
    assert jobs.get_jobs(["html/css/CSS21/test.html"],
                         includes=["build_css"]) == set()


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_update_built():
    assert jobs.get_jobs(["2dcontext/foo.html"],
                         includes=["update_built"]) == set(["update_built"])
    assert jobs.get_jobs(["html/foo.html"],
                         includes=["update_built"]) == set(["update_built"])
    assert jobs.get_jobs(["offscreen-canvas/foo.html"],
                         includes=["update_built"]) == set(["update_built"])


@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_wpt_integration():
    assert jobs.get_jobs(["tools/wpt/wpt.py"],
                         includes=["wpt_integration"]) == set(["wpt_integration"])
    assert jobs.get_jobs(["tools/wptrunner/wptrunner/wptrunner.py"],
                         includes=["wpt_integration"]) == set(["wpt_integration"])

@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/12949")
def test_wpt_infrastructure():
    assert jobs.get_jobs(["tools/hammer.html"],
                         includes=["wptrunner_infrastructure"]) == set(["wptrunner_infrastructure"])
    assert jobs.get_jobs(["infrastructure/assumptions/ahem.html"],
                         includes=["wptrunner_infrastructure"]) == set(["wptrunner_infrastructure"])
