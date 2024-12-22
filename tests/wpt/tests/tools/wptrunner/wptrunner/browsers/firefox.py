# mypy: allow-untyped-defs

import json
import os
import re
import signal
import subprocess
import sys
import tempfile
import time
from abc import ABCMeta, abstractmethod
from http.client import HTTPConnection

import mozinfo
import mozleak
import mozversion
from mozprocess import ProcessHandler
from mozprofile import FirefoxProfile, Preferences
from mozrunner import FirefoxRunner
from mozrunner.utils import test_environment, get_stack_fixer_function
from mozcrash import mozcrash

from .base import (Browser,
                   ExecutorBrowser,
                   WebDriverBrowser,
                   OutputHandler,
                   OutputHandlerState,
                   browser_command,
                   cmd_arg,
                   get_free_port,
                   require_arg)
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.executormarionette import (MarionetteTestharnessExecutor,  # noqa: F401
                                            MarionetteRefTestExecutor,  # noqa: F401
                                            MarionettePrintRefTestExecutor,  # noqa: F401
                                            MarionetteWdspecExecutor,  # noqa: F401
                                            MarionetteCrashtestExecutor)  # noqa: F401



__wptrunner__ = {"product": "firefox",
                 "check_args": "check_args",
                 "browser": {None: "FirefoxBrowser",
                             "wdspec": "FirefoxWdSpecBrowser"},
                 "executor": {"crashtest": "MarionetteCrashtestExecutor",
                              "testharness": "MarionetteTestharnessExecutor",
                              "reftest": "MarionetteRefTestExecutor",
                              "print-reftest": "MarionettePrintRefTestExecutor",
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

    multiplier = 1
    if run_info_data["verify"]:
        if kwargs.get("chaos_mode_flags", None) is not None:
            multiplier = 2

    if test_type == "reftest":
        if (run_info_data["debug"] or
            run_info_data.get("asan") or
            run_info_data.get("tsan")):
            return 4 * multiplier
        else:
            return 2 * multiplier
    elif test_type == "wdspec":
        if (run_info_data.get("asan") or
            run_info_data.get("ccov") or
            run_info_data.get("debug")):
            return 4 * multiplier
        elif run_info_data.get("tsan"):
            return 8 * multiplier

        if run_info_data["os"] == "android":
            return 4 * multiplier
        return 1 * multiplier
    elif (run_info_data["debug"] or
          run_info_data.get("asan") or
          run_info_data.get("tsan")):
        if run_info_data.get("ccov"):
            return 4 * multiplier
        else:
            return 3 * multiplier
    elif run_info_data["os"] == "android":
        return 4 * multiplier
    # https://bugzilla.mozilla.org/show_bug.cgi?id=1538725
    elif run_info_data["os"] == "win" and run_info_data["processor"] == "aarch64":
        return 4 * multiplier
    elif run_info_data.get("ccov"):
        return 2 * multiplier
    return 1 * multiplier


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(logger, test_type, run_info_data, config, subsuite, **kwargs):
    browser_kwargs = {"binary": kwargs["binary"],
                      "package_name": None,
                      "webdriver_binary": kwargs["webdriver_binary"],
                      "webdriver_args": kwargs["webdriver_args"].copy(),
                      "prefs_root": kwargs["prefs_root"],
                      "extra_prefs": kwargs["extra_prefs"].copy(),
                      "test_type": test_type,
                      "debug_info": kwargs["debug_info"],
                      "symbols_path": kwargs["symbols_path"],
                      "stackwalk_binary": kwargs["stackwalk_binary"],
                      "certutil_binary": kwargs["certutil_binary"],
                      "ca_certificate_path": config.ssl_config["ca_cert_path"],
                      "e10s": kwargs["gecko_e10s"],
                      "disable_fission": kwargs["disable_fission"],
                      "stackfix_dir": kwargs["stackfix_dir"],
                      "binary_args": kwargs["binary_args"].copy(),
                      "timeout_multiplier": get_timeout_multiplier(test_type, run_info_data, **kwargs),
                      "leak_check": run_info_data["debug"] and (kwargs["leak_check"] is not False),
                      "asan": run_info_data.get("asan"),
                      "chaos_mode_flags": kwargs["chaos_mode_flags"],
                      "config": config,
                      "browser_channel": kwargs["browser_channel"],
                      "headless": kwargs["headless"],
                      "preload_browser": kwargs["preload_browser"] and not kwargs["pause_after_test"] and not kwargs["num_test_groups"] == 1,
                      "specialpowers_path": kwargs["specialpowers_path"],
                      "allow_list_paths": kwargs["allow_list_paths"],
                      "gmp_path": kwargs["gmp_path"] if "gmp_path" in kwargs else None,
                      "debug_test": kwargs["debug_test"]}
    if test_type == "wdspec" and kwargs["binary"]:
        browser_kwargs["webdriver_args"].extend(["--binary", kwargs["binary"]])
    browser_kwargs["binary_args"].extend(subsuite.config.get("binary_args", []))
    browser_kwargs["extra_prefs"].extend(subsuite.config.get("prefs", []))
    return browser_kwargs


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data,
                                           **kwargs)
    executor_kwargs["close_after_done"] = test_type != "reftest"
    executor_kwargs["timeout_multiplier"] = get_timeout_multiplier(test_type,
                                                                   run_info_data,
                                                                   **kwargs)
    executor_kwargs["e10s"] = run_info_data["e10s"]
    capabilities = {}
    if test_type == "testharness":
        capabilities["pageLoadStrategy"] = "eager"
    if test_type in ("reftest", "print-reftest"):
        executor_kwargs["reftest_internal"] = kwargs["reftest_internal"]
    if test_type == "wdspec":
        options = {"args": []}
        if kwargs["binary"]:
            executor_kwargs["webdriver_args"].extend(["--binary", kwargs["binary"]])
        if kwargs["binary_args"]:
            options["args"] = kwargs["binary_args"]

        if not kwargs["binary"] and kwargs["headless"] and "--headless" not in options["args"]:
            options["args"].append("--headless")

        executor_kwargs["binary_args"] = options["args"]
        capabilities["moz:firefoxOptions"] = options

    if kwargs["certutil_binary"] is None:
        capabilities["acceptInsecureCerts"] = True
    if capabilities:
        executor_kwargs["capabilities"] = capabilities
    executor_kwargs["debug"] = run_info_data["debug"]
    executor_kwargs["ccov"] = run_info_data.get("ccov", False)
    executor_kwargs["browser_version"] = run_info_data.get("browser_version")
    executor_kwargs["debug_test"] = kwargs["debug_test"]
    executor_kwargs["disable_fission"] = kwargs["disable_fission"]
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


def get_bool_pref(default_prefs, extra_prefs, pref):
    pref_value = False

    for key, value in extra_prefs + default_prefs:
        if pref == key:
            pref_value = value.lower() in ('true', '1')
            break

    return pref_value


def run_info_extras(logger, default_prefs=None, **kwargs):
    extra_prefs = kwargs.get("extra_prefs", [])
    default_prefs = list(default_prefs.items()) if default_prefs is not None else []

    def bool_pref(pref):
        return get_bool_pref(default_prefs, extra_prefs, pref)

    # Default fission to on, unless we get --disable-fission
    rv = {"e10s": kwargs["gecko_e10s"],
          "wasm": kwargs.get("wasm", True),
          "verify": kwargs["verify"],
          "headless": kwargs.get("headless", False) or "MOZ_HEADLESS" in os.environ,
          "fission": not kwargs.get("disable_fission"),
          "sessionHistoryInParent": (not kwargs.get("disable_fission") or
                                     not bool_pref("fission.disableSessionHistoryInParent")),
          "swgl": bool_pref("gfx.webrender.software"),
          "privateBrowsing": (kwargs["tags"] is not None and ("privatebrowsing" in kwargs["tags"])),
          "remoteAsyncEvents": bool_pref("remote.events.async.enabled"),
          }
    rv.update(run_info_browser_version(**kwargs))

    return rv


def run_info_browser_version(**kwargs):
    try:
        version_info = mozversion.get_version(kwargs["binary"])
    except mozversion.errors.VersionError:
        version_info = None
    if version_info:
        rv = {"browser_build_id": version_info.get("application_buildid", None),
              "browser_changeset": version_info.get("application_changeset", None)}
        if "browser_version" not in kwargs:
            rv["browser_version"] = version_info.get("application_version")
        return rv
    return {}


def update_properties():
    return ([
        "os",
        "debug",
        "fission",
        "processor",
        "swgl",
        "asan",
        "tsan",
        "remoteAsyncEvents",
        "sessionHistoryInParent",
        "subsuite"], {
        "os": ["version"],
        "processor": ["bits"]})


def log_gecko_crashes(logger, process, test, profile_dir, symbols_path, stackwalk_binary):
    dump_dir = os.path.join(profile_dir, "minidumps")

    try:
        return bool(mozcrash.log_crashes(logger,
                                         dump_dir,
                                         symbols_path=symbols_path,
                                         stackwalk_binary=stackwalk_binary,
                                         process=process,
                                         test=test))
    except OSError:
        logger.warning("Looking for crash dump files failed")
        return False


def get_environ(logger, binary, debug_info, headless, gmp_path, chaos_mode_flags=None, e10s=True):
    # Hack: test_environment expects a bin_suffix key in mozinfo that in gecko infrastructure
    # is set in the build system. Set it manually here.
    if "bin_suffix" not in mozinfo.info:
        mozinfo.info["bin_suffix"] = (".exe" if sys.platform in ["win32", "msys", "cygwin"]
                                      else "")

    # test_environment has started returning None values for some environment variables
    # that are only set in a gecko checkout
    env = {key: value for key, value in
           test_environment(xrePath=os.path.abspath(os.path.dirname(binary)),
                            debugger=debug_info is not None,
                            useLSan=True,
                            log=logger).items()
           if value is not None}

    if gmp_path is not None:
        env["MOZ_GMP_PATH"] = gmp_path
    # Disable window occlusion. Bug 1733955
    env["MOZ_WINDOW_OCCLUSION"] = "0"
    if chaos_mode_flags is not None:
        env["MOZ_CHAOSMODE"] = hex(chaos_mode_flags)
    if headless:
        env["MOZ_HEADLESS"] = "1"
    if not e10s:
        env["MOZ_FORCE_DISABLE_E10S"] = "1"
    return env


def setup_leak_report(leak_check, profile, env):
    leak_report_file = None
    if leak_check:
        filename = "runtests_leaks_%s.log" % os.getpid()
        if profile is not None:
            leak_report_file = os.path.join(profile.profile, filename)
        else:
            leak_report_file = os.path.join(tempfile.gettempdir(), filename)
        if os.path.exists(leak_report_file):
            os.remove(leak_report_file)
        env["XPCOM_MEM_BLOAT_LOG"] = leak_report_file

    return leak_report_file


class FirefoxInstanceManager:
    __metaclass__ = ABCMeta

    def __init__(self, logger, binary, binary_args, profile_creator, debug_info,
                 chaos_mode_flags, headless,
                 leak_check, stackfix_dir, symbols_path, gmp_path, asan, e10s):
        """Object that manages starting and stopping instances of Firefox."""
        self.logger = logger
        self.binary = binary
        self.binary_args = binary_args
        self.base_profile = profile_creator.create()
        self.debug_info = debug_info
        self.chaos_mode_flags = chaos_mode_flags
        self.headless = headless
        self.leak_check = leak_check
        self.stackfix_dir = stackfix_dir
        self.symbols_path = symbols_path
        self.gmp_path = gmp_path
        self.asan = asan
        self.e10s = e10s

        self.previous = None
        self.current = None

    @abstractmethod
    def teardown(self, force=False):
        pass

    @abstractmethod
    def get(self):
        """Get a BrowserInstance for a running Firefox.

        This can only be called once per instance, and between calls stop_current()
        must be called."""
        pass

    def stop_current(self, force=False):
        """Shutdown the current instance of Firefox.

        The BrowserInstance remains available through self.previous, since some
        operations happen after shutdown."""
        if not self.current:
            return

        self.current.stop(force)
        self.previous = self.current
        self.current = None

    def start(self):
        """Start an instance of Firefox, returning a BrowserInstance handle"""
        profile = self.base_profile.clone(self.base_profile.profile)

        marionette_port = get_free_port()
        profile.set_preferences({"marionette.port": marionette_port})

        env = get_environ(self.logger, self.binary, self.debug_info,
                          self.headless, self.gmp_path, self.chaos_mode_flags,
                          self.e10s)

        args = self.binary_args[:] if self.binary_args else []
        args += [cmd_arg("marionette"), "about:blank"]

        debug_args, cmd = browser_command(self.binary,
                                          args,
                                          self.debug_info)

        leak_report_file = setup_leak_report(self.leak_check, profile, env)
        output_handler = FirefoxOutputHandler(self.logger,
                                              cmd,
                                              stackfix_dir=self.stackfix_dir,
                                              symbols_path=self.symbols_path,
                                              asan=self.asan,
                                              leak_report_file=leak_report_file)
        runner = FirefoxRunner(profile=profile,
                               binary=cmd[0],
                               cmdargs=cmd[1:],
                               env=env,
                               process_class=ProcessHandler,
                               process_args={"processOutputLine": [output_handler]})
        instance = BrowserInstance(self.logger, runner, marionette_port,
                                   output_handler, leak_report_file)

        self.logger.debug("Starting Firefox")
        runner.start(debug_args=debug_args,
                     interactive=self.debug_info and self.debug_info.interactive)
        output_handler.after_process_start(runner.process_handler.pid)
        self.logger.debug("Firefox Started")

        return instance


class SingleInstanceManager(FirefoxInstanceManager):
    """FirefoxInstanceManager that manages a single Firefox instance"""
    def get(self):
        assert not self.current, ("Tried to call get() on InstanceManager that has "
                                  "an existing instance")
        if self.previous:
            self.previous.cleanup()
            self.previous = None
        self.current = self.start()
        return self.current

    def teardown(self, force=False):
        for instance in [self.previous, self.current]:
            if instance:
                instance.stop(force)
                instance.cleanup()
        self.base_profile.cleanup()


class PreloadInstanceManager(FirefoxInstanceManager):
    def __init__(self, *args, **kwargs):
        """FirefoxInstanceManager that keeps once Firefox instance preloaded
        to allow rapid resumption after an instance shuts down."""
        super().__init__(*args, **kwargs)
        self.pending = None

    def get(self):
        assert not self.current, ("Tried to call get() on InstanceManager that has "
                                  "an existing instance")
        if self.previous:
            self.previous.cleanup()
            self.previous = None
        if not self.pending:
            self.pending = self.start()
        self.current = self.pending
        self.pending = self.start()
        return self.current

    def teardown(self, force=False):
        for instance, unused in [(self.previous, False),
                                 (self.current, False),
                                 (self.pending, True)]:
            if instance:
                instance.stop(force, unused)
                instance.cleanup()
        self.base_profile.cleanup()


class BrowserInstance:
    shutdown_timeout = 70

    def __init__(self, logger, runner, marionette_port, output_handler, leak_report_file):
        """Handle to a running Firefox instance"""
        self.logger = logger
        self.runner = runner
        self.marionette_port = marionette_port
        self.output_handler = output_handler
        self.leak_report_file = leak_report_file

    def stop(self, force=False, unused=False):
        """Stop Firefox

        :param force: Signal the firefox process without waiting for a clean shutdown
        :param unused: This instance was not used for running tests and so
                       doesn't have an active marionette session and doesn't require
                       output postprocessing.
        """
        is_running = self.runner is not None and self.runner.is_running()
        if is_running:
            self.logger.debug("Stopping Firefox %s" % self.pid())
            shutdown_methods = [(True, lambda: self.runner.wait(self.shutdown_timeout)),
                                (False, lambda: self.runner.stop(signal.SIGTERM,
                                                                 self.shutdown_timeout))]
            if hasattr(signal, "SIGKILL"):
                shutdown_methods.append((False, lambda: self.runner.stop(signal.SIGKILL,
                                                                         self.shutdown_timeout)))
            if unused or force:
                # Don't wait for the instance to close itself
                shutdown_methods = shutdown_methods[1:]
            try:
                # For Firefox we assume that stopping the runner prompts the
                # browser to shut down. This allows the leak log to be written
                for i, (clean, stop_f) in enumerate(shutdown_methods):
                    self.logger.debug("Shutting down attempt %i/%i" % (i + 1, len(shutdown_methods)))
                    retcode = stop_f()
                    if retcode is not None:
                        self.logger.info("Browser exited with return code %s" % retcode)
                        break
            except OSError:
                # This can happen on Windows if the process is already dead
                pass
        elif self.runner:
            # The browser was already stopped, which we assume was a crash
            # TODO: Should we check the exit code here?
            clean = False
        if not unused:
            self.output_handler.after_process_stop(clean_shutdown=clean)

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

    def cleanup(self):
        self.runner.cleanup()
        self.runner = None


class FirefoxOutputHandler(OutputHandler):
    def __init__(self, logger, command, symbols_path=None, stackfix_dir=None, asan=False,
                 leak_report_file=None):
        """Filter for handling Firefox process output.

        This receives Firefox process output in the __call__ function, does
        any additional processing that's required, and decides whether to log
        the output. Because the Firefox process can be started before we know
        which filters are going to be required, we buffer all output until
        setup() is called. This is responsible for doing the final configuration
        of the output handlers.
        """

        super().__init__(logger, command)

        self.symbols_path = symbols_path
        if stackfix_dir:
            # We hide errors because they cause disconcerting `CRITICAL`
            # warnings in web platform test output.
            self.stack_fixer = get_stack_fixer_function(stackfix_dir,
                                                        self.symbols_path,
                                                        hideErrors=True)
        else:
            self.stack_fixer = None
        self.asan = asan
        self.leak_report_file = leak_report_file

        # These are filled in after configure_handlers() is called
        self.lsan_handler = None
        self.mozleak_allowed = None
        self.mozleak_thresholds = None
        self.group_metadata = {}

    def start(self, group_metadata=None, lsan_disabled=False, lsan_allowed=None,
              lsan_max_stack_depth=None, mozleak_allowed=None, mozleak_thresholds=None,
              **kwargs):
        """Configure the output handler"""
        if group_metadata is None:
            group_metadata = {}
        self.group_metadata = group_metadata

        self.mozleak_allowed = mozleak_allowed
        self.mozleak_thresholds = mozleak_thresholds

        if self.asan:
            self.lsan_handler = mozleak.LSANLeaks(self.logger,
                                                  scope=group_metadata.get("scope", "/"),
                                                  allowed=lsan_allowed,
                                                  maxNumRecordedFrames=lsan_max_stack_depth,
                                                  allowAll=lsan_disabled)
        else:
            self.lsan_handler = None
        super().start()

    def after_process_stop(self, clean_shutdown=True):
        super().after_process_stop(clean_shutdown)
        if self.lsan_handler:
            self.lsan_handler.process()
        if self.leak_report_file is not None:
            processed_files = None
            if not clean_shutdown:
                # If we didn't get a clean shutdown there probably isn't a leak report file
                self.logger.warning("Firefox didn't exit cleanly, not processing leak logs")
            else:
                # We have to ignore missing leaks in the tab because it can happen that the
                # content process crashed and in that case we don't want the test to fail.
                # Ideally we would record which content process crashed and just skip those.
                self.logger.info("PROCESS LEAKS %s" % self.leak_report_file)
                processed_files = mozleak.process_leak_log(
                    self.leak_report_file,
                    leak_thresholds=self.mozleak_thresholds,
                    ignore_missing_leaks=["tab", "gmplugin"],
                    log=self.logger,
                    stack_fixer=self.stack_fixer,
                    scope=self.group_metadata.get("scope"),
                    allowed=self.mozleak_allowed)
            if processed_files:
                for path in processed_files:
                    if os.path.exists(path):
                        os.unlink(path)
            # Fallback for older versions of mozleak, or if we didn't shutdown cleanly
            if os.path.exists(self.leak_report_file):
                os.unlink(self.leak_report_file)

    def __call__(self, line):
        """Write a line of output from the firefox process to the log"""
        if b"GLib-GObject-CRITICAL" in line:
            return
        if line:
            if self.state < OutputHandlerState.AFTER_HANDLER_START:
                self.line_buffer.append(line)
                return
            data = line.decode("utf8", "replace")
            if self.stack_fixer:
                data = self.stack_fixer(data)
            if self.lsan_handler:
                data = self.lsan_handler.log(data)
            if data is not None:
                self.logger.process_output(self.pid,
                                           data,
                                           command=" ".join(self.command))


class GeckodriverOutputHandler(FirefoxOutputHandler):
    PORT_RE = re.compile(rb".*Listening on [^ :]*:(\d+)")

    def __init__(self, logger, command, symbols_path=None, stackfix_dir=None, asan=False,
                 leak_report_file=None, init_deadline=None):
        super().__init__(logger, command, symbols_path=symbols_path, stackfix_dir=stackfix_dir, asan=asan,
                         leak_report_file=leak_report_file)
        self.port = None
        self.init_deadline = None

    def after_process_start(self, pid):
        super().after_process_start(pid)
        while self.port is None:
            time.sleep(0.1)
            if self.init_deadline is not None and time.time() > self.init_deadline:
                raise TimeoutError("Failed to get geckodriver port within the timeout")

    def __call__(self, line):
        if self.port is None:
            m = self.PORT_RE.match(line)
            if m is not None:
                self.port = int(m.groups()[0])
                self.logger.debug(f"Got geckodriver port {self.port}")
        super().__call__(line)


class ProfileCreator:
    def __init__(self, logger, prefs_root, config, test_type, extra_prefs,
                 disable_fission, debug_test, browser_channel, binary,
                 package_name, certutil_binary, ca_certificate_path,
                 allow_list_paths):
        self.logger = logger
        self.prefs_root = prefs_root
        self.config = config
        self.test_type = test_type
        self.extra_prefs = extra_prefs
        self.disable_fission = disable_fission
        self.debug_test = debug_test
        self.browser_channel = browser_channel
        self.ca_certificate_path = ca_certificate_path
        self.binary = binary
        self.package_name = package_name
        self.certutil_binary = certutil_binary
        self.ca_certificate_path = ca_certificate_path
        self.allow_list_paths = allow_list_paths

    def create(self, **kwargs):
        """Create a Firefox profile and return the mozprofile Profile object pointing at that
        profile

        :param kwargs: Additional arguments to pass into the profile constructor
        """
        preferences = self._load_prefs()

        profile = FirefoxProfile(preferences=preferences,
                                 restore=False,
                                 allowlistpaths=self.allow_list_paths,
                                 **kwargs)
        self._set_required_prefs(profile)
        if self.ca_certificate_path is not None:
            self._setup_ssl(profile)

        return profile

    def _load_prefs(self):
        prefs = Preferences()

        pref_paths = []

        profiles = os.path.join(self.prefs_root, 'profiles.json')
        if os.path.isfile(profiles):
            with open(profiles) as fh:
                for name in json.load(fh)['web-platform-tests']:
                    if self.browser_channel in (None, 'nightly'):
                        pref_paths.append(os.path.join(self.prefs_root, name, 'user.js'))
                    elif name != 'unittest-features':
                        pref_paths.append(os.path.join(self.prefs_root, name, 'user.js'))
        else:
            self.logger.warning(f"Failed to load profiles from {profiles}")

        for path in pref_paths:
            if os.path.exists(path):
                prefs.add(Preferences.read_prefs(path))
            else:
                self.logger.warning(f"Failed to find prefs file in {path}")

        # Add any custom preferences
        prefs.add(self.extra_prefs, cast=True)

        return prefs()

    def _set_required_prefs(self, profile):
        """Set preferences required for wptrunner to function.

        Note that this doesn't set the marionette port, since we don't always
        know that at profile creation time. So the caller is responisble for
        setting that once it's available."""
        profile.set_preferences({
            "network.dns.localDomains": ",".join(self.config.domains_set),
            "dom.file.createInChild": True,
            # TODO: Remove preferences once Firefox 64 is stable (Bug 905404)
            "network.proxy.type": 0,
            "places.history.enabled": False,
        })

        profile.set_preferences({"fission.autostart": True})
        if self.disable_fission:
            profile.set_preferences({"fission.autostart": False})

        if self.test_type in ("reftest", "print-reftest"):
            profile.set_preferences({"layout.interruptible-reflow.enabled": False})

        if self.test_type == "print-reftest":
            profile.set_preferences({"print.always_print_silent": True})

        if self.test_type == "wdspec":
            profile.set_preferences({"remote.prefs.recommended": True})

        if self.debug_test:
            profile.set_preferences({"devtools.console.stdout.content": True})

    def _setup_ssl(self, profile):
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
        elif mozinfo.isLinux:
            env_var = "LD_LIBRARY_PATH"
        else:
            env_var = "PATH"


        env[env_var] = (os.path.pathsep.join([certutil_dir, env[env_var]])
                        if env_var in env else certutil_dir)

        def certutil(*args):
            cmd = [self.certutil_binary] + list(args)
            self.logger.process_output("certutil",
                                       subprocess.check_output(cmd,
                                                               env=env,
                                                               stderr=subprocess.STDOUT),
                                       " ".join(cmd))

        pw_path = os.path.join(profile.profile, ".crtdbpw")
        with open(pw_path, "w") as f:
            # Use empty password for certificate db
            f.write("\n")

        cert_db_path = profile.profile

        # Create a new certificate db
        certutil("-N", "-d", cert_db_path, "-f", pw_path)

        # Add the CA certificate to the database and mark as trusted to issue server certs
        certutil("-A", "-d", cert_db_path, "-f", pw_path, "-t", "CT,,",
                 "-n", "web-platform-tests", "-i", self.ca_certificate_path)

        # List all certs in the database
        certutil("-L", "-d", cert_db_path)


class FirefoxBrowser(Browser):
    init_timeout = 70

    def __init__(self, logger, binary, package_name, prefs_root, test_type,
                 extra_prefs=None, debug_info=None,
                 symbols_path=None, stackwalk_binary=None, certutil_binary=None,
                 ca_certificate_path=None, e10s=False, disable_fission=False,
                 stackfix_dir=None, binary_args=None, timeout_multiplier=None, leak_check=False,
                 asan=False, chaos_mode_flags=None, config=None,
                 browser_channel="nightly", headless=None, preload_browser=False,
                 specialpowers_path=None, debug_test=False, allow_list_paths=None,
                 gmp_path=None, **kwargs):
        Browser.__init__(self, logger)

        self.logger = logger

        if timeout_multiplier:
            self.init_timeout = self.init_timeout * timeout_multiplier

        self.instance = None
        self._settings = None

        self.stackfix_dir = stackfix_dir
        self.symbols_path = symbols_path
        self.stackwalk_binary = stackwalk_binary

        self.asan = asan
        self.leak_check = leak_check

        self.specialpowers_path = specialpowers_path

        profile_creator = ProfileCreator(logger,
                                         prefs_root,
                                         config,
                                         test_type,
                                         extra_prefs,
                                         disable_fission,
                                         debug_test,
                                         browser_channel,
                                         binary,
                                         package_name,
                                         certutil_binary,
                                         ca_certificate_path,
                                         allow_list_paths)

        if preload_browser:
            instance_manager_cls = PreloadInstanceManager
        else:
            instance_manager_cls = SingleInstanceManager
        self.instance_manager = instance_manager_cls(logger,
                                                     binary,
                                                     binary_args,
                                                     profile_creator,
                                                     debug_info,
                                                     chaos_mode_flags,
                                                     headless,
                                                     leak_check,
                                                     stackfix_dir,
                                                     symbols_path,
                                                     gmp_path,
                                                     asan,
                                                     e10s)

    def settings(self, test):
        self._settings = {"check_leaks": self.leak_check and not test.leaks,
                          "lsan_disabled": test.lsan_disabled,
                          "lsan_allowed": test.lsan_allowed,
                          "lsan_max_stack_depth": test.lsan_max_stack_depth,
                          "mozleak_allowed": self.leak_check and test.mozleak_allowed,
                          "mozleak_thresholds": self.leak_check and test.mozleak_threshold,
                          "special_powers": self.specialpowers_path and test.url_base == "/_mozilla/"}
        return self._settings

    def start(self, group_metadata=None, **kwargs):
        self.instance = self.instance_manager.get()
        self.instance.output_handler.start(group_metadata,
                                           **kwargs)

    def stop(self, force=False):
        self.instance_manager.stop_current(force)
        self.logger.debug("stopped")

    @property
    def pid(self):
        return self.instance.pid()

    def is_alive(self):
        return self.instance and self.instance.is_alive()

    def cleanup(self, force=False):
        self.instance_manager.teardown(force)

    def executor_browser(self):
        assert self.instance is not None
        extensions = []
        if self._settings.get("special_powers", False):
            extensions.append(self.specialpowers_path)
        return ExecutorBrowser, {"marionette_port": self.instance.marionette_port,
                                 "extensions": extensions,
                                 "supports_devtools": True}

    def check_crash(self, process, test):
        return log_gecko_crashes(self.logger,
                                 process,
                                 test,
                                 self.instance.runner.profile.profile,
                                 self.symbols_path,
                                 self.stackwalk_binary)


class FirefoxWdSpecBrowser(WebDriverBrowser):
    def __init__(self, logger, binary, package_name, prefs_root, webdriver_binary, webdriver_args,
                 extra_prefs=None, debug_info=None, symbols_path=None, stackwalk_binary=None,
                 certutil_binary=None, ca_certificate_path=None, e10s=False,
                 disable_fission=False, stackfix_dir=None, leak_check=False,
                 asan=False, chaos_mode_flags=None, config=None, browser_channel="nightly",
                 headless=None, debug_test=False, profile_creator_cls=ProfileCreator,
                 allow_list_paths=None, gmp_path=None, **kwargs):

        super().__init__(logger, binary, webdriver_binary, webdriver_args)
        self.binary = binary
        self.package_name = package_name
        self.webdriver_binary = webdriver_binary

        self.stackfix_dir = stackfix_dir
        self.symbols_path = symbols_path
        self.stackwalk_binary = stackwalk_binary

        self.asan = asan
        self.leak_check = leak_check
        self.leak_report_file = None

        self.env = self.get_env(binary, debug_info, headless, gmp_path, chaos_mode_flags, e10s)

        profile_creator = profile_creator_cls(logger,
                                              prefs_root,
                                              config,
                                              "wdspec",
                                              extra_prefs,
                                              disable_fission,
                                              debug_test,
                                              browser_channel,
                                              binary,
                                              package_name,
                                              certutil_binary,
                                              ca_certificate_path,
                                              allow_list_paths)

        self.profile = profile_creator.create()
        self.marionette_port = None

    def get_env(self, binary, debug_info, headless, gmp_path, chaos_mode_flags, e10s):
        env = get_environ(self.logger,
                          binary,
                          debug_info,
                          headless,
                          gmp_path,
                          chaos_mode_flags, e10s)
        env["RUST_BACKTRACE"] = "1"
        return env

    def create_output_handler(self, cmd):
        return GeckodriverOutputHandler(self.logger,
                                        cmd,
                                        stackfix_dir=self.stackfix_dir,
                                        symbols_path=self.symbols_path,
                                        asan=self.asan,
                                        leak_report_file=self.leak_report_file,
                                        init_deadline=self.init_deadline)

    def start(self, group_metadata, **kwargs):
        self.leak_report_file = setup_leak_report(self.leak_check, self.profile, self.env)
        super().start(group_metadata, **kwargs)

    def stop(self, force=False):
        # Initially wait for any WebDriver session to cleanly shutdown if the
        # process doesn't have to be force stopped.
        # When this is called the executor is usually sending an end session
        # command to the browser. We don't have a synchronisation mechanism
        # that allows us to know that process is ongoing, so poll the status
        # endpoint until there isn't a session, before killing the driver.
        if self.is_alive() and not force:
            end_time = time.time() + BrowserInstance.shutdown_timeout
            while time.time() < end_time:
                self.logger.debug("Waiting for WebDriver session to end")
                try:
                    self.logger.debug(f"Connecting to http://{self.host}:{self.port}/status")
                    conn = HTTPConnection(self.host, self.port)
                    conn.request("GET", "/status")
                    res = conn.getresponse()
                    self.logger.debug(f"Got response from http://{self.host}:{self.port}/status")
                except Exception:
                    self.logger.debug(
                        f"Connecting to http://{self.host}:{self.port}/status failed")
                    break
                if res.status != 200:
                    self.logger.debug(f"Connecting to http://{self.host}:{self.port}/status "
                                      f"gave status {res.status}")
                    break
                data = res.read()
                try:
                    msg = json.loads(data)
                except ValueError:
                    self.logger.debug("/status response was not valid JSON")
                    break
                if msg.get("value", {}).get("ready") is True:
                    self.logger.debug("Got ready status")
                    break
                self.logger.debug(f"Got status response {data}")
                time.sleep(1)
            else:
                self.logger.debug("WebDriver session didn't end")
        try:
            super().stop(force=force)
        finally:
            if self._output_handler is not None:
                self._output_handler.port = None
            self._port = None

    def cleanup(self):
        super().cleanup()
        self.profile.cleanup()

    def settings(self, test):
        return {"check_leaks": self.leak_check and not test.leaks,
                "lsan_disabled": test.lsan_disabled,
                "lsan_allowed": test.lsan_allowed,
                "lsan_max_stack_depth": test.lsan_max_stack_depth,
                "mozleak_allowed": self.leak_check and test.mozleak_allowed,
                "mozleak_thresholds": self.leak_check and test.mozleak_threshold}

    @property
    def port(self):
        # We read the port from geckodriver on startup
        if self._port is None:
            if self._output_handler is None or self._output_handler.port is None:
                raise ValueError("Can't get geckodriver port before it's started")
            self._port = self._output_handler.port
        return self._port

    def make_command(self):
        return [self.webdriver_binary,
                "--host", self.host,
                "--port", "0"] + self.webdriver_args

    def executor_browser(self):
        cls, args = super().executor_browser()
        args["supports_devtools"] = False
        args["profile"] = self.profile.profile
        return cls, args

    def check_crash(self, process, test):
        return log_gecko_crashes(self.logger,
                                 process,
                                 test,
                                 self.profile.profile,
                                 self.symbols_path,
                                 self.stackwalk_binary)
