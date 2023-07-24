# mypy: allow-untyped-defs

import time
import subprocess
from .base import require_arg
from .base import WebDriverBrowser
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorselenium import (SeleniumTestharnessExecutor,  # noqa: F401
                                          SeleniumRefTestExecutor)  # noqa: F401

__wptrunner__ = {"product": "edge",
                 "check_args": "check_args",
                 "browser": "EdgeBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor",
                              "wdspec": "WdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "timeout_multiplier": "get_timeout_multiplier"}


def get_timeout_multiplier(test_type, run_info_data, **kwargs):
    if kwargs["timeout_multiplier"] is not None:
        return kwargs["timeout_multiplier"]
    if test_type == "wdspec":
        return 10
    return 1


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args"),
            "timeout_multiplier": get_timeout_multiplier(test_type,
                                                         run_info_data,
                                                         **kwargs)}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["timeout_multiplier"] = get_timeout_multiplier(test_type,
                                                                   run_info_data,
                                                                   **kwargs)
    executor_kwargs["capabilities"] = {}
    if test_type == "testharness":
        executor_kwargs["capabilities"]["pageLoadStrategy"] = "eager"
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {"supports_debugger": False}


class EdgeBrowser(WebDriverBrowser):
    init_timeout = 60

    def __init__(self, logger, binary, webdriver_binary, webdriver_args=None,
                 host="localhost", port=None, base_path="/", env=None, **kwargs):
        super().__init__(logger, binary, webdriver_binary, webdriver_args=webdriver_args,
                         host=host, port=port, base_path=base_path, env=env, **kwargs)
        self.host = "localhost"

    def stop(self, force=False):
        super(self).stop(force)
        # Wait for Edge browser process to exit if driver process is found
        edge_proc_name = 'MicrosoftEdge.exe'
        for i in range(0, 5):
            procs = subprocess.check_output(['tasklist', '/fi', 'ImageName eq ' + edge_proc_name])
            if b'MicrosoftWebDriver.exe' not in procs:
                # Edge driver process already exited, don't wait for browser process to exit
                break
            elif edge_proc_name.encode() in procs:
                time.sleep(0.5)
            else:
                break

        if edge_proc_name.encode() in procs:
            # close Edge process if it is still running
            subprocess.call(['taskkill.exe', '/f', '/im', 'microsoftedge*'])

    def make_command(self):
        return [self.webdriver_binary, f"--port={self.port}"] + self.webdriver_args


def run_info_extras(**kwargs):
    osReleaseCommand = r"(Get-ItemProperty 'HKLM:\Software\Microsoft\Windows NT\CurrentVersion').ReleaseId"
    osBuildCommand = r"(Get-ItemProperty 'HKLM:\Software\Microsoft\Windows NT\CurrentVersion').BuildLabEx"
    try:
        os_release = subprocess.check_output(["powershell.exe", osReleaseCommand]).strip()
        os_build = subprocess.check_output(["powershell.exe", osBuildCommand]).strip()
    except (subprocess.CalledProcessError, OSError):
        return {}

    rv = {"os_build": os_build,
          "os_release": os_release}
    return rv
