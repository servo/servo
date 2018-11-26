import argparse
import os
import platform
import sys
from distutils.spawn import find_executable
from six.moves import input

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, os.pardir))
sys.path.insert(0, os.path.abspath(os.path.join(wpt_root, "tools")))

from . import browser, install, utils, virtualenv
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
        super(WptrunnerHelpAction, self).__init__(
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

    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("product", action="store",
                        help="Browser to run tests in")
    parser.add_argument("--yes", "-y", dest="prompt", action="store_false", default=True,
                        help="Don't prompt before installing components")
    parser.add_argument("--install-browser", action="store_true",
                        help="Install the browser from the release channel specified by --channel "
                        "(or the nightly channel by default).")
    parser.add_argument("--channel", action="store",
                        choices=install.channel_by_name.keys(),
                        default=None, help='Name of browser release channel. '
                        '"stable" and "release" are synonyms for the latest browser stable '
                        'release, "nightly", "dev", "experimental", and "preview" are all '
                        'synonyms for the latest available development release. (For WebDriver '
                        'installs, we attempt to select an appropriate, compatible version for '
                        'the latest browser release on the selected channel.) '
                        'This flag overrides --browser-channel.')
    parser._add_container_actions(wptcommandline.create_parser())
    return parser


def exit(msg):
    logger.error(msg)
    sys.exit(1)


def args_general(kwargs):
    kwargs.set_if_none("tests_root", wpt_root)
    kwargs.set_if_none("metadata_root", wpt_root)
    kwargs.set_if_none("manifest_update", True)
    kwargs.set_if_none("manifest_download", True)

    if kwargs["ssl_type"] in (None, "pregenerated"):
        cert_root = os.path.join(wpt_root, "tools", "certs")
        if kwargs["ca_cert_path"] is None:
            kwargs["ca_cert_path"] = os.path.join(cert_root, "cacert.pem")

        if kwargs["host_key_path"] is None:
            kwargs["host_key_path"] = os.path.join(cert_root, "web-platform.test.key")

        if kwargs["host_cert_path"] is None:
            kwargs["host_cert_path"] = os.path.join(cert_root, "web-platform.test.pem")
    elif kwargs["ssl_type"] == "openssl":
        if not find_executable(kwargs["openssl_binary"]):
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
    if product not in ("firefox", "servo"):
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
            hosts_path = "%s\System32\drivers\etc\hosts" % os.environ.get("SystemRoot", "C:\Windows")
        else:
            hosts_path = "/etc/hosts"

        with open(hosts_path, "r") as f:
            for line in f:
                line = line.split("#", 1)[0].strip()
                parts = line.split()
                hosts = parts[1:]
                for host in hosts:
                    missing_hosts.discard(host)
            if missing_hosts:
                if is_windows:
                    message = """Missing hosts file configuration. Run

python wpt make-hosts-file | Out-File %s -Encoding ascii -Append

in PowerShell with Administrator privileges.""" % hosts_path
                else:
                    message = """Missing hosts file configuration. Run

./wpt make-hosts-file | sudo tee -a %s""" % hosts_path
                raise WptrunError(message)


class BrowserSetup(object):
    name = None
    browser_cls = None

    def __init__(self, venv, prompt=True, sub_product=None):
        self.browser = self.browser_cls()
        self.venv = venv
        self.prompt = prompt
        self.sub_product = sub_product

    def prompt_install(self, component):
        if not self.prompt:
            return True
        while True:
            resp = input("Download and install %s [Y/n]? " % component).strip().lower()
            if not resp or resp == "y":
                return True
            elif resp == "n":
                return False

    def install(self, venv, channel=None):
        if self.prompt_install(self.name):
            return self.browser.install(venv.path, channel)

    def install_requirements(self):
        self.venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", self.browser.requirements))

    def setup(self, kwargs):
        self.setup_kwargs(kwargs)


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
            webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("geckodriver")

                if install:
                    logger.info("Downloading geckodriver")
                    webdriver_binary = self.browser.install_webdriver(dest=self.venv.bin_path)
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

        if kwargs["headless"] is None:
            kwargs["headless"] = True
            logger.info("Running in headless mode, pass --no-headless to disable")

        # Turn off Firefox WebRTC ICE logging on WPT (turned on by mozrunner)
        os.unsetenv('R_LOG_LEVEL')
        os.unsetenv('R_LOG_DESTINATION')
        os.unsetenv('R_LOG_VERBOSE')

        # Allow WebRTC tests to call getUserMedia.
        kwargs["extra_prefs"].append("media.navigator.streams.fake=true")


class Fennec(BrowserSetup):
    name = "fennec"
    browser_cls = browser.Fennec

    def setup_kwargs(self, kwargs):
        pass


class Chrome(BrowserSetup):
    name = "chrome"
    browser_cls = browser.Chrome

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("chromedriver")

                if install:
                    logger.info("Downloading chromedriver")
                    webdriver_binary = self.browser.install_webdriver(dest=self.venv.bin_path)
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                raise WptrunError("Unable to locate or install chromedriver binary")
        if kwargs["browser_channel"] == "dev":
            logger.info("Automatically turning on experimental features for Chrome Dev")
            kwargs["binary_args"].append("--enable-experimental-web-platform-features")
            # TODO(foolip): remove after unified plan is enabled on Chrome stable
            kwargs["binary_args"].append("--enable-features=RTCUnifiedPlanByDefault")

        # Allow audio autoplay without a user gesture.
        kwargs["binary_args"].append("--autoplay-policy=no-user-gesture-required")

        # Allow WebRTC tests to call getUserMedia.
        kwargs["binary_args"] += ["--use-fake-ui-for-media-stream", "--use-fake-device-for-media-stream"]


class ChromeAndroid(BrowserSetup):
    name = "chrome_android"
    browser_cls = browser.ChromeAndroid

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("chromedriver")

                if install:
                    logger.info("Downloading chromedriver")
                    webdriver_binary = self.browser.install_webdriver(dest=self.venv.bin_path)
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                raise WptrunError("Unable to locate or install chromedriver binary")


class Opera(BrowserSetup):
    name = "opera"
    browser_cls = browser.Opera

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                install = self.prompt_install("operadriver")

                if install:
                    logger.info("Downloading operadriver")
                    webdriver_binary = self.browser.install_webdriver(dest=self.venv.bin_path)
            else:
                logger.info("Using webdriver binary %s" % webdriver_binary)

            if webdriver_binary:
                kwargs["webdriver_binary"] = webdriver_binary
            else:
                raise WptrunError("Unable to locate or install operadriver binary")


class Edge(BrowserSetup):
    name = "edge"
    browser_cls = browser.Edge

    def install(self, venv, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                raise WptrunError("""Unable to find WebDriver and we aren't yet clever enough to work out which
version to download. Please go to the following URL and install the correct
version for your Edge/Windows release somewhere on the %PATH%:

https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/
""")
            kwargs["webdriver_binary"] = webdriver_binary


class EdgeWebDriver(Edge):
    name = "edge_webdriver"
    browser_cls = browser.EdgeWebDriver


class InternetExplorer(BrowserSetup):
    name = "ie"
    browser_cls = browser.InternetExplorer

    def install(self, venv, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        if kwargs["webdriver_binary"] is None:
            webdriver_binary = self.browser.find_webdriver()

            if webdriver_binary is None:
                raise WptrunError("""Unable to find WebDriver and we aren't yet clever enough to work out which
version to download. Please go to the following URL and install the driver for Internet Explorer
somewhere on the %PATH%:

https://selenium-release.storage.googleapis.com/index.html
""")
            kwargs["webdriver_binary"] = webdriver_binary


class Safari(BrowserSetup):
    name = "safari"
    browser_cls = browser.Safari

    def install(self, venv, channel=None):
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

    def install(self, venv, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        kwargs.set_if_none("sauce_browser", self.sub_product[0])
        kwargs.set_if_none("sauce_version", self.sub_product[1])
        kwargs["test_types"] = ["testharness", "reftest"]


class Servo(BrowserSetup):
    name = "servo"
    browser_cls = browser.Servo

    def install(self, venv, channel=None):
        if self.prompt_install(self.name):
            return self.browser.install(venv.path)

    def setup_kwargs(self, kwargs):
        if kwargs["binary"] is None:
            binary = self.browser.find_binary(self.venv.path, None)

            if binary is None:
                raise WptrunError("Unable to find servo binary on the PATH")
            kwargs["binary"] = binary


class ServoWebDriver(Servo):
    name = "servodriver"
    browser_cls = browser.ServoWebDriver


class WebKit(BrowserSetup):
    name = "webkit"
    browser_cls = browser.WebKit

    def install(self, venv, channel=None):
        raise NotImplementedError

    def setup_kwargs(self, kwargs):
        pass


product_setup = {
    "fennec": Fennec,
    "firefox": Firefox,
    "chrome": Chrome,
    "chrome_android": ChromeAndroid,
    "edge": Edge,
    "edge_webdriver": EdgeWebDriver,
    "ie": InternetExplorer,
    "safari": Safari,
    "servo": Servo,
    "servodriver": ServoWebDriver,
    "sauce": Sauce,
    "opera": Opera,
    "webkit": WebKit,
}


def setup_logging(kwargs):
    import mozlog
    from wptrunner import wptrunner

    global logger

    # Use the grouped formatter by default where mozlog 3.9+ is installed
    if hasattr(mozlog.formatters, "GroupingFormatter"):
        default_formatter = "grouped"
    else:
        default_formatter = "mach"
    wptrunner.setup_logging(kwargs, {default_formatter: sys.stdout})
    logger = wptrunner.logger


def setup_wptrunner(venv, prompt=True, install_browser=False, **kwargs):
    from wptrunner import wptcommandline

    kwargs = utils.Kwargs(kwargs.iteritems())

    product_parts = kwargs["product"].split(":")
    kwargs["product"] = product_parts[0]
    sub_product = product_parts[1:]

    check_environ(kwargs["product"])
    args_general(kwargs)

    if kwargs["product"] not in product_setup:
        raise WptrunError("Unsupported product %s" % kwargs["product"])

    setup_cls = product_setup[kwargs["product"]](venv, prompt, sub_product)
    setup_cls.install_requirements()

    if install_browser and not kwargs["channel"]:
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
            logger.info("Valid channels for %s not known; using argument unmodified" % kwargs["product"])
    del kwargs["channel"]

    if install_browser:
        logger.info("Installing browser")
        kwargs["binary"] = setup_cls.install(venv, channel=channel)

    setup_cls.setup(kwargs)

    wptcommandline.check_args(kwargs)

    wptrunner_path = os.path.join(wpt_root, "tools", "wptrunner")

    venv.install_requirements(os.path.join(wptrunner_path, "requirements.txt"))

    kwargs['browser_version'] = setup_cls.browser.version(binary=kwargs.get("binary"),
                                                          webdriver_binary=kwargs.get("webdriver_binary"))
    return kwargs


def run(venv, **kwargs):
    setup_logging(kwargs)

    # Remove arguments that aren't passed to wptrunner
    prompt = kwargs.pop("prompt", True)
    install_browser = kwargs.pop("install_browser", False)

    kwargs = setup_wptrunner(venv,
                             prompt=prompt,
                             install_browser=install_browser,
                             **kwargs)

    rv = run_single(venv, **kwargs) > 0

    return rv


def run_single(venv, **kwargs):
    from wptrunner import wptrunner
    return wptrunner.start(**kwargs)


def main():
    try:
        parser = create_parser()
        args = parser.parse_args()

        venv = virtualenv.Virtualenv(os.path.join(wpt_root, "_venv_%s") % platform.uname()[0])
        venv.start()
        venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", "requirements.txt"))
        venv.install("requests")

        return run(venv, vars(args))
    except WptrunError as e:
        exit(e.message)


if __name__ == "__main__":
    import pdb
    from tools import localpaths  # noqa: flake8
    try:
        main()
    except Exception:
        pdb.post_mortem()
