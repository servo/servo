# mypy: allow-untyped-defs

from .base import NullBrowser   # noqa: F401
from .base import require_arg
from .base import get_timeout_multiplier   # noqa: F401
from .chrome import executor_kwargs as chrome_executor_kwargs
from .chrome_android import ChromeAndroidBrowserBase
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorchrome import ChromeDriverPrintRefTestExecutor  # noqa: F401
from ..executors.executorwebdriver import (WebDriverCrashtestExecutor,  # noqa: F401
                                           WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401


__wptrunner__ = {"product": "android_webview",
                 "check_args": "check_args",
                 "browser": "SystemWebViewShell",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "print-reftest": "ChromeDriverPrintRefTestExecutor",
                              "wdspec": "WdspecExecutor",
                              "crashtest": "WebDriverCrashtestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}

_wptserve_ports = set()


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "adb_binary": kwargs["adb_binary"],
            "device_serial": kwargs["device_serial"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args"),
            "stackwalk_binary": kwargs.get("stackwalk_binary"),
            "symbols_path": kwargs.get("symbols_path")}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    # Use update() to modify the global list in place.
    _wptserve_ports.update(set(
        test_environment.config['ports']['http'] + test_environment.config['ports']['https'] +
        test_environment.config['ports']['ws'] + test_environment.config['ports']['wss']
    ))

    executor_kwargs = chrome_executor_kwargs(logger, test_type, test_environment, run_info_data,
                                             **kwargs)
    del executor_kwargs["capabilities"]["goog:chromeOptions"]["prefs"]
    capabilities = executor_kwargs["capabilities"]
    # Note that for WebView, we launch a test shell and have the test shell use WebView.
    # https://chromium.googlesource.com/chromium/src/+/HEAD/android_webview/docs/webview-shell.md
    capabilities["goog:chromeOptions"]["androidPackage"] = \
        kwargs.get("package_name", "org.chromium.webview_shell")
    capabilities["goog:chromeOptions"]["androidActivity"] = \
        "org.chromium.webview_shell.WebPlatformTestsActivity"
    capabilities["goog:chromeOptions"]["androidKeepAppDataDir"] = \
        kwargs.get("keep_app_data_directory")

    # Workaround: driver.quit() cannot quit SystemWebViewShell.
    executor_kwargs["pause_after_test"] = False
    # Workaround: driver.close() is not supported.
    executor_kwargs["restart_after_test"] = True
    executor_kwargs["close_after_done"] = False
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    # allow the use of host-resolver-rules in lieu of modifying /etc/hosts file
    return {"server_host": "127.0.0.1"}


class SystemWebViewShell(ChromeAndroidBrowserBase):
    """Chrome is backed by chromedriver, which is supplied through
    ``wptrunner.webdriver.ChromeDriverServer``.
    """

    def __init__(self, logger, binary, webdriver_binary="chromedriver",
                 adb_binary=None,
                 remote_queue=None,
                 device_serial=None,
                 webdriver_args=None,
                 stackwalk_binary=None,
                 symbols_path=None):
        """Creates a new representation of Chrome.  The `binary` argument gives
        the browser binary to use for testing."""
        super().__init__(logger,
                         webdriver_binary, adb_binary, remote_queue,
                         device_serial, webdriver_args, stackwalk_binary,
                         symbols_path)
        self.binary = binary
        self.wptserver_ports = _wptserve_ports
