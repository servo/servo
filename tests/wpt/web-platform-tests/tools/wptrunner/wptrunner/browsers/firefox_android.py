import os

import moznetwork
from mozprofile import FirefoxProfile
from mozrunner import FennecEmulatorRunner

from .base import (get_free_port,
                   cmd_arg,
                   browser_command)
from ..executors.executormarionette import (MarionetteTestharnessExecutor,  # noqa: F401
                                            MarionetteRefTestExecutor)  # noqa: F401
from .firefox import (get_timeout_multiplier,  # noqa: F401
                      run_info_extras as fx_run_info_extras,
                      update_properties,  # noqa: F401
                      executor_kwargs,  # noqa: F401
                      FirefoxBrowser)  # noqa: F401


__wptrunner__ = {"product": "firefox_android",
                 "check_args": "check_args",
                 "browser": "FirefoxAndroidBrowser",
                 "executor": {"testharness": "MarionetteTestharnessExecutor",
                              "reftest": "MarionetteRefTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "update_properties": "update_properties",
                 "timeout_multiplier": "get_timeout_multiplier"}


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
            "enable_webrender": kwargs["enable_webrender"],
            "stackfix_dir": kwargs["stackfix_dir"],
            "binary_args": kwargs["binary_args"],
            "timeout_multiplier": get_timeout_multiplier(test_type,
                                                         run_info_data,
                                                         **kwargs),
            # desktop only
            "leak_check": False,
            "stylo_threads": kwargs["stylo_threads"],
            "chaos_mode_flags": kwargs["chaos_mode_flags"],
            "config": config,
            "install_fonts": kwargs["install_fonts"],
            "tests_root": config.doc_root}


def env_extras(**kwargs):
    return []


def run_info_extras(**kwargs):
    rv = fx_run_info_extras(**kwargs)
    package = kwargs["package_name"]
    rv.update({"e10s": True if package is not None and "geckoview" in package else False,
               "headless": False})
    return rv


def env_options():
    # The server host is set to public localhost IP so that resources can be accessed
    # from Android emulator
    return {"server_host": moznetwork.get_ip(),
            "bind_address": False,
            "supports_debugger": True}


class FirefoxAndroidBrowser(FirefoxBrowser):
    init_timeout = 300
    shutdown_timeout = 60

    def __init__(self, logger, prefs_root, test_type, package_name="org.mozilla.geckoview.test",
                 device_serial="emulator-5444", **kwargs):
        FirefoxBrowser.__init__(self, logger, None, prefs_root, test_type, **kwargs)
        self.package_name = package_name
        self.device_serial = device_serial
        self.tests_root = kwargs["tests_root"]
        self.install_fonts = kwargs["install_fonts"]
        self.stackwalk_binary = kwargs["stackwalk_binary"]

    def start(self, **kwargs):
        if self.marionette_port is None:
            self.marionette_port = get_free_port()

        env = {}
        env["MOZ_CRASHREPORTER"] = "1"
        env["MOZ_CRASHREPORTER_SHUTDOWN"] = "1"
        env["MOZ_DISABLE_NONLOCAL_CONNECTIONS"] = "1"
        env["STYLO_THREADS"] = str(self.stylo_threads)
        if self.chaos_mode_flags is not None:
            env["MOZ_CHAOSMODE"] = str(self.chaos_mode_flags)
        if self.enable_webrender:
            env["MOZ_WEBRENDER"] = "1"
        else:
            env["MOZ_WEBRENDER"] = "0"

        preferences = self.load_prefs()

        self.profile = FirefoxProfile(preferences=preferences)
        self.profile.set_preferences({
            "marionette.port": self.marionette_port,
            "network.dns.localDomains": ",".join(self.config.domains_set),
            "dom.disable_open_during_load": False,
            "places.history.enabled": False,
            "dom.send_after_paint_to_content": True,
            "network.preload": True,
        })

        if self.test_type == "reftest":
            self.logger.info("Setting android reftest preferences")
            self.profile.set_preferences({
                "browser.viewport.desktopWidth": 800,
                # Disable high DPI
                "layout.css.devPixelsPerPx": "1.0",
                # Ensure that the full browser element
                # appears in the screenshot
                "apz.allow_zooming": False,
                "android.widget_paints_background": False,
                # Ensure that scrollbars are always painted
                "layout.testing.overlay-scrollbars.always-visible": True,
            })

        if self.install_fonts:
            self.logger.debug("Copying Ahem font to profile")
            font_dir = os.path.join(self.profile.profile, "fonts")
            if not os.path.exists(font_dir):
                os.makedirs(font_dir)
            with open(os.path.join(self.tests_root, "fonts", "Ahem.ttf"), "rb") as src:
                with open(os.path.join(font_dir, "Ahem.ttf"), "wb") as dest:
                    dest.write(src.read())

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
                                           logdir=os.getcwd())

        self.logger.debug("Starting %s" % self.package_name)
        # connect to a running emulator
        self.runner.device.connect()

        self.runner.stop()
        self.runner.start(debug_args=debug_args,
                          interactive=self.debug_info and self.debug_info.interactive)

        self.runner.device.device.forward(
            local="tcp:{}".format(self.marionette_port),
            remote="tcp:{}".format(self.marionette_port))

        for ports in self.config.ports.values():
            for port in ports:
                self.runner.device.device.reverse(
                    local="tcp:{}".format(port),
                    remote="tcp:{}".format(port))

        self.logger.debug("%s Started" % self.package_name)

    def stop(self, force=False):
        if self.runner is not None:
            if self.runner.device.connected:
                try:
                    self.runner.device.device.remove_forwards()
                    self.runner.device.device.remove_reverses()
                except Exception as e:
                    self.logger.warning("Failed to remove forwarded or reversed ports: %s" % e)
            # We assume that stopping the runner prompts the
            # browser to shut down.
            self.runner.stop()
        self.logger.debug("stopped")

    def check_crash(self, process, test):
        if not os.environ.get("MINIDUMP_STACKWALK", "") and self.stackwalk_binary:
            os.environ["MINIDUMP_STACKWALK"] = self.stackwalk_binary
        return bool(self.runner.check_for_crashes(test_name=test))
