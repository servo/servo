import os
import tempfile

import moznetwork
from mozprocess import ProcessHandler
from mozprofile import FirefoxProfile
from mozrunner import FennecEmulatorRunner

from serve.serve import make_hosts_file

from .base import (get_free_port,
                   cmd_arg,
                   browser_command)
from ..executors.executormarionette import (MarionetteTestharnessExecutor,  # noqa: F401
                                            MarionetteRefTestExecutor)  # noqa: F401
from .firefox import (get_timeout_multiplier, update_properties, executor_kwargs, FirefoxBrowser)  # noqa: F401


__wptrunner__ = {"product": "fennec",
                 "check_args": "check_args",
                 "browser": "FennecBrowser",
                 "executor": {"testharness": "MarionetteTestharnessExecutor",
                              "reftest": "MarionetteRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "update_properties": "update_properties"}


def check_args(**kwargs):
    pass


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"package_name": kwargs["package_name"],
            "device_serial": kwargs["device_serial"],
            "prefs_root": kwargs["prefs_root"],
            "extra_prefs": kwargs["extra_prefs"],
            "test_type": test_type,
            "debug_info": kwargs["debug_info"],
            "symbols_path": kwargs["symbols_path"],
            "stackwalk_binary": kwargs["stackwalk_binary"],
            "certutil_binary": kwargs["certutil_binary"],
            "ca_certificate_path": config.ssl_config["ca_cert_path"],
            "stackfix_dir": kwargs["stackfix_dir"],
            "binary_args": kwargs["binary_args"],
            "timeout_multiplier": get_timeout_multiplier(test_type,
                                                         run_info_data,
                                                         **kwargs),
            "leak_check": kwargs["leak_check"],
            "stylo_threads": kwargs["stylo_threads"],
            "chaos_mode_flags": kwargs["chaos_mode_flags"],
            "config": config,
            "install_fonts": kwargs["install_fonts"],
            "tests_root": config.doc_root}


def env_extras(**kwargs):
    return []


def run_info_extras(**kwargs):
    return {"e10s": False,
            "headless": False,
            "sw-e10s": False}


def env_options():
    # The server host is set to public localhost IP so that resources can be accessed
    # from Android emulator
    return {"server_host": moznetwork.get_ip(),
            "bind_address": False,
            "supports_debugger": True}


def write_hosts_file(config, device):
    new_hosts = make_hosts_file(config, moznetwork.get_ip())
    current_hosts = device.get_file("/etc/hosts")
    if new_hosts == current_hosts:
        return
    hosts_fd, hosts_path = tempfile.mkstemp()
    try:
        with os.fdopen(hosts_fd, "w") as f:
            f.write(new_hosts)
        device.remount()
        device.push(hosts_path, "/etc/hosts")
    finally:
        os.remove(hosts_path)


class FennecBrowser(FirefoxBrowser):
    used_ports = set()
    init_timeout = 300
    shutdown_timeout = 60

    def __init__(self, logger, prefs_root, test_type, package_name=None,
                 device_serial="emulator-5444", **kwargs):
        FirefoxBrowser.__init__(self, logger, None, prefs_root, test_type, **kwargs)
        self._package_name = package_name
        self.device_serial = device_serial
        self.tests_root = kwargs["tests_root"]
        self.install_fonts = kwargs["install_fonts"]

    @property
    def package_name(self):
        """
        Name of app to run on emulator.
        """
        if self._package_name is None:
            self._package_name = "org.mozilla.fennec"
            user = os.getenv("USER")
            if user:
                self._package_name += "_" + user
        return self._package_name

    def start(self, **kwargs):
        if self.marionette_port is None:
            self.marionette_port = get_free_port(2828, exclude=self.used_ports)
            self.used_ports.add(self.marionette_port)

        env = {}
        env["MOZ_CRASHREPORTER"] = "1"
        env["MOZ_CRASHREPORTER_SHUTDOWN"] = "1"
        env["MOZ_DISABLE_NONLOCAL_CONNECTIONS"] = "1"
        env["STYLO_THREADS"] = str(self.stylo_threads)
        if self.chaos_mode_flags is not None:
            env["MOZ_CHAOSMODE"] = str(self.chaos_mode_flags)

        preferences = self.load_prefs()

        self.profile = FirefoxProfile(preferences=preferences)
        self.profile.set_preferences({"marionette.port": self.marionette_port,
                                      "dom.disable_open_during_load": False,
                                      "places.history.enabled": False,
                                      "dom.send_after_paint_to_content": True,
                                      "network.preload": True})
        if self.test_type == "reftest":
            self.logger.info("Setting android reftest preferences")
            self.profile.set_preferences({"browser.viewport.desktopWidth": 600,
                                          # Disable high DPI
                                          "layout.css.devPixelsPerPx": "1.0",
                                          # Ensure that the full browser element
                                          # appears in the screenshot
                                          "apz.allow_zooming": False,
                                          "android.widget_paints_background": False,
                                          # Ensure that scrollbars are always painted
                                          "ui.scrollbarFadeBeginDelay": 100000})

        if self.install_fonts:
            self.logger.debug("Copying Ahem font to profile")
            font_dir = os.path.join(self.profile.profile, "fonts")
            if not os.path.exists(font_dir):
                os.makedirs(font_dir)
            with open(os.path.join(self.tests_root, "fonts", "Ahem.ttf"), "rb") as src:
                with open(os.path.join(font_dir, "Ahem.ttf"), "wb") as dest:
                    dest.write(src.read())

        if self.leak_check and kwargs.get("check_leaks", True):
            self.leak_report_file = os.path.join(self.profile.profile, "runtests_leaks.log")
            if os.path.exists(self.leak_report_file):
                os.remove(self.leak_report_file)
            env["XPCOM_MEM_BLOAT_LOG"] = self.leak_report_file
        else:
            self.leak_report_file = None

        if self.ca_certificate_path is not None:
            self.setup_ssl()

        debug_args, cmd = browser_command(self.package_name,
                                          self.binary_args if self.binary_args else [] +
                                          [cmd_arg("marionette"), "about:blank"],
                                          self.debug_info)

        self.runner = FennecEmulatorRunner(app=self.package_name,
                                           profile=self.profile,
                                           cmdargs=cmd[1:],
                                           env=env,
                                           symbols_path=self.symbols_path,
                                           serial=self.device_serial,
                                           # TODO - choose appropriate log dir
                                           logdir=os.getcwd(),
                                           process_class=ProcessHandler,
                                           process_args={"processOutputLine": [self.on_output]})

        self.logger.debug("Starting %s" % self.package_name)
        # connect to a running emulator
        self.runner.device.connect()

        write_hosts_file(self.config, self.runner.device.device)

        self.runner.stop()
        self.runner.start(debug_args=debug_args, interactive=self.debug_info and self.debug_info.interactive)

        self.runner.device.device.forward(
            local="tcp:{}".format(self.marionette_port),
            remote="tcp:{}".format(self.marionette_port))

        self.logger.debug("%s Started" % self.package_name)

    def stop(self, force=False):
        if self.runner is not None:
            if (self.runner.device.connected and
                len(self.runner.device.device.list_forwards()) > 0):
                try:
                    self.runner.device.device.remove_forwards(
                        "tcp:{}".format(self.marionette_port))
                except Exception:
                    self.logger.warning("Failed to remove port forwarding")
            # We assume that stopping the runner prompts the
            # browser to shut down. This allows the leak log to be written
            self.runner.stop()
        self.logger.debug("stopped")
