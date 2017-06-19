import os

from .base import NullBrowser, ExecutorBrowser, require_arg
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorservo import ServoTestharnessExecutor, ServoRefTestExecutor, ServoWdspecExecutor

here = os.path.join(os.path.split(__file__)[0])

__wptrunner__ = {
    "product": "servo",
    "check_args": "check_args",
    "browser": "ServoBrowser",
    "executor": {
        "testharness": "ServoTestharnessExecutor",
        "reftest": "ServoRefTestExecutor",
        "wdspec": "ServoWdspecExecutor",
    },
    "browser_kwargs": "browser_kwargs",
    "executor_kwargs": "executor_kwargs",
    "env_extras": "env_extras",
    "env_options": "env_options",
    "update_properties": "update_properties",
}


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(test_type, run_info_data, **kwargs):
    return {
        "binary": kwargs["binary"],
        "debug_info": kwargs["debug_info"],
        "binary_args": kwargs["binary_args"],
        "user_stylesheets": kwargs.get("user_stylesheets"),
        "ca_certificate_path": kwargs["ssl_env"].ca_cert_path(),
    }


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    rv = base_executor_kwargs(test_type, server_config,
                              cache_manager, **kwargs)
    rv["pause_after_test"] = kwargs["pause_after_test"]
    return rv


def env_extras(**kwargs):
    return []


def env_options():
    return {"host": "127.0.0.1",
            "external_host": "web-platform.test",
            "bind_hostname": "true",
            "testharnessreport": "testharnessreport-servo.js",
            "supports_debugger": True}


def update_properties():
    return ["debug", "os", "version", "processor", "bits"], None


class ServoBrowser(NullBrowser):
    def __init__(self, logger, binary, debug_info=None, binary_args=None,
                 user_stylesheets=None, ca_certificate_path=None):
        NullBrowser.__init__(self, logger)
        self.binary = binary
        self.debug_info = debug_info
        self.binary_args = binary_args or []
        self.user_stylesheets = user_stylesheets or []
        self.ca_certificate_path = ca_certificate_path

    def executor_browser(self):
        return ExecutorBrowser, {
            "binary": self.binary,
            "debug_info": self.debug_info,
            "binary_args": self.binary_args,
            "user_stylesheets": self.user_stylesheets,
            "ca_certificate_path": self.ca_certificate_path,
        }
