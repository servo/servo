import enum
import os
import platform
import socket
from abc import ABCMeta, abstractmethod

from ..wptcommandline import require_arg  # noqa: F401

here = os.path.dirname(__file__)


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
        except OSError:
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


class ExecutorBrowser:
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


@enum.unique
class OutputHandlerState(enum.IntEnum):
    BEFORE_PROCESS_START = 1
    AFTER_PROCESS_START = 2
    AFTER_HANDLER_START = 3
    AFTER_PROCESS_STOP = 4


class OutputHandler:
    """Class for handling output from a browser process.

    This class is responsible for consuming the logging from a browser process
    and passing it into the relevant logger. A class instance is designed to
    be passed as the processOutputLine argument to mozprocess.ProcessHandler.

    The setup of this class is complex for various reasons:

    * We need to create an instance of the class before starting the process
    * We want access to data about the running process e.g. the pid
    * We want to launch the process and later setup additional log handling
      which is restrospectively applied to any existing output (this supports
      prelaunching browsers for performance, but having log output depend on the
      tests that are run e.g. for leak suppression).

    Therefore the lifecycle is as follows::

      output_handler = OutputHandler(logger, command, **output_handler_kwargs)
      proc = ProcessHandler(command, ..., processOutputLine=output_handler)
      output_handler.after_process_start(proc.pid)
      [...]
      # All logging to this point was buffered in-memory, but after start()
      # it's actually sent to the logger.
      output_handler.start(**output_logger_start_kwargs)
      [...]
      proc.wait()
      output_handler.after_process_stop()

    Since the process lifetime and the output handler lifetime are coupled (it doesn't
    work to reuse an output handler for multiple processes), it might make sense to have
    a single class that owns the process and the output processing for the process.
    This is complicated by the fact that we don't always run the process directly,
    but sometimes use a wrapper e.g. mozrunner.
    """

    def __init__(self, logger, command, **kwargs):
        self.logger = logger
        self.command = command
        self.pid = None
        self.state = OutputHandlerState.BEFORE_PROCESS_START
        self.line_buffer = []

    def after_process_start(self, pid):
        assert self.state == OutputHandlerState.BEFORE_PROCESS_START
        self.logger.debug("OutputHandler.after_process_start")
        self.pid = pid
        self.state = OutputHandlerState.AFTER_PROCESS_START

    def start(self, **kwargs):
        assert self.state == OutputHandlerState.AFTER_PROCESS_START
        self.logger.debug("OutputHandler.start")
        # Need to change the state here before we try to empty the buffer
        # or we'll just re-buffer the existing output.
        self.state = OutputHandlerState.AFTER_HANDLER_START
        for item in self.line_buffer:
            self(item)
        self.line_buffer = None

    def after_process_stop(self, clean_shutdown=True):
        # If we didn't get as far as configure, just
        # dump all logs with no configuration
        self.logger.debug("OutputHandler.after_process_stop")
        if self.state < OutputHandlerState.AFTER_HANDLER_START:
            self.start()
        self.state = OutputHandlerState.AFTER_PROCESS_STOP

    def __call__(self, line):
        if self.state < OutputHandlerState.AFTER_HANDLER_START:
            self.line_buffer.append(line)
            return

        # Could assert that there's no output handled once we're in the
        # after_process_stop phase, although technically there's a race condition
        # here because we don't know the logging thread has finished draining the
        # logs. The solution might be to move this into mozprocess itself.

        self.logger.process_output(self.pid,
                                   line.decode("utf8", "replace"),
                                   command=" ".join(self.command) if self.command else "")
