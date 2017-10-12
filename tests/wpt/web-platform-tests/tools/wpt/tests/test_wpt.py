import os
import shutil
import socket
import subprocess
import time
import urllib2

import pytest

from tools.wpt import wpt


# Tests currently don't work on Windows for path reasons

def test_missing():
    with pytest.raises(SystemExit):
        wpt.main(argv=["#missing-command"])


def test_help():
    # TODO: It seems like there's a bug in argparse that makes this argument order required
    # should try to work around that
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["--help"])
    assert excinfo.value.code == 0


def test_run_firefox():
    # TODO: It seems like there's a bug in argparse that makes this argument order required
    # should try to work around that
    os.environ["MOZ_HEADLESS"] = "1"
    try:
        fx_path = os.path.join(wpt.localpaths.repo_root, "_venv", "firefox")
        if os.path.exists(fx_path):
            shutil.rmtree(fx_path)
        with pytest.raises(SystemExit) as excinfo:
            wpt.main(argv=["run", "--no-pause", "--install-browser", "--yes",
                           "--metadata", "~/meta/",
                           "firefox", "/dom/nodes/Element-tagName.html"])
        assert os.path.exists(fx_path)
        shutil.rmtree(fx_path)
        assert excinfo.value.code == 0
    finally:
        del os.environ["MOZ_HEADLESS"]


def test_run_chrome():
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["run", "--yes", "--no-pause", "--binary-arg", "headless",
                       "--metadata", "~/meta/",
                       "chrome", "/dom/nodes/Element-tagName.html"])
    assert excinfo.value.code == 0


def test_install_chromedriver():
    chromedriver_path = os.path.join(wpt.localpaths.repo_root, "_venv", "bin", "chromedriver")
    if os.path.exists(chromedriver_path):
        os.unlink(chromedriver_path)
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "chrome", "webdriver"])
    assert excinfo.value.code == 0
    assert os.path.exists(chromedriver_path)
    os.unlink(chromedriver_path)


def test_install_firefox():
    fx_path = os.path.join(wpt.localpaths.repo_root, "_venv", "firefox")
    if os.path.exists(fx_path):
        shutil.rmtree(fx_path)
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "firefox", "browser"])
    assert excinfo.value.code == 0
    assert os.path.exists(fx_path)
    shutil.rmtree(fx_path)


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


def test_tests_affected(capsys):
    # This doesn't really work properly for random commits because we test the files in
    # the current working directory for references to the changed files, not the ones at
    # that specific commit. But we can at least test it returns something sensible
    commit = "9047ac1d9f51b1e9faa4f9fad9c47d109609ab09"
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["tests-affected", "--metadata", "~/meta/", "%s~..%s" % (commit, commit)])
    assert excinfo.value.code == 0
    out, err = capsys.readouterr()
    assert "html/browsers/offline/appcache/workers/appcache-worker.html" in out
    assert err == ""


def test_serve():
    def test():
        s = socket.socket()
        s.connect(("127.0.0.1", 8000))
    with pytest.raises(socket.error):
        test()

    p = subprocess.Popen([os.path.join(wpt.localpaths.repo_root, "wpt"), "serve"],
                         preexec_fn=os.setsid)

    start = time.time()
    try:
        while True:
            if time.time() - start > 60:
                assert False
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
