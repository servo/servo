# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import tempfile
import shutil
import subprocess

import fxos_appgen
import gaiatest
import mozdevice
import moznetwork
import mozrunner
from marionette import expected
from marionette.by import By
from marionette.wait import Wait
from mozprofile import FirefoxProfile, Preferences

from .base import get_free_port, BrowserError, Browser, ExecutorBrowser
from ..executors.executormarionette import MarionetteTestharnessExecutor
from ..hosts import HostsFile, HostsLine
from ..environment import hostnames

here = os.path.split(__file__)[0]

__wptrunner__ = {"product": "b2g",
                 "check_args": "check_args",
                 "browser": "B2GBrowser",
                 "executor": {"testharness": "B2GMarionetteTestharnessExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_options": "env_options"}


def check_args(**kwargs):
    pass


def browser_kwargs(test_environment, **kwargs):
    return {"prefs_root": kwargs["prefs_root"],
            "no_backup": kwargs.get("b2g_no_backup", False)}


def executor_kwargs(test_type, server_config, cache_manager, **kwargs):
    timeout_multiplier = kwargs["timeout_multiplier"]
    if timeout_multiplier is None:
        timeout_multiplier = 2

    executor_kwargs = {"server_config": server_config,
                       "timeout_multiplier": timeout_multiplier,
                       "close_after_done": False}

    if test_type == "reftest":
        executor_kwargs["cache_manager"] = cache_manager

    return executor_kwargs


def env_options():
    return {"host": "web-platform.test",
            "bind_hostname": "false",
            "test_server_port": False}


class B2GBrowser(Browser):
    used_ports = set()
    init_timeout = 180

    def __init__(self, logger, prefs_root, no_backup=False):
        Browser.__init__(self, logger)
        logger.info("Waiting for device")
        subprocess.call(["adb", "wait-for-device"])
        self.device = mozdevice.DeviceManagerADB()
        self.marionette_port = get_free_port(2828, exclude=self.used_ports)
        self.used_ports.add(self.marionette_port)
        self.cert_test_app = None
        self.runner = None
        self.prefs_root = prefs_root

        self.no_backup = no_backup
        self.backup_path = None
        self.backup_paths = []
        self.backup_dirs = []

    def setup(self):
        self.logger.info("Running B2G setup")
        self.backup_path = tempfile.mkdtemp()

        self.logger.debug("Backing up device to %s"  % (self.backup_path,))

        if not self.no_backup:
            self.backup_dirs = [("/data/local", os.path.join(self.backup_path, "local")),
                                ("/data/b2g/mozilla", os.path.join(self.backup_path, "profile"))]

            self.backup_paths = [("/system/etc/hosts", os.path.join(self.backup_path, "hosts"))]

            for remote, local in self.backup_dirs:
                self.device.getDirectory(remote, local)

            for remote, local in self.backup_paths:
                self.device.getFile(remote, local)

        self.setup_hosts()

    def start(self):
        profile = FirefoxProfile()

        profile.set_preferences({"dom.disable_open_during_load": False,
                                 "marionette.defaultPrefs.enabled": True})

        self.logger.debug("Creating device runner")
        self.runner = mozrunner.B2GDeviceRunner(profile=profile)
        self.logger.debug("Starting device runner")
        self.runner.start()
        self.logger.debug("Device runner started")

    def setup_hosts(self):
        host_ip = moznetwork.get_ip()

        temp_dir = tempfile.mkdtemp()
        hosts_path = os.path.join(temp_dir, "hosts")
        remote_path = "/system/etc/hosts"
        try:
            self.device.getFile("/system/etc/hosts", hosts_path)

            with open(hosts_path) as f:
                hosts_file = HostsFile.from_file(f)

            for canonical_hostname in hostnames:
                hosts_file.set_host(HostsLine(host_ip, canonical_hostname))

            with open(hosts_path, "w") as f:
                hosts_file.to_file(f)

            self.logger.info("Installing hosts file")

            self.device.remount()
            self.device.removeFile(remote_path)
            self.device.pushFile(hosts_path, remote_path)
        finally:
            os.unlink(hosts_path)
            os.rmdir(temp_dir)

    def load_prefs(self):
        prefs_path = os.path.join(self.prefs_root, "prefs_general.js")
        if os.path.exists(prefs_path):
            preferences = Preferences.read_prefs(prefs_path)
        else:
            self.logger.warning("Failed to find base prefs file in %s" % prefs_path)
            preferences = []

        return preferences

    def stop(self):
        pass

    def on_output(self):
        raise NotImplementedError

    def cleanup(self):
        self.logger.debug("Running browser cleanup steps")

        self.device.remount()

        for remote, local in self.backup_dirs:
            self.device.removeDir(remote)
            self.device.pushDir(local, remote)

        for remote, local in self.backup_paths:
            self.device.removeFile(remote)
            self.device.pushFile(local, remote)

        shutil.rmtree(self.backup_path)
        self.device.reboot(wait=True)

    def pid(self):
        return None

    def is_alive(self):
        return True

    def executor_browser(self):
        return B2GExecutorBrowser, {"marionette_port": self.marionette_port}


class B2GExecutorBrowser(ExecutorBrowser):
    # The following methods are called from a different process
    def __init__(self, *args, **kwargs):
        ExecutorBrowser.__init__(self, *args, **kwargs)

        import sys, subprocess

        self.device = mozdevice.ADBDevice()
        self.device.forward("tcp:%s" % self.marionette_port,
                            "tcp:2828")
        self.executor = None
        self.marionette = None
        self.gaia_device = None
        self.gaia_apps = None

    def after_connect(self, executor):
        self.executor = executor
        self.marionette = executor.marionette
        self.executor.logger.debug("Running browser.after_connect steps")

        self.gaia_apps = gaiatest.GaiaApps(marionette=executor.marionette)

        self.executor.logger.debug("Waiting for homescreen to load")

        # Moved out of gaia_test temporarily
        self.executor.logger.info("Waiting for B2G to be ready")
        self.wait_for_homescreen(timeout=60)

        self.install_cert_app()
        self.use_cert_app()

    def install_cert_app(self):
        """Install the container app used to run the tests"""
        if fxos_appgen.is_installed("CertTest App"):
            self.executor.logger.info("CertTest App is already installed")
            return
        self.executor.logger.info("Installing CertTest App")
        app_path = os.path.join(here, "b2g_setup", "certtest_app.zip")
        fxos_appgen.install_app("CertTest App", app_path, marionette=self.marionette)
        self.executor.logger.debug("Install complete")

    def use_cert_app(self):
        """Start the app used to run the tests"""
        self.executor.logger.info("Homescreen loaded")
        self.gaia_apps.launch("CertTest App")

    def wait_for_homescreen(self, timeout):
        self.executor.logger.info("Waiting for home screen to load")
        Wait(self.marionette, timeout).until(expected.element_present(
            By.CSS_SELECTOR, '#homescreen[loading-state=false]'))


class B2GMarionetteTestharnessExecutor(MarionetteTestharnessExecutor):
    def after_connect(self):
        self.browser.after_connect(self)
        MarionetteTestharnessExecutor.after_connect(self)
