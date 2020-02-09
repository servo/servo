from .base import Browser, ExecutorBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from ..webdriver_server import OperaDriverServer
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorselenium import (SeleniumTestharnessExecutor,  # noqa: F401
                                          SeleniumRefTestExecutor)  # noqa: F401
from ..executors.executoropera import OperaDriverWdspecExecutor  # noqa: F401


__wptrunner__ = {"product": "opera",
                 "check_args": "check_args",
                 "browser": "OperaBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor",
                              "wdspec": "OperaDriverWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    from selenium.webdriver import DesiredCapabilities

    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, run_info_data, **kwargs)
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


class OperaBrowser(Browser):
    """Opera is backed by operadriver, which is supplied through
    ``wptrunner.webdriver.OperaDriverServer``.
    """

    def __init__(self, logger, binary, webdriver_binary="operadriver",
                 webdriver_args=None):
        """Creates a new representation of Opera.  The `binary` argument gives
        the browser binary to use for testing."""
        Browser.__init__(self, logger)
        self.binary = binary
        self.server = OperaDriverServer(self.logger,
                                        binary=webdriver_binary,
                                        args=webdriver_args)

    def start(self, **kwargs):
        self.server.start(block=False)

    def stop(self, force=False):
        self.server.stop(force=force)

    def pid(self):
        return self.server.pid

    def is_alive(self):
        # TODO(ato): This only indicates the driver is alive,
        # and doesn't say anything about whether a browser session
        # is active.
        return self.server.is_alive

    def cleanup(self):
        self.stop()

    def executor_browser(self):
        return ExecutorBrowser, {"webdriver_url": self.server.url}
