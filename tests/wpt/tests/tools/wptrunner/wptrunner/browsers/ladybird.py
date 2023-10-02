# mypy: allow-untyped-defs

from .base import (WebDriverBrowser,  # noqa: F401
                   get_timeout_multiplier,  # noqa: F401
                   require_arg)
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor,  # noqa: F401
                                           WebDriverCrashtestExecutor)  # noqa: F401

__wptrunner__ = {
    "product": "ladybird",
    "check_args": "check_args",
    "browser": "LadybirdBrowser",
    "browser_kwargs": "browser_kwargs",
    "executor_kwargs": "executor_kwargs",
    "env_options": "env_options",
    "env_extras": "env_extras",
    "timeout_multiplier": "get_timeout_multiplier",
    "executor": {
        "testharness": "WebDriverTestharnessExecutor",
        "reftest": "WebDriverRefTestExecutor",
        "wdspec": "WdspecExecutor",
        "crashtest": "WebDriverCrashtestExecutor"
    }
}

def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    executor_kwargs["capabilities"] = {}
    return executor_kwargs


def env_options():
    return {}


def env_extras(**kwargs):
    return []


class LadybirdBrowser(WebDriverBrowser):
    def make_command(self):
        return [self.webdriver_binary, "--port", str(self.port)] + self.webdriver_args
