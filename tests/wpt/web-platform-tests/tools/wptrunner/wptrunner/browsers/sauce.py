# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import glob
import os
import shutil
import subprocess
import tarfile
import tempfile
import time
from cStringIO import StringIO as CStringIO

import requests

from .base import Browser, ExecutorBrowser, require_arg
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executorselenium import (SeleniumTestharnessExecutor,
                                          SeleniumRefTestExecutor)

here = os.path.split(__file__)[0]


__wptrunner__ = {"product": "sauce",
                 "check_args": "check_args",
                 "browser": "SauceBrowser",
                 "executor": {"testharness": "SeleniumTestharnessExecutor",
                              "reftest": "SeleniumRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options"}


def get_capabilities(**kwargs):
    browser_name = kwargs["sauce_browser"]
    platform = kwargs["sauce_platform"]
    version = kwargs["sauce_version"]
    build = kwargs["sauce_build"]
    tags = kwargs["sauce_tags"]
    tunnel_id = kwargs["sauce_tunnel_id"]
    prerun_script = {
        "MicrosoftEdge": {
            "executable": "sauce-storage:edge-prerun.bat",
            "background": False,
        },
        "safari": {
            "executable": "sauce-storage:safari-prerun.sh",
            "background": False,
        }
    }
    capabilities = {
        "browserName": browser_name,
        "build": build,
        "disablePopupHandler": True,
        "name": "%s %s on %s" % (browser_name, version, platform),
        "platform": platform,
        "public": "public",
        "selenium-version": "3.3.1",
        "tags": tags,
        "tunnel-identifier": tunnel_id,
        "version": version,
        "prerun": prerun_script.get(browser_name)
    }

    if browser_name == 'MicrosoftEdge':
        capabilities['selenium-version'] = '2.4.8'

    return capabilities


def get_sauce_config(**kwargs):
    browser_name = kwargs["sauce_browser"]
    sauce_user = kwargs["sauce_user"]
    sauce_key = kwargs["sauce_key"]

    hub_url = "%s:%s@localhost:4445" % (sauce_user, sauce_key)
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


def browser_kwargs(test_type, run_info_data, **kwargs):
    sauce_config = get_sauce_config(**kwargs)

    return {"sauce_config": sauce_config}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, **kwargs)

    executor_kwargs["capabilities"] = get_capabilities(**kwargs)

    return executor_kwargs


def env_extras(**kwargs):
    return [SauceConnect(**kwargs)]


def env_options():
    return {"host": "web-platform.test",
            "bind_hostname": "true",
            "supports_debugger": False}


def get_tar(url, dest):
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    with tarfile.open(fileobj=CStringIO(resp.raw.read())) as f:
        f.extractall(path=dest)


class SauceConnect():

    def __init__(self, **kwargs):
        self.sauce_user = kwargs["sauce_user"]
        self.sauce_key = kwargs["sauce_key"]
        self.sauce_tunnel_id = kwargs["sauce_tunnel_id"]
        self.sauce_connect_binary = kwargs.get("sauce_connect_binary")
        self.sc_process = None
        self.temp_dir = None

    def __enter__(self, options):
        if not self.sauce_connect_binary:
            self.temp_dir = tempfile.mkdtemp()
            get_tar("https://saucelabs.com/downloads/sc-latest-linux.tar.gz", self.temp_dir)
            self.sauce_connect_binary = glob.glob(os.path.join(self.temp_dir, "sc-*-linux/bin/sc"))[0]

        self.upload_prerun_exec('edge-prerun.bat')
        self.upload_prerun_exec('safari-prerun.sh')

        self.sc_process = subprocess.Popen([
            self.sauce_connect_binary,
            "--user=%s" % self.sauce_user,
            "--api-key=%s" % self.sauce_key,
            "--no-remove-colliding-tunnels",
            "--tunnel-identifier=%s" % self.sauce_tunnel_id,
            "--readyfile=./sauce_is_ready",
            "--tunnel-domains",
            "web-platform.test",
            "*.web-platform.test"
        ])
        while not os.path.exists('./sauce_is_ready') and not self.sc_process.poll():
            time.sleep(5)

        if self.sc_process.returncode is not None and self.sc_process.returncode > 0:
            raise SauceException("Unable to start Sauce Connect Proxy. Process exited with code %s", self.sc_process.returncode)

    def __exit__(self, *args):
        self.sc_process.terminate()
        if os.path.exists(self.temp_dir):
            try:
                shutil.rmtree(self.temp_dir)
            except OSError:
                pass

    def upload_prerun_exec(self, file_name):
        auth = (self.sauce_user, self.sauce_key)
        url = "https://saucelabs.com/rest/v1/storage/%s/%s?overwrite=true" % (self.sauce_user, file_name)

        with open(os.path.join(here, 'sauce_setup', file_name), 'rb') as f:
            requests.post(url, data=f, auth=auth)


class SauceException(Exception):
    pass


class SauceBrowser(Browser):
    init_timeout = 300

    def __init__(self, logger, sauce_config):
        Browser.__init__(self, logger)
        self.sauce_config = sauce_config

    def start(self):
        pass

    def stop(self, force=False):
        pass

    def pid(self):
        return None

    def is_alive(self):
        # TODO: Should this check something about the connection?
        return True

    def cleanup(self):
        pass

    def executor_browser(self):
        return ExecutorBrowser, {"webdriver_url": self.sauce_config["url"]}
