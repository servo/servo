import json
import os
import platform
import signal
import subprocess
import sys

import mozinfo
import mozleak
import mozversion
from mozprocess import ProcessHandler
from mozprofile import FirefoxProfile, Preferences
from mozrunner import FirefoxRunner
from mozrunner.utils import test_environment, get_stack_fixer_function
from mozcrash import mozcrash

from .base import (get_free_port,
                   Browser,
                   ExecutorBrowser,
                   require_arg,
                   cmd_arg,
                   browser_command)
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executormarionette import (MarionetteTestharnessExecutor,  # noqa: F401
                                            MarionetteRefTestExecutor,  # noqa: F401
                                            MarionetteWdspecExecutor)  # noqa: F401


here = os.path.join(os.path.split(__file__)[0])

__wptrunner__ = {"product": "firefox",
                 "check_args": "check_args",
                 "browser": "FirefoxBrowser",
                 "executor": {"testharness": "MarionetteTestharnessExecutor",
                              "reftest": "MarionetteRefTestExecutor",
                              "wdspec": "MarionetteWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "run_info_extras": "run_info_extras",
                 "update_properties": "update_properties",
                 "timeout_multiplier": "get_timeout_multiplier"}


def get_timeout_multiplier(test_type, run_info_data, **kwargs):
    if kwargs["timeout_multiplier"] is not None:
        return kwargs["timeout_multiplier"]
    if test_type == "reftest":
        if run_info_data["debug"] or run_info_data.get("asan"):
            return 4
        else:
            return 2
    elif run_info_data["debug"] or run_info_data.get("asan"):
        if run_info_data.get("ccov"):
            return 4
        else:
            return 3
    elif run_info_data["os"] == "android":
        return 4
    # https://bugzilla.mozilla.org/show_bug.cgi?id=1538725
    elif run_info_data["os"] == "win" and run_info_data["processor"] == "aarch64":
        return 4
    return 1


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "prefs_root": kwargs["prefs_root"],
            "extra_prefs": kwargs["extra_prefs"],
            "test_type": test_type,
            "debug_info": kwargs["debug_info"],
            "symbols_path": kwargs["symbols_path"],
            "stackwalk_binary": kwargs["stackwalk_binary"],
            "certutil_binary": kwargs["certutil_binary"],
            "ca_certificate_path": config.ssl_config["ca_cert_path"],
            "e10s": kwargs["gecko_e10s"],
            "enable_webrender": kwargs["enable_webrender"],
            "stackfix_dir": kwargs["stackfix_dir"],
            "binary_args": kwargs["binary_args"],
            "timeout_multiplier": get_timeout_multiplier(test_type,
                                                         run_info_data,
                                                         **kwargs),
            "leak_check": run_info_data["debug"] and (kwargs["leak_check"] is not False),
            "asan": run_info_data.get("asan"),
            "stylo_threads": kwargs["stylo_threads"],
            "chaos_mode_flags": kwargs["chaos_mode_flags"],
            "config": config,
            "browser_channel": kwargs["browser_channel"],
            "headless": kwargs["headless"]}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, server_config,
                                           cache_manager, run_info_data,
                                           **kwargs)
    executor_kwargs["close_after_done"] = test_type != "reftest"
    executor_kwargs["timeout_multiplier"] = get_timeout_multiplier(test_type,
                                                                   run_info_data,
                                                                   **kwargs)
    executor_kwargs["e10s"] = run_info_data["e10s"]
    capabilities = {}
    if test_type == "testharness":
        capabilities["pageLoadStrategy"] = "eager"
    if test_type == "reftest":
        executor_kwargs["reftest_internal"] = kwargs["reftest_internal"]
        executor_kwargs["reftest_screenshot"] = kwargs["reftest_screenshot"]
    if test_type == "wdspec":
        options = {}
        if kwargs["binary"]:
            options["binary"] = kwargs["binary"]
        if kwargs["binary_args"]:
            options["args"] = kwargs["binary_args"]
        if kwargs["headless"]:
            if "args" not in options:
                options["args"] = []
            if "--headless" not in options["args"]:
                options["args"].append("--headless")
        options["prefs"] = {
            "network.dns.localDomains": ",".join(server_config.domains_set)
        }
        for pref, value in kwargs["extra_prefs"]:
            options["prefs"].update({pref: Preferences.cast(value)})
        capabilities["moz:firefoxOptions"] = options
    if kwargs["certutil_binary"] is None:
        capabilities["acceptInsecureCerts"] = True
    if capabilities:
        executor_kwargs["capabilities"] = capabilities
    executor_kwargs["debug"] = run_info_data["debug"]
    executor_kwargs["ccov"] = run_info_data.get("ccov", False)
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    # The server host is set to 127.0.0.1 as Firefox is configured (through the
    # network.dns.localDomains preference set below) to resolve the test
    # domains to localhost without relying on the network stack.
    #
    # https://github.com/web-platform-tests/wpt/pull/9480
    return {"server_host": "127.0.0.1",
            "supports_debugger": True}


def run_info_extras(**kwargs):

    def get_bool_pref_if_exists(pref):
        for key, value in kwargs.get('extra_prefs', []):
            if pref == key:
                return value.lower() in ('true', '1')
        return None

    def get_bool_pref(pref):
        pref_value = get_bool_pref_if_exists(pref)
        return pref_value if pref_value is not None else False

    rv = {"e10s": kwargs["gecko_e10s"],
          "wasm": kwargs.get("wasm", True),
          "verify": kwargs["verify"],
          "headless": kwargs.get("headless", False) or "MOZ_HEADLESS" in os.environ,
          "fission": get_bool_pref("fission.autostart")}

    # The value of `sw-e10s` defaults to whether the "parent_intercept"
    # implementation is enabled for the current build. This value, however,
    # can be overridden by explicitly setting the pref with the `--setpref` CLI
    # flag, which is checked here. If not supplied, the default value of
    # `sw-e10s` will be filled in in `RunInfo`'s constructor.
    #
    # We can't capture the default value right now because (currently), it
    # defaults to the value of `nightly_build`, which isn't known until
    # `RunInfo`'s constructor.
    sw_e10s_override = get_bool_pref_if_exists("dom.serviceWorkers.parent_intercept")
    if sw_e10s_override is not None:
        rv["sw-e10s"] = sw_e10s_override

    rv.update(run_info_browser_version(kwargs["binary"]))
    return rv


def run_info_browser_version(binary):
    try:
        version_info = mozversion.get_version(binary)
    except mozversion.errors.VersionError:
        version_info = None
    if version_info:
        return {"browser_build_id": version_info.get("application_buildid", None),
                "browser_changeset": version_info.get("application_changeset", None)}
    return {}


def update_properties():
    return (["os", "debug", "webrender", "fission", "e10s", "sw-e10s", "processor"],
            {"os": ["version"], "processor": ["bits"]})


class FirefoxBrowser(Browser):
    init_timeout = 70
    shutdown_timeout = 70

    def __init__(self, logger, binary, prefs_root, test_type, extra_prefs=None, debug_info=None,
                 symbols_path=None, stackwalk_binary=None, certutil_binary=None,
                 ca_certificate_path=None, e10s=False, enable_webrender=False, stackfix_dir=None,
                 binary_args=None, timeout_multiplier=None, leak_check=False, asan=False,
                 stylo_threads=1, chaos_mode_flags=None, config=None, browser_channel="nightly", headless=None, **kwargs):
        Browser.__init__(self, logger)
        self.binary = binary
        self.prefs_root = prefs_root
        self.test_type = test_type
        self.extra_prefs = extra_prefs
        self.marionette_port = None
        self.runner = None
        self.debug_info = debug_info
        self.profile = None
        self.symbols_path = symbols_path
        self.stackwalk_binary = stackwalk_binary
        self.ca_certificate_path = ca_certificate_path
        self.certutil_binary = certutil_binary
        self.e10s = e10s
        self.enable_webrender = enable_webrender
        self.binary_args = binary_args
        self.config = config
        if stackfix_dir:
            self.stack_fixer = get_stack_fixer_function(stackfix_dir,
                                                        self.symbols_path)
        else:
            self.stack_fixer = None

        if timeout_multiplier:
            self.init_timeout = self.init_timeout * timeout_multiplier

        self.asan = asan
        self.lsan_allowed = None
        self.lsan_max_stack_depth = None
        self.mozleak_allowed = None
        self.mozleak_thresholds = None
        self.leak_check = leak_check
        self.leak_report_file = None
        self.lsan_handler = None
        self.stylo_threads = stylo_threads
        self.chaos_mode_flags = chaos_mode_flags
        self.browser_channel = browser_channel
        self.headless = headless

    def settings(self, test):
        return {"check_leaks": self.leak_check and not test.leaks,
                "lsan_allowed": test.lsan_allowed,
                "lsan_max_stack_depth": test.lsan_max_stack_depth,
                "mozleak_allowed": self.leak_check and test.mozleak_allowed,
                "mozleak_thresholds": self.leak_check and test.mozleak_threshold}

    def start(self, group_metadata=None, **kwargs):
        if group_metadata is None:
            group_metadata = {}

        self.group_metadata = group_metadata
        self.lsan_allowed = kwargs.get("lsan_allowed")
        self.lsan_max_stack_depth = kwargs.get("lsan_max_stack_depth")
        self.mozleak_allowed = kwargs.get("mozleak_allowed")
        self.mozleak_thresholds = kwargs.get("mozleak_thresholds")

        if self.marionette_port is None:
            self.marionette_port = get_free_port()

        if self.asan:
            self.lsan_handler = mozleak.LSANLeaks(self.logger,
                                                  scope=group_metadata.get("scope", "/"),
                                                  allowed=self.lsan_allowed,
                                                  maxNumRecordedFrames=self.lsan_max_stack_depth)

        env = test_environment(xrePath=os.path.dirname(self.binary),
                               debugger=self.debug_info is not None,
                               useLSan=True, log=self.logger)

        env["STYLO_THREADS"] = str(self.stylo_threads)
        if self.chaos_mode_flags is not None:
            env["MOZ_CHAOSMODE"] = str(self.chaos_mode_flags)
        if self.headless:
            env["MOZ_HEADLESS"] = "1"
        if self.enable_webrender:
            env["MOZ_WEBRENDER"] = "1"
            env["MOZ_ACCELERATED"] = "1"
        else:
            env["MOZ_WEBRENDER"] = "0"

        preferences = self.load_prefs()

        self.profile = FirefoxProfile(preferences=preferences)
        self.profile.set_preferences({
            "marionette.port": self.marionette_port,
            "network.dns.localDomains": ",".join(self.config.domains_set),
            "dom.file.createInChild": True,
            # TODO: Remove preferences once Firefox 64 is stable (Bug 905404)
            "network.proxy.type": 0,
            "places.history.enabled": False,
            "network.preload": True,
        })
        if self.e10s:
            self.profile.set_preferences({"browser.tabs.remote.autostart": True})

        if self.test_type == "reftest":
            self.profile.set_preferences({"layout.interruptible-reflow.enabled": False})

        if self.leak_check:
            self.leak_report_file = os.path.join(self.profile.profile, "runtests_leaks_%s.log" % os.getpid())
            if os.path.exists(self.leak_report_file):
                os.remove(self.leak_report_file)
            env["XPCOM_MEM_BLOAT_LOG"] = self.leak_report_file
        else:
            self.leak_report_file = None

        # Bug 1262954: winxp + e10s, disable hwaccel
        if (self.e10s and platform.system() in ("Windows", "Microsoft") and
            '5.1' in platform.version()):
            self.profile.set_preferences({"layers.acceleration.disabled": True})

        if self.ca_certificate_path is not None:
            self.setup_ssl()

        args = self.binary_args[:] if self.binary_args else []
        args += [cmd_arg("marionette"), "about:blank"]

        debug_args, cmd = browser_command(self.binary,
                                          args,
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
        prefs = Preferences()

        pref_paths = []

        profiles = os.path.join(self.prefs_root, 'profiles.json')
        if os.path.isfile(profiles):
            with open(profiles, 'r') as fh:
                for name in json.load(fh)['web-platform-tests']:
                    if self.browser_channel in (None, 'nightly'):
                        pref_paths.append(os.path.join(self.prefs_root, name, 'user.js'))
                    elif name != 'unittest-features':
                        pref_paths.append(os.path.join(self.prefs_root, name, 'user.js'))
        else:
            # Old preference files used before the creation of profiles.json (remove when no longer supported)
            legacy_pref_paths = (
                os.path.join(self.prefs_root, 'prefs_general.js'),   # Used in Firefox 60 and below
                os.path.join(self.prefs_root, 'common', 'user.js'),  # Used in Firefox 61
            )
            for path in legacy_pref_paths:
                if os.path.isfile(path):
                    pref_paths.append(path)

        for path in pref_paths:
            if os.path.exists(path):
                prefs.add(Preferences.read_prefs(path))
            else:
                self.logger.warning("Failed to find base prefs file in %s" % path)

        # Add any custom preferences
        prefs.add(self.extra_prefs, cast=True)

        return prefs()

    def stop(self, force=False):
        if self.runner is not None and self.runner.is_running():
            try:
                # For Firefox we assume that stopping the runner prompts the
                # browser to shut down. This allows the leak log to be written
                for clean, stop_f in [(True, lambda: self.runner.wait(self.shutdown_timeout)),
                                      (False, lambda: self.runner.stop(signal.SIGTERM)),
                                      (False, lambda: self.runner.stop(signal.SIGKILL))]:
                    if not force or not clean:
                        retcode = stop_f()
                        if retcode is not None:
                            self.logger.info("Browser exited with return code %s" % retcode)
                            break
            except OSError:
                # This can happen on Windows if the process is already dead
                pass
        self.process_leaks()
        self.logger.debug("stopped")

    def process_leaks(self):
        self.logger.info("PROCESS LEAKS %s" % self.leak_report_file)
        if self.lsan_handler:
            self.lsan_handler.process()
        if self.leak_report_file is not None:
            mozleak.process_leak_log(
                self.leak_report_file,
                leak_thresholds=self.mozleak_thresholds,
                ignore_missing_leaks=["gmplugin"],
                log=self.logger,
                stack_fixer=self.stack_fixer,
                scope=self.group_metadata.get("scope"),
                allowed=self.mozleak_allowed
            )

    def pid(self):
        if self.runner.process_handler is None:
            return None

        try:
            return self.runner.process_handler.pid
        except AttributeError:
            return None

    def on_output(self, line):
        """Write a line of output from the firefox process to the log"""
        if "GLib-GObject-CRITICAL" in line:
            return
        if line:
            data = line.decode("utf8", "replace")
            if self.stack_fixer:
                data = self.stack_fixer(data)
            if self.lsan_handler:
                data = self.lsan_handler.log(data)
            if data is not None:
                self.logger.process_output(self.pid(),
                                           data,
                                           command=" ".join(self.runner.command))

    def is_alive(self):
        if self.runner:
            return self.runner.is_running()
        return False

    def cleanup(self, force=False):
        self.stop(force)

    def executor_browser(self):
        assert self.marionette_port is not None
        return ExecutorBrowser, {"marionette_port": self.marionette_port}

    def check_crash(self, process, test):
        dump_dir = os.path.join(self.profile.profile, "minidumps")

        return bool(mozcrash.log_crashes(self.logger,
                                         dump_dir,
                                         symbols_path=self.symbols_path,
                                         stackwalk_binary=self.stackwalk_binary,
                                         process=process,
                                         test=test))

    def setup_ssl(self):
        """Create a certificate database to use in the test profile. This is configured
        to trust the CA Certificate that has signed the web-platform.test server
        certificate."""
        if self.certutil_binary is None:
            self.logger.info("--certutil-binary not supplied; Firefox will not check certificates")
            return

        self.logger.info("Setting up ssl")

        # Make sure the certutil libraries from the source tree are loaded when using a
        # local copy of certutil
        # TODO: Maybe only set this if certutil won't launch?
        env = os.environ.copy()
        certutil_dir = os.path.dirname(self.binary or self.certutil_binary)
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
