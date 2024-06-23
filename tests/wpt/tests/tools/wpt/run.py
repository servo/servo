# mypy: allow-untyped-defs

import argparse
import os
import platform
import subprocess
import sys
from shutil import copyfile, which
from typing import ClassVar, Tuple, Type

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, os.pardir))
sys.path.insert(0, os.path.abspath(os.path.join(wpt_root, "tools")))

from . import browser, install, testfiles
from ..serve import serve

logger = None


class WptrunError(Exception):
    pass


class WptrunnerHelpAction(argparse.Action):
    def __init__(self,
                 option_strings,
                 dest=argparse.SUPPRESS,
                 default=argparse.SUPPRESS,
                 help=None):
        super().__init__(
            option_strings=option_strings,
            dest=dest,
            default=default,
            nargs=0,
            help=help)

    def __call__(self, parser, namespace, values, option_string=None):
        from wptrunner import wptcommandline
        wptparser = wptcommandline.create_parser()
        wptparser.usage = parser.usage
        wptparser.print_help()
        parser.exit()


def create_parser():
    from wptrunner import wptcommandline

    parser = argparse.ArgumentParser(add_help=False, parents=[install.channel_args])
    parser.add_argument("product", action="store",
                        help="Browser to run tests in")
    parser.add_argument("--affected", action="store", default=None,
                        help="Run affected tests since revish")
    parser.add_argument("--yes", "-y", dest="prompt", action="store_false", default=True,
                        help="Don't prompt before installing components")
    parser.add_argument("--install-browser", action="store_true",
                        help="Install the browser from the release channel specified by --channel "
                        "(or the nightly channel by default).")
    parser.add_argument("--install-webdriver", action="store_true",
                        help="Install WebDriver from the release channel specified by --channel "
                        "(or the nightly channel by default).")
    parser.add_argument("--logcat-dir", action="store", default=None,
                        help="Directory to write Android logcat files to")
    parser._add_container_actions(wptcommandline.create_parser())
    return parser


def exit(msg=None):
    if msg:
        logger.critical(msg)
        sys.exit(1)
    else:
        sys.exit(0)


def args_general(kwargs):

    def set_if_none(name, value):
        if kwargs.get(name) is None:
            kwargs[name] = value
            logger.info("Set %s to %s" % (name, value))

    set_if_none("tests_root", wpt_root)
    set_if_none("metadata_root", wpt_root)
    set_if_none("manifest_update", True)
    set_if_none("manifest_download", True)

    if kwargs["ssl_type"] in (None, "pregenerated"):
        cert_root = os.path.join(wpt_root, "tools", "certs")
        if kwargs["ca_cert_path"] is None:
            kwargs["ca_cert_path"] = os.path.join(cert_root, "cacert.pem")

        if kwargs["host_key_path"] is None:
            kwargs["host_key_path"] = os.path.join(cert_root, "web-platform.test.key")

        if kwargs["host_cert_path"] is None:
            kwargs["host_cert_path"] = os.path.join(cert_root, "web-platform.test.pem")
    elif kwargs["ssl_type"] == "openssl":
        if not which(kwargs["openssl_binary"]):
            if os.uname()[0] == "Windows":
                raise WptrunError("""OpenSSL binary not found. If you need HTTPS tests, install OpenSSL from

https://slproweb.com/products/Win32OpenSSL.html

Ensuring that libraries are added to /bin and add the resulting bin directory to
your PATH.

Otherwise run with --ssl-type=none""")
            else:
                raise WptrunError("""OpenSSL not found. If you don't need HTTPS support run with --ssl-type=none,
otherwise install OpenSSL and ensure that it's on your $PATH.""")


def check_environ(product):
    if product not in ("android_webview", "chrome", "chrome_android", "chrome_ios",
                       "edge", "firefox", "firefox_android", "headless_shell",
                       "ladybird", "servo", "wktr"):
        config_builder = serve.build_config(os.path.join(wpt_root, "config.json"))
        # Override the ports to avoid looking for free ports
        config_builder.ssl = {"type": "none"}
        config_builder.ports = {"http": [8000]}

        is_windows = platform.uname()[0] == "Windows"

        with config_builder as config:
            expected_hosts = set(config.domains_set)
            if is_windows:
                expected_hosts.update(config.not_domains_set)

        missing_hosts = set(expected_hosts)
        if is_windows:
            hosts_path = r"%s\System32\drivers\etc\hosts" % os.environ.get(
                "SystemRoot", r"C:\Windows")
        else:
            hosts_path = "/etc/hosts"

        if os.path.abspath(os.curdir) == wpt_root:
            wpt_path = "wpt"
        else:
            wpt_path = os.path.join(wpt_root, "wpt")

        with open(hosts_path) as f:
            for line in f:
                line = line.split("#", 1)[0].strip()
                parts = line.split()
                hosts = parts[1:]
                for host in hosts:
                    missing_hosts.discard(host)
            if missing_hosts:
                if is_windows:
                    message = """Missing hosts file configuration. Run

python %s make-hosts-file | Out-File %s -Encoding ascii -Append

in PowerShell with Administrator privileges.""" % (wpt_path, hosts_path)
                else:
                    message = """Missing hosts file configuration. Run

%s make-hosts-file | sudo tee -a %s""" % ("./wpt" if wpt_path == "wpt" else wpt_path,
                                          hosts_path)
                raise WptrunError(message)


class AndroidLogcat:
    def __init__(self, adb_path, base_path=None):
        self.adb_path = adb_path
        self.base_path = base_path if base_path is not None else os.curdir
        self.procs = {}

    def start(self, device_serial):
        """
        Start recording logcat. Writes logcat to the upload directory.
        """
        # Start logcat for the device. The adb process runs until the
        # corresponding device is stopped. Output is written directly to
        # the blobber upload directory so that it is uploaded automatically
        # at the end of the job.
        if device_serial in self.procs:
            logger.warning(f"Logcat for {device_serial} already started")
            return

        logcat_path = os.path.join(self.base_path, f"logcat-{device_serial}.log")
        out_file = open(logcat_path, "w")
        cmd = [
            self.adb_path,
            "-s",
            device_serial,
            "logcat",
            "-v",
            "threadtime",
            "Trace:S",
            "StrictMode:S",
            "ExchangeService:S",
        ]
        logger.debug(" ".join(cmd))
        proc = subprocess.Popen(
            cmd, stdout=out_file, stdin=subprocess.PIPE
        )
        logger.info(f"Started logcat for device {device_serial} pid {proc.pid}")
        self.procs[device_serial] = (proc, out_file)

    def stop(self, device_serial=None):
        """
        Stop logcat process started by logcat_start.
        """
        if device_serial is None:
            for key in list(self.procs.keys()):
                self.stop(key)
            return

        proc, out_file = self.procs.get(device_serial, (None, None))
        if proc is not None:
            try:
                proc.kill()
                out_file.close()
            finally:
                del self.procs[device_serial]


class BrowserSetup:
    name: ClassVar[str]
    browser_cls: ClassVar[Type[browser.Browser]]

    def __init__(self, venv, prompt=True):
        self.browser = self.browser_cls(logger)
        self.venv = venv
        self.prompt = prompt

    def prompt_install(self, component):
        if not self.prompt:
            return True
        while True:
            resp = input("Download and install %s [Y/n]? " % component).strip().lower()
            if not resp or resp == "y":
                return True
            elif resp == "n":
                return False

    def install(self, channel=None):
        if self.prompt_install(self.name):
            return self.browser.install(self.venv.path, channel)

    def requirements(self):
        if self.browser.requirements:
            return [os.path.join(wpt_root, "tools", "wptrunner", self.browser.requirements)]
        return []

    def setup(self, kwargs):
        self.setup_kwargs(kwargs)

    def teardown(self):
        pass


def safe_unsetenv(env_var):
    """Safely remove an environment variable.

    Python3 does not support os.unsetenv in Windows for python<3.9, so we better
    remove the variable directly from os.environ.
    """
    try:
        del os.environ[env_var]
    except KeyError:
        pass


class Firefox(BrowserSetup):
    name = "firefox"
    browser_cls = browser.Firefox

    def setup_kwargs(self, kwargs):
        if kwargs["binary"] is None:
            if kwargs["browser_channel"] is None:
                kwargs["browser_channel"] = "nightly"
                logger.info("No browser channel specified. Running nightly instead.")

            binary = self.browser.find_binary(self.venv.path,
                                              kwargs["browser_channel"])
            if binary is None:
                raise WptrunError("""Firefox binary not found on $PATH.

Install Firefox or use --binary to set the binary path""")
            kwargs["binary"] = binary

        if kwargs["certutil_binary"] is None and kwargs["ssl_type"] != "none":
            certutil = self.browser.find_certutil()

            if certutil is None:
                # Can't download this for now because it's missing the libnss3 library
                logger.info("""Can't find certutil, certificates will not be checked.
Consider installing certutil via your OS package manager or directly.""")
            else:
                logger.info("Using certutil %s" % certutil)

            kwargs["certutil_binary"] = certutil

        if kwargs["webdriver_binary"] is None and "wdspec" in kwargs["test_types"]:
            webdriver_binary = None
            if not kwargs["install_webdriver"]:
                webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("geckodriver")

                if install:
                    logger.info("Downloading geckodriver")
                    webdriver_binary = self.browser.install_webdriver(
                        dest=self.venv.bin_path,
                        channel=kwargs["browser_channel"],
                        browser_binary=kwargs["binary"])
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                logger.info("Unable to find or install geckodriver, skipping wdspec tests")
                kwargs["test_types"].remove("wdspec")

        if kwargs["prefs_root"] is None:
            prefs_root = self.browser.install_prefs(kwargs["binary"],
                                                    self.venv.path,
                                                    channel=kwargs["browser_channel"])
            kwargs["prefs_root"] = prefs_root

        if kwargs["headless"] is None and not kwargs["debug_test"]:
            kwargs["headless"] = True
            logger.info("Running in headless mode, pass --no-headless to disable")

        if kwargs["browser_channel"] == "nightly" and kwargs["enable_webtransport_h3"] is None:
            kwargs["enable_webtransport_h3"] = True

        # Turn off Firefox WebRTC ICE logging on WPT (turned on by mozrunner)
        safe_unsetenv('R_LOG_LEVEL')
        safe_unsetenv('R_LOG_DESTINATION')
        safe_unsetenv('R_LOG_VERBOSE')

        # Allow WebRTC tests to call getUserMedia.
        kwargs["extra_prefs"].append("media.navigator.streams.fake=true")


class FirefoxAndroid(BrowserSetup):
    name = "firefox_android"
    browser_cls = browser.FirefoxAndroid

    def setup_kwargs(self, kwargs):
        from . import android
        import mozdevice

        # We don't support multiple channels for android yet
        if kwargs["browser_channel"] is None:
            kwargs["browser_channel"] = "nightly"

        if kwargs["prefs_root"] is None:
            prefs_root = self.browser.install_prefs(kwargs["binary"],
                                                    self.venv.path,
                                                    channel=kwargs["browser_channel"])
            kwargs["prefs_root"] = prefs_root

        if kwargs["package_name"] is None:
            kwargs["package_name"] = "org.mozilla.geckoview.test_runner"
        app = kwargs["package_name"]

        if not kwargs["device_serial"]:
            kwargs["device_serial"] = ["emulator-5554"]

        if kwargs["webdriver_binary"] is None and "wdspec" in kwargs["test_types"]:
            webdriver_binary = None
            if not kwargs["install_webdriver"]:
                webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("geckodriver")

                if install:
                    logger.info("Downloading geckodriver")
                    webdriver_binary = self.browser.install_webdriver(
                        dest=self.venv.bin_path,
                        channel=kwargs["browser_channel"],
                        browser_binary=kwargs["binary"])
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                logger.info("Unable to find or install geckodriver, skipping wdspec tests")
                kwargs["test_types"].remove("wdspec")

        if kwargs["adb_binary"] is None:
            if "ADB_PATH" not in os.environ:
                adb_path = os.path.join(android.get_paths(None)["sdk"],
                                        "platform-tools",
                                        "adb")
                os.environ["ADB_PATH"] = adb_path
            kwargs["adb_binary"] = os.environ["ADB_PATH"]

        self._logcat = AndroidLogcat(kwargs["adb_binary"], base_path=kwargs["logcat_dir"])

        for device_serial in kwargs["device_serial"]:
            if device_serial.startswith("emulator-"):
                # We're running on an emulator so ensure that's set up
                android.start(logger,
                              reinstall=False,
                              device_serial=device_serial,
                              prompt=kwargs["prompt"])

        for device_serial in kwargs["device_serial"]:
            device = mozdevice.ADBDeviceFactory(adb=kwargs["adb_binary"],
                                                device=device_serial)
            self._logcat.start(device_serial)
            max_retries = 5
            last_exception = None
            if self.browser.apk_path:
                device.uninstall_app(app)
                for i in range(max_retries + 1):
                    logger.info(f"Installing {app} on {device_serial} "
                                f"attempt {i + 1}/{max_retries + 1}")
                    try:
                        # Temporarily replace mozdevice function with custom code
                        # that passes in the `--no-incremental` option
                        cmd = ["install", "--no-incremental", self.browser.apk_path]
                        logger.debug(" ".join(cmd))
                        data = device.command_output(cmd, timeout=120)
                        if data.find("Success") == -1:
                            raise mozdevice.ADBError(f"Install failed for {self.browser.apk_path}."
                                                     f" Got: {data}")
                    except Exception as e:
                        last_exception = e
                    else:
                        break
                else:
                    assert last_exception is not None
                    raise WptrunError(f"Failed to install {app} on device {device_serial} "
                                      f"after {max_retries} retries") from last_exception
            elif not device.is_app_installed(app):
                raise WptrunError(f"app {app} not installed on device {device_serial}")


    def teardown(self):
        from . import android

        if hasattr(self, "_logcat"):
            emulator_log = os.path.join(android.get_paths(None)["sdk"],
                                        ".android",
                                        "emulator.log")
            if os.path.exists(emulator_log):
                dest_path = os.path.join(self._logcat.base_path, "emulator.log")
                copyfile(emulator_log, dest_path)

            self._logcat.stop()


class Chrome(BrowserSetup):
    name = "chrome"
    browser_cls: ClassVar[Type[browser.ChromeChromiumBase]] = browser.Chrome
    experimental_channels: ClassVar[Tuple[str, ...]] = ("dev", "canary")

    def setup_kwargs(self, kwargs):
        browser_channel = kwargs["browser_channel"]
        if kwargs["binary"] is None:
            binary = self.browser.find_binary(venv_path=self.venv.path, channel=browser_channel)
            if binary:
                kwargs["binary"] = binary
            else:
                raise WptrunError(f"Unable to locate {self.name.capitalize()} binary")

        if kwargs["mojojs_path"]:
            kwargs["enable_mojojs"] = True
            logger.info("--mojojs-path is provided, enabling MojoJS")
        else:
            path = self.browser.install_mojojs(dest=self.venv.path,
                                               browser_binary=kwargs["binary"])
            if path:
                kwargs["mojojs_path"] = path
                kwargs["enable_mojojs"] = True
                logger.info(f"MojoJS enabled automatically (mojojs_path: {path})")
            else:
                kwargs["enable_mojojs"] = False
                logger.info("MojoJS is disabled for this run.")

        if kwargs["webdriver_binary"] is None:
            webdriver_binary = None
            if not kwargs["install_webdriver"]:
                webdriver_binary = self.browser.find_webdriver(self.venv.bin_path)
                if webdriver_binary and not self.browser.webdriver_supports_browser(
                        webdriver_binary, kwargs["binary"], browser_channel):
                    webdriver_binary = None

            if webdriver_binary is None:
                install = self.prompt_install("chromedriver")

                if install:
                    webdriver_binary = self.browser.install_webdriver(
                        dest=self.venv.bin_path,
                        channel=browser_channel,
                        browser_binary=kwargs["binary"],
                    )
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                raise WptrunError("Unable to locate or install matching ChromeDriver binary")
        if kwargs["headless"] is None and not kwargs["debug_test"]:
            kwargs["headless"] = True
            logger.info("Running in headless mode, pass --no-headless to disable")
        if browser_channel in self.experimental_channels:
            # HACK(Hexcles): work around https://github.com/web-platform-tests/wpt/issues/16448
            kwargs["webdriver_args"].append("--disable-build-check")
            if kwargs["enable_experimental"] is None:
                logger.info(
                    "Automatically turning on experimental features for Chrome Dev/Canary or Chromium trunk")
                kwargs["enable_experimental"] = True
            if kwargs["enable_webtransport_h3"] is None:
                # To start the WebTransport over HTTP/3 test server.
                kwargs["enable_webtransport_h3"] = True
        if os.getenv("TASKCLUSTER_ROOT_URL"):
            # We are on Taskcluster, where our Docker container does not have
            # enough capabilities to run Chrome with sandboxing. (gh-20133)
            kwargs["binary_args"].append("--no-sandbox")


class HeadlessShell(BrowserSetup):
    name = "headless_shell"
    browser_cls = browser.HeadlessShell
    experimental_channels = ("dev", "canary", "nightly")

    def setup_kwargs(self, kwargs):
        browser_channel = kwargs["browser_channel"]
        if kwargs["binary"] is None:
            binary = self.browser.find_binary(venv_path=self.venv.path, channel=browser_channel)
            if binary:
                kwargs["binary"] = binary
            else:
                raise WptrunError(f"Unable to locate {self.name!r} binary")

        if kwargs["mojojs_path"]:
            kwargs["enable_mojojs"] = True
            logger.info("--mojojs-path is provided, enabling MojoJS")
        elif kwargs["enable_mojojs"]:
            logger.warning(f"Cannot install MojoJS for {self.name}, "
                           "which does not return version information. "
                           "Provide '--mojojs-path' explicitly instead.")
            logger.warning("MojoJS is disabled for this run.")

        # Never pause after test, since headless shell is not interactive.
        kwargs["pause_after_test"] = False
        # Don't add a `--headless` switch.
        kwargs["headless"] = False

        if kwargs["enable_webtransport_h3"] is None:
            kwargs["enable_webtransport_h3"] = True


class Chromium(Chrome):
    name = "chromium"
    browser_cls: ClassVar[Type[browser.ChromeChromiumBase]] = browser.Chromium
    experimental_channels = ("nightly",)


class ChromeAndroidBase(BrowserSetup):
    experimental_channels = ("dev", "canary")

    def setup_kwargs(self, kwargs):
        if kwargs.get("device_serial"):
            self.browser.device_serial = kwargs["device_serial"]
        if kwargs.get("adb_binary"):
            self.browser.adb_binary = kwargs["adb_binary"]
        browser_channel = kwargs["browser_channel"]
        if kwargs["package_name"] is None:
            kwargs["package_name"] = self.browser.find_binary(
                channel=browser_channel)
        if not kwargs["device_serial"]:
            kwargs["device_serial"] = ["emulator-5554"]
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = None
            if not kwargs["install_webdriver"]:
                webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("chromedriver")

                if install:
                    logger.info("Downloading chromedriver")
                    webdriver_binary = self.browser.install_webdriver(
                        dest=self.venv.bin_path,
                        channel=browser_channel,
                        browser_binary=kwargs["package_name"],
                    )
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                raise WptrunError("Unable to locate or install chromedriver binary")


class ChromeAndroid(ChromeAndroidBase):
    name = "chrome_android"
    browser_cls = browser.ChromeAndroid

    def setup_kwargs(self, kwargs):
        super().setup_kwargs(kwargs)
        if kwargs["browser_channel"] in self.experimental_channels:
            # HACK(Hexcles): work around https://github.com/web-platform-tests/wpt/issues/16448
            kwargs["webdriver_args"].append("--disable-build-check")
            if kwargs["enable_experimental"] is None:
                logger.info("Automatically turning on experimental features for Chrome Dev/Canary")
                kwargs["enable_experimental"] = True


class ChromeiOS(BrowserSetup):
    name = "chrome_ios"
    browser_cls = browser.ChromeiOS

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            raise WptrunError("Unable to locate or install chromedriver binary")


class AndroidWebview(ChromeAndroidBase):
    name = "android_webview"
    browser_cls = browser.AndroidWebview

    def setup_kwargs(self, kwargs):
        if kwargs["mojojs_path"]:
            kwargs["enable_mojojs"] = True
            logger.info("--mojojs-path is provided, enabling MojoJS")


class Opera(BrowserSetup):
    name = "opera"
    browser_cls = browser.Opera

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = None
            if not kwargs["install_webdriver"]:
                webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("operadriver")

                if install:
                    logger.info("Downloading operadriver")
                    webdriver_binary = self.browser.install_webdriver(
                        dest=self.venv.bin_path,
                        channel=kwargs["browser_channel"])
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                raise WptrunError("Unable to locate or install operadriver binary")


class Edge(BrowserSetup):
    name = "MicrosoftEdge"
    browser_cls = browser.Edge
    experimental_channels: ClassVar[Tuple[str, ...]] = ("dev", "canary")

    def setup_kwargs(self, kwargs):
        browser_channel = kwargs["browser_channel"]
        if kwargs["binary"] is None:
            binary = self.browser.find_binary(venv_path=self.venv.path, channel=browser_channel)
            if binary:
                kwargs["binary"] = binary
            else:
                raise WptrunError(f"Unable to locate {self.name.capitalize()} binary")

        if kwargs["mojojs_path"]:
            kwargs["enable_mojojs"] = True
            logger.info("--mojojs-path is provided, enabling MojoJS")
        else:
            path = self.browser.install_mojojs(dest=self.venv.path,
                                               browser_binary=kwargs["binary"])
            if path:
                kwargs["mojojs_path"] = path
                kwargs["enable_mojojs"] = True
                logger.info(f"MojoJS enabled automatically (mojojs_path: {path})")
            else:
                kwargs["enable_mojojs"] = False
                logger.info("MojoJS is disabled for this run.")

        if kwargs["webdriver_binary"] is None:
            webdriver_binary = None
            if not kwargs["install_webdriver"]:
                webdriver_binary = self.browser.find_webdriver(self.venv.bin_path)
                if webdriver_binary and not self.browser.webdriver_supports_browser(
                        webdriver_binary, kwargs["binary"], browser_channel):
                    webdriver_binary = None

            if webdriver_binary is None:
                install = self.prompt_install("msedgedriver")

                if install:
                    webdriver_binary = self.browser.install_webdriver(
                        dest=self.venv.bin_path,
                        channel=browser_channel,
                        browser_binary=kwargs["binary"],
                    )
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                raise WptrunError("Unable to locate or install matching msedgedriver binary")
        if browser_channel in self.experimental_channels:
            # HACK(Hexcles): work around https://github.com/web-platform-tests/wpt/issues/16448
            kwargs["webdriver_args"].append("--disable-build-check")
            if kwargs["enable_experimental"] is None:
                logger.info(
                    "Automatically turning on experimental features for Microsoft Edge Dev/Canary")
                kwargs["enable_experimental"] = True
            if kwargs["enable_webtransport_h3"] is None:
                # To start the WebTransport over HTTP/3 test server.
                kwargs["enable_webtransport_h3"] = True
        if os.getenv("TASKCLUSTER_ROOT_URL"):
            # We are on Taskcluster, where our Docker container does not have
            # enough capabilities to run Microsoft Edge with sandboxing. (gh-20133)
            kwargs["binary_args"].append("--no-sandbox")


class Safari(BrowserSetup):
    name = "safari"
    browser_cls = browser.Safari

    def install(self, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver(channel=kwargs["browser_channel"])

            if webdriver_binary is None:
                raise WptrunError("Unable to locate safaridriver binary")

            kwargs["webdriver_binary"] = webdriver_binary


class Sauce(BrowserSetup):
    name = "sauce"
    browser_cls = browser.Sauce

    def install(self, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        if kwargs["sauce_browser"] is None:
            raise WptrunError("Missing required argument --sauce-browser")
        if kwargs["sauce_version"] is None:
            raise WptrunError("Missing required argument --sauce-version")
        kwargs["test_types"] = ["testharness", "reftest"]


class Servo(BrowserSetup):
    name = "servo"
    browser_cls = browser.Servo

    def install(self, channel=None):
        if self.prompt_install(self.name):
            return self.browser.install(self.venv.path)

    def setup_kwargs(self, kwargs):
        if kwargs["binary"] is None:
            binary = self.browser.find_binary(self.venv.path, None)

            if binary is None:
                raise WptrunError("Unable to find servo binary in PATH")
            kwargs["binary"] = binary


class ServoWebDriver(Servo):
    name = "servodriver"
    browser_cls = browser.ServoWebDriver


class WebKit(BrowserSetup):
    name = "webkit"
    browser_cls = browser.WebKit

    def install(self, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        pass

class Ladybird(BrowserSetup):
    name = "ladybird"
    browser_cls = browser.Ladybird

    def install(self, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        pass

class WebKitTestRunner(BrowserSetup):
    name = "wktr"
    browser_cls = browser.WebKitTestRunner

    def install(self, channel=None):
        if self.prompt_install(self.name):
            return self.browser.install(self.venv.path, channel=channel)

    def setup_kwargs(self, kwargs):
        if kwargs["binary"] is None:
            binary = self.browser.find_binary(self.venv.path, channel=kwargs["browser_channel"])

            if binary is None:
                raise WptrunError("Unable to find binary in PATH")
            kwargs["binary"] = binary


class WebKitGTKMiniBrowser(BrowserSetup):
    name = "webkitgtk_minibrowser"
    browser_cls = browser.WebKitGTKMiniBrowser

    def install(self, channel=None):
        if self.prompt_install(self.name):
            return self.browser.install(self.venv.path, channel, self.prompt)

    def setup_kwargs(self, kwargs):
        if kwargs["binary"] is None:
            binary = self.browser.find_binary(
                venv_path=self.venv.path, channel=kwargs["browser_channel"])

            if binary is None:
                raise WptrunError("Unable to find MiniBrowser binary")
            kwargs["binary"] = binary

        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver(
                venv_path=self.venv.path, channel=kwargs["browser_channel"])

            if webdriver_binary is None:
                raise WptrunError("Unable to find WebKitWebDriver in PATH")
            kwargs["webdriver_binary"] = webdriver_binary


class Epiphany(BrowserSetup):
    name = "epiphany"
    browser_cls = browser.Epiphany

    def install(self, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        if kwargs["binary"] is None:
            binary = self.browser.find_binary()

            if binary is None:
                raise WptrunError("Unable to find epiphany in PATH")
            kwargs["binary"] = binary

        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                raise WptrunError("Unable to find WebKitWebDriver in PATH")
            kwargs["webdriver_binary"] = webdriver_binary


product_setup = {
    "android_webview": AndroidWebview,
    "firefox": Firefox,
    "firefox_android": FirefoxAndroid,
    "chrome": Chrome,
    "chrome_android": ChromeAndroid,
    "chrome_ios": ChromeiOS,
    "chromium": Chromium,
    "edge": Edge,
    "headless_shell": HeadlessShell,
    "safari": Safari,
    "servo": Servo,
    "servodriver": ServoWebDriver,
    "sauce": Sauce,
    "opera": Opera,
    "webkit": WebKit,
    "wktr": WebKitTestRunner,
    "webkitgtk_minibrowser": WebKitGTKMiniBrowser,
    "epiphany": Epiphany,
    "ladybird": Ladybird,
}


def setup_logging(kwargs, default_config=None, formatter_defaults=None):
    import mozlog
    from wptrunner import wptrunner

    global logger

    # Use the grouped formatter by default where mozlog 3.9+ is installed
    if default_config is None:
        if hasattr(mozlog.formatters, "GroupingFormatter"):
            default_formatter = "grouped"
        else:
            default_formatter = "mach"
        default_config = {default_formatter: sys.stdout}
    wptrunner.setup_logging(kwargs, default_config, formatter_defaults=formatter_defaults)
    logger = wptrunner.logger
    return logger


def setup_wptrunner(venv, **kwargs):
    from wptrunner import wptcommandline

    kwargs = kwargs.copy()

    kwargs["product"] = kwargs["product"].replace("-", "_")

    check_environ(kwargs["product"])
    args_general(kwargs)

    if kwargs["product"] not in product_setup:
        if kwargs["product"] == "edgechromium":
            raise WptrunError("edgechromium has been renamed to edge.")

        raise WptrunError("Unsupported product %s" % kwargs["product"])

    setup_cls = product_setup[kwargs["product"]](venv, kwargs["prompt"])
    if not venv.skip_virtualenv_setup:
        requirements = [os.path.join(wpt_root, "tools", "wptrunner", "requirements.txt")]
        requirements.extend(setup_cls.requirements())
        venv.install_requirements(*requirements)

    affected_revish = kwargs.get("affected")
    if affected_revish is not None:
        files_changed, _ = testfiles.files_changed(
            affected_revish, include_uncommitted=True, include_new=True)
        # TODO: Perhaps use wptrunner.testloader.ManifestLoader here
        # and remove the manifest-related code from testfiles.
        # https://github.com/web-platform-tests/wpt/issues/14421
        tests_changed, tests_affected = testfiles.affected_testfiles(
            files_changed, manifest_path=kwargs.get("manifest_path"), manifest_update=kwargs["manifest_update"])
        test_list = tests_changed | tests_affected
        logger.info("Identified %s affected tests" % len(test_list))
        test_list = [os.path.relpath(item, wpt_root) for item in test_list]
        kwargs["test_list"] += test_list
        kwargs["default_exclude"] = True

    if kwargs["install_browser"] and not kwargs["channel"]:
        logger.info("--install-browser is given but --channel is not set, default to nightly channel")
        kwargs["channel"] = "nightly"

    if kwargs["channel"]:
        channel = install.get_channel(kwargs["product"], kwargs["channel"])
        if channel is not None:
            if channel != kwargs["channel"]:
                logger.info("Interpreting channel '%s' as '%s'" % (kwargs["channel"],
                                                                   channel))
            kwargs["browser_channel"] = channel
        else:
            logger.info("Valid channels for %s not known; using argument unmodified" %
                        kwargs["product"])
            kwargs["browser_channel"] = kwargs["channel"]

    if kwargs["install_browser"]:
        logger.info("Installing browser")
        kwargs["binary"] = setup_cls.install(channel=channel)

    setup_cls.setup(kwargs)

    # Remove kwargs we handle here
    wptrunner_kwargs = kwargs.copy()
    for kwarg in ["affected",
                  "install_browser",
                  "install_webdriver",
                  "channel",
                  "prompt",
                  "logcat_dir"]:
        del wptrunner_kwargs[kwarg]

    wptcommandline.check_args(wptrunner_kwargs)

    # Only update browser_version if it was not given as a command line
    # argument, so that it can be overridden on the command line.
    if not wptrunner_kwargs["browser_version"]:
        wptrunner_kwargs["browser_version"] = setup_cls.browser.version(
            binary=wptrunner_kwargs.get("binary") or wptrunner_kwargs.get("package_name"),
            webdriver_binary=wptrunner_kwargs.get("webdriver_binary"),
        )

    return setup_cls, wptrunner_kwargs


def run(venv, **kwargs):
    setup_logging(kwargs)

    setup_cls, wptrunner_kwargs = setup_wptrunner(venv, **kwargs)

    try:
        rv = run_single(venv, **wptrunner_kwargs) > 0
    finally:
        setup_cls.teardown()

    return rv


def run_single(venv, **kwargs):
    from wptrunner import wptrunner
    return wptrunner.start(**kwargs)
