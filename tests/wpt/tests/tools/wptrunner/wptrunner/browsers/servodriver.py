# mypy: allow-untyped-defs

import os
import tempfile

from tools.serve.serve import make_hosts_file

from .base import (WebDriverBrowser,
                   require_arg,
                   get_free_port)
from .base import get_timeout_multiplier   # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorservodriver import (ServoWebDriverTestharnessExecutor,  # noqa: F401
                                             ServoWebDriverRefTestExecutor)  # noqa: F401

here = os.path.dirname(__file__)

__wptrunner__ = {
    "product": "servodriver",
    "check_args": "check_args",
    "browser": "ServoWebDriverBrowser",
    "executor": {
        "testharness": "ServoWebDriverTestharnessExecutor",
        "reftest": "ServoWebDriverRefTestExecutor",
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
        "binary_args": kwargs["binary_args"],
        "debug_info": kwargs["debug_info"],
        "server_config": config,
        "user_stylesheets": kwargs.get("user_stylesheets"),
        "headless": kwargs.get("headless"),
    }


def executor_kwargs(logger, test_type, test_environment, run_info_data, **kwargs):
    rv = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    return rv


def env_extras(**kwargs):
    return []


def env_options():
    return {"server_host": "127.0.0.1",
            "supports_debugger": False}


def update_properties():
    return (["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]})


def write_hosts_file(config):
    hosts_fd, hosts_path = tempfile.mkstemp()
    with os.fdopen(hosts_fd, "w") as f:
        f.write(make_hosts_file(config, "127.0.0.1"))
    return hosts_path


class ServoWebDriverBrowser(WebDriverBrowser):
    init_timeout = 300  # Large timeout for cases where we're booting an Android emulator

    def __init__(self, logger, binary, debug_info=None, webdriver_host="127.0.0.1",
                 server_config=None, binary_args=None,
                 user_stylesheets=None, headless=None, **kwargs):
        hosts_path = write_hosts_file(server_config)
        port = get_free_port()
        env = os.environ.copy()
        env["HOST_FILE"] = hosts_path
        env["RUST_BACKTRACE"] = "1"

        args = [
            "--hard-fail",
            "--webdriver=%s" % port,
            "about:blank",
        ]

        ca_cert_path = server_config.ssl_config["ca_cert_path"]
        if ca_cert_path:
            args += ["--certificate-path", ca_cert_path]
        if binary_args:
            args += binary_args
        if user_stylesheets:
            for stylesheet in user_stylesheets:
                args += ["--user-stylesheet", stylesheet]
        if headless:
            args += ["--headless"]

        WebDriverBrowser.__init__(self, env=env, logger=logger, host=webdriver_host, port=port,
                                  supports_pac=False, webdriver_binary=binary, webdriver_args=args,
                                  binary=binary)
        self.hosts_path = hosts_path

    def cleanup(self):
        WebDriverBrowser.cleanup(self)
        os.remove(self.hosts_path)
