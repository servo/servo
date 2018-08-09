from .base import Browser, ExecutorBrowser, require_arg
from ..webdriver_server import EdgeDriverServer
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorselenium import (SeleniumTestharnessExecutor,  # noqa: F401
                                          SeleniumRefTestExecutor)  # noqa: F401
from ..executors.executoredge import EdgeDriverWdspecExecutor  # noqa: F401

__wptrunner__ = {"product": "edge",
                 "check_args": "check_args",
                 "browser": "EdgeBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor",
                              "wdspec": "EdgeDriverWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options"}

def get_timeout_multiplier(test_type, run_info_data, **kwargs):
    if kwargs["timeout_multiplier"] is not None:
        return kwargs["timeout_multiplier"]
    if test_type == "wdspec":
        return 10
    return 1

def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")

def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args"),
            "timeout_multiplier": get_timeout_multiplier(test_type,
                                                         run_info_data,
                                                         **kwargs)}

def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["timeout_multiplier"] = get_timeout_multiplier(test_type,
                                                                   run_info_data,
                                                                   **kwargs)
    executor_kwargs["capabilities"] = {}
    return executor_kwargs

def env_extras(**kwargs):
    return []

def env_options():
    return {"supports_debugger": False}

class EdgeBrowser(Browser):
    used_ports = set()
    init_timeout = 60

    def __init__(self, logger, webdriver_binary, timeout_multiplier=None, webdriver_args=None):
        Browser.__init__(self, logger)
        self.server = EdgeDriverServer(self.logger,
                                       binary=webdriver_binary,
                                       args=webdriver_args)
        self.webdriver_host = "localhost"
        self.webdriver_port = self.server.port
        if timeout_multiplier:
            self.init_timeout = self.init_timeout * timeout_multiplier


    def start(self, **kwargs):
        print self.server.url
        self.server.start()

    def stop(self, force=False):
        self.server.stop(force=force)

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
