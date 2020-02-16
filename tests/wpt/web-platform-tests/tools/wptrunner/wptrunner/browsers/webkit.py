from .base import Browser, ExecutorBrowser, require_arg
from .base import get_timeout_multiplier, certificate_domain_list  # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor,  # noqa: F401
                                           WebDriverCrashtestExecutor)  # noqa: F401
from ..executors.executorwebkit import WebKitDriverWdspecExecutor  # noqa: F401
from ..webdriver_server import WebKitDriverServer


__wptrunner__ = {"product": "webkit",
                 "check_args": "check_args",
                 "browser": "WebKitBrowser",
                 "browser_kwargs": "browser_kwargs",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "WebKitDriverWdspecExecutor",
                              "crashtest": "WebDriverCrashtestExecutor"},
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "timeout_multiplier": "get_timeout_multiplier"}


def check_args(**kwargs):
    require_arg(kwargs, "binary")
    require_arg(kwargs, "webdriver_binary")
    require_arg(kwargs, "webkit_port")


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def capabilities_for_port(server_config, **kwargs):
    port_name = kwargs["webkit_port"]
    if port_name in ["gtk", "wpe"]:
        port_key_map = {"gtk": "webkitgtk"}
        browser_options_port = port_key_map.get(port_name, port_name)
        browser_options_key = "%s:browserOptions" % browser_options_port

        return {
            "browserName": "MiniBrowser",
            "browserVersion": "2.20",
            "platformName": "ANY",
            browser_options_key: {
                "binary": kwargs["binary"],
                "args": kwargs.get("binary_args", []),
                "certificates": certificate_domain_list(server_config.domains_set, kwargs["host_cert_path"])}}

    return {}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = capabilities_for_port(server_config,
                                                            **kwargs)
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


def run_info_extras(**kwargs):
    return {"webkit_port": kwargs["webkit_port"]}


class WebKitBrowser(Browser):
    """Generic WebKit browser is backed by WebKit's WebDriver implementation,
    which is supplied through ``wptrunner.webdriver.WebKitDriverServer``.
    """

    def __init__(self, logger, binary, webdriver_binary=None,
                 webdriver_args=None):
        Browser.__init__(self, logger)
        self.binary = binary
        self.server = WebKitDriverServer(self.logger, binary=webdriver_binary,
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
