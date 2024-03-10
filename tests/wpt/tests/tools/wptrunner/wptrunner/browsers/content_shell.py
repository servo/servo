# mypy: allow-untyped-defs

from . import chrome_spki_certs
from .base import cmd_arg, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from .chrome import ChromeBrowser, debug_args
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorchrome import (  # noqa: F401
    ChromeDriverPrintRefTestExecutor,
    ChromeDriverRefTestExecutor,
    ChromeDriverTestharnessExecutor,
)
from ..executors.executorwebdriver import WebDriverCrashtestExecutor  # noqa: F401

ENABLE_THREADED_COMPOSITING_FLAG = '--enable-threaded-compositing'
DISABLE_THREADED_COMPOSITING_FLAG = '--disable-threaded-compositing'
DISABLE_THREADED_ANIMATION_FLAG = '--disable-threaded-animation'


__wptrunner__ = {"product": "content_shell",
                 "check_args": "check_args",
                 "browser": "ContentShellBrowser",
                 "executor": {
                     "crashtest": "WebDriverCrashtestExecutor",
                     "print-reftest": "ChromeDriverPrintRefTestExecutor",
                     "reftest": "ChromeDriverRefTestExecutor",
                     "testharness": "ChromeDriverTestharnessExecutor",
                     "wdspec": "WdspecExecutor",
                 },
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "update_properties": "update_properties",
                 "timeout_multiplier": "get_timeout_multiplier",}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args"),
            "debug_info": kwargs["debug_info"]}


def executor_kwargs(logger, test_type, test_environment, run_info_data, subsuite,
                    **kwargs):
    sanitizer_enabled = kwargs.get("sanitizer_enabled")
    if sanitizer_enabled:
        test_type = "crashtest"
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data,
                                           subsuite, **kwargs)
    executor_kwargs["sanitizer_enabled"] = sanitizer_enabled
    executor_kwargs["close_after_done"] = True
    executor_kwargs["reuse_window"] = kwargs.get("reuse_window", False)

    capabilities = {
        "goog:chromeOptions": {
            "prefs": {
                "profile": {
                    "default_content_setting_values": {
                        "popups": 1
                    }
                }
            },
            "excludeSwitches": ["enable-automation"],
            "w3c": True,
        }
    }

    chrome_options = capabilities["goog:chromeOptions"]
    if kwargs["binary"] is not None:
        chrome_options["binary"] = kwargs["binary"]

    chrome_options["args"] = []
    chrome_options["args"].append("--ignore-certificate-errors-spki-list=%s" %
        ','.join(chrome_spki_certs.IGNORE_CERTIFICATE_ERRORS_SPKI_LIST))
    # For WebTransport tests.
    chrome_options["args"].append("--webtransport-developer-mode")
    chrome_options["args"].append("--enable-blink-test-features")

    # always run in headful mode for content_shell

    if kwargs["debug_info"]:
        chrome_options["args"].extend(debug_args(kwargs["debug_info"]))

    for arg in kwargs.get("binary_args", []):
        # skip empty --user-data-dir args, and allow chromedriver to pick one.
        # Do not pass in --run-web-tests, otherwise content_shell will hang.
        if arg in ['--user-data-dir', '--run-web-tests']:
            continue
        if arg not in chrome_options["args"]:
            chrome_options["args"].append(arg)

    # Temporary workaround to align with RWT behavior. Unless a vts explicitly
    # enables threaded compositing, we should use single threaded compositing
    # Do not pass in DISABLE_THREADED_COMPOSITING_FLAG or
    # DISABLE_THREADED_ANIMATION_FLAG. Content shell will hang due to that.
    #if ENABLE_THREADED_COMPOSITING_FLAG not in subsuite.config.get("binary_args", []):
    #    chrome_options["args"].extend([DISABLE_THREADED_COMPOSITING_FLAG,
    #                 DISABLE_THREADED_ANIMATION_FLAG])

    for arg in subsuite.config.get("binary_args", []):
        if arg not in chrome_options["args"]:
            chrome_options["args"].append(arg)

    executor_kwargs["capabilities"] = capabilities

    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {"server_host": "127.0.0.1",
            "supports_debugger": True}


def update_properties():
    return (["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]})


class ContentShellBrowser(ChromeBrowser):
    def make_command(self):
        return [self.webdriver_binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path),
                cmd_arg("enable-chrome-logs")] + self.webdriver_args
