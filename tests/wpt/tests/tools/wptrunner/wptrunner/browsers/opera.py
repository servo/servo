# mypy: allow-untyped-defs

from .base import require_arg
from .base import get_timeout_multiplier   # noqa: F401
from .chrome import ChromeBrowser
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorselenium import (SeleniumTestharnessExecutor,  # noqa: F401
                                          SeleniumRefTestExecutor)  # noqa: F401


__wptrunner__ = {"product": "opera",
                 "check_args": "check_args",
                 "browser": "OperaBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor",
                              "wdspec": "WdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    from selenium.webdriver import DesiredCapabilities

    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    capabilities = dict(DesiredCapabilities.OPERA.items())
    capabilities.setdefault("operaOptions", {})["prefs"] = {
        "profile": {
            "default_content_setting_values": {
                "popups": 1
            }
        }
    }
    for (kwarg, capability) in [("binary", "binary"), ("binary_args", "args")]:
        if kwargs[kwarg] is not None:
            capabilities["operaOptions"][capability] = kwargs[kwarg]
    if test_type == "testharness":
        capabilities["operaOptions"]["useAutomationExtension"] = False
        capabilities["operaOptions"]["excludeSwitches"] = ["enable-automation"]
    if test_type == "wdspec":
        capabilities["operaOptions"]["w3c"] = True
    executor_kwargs["capabilities"] = capabilities
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


class OperaBrowser(ChromeBrowser):
    pass
