# mypy: allow-untyped-defs

import mozprocess
import subprocess

from .base import cmd_arg, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from .base import WebDriverBrowser  # noqa: F401
from .chrome import executor_kwargs as chrome_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorchrome import ChromeDriverPrintRefTestExecutor  # noqa: F401
from ..executors.executorwebdriver import (WebDriverCrashtestExecutor,  # noqa: F401
                                           WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401


__wptrunner__ = {"product": "chrome_android",
                 "check_args": "check_args",
                 "browser": "ChromeAndroidBrowser",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "print-reftest": "ChromeDriverPrintRefTestExecutor",
                              "wdspec": "WdspecExecutor",
                              "crashtest": "WebDriverCrashtestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}

_wptserve_ports = set()


def check_args(**kwargs):
    require_arg(kwargs, "package_name")
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"package_name": kwargs["package_name"],
            "adb_binary": kwargs["adb_binary"],
            "device_serial": kwargs["device_serial"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args"),
            "stackwalk_binary": kwargs.get("stackwalk_binary"),
            "symbols_path": kwargs.get("symbols_path")}


def executor_kwargs(logger, test_type, test_environment, run_info_data,
                    **kwargs):
    # Use update() to modify the global list in place.
    _wptserve_ports.update(set(
        test_environment.config['ports']['http'] + test_environment.config['ports']['https'] +
        test_environment.config['ports']['ws'] + test_environment.config['ports']['wss']
    ))

    executor_kwargs = chrome_executor_kwargs(logger, test_type, test_environment, run_info_data,
                                             **kwargs)
    # Remove unsupported options on mobile.
    del executor_kwargs["capabilities"]["goog:chromeOptions"]["prefs"]

    assert kwargs["package_name"], "missing --package-name"
    capabilities = executor_kwargs["capabilities"]
    capabilities["goog:chromeOptions"]["androidPackage"] = \
        kwargs["package_name"]
    capabilities["goog:chromeOptions"]["androidKeepAppDataDir"] = \
        kwargs.get("keep_app_data_directory")

    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    # allow the use of host-resolver-rules in lieu of modifying /etc/hosts file
    return {"server_host": "127.0.0.1"}


class LogcatRunner:
    def __init__(self, logger, browser, remote_queue):
        self.logger = logger
        self.browser = browser
        self.remote_queue = remote_queue

    def start(self):
        try:
            self._run()
        except KeyboardInterrupt:
            self.stop()

    def _run(self):
        try:
            # TODO: adb logcat -c fail randomly with message
            # "failed to clear the 'main' log"
            self.browser.clear_log()
        except subprocess.CalledProcessError:
            self.logger.error("Failed to clear logcat buffer")

        self._cmd = self.browser.logcat_cmd()
        self._proc = mozprocess.ProcessHandler(
            self._cmd,
            processOutputLine=self.on_output,
            storeOutput=False)
        self._proc.run()

    def _send_message(self, command, *args):
        try:
            self.remote_queue.put((command, args))
        except AssertionError:
            self.logger.warning("Error when send to remote queue")

    def stop(self, force=False):
        if self.is_alive():
            kill_result = self._proc.kill()
            if force and kill_result != 0:
                self._proc.kill(9)

    def is_alive(self):
        return hasattr(self._proc, "proc") and self._proc.poll() is None

    def on_output(self, line):
        data = {
            "action": "process_output",
            "process": "LOGCAT",
            "command": "logcat",
            "data": line
        }
        self._send_message("log", data)


class ChromeAndroidBrowserBase(WebDriverBrowser):
    def __init__(self,
                 logger,
                 webdriver_binary="chromedriver",
                 adb_binary=None,
                 remote_queue=None,
                 device_serial=None,
                 webdriver_args=None,
                 stackwalk_binary=None,
                 symbols_path=None):
        super().__init__(logger,
                         binary=None,
                         webdriver_binary=webdriver_binary,
                         webdriver_args=webdriver_args,)
        self.adb_binary = adb_binary or "adb"
        self.device_serial = device_serial
        self.stackwalk_binary = stackwalk_binary
        self.symbols_path = symbols_path
        self.remote_queue = remote_queue

        if self.remote_queue is not None:
            self.logcat_runner = LogcatRunner(self.logger, self, self.remote_queue)

    def setup(self):
        self.setup_adb_reverse()
        if self.remote_queue is not None:
            self.logcat_runner.start()

    def _adb_run(self, args):
        cmd = [self.adb_binary]
        if self.device_serial:
            cmd.extend(['-s', self.device_serial])
        cmd.extend(args)
        self.logger.info(' '.join(cmd))
        subprocess.check_call(cmd)

    def make_command(self):
        return [self.webdriver_binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path),
                cmd_arg("enable-chrome-logs")] + self.webdriver_args

    def cleanup(self):
        super().cleanup()
        self._adb_run(['forward', '--remove-all'])
        self._adb_run(['reverse', '--remove-all'])
        if self.remote_queue is not None:
            self.logcat_runner.stop(force=True)

    def executor_browser(self):
        cls, kwargs = super().executor_browser()
        kwargs["capabilities"] = {
            "goog:chromeOptions": {
                "androidDeviceSerial": self.device_serial
            }
        }
        return cls, kwargs

    def clear_log(self):
        self._adb_run(['logcat', '-c'])

    def logcat_cmd(self):
        cmd = [self.adb_binary]
        if self.device_serial:
            cmd.extend(['-s', self.device_serial])
        cmd.extend(['logcat', '*:D'])
        return cmd

    def check_crash(self, process, test):
        self.maybe_parse_tombstone()
        # Existence of a tombstone does not necessarily mean test target has
        # crashed. Always return False so we don't change the test results.
        return False

    def maybe_parse_tombstone(self):
        if self.stackwalk_binary:
            cmd = [self.stackwalk_binary, "-a", "-w"]
            if self.device_serial:
                cmd.extend(["--device", self.device_serial])
            cmd.extend(["--output-directory", self.symbols_path])
            raw_output = subprocess.check_output(cmd)
            for line in raw_output.splitlines():
                self.logger.process_output("TRACE", line, "logcat")

    def setup_adb_reverse(self):
        self._adb_run(['wait-for-device'])
        self._adb_run(['forward', '--remove-all'])
        self._adb_run(['reverse', '--remove-all'])
        # "adb reverse" forwards network connection from device to host.
        for port in self.wptserver_ports:
            self._adb_run(['reverse', 'tcp:%d' % port, 'tcp:%d' % port])


class ChromeAndroidBrowser(ChromeAndroidBrowserBase):
    """Chrome is backed by chromedriver, which is supplied through
    ``wptrunner.webdriver.ChromeDriverServer``.
    """

    def __init__(self, logger, package_name,
                 webdriver_binary="chromedriver",
                 adb_binary=None,
                 remote_queue = None,
                 device_serial=None,
                 webdriver_args=None,
                 stackwalk_binary=None,
                 symbols_path=None):
        super().__init__(logger,
                         webdriver_binary, adb_binary, remote_queue,
                         device_serial, webdriver_args, stackwalk_binary,
                         symbols_path)
        self.package_name = package_name
        self.wptserver_ports = _wptserve_ports
