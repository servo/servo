from .base import get_timeout_multiplier, maybe_add_args, certificate_domain_list  # noqa: F401
from .webkit import WebKitBrowser
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401
from ..executors.executorwebkit import WebKitDriverWdspecExecutor  # noqa: F401

__wptrunner__ = {"product": "epiphany",
                 "check_args": "check_args",
                 "browser": "EpiphanyBrowser",
                 "browser_kwargs": "browser_kwargs",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "WebKitDriverWdspecExecutor"},
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "timeout_multiplier": "get_timeout_multiplier"}


def check_args(**kwargs):
    pass


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    # Workaround for https://gitlab.gnome.org/GNOME/libsoup/issues/172
    webdriver_required_args = ["--host=127.0.0.1"]
    webdriver_args = maybe_add_args(webdriver_required_args, kwargs.get("webdriver_args"))
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": webdriver_args}


def capabilities(server_config, **kwargs):
    args = kwargs.get("binary_args", [])
    if "--automation-mode" not in args:
        args.append("--automation-mode")

    return {
        "browserName": "Epiphany",
        "browserVersion": "3.31.4",  # First version to support automation
        "platformName": "ANY",
        "webkitgtk:browserOptions": {
            "binary": kwargs["binary"],
            "args": args,
            "certificates": certificate_domain_list(server_config.domains_set, kwargs["host_cert_path"])}}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = capabilities(server_config, **kwargs)
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {}


def run_info_extras(**kwargs):
    return {"webkit_port": "gtk"}


class EpiphanyBrowser(WebKitBrowser):
    def __init__(self, logger, binary=None, webdriver_binary=None,
                 webdriver_args=None):
        WebKitBrowser.__init__(self, logger, binary, webdriver_binary,
                               webdriver_args)
