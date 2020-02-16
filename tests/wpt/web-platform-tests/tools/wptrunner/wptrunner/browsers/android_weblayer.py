import subprocess

from .base import Browser, ExecutorBrowser, require_arg
from .base import get_timeout_multiplier   # noqa: F401
from .chrome import executor_kwargs as chrome_executor_kwargs
from ..webdriver_server import ChromeDriverServer
from ..executors.executorwebdriver import (WebDriverTestharnessExecutor,  # noqa: F401
                                           WebDriverRefTestExecutor)  # noqa: F401
from ..executors.executorchrome import ChromeDriverWdspecExecutor  # noqa: F401


__wptrunner__ = {"product": "android_weblayer",
                 "check_args": "check_args",
                 "browser": "WeblayerShell",
                 "executor": {"testharness": "WebDriverTestharnessExecutor",
                              "reftest": "WebDriverRefTestExecutor",
                              "wdspec": "ChromeDriverWdspecExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "timeout_multiplier": "get_timeout_multiplier"}

_wptserve_ports = set()


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "device_serial": kwargs["device_serial"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(test_type, server_config, cache_manager, run_info_data,
                    **kwargs):
    # Use update() to modify the global list in place.
    _wptserve_ports.update(set(
        server_config['ports']['http'] + server_config['ports']['https'] +
        server_config['ports']['ws'] + server_config['ports']['wss']
    ))

    executor_kwargs = chrome_executor_kwargs(test_type, server_config,
                                             cache_manager, run_info_data,
                                             **kwargs)
    del executor_kwargs["capabilities"]["goog:chromeOptions"]["prefs"]
    del executor_kwargs["capabilities"]["goog:chromeOptions"]["useAutomationExtension"]
    capabilities = executor_kwargs["capabilities"]
    # Note that for WebLayer, we launch a test shell and have the test shell use
    # WebLayer.
    # https://cs.chromium.org/chromium/src/weblayer/shell/android/shell_apk/
    capabilities["goog:chromeOptions"]["androidPackage"] = \
        "org.chromium.weblayer.shell"
    capabilities["goog:chromeOptions"]["androidActivity"] = ".WebLayerShellActivity"
    if kwargs.get('device_serial'):
        capabilities["goog:chromeOptions"]["androidDeviceSerial"] = kwargs['device_serial']

    # Workaround: driver.quit() cannot quit WeblayerShell.
    executor_kwargs["pause_after_test"] = False
    # Workaround: driver.close() is not supported.
    executor_kwargs["restart_after_test"] = True
    executor_kwargs["close_after_done"] = False
    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    # allow the use of host-resolver-rules in lieu of modifying /etc/hosts file
    return {"server_host": "127.0.0.1"}


#TODO: refactor common elements of WeblayerShell and ChromeAndroidBrowser
class WeblayerShell(Browser):
    """Chrome is backed by chromedriver, which is supplied through
    ``wptrunner.webdriver.ChromeDriverServer``.
    """

    def __init__(self, logger, binary, webdriver_binary="chromedriver",
                 device_serial=None,
                 webdriver_args=None):
        """Creates a new representation of Chrome.  The `binary` argument gives
        the browser binary to use for testing."""
        Browser.__init__(self, logger)
        self.binary = binary
        self.device_serial = device_serial
        self.server = ChromeDriverServer(self.logger,
                                         binary=webdriver_binary,
                                         args=webdriver_args)
        self.setup_adb_reverse()

    def _adb_run(self, args):
        cmd = ['adb']
        if self.device_serial:
            cmd.extend(['-s', self.device_serial])
        cmd.extend(args)
        self.logger.info(' '.join(cmd))
        subprocess.check_call(cmd)

    def setup_adb_reverse(self):
        self._adb_run(['wait-for-device'])
        self._adb_run(['forward', '--remove-all'])
        self._adb_run(['reverse', '--remove-all'])
        # "adb reverse" basically forwards network connection from device to
        # host.
        for port in _wptserve_ports:
            self._adb_run(['reverse', 'tcp:%d' % port, 'tcp:%d' % port])

    def start(self, **kwargs):
        self.server.start(block=False)

    def stop(self, force=False):
        self.server.stop(force=force)

    def pid(self):
        return self.server.pid

    def is_alive(self):
        # TODO(ato): This only indicates the driver is alive,
        # and doesn't say anything about whether a browser session
        # is active.
        return self.server.is_alive()

    def cleanup(self):
        self.stop()
        self._adb_run(['forward', '--remove-all'])
        self._adb_run(['reverse', '--remove-all'])

    def executor_browser(self):
        return ExecutorBrowser, {"webdriver_url": self.server.url}
