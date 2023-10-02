# mypy: allow-untyped-defs
from .base import WebDriverBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from .base import cmd_arg
from .chrome import executor_kwargs as chrome_executor_kwargs
from ..executors.executorwebdriver import WebDriverCrashtestExecutor  # noqa: F401
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executoredge import (  # noqa: F401
    EdgeChromiumDriverPrintRefTestExecutor,
    EdgeChromiumDriverRefTestExecutor,
    EdgeChromiumDriverTestharnessExecutor,
)


__wptrunner__ = {"product": "edgechromium",
                 "check_args": "check_args",
                 "browser": "EdgeChromiumBrowser",
                 "executor": {"testharness": "EdgeChromiumDriverTestharnessExecutor",
                              "reftest": "EdgeChromiumDriverRefTestExecutor",
                              "print-reftest": "EdgeChromiumDriverPrintRefTestExecutor",
                              "wdspec": "WdspecExecutor",
                              "crashtest": "WebDriverCrashtestExecutor"},
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
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    executor_kwargs = chrome_executor_kwargs(logger, test_type, test_environment, run_info_data, **kwargs)
    capabilities = executor_kwargs["capabilities"]
    capabilities["ms:edgeOptions"] = capabilities.pop("goog:chromeOptions")
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {"server_host": "127.0.0.1"}


def update_properties():
    return (["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]})


class EdgeChromiumBrowser(WebDriverBrowser):
    """MicrosoftEdge is backed by MSEdgeDriver, which is supplied through
    ``wptrunner.webdriver.EdgeChromiumDriverServer``.
    """

    def make_command(self):
        return [self.webdriver_binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path),
                cmd_arg("enable-edge-logs")] + self.webdriver_args
