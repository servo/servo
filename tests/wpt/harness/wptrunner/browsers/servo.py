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
            "debug_args": kwargs["debug_args"],
            "interactive": kwargs["interactive"]}


def executor_kwargs(test_type, server_config, cache_manager, **kwargs):
    rv = base_executor_kwargs(test_type, server_config,
                              cache_manager, **kwargs)
    rv["pause_after_test"] = kwargs["pause_after_test"]
    return rv

def env_options():
    return {"host": "localhost",
            "bind_hostname": "true",
            "testharnessreport": "testharnessreport-servo.js"}


class ServoBrowser(NullBrowser):
    def __init__(self, logger, binary, debug_args=None, interactive=False):
        NullBrowser.__init__(self, logger)
        self.binary = binary
        self.debug_args = debug_args
        self.interactive = interactive

    def executor_browser(self):
        return ExecutorBrowser, {"binary": self.binary,
                                 "debug_args": self.debug_args,
                                 "interactive": self.interactive}
