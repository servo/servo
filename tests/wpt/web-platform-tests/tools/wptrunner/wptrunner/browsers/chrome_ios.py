from .base import Browser, ExecutorBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from ..webdriver_server import CWTChromeDriverServer
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401
from ..executors.executorchrome import ChromeDriverWdspecExecutor  # noqa: F401


__wptrunner__ = {"product": "chrome_ios",
                 "check_args": "check_args",
                 "browser": "ChromeiOSBrowser",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}

def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                                           **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = {}
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


class ChromeiOSBrowser(Browser):
    """ChromeiOS is backed by CWTChromeDriver, which is supplied through
    ``wptrunner.webdriver.CWTChromeDriverServer``.
    """

    init_timeout = 120

    def __init__(self, logger, webdriver_binary, webdriver_args=None):
        """Creates a new representation of Chrome."""
        Browser.__init__(self, logger)
        self.server = CWTChromeDriverServer(self.logger,
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
