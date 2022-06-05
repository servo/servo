# mypy: allow-untyped-defs

from .base import cmd_arg, require_arg
from .base import WebDriverBrowser
from .base import get_timeout_multiplier   # noqa: F401
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401


__wptrunner__ = {"product": "edgechromium",
                 "check_args": "check_args",
                 "browser": "EdgeChromiumBrowser",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "WdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier",}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type,
                                           test_environment,
                                           run_info_data,
                                           **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["supports_eager_pageload"] = False

    capabilities = {
        "ms:edgeOptions": {
            "prefs": {
                "profile": {
                    "default_content_setting_values": {
                        "popups": 1
                    }
                }
            },
            "useAutomationExtension": False,
            "excludeSwitches": ["enable-automation"],
            "w3c": True
        }
    }

    if test_type == "testharness":
        capabilities["pageLoadStrategy"] = "none"

    for (kwarg, capability) in [("binary", "binary"), ("binary_args", "args")]:
        if kwargs[kwarg] is not None:
            capabilities["ms:edgeOptions"][capability] = kwargs[kwarg]

    if kwargs["headless"]:
        if "args" not in capabilities["ms:edgeOptions"]:
            capabilities["ms:edgeOptions"]["args"] = []
        if "--headless" not in capabilities["ms:edgeOptions"]["args"]:
            capabilities["ms:edgeOptions"]["args"].append("--headless")
        capabilities["ms:edgeOptions"]["args"].append("--use-fake-device-for-media-stream")

    executor_kwargs["capabilities"] = capabilities

    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


class EdgeChromiumBrowser(WebDriverBrowser):
    """MicrosoftEdge is backed by MSEdgeDriver, which is supplied through
    ``wptrunner.webdriver.EdgeChromiumDriverServer``.
    """

    def make_command(self):
        return [self.webdriver_binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path)] + self.webdriver_args
