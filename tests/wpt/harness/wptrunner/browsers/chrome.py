# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

from .base import Browser, ExecutorBrowser, require_arg
from .webdriver import ChromedriverLocalServer
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorselenium import (SeleniumTestharnessExecutor,
                                          SeleniumRefTestExecutor)


__wptrunner__ = {"product": "chrome",
                 "check_args": "check_args",
                 "browser": "ChromeBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_options": "env_options"}


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(**kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"]}


def executor_kwargs(test_type, server_config, cache_manager, **kwargs):
    from selenium.webdriver import DesiredCapabilities

    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["capabilities"] = dict(DesiredCapabilities.CHROME.items() +
                                           {"chromeOptions":
                                            {"binary": kwargs["binary"]}}.items())

    return executor_kwargs


def env_options():
    return {"host": "web-platform.test",
            "bind_hostname": "true"}


class ChromeBrowser(Browser):
    """Chrome is backed by chromedriver, which is supplied through
    ``browsers.webdriver.ChromedriverLocalServer``."""

    def __init__(self, logger, binary, webdriver_binary="chromedriver"):
        """Creates a new representation of Chrome.  The `binary` argument gives
        the browser binary to use for testing."""
        Browser.__init__(self, logger)
        self.binary = binary
        self.driver = ChromedriverLocalServer(self.logger, binary=webdriver_binary)

    def start(self):
        self.driver.start()

    def stop(self):
        self.driver.stop()

    def pid(self):
        return self.driver.pid

    def is_alive(self):
        # TODO(ato): This only indicates the driver is alive,
        # and doesn't say anything about whether a browser session
        # is active.
        return self.driver.is_alive()

    def cleanup(self):
        self.stop()

    def executor_browser(self):
        return ExecutorBrowser, {"webdriver_url": self.driver.url}
