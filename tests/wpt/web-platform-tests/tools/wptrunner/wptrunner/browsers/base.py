import os
import platform
import socket
from abc import ABCMeta, abstractmethod
from copy import deepcopy

from ..wptcommandline import require_arg  # noqa: F401

here = os.path.dirname(__file__)


def inherit(super_module, child_globals, product_name):
    super_wptrunner = super_module.__wptrunner__
    child_globals["__wptrunner__"] = child_wptrunner = deepcopy(super_wptrunner)

    child_wptrunner["product"] = product_name

    for k in ("check_args", "browser", "browser_kwargs", "executor_kwargs",
              "env_extras", "env_options", "timeout_multiplier"):
        attr = super_wptrunner[k]
        child_globals[attr] = getattr(super_module, attr)

    for v in super_module.__wptrunner__["executor"].values():
        child_globals[v] = getattr(super_module, v)

    if "run_info_extras" in super_wptrunner:
        attr = super_wptrunner["run_info_extras"]
        child_globals[attr] = getattr(super_module, attr)


def cmd_arg(name, value=None):
    prefix = "-" if platform.system() == "Windows" else "--"
    rv = prefix + name
    if value is not None:
        rv += "=" + value
    return rv


def maybe_add_args(required_args, current_args):
    for required_arg in required_args:
        # If the arg is in the form of "variable=value", only add it if
        # no arg with another value for "variable" is already there.
        if "=" in required_arg:
            required_arg_prefix = "%s=" % required_arg.split("=")[0]
            if not any(item.startswith(required_arg_prefix) for item in current_args):
                current_args.append(required_arg)
        else:
            if required_arg not in current_args:
                current_args.append(required_arg)
    return current_args


def certificate_domain_list(list_of_domains, certificate_file):
    """Build a list of domains where certificate_file should be used"""
    cert_list = []
    for domain in list_of_domains:
        cert_list.append({"host": domain, "certificateFile": certificate_file})
    return cert_list


def get_free_port():
    """Get a random unbound port"""
    while True:
        s = socket.socket()
        try:
            s.bind(("127.0.0.1", 0))
        except socket.error:
            continue
        else:
            return s.getsockname()[1]
        finally:
            s.close()


def get_timeout_multiplier(test_type, run_info_data, **kwargs):
    if kwargs["timeout_multiplier"] is not None:
        return kwargs["timeout_multiplier"]
    return 1


def browser_command(binary, args, debug_info):
    if debug_info:
        if debug_info.requiresEscapedArgs:
            args = [item.replace("&", "\\&") for item in args]
        debug_args = [debug_info.path] + debug_info.args
    else:
        debug_args = []

    command = [binary] + args

    return debug_args, command


class BrowserError(Exception):
    pass


class Browser(object):
    """Abstract class serving as the basis for Browser implementations.

    The Browser is used in the TestRunnerManager to start and stop the browser
    process, and to check the state of that process. This class also acts as a
    context manager, enabling it to do browser-specific setup at the start of
    the testrun and cleanup after the run is complete.

    :param logger: Structured logger to use for output.
    """
    __metaclass__ = ABCMeta

    process_cls = None
    init_timeout = 30

    def __init__(self, logger):
        self.logger = logger

    def __enter__(self):
        self.setup()
        return self

    def __exit__(self, *args, **kwargs):
        self.cleanup()

    def setup(self):
        """Used for browser-specific setup that happens at the start of a test run"""
        pass

    def settings(self, test):
        """Dictionary of metadata that is constant for a specific launch of a browser.

        This is used to determine when the browser instance configuration changes, requiring
        a relaunch of the browser. The test runner calls this method for each test, and if the
        returned value differs from that for the previous test, the browser is relaunched.
        """
        return {}

    @abstractmethod
    def start(self, group_metadata, **kwargs):
        """Launch the browser object and get it into a state where is is ready to run tests"""
        pass

    @abstractmethod
    def stop(self, force=False):
        """Stop the running browser process."""
        pass

    @abstractmethod
    def pid(self):
        """pid of the browser process or None if there is no pid"""
        pass

    @abstractmethod
    def is_alive(self):
        """Boolean indicating whether the browser process is still running"""
        pass

    def cleanup(self):
        """Browser-specific cleanup that is run after the testrun is finished"""
        pass

    def executor_browser(self):
        """Returns the ExecutorBrowser subclass for this Browser subclass and the keyword arguments
        with which it should be instantiated"""
        return ExecutorBrowser, {}

    def maybe_parse_tombstone(self):
        """Possibly parse tombstones on Android device for Android target"""
        pass

    def check_crash(self, process, test):
        """Check if a crash occured and output any useful information to the
        log. Returns a boolean indicating whether a crash occured."""
        return False


class NullBrowser(Browser):
    def __init__(self, logger, **kwargs):
        super(NullBrowser, self).__init__(logger)

    def start(self, **kwargs):
        """No-op browser to use in scenarios where the TestRunnerManager shouldn't
        actually own the browser process (e.g. Servo where we start one browser
        per test)"""
        pass

    def stop(self, force=False):
        pass

    def pid(self):
        return None

    def is_alive(self):
        return True

    def on_output(self, line):
        raise NotImplementedError


class ExecutorBrowser(object):
    """View of the Browser used by the Executor object.
    This is needed because the Executor runs in a child process and
    we can't ship Browser instances between processes on Windows.

    Typically this will have a few product-specific properties set,
    but in some cases it may have more elaborate methods for setting
    up the browser from the runner process.
    """
    def __init__(self, **kwargs):
        for k, v in kwargs.items():
            setattr(self, k, v)
