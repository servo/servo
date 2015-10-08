# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import subprocess
import tempfile

from mozprocess import ProcessHandler

from .base import Browser, require_arg, get_free_port, browser_command, ExecutorBrowser
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorservodriver import (ServoWebDriverTestharnessExecutor,
                                             ServoWebDriverRefTestExecutor)

here = os.path.join(os.path.split(__file__)[0])

__wptrunner__ = {"product": "servodriver",
                 "check_args": "check_args",
                 "browser": "ServoWebDriverBrowser",
                 "executor": {"testharness": "ServoWebDriverTestharnessExecutor",
                              "reftest": "ServoWebDriverRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_options": "env_options"}

hosts_text = """127.0.0.1 web-platform.test
127.0.0.1 www.web-platform.test
127.0.0.1 www1.web-platform.test
127.0.0.1 www2.web-platform.test
127.0.0.1 xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1 xn--lve-6lad.web-platform.test
"""


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(**kwargs):
    return {"binary": kwargs["binary"],
            "debug_info": kwargs["debug_info"]}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data, **kwargs):
    rv = base_executor_kwargs(test_type, server_config,
                              cache_manager, **kwargs)
    return rv


def env_options():
    return {"host": "127.0.0.1",
            "external_host": "web-platform.test",
            "bind_hostname": "true",
            "testharnessreport": "testharnessreport-servodriver.js",
            "supports_debugger": True}


def make_hosts_file():
    hosts_fd, hosts_path = tempfile.mkstemp()
    with os.fdopen(hosts_fd, "w") as f:
        f.write(hosts_text)
    return hosts_path


class ServoWebDriverBrowser(Browser):
    used_ports = set()

    def __init__(self, logger, binary, debug_info=None, webdriver_host="127.0.0.1"):
        Browser.__init__(self, logger)
        self.binary = binary
        self.webdriver_host = webdriver_host
        self.webdriver_port = None
        self.proc = None
        self.debug_info = debug_info
        self.hosts_path = make_hosts_file()
        self.command = None

    def start(self):
        self.webdriver_port = get_free_port(4444, exclude=self.used_ports)
        self.used_ports.add(self.webdriver_port)

        env = os.environ.copy()
        env["HOST_FILE"] = self.hosts_path

        debug_args, command = browser_command(self.binary,
                                              ["--cpu", "--hard-fail",
                                               "--webdriver", str(self.webdriver_port),
                                               "about:blank"],
                                              self.debug_info)

        self.command = command

        self.command = debug_args + self.command

        if not self.debug_info or not self.debug_info.interactive:
            self.proc = ProcessHandler(self.command,
                                       processOutputLine=[self.on_output],
                                       env=env,
                                       storeOutput=False)
            self.proc.run()
        else:
            self.proc = subprocess.Popen(self.command, env=env)

        self.logger.debug("Servo Started")

    def stop(self):
        self.logger.debug("Stopping browser")
        if self.proc is not None:
            try:
                self.proc.kill()
            except OSError:
                # This can happen on Windows if the process is already dead
                pass

    def pid(self):
        if self.proc is None:
            return None

        try:
            return self.proc.pid
        except AttributeError:
            return None

    def on_output(self, line):
        """Write a line of output from the process to the log"""
        self.logger.process_output(self.pid(),
                                   line.decode("utf8", "replace"),
                                   command=" ".join(self.command))

    def is_alive(self):
        if self.runner:
            return self.runner.is_running()
        return False

    def cleanup(self):
        self.stop()

    def executor_browser(self):
        assert self.webdriver_port is not None
        return ExecutorBrowser, {"webdriver_host": self.webdriver_host,
                                 "webdriver_port": self.webdriver_port}
