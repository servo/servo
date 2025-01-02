# mypy: allow-untyped-defs

import logging
import os
import sys

import pytest

from tools.wpt import browser, utils, wpt


@pytest.mark.slow
@pytest.mark.remote_network
def test_install_chromium():
    venv_path = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir())
    channel = "nightly"
    dest = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir(), "browsers", channel)
    if sys.platform == "win32":
        chromium_path = os.path.join(dest, "chrome-win")
    elif sys.platform == "darwin":
        chromium_path = os.path.join(dest, "chrome-mac")
    else:
        chromium_path = os.path.join(dest, "chrome-linux")

    if os.path.exists(chromium_path):
        utils.rmtree(chromium_path)
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "chromium", "browser"])
    assert excinfo.value.code == 0
    assert os.path.exists(chromium_path)

    chromium = browser.Chromium(logging.getLogger("Chromium"))
    binary = chromium.find_binary(venv_path, channel)
    assert binary is not None and os.path.exists(binary)

    utils.rmtree(chromium_path)


@pytest.mark.slow
@pytest.mark.remote_network
def test_install_chrome():
    venv_path = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir())
    channel = "dev"
    dest = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir(), "browsers", channel)
    if sys.platform == "win32":
        chrome_path = os.path.join(dest, "chrome-win32")
    elif sys.platform == "darwin":
        chrome_path = os.path.join(dest, "chrome-mac-x64")
    else:
        chrome_path = os.path.join(dest, "chrome-linux64")

    if os.path.exists(chrome_path):
        utils.rmtree(chrome_path)
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "--channel", channel, "chrome", "browser"])
    assert excinfo.value.code == 0
    assert os.path.exists(chrome_path)

    chrome = browser.Chrome(logging.getLogger("Chrome"))
    binary = chrome.find_binary(venv_path, channel)
    assert binary is not None and os.path.exists(binary)

    utils.rmtree(chrome_path)


@pytest.mark.slow
@pytest.mark.remote_network
def test_install_chrome_chromedriver_by_version():
    # This is not technically an integration test as we do not want to require Chrome Stable to run it.
    chrome = browser.Chrome(logging.getLogger("Chrome"))
    if sys.platform == "win32":
        dest = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir(), "Scripts")
        chromedriver_path = os.path.join(dest, "chrome", "chromedriver.exe")
        # By default Windows treats paths as case-insensitive
        path_fn = lambda path: path.lower()
    else:
        dest = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir(), "bin")
        chromedriver_path = os.path.join(dest, "chrome", "chromedriver")
        path_fn = lambda path: path
    if os.path.exists(chromedriver_path):
        os.unlink(chromedriver_path)
    # This is a stable version.
    binary_path = chrome.install_webdriver_by_version(
        dest=dest, version="115.0.5790.170", channel="stable")
    assert path_fn(binary_path) == path_fn(chromedriver_path)
    assert os.path.exists(chromedriver_path)
    os.unlink(chromedriver_path)


@pytest.mark.slow
@pytest.mark.remote_network
@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/17074")
def test_install_firefox():
    if sys.platform == "darwin":
        fx_path = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir(), "browsers", "nightly", "Firefox Nightly.app")
    else:
        fx_path = os.path.join(wpt.localpaths.repo_root, wpt.venv_dir(), "browsers", "nightly", "firefox")
    if os.path.exists(fx_path):
        utils.rmtree(fx_path)
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "firefox", "browser", "--channel=nightly"])
    assert excinfo.value.code == 0
    assert os.path.exists(fx_path)
    utils.rmtree(fx_path)
