# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import os

from .base import NullBrowser, ExecutorBrowser, require_arg
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorservo import ServoTestharnessExecutor, ServoRefTestExecutor

here = os.path.join(os.path.split(__file__)[0])

__wptrunner__ = {"product": "servo",
                 "check_args": "check_args",
                 "browser": "ServoBrowser",
                 "executor": {"testharness": "ServoTestharnessExecutor",
                              "reftest": "ServoRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_options": "env_options"}


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(**kwargs):
    return {"binary": kwargs["binary"],
            "debug_info": kwargs["debug_info"],
            "user_stylesheets": kwargs.get("user_stylesheets")}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    rv = base_executor_kwargs(test_type, server_config,
                              cache_manager, **kwargs)
    rv["pause_after_test"] = kwargs["pause_after_test"]
    return rv

def env_options():
    return {"host": "localhost",
            "bind_hostname": "true",
            "testharnessreport": "testharnessreport-servo.js",
            "supports_debugger": True}


class ServoBrowser(NullBrowser):
    def __init__(self, logger, binary, debug_info=None, user_stylesheets=None):
        NullBrowser.__init__(self, logger)
        self.binary = binary
        self.debug_info = debug_info
        self.user_stylesheets = user_stylesheets or []

    def executor_browser(self):
        return ExecutorBrowser, {"binary": self.binary,
                                 "debug_info": self.debug_info,
                                 "user_stylesheets": self.user_stylesheets}
