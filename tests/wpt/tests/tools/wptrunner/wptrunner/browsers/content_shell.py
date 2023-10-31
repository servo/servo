# mypy: allow-untyped-defs

import contextlib
import os
import subprocess
from multiprocessing import Queue, Event
from threading import Thread
from urllib.parse import urljoin

from . import chrome_spki_certs
from .base import (
    Browser,
    ExecutorBrowser,
    OutputHandler,
    browser_command,
)
from .base import get_timeout_multiplier   # noqa: F401
from .chrome import debug_args
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import server_url
from ..executors.executorcontentshell import (  # noqa: F401
    ContentShellCrashtestExecutor,
    ContentShellPrintRefTestExecutor,
    ContentShellRefTestExecutor,
    ContentShellTestharnessExecutor,
)

ENABLE_THREADED_COMPOSITING_FLAG = '--enable-threaded-compositing'
DISABLE_THREADED_COMPOSITING_FLAG = '--disable-threaded-compositing'
DISABLE_THREADED_ANIMATION_FLAG = '--disable-threaded-animation'


__wptrunner__ = {"product": "content_shell",
                 "check_args": "check_args",
                 "browser": "ContentShellBrowser",
                 "executor": {
                     "crashtest": "ContentShellCrashtestExecutor",
                     "print-reftest": "ContentShellPrintRefTestExecutor",
                     "reftest": "ContentShellRefTestExecutor",
                     "testharness": "ContentShellTestharnessExecutor",
                 },
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "update_properties": "update_properties",
                 "timeout_multiplier": "get_timeout_multiplier",}


def check_args(**kwargs):
    pass


def browser_kwargs(logger, test_type, run_info_data, config, subsuite, **kwargs):
    args = []
    args.append("--ignore-certificate-errors-spki-list=%s" %
        ','.join(chrome_spki_certs.IGNORE_CERTIFICATE_ERRORS_SPKI_LIST))
    # For WebTransport tests.
    args.append("--webtransport-developer-mode")

    if not kwargs["headless"]:
        args.append("--disable-headless-mode")

    if kwargs["debug_info"]:
        args.extend(debug_args(kwargs["debug_info"]))

    # `--run-web-tests -` are specific to content_shell - they activate web
    # test protocol mode.
    args.append("--run-web-tests")
    for arg in kwargs.get("binary_args", []):
        if arg not in args:
            args.append(arg)

    # Temporary workaround to align with RWT behavior. Unless a vts explicitly
    # enables threaded compositing, we should use single threaded compositing
    if ENABLE_THREADED_COMPOSITING_FLAG not in subsuite.config.get("binary_args", []):
        args.extend([DISABLE_THREADED_COMPOSITING_FLAG,
                     DISABLE_THREADED_ANIMATION_FLAG])

    for arg in subsuite.config.get("binary_args", []):
        if arg not in args:
            args.append(arg)
    args.append("-")

    return {"binary": kwargs["binary"],
            "binary_args": args,
            "debug_info": kwargs["debug_info"],
            "pac_origin": server_url(config, "http")}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    sanitizer_enabled = kwargs.get("sanitizer_enabled")
    if sanitizer_enabled:
        test_type = "crashtest"
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data,
                                           **kwargs)
    executor_kwargs["sanitizer_enabled"] = sanitizer_enabled
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    return {"server_host": "127.0.0.1",
            "testharnessreport": "testharnessreport-content-shell.js",
            "supports_debugger": True}


def update_properties():
    return (["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]})


class ContentShellBrowser(Browser):
    """Class that represents an instance of content_shell.

    Upon startup, the stdout, stderr, and stdin pipes of the underlying content_shell
    process are connected to multiprocessing Queues so that the runner process can
    interact with content_shell through its protocol mode.

    See Also:
        Protocol Mode: https://chromium.googlesource.com/chromium/src.git/+/HEAD/content/web_test/browser/test_info_extractor.h
    """
    # Seconds to wait for the process to stop after it was sent a `QUIT`
    # command, after which `SIGTERM` or `TerminateProcess()` forces termination.
    # The timeout is ported from:
    # https://chromium.googlesource.com/chromium/src/+/b175d48d3ea4ea66eea35c88c11aa80d233f3bee/third_party/blink/tools/blinkpy/web_tests/port/base.py#476
    termination_timeout: float = 3

    def __init__(self, logger, binary="content_shell", binary_args=None,
                 debug_info=None, pac_origin=None, **kwargs):
        super().__init__(logger)
        self._debug_cmd_prefix, self._browser_cmd = browser_command(
            binary, binary_args or [], debug_info)
        self._output_handler = None
        self._proc = None
        self._pac_origin = pac_origin
        self._pac = None

    def start(self, group_metadata, **settings):
        browser_cmd, pac = list(self._browser_cmd), settings.get("pac")
        if pac:
            browser_cmd.insert(1, f"--proxy-pac-url={pac}")
        self.logger.debug(f"Starting content shell: {browser_cmd[0]}...")
        args = [*self._debug_cmd_prefix, *browser_cmd]
        self._output_handler = OutputHandler(self.logger, args)
        if os.name == "posix":
            close_fds, preexec_fn = True, lambda: os.setpgid(0, 0)
        else:
            close_fds, preexec_fn = False, None
        self._proc = subprocess.Popen(args,
                                      stdin=subprocess.PIPE,
                                      stdout=subprocess.PIPE,
                                      stderr=subprocess.PIPE,
                                      close_fds=close_fds,
                                      preexec_fn=preexec_fn)
        self._output_handler.after_process_start(self._proc.pid)

        self._stdout_queue = Queue()
        self._stderr_queue = Queue()
        self._stdin_queue = Queue()
        self._io_stopped = Event()

        self._stdout_reader = self._create_reader_thread("stdout-reader",
                                                         self._proc.stdout,
                                                         self._stdout_queue,
                                                         prefix=b"OUT: ")
        self._stderr_reader = self._create_reader_thread("stderr-reader",
                                                         self._proc.stderr,
                                                         self._stderr_queue,
                                                         prefix=b"ERR: ")
        self._stdin_writer = self._create_writer_thread("stdin-writer",
                                                        self._proc.stdin,
                                                        self._stdin_queue)

        # Content shell is likely still in the process of initializing. The actual waiting
        # for the startup to finish is done in the ContentShellProtocol.
        self.logger.debug("Content shell has been started.")
        self._output_handler.start(group_metadata=group_metadata, **settings)

    def stop(self, force=False):
        self.logger.debug("Stopping content shell...")

        clean_shutdown = stopped = True
        if self.is_alive():
            clean_shutdown = self._terminate_process(force=force)

        # Close these queues cleanly to avoid broken pipe error spam in the logs.
        self._stdin_queue.put(None)
        for thread in [self._stdout_reader, self._stderr_reader, self._stdin_writer]:
            thread.join(2)
            if thread.is_alive():
                self.logger.warning(f"Content shell IO thread {thread.name} did not shut down gracefully.")
                stopped = False

        if not self.is_alive():
            self.logger.debug(
                "Content shell has been stopped "
                f"(PID: {self._proc.pid}, exit code: {self._proc.returncode})")
        else:
            stopped = False
            self.logger.warning(f"Content shell failed to stop (PID: {self._proc.pid})")
        if stopped and self._output_handler is not None:
            self._output_handler.after_process_stop(clean_shutdown)
            self._output_handler = None
        return stopped

    def _terminate_process(self, force: bool = False) -> bool:
        self._stdin_queue.put(b"QUIT\n")
        with contextlib.suppress(subprocess.TimeoutExpired):
            self._proc.wait(timeout=self.termination_timeout)
            return True
        self.logger.warning(
            "Content shell failed to respond to QUIT command "
            f"(PID: {self._proc.pid}, timeout: {self.termination_timeout}s)")
        # Skip `terminate()` on Windows, which is an alias for `kill()`, and
        # only `kill()` for `force=True`.
        #
        # [1]: https://docs.python.org/3/library/subprocess.html#subprocess.Popen.kill
        if os.name == "posix":
            self._proc.terminate()
            with contextlib.suppress(subprocess.TimeoutExpired):
                self._proc.wait(timeout=1)
                return False
        if force:
            self._proc.kill()
        return False

    def is_alive(self):
        return self._proc is not None and self._proc.poll() is None

    def pid(self):
        return self._proc.pid if self._proc else None

    def executor_browser(self):
        """This function returns the `ExecutorBrowser` object that is used by other
        processes to interact with content_shell. In our case, this consists of the three
        multiprocessing Queues as well as an `io_stopped` event to signal when the
        underlying pipes have reached EOF.
        """
        return ExecutorBrowser, {"stdout_queue": self._stdout_queue,
                                 "stderr_queue": self._stderr_queue,
                                 "stdin_queue": self._stdin_queue,
                                 "io_stopped": self._io_stopped}

    def check_crash(self, process, test):
        return not self.is_alive()

    def settings(self, test):
        pac_path = test.environment.get("pac")
        if self._pac_origin and pac_path:
            self._pac = urljoin(self._pac_origin, pac_path)
            return {"pac": self._pac}
        return {}

    def _create_reader_thread(self, name, stream, queue, prefix=b""):
        """This creates (and starts) a background thread which reads lines from `stream` and
        puts them into `queue` until `stream` reports EOF.
        """
        def reader_thread(stream, queue, stop_event):
            while True:
                line = stream.readline()
                if not line:
                    break
                self._output_handler(prefix + line.rstrip())
                queue.put(line)

            stop_event.set()
            queue.close()
            queue.join_thread()

        result = Thread(name=name,
                        target=reader_thread,
                        args=(stream, queue, self._io_stopped),
                        daemon=True)
        result.start()
        return result

    def _create_writer_thread(self, name, stream, queue):
        """This creates (and starts) a background thread which gets items from `queue` and
        writes them into `stream` until it encounters a None item in the queue.
        """
        def writer_thread(stream, queue):
            while True:
                line = queue.get()
                if not line:
                    break

                stream.write(line)
                stream.flush()

        result = Thread(name=name,
                        target=writer_thread,
                        args=(stream, queue),
                        daemon=True)
        result.start()
        return result
