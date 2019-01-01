import os
import platform
import socket
from abc import ABCMeta, abstractmethod
from copy import deepcopy

from ..wptcommandline import require_arg  # noqa: F401

here = os.path.split(__file__)[0]


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


def get_free_port(start_port, exclude=None):
    """Get the first port number after start_port (inclusive) that is
    not currently bound.

    :param start_port: Integer port number at which to start testing.
    :param exclude: Set of port numbers to skip"""
    port = start_port
    while True:
        if exclude and port in exclude:
            port += 1
            continue
        s = socket.socket()
        try:
            s.bind(("127.0.0.1", port))
        except socket.error:
            port += 1
        else:
            return port
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
    __metaclass__ = ABCMeta

    process_cls = None
    init_timeout = 30

    def __init__(self, logger):
        """Abstract class serving as the basis for Browser implementations.

        The Browser is used in the TestRunnerManager to start and stop the browser
        process, and to check the state of that process. This class also acts as a
        context manager, enabling it to do browser-specific setup at the start of
        the testrun and cleanup after the run is complete.

        :param logger: Structured logger to use for output.
        """
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

    def setup_ssl(self, hosts):
        """Return a certificate to use for tests requiring ssl that will be trusted by the browser"""
        raise NotImplementedError("ssl testing not supported")

    def cleanup(self):
        """Browser-specific cleanup that is run after the testrun is finished"""
        pass

    def executor_browser(self):
        """Returns the ExecutorBrowser subclass for this Browser subclass and the keyword arguments
        with which it should be instantiated"""
        return ExecutorBrowser, {}

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
    def __init__(self, **kwargs):
        """View of the Browser used by the Executor object.
        This is needed because the Executor runs in a child process and
        we can't ship Browser instances between processes on Windows.

        Typically this will have a few product-specific properties set,
        but in some cases it may have more elaborate methods for setting
        up the browser from the runner process.
        """
        for k, v in kwargs.iteritems():
            setattr(self, k, v)
