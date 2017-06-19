import os
import subprocess
import tempfile

from mozprocess import ProcessHandler

from .base import Browser, require_arg, get_free_port, browser_command, ExecutorBrowser
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorservodriver import (ServoWebDriverTestharnessExecutor,
                                             ServoWebDriverRefTestExecutor)

here = os.path.join(os.path.split(__file__)[0])

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
    "update_properties": "update_properties",
}

hosts_text = """127.0.0.1 web-platform.test
127.0.0.1 www.web-platform.test
127.0.0.1 www1.web-platform.test
127.0.0.1 www2.web-platform.test
127.0.0.1 xn--n8j6ds53lwwkrqhv28a.web-platform.test
127.0.0.1 xn--lve-6lad.web-platform.test
"""


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(test_type, run_info_data, **kwargs):
    return {
        "binary": kwargs["binary"],
        "debug_info": kwargs["debug_info"],
        "user_stylesheets": kwargs.get("user_stylesheets"),
    }


def executor_kwargs(test_type, server_config, cache_manager, run_info_data, **kwargs):
    rv = base_executor_kwargs(test_type, server_config,
                              cache_manager, **kwargs)
    return rv


def env_extras(**kwargs):
    return []


def env_options():
    return {"host": "127.0.0.1",
            "external_host": "web-platform.test",
            "bind_hostname": "true",
            "testharnessreport": "testharnessreport-servodriver.js",
            "supports_debugger": True}


def update_properties():
    return ["debug", "os", "version", "processor", "bits"], None


def make_hosts_file():
    hosts_fd, hosts_path = tempfile.mkstemp()
    with os.fdopen(hosts_fd, "w") as f:
        f.write(hosts_text)
    return hosts_path


class ServoWebDriverBrowser(Browser):
    used_ports = set()

    def __init__(self, logger, binary, debug_info=None, webdriver_host="127.0.0.1",
                 user_stylesheets=None):
        Browser.__init__(self, logger)
        self.binary = binary
        self.webdriver_host = webdriver_host
        self.webdriver_port = None
        self.proc = None
        self.debug_info = debug_info
        self.hosts_path = make_hosts_file()
        self.command = None
        self.user_stylesheets = user_stylesheets if user_stylesheets else []

    def start(self, **kwargs):
        self.webdriver_port = get_free_port(4444, exclude=self.used_ports)
        self.used_ports.add(self.webdriver_port)

        env = os.environ.copy()
        env["HOST_FILE"] = self.hosts_path
        env["RUST_BACKTRACE"] = "1"

        debug_args, command = browser_command(
            self.binary,
            [
                "--hard-fail",
                "--webdriver", str(self.webdriver_port),
                "about:blank",
            ],
            self.debug_info
        )

        for stylesheet in self.user_stylesheets:
            command += ["--user-stylesheet", stylesheet]

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

    def stop(self, force=False):
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
