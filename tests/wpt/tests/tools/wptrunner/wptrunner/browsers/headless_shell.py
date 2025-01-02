# mypy: allow-untyped-defs

from .base import require_arg
from .base import get_timeout_multiplier   # noqa: F401
from .chrome import ChromeBrowser, debug_args  # noqa: F401
from .chrome import executor_kwargs as chrome_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorchrome import (  # noqa: F401
    ChromeDriverCrashTestExecutor,
    ChromeDriverPrintRefTestExecutor,
    ChromeDriverRefTestExecutor,
    ChromeDriverTestharnessExecutor,
)


__wptrunner__ = {"product": "headless_shell",
                 "check_args": "check_args",
                 "browser": "HeadlessShellBrowser",
                 "executor": {
                     "crashtest": "ChromeDriverCrashTestExecutor",
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
    executor_kwargs = chrome_executor_kwargs(logger, test_type, test_environment, run_info_data,
                                             subsuite, **kwargs)
    chrome_options = executor_kwargs["capabilities"]["goog:chromeOptions"]
    # Defaultly enable SiteIsolation in headless shell
    if "--disable-site-isolation-trials" not in chrome_options["args"]:
        chrome_options["args"].append("--site-per-process")
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {"server_host": "127.0.0.1",
            "supports_debugger": True}


def update_properties():
    return (["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]})


class HeadlessShellBrowser(ChromeBrowser):
    pass
