import argparse
import os
import platform
import shutil
import subprocess
import sys
import tarfile
from distutils.spawn import find_executable

import localpaths
from browserutils import browser, utils, virtualenv
logger = None

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))


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
    parser = argparse.ArgumentParser()
    parser.add_argument("product", action="store",
                        help="Browser to run tests in")
    parser.add_argument("tests", action="store", nargs="*",
                        help="Path to tests to run")
    parser.add_argument("wptrunner_args", nargs=argparse.REMAINDER,
                        help="Arguments to pass through to wptrunner")
    parser.add_argument("--yes", "-y", dest="prompt", action="store_false", default=True,
                        help="Don't prompt before installing components")
    parser.add_argument("--wptrunner-help",
                        action=WptrunnerHelpAction, default=argparse.SUPPRESS,
                        help="Print wptrunner help")
    return parser


def exit(msg):
    logger.error(msg)
    sys.exit(1)


def args_general(kwargs):
    kwargs.set_if_none("tests_root", wpt_root)
    kwargs.set_if_none("metadata_root", wpt_root)
    kwargs.set_if_none("manifest_update", True)

    if kwargs["ssl_type"] == "openssl":
        if not find_executable(kwargs["openssl_binary"]):
            if os.uname()[0] == "Windows":
                exit("""OpenSSL binary not found. If you need HTTPS tests, install OpenSSL from

https://slproweb.com/products/Win32OpenSSL.html

Ensuring that libraries are added to /bin and add the resulting bin directory to
your PATH.

Otherwise run with --ssl-type=none""")
            else:
                exit("""OpenSSL not found. If you don't need HTTPS support run with --ssl-type=none,
otherwise install OpenSSL and ensure that it's on your $PATH.""")


def check_environ(product):
    if product != "firefox":
        expected_hosts = set(["web-platform.test",
                              "www.web-platform.test",
                              "www1.web-platform.test",
                              "www2.web-platform.test",
                              "xn--n8j6ds53lwwkrqhv28a.web-platform.test",
                              "xn--lve-6lad.web-platform.test",
                              "nonexistent-origin.web-platform.test"])
        if platform.uname()[0] != "Windows":
            hosts_path = "/etc/hosts"
        else:
            hosts_path = "C:\Windows\System32\drivers\etc\hosts"
        with open(hosts_path, "r") as f:
            for line in f:
                line = line.split("#", 1)[0].strip()
                parts = line.split()
                if len(parts) == 2:
                    host = parts[1]
                    expected_hosts.discard(host)
            if expected_hosts:
                exit("""Missing hosts file configuration for %s.
See README.md for more details.""" % ",".join(expected_hosts))

def prompt_install(component, prompt):
    if not prompt:
        return True
    while True:
        resp = raw_input("Download and install %s [Y/n]? " % component).strip().lower()
        if not resp or resp == "y":
            return True
        elif resp == "n":
            return False


def args_firefox(venv, kwargs, firefox, prompt=True):
    if kwargs["binary"] is None:
        binary = firefox.find_binary()
        if binary is None:
            exit("""Firefox binary not found on $PATH.

Install Firefox or use --binary to set the binary path""")
        kwargs["binary"] = binary

    if kwargs["certutil_binary"] is None and kwargs["ssl_type"] != "none":
        certutil = firefox.find_certutil()

        if certutil is None:
            # Can't download this for now because it's missing the libnss3 library
            exit("""Can't find certutil.

This must be installed using your OS package manager or directly e.g.

Debian/Ubuntu:
sudo apt install libnss3-tools

macOS/Homebrew:
brew install nss

Others:
Download the firefox archive and common.tests.zip archive for your platform
from
https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/
Then extract certutil[.exe] from the tests.zip package and
libnss3[.so|.dll|.dynlib] and but the former on your path and the latter on
your library path.
""")
        else:
            print("Using certutil %s" % certutil)

        if certutil is not None:
            kwargs["certutil_binary"] = certutil
        else:
            print("Unable to find or install certutil, setting ssl-type to none")
            kwargs["ssl_type"] = "none"

    if kwargs["webdriver_binary"] is None and "wdspec" in kwargs["test_types"]:
        webdriver_binary = firefox.find_webdriver()

        if webdriver_binary is None:
            install = prompt_install("geckodriver", prompt)

            if install:
                print("Downloading geckodriver")
                webdriver_binary = firefox.install_webdriver(dest=venv.bin_path)
        else:
            print("Using webdriver binary %s" % webdriver_binary)

        if webdriver_binary:
            kwargs["webdriver_binary"] = webdriver_binary
        else:
            print("Unable to find or install geckodriver, skipping wdspec tests")
            kwargs["test_types"].remove("wdspec")

    if kwargs["prefs_root"] is None:
        print("Downloading gecko prefs")
        prefs_root = firefox.install_prefs(venv.path)
        kwargs["prefs_root"] = prefs_root


def setup_firefox(venv, kwargs, prompt=True):
    firefox = browser.Firefox()
    args_firefox(venv, kwargs, firefox, prompt)
    venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", firefox.requirements))


def args_chrome(venv, kwargs, chrome, prompt=True):
    if kwargs["webdriver_binary"] is None:
        webdriver_binary = chrome.find_webdriver()

        if webdriver_binary is None:
            install = prompt_install("chromedriver", prompt)

            if install:
                print("Downloading chromedriver")
                webdriver_binary = chrome.install_webdriver(dest=venv.bin_path)
        else:
            print("Using webdriver binary %s" % webdriver_binary)

        if webdriver_binary:
            kwargs["webdriver_binary"] = webdriver_binary
        else:
            exit("Unable to locate or install chromedriver binary")


def setup_chrome(venv, kwargs, prompt=True):
    chrome = browser.Chrome()
    args_chrome(venv, kwargs, chrome, prompt)
    venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", chrome.requirements))


def args_edge(venv, kwargs, edge, prompt=True):
    if kwargs["webdriver_binary"] is None:
        webdriver_binary = edge.find_webdriver()

        if webdriver_binary is None:
            exit("""Unable to find WebDriver and we aren't yet clever enough to work out which
version to download. Please go to the following URL and install the correct
version for your Edge/Windows release somewhere on the %PATH%:

https://developer.microsoft.com/en-us/microsoft-edge/tools/webdriver/
 """)
        kwargs["webdriver_binary"] = webdriver_binary


def setup_edge(venv, kwargs, prompt=True):
    edge = browser.Edge()
    args_edge(venv, kwargs, edge, prompt)
    venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", edge.requirements))


def setup_sauce(kwargs):
    raise NotImplementedError


def args_servo(venv, kwargs, servo, prompt=True):
    if kwargs["binary"] is None:
        binary = servo.find_binary()

        if binary is None:
            exit("Unable to find servo binary on the PATH")
        kwargs["binary"] = binary


def setup_servo(venv, kwargs, prompt=True):
    servo = browser.Servo()
    args_servo(venv, kwargs, servo, prompt)
    venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", servo.requirements))


product_setup = {
    "firefox": setup_firefox,
    "chrome": setup_chrome,
    "edge": setup_edge,
    "servo": setup_servo
}


def setup_wptrunner(venv, product, tests, wptrunner_args, prompt=True,):
    from wptrunner import wptrunner, wptcommandline

    global logger

    wptparser = wptcommandline.create_parser()
    kwargs = utils.Kwargs(vars(wptparser.parse_args(wptrunner_args)).iteritems())

    wptrunner.setup_logging(kwargs, {"mach": sys.stdout})
    logger = wptrunner.logger

    kwargs["product"] = product
    kwargs["test_list"] = tests

    check_environ(product)
    args_general(kwargs)

    if product not in product_setup:
        exit("Unsupported product %s" % product)

    product_setup[product](venv, kwargs, prompt)

    wptcommandline.check_args(kwargs)

    wptrunner_path = os.path.join(wpt_root, "tools", "wptrunner")

    venv.install_requirements(os.path.join(wptrunner_path, "requirements.txt"))

    return kwargs


def main():
    parser = create_parser()
    args = parser.parse_args()

    venv = virtualenv.Virtualenv(os.path.join(wpt_root, "_venv_%s") % platform.uname()[0])
    venv.start()
    venv.install_requirements(os.path.join(wpt_root, "tools", "wptrunner", "requirements.txt"))
    venv.install("requests")

    kwargs = setup_wptrunner(venv, args.product, args.tests, args.wptrunner_args, prompt=args.prompt)
    from wptrunner import wptrunner
    wptrunner.start(**kwargs)


if __name__ == "__main__":
    import pdb
    try:
        main()
    except:
        pdb.post_mortem()
