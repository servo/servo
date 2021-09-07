import os

import moznetwork
from mozrunner import FennecEmulatorRunner

from .base import (get_free_port,
                   cmd_arg,
                   browser_command)
from ..executors.executormarionette import (MarionetteTestharnessExecutor,  # noqa: F401
                                            MarionetteRefTestExecutor,  # noqa: F401
                                            MarionetteCrashtestExecutor,  # noqa: F401
                                            MarionetteWdspecExecutor)  # noqa: F401
from .base import (Browser,
                   ExecutorBrowser)
from .firefox import (get_timeout_multiplier,  # noqa: F401
                      run_info_extras as fx_run_info_extras,
                      update_properties,  # noqa: F401
                      executor_kwargs as fx_executor_kwargs,  # noqa: F401
                      ProfileCreator as FirefoxProfileCreator)


__wptrunner__ = {"product": "firefox_android",
                 "check_args": "check_args",
                 "browser": "FirefoxAndroidBrowser",
                 "executor": {"testharness": "MarionetteTestharnessExecutor",
                              "reftest": "MarionetteRefTestExecutor",
                              "crashtest": "MarionetteCrashtestExecutor",
                              "wdspec": "MarionetteWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "update_properties": "update_properties",
                 "timeout_multiplier": "get_timeout_multiplier"}


def check_args(**kwargs):
    pass


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"adb_binary": kwargs["adb_binary"],
            "package_name": kwargs["package_name"],
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
            "e10s": run_info_data["e10s"],
            # desktop only
            "leak_check": False,
            "stylo_threads": kwargs["stylo_threads"],
            "chaos_mode_flags": kwargs["chaos_mode_flags"],
            "config": config,
            "install_fonts": kwargs["install_fonts"],
            "tests_root": config.doc_root,
            "specialpowers_path": kwargs["specialpowers_path"]}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    rv = fx_executor_kwargs(logger, test_type, test_environment, run_info_data,
                            **kwargs)
    if test_type == "wdspec":
        rv["capabilities"]["moz:firefoxOptions"]["androidPackage"] = kwargs["package_name"]
    return rv


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


class ProfileCreator(FirefoxProfileCreator):
    def __init__(self, logger, prefs_root, config, test_type, extra_prefs,
                 enable_fission, browser_channel, certutil_binary, ca_certificate_path):
        super(ProfileCreator, self).__init__(logger, prefs_root, config, test_type, extra_prefs,
                                             True, enable_fission, browser_channel, None,
                                             certutil_binary, ca_certificate_path)

    def _set_required_prefs(self, profile):
        profile.set_preferences({
            "network.dns.localDomains": ",".join(self.config.domains_set),
            "dom.disable_open_during_load": False,
            "places.history.enabled": False,
            "dom.send_after_paint_to_content": True,
            "network.preload": True,
            "browser.tabs.remote.autostart": True,
        })

        if self.test_type == "reftest":
            self.logger.info("Setting android reftest preferences")
            profile.set_preferences({
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


class FirefoxAndroidBrowser(Browser):
    init_timeout = 300
    shutdown_timeout = 60

    def __init__(self, logger, prefs_root, test_type, package_name="org.mozilla.geckoview.test",
                 device_serial="emulator-5444", extra_prefs=None, debug_info=None,
                 symbols_path=None, stackwalk_binary=None, certutil_binary=None,
                 ca_certificate_path=None, e10s=False, enable_webrender=False, stackfix_dir=None,
                 binary_args=None, timeout_multiplier=None, leak_check=False, asan=False,
                 stylo_threads=1, chaos_mode_flags=None, config=None, browser_channel="nightly",
                 install_fonts=False, tests_root=None, specialpowers_path=None, adb_binary=None,
                 **kwargs):

        super(FirefoxAndroidBrowser, self).__init__(logger)
        self.prefs_root = prefs_root
        self.test_type = test_type
        self.package_name = package_name
        self.device_serial = device_serial
        self.debug_info = debug_info
        self.symbols_path = symbols_path
        self.stackwalk_binary = stackwalk_binary
        self.certutil_binary = certutil_binary
        self.ca_certificate_path = ca_certificate_path
        self.e10s = True
        self.enable_webrender = enable_webrender
        self.stackfix_dir = stackfix_dir
        self.binary_args = binary_args
        self.timeout_multiplier = timeout_multiplier
        self.leak_check = leak_check
        self.asan = asan
        self.stylo_threads = stylo_threads
        self.chaos_mode_flags = chaos_mode_flags
        self.config = config
        self.browser_channel = browser_channel
        self.install_fonts = install_fonts
        self.tests_root = tests_root
        self.specialpowers_path = specialpowers_path
        self.adb_binary = adb_binary

        self.profile_creator = ProfileCreator(logger,
                                              prefs_root,
                                              config,
                                              test_type,
                                              extra_prefs,
                                              False,
                                              browser_channel,
                                              certutil_binary,
                                              ca_certificate_path)

        self.marionette_port = None
        self.profile = None
        self.runner = None
        self._settings = {}

    def settings(self, test):
        self._settings = {"check_leaks": self.leak_check and not test.leaks,
                          "lsan_allowed": test.lsan_allowed,
                          "lsan_max_stack_depth": test.lsan_max_stack_depth,
                          "mozleak_allowed": self.leak_check and test.mozleak_allowed,
                          "mozleak_thresholds": self.leak_check and test.mozleak_threshold,
                          "special_powers": self.specialpowers_path and test.url_base == "/_mozilla/"}
        return self._settings

    def start(self, **kwargs):
        if self.marionette_port is None:
            self.marionette_port = get_free_port()

        addons = [self.specialpowers_path] if self._settings.get("special_powers") else None
        self.profile = self.profile_creator.create(addons=addons)
        self.profile.set_preferences({"marionette.port": self.marionette_port})

        if self.install_fonts:
            self.logger.debug("Copying Ahem font to profile")
            font_dir = os.path.join(self.profile.profile, "fonts")
            if not os.path.exists(font_dir):
                os.makedirs(font_dir)
            with open(os.path.join(self.tests_root, "fonts", "Ahem.ttf"), "rb") as src:
                with open(os.path.join(font_dir, "Ahem.ttf"), "wb") as dest:
                    dest.write(src.read())

        self.leak_report_file = None

        debug_args, cmd = browser_command(self.package_name,
                                          self.binary_args if self.binary_args else [] +
                                          [cmd_arg("marionette"), "about:blank"],
                                          self.debug_info)

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

        self.runner = FennecEmulatorRunner(app=self.package_name,
                                           profile=self.profile,
                                           cmdargs=cmd[1:],
                                           env=env,
                                           symbols_path=self.symbols_path,
                                           serial=self.device_serial,
                                           # TODO - choose appropriate log dir
                                           logdir=os.getcwd(),
                                           adb_path=self.adb_binary,
                                           explicit_cleanup=True)

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
            self.runner.cleanup()
        self.logger.debug("stopped")

    def pid(self):
        if self.runner.process_handler is None:
            return None

        try:
            return self.runner.process_handler.pid
        except AttributeError:
            return None

    def is_alive(self):
        if self.runner:
            return self.runner.is_running()
        return False

    def cleanup(self, force=False):
        self.stop(force)

    def executor_browser(self):
        return ExecutorBrowser, {"marionette_port": self.marionette_port,
                                 # We never want marionette to install extensions because
                                 # that doesn't work on Android; instead they are in the profile
                                 "extensions": []}

    def check_crash(self, process, test):
        if not os.environ.get("MINIDUMP_STACKWALK", "") and self.stackwalk_binary:
            os.environ["MINIDUMP_STACKWALK"] = self.stackwalk_binary
        return bool(self.runner.check_for_crashes(test_name=test))
