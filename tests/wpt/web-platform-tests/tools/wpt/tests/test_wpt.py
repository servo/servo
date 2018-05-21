import errno
import os
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import urllib2

import pytest

from tools.wpt import wpt


here = os.path.abspath(os.path.dirname(__file__))


def is_port_8000_in_use():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        s.bind(("127.0.0.1", 8000))
    except socket.error as e:
        if e.errno == errno.EADDRINUSE:
            return True
        else:
            raise e
    finally:
        s.close()
    return False


@pytest.fixture(scope="module")
def manifest_dir():
    def update_manifest():
        with pytest.raises(SystemExit) as excinfo:
            wpt.main(argv=["manifest", "--no-download", "--path", os.path.join(path, "MANIFEST.json")])
        assert excinfo.value.code == 0

    if os.environ.get('TRAVIS') == "true":
        path = "~/meta"
        update_manifest()
        yield path
    else:
        try:
            path = tempfile.mkdtemp()
            old_path = os.path.join(wpt.localpaths.repo_root, "MANIFEST.json")
            if os.path.exists(os.path.join(wpt.localpaths.repo_root, "MANIFEST.json")):
                shutil.copyfile(old_path, os.path.join(path, "MANIFEST.json"))
            update_manifest()
            yield path
        finally:
            shutil.rmtree(path)


def test_missing():
    with pytest.raises(SystemExit):
        wpt.main(argv=["#missing-command"])


def test_help():
    # TODO: It seems like there's a bug in argparse that makes this argument order required
    # should try to work around that
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["--help"])
    assert excinfo.value.code == 0


@pytest.mark.slow
def test_list_tests(manifest_dir):
    """The `--list-tests` option should not produce an error under normal
    conditions."""

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run", "--metadata", manifest_dir, "--list-tests",
                       "--yes", "chrome", "/dom/nodes/Element-tagName.html"])
    assert excinfo.value.code == 0


@pytest.mark.slow
def test_list_tests_missing_manifest(manifest_dir):
    """The `--list-tests` option should not produce an error in the absence of
    a test manifest file."""

    os.remove(os.path.join(manifest_dir, "MANIFEST.json"))

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run",
                       # This test triggers the creation of a new manifest
                       # file which is not necessary to ensure successful
                       # process completion. Specifying the current directory
                       # as the tests source via the --tests` option
                       # drastically reduces the time to execute the test.
                       "--tests", here,
                       "--metadata", manifest_dir,
                       "--list-tests",
                       "--yes",
                       "firefox", "/dom/nodes/Element-tagName.html"])

    assert excinfo.value.code == 0


@pytest.mark.slow
def test_list_tests_invalid_manifest(manifest_dir):
    """The `--list-tests` option should not produce an error in the presence of
    a malformed test manifest file."""

    manifest_filename = os.path.join(manifest_dir, "MANIFEST.json")

    assert os.path.isfile(manifest_filename)

    with open(manifest_filename, "a+") as handle:
        handle.write("extra text which invalidates the file")

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run",
                       # This test triggers the creation of a new manifest
                       # file which is not necessary to ensure successful
                       # process completion. Specifying the current directory
                       # as the tests source via the --tests` option
                       # drastically reduces the time to execute the test.
                       "--tests", here,
                       "--metadata", manifest_dir,
                       "--list-tests",
                       "--yes",
                       "firefox", "/dom/nodes/Element-tagName.html"])

    assert excinfo.value.code == 0


@pytest.mark.slow
@pytest.mark.remote_network
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_run_firefox(manifest_dir):
    # TODO: It seems like there's a bug in argparse that makes this argument order required
    # should try to work around that
    if is_port_8000_in_use():
        pytest.skip("port 8000 already in use")

    os.environ["MOZ_HEADLESS"] = "1"
    try:
        if sys.platform == "darwin":
            fx_path = os.path.join(wpt.localpaths.repo_root, "_venv", "browsers", "Firefox Nightly.app")
        else:
            fx_path = os.path.join(wpt.localpaths.repo_root, "_venv", "browsers", "firefox")
        if os.path.exists(fx_path):
            shutil.rmtree(fx_path)
        with pytest.raises(SystemExit) as excinfo:
            wpt.main(argv=["run", "--no-pause", "--install-browser", "--yes",
                           "--metadata", manifest_dir,
                           "firefox", "/dom/nodes/Element-tagName.html"])
        assert os.path.exists(fx_path)
        shutil.rmtree(fx_path)
        assert excinfo.value.code == 0
    finally:
        del os.environ["MOZ_HEADLESS"]


@pytest.mark.slow
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_run_chrome(manifest_dir):
    if is_port_8000_in_use():
        pytest.skip("port 8000 already in use")

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run", "--yes", "--no-pause", "--binary-arg", "headless",
                       "--metadata", manifest_dir,
                       "chrome", "/dom/nodes/Element-tagName.html"])
    assert excinfo.value.code == 0


@pytest.mark.slow
@pytest.mark.remote_network
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_run_zero_tests():
    """A test execution describing zero tests should be reported as an error
    even in the presence of the `--no-fail-on-unexpected` option."""
    if is_port_8000_in_use():
        pytest.skip("port 8000 already in use")

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run", "--yes", "--no-pause", "--binary-arg", "headless",
                       "chrome", "/non-existent-dir/non-existent-file.html"])
    assert excinfo.value.code != 0

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run", "--yes", "--no-pause", "--binary-arg", "headless",
                       "--no-fail-on-unexpected",
                       "chrome", "/non-existent-dir/non-existent-file.html"])
    assert excinfo.value.code != 0

@pytest.mark.slow
@pytest.mark.remote_network
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_run_failing_test():
    """Failing tests should be reported with a non-zero exit status unless the
    `--no-fail-on-unexpected` option has been specified."""
    if is_port_8000_in_use():
        pytest.skip("port 8000 already in use")
    failing_test = "/infrastructure/expected-fail/failing-test.html"

    assert os.path.isfile("../../%s" % failing_test)

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run", "--yes", "--no-pause", "--binary-arg", "headless",
                       "chrome", failing_test])
    assert excinfo.value.code != 0

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run", "--yes", "--no-pause", "--binary-arg", "headless",
                       "--no-fail-on-unexpected",
                       "chrome", failing_test])
    assert excinfo.value.code == 0


@pytest.mark.slow
@pytest.mark.remote_network
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_install_chromedriver():
    chromedriver_path = os.path.join(wpt.localpaths.repo_root, "_venv", "bin", "chromedriver")
    if os.path.exists(chromedriver_path):
        os.unlink(chromedriver_path)
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "chrome", "webdriver"])
    assert excinfo.value.code == 0
    assert os.path.exists(chromedriver_path)
    os.unlink(chromedriver_path)


@pytest.mark.slow
@pytest.mark.remote_network
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_install_firefox():

    if sys.platform == "darwin":
        fx_path = os.path.join(wpt.localpaths.repo_root, "_venv", "browsers", "Firefox Nightly.app")
    else:
        fx_path = os.path.join(wpt.localpaths.repo_root, "_venv", "browsers", "firefox")
    if os.path.exists(fx_path):
        shutil.rmtree(fx_path)
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "firefox", "browser"])
    assert excinfo.value.code == 0
    assert os.path.exists(fx_path)
    shutil.rmtree(fx_path)


@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_files_changed(capsys):
    commit = "9047ac1d9f51b1e9faa4f9fad9c47d109609ab09"
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["files-changed", "%s~..%s" % (commit, commit)])
    assert excinfo.value.code == 0
    out, err = capsys.readouterr()
    assert out == """html/browsers/offline/appcache/workers/appcache-worker.html
html/browsers/offline/appcache/workers/resources/appcache-dedicated-worker-not-in-cache.js
html/browsers/offline/appcache/workers/resources/appcache-shared-worker-not-in-cache.js
html/browsers/offline/appcache/workers/resources/appcache-worker-data.py
html/browsers/offline/appcache/workers/resources/appcache-worker-import.py
html/browsers/offline/appcache/workers/resources/appcache-worker.manifest
html/browsers/offline/appcache/workers/resources/appcache-worker.py
"""
    assert err == ""


def test_files_changed_ignore():
    from tools.wpt.testfiles import exclude_ignored
    files = ["resources/testharness.js", "resources/webidl2/index.js", "test/test.js"]
    changed, ignored = exclude_ignored(files, ignore_rules=["resources/testharness*"])
    assert changed == [os.path.join(wpt.wpt_root, item) for item in
                       ["resources/webidl2/index.js", "test/test.js"]]
    assert ignored == [os.path.join(wpt.wpt_root, item) for item in
                       ["resources/testharness.js"]]


def test_files_changed_ignore_rules():
    from tools.wpt.testfiles import compile_ignore_rule
    assert compile_ignore_rule("foo*bar*/baz").pattern == "^foo\*bar[^/]*/baz$"
    assert compile_ignore_rule("foo**bar**/baz").pattern == "^foo\*\*bar.*/baz$"
    assert compile_ignore_rule("foobar/baz/*").pattern == "^foobar/baz/[^/]*$"
    assert compile_ignore_rule("foobar/baz/**").pattern == "^foobar/baz/.*$"


@pytest.mark.slow  # this updates the manifest
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_tests_affected(capsys, manifest_dir):
    # This doesn't really work properly for random commits because we test the files in
    # the current working directory for references to the changed files, not the ones at
    # that specific commit. But we can at least test it returns something sensible.
    # The test will fail if the file we assert is renamed, so we choose a stable one.
    commit = "3a055e818218f548db240c316654f3cc1aeeb733"
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["tests-affected", "--metadata", manifest_dir, "%s~..%s" % (commit, commit)])
    assert excinfo.value.code == 0
    out, err = capsys.readouterr()
    assert "infrastructure/reftest-wait.html" in out


@pytest.mark.slow
@pytest.mark.xfail(sys.platform == "win32",
                   reason="Tests currently don't work on Windows for path reasons")
def test_serve():
    if is_port_8000_in_use():
        pytest.skip("port 8000 already in use")

    p = subprocess.Popen([os.path.join(wpt.localpaths.repo_root, "wpt"), "serve"],
                         preexec_fn=os.setsid)

    start = time.time()
    try:
        while True:
            if p.poll() is not None:
                assert False, "server not running"
            if time.time() - start > 60:
                assert False, "server did not start responding within 60s"
            try:
                resp = urllib2.urlopen("http://web-platform.test:8000")
                print resp
            except urllib2.URLError:
                print "URLError"
                time.sleep(1)
            else:
                assert resp.code == 200
                break
    finally:
        os.killpg(p.pid, 15)

# The following commands are slow running and used implicitly in other CI
# jobs, so we skip them here:
# wpt check-stability
# wpt manifest
# wpt lint
