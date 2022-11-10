# mypy: allow-untyped-defs

import os

from .base import ExecutorBrowser, NullBrowser, WebDriverBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorservo import (ServoCrashtestExecutor,  # noqa: F401
                                       ServoTestharnessExecutor,  # noqa: F401
                                       ServoRefTestExecutor)  # noqa: F401


here = os.path.dirname(__file__)

__wptrunner__ = {
    "product": "servo",
    "check_args": "check_args",
    "browser": {None: "ServoBrowser",
                "wdspec": "ServoWdspecBrowser"},
    "executor": {
        "crashtest": "ServoCrashtestExecutor",
        "testharness": "ServoTestharnessExecutor",
        "reftest": "ServoRefTestExecutor",
        "wdspec": "WdspecExecutor",
    },
    "browser_kwargs": "browser_kwargs",
    "executor_kwargs": "executor_kwargs",
    "env_extras": "env_extras",
    "env_options": "env_options",
    "timeout_multiplier": "get_timeout_multiplier",
    "update_properties": "update_properties",
}


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {
        "binary": kwargs["binary"],
        "debug_info": kwargs["debug_info"],
        "binary_args": kwargs["binary_args"],
        "user_stylesheets": kwargs.get("user_stylesheets"),
        "ca_certificate_path": config.ssl_config["ca_cert_path"],
    }


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    rv = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    rv["pause_after_test"] = kwargs["pause_after_test"]
    if test_type == "wdspec":
        rv["capabilities"] = {}
    return rv


def env_extras(**kwargs):
    return []


def env_options():
    return {"server_host": "127.0.0.1",
            "bind_address": False,
            "testharnessreport": "testharnessreport-servo.js",
            "supports_debugger": True}


def update_properties():
    return ["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]}


class ServoBrowser(NullBrowser):
    def __init__(self, logger, binary, debug_info=None, binary_args=None,
                 user_stylesheets=None, ca_certificate_path=None, **kwargs):
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


class ServoWdspecBrowser(WebDriverBrowser):
    # TODO: could share an implemenation with servodriver.py, perhaps
    def __init__(self, logger, binary="servo", webdriver_args=None,
                 binary_args=None, host="127.0.0.1", env=None, port=None):

        env = os.environ.copy() if env is None else env
        env["RUST_BACKTRACE"] = "1"

        super().__init__(logger,
                         binary,
                         None,
                         webdriver_args=webdriver_args,
                         host=host,
                         port=port,
                         env=env)
        self.binary_args = binary_args

    def make_command(self):
        command = [self.binary,
                   f"--webdriver={self.port}",
                   "--hard-fail",
                   "--headless"] + self.webdriver_args
        if self.binary_args:
            command += self.binary_args
        return command
