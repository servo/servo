from .base import inherit
from . import chrome

from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401


inherit(chrome, globals(), "chrome_webdriver")

# __wptrunner__ magically appears from inherit, F821 is undefined name
__wptrunner__["executor_kwargs"] = "executor_kwargs"  # noqa: F821
__wptrunner__["executor"]["testharness"] = "WebDriverTestharnessExecutor"  # noqa: F821
__wptrunner__["executor"]["reftest"] = "WebDriverRefTestExecutor"  # noqa: F821


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, run_info_data,
                                           **kwargs)
    executor_kwargs["close_after_done"] = True

    capabilities = {
        "browserName": "chrome",
        "platform": "ANY",
        "version": "",
        "goog:chromeOptions": {
            "prefs": {
                "profile": {
                    "default_content_setting_values": {
                        "popups": 1
                    }
                }
            },
            "w3c": True
        }
    }

    for (kwarg, capability) in [("binary", "binary"), ("binary_args", "args")]:
        if kwargs[kwarg] is not None:
            capabilities["goog:chromeOptions"][capability] = kwargs[kwarg]

    if test_type == "testharness":
        capabilities["goog:chromeOptions"]["useAutomationExtension"] = False
        capabilities["goog:chromeOptions"]["excludeSwitches"] = ["enable-automation"]

    executor_kwargs["capabilities"] = capabilities

    return executor_kwargs
