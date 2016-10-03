# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from .base import Browser, ExecutorBrowser, require_arg
from ..webdriver_server import EdgeDriverServer
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorselenium import (SeleniumTestharnessExecutor,
                                          SeleniumRefTestExecutor)

__wptrunner__ = {"product": "edge",
                 "check_args": "check_args",
                 "browser": "EdgeBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_options": "env_options"}


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")

def browser_kwargs(**kwargs):
    return {"webdriver_binary": kwargs["webdriver_binary"]}

def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    from selenium.webdriver import DesiredCapabilities

    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = dict(DesiredCapabilities.EDGE.items())
    return executor_kwargs

def env_options():
    return {"host": "web-platform.test",
            "bind_hostname": "true",
            "supports_debugger": False}

class EdgeBrowser(Browser):
    used_ports = set()

    def __init__(self, logger, webdriver_binary):
        Browser.__init__(self, logger)
        self.server = EdgeDriverServer(self.logger, binary=webdriver_binary)
        self.webdriver_host = "localhost"
        self.webdriver_port = self.server.port

    def start(self):
        print self.server.url
        self.server.start()

    def stop(self):
        self.server.stop()

    def pid(self):
        return self.server.pid

    def is_alive(self):
        # TODO(ato): This only indicates the server is alive,
        # and doesn't say anything about whether a browser session
        # is active.
        return self.server.is_alive()

    def cleanup(self):
        self.stop()

    def executor_browser(self):
        return ExecutorBrowser, {"webdriver_url": self.server.url}
