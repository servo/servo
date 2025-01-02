# mypy: allow-untyped-defs

import glob
import os
import shutil
import subprocess
import tarfile
import tempfile
import time

import requests

from io import StringIO

from .base import Browser, ExecutorBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorselenium import (SeleniumTestharnessExecutor,  # noqa: F401
                                          SeleniumRefTestExecutor)  # noqa: F401

here = os.path.dirname(__file__)
# Number of seconds to wait between polling operations when detecting status of
# Sauce Connect sub-process.
sc_poll_period = 1


__wptrunner__ = {"product": "sauce",
                 "check_args": "check_args",
                 "browser": "SauceBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}


def get_capabilities(**kwargs):
    browser_name = kwargs["sauce_browser"]
    platform = kwargs["sauce_platform"]
    version = kwargs["sauce_version"]
    build = kwargs["sauce_build"]
    tags = kwargs["sauce_tags"]
    tunnel_id = kwargs["sauce_tunnel_id"]
    prerun_script = {
        "safari": {
            "executable": "sauce-storage:safari-prerun.sh",
            "background": False,
        }
    }
    capabilities = {
        "browserName": browser_name,
        "build": build,
        "disablePopupHandler": True,
        "name": f"{browser_name} {version} on {platform}",
        "platform": platform,
        "public": "public",
        "selenium-version": "3.3.1",
        "tags": tags,
        "tunnel-identifier": tunnel_id,
        "version": version,
        "prerun": prerun_script.get(browser_name)
    }

    return capabilities


def get_sauce_config(**kwargs):
    browser_name = kwargs["sauce_browser"]
    sauce_user = kwargs["sauce_user"]
    sauce_key = kwargs["sauce_key"]

    hub_url = f"{sauce_user}:{sauce_key}@localhost:4445"
    data = {
        "url": "http://%s/wd/hub" % hub_url,
        "browserName": browser_name,
        "capabilities": get_capabilities(**kwargs)
    }

    return data


def check_args(**kwargs):
    require_arg(kwargs, "sauce_browser")
    require_arg(kwargs, "sauce_platform")
    require_arg(kwargs, "sauce_version")
    require_arg(kwargs, "sauce_user")
    require_arg(kwargs, "sauce_key")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    sauce_config = get_sauce_config(**kwargs)

    return {"sauce_config": sauce_config}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data, **kwargs)

    executor_kwargs["capabilities"] = get_capabilities(**kwargs)

    return executor_kwargs


def env_extras(**kwargs):
    return [SauceConnect(**kwargs)]


def env_options():
    return {"supports_debugger": False}


def get_tar(url, dest):
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    with tarfile.open(fileobj=StringIO(resp.raw.read())) as f:
        f.extractall(path=dest)


class SauceConnect():

    def __init__(self, **kwargs):
        self.sauce_user = kwargs["sauce_user"]
        self.sauce_key = kwargs["sauce_key"]
        self.sauce_tunnel_id = kwargs["sauce_tunnel_id"]
        self.sauce_connect_binary = kwargs.get("sauce_connect_binary")
        self.sauce_connect_args = kwargs.get("sauce_connect_args")
        self.sauce_init_timeout = kwargs.get("sauce_init_timeout")
        self.sc_process = None
        self.temp_dir = None
        self.env_config = None

    def __call__(self, env_options, env_config):
        self.env_config = env_config

        return self

    def __enter__(self):
        # Because this class implements the context manager protocol, it is
        # possible for instances to be provided to the `with` statement
        # directly. This class implements the callable protocol so that data
        # which is not available during object initialization can be provided
        # prior to this moment. Instances must be invoked in preparation for
        # the context manager protocol, but this additional constraint is not
        # itself part of the protocol.
        assert self.env_config is not None, 'The instance has been invoked.'

        if not self.sauce_connect_binary:
            self.temp_dir = tempfile.mkdtemp()
            get_tar("https://saucelabs.com/downloads/sc-4.4.9-linux.tar.gz", self.temp_dir)
            self.sauce_connect_binary = glob.glob(os.path.join(self.temp_dir, "sc-*-linux/bin/sc"))[0]

        self.upload_prerun_exec('edge-prerun.bat')
        self.upload_prerun_exec('safari-prerun.sh')

        self.sc_process = subprocess.Popen([
            self.sauce_connect_binary,
            "--user=%s" % self.sauce_user,
            "--api-key=%s" % self.sauce_key,
            "--no-remove-colliding-tunnels",
            "--tunnel-identifier=%s" % self.sauce_tunnel_id,
            "--metrics-address=0.0.0.0:9876",
            "--readyfile=./sauce_is_ready",
            "--tunnel-domains",
            ",".join(self.env_config.domains_set)
        ] + self.sauce_connect_args)

        tot_wait = 0
        while not os.path.exists('./sauce_is_ready') and self.sc_process.poll() is None:
            if not self.sauce_init_timeout or (tot_wait >= self.sauce_init_timeout):
                self.quit()

                raise SauceException("Sauce Connect Proxy was not ready after %d seconds" % tot_wait)

            time.sleep(sc_poll_period)
            tot_wait += sc_poll_period

        if self.sc_process.returncode is not None:
            raise SauceException("Unable to start Sauce Connect Proxy. Process exited with code %s", self.sc_process.returncode)

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.env_config = None
        self.quit()
        if self.temp_dir and os.path.exists(self.temp_dir):
            try:
                shutil.rmtree(self.temp_dir)
            except OSError:
                pass

    def upload_prerun_exec(self, file_name):
        auth = (self.sauce_user, self.sauce_key)
        url = f"https://saucelabs.com/rest/v1/storage/{self.sauce_user}/{file_name}?overwrite=true"

        with open(os.path.join(here, 'sauce_setup', file_name), 'rb') as f:
            requests.post(url, data=f, auth=auth)

    def quit(self):
        """The Sauce Connect process may be managing an active "tunnel" to the
        Sauce Labs service. Issue a request to the process to close any tunnels
        and exit. If this does not occur within 5 seconds, force the process to
        close."""
        kill_wait = 5
        tot_wait = 0
        self.sc_process.terminate()

        while self.sc_process.poll() is None:
            time.sleep(sc_poll_period)
            tot_wait += sc_poll_period

            if tot_wait >= kill_wait:
                self.sc_process.kill()
                break


class SauceException(Exception):
    pass


class SauceBrowser(Browser):
    init_timeout = 300

    def __init__(self, logger, sauce_config, **kwargs):
        Browser.__init__(self, logger)
        self.sauce_config = sauce_config

    def start(self, **kwargs):
        pass

    def stop(self, force=False):
        pass

    @property
    def pid(self):
        return None

    def is_alive(self):
        # TODO: Should this check something about the connection?
        return True

    def cleanup(self):
        pass

    def executor_browser(self):
        return ExecutorBrowser, {"webdriver_url": self.sauce_config["url"]}
