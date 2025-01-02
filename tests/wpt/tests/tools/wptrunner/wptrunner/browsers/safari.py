# mypy: allow-untyped-defs

import os
import plistlib
from packaging.version import Version
from shutil import which

import psutil

from .base import WebDriverBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor,  # noqa: F401
                                           WebDriverCrashtestExecutor)  # noqa: F401


__wptrunner__ = {"product": "safari",
                 "check_args": "check_args",
                 "browser": "SafariBrowser",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "WdspecExecutor",
                              "crashtest": "WebDriverCrashtestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "timeout_multiplier": "get_timeout_multiplier"}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args"),
            "kill_safari": kwargs.get("kill_safari", False)}


def executor_kwargs(logger, test_type, test_environment, run_info_data, **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = {}
    if test_type == "testharness":
        executor_kwargs["capabilities"]["pageLoadStrategy"] = "eager"
    if kwargs["binary"] is not None:
        raise ValueError("Safari doesn't support setting executable location")

    browser_bundle_version = run_info_data["browser_bundle_version"]
    if (browser_bundle_version is not None and
        Version(browser_bundle_version[2:]) >= Version("613.1.7.1")):
        logger.debug("using acceptInsecureCerts=True")
        executor_kwargs["capabilities"]["acceptInsecureCerts"] = True
    else:
        logger.warning("not using acceptInsecureCerts, Safari will require certificates to be trusted")

    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


def run_info_extras(logger, **kwargs):
    webdriver_binary = kwargs["webdriver_binary"]
    rv = {}

    safari_bundle, safari_info = get_safari_info(webdriver_binary)

    if safari_info is not None:
        assert safari_bundle is not None  # if safari_info is not None, this can't be
        _, webkit_info = get_webkit_info(safari_bundle)
        if webkit_info is None:
            webkit_info = {}
    else:
        safari_info = {}
        webkit_info = {}

    rv["browser_marketing_version"] = safari_info.get("CFBundleShortVersionString")
    rv["browser_bundle_version"] = safari_info.get("CFBundleVersion")
    rv["browser_webkit_bundle_version"] = webkit_info.get("CFBundleVersion")

    with open("/System/Library/CoreServices/SystemVersion.plist", "rb") as fp:
        system_version = plistlib.load(fp)

    rv["os_build"] = system_version["ProductBuildVersion"]

    return rv


def get_safari_info(wd_path):
    bundle_paths = [
        os.path.join(os.path.dirname(wd_path), "..", ".."),  # bundled Safari (e.g. STP)
        os.path.join(os.path.dirname(wd_path), "Safari.app"),  # local Safari build
        "/Applications/Safari.app",  # system Safari
    ]

    for bundle_path in bundle_paths:
        info_path = os.path.join(bundle_path, "Contents", "Info.plist")
        if not os.path.isfile(info_path):
            continue

        with open(info_path, "rb") as fp:
            info = plistlib.load(fp)

        # check we have a Safari family bundle
        ident = info.get("CFBundleIdentifier")
        if not isinstance(ident, str) or not ident.startswith("com.apple.Safari"):
            continue

        return (bundle_path, info)

    return (None, None)


def get_webkit_info(safari_bundle_path):
    framework_paths = [
        os.path.join(os.path.dirname(safari_bundle_path), "Contents", "Frameworks"),  # bundled Safari (e.g. STP)
        os.path.join(os.path.dirname(safari_bundle_path), ".."),  # local Safari build
        "/System/Library/PrivateFrameworks",
        "/Library/Frameworks",
        "/System/Library/Frameworks",
    ]

    for framework_path in framework_paths:
        info_path = os.path.join(framework_path, "WebKit.framework", "Versions", "Current", "Resources", "Info.plist")
        if not os.path.isfile(info_path):
            continue

        with open(info_path, "rb") as fp:
            info = plistlib.load(fp)
            return (framework_path, info)

    return (None, None)


class SafariBrowser(WebDriverBrowser):
    """Safari is backed by safaridriver, which is supplied through
    ``wptrunner.webdriver.SafariDriverServer``.
    """
    def __init__(self, logger, binary=None, webdriver_binary=None, webdriver_args=None,
                 port=None, env=None, kill_safari=False, **kwargs):
        """Creates a new representation of Safari.  The `webdriver_binary`
        argument gives the WebDriver binary to use for testing. (The browser
        binary location cannot be specified, as Safari and SafariDriver are
        coupled.) If `kill_safari` is True, then `Browser.stop` will stop Safari."""
        super().__init__(logger,
                         binary,
                         webdriver_binary,
                         webdriver_args=webdriver_args,
                         port=None,
                         supports_pac=False,
                         env=env)

        if "/" not in webdriver_binary:
            wd_path = which(webdriver_binary)
        else:
            wd_path = webdriver_binary
        self.safari_path = self._find_safari_executable(wd_path)

        logger.debug("WebDriver executable path: %s" % wd_path)
        logger.debug("Safari executable path: %s" % self.safari_path)

        self.kill_safari = kill_safari

    def _find_safari_executable(self, wd_path):
        bundle_path, info = get_safari_info(wd_path)

        exe = info.get("CFBundleExecutable")
        if not isinstance(exe, str):
            return None

        exe_path = os.path.join(bundle_path, "Contents", "MacOS", exe)
        if not os.path.isfile(exe_path):
            return None

        return exe_path

    def make_command(self):
        return [self.webdriver_binary, f"--port={self.port}"] + self.webdriver_args

    def stop(self, force=False):
        super().stop(force)

        if self.kill_safari:
            self.logger.debug("Going to stop Safari")
            for proc in psutil.process_iter(attrs=["exe"]):
                if proc.info["exe"] is None:
                    continue

                try:
                    if not os.path.samefile(proc.info["exe"], self.safari_path):
                        continue
                except OSError:
                    continue

                self.logger.debug("Stopping Safari %s" % proc.pid)
                try:
                    proc.terminate()
                    try:
                        proc.wait(10)
                    except psutil.TimeoutExpired:
                        proc.kill()
                        proc.wait(10)
                except psutil.NoSuchProcess:
                    pass
                except Exception:
                    # Safari is a singleton, so treat failure here as a critical error.
                    self.logger.critical("Failed to stop Safari")
                    raise
