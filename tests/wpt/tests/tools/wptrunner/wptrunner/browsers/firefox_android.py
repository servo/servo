# mypy: allow-untyped-defs

import os
import re
import subprocess
import traceback

from mozrunner import FennecEmulatorRunner, get_app_context

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
                      run_info_browser_version,
                      run_info_extras as fx_run_info_extras,
                      update_properties,  # noqa: F401
                      executor_kwargs as fx_executor_kwargs,  # noqa: F401
                      FirefoxWdSpecBrowser,
                      ProfileCreator as FirefoxProfileCreator)


__wptrunner__ = {"product": "firefox_android",
                 "check_args": "check_args",
                 "browser": {None: "FirefoxAndroidBrowser",
                             "wdspec": "FirefoxAndroidWdSpecBrowser"},
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
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs["webdriver_args"].copy(),
            "binary": None,
            "package_name": kwargs["package_name"],
            "device_serial": kwargs["device_serial"],
            "prefs_root": kwargs["prefs_root"],
            "extra_prefs": kwargs["extra_prefs"].copy(),
            "test_type": test_type,
            "debug_info": kwargs["debug_info"],
            "symbols_path": kwargs["symbols_path"],
            "stackwalk_binary": kwargs["stackwalk_binary"],
            "certutil_binary": kwargs["certutil_binary"],
            "ca_certificate_path": config.ssl_config["ca_cert_path"],
            "stackfix_dir": kwargs["stackfix_dir"],
            "binary_args": kwargs["binary_args"].copy(),
            "timeout_multiplier": get_timeout_multiplier(test_type,
                                                         run_info_data,
                                                         **kwargs),
            "disable_fission": kwargs["disable_fission"],
            # desktop only
            "leak_check": False,
            "chaos_mode_flags": kwargs["chaos_mode_flags"],
            "config": config,
            "install_fonts": kwargs["install_fonts"],
            "tests_root": config.doc_root,
            "specialpowers_path": kwargs["specialpowers_path"],
            "debug_test": kwargs["debug_test"],
            "env_extras": dict([x.split('=') for x in kwargs.get("env", [])])}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    rv = fx_executor_kwargs(logger, test_type, test_environment, run_info_data,
                            **kwargs)
    if test_type == "wdspec":
        rv["capabilities"]["moz:firefoxOptions"]["androidPackage"] = kwargs["package_name"]
    return rv


def env_extras(**kwargs):
    return []

# Default preferences for Android to use when the preference is not specifically stated.
# See Bug 1577912
def default_prefs():
    return {"fission.disableSessionHistoryInParent": "true"}

def run_info_extras(logger, **kwargs):
    rv = fx_run_info_extras(logger, default_prefs=default_prefs(), **kwargs)
    rv.update({"headless": False})

    if kwargs["browser_version"] is None:
        rv.update(run_info_browser_version(**kwargs))

        if rv.get("browser_version") is None:
            # If we didn't get the browser version from the apk, try to get it from adb dumpsys
            rv["browser_version"] = get_package_browser_version(logger,
                                                                kwargs["adb_binary"],
                                                                kwargs["package_name"])

    return rv


def get_package_browser_version(logger, adb_binary, package_name):
    if adb_binary is None:
        logger.warning("Couldn't run adb to get Firefox Android version number")
        return None
    try:
        completed = subprocess.run([adb_binary, "shell", "dumpsys", "package", package_name],
                                   check=True,
                                   capture_output=True,
                                   encoding="utf8")
    except subprocess.CalledProcessError as e:
        logger.warning(f"adb failed with return code {e.returncode}\nCaptured stderr:\n{e.stderr}")
        return None

    version_name_re = re.compile(r"^\s+versionName=(.*)")
    for line in completed.stdout.splitlines():
        m = version_name_re.match(line)
        if m is not None:
            return m.group(1)
    logger.warning("Failed to find versionName property in dumpsys output")


def env_options():
    return {"server_host": "127.0.0.1",
            "supports_debugger": True}


def get_environ(chaos_mode_flags, env_extras=None):
    env = {}
    if env_extras is not None:
        env.update(env_extras)
    env["MOZ_CRASHREPORTER"] = "1"
    env["MOZ_CRASHREPORTER_SHUTDOWN"] = "1"
    env["MOZ_DISABLE_NONLOCAL_CONNECTIONS"] = "1"
    if chaos_mode_flags is not None:
        env["MOZ_CHAOSMODE"] = hex(chaos_mode_flags)
    return env


class ProfileCreator(FirefoxProfileCreator):
    def __init__(self, logger, prefs_root, config, test_type, extra_prefs,
                 disable_fission, debug_test, browser_channel, binary,
                 package_name, certutil_binary, ca_certificate_path,
                 allow_list_paths=None):

        super().__init__(logger, prefs_root, config, test_type, extra_prefs,
                         disable_fission, debug_test, browser_channel, None,
                         package_name, certutil_binary, ca_certificate_path,
                         allow_list_paths)

    def _set_required_prefs(self, profile):
        profile.set_preferences({
            "network.dns.localDomains": ",".join(self.config.domains_set),
            "dom.disable_open_during_load": False,
            "places.history.enabled": False,
            "dom.send_after_paint_to_content": True,
        })

        if self.package_name == "org.mozilla.geckoview.test_runner":
            # Bug 1879324: The TestRunner doesn't support "beforeunload" prompts yet
            profile.set_preferences({"dom.disable_beforeunload": True})

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

        if self.test_type == "wdspec":
            profile.set_preferences({"remote.prefs.recommended": True})

        profile.set_preferences({"fission.autostart": True})
        if self.disable_fission:
            profile.set_preferences({"fission.autostart": False})

        profile.set_preferences(default_prefs())


class FirefoxAndroidBrowser(Browser):
    init_timeout = 300
    shutdown_timeout = 60

    def __init__(self, logger, prefs_root, test_type, package_name="org.mozilla.geckoview.test_runner",
                 device_serial=None, extra_prefs=None, debug_info=None,
                 symbols_path=None, stackwalk_binary=None, certutil_binary=None,
                 ca_certificate_path=None, stackfix_dir=None,
                 binary_args=None, timeout_multiplier=None, leak_check=False, asan=False,
                 chaos_mode_flags=None, config=None, browser_channel="nightly",
                 install_fonts=False, tests_root=None, specialpowers_path=None, adb_binary=None,
                 debug_test=False, disable_fission=False, **kwargs):

        super().__init__(logger)
        self.prefs_root = prefs_root
        self.test_type = test_type
        self.package_name = package_name
        self.device_serial = device_serial
        self.debug_info = debug_info
        self.symbols_path = symbols_path
        self.stackwalk_binary = stackwalk_binary
        self.certutil_binary = certutil_binary
        self.ca_certificate_path = ca_certificate_path
        self.stackfix_dir = stackfix_dir
        self.binary_args = binary_args
        self.timeout_multiplier = timeout_multiplier
        self.leak_check = leak_check
        self.asan = asan
        self.chaos_mode_flags = chaos_mode_flags
        self.config = config
        self.browser_channel = browser_channel
        self.install_fonts = install_fonts
        self.tests_root = tests_root
        self.specialpowers_path = specialpowers_path
        self.adb_binary = adb_binary
        self.disable_fission = disable_fission

        self.profile_creator = ProfileCreator(logger,
                                              prefs_root,
                                              config,
                                              test_type,
                                              extra_prefs,
                                              disable_fission,
                                              debug_test,
                                              browser_channel,
                                              None,
                                              package_name,
                                              certutil_binary,
                                              ca_certificate_path)

        self.marionette_port = None
        self.profile = None
        self.runner = None
        self.env_extras = kwargs["env_extras"]
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

        env = get_environ(self.chaos_mode_flags, self.env_extras)

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
            local=f"tcp:{self.marionette_port}",
            remote=f"tcp:{self.marionette_port}")

        for ports in self.config.ports.values():
            for port in ports:
                self.runner.device.device.reverse(
                    local=f"tcp:{port}",
                    remote=f"tcp:{port}")

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

    @property
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
                                 "extensions": [],
                                 "supports_devtools": False}

    def check_crash(self, process, test):
        if not os.environ.get("MINIDUMP_STACKWALK", "") and self.stackwalk_binary:
            os.environ["MINIDUMP_STACKWALK"] = self.stackwalk_binary
        try:
            return bool(self.runner.check_for_crashes(test_name=test))
        except Exception:
            # We sometimes see failures trying to copy the minidump files
            self.logger.warning(f"""Failed to complete crash check, assuming no crash:
{traceback.format_exc()}""")
            return False


class FirefoxAndroidWdSpecBrowser(FirefoxWdSpecBrowser):
    def __init__(self, logger, prefs_root, webdriver_binary, webdriver_args,
                 extra_prefs=None, debug_info=None, symbols_path=None, stackwalk_binary=None,
                 certutil_binary=None, ca_certificate_path=None,
                 disable_fission=False, stackfix_dir=None, leak_check=False,
                 asan=False, chaos_mode_flags=None, config=None,
                 browser_channel="nightly", headless=None, debug_test=None,
                 binary=None, package_name="org.mozilla.geckoview.test_runner", device_serial=None,
                 adb_binary=None, profile_creator_cls=ProfileCreator, **kwargs):

        super().__init__(logger, None, package_name, prefs_root, webdriver_binary, webdriver_args,
                         extra_prefs=extra_prefs, debug_info=debug_info, symbols_path=symbols_path,
                         stackwalk_binary=stackwalk_binary, certutil_binary=certutil_binary,
                         ca_certificate_path=ca_certificate_path,
                         disable_fission=disable_fission, stackfix_dir=stackfix_dir,
                         leak_check=leak_check, asan=asan,
                         chaos_mode_flags=chaos_mode_flags, config=config,
                         browser_channel=browser_channel, headless=headless,
                         debug_test=debug_test, profile_creator_cls=profile_creator_cls, **kwargs)

        self.config = config
        self.device_serial = device_serial
        # This is just to support the same adb lookup as for other test types
        context = get_app_context("fennec")(adb_path=adb_binary, device_serial=device_serial)
        self.device = context.get_device(context.adb, self.device_serial)

    def start(self, group_metadata, **kwargs):
        for ports in self.config.ports.values():
            for port in ports:
                self.device.reverse(
                    local=f"tcp:{port}",
                    remote=f"tcp:{port}")
        super().start(group_metadata, **kwargs)

    def stop(self, force=False):
        try:
            self.device.remove_reverses()
        except Exception as e:
            self.logger.warning("Failed to remove forwarded or reversed ports: %s" % e)
        super().stop(force=force)

    def get_env(self, binary, debug_info, headless, gmp_path, chaos_mode_flags, e10s):
        env = get_environ(chaos_mode_flags)
        env["RUST_BACKTRACE"] = "1"
        return env

    def executor_browser(self):
        cls, args = super().executor_browser()
        args["androidPackage"] = self.package_name
        args["androidDeviceSerial"] = self.device_serial
        args["env"] = self.env
        args["supports_devtools"] = False
        return cls, args
