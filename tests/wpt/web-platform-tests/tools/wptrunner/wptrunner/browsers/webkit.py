# mypy: allow-untyped-defs

from .base import WebDriverBrowser, require_arg
from .base import get_timeout_multiplier, certificate_domain_list  # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor,  # noqa: F401
                                           WebDriverCrashtestExecutor)  # noqa: F401


__wptrunner__ = {"product": "webkit",
                 "check_args": "check_args",
                 "browser": "WebKitBrowser",
                 "browser_kwargs": "browser_kwargs",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "WdspecExecutor",
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


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
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


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = capabilities_for_port(test_environment.config,
                                                            **kwargs)
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


def run_info_extras(**kwargs):
    return {"webkit_port": kwargs["webkit_port"]}


class WebKitBrowser(WebDriverBrowser):
    """Generic WebKit browser is backed by WebKit's WebDriver implementation"""

    def make_command(self):
        return [self.webdriver_binary, f"--port={self.port}"] + self.webdriver_args
