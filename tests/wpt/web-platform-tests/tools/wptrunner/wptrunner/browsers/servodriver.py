import os
import subprocess
import tempfile

from mozprocess import ProcessHandler

from tools.serve.serve import make_hosts_file

from .base import Browser, require_arg, get_free_port, browser_command, ExecutorBrowser
from .base import get_timeout_multiplier   # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorservodriver import (ServoWebDriverTestharnessExecutor,  # noqa: F401
                                             ServoWebDriverRefTestExecutor)  # noqa: F401

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
    "timeout_multiplier": "get_timeout_multiplier",
    "update_properties": "update_properties",
}


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {
        "binary": kwargs["binary"],
        "binary_args": kwargs["binary_args"],
        "debug_info": kwargs["debug_info"],
        "server_config": config,
        "user_stylesheets": kwargs.get("user_stylesheets"),
        "headless": kwargs.get("headless"),
    }


def executor_kwargs(test_type, server_config, cache_manager, run_info_data, **kwargs):
    rv = base_executor_kwargs(test_type, server_config,
                              cache_manager, run_info_data, **kwargs)
    return rv


def env_extras(**kwargs):
    return []


def env_options():
    return {"server_host": "127.0.0.1",
            "testharnessreport": "testharnessreport-servodriver.js",
            "supports_debugger": True}


def update_properties():
    return (["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]})


def write_hosts_file(config):
    hosts_fd, hosts_path = tempfile.mkstemp()
    with os.fdopen(hosts_fd, "w") as f:
        f.write(make_hosts_file(config, "127.0.0.1"))
    return hosts_path

class ServoWebDriverBrowser(Browser):
    init_timeout = 300  # Large timeout for cases where we're booting an Android emulator

    def __init__(self, logger, binary, debug_info=None, webdriver_host="127.0.0.1",
                 server_config=None, binary_args=None, user_stylesheets=None, headless=None):
        Browser.__init__(self, logger)
        self.binary = binary
        self.binary_args = binary_args or []
        self.webdriver_host = webdriver_host
        self.webdriver_port = None
        self.proc = None
        self.debug_info = debug_info
        self.hosts_path = write_hosts_file(server_config)
        self.server_ports = server_config.ports if server_config else {}
        self.command = None
        self.user_stylesheets = user_stylesheets if user_stylesheets else []
        self.headless = headless if headless else False
        self.ca_certificate_path = server_config.ssl_config["ca_cert_path"]

    def start(self, **kwargs):
        self.webdriver_port = get_free_port()

        env = os.environ.copy()
        env["HOST_FILE"] = self.hosts_path
        env["RUST_BACKTRACE"] = "1"
        env["EMULATOR_REVERSE_FORWARD_PORTS"] = ",".join(
            str(port)
            for _protocol, ports in self.server_ports.items()
            for port in ports
            if port
        )

        debug_args, command = browser_command(
            self.binary,
            self.binary_args + [
                "--hard-fail",
                "--webdriver=%s" % self.webdriver_port,
                "about:blank",
            ],
            self.debug_info
        )

        if self.headless:
            command += ["--headless"]

        if self.ca_certificate_path:
            command += ["--certificate-path", self.ca_certificate_path]

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
        return self.proc.poll() is None

    def cleanup(self):
        self.stop()
        os.remove(self.hosts_path)

    def executor_browser(self):
        assert self.webdriver_port is not None
        return ExecutorBrowser, {"webdriver_host": self.webdriver_host,
                                 "webdriver_port": self.webdriver_port,
                                 "init_timeout": self.init_timeout}
