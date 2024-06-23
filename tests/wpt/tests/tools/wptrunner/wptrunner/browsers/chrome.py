# mypy: allow-untyped-defs

from . import chrome_spki_certs
from .base import WebDriverBrowser, require_arg
from .base import NullBrowser  # noqa: F401
from .base import get_timeout_multiplier   # noqa: F401
from .base import cmd_arg
from ..executors import executor_kwargs as base_executor_kwargs
from ..executors.base import WdspecExecutor  # noqa: F401
from ..executors.executorchrome import (  # noqa: F401
    ChromeDriverPrintRefTestExecutor,
    ChromeDriverRefTestExecutor,
    ChromeDriverTestharnessExecutor,
    ChromeDriverCrashTestExecutor
)


__wptrunner__ = {"product": "chrome",
                 "check_args": "check_args",
                 "browser": "ChromeBrowser",
                 "executor": {"testharness": "ChromeDriverTestharnessExecutor",
                              "reftest": "ChromeDriverRefTestExecutor",
                              "print-reftest": "ChromeDriverPrintRefTestExecutor",
                              "wdspec": "WdspecExecutor",
                              "crashtest": "ChromeDriverCrashTestExecutor"},
                 "browser_kwargs": "browser_kwargs",
                 "executor_kwargs": "executor_kwargs",
                 "env_extras": "env_extras",
                 "env_options": "env_options",
                 "update_properties": "update_properties",
                 "timeout_multiplier": "get_timeout_multiplier",}


def debug_args(debug_info):
    if debug_info.interactive:
        # Keep in sync with:
        # https://chromium.googlesource.com/chromium/src/+/main/third_party/blink/tools/debug_renderer
        return [
            "--no-sandbox",
            "--disable-hang-monitor",
            "--wait-for-debugger-on-navigation",
        ]
    return []


def check_args(**kwargs):
    require_arg(kwargs, "webdriver_binary")


def browser_kwargs(logger, test_type, run_info_data, config, **kwargs):
    return {"binary": kwargs["binary"],
            "webdriver_binary": kwargs["webdriver_binary"],
            "webdriver_args": kwargs.get("webdriver_args")}


def executor_kwargs(logger, test_type, test_environment, run_info_data, subsuite,
                    **kwargs):
    sanitizer_enabled = kwargs.get("sanitizer_enabled")
    if sanitizer_enabled:
        test_type = "crashtest"
    executor_kwargs = base_executor_kwargs(test_type, test_environment, run_info_data,
                                           subsuite, **kwargs)
    executor_kwargs["close_after_done"] = True
    executor_kwargs["sanitizer_enabled"] = sanitizer_enabled
    executor_kwargs["reuse_window"] = kwargs.get("reuse_window", False)

    capabilities = {
        "goog:chromeOptions": {
            "prefs": {
                "profile": {
                    "default_content_setting_values": {
                        "popups": 1
                    }
                }
            },
            "excludeSwitches": ["enable-automation"],
            "w3c": True,
        }
    }

    chrome_options = capabilities["goog:chromeOptions"]
    if kwargs["binary"] is not None:
        chrome_options["binary"] = kwargs["binary"]

    # Here we set a few Chrome flags that are always passed.
    # ChromeDriver's "acceptInsecureCerts" capability only controls the current
    # browsing context, whereas the CLI flag works for workers, too.
    chrome_options["args"] = []

    chrome_options["args"].append("--ignore-certificate-errors-spki-list=%s" %
                                  ','.join(chrome_spki_certs.IGNORE_CERTIFICATE_ERRORS_SPKI_LIST))

    # Allow audio autoplay without a user gesture.
    chrome_options["args"].append("--autoplay-policy=no-user-gesture-required")
    # Allow WebRTC tests to call getUserMedia and getDisplayMedia.
    chrome_options["args"].append("--use-fake-device-for-media-stream")
    chrome_options["args"].append("--use-fake-ui-for-media-stream")
    # Use a fake UI for FedCM to allow testing it.
    chrome_options["args"].append("--use-fake-ui-for-fedcm")
    # This is needed until https://github.com/web-platform-tests/wpt/pull/40709
    # is merged.
    chrome_options["args"].append("--enable-features=FedCmWithoutWellKnownEnforcement")
    # Use a fake UI for digital identity to allow testing it.
    chrome_options["args"].append("--use-fake-ui-for-digital-identity")
    # Shorten delay for Reporting <https://w3c.github.io/reporting/>.
    chrome_options["args"].append("--short-reporting-delay")
    # Point all .test domains to localhost for Chrome
    chrome_options["args"].append("--host-resolver-rules=MAP nonexistent.*.test ^NOTFOUND, MAP *.test 127.0.0.1, MAP *.test. 127.0.0.1")
    # Enable Secure Payment Confirmation for Chrome. This is normally disabled
    # on Linux as it hasn't shipped there yet, but in WPT we enable virtual
    # authenticator devices anyway for testing and so SPC works.
    chrome_options["args"].append("--enable-features=SecurePaymentConfirmationBrowser")
    # For WebTransport tests.
    chrome_options["args"].append("--webtransport-developer-mode")
    # The GenericSensorExtraClasses flag enables the browser-side
    # implementation of sensors such as Ambient Light Sensor.
    chrome_options["args"].append("--enable-features=GenericSensorExtraClasses")
    # Do not show Chrome for Testing infobar. For other Chromium build this
    # flag is no-op. Required to avoid flakiness in tests, as the infobar
    # changes the viewport, which can happen during the test run.
    chrome_options["args"].append("--disable-infobars")

    # Classify `http-private`, `http-public` and https variants in the
    # appropriate IP address spaces.
    # For more details, see: https://github.com/web-platform-tests/rfcs/blob/master/rfcs/address_space_overrides.md
    address_space_overrides_ports = [
        ("http-private", "private"),
        ("http-public", "public"),
        ("https-private", "private"),
        ("https-public", "public"),
    ]
    address_space_overrides_arg = ",".join(
        f"127.0.0.1:{port_number}={address_space}"
        for port_name, address_space in address_space_overrides_ports
        for port_number in test_environment.config.ports.get(port_name, [])
    )
    if address_space_overrides_arg:
        chrome_options["args"].append(
            "--ip-address-space-overrides=" + address_space_overrides_arg)

    # Always disable antialiasing on the Ahem font.
    blink_features = ['DisableAhemAntialias']

    if kwargs["enable_mojojs"]:
        blink_features.append('MojoJS')
        blink_features.append('MojoJSTest')

    chrome_options["args"].append("--enable-blink-features=" + ','.join(blink_features))

    if kwargs["enable_swiftshader"]:
        # https://chromium.googlesource.com/chromium/src/+/HEAD/docs/gpu/swiftshader.md
        chrome_options["args"].extend(["--use-gl=angle", "--use-angle=swiftshader"])

    if kwargs["enable_experimental"]:
        chrome_options["args"].extend(["--enable-experimental-web-platform-features"])

    # Copy over any other flags that were passed in via `--binary-arg`
    for arg in kwargs.get("binary_args", []):
        if arg not in chrome_options["args"]:
            chrome_options["args"].append(arg)

    for arg in subsuite.config.get("binary_args", []):
        if arg not in chrome_options["args"]:
            chrome_options["args"].append(arg)

    # Pass the --headless=new flag to Chrome if WPT's own --headless flag was
    # set. '--headless' should always mean the new headless mode, as the old
    # headless mode is not used anyway.
    if kwargs["headless"] and ("--headless=new" not in chrome_options["args"] and
                               "--headless=old" not in chrome_options["args"] and
                               "--headless" not in chrome_options["args"]):
        chrome_options["args"].append("--headless=new")

    if test_type == "wdspec":
        executor_kwargs["binary_args"] = chrome_options["args"]

    executor_kwargs["capabilities"] = capabilities

    return executor_kwargs


def env_extras(**kwargs):
    return []


def env_options():
    # TODO(crbug.com/1440021): Support text-based debuggers for `chrome` through
    # `chromedriver`.
    return {"server_host": "127.0.0.1"}


def update_properties():
    return (["debug", "os", "processor"], {"os": ["version"], "processor": ["bits"]})


class ChromeBrowser(WebDriverBrowser):
    def restart_on_test_type_change(self, new_test_type: str, old_test_type: str) -> bool:
        # Restart the test runner when switch from/to wdspec tests. Wdspec test
        # is using a different protocol class so a restart is always needed.
        if "wdspec" in [old_test_type, new_test_type]:
            return True
        return False

    def make_command(self):
        return [self.webdriver_binary,
                cmd_arg("port", str(self.port)),
                cmd_arg("url-base", self.base_path),
                cmd_arg("enable-chrome-logs")] + self.webdriver_args
