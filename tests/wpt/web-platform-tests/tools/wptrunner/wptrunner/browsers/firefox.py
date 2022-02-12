import base64
import io
import json
import os
import platform
import signal
import subprocess
import tempfile
import zipfile
from abc import ABCMeta, abstractmethod

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
                   NullBrowser,
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
from ..webdriver_server import WebDriverServer


here = os.path.dirname(__file__)

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
    if test_type == "reftest":
        if run_info_data["debug"] or run_info_data.get("asan") or run_info_data.get("tsan"):
            return 4
        else:
            return 2
    elif run_info_data["debug"] or run_info_data.get("asan") or run_info_data.get("tsan"):
        if run_info_data.get("ccov"):
            return 4
        else:
            return 3
    elif run_info_data["os"] == "android":
        return 4
    # https://bugzilla.mozilla.org/show_bug.cgi?id=1538725
    elif run_info_data["os"] == "win" and run_info_data["processor"] == "aarch64":
        return 4
    elif run_info_data.get("ccov"):
        return 2
    return 1


def check_args(**kwargs):
    require_arg(kwargs, "binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
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
            "enable_fission": kwargs["enable_fission"],
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
            "headless": kwargs["headless"],
            "preload_browser": kwargs["preload_browser"] and not kwargs["pause_after_test"] and not kwargs["num_test_groups"] == 1,
            "specialpowers_path": kwargs["specialpowers_path"]}


class WdSpecProfile(object):
    def __init__(self, profile):
        self.profile = profile

    def __enter__(self):
        return self

    def __exit__(self, *args, **kwargs):
        self.profile.cleanup()


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
        executor_kwargs["reftest_screenshot"] = kwargs["reftest_screenshot"]
    if test_type == "wdspec":
        options = {"args": []}
        if kwargs["binary"]:
            options["binary"] = kwargs["binary"]
        if kwargs["binary_args"]:
            options["args"] = kwargs["binary_args"]

        profile_creator = ProfileCreator(logger,
                                         kwargs["prefs_root"],
                                         test_environment.config,
                                         test_type,
                                         kwargs["extra_prefs"],
                                         kwargs["gecko_e10s"],
                                         kwargs["enable_fission"],
                                         kwargs["browser_channel"],
                                         kwargs["binary"],
                                         kwargs["certutil_binary"],
                                         test_environment.config.ssl_config["ca_cert_path"])
        if kwargs["processes"] > 1:
            # With multiple processes, we would need a profile directory per process, but we
            # don't have an easy way to do that, so include the profile in the capabilties
            # directly instead. This means recreating it per session, which is slow
            options["profile"] = profile_creator.create_base64()
            profile = None
        else:
            profile = profile_creator.create()
            options["args"].extend(["--profile", profile.profile])
            test_environment.env_extras_cms.append(WdSpecProfile(profile))

        capabilities["moz:firefoxOptions"] = options

        # This gets reused for firefox_android, but the environment setup
        # isn't required in that case
        if kwargs["binary"]:
            environ = get_environ(logger,
                                  kwargs["binary"],
                                  kwargs["debug_info"],
                                  kwargs["stylo_threads"],
                                  kwargs["headless"],
                                  kwargs["enable_webrender"],
                                  kwargs["chaos_mode_flags"])
            leak_report_file = setup_leak_report(kwargs["leak_check"], profile, environ)

            # This doesn't work with wdspec tests
            # In particular tests can create a session without passing in the capabilites
            # and in those cases we get the default geckodriver profile which doesn't
            # guarantee zero network access
            del environ["MOZ_DISABLE_NONLOCAL_CONNECTIONS"]
            executor_kwargs["environ"] = environ
        else:
            if kwargs["headless"] and "--headless" not in options["args"]:
                options["args"].append("--headless")
            leak_report_file = None

        executor_kwargs["stackfix_dir"] = kwargs["stackfix_dir"],
        executor_kwargs["leak_report_file"] = leak_report_file
        executor_kwargs["asan"] = run_info_data.get("asan")

    if kwargs["certutil_binary"] is None:
        capabilities["acceptInsecureCerts"] = True
    if capabilities:
        executor_kwargs["capabilities"] = capabilities
    executor_kwargs["debug"] = run_info_data["debug"]
    executor_kwargs["ccov"] = run_info_data.get("ccov", False)
    executor_kwargs["browser_version"] = run_info_data.get("browser_version")
    executor_kwargs["debug_test"] = kwargs["debug_test"]
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
          "fission": kwargs.get("enable_fission") or get_bool_pref("fission.autostart"),
          "sessionHistoryInParent": (kwargs.get("enable_fission") or
                                     get_bool_pref("fission.autostart") or
                                     get_bool_pref("fission.sessionHistoryInParent")),
          "swgl": get_bool_pref("gfx.webrender.software")}

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
    return (["os", "debug", "webrender", "fission", "e10s", "processor", "swgl"],
            {"os": ["version"], "processor": ["bits"]})


def get_environ(logger, binary, debug_info, stylo_threads, headless, enable_webrender,
                chaos_mode_flags=None):
    env = test_environment(xrePath=os.path.abspath(os.path.dirname(binary)),
                           debugger=debug_info is not None,
                           useLSan=True,
                           log=logger)

    env["STYLO_THREADS"] = str(stylo_threads)
    # Disable window occlusion. Bug 1733955
    env["MOZ_WINDOW_OCCLUSION"] = "0"
    if chaos_mode_flags is not None:
        env["MOZ_CHAOSMODE"] = str(chaos_mode_flags)
    if headless:
        env["MOZ_HEADLESS"] = "1"
    if enable_webrender:
        env["MOZ_WEBRENDER"] = "1"
        env["MOZ_ACCELERATED"] = "1"
    else:
        env["MOZ_WEBRENDER"] = "0"
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
                 chaos_mode_flags, headless, enable_webrender, stylo_threads,
                 leak_check, stackfix_dir, symbols_path, asan):
        """Object that manages starting and stopping instances of Firefox."""
        self.logger = logger
        self.binary = binary
        self.binary_args = binary_args
        self.base_profile = profile_creator.create()
        self.debug_info = debug_info
        self.chaos_mode_flags = chaos_mode_flags
        self.headless = headless
        self.enable_webrender = enable_webrender
        self.stylo_threads = stylo_threads
        self.leak_check = leak_check
        self.stackfix_dir = stackfix_dir
        self.symbols_path = symbols_path
        self.asan = asan

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

        env = get_environ(self.logger, self.binary, self.debug_info, self.stylo_threads,
                          self.headless, self.enable_webrender, self.chaos_mode_flags)

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
        super(PreloadInstanceManager, self).__init__(*args, **kwargs)
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
            if not clean_shutdown:
                # If we didn't get a clean shutdown there probably isn't a leak report file
                self.logger.warning("Firefox didn't exit cleanly, not processing leak logs")
            else:
                # We have to ignore missing leaks in the tab because it can happen that the
                # content process crashed and in that case we don't want the test to fail.
                # Ideally we would record which content process crashed and just skip those.
                self.logger.info("PROCESS LEAKS %s" % self.leak_report_file)
                mozleak.process_leak_log(
                    self.leak_report_file,
                    leak_thresholds=self.mozleak_thresholds,
                    ignore_missing_leaks=["tab", "gmplugin"],
                    log=self.logger,
                    stack_fixer=self.stack_fixer,
                    scope=self.group_metadata.get("scope"),
                    allowed=self.mozleak_allowed)
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


class ProfileCreator:
    def __init__(self, logger, prefs_root, config, test_type, extra_prefs, e10s,
                 enable_fission, browser_channel, binary, certutil_binary, ca_certificate_path):
        self.logger = logger
        self.prefs_root = prefs_root
        self.config = config
        self.test_type = test_type
        self.extra_prefs = extra_prefs
        self.e10s = e10s
        self.enable_fission = enable_fission
        self.browser_channel = browser_channel
        self.ca_certificate_path = ca_certificate_path
        self.binary = binary
        self.certutil_binary = certutil_binary
        self.ca_certificate_path = ca_certificate_path

    def create(self, **kwargs):
        """Create a Firefox profile and return the mozprofile Profile object pointing at that
        profile

        :param kwargs: Additional arguments to pass into the profile constructor
        """
        preferences = self._load_prefs()

        profile = FirefoxProfile(preferences=preferences,
                                 restore=False,
                                 **kwargs)
        self._set_required_prefs(profile)
        if self.ca_certificate_path is not None:
            self._setup_ssl(profile)

        return profile

    def create_base64(self, **kwargs):
        profile = self.create(**kwargs)
        try:
            with io.BytesIO() as buf:
                with zipfile.ZipFile(buf, "w", compression=zipfile.ZIP_DEFLATED) as zipf:
                    for dirpath, _, filenames in os.walk(profile.profile):
                        for filename in filenames:
                            src_path = os.path.join(dirpath, filename)
                            dest_path = os.path.relpath(src_path, profile.profile)
                            with open(src_path, "rb") as f:
                                zipf.writestr(dest_path, f.read())
                return base64.b64encode(buf.getvalue()).decode("ascii").strip()
        finally:
            profile.cleanup()

    def _load_prefs(self):
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
            "network.preload": True,
        })
        if self.e10s:
            profile.set_preferences({"browser.tabs.remote.autostart": True})

        if self.enable_fission:
            profile.set_preferences({"fission.autostart": True})

        if self.test_type in ("reftest", "print-reftest"):
            profile.set_preferences({"layout.interruptible-reflow.enabled": False})

        if self.test_type == "print-reftest":
            profile.set_preferences({"print.always_print_silent": True})

        # Bug 1262954: winxp + e10s, disable hwaccel
        if (self.e10s and platform.system() in ("Windows", "Microsoft") and
            "5.1" in platform.version()):
            self.profile.set_preferences({"layers.acceleration.disabled": True})

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
        elif mozinfo.isUnix:
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

    def __init__(self, logger, binary, prefs_root, test_type, extra_prefs=None, debug_info=None,
                 symbols_path=None, stackwalk_binary=None, certutil_binary=None,
                 ca_certificate_path=None, e10s=False, enable_webrender=False, enable_fission=False,
                 stackfix_dir=None, binary_args=None, timeout_multiplier=None, leak_check=False,
                 asan=False, stylo_threads=1, chaos_mode_flags=None, config=None,
                 browser_channel="nightly", headless=None, preload_browser=False,
                 specialpowers_path=None, **kwargs):
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
                                         e10s,
                                         enable_fission,
                                         browser_channel,
                                         binary,
                                         certutil_binary,
                                         ca_certificate_path)

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
                                                     enable_webrender,
                                                     stylo_threads,
                                                     leak_check,
                                                     stackfix_dir,
                                                     symbols_path,
                                                     asan)

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
                                 "extensions": extensions}

    def check_crash(self, process, test):
        dump_dir = os.path.join(self.instance.runner.profile.profile, "minidumps")

        try:
            return bool(mozcrash.log_crashes(self.logger,
                                             dump_dir,
                                             symbols_path=self.symbols_path,
                                             stackwalk_binary=self.stackwalk_binary,
                                             process=process,
                                             test=test))
        except IOError:
            self.logger.warning("Looking for crash dump files failed")
            return False


class FirefoxWdSpecBrowser(NullBrowser):
    def __init__(self, logger, leak_check=False, **kwargs):
        super().__init__(logger, **kwargs)
        self.leak_check = leak_check

    def settings(self, test):
        return {"check_leaks": self.leak_check and not test.leaks,
                "lsan_disabled": test.lsan_disabled,
                "lsan_allowed": test.lsan_allowed,
                "lsan_max_stack_depth": test.lsan_max_stack_depth,
                "mozleak_allowed": self.leak_check and test.mozleak_allowed,
                "mozleak_thresholds": self.leak_check and test.mozleak_threshold}


class GeckoDriverServer(WebDriverServer):
    output_handler_cls = FirefoxOutputHandler

    def __init__(self, logger, marionette_port=2828, binary="geckodriver",
                 host="127.0.0.1", port=None, env=None, args=None):
        if env is None:
            env = os.environ.copy()
        env["RUST_BACKTRACE"] = "1"
        WebDriverServer.__init__(self, logger, binary,
                                 host=host,
                                 port=port,
                                 env=env,
                                 args=args)
        self.marionette_port = marionette_port

    def make_command(self):
        return [self.binary,
                "--marionette-port", str(self.marionette_port),
                "--host", self.host,
                "--port", str(self.port)] + self._args
