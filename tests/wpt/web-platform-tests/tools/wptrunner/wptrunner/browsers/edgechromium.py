from .base import Browser, ExecutorBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from ..webdriver_server import EdgeChromiumDriverServer
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401
from ..executors.executoredgechromium import EdgeChromiumDriverWdspecExecutor  # noqa: F401


__wptrunner__ = {"product": "edgechromium",
                 "check_args": "check_args",
                 "browser": "EdgeChromiumBrowser",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "EdgeChromiumDriverWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier",}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, run_info_data,
                                           **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["supports_eager_pageload"] = False

    capabilities = {
        "goog:chromeOptions": {
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
            capabilities["goog:chromeOptions"][capability] = kwargs[kwarg]

    if kwargs["headless"]:
        if "args" not in capabilities["goog:chromeOptions"]:
            capabilities["goog:chromeOptions"]["args"] = []
        if "--headless" not in capabilities["goog:chromeOptions"]["args"]:
            capabilities["goog:chromeOptions"]["args"].append("--headless")

    executor_kwargs["capabilities"] = capabilities

    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


class EdgeChromiumBrowser(Browser):
    """MicrosoftEdge is backed by MSEdgeDriver, which is supplied through
    ``wptrunner.webdriver.EdgeChromiumDriverServer``.
    """

    def __init__(self, logger, binary, webdriver_binary="msedgedriver",
                 webdriver_args=None):
        """Creates a new representation of MicrosoftEdge.  The `binary` argument gives
        the browser binary to use for testing."""
        Browser.__init__(self, logger)
        self.binary = binary
        self.server = EdgeChromiumDriverServer(self.logger,
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
        return self.server.is_alive()

    def cleanup(self):
        self.stop()

    def executor_browser(self):
        return ExecutorBrowser, {"webdriver_url": self.server.url}
