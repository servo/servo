import json
import os
import platform
import signal
import subprocess
import sys
from abc import ABCMeta, abstractmethod

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
                                            MarionetteWdspecExecutor,  # noqa: F401
                                            MarionetteCrashtestExecutor)  # noqa: F401
from ..process import cast_env


here = os.path.join(os.path.split(__file__)[0])

__wptrunner__ = {"product": "firefox",
                 "check_args": "check_args",
                 "browser": "FirefoxBrowser",
                 "executor": {"crashtest": "MarionetteCrashtestExecutor",
                              "testharness": "MarionetteTestharnessExecutor",
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
    elif run_info_data.get("ccov"):
        return 2
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
            "preload_browser": kwargs["preload_browser"]}


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
          "sw-e10s": True,
          "fission": kwargs.get("enable_fission") or get_bool_pref("fission.autostart")}

    # The value of `sw-e10s` defaults to whether the "parent_intercept"
    # implementation is enabled for the current build. This value, however,
    # can be overridden by explicitly setting the pref with the `--setpref` CLI
    # flag, which is checked here.
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


class FirefoxInstanceManager(object):
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

        args = self.binary_args[:] if self.binary_args else []
        args += [cmd_arg("marionette"), "about:blank"]

        debug_args, cmd = browser_command(self.binary,
                                          args,
                                          self.debug_info)

        if self.leak_check:
            leak_report_file = os.path.join(profile.profile, "runtests_leaks_%s.log" % os.getpid())
            if os.path.exists(leak_report_file):
                os.remove(leak_report_file)
            env["XPCOM_MEM_BLOAT_LOG"] = leak_report_file
        else:
            leak_report_file = None

        output_handler = OutputHandler(self.logger, self.stackfix_dir, self.symbols_path, self.asan)
        runner = FirefoxRunner(profile=profile,
                               binary=cmd[0],
                               cmdargs=cmd[1:],
                               env=cast_env(env),
                               process_class=ProcessHandler,
                               process_args={"processOutputLine": [output_handler]})
        instance = BrowserInstance(self.logger, runner, marionette_port,
                                   output_handler, leak_report_file)

        self.logger.debug("Starting Firefox")
        runner.start(debug_args=debug_args, interactive=self.debug_info and self.debug_info.interactive)
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
        for instance, skip_marionette in [(self.previous, False),
                                          (self.current, False),
                                          (self.pending, True)]:
            if instance:
                instance.stop(force, skip_marionette=skip_marionette)
                instance.cleanup()


class BrowserInstance(object):
    shutdown_timeout = 70

    def __init__(self, logger, runner, marionette_port, output_handler, leak_report_file):
        """Handle to a running Firefox instance"""
        self.logger = logger
        self.runner = runner
        self.marionette_port = marionette_port
        self.output_handler = output_handler
        self.leak_report_file = leak_report_file

    def stop(self, force=False, skip_marionette=False):
        """Stop Firefox"""
        if self.runner is not None and self.runner.is_running():
            self.logger.debug("Stopping Firefox %s" % self.pid())
            shutdown_methods = [(True, lambda: self.runner.wait(self.shutdown_timeout)),
                                (False, lambda: self.runner.stop(signal.SIGTERM)),
                                (False, lambda: self.runner.stop(signal.SIGKILL))]
            if skip_marionette:
                shutdown_methods = shutdown_methods[1:]
            try:
                # For Firefox we assume that stopping the runner prompts the
                # browser to shut down. This allows the leak log to be written
                for clean, stop_f in shutdown_methods:
                    if not force or not clean:
                        retcode = stop_f()
                        if retcode is not None:
                            self.logger.info("Browser exited with return code %s" % retcode)
                            break
            except OSError:
                # This can happen on Windows if the process is already dead
                pass
        if not skip_marionette:
            self.output_handler.after_stop()

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
        # mozprofile handles deleting the profile when the refcount reaches 0
        self.runner = None


class OutputHandler(object):
    def __init__(self, logger, stackfix_dir, symbols_path, asan):
        """Filter for handling Firefox process output.

        This receives Firefox process output in the __call__ function, does
        any additional processing that's required, and decides whether to log
        the output. Because the Firefox process can be started before we know
        which filters are going to be required, we buffer all output until
        setup() is called. This is responsible for doing the final configuration
        of the output handlers.
        """

        self.logger = logger
        # These are filled in after setup() is called
        self.instance = None

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

        self.lsan_handler = None
        self.mozleak_allowed = None
        self.mozleak_thresholds = None
        self.group_metadata = {}

        self.line_buffer = []
        self.setup_ran = False

    def setup(self, instance=None, group_metadata=None, lsan_disabled=False,
              lsan_allowed=None, lsan_max_stack_depth=None, mozleak_allowed=None,
              mozleak_thresholds=None, **kwargs):
        """Configure the output handler"""
        self.instance = instance

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

        self.setup_ran = True

        for line in self.line_buffer:
            self.__call__(line)
        self.line_buffer = []

    def after_stop(self):
        self.logger.info("PROCESS LEAKS %s" % self.instance.leak_report_file)
        if self.lsan_handler:
            self.lsan_handler.process()
        if self.instance.leak_report_file is not None:
            # We have to ignore missing leaks in the tab because it can happen that the
            # content process crashed and in that case we don't want the test to fail.
            # Ideally we would record which content process crashed and just skip those.
            mozleak.process_leak_log(
                self.instance.leak_report_file,
                leak_thresholds=self.mozleak_thresholds,
                ignore_missing_leaks=["tab", "gmplugin"],
                log=self.logger,
                stack_fixer=self.stack_fixer,
                scope=self.group_metadata.get("scope"),
                allowed=self.mozleak_allowed)

    def __call__(self, line):
        """Write a line of output from the firefox process to the log"""
        if b"GLib-GObject-CRITICAL" in line:
            return
        if line:
            if not self.setup_ran:
                self.line_buffer.append(line)
                return
            data = line.decode("utf8", "replace")
            if self.stack_fixer:
                data = self.stack_fixer(data)
            if self.lsan_handler:
                data = self.lsan_handler.log(data)
            if data is not None:
                self.logger.process_output(self.instance and
                                           self.instance.runner.process_handler and
                                           self.instance.runner.process_handler.pid,
                                           data,
                                           command=" ".join(self.instance.runner.command))


class ProfileCreator(object):
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

    def create(self):
        """Create a Firefox profile and return the mozprofile Profile object pointing at that
        profile"""
        preferences = self._load_prefs()

        profile = FirefoxProfile(preferences=preferences)
        self._set_required_prefs(profile)
        if self.ca_certificate_path is not None:
            self._setup_ssl(profile)

        return profile

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

        if self.test_type == "reftest":
            profile.set_preferences({"layout.interruptible-reflow.enabled": False})

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
                        if env_var in env else certutil_dir).encode(
                            sys.getfilesystemencoding() or 'utf-8', 'replace')

        def certutil(*args):
            cmd = [self.certutil_binary] + list(args)
            self.logger.process_output("certutil",
                                       subprocess.check_output(cmd,
                                                               env=cast_env(env),
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
                 browser_channel="nightly", headless=None, preload_browser=False, **kwargs):
        Browser.__init__(self, logger)

        self.logger = logger

        if timeout_multiplier:
            self.init_timeout = self.init_timeout * timeout_multiplier

        self.instance = None

        self.stackfix_dir = stackfix_dir
        self.symbols_path = symbols_path
        self.stackwalk_binary = stackwalk_binary

        self.asan = asan
        self.leak_check = leak_check

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
        return {"check_leaks": self.leak_check and not test.leaks,
                "lsan_disabled": test.lsan_disabled,
                "lsan_allowed": test.lsan_allowed,
                "lsan_max_stack_depth": test.lsan_max_stack_depth,
                "mozleak_allowed": self.leak_check and test.mozleak_allowed,
                "mozleak_thresholds": self.leak_check and test.mozleak_threshold}

    def start(self, group_metadata=None, **kwargs):
        self.instance = self.instance_manager.get()
        self.instance.output_handler.setup(self.instance,
                                           group_metadata,
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
        return ExecutorBrowser, {"marionette_port": self.instance.marionette_port}

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
