# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import platform
import subprocess
import sys

import mozinfo
from mozprocess import ProcessHandler
from mozprofile import FirefoxProfile, Preferences
from mozprofile.permissions import ServerLocations
from mozrunner import FirefoxRunner
from mozcrash import mozcrash

from .base import (get_free_port,
                   Browser,
                   ExecutorBrowser,
                   require_arg,
                   cmd_arg,
                   browser_command)
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executormarionette import (MarionetteTestharnessExecutor,
                                            MarionetteRefTestExecutor,
                                            MarionetteWdspecExecutor)
from ..environment import hostnames


here = os.path.join(os.path.split(__file__)[0])

__wptrunner__ = {"product": "firefox",
                 "check_args": "check_args",
                 "browser": "FirefoxBrowser",
                 "executor": {"testharness": "MarionetteTestharnessExecutor",
                              "reftest": "MarionetteRefTestExecutor",
                              "wdspec": "MarionetteWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "update_properties": "update_properties"}


def check_args(**kwargs):
    require_arg(kwargs, "binary")
    if kwargs["ssl_type"] != "none":
        require_arg(kwargs, "certutil_binary")


def browser_kwargs(**kwargs):
    return {"binary": kwargs["binary"],
            "prefs_root": kwargs["prefs_root"],
            "debug_info": kwargs["debug_info"],
            "symbols_path": kwargs["symbols_path"],
            "stackwalk_binary": kwargs["stackwalk_binary"],
            "certutil_binary": kwargs["certutil_binary"],
            "ca_certificate_path": kwargs["ssl_env"].ca_cert_path(),
            "e10s": kwargs["gecko_e10s"]}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, **kwargs)
    executor_kwargs["close_after_done"] = True
    if kwargs["timeout_multiplier"] is None:
        if test_type == "reftest":
            if run_info_data["debug"] or run_info_data.get("asan"):
                executor_kwargs["timeout_multiplier"] = 4
            else:
                executor_kwargs["timeout_multiplier"] = 2
        elif run_info_data["debug"] or run_info_data.get("asan"):
            executor_kwargs["timeout_multiplier"] = 3
    if test_type == "wdspec":
        executor_kwargs["webdriver_binary"] = kwargs.get("webdriver_binary")
    return executor_kwargs


def env_options():
    return {"host": "127.0.0.1",
            "external_host": "web-platform.test",
            "bind_hostname": "false",
            "certificate_domain": "web-platform.test",
            "supports_debugger": True}


def run_info_extras(**kwargs):
    return {"e10s": kwargs["gecko_e10s"]}


def update_properties():
    return ["debug", "e10s", "os", "version", "processor", "bits"], {"debug", "e10s"}


class FirefoxBrowser(Browser):
    used_ports = set()
    init_timeout = 60

    def __init__(self, logger, binary, prefs_root, debug_info=None,
                 symbols_path=None, stackwalk_binary=None, certutil_binary=None,
                 ca_certificate_path=None, e10s=False):
        Browser.__init__(self, logger)
        self.binary = binary
        self.prefs_root = prefs_root
        self.marionette_port = None
        self.runner = None
        self.debug_info = debug_info
        self.profile = None
        self.symbols_path = symbols_path
        self.stackwalk_binary = stackwalk_binary
        self.ca_certificate_path = ca_certificate_path
        self.certutil_binary = certutil_binary
        self.e10s = e10s

    def start(self):
        self.marionette_port = get_free_port(2828, exclude=self.used_ports)
        self.used_ports.add(self.marionette_port)

        env = os.environ.copy()
        env["MOZ_DISABLE_NONLOCAL_CONNECTIONS"] = "1"

        locations = ServerLocations(filename=os.path.join(here, "server-locations.txt"))

        preferences = self.load_prefs()

        self.profile = FirefoxProfile(locations=locations,
                                      preferences=preferences)
        self.profile.set_preferences({"marionette.defaultPrefs.enabled": True,
                                      "marionette.defaultPrefs.port": self.marionette_port,
                                      "dom.disable_open_during_load": False,
                                      "network.dns.localDomains": ",".join(hostnames),
                                      "places.history.enabled": False})
        if self.e10s:
            self.profile.set_preferences({"browser.tabs.remote.autostart": True})

        # Bug 1262954: winxp + e10s, disable hwaccel
        if (self.e10s and platform.system() in ("Windows", "Microsoft") and
            '5.1' in platform.version()):
            self.profile.set_preferences({"layers.acceleration.disabled": True})

        if self.ca_certificate_path is not None:
            self.setup_ssl()

        debug_args, cmd = browser_command(self.binary, [cmd_arg("marionette"), "about:blank"],
                                          self.debug_info)

        self.runner = FirefoxRunner(profile=self.profile,
                                    binary=cmd[0],
                                    cmdargs=cmd[1:],
                                    env=env,
                                    process_class=ProcessHandler,
                                    process_args={"processOutputLine": [self.on_output]})

        self.logger.debug("Starting Firefox")

        self.runner.start(debug_args=debug_args, interactive=self.debug_info and self.debug_info.interactive)
        self.logger.debug("Firefox Started")

    def load_prefs(self):
        prefs_path = os.path.join(self.prefs_root, "prefs_general.js")
        if os.path.exists(prefs_path):
            preferences = Preferences.read_prefs(prefs_path)
        else:
            self.logger.warning("Failed to find base prefs file in %s" % prefs_path)
            preferences = []

        return preferences

    def stop(self):
        self.logger.debug("Stopping browser")
        if self.runner is not None:
            try:
                self.runner.stop()
            except OSError:
                # This can happen on Windows if the process is already dead
                pass

    def pid(self):
        if self.runner.process_handler is None:
            return None

        try:
            return self.runner.process_handler.pid
        except AttributeError:
            return None

    def on_output(self, line):
        """Write a line of output from the firefox process to the log"""
        self.logger.process_output(self.pid(),
                                   line.decode("utf8", "replace"),
                                   command=" ".join(self.runner.command))

    def is_alive(self):
        if self.runner:
            return self.runner.is_running()
        return False

    def cleanup(self):
        self.stop()

    def executor_browser(self):
        assert self.marionette_port is not None
        return ExecutorBrowser, {"marionette_port": self.marionette_port}

    def log_crash(self, process, test):
        dump_dir = os.path.join(self.profile.profile, "minidumps")

        mozcrash.log_crashes(self.logger,
                             dump_dir,
                             symbols_path=self.symbols_path,
                             stackwalk_binary=self.stackwalk_binary,
                             process=process,
                             test=test)

    def setup_ssl(self):
        """Create a certificate database to use in the test profile. This is configured
        to trust the CA Certificate that has signed the web-platform.test server
        certificate."""

        self.logger.info("Setting up ssl")

        # Make sure the certutil libraries from the source tree are loaded when using a
        # local copy of certutil
        # TODO: Maybe only set this if certutil won't launch?
        env = os.environ.copy()
        certutil_dir = os.path.dirname(self.binary)
        if mozinfo.isMac:
            env_var = "DYLD_LIBRARY_PATH"
        elif mozinfo.isUnix:
            env_var = "LD_LIBRARY_PATH"
        else:
            env_var = "PATH"


        env[env_var] = (os.path.pathsep.join([certutil_dir, env[env_var]])
                        if env_var in env else certutil_dir).encode(
                                sys.getfilesystemencoding() or 'utf-8', 'replace')

        def certutil(*args):
            cmd = [self.certutil_binary] + list(args)
            self.logger.process_output("certutil",
                                       subprocess.check_output(cmd,
                                                               env=env,
                                                               stderr=subprocess.STDOUT),
                                       " ".join(cmd))

        pw_path = os.path.join(self.profile.profile, ".crtdbpw")
        with open(pw_path, "w") as f:
            # Use empty password for certificate db
            f.write("\n")

        cert_db_path = self.profile.profile

        # Create a new certificate db
        certutil("-N", "-d", cert_db_path, "-f", pw_path)

        # Add the CA certificate to the database and mark as trusted to issue server certs
        certutil("-A", "-d", cert_db_path, "-f", pw_path, "-t", "CT,,",
                 "-n", "web-platform-tests", "-i", self.ca_certificate_path)

        # List all certs in the database
        certutil("-L", "-d", cert_db_path)
