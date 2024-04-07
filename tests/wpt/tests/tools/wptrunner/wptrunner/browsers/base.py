import enum
import errno
import os
import platform
import socket
import time
import traceback
from abc import ABCMeta, abstractmethod
from typing import cast, Any, List, Mapping, Optional, Tuple, Type

import mozprocess
from mozdebug import DebuggerInfo
from mozlog.structuredlog import StructuredLogger

from ..environment import wait_for_service
from ..testloader import GroupMetadata
from ..wptcommandline import require_arg  # noqa: F401
from ..wpttest import Test

here = os.path.dirname(__file__)


def cmd_arg(name: str, value: Optional[str] = None) -> str:
    prefix = "-" if platform.system() == "Windows" else "--"
    rv = prefix + name
    if value is not None:
        rv += "=" + value
    return rv


def maybe_add_args(required_args: List[str], current_args: List[str]) -> List[str]:
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


def certificate_domain_list(list_of_domains: List[str],
                            certificate_file: str) -> List[Mapping[str, Any]]:
    """Build a list of domains where certificate_file should be used"""
    cert_list: List[Mapping[str, Any]] = []
    for domain in list_of_domains:
        cert_list.append({"host": domain, "certificateFile": certificate_file})
    return cert_list


def get_free_port() -> int:
    """Get a random unbound port"""
    while True:
        s = socket.socket()
        try:
            s.bind(("127.0.0.1", 0))
        except OSError:
            continue
        else:
            return cast(int, s.getsockname()[1])
        finally:
            s.close()


def get_timeout_multiplier(test_type: str, run_info_data: Mapping[str, Any], **kwargs: Any) -> float:
    if kwargs["timeout_multiplier"] is not None:
        return cast(float, kwargs["timeout_multiplier"])
    return 1


def browser_command(binary: str,
                    args: List[str],
                    debug_info: DebuggerInfo) -> Tuple[List[str], List[str]]:
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


BrowserSettings = Mapping[str, Any]


class Browser:
    """Abstract class serving as the basis for Browser implementations.

    The Browser is used in the TestRunnerManager to start and stop the browser
    process, and to check the state of that process.

    :param logger: Structured logger to use for output.
    """
    __metaclass__ = ABCMeta

    init_timeout: float = 30

    def __init__(self, logger: StructuredLogger):
        self.logger = logger

    def setup(self) -> None:
        """Used for browser-specific setup that happens at the start of a test run"""
        pass

    def settings(self, test: Test) -> BrowserSettings:
        """Dictionary of metadata that is constant for a specific launch of a browser.

        This is used to determine when the browser instance configuration changes, requiring
        a relaunch of the browser. The test runner calls this method for each test, and if the
        returned value differs from that for the previous test, the browser is relaunched.
        """
        return {}

    @abstractmethod
    def start(self, group_metadata: GroupMetadata, **kwargs: Any) -> None:
        """Launch the browser object and get it into a state where is is ready to run tests"""
        pass

    @abstractmethod
    def stop(self, force: bool = False) -> bool:
        """Stop the running browser process.

        Return True iff the browser was successfully stopped.
        """
        pass

    @property
    @abstractmethod
    def pid(self) -> Optional[int]:
        """pid of the browser process or None if there is no pid"""
        pass

    @abstractmethod
    def is_alive(self) -> bool:
        """Boolean indicating whether the browser process is still running"""
        pass

    def cleanup(self) -> None:
        """Browser-specific cleanup that is run after the testrun is finished"""
        pass

    def executor_browser(self) -> Tuple[Type['ExecutorBrowser'], Mapping[str, Any]]:
        """Returns the ExecutorBrowser subclass for this Browser subclass and the keyword arguments
        with which it should be instantiated"""
        return ExecutorBrowser, {}

    def check_crash(self, process: int, test: str) -> bool:
        """Check if a crash occured and output any useful information to the
        log. Returns a boolean indicating whether a crash occured."""
        return False

    @property
    def pac(self) -> Optional[str]:
        return None


class NullBrowser(Browser):
    def __init__(self, logger: StructuredLogger, **kwargs: Any):
        super().__init__(logger)

    def start(self, group_metadata: GroupMetadata, **kwargs: Any) -> None:
        """No-op browser to use in scenarios where the TestRunnerManager shouldn't
        actually own the browser process (e.g. Servo where we start one browser
        per test)"""
        pass

    def stop(self, force: bool = False) -> bool:
        return True

    @property
    def pid(self) -> Optional[int]:
        return None

    def is_alive(self) -> bool:
        return True


class ExecutorBrowser:
    """View of the Browser used by the Executor object.
    This is needed because the Executor runs in a child process and
    we can't ship Browser instances between processes on Windows.

    Typically this will have a few product-specific properties set,
    but in some cases it may have more elaborate methods for setting
    up the browser from the runner process.
    """
    def __init__(self, **kwargs: Any):
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

    def __init__(self, logger: StructuredLogger, command: List[str], **kwargs: Any):
        self.logger = logger
        self.command = command
        self.pid: Optional[int] = None
        self.state = OutputHandlerState.BEFORE_PROCESS_START
        self.line_buffer: List[bytes] = []

    def after_process_start(self, pid: int) -> None:
        assert self.state == OutputHandlerState.BEFORE_PROCESS_START
        self.logger.debug("OutputHandler.after_process_start")
        self.pid = pid
        self.state = OutputHandlerState.AFTER_PROCESS_START

    def start(self, **kwargs: Any) -> None:
        assert self.state == OutputHandlerState.AFTER_PROCESS_START
        self.logger.debug("OutputHandler.start")
        # Need to change the state here before we try to empty the buffer
        # or we'll just re-buffer the existing output.
        self.state = OutputHandlerState.AFTER_HANDLER_START
        for item in self.line_buffer:
            self(item)
        self.line_buffer.clear()

    def after_process_stop(self, clean_shutdown: bool = True) -> None:
        # If we didn't get as far as configure, just
        # dump all logs with no configuration
        self.logger.debug("OutputHandler.after_process_stop")
        if self.state < OutputHandlerState.AFTER_HANDLER_START:
            self.start()
        self.state = OutputHandlerState.AFTER_PROCESS_STOP

    def __call__(self, line: bytes) -> None:
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


class WebDriverBrowser(Browser):
    __metaclass__ = ABCMeta

    def __init__(self,
                 logger: StructuredLogger,
                 binary: Optional[str] = None,
                 webdriver_binary: Optional[str] = None,
                 webdriver_args: Optional[List[str]] = None,
                 host: str = "127.0.0.1",
                 port: Optional[int] = None,
                 base_path: str = "/",
                 env: Optional[Mapping[str, str]] = None,
                 supports_pac: bool = True,
                 **kwargs: Any):
        super().__init__(logger)

        if webdriver_binary is None:
            raise ValueError("WebDriver server binary must be given "
                             "to --webdriver-binary argument")

        self.logger = logger
        self.binary = binary
        self.webdriver_binary = webdriver_binary

        self.host = host
        self._port = port
        self._supports_pac = supports_pac

        self.base_path = base_path
        self.env = os.environ.copy() if env is None else env
        self.webdriver_args = webdriver_args if webdriver_args is not None else []

        self.init_deadline: Optional[float] = None
        self._output_handler: Optional[OutputHandler] = None
        self._cmd = None
        self._proc: Optional[mozprocess.ProcessHandler] = None
        self._pac = None

    def make_command(self) -> List[str]:
        """Returns the full command for starting the server process as a list."""
        return [self.webdriver_binary] + self.webdriver_args

    def start(self, group_metadata: GroupMetadata, **kwargs: Any) -> None:
        self.init_deadline = time.time() + self.init_timeout
        try:
            self._run_server(group_metadata, **kwargs)
        except KeyboardInterrupt:
            self.stop()
            raise

    def create_output_handler(self, cmd: List[str]) -> OutputHandler:
        """Return an instance of the class used to handle application output.

        This can be overridden by subclasses which have particular requirements
        for parsing, or otherwise using, the output."""
        return OutputHandler(self.logger, cmd)

    def _run_server(self, group_metadata: GroupMetadata, **kwargs: Any) -> None:
        assert self.init_deadline is not None
        cmd = self.make_command()
        self._output_handler = self.create_output_handler(cmd)

        self._proc = mozprocess.ProcessHandler(
            cmd,
            processOutputLine=self._output_handler,
            env=self.env,
            storeOutput=False)

        self.logger.debug("Starting WebDriver: %s" % ' '.join(cmd))
        try:
            self._proc.run()
        except OSError as e:
            if e.errno == errno.ENOENT:
                raise OSError(
                    "WebDriver executable not found: %s" % self.webdriver_binary) from e
            raise
        self._output_handler.after_process_start(self._proc.pid)

        try:
            wait_for_service(
                self.logger,
                self.host,
                self.port,
                timeout=self.init_deadline - time.time(),
                server_process=self._proc,
            )
        except Exception:
            self.logger.error(
                "WebDriver was not accessible "
                f"within the timeout:\n{traceback.format_exc()}")
            raise
        finally:
            self._output_handler.start(group_metadata=group_metadata, **kwargs)
        self.logger.debug("_run complete")

    def stop(self, force: bool = False) -> bool:
        self.logger.debug("Stopping WebDriver")
        clean = True
        if self.is_alive():
            proc = cast(mozprocess.ProcessHandler, self._proc)
            # Pass a timeout value to mozprocess Processhandler.kill()
            # to ensure it always returns within it.
            # See https://bugzilla.mozilla.org/show_bug.cgi?id=1760080
            kill_result = proc.kill(timeout=5)
            if force and kill_result != 0:
                clean = False
                proc.kill(9, timeout=5)
        success = not self.is_alive()
        if success and self._output_handler is not None:
            # Only try to do output post-processing if we managed to shut down
            self._output_handler.after_process_stop(clean)
            self._output_handler = None
        return success

    def is_alive(self) -> bool:
        return self._proc is not None and hasattr(self._proc, "proc") and self._proc.poll() is None

    @property
    def url(self) -> str:
        if self.port is not None:
            return f"http://{self.host}:{self.port}{self.base_path}"
        raise ValueError("Can't get WebDriver URL before port is assigned")

    @property
    def pid(self) -> Optional[int]:
        return self._proc.pid if self._proc is not None else None

    @property
    def port(self) -> int:
        # If no port is supplied, we'll get a free port right before we use it.
        # Nothing guarantees an absence of race conditions here.
        if self._port is None:
            self._port = get_free_port()
        return self._port

    def cleanup(self) -> None:
        self.stop()

    def executor_browser(self) -> Tuple[Type[ExecutorBrowser], Mapping[str, Any]]:
        return ExecutorBrowser, {"webdriver_url": self.url,
                                 "host": self.host,
                                 "port": self.port,
                                 "pac": self.pac,
                                 "env": self.env}

    def settings(self, test: Test) -> BrowserSettings:
        self._pac = test.environment.get("pac", None) if self._supports_pac else None
        return {"pac": self._pac}

    @property
    def pac(self) -> Optional[str]:
        return self._pac
