# mypy: allow-untyped-defs

import logging
import os
import platform
import sys

import pytest

from tools.wpt import browser, wpt


@pytest.mark.slow
@pytest.mark.remote_network
def test_install_chromium(tmp_path):
    channel = "nightly"
    if sys.platform == "win32":
        chromium_dir = "chrome-win"
    elif sys.platform == "darwin":
        chromium_dir = "chrome-mac"
    else:
        chromium_dir = "chrome-linux"

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "-d", str(tmp_path), "chromium", "browser"])
    assert excinfo.value.code == 0
    assert os.path.isdir(os.path.join(tmp_path, "browsers", channel, chromium_dir))

    chromium = browser.Chromium(logging.getLogger("Chromium"))
    binary = chromium.find_binary(str(tmp_path), channel)
    assert binary is not None and os.path.exists(binary)


@pytest.mark.slow
@pytest.mark.remote_network
def test_install_chrome(tmp_path):
    channel = "dev"
    uname = platform.uname()
    chrome_platform = {
        "Linux": "linux",
        "Windows": "win",
        "Darwin": "mac",
    }.get(uname[0])

    if chrome_platform in ("linux", "win"):
        bits = "64" if uname.machine == "x86_64" else "32"
    elif chrome_platform == "mac":
        bits = "-arm64" if uname.machine == "arm64" else "-x64"
    else:
        bits = ""

    chrome_dir = f"chrome-{chrome_platform}{bits}"

    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "-d", str(tmp_path), "--channel", channel, "chrome", "browser"])
    assert excinfo.value.code == 0
    assert os.path.isdir(os.path.join(tmp_path, "browsers", channel, chrome_dir))

    chrome = browser.Chrome(logging.getLogger("Chrome"))
    binary = chrome.find_binary(tmp_path, channel)
    assert binary is not None and os.path.exists(binary)


@pytest.mark.slow
@pytest.mark.remote_network
def test_install_chrome_chromedriver_by_version(tmp_path):
    # This is not technically an integration test as we do not want to require Chrome Stable to run it.
    chrome = browser.Chrome(logging.getLogger("Chrome"))
    if sys.platform == "win32":
        chromedriver_binary = "chromedriver.exe"
    else:
        chromedriver_binary = "chromedriver"
    # This is a stable version.
    binary_path = chrome.install_webdriver_by_version(
        dest=str(tmp_path), version="115.0.5790.170", channel="stable")
    assert os.path.samefile(
        binary_path,
        os.path.join(tmp_path, "chrome", chromedriver_binary),
    )


@pytest.mark.slow
@pytest.mark.remote_network
@pytest.mark.xfail(sys.platform == "win32",
                   reason="https://github.com/web-platform-tests/wpt/issues/17074")
def test_install_firefox(tmp_path):
    if sys.platform == "darwin":
        fx_binary = "Firefox Nightly.app"
    else:
        fx_binary = "firefox"
    with pytest.raises(SystemExit) as excinfo:
        wpt.main(argv=["install", "-d", str(tmp_path), "firefox", "browser", "--channel=nightly"])
    assert excinfo.value.code == 0
    assert os.path.exists(os.path.join(tmp_path, "browsers", "nightly", fx_binary))
