from . import chrome
from .base import NullBrowser  # noqa: F401
from .base import get_timeout_multiplier   # noqa: F401
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor,  # noqa: F401
                                           WebDriverCrashtestExecutor)  # noqa: F401
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorchrome import ChromeDriverPrintRefTestExecutor  # noqa: F401


__wptrunner__ = {"product": "chromium",
                 "check_args": "check_args",
                 "browser": "ChromiumBrowser",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "print-reftest": "ChromeDriverPrintRefTestExecutor",
                              "wdspec": "WdspecExecutor",
                              "crashtest": "WebDriverCrashtestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "update_properties": "update_properties",
                 "timeout_multiplier": "get_timeout_multiplier"}


# Chromium will rarely need a product definition that is different from Chrome.
# If any wptrunner options need to differ from Chrome, they can be added as
# an additional step after the execution of Chrome's functions.
def check_args(**kwargs):
    chrome.check_args(**kwargs)


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return chrome.browser_kwargs(logger, test_type, run_info_data, config, **kwargs)


def executor_kwargs(logger, test_type, test_environment, run_info_data, **kwargs):
    return chrome.executor_kwargs(logger, test_type, test_environment, run_info_data, **kwargs)


def env_extras(**kwargs):
    return chrome.env_extras(**kwargs)


def env_options():
    return chrome.env_options()


def update_properties():
    return chrome.update_properties()


class ChromiumBrowser(chrome.ChromeBrowser):
    pass
