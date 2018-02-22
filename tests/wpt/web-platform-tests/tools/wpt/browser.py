import logging
import os
import platform
import re
import shutil
import stat
import subprocess
import sys
from abc import ABCMeta, abstractmethod
from ConfigParser import RawConfigParser
from datetime import datetime, timedelta
from distutils.spawn import find_executable
from io import BytesIO

from utils import call, get, untar, unzip

logger = logging.getLogger(__name__)

uname = platform.uname()

def path(path, exe):
    path = path.replace("/", os.path.sep)
    if exe and uname[0] == "Windows":
        path += ".exe"
    return path


class Browser(object):
    __metaclass__ = ABCMeta

    @abstractmethod
    def install(self, dest=None):
        return NotImplemented

    @abstractmethod
    def install_webdriver(self):
        return NotImplemented

    @abstractmethod
    def version(self):
        return NotImplemented

    @abstractmethod
    def requirements(self):
        """Name of the browser-specific wptrunner requirements file"""
        return NotImplemented

    def prepare_environment(self):
        """Do any additional setup of the environment required to start the
           browser successfully
        """
        pass


class Firefox(Browser):
    """Firefox-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "firefox"
    binary = "firefox/firefox"
    platform_ini = "firefox/platform.ini"
    requirements = "requirements_firefox.txt"


    def platform_string(self):
        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "mac"
        }.get(uname[0])

        if platform is None:
            raise ValueError("Unable to construct a valid Firefox package name for current platform")

        if platform == "linux":
            bits = "-%s" % uname[4]
        elif platform == "win":
            bits = "64" if uname[4] == "x86_64" else "32"
        else:
            bits = ""

        return "%s%s" % (platform, bits)

    def platform_string_geckodriver(self):
        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "macos"
        }.get(uname[0])

        if platform is None:
            raise ValueError("Unable to construct a valid Geckodriver package name for current platform")

        if platform in ("linux", "win"):
            bits = "64" if uname[4] == "x86_64" else "32"
        else:
            bits = ""

        return "%s%s" % (platform, bits)

    def latest_nightly_listing(self):
        resp = get("https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/")
        resp.raise_for_status()
        return resp.text

    def get_nightly_link(self, index, platform):
        pattern = re.compile("<a[^>]*>(firefox-(\d+)\.\d(?:\w\d)?.en-US.%s\.tar\.bz2)" % platform)
        max_version = None
        for match in pattern.finditer(index):
            try:
                version = int(match.group(2))
            except ValueError:
                continue
            if max_version is None or version > max_version[0]:
                max_version = (version, match.group(1))
        if not max_version:
            raise ValueError("Failed to find version to download")
        return ("https://archive.mozilla.org/pub/firefox/nightly/latest-mozilla-central/%s" %
                max_version[1])

    def install(self, dest=None):
        """Install Firefox."""
        if dest is None:
            dest = os.getcwd()

        nightly_link = self.get_nightly_link(self.latest_nightly_listing(),
                                             self.platform_string())
        resp = get(nightly_link)
        resp.raise_for_status()
        untar(resp.raw, dest=dest)
        return find_executable("firefox", os.path.join(dest, "firefox"))

    def find_binary(self, path=None):
        return find_executable("firefox", path)

    def find_certutil(self):
        path = find_executable("certutil")
        if path is None:
            return None
        if os.path.splitdrive(path)[1].split(os.path.sep) == ["", "Windows", "system32", "certutil.exe"]:
            return None
        return path

    def find_webdriver(self):
        return find_executable("geckodriver")

    def install_prefs(self, dest=None):
        if dest is None:
            dest = os.pwd

        dest = os.path.join(dest, "profiles")
        if not os.path.exists(dest):
            os.makedirs(dest)
        prefs_path = os.path.join(dest, "prefs_general.js")

        now = datetime.now()
        if (not os.path.exists(prefs_path) or
            (datetime.fromtimestamp(os.stat(prefs_path).st_mtime) <
             now - timedelta(days=2))):
            with open(prefs_path, "wb") as f:
                resp = get("https://hg.mozilla.org/mozilla-central/raw-file/tip/testing/profiles/prefs_general.js")
                f.write(resp.content)

        return dest

    def _latest_geckodriver_version(self):
        """Get and return latest version number for geckodriver."""
        # This is used rather than an API call to avoid rate limits
        tags = call("git", "ls-remote", "--tags", "--refs",
                    "https://github.com/mozilla/geckodriver.git")
        release_re = re.compile(".*refs/tags/v(\d+)\.(\d+)\.(\d+)")
        latest_release = 0
        for item in tags.split("\n"):
            m = release_re.match(item)
            if m:
                version = [int(item) for item in m.groups()]
                if version > latest_release:
                    latest_release = version
        assert latest_release != 0
        return "v%s.%s.%s" % tuple(str(item) for item in latest_release)

    def install_webdriver(self, dest=None):
        """Install latest Geckodriver."""
        if dest is None:
            dest = os.getcwd()

        version = self._latest_geckodriver_version()
        format = "zip" if uname[0] == "Windows" else "tar.gz"
        logger.debug("Latest geckodriver release %s" % version)
        url = ("https://github.com/mozilla/geckodriver/releases/download/%s/geckodriver-%s-%s.%s" %
               (version, version, self.platform_string_geckodriver(), format))
        if format == "zip":
            unzip(get(url).raw, dest=dest)
        else:
            untar(get(url).raw, dest=dest)
        return find_executable(os.path.join(dest, "geckodriver"))

    def version(self, root):
        """Retrieve the release version of the installed browser."""
        platform_info = RawConfigParser()

        with open(os.path.join(root, self.platform_ini), "r") as fp:
            platform_info.readfp(BytesIO(fp.read()))
            return "BuildID %s; SourceStamp %s" % (
                platform_info.get("Build", "BuildID"),
                platform_info.get("Build", "SourceStamp"))


class Chrome(Browser):
    """Chrome-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "chrome"
    binary = "/usr/bin/google-chrome"
    requirements = "requirements_chrome.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def platform_string(self):
        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "mac"
        }.get(uname[0])

        if platform is None:
            raise ValueError("Unable to construct a valid Chrome package name for current platform")

        if platform == "linux":
            bits = "64" if uname[4] == "x86_64" else "32"
        elif platform == "mac":
            bits = "64"
        elif platform == "win":
            bits = "32"

        return "%s%s" % (platform, bits)

    def find_webdriver(self):
        return find_executable("chromedriver")

    def install_webdriver(self, dest=None):
        """Install latest Webdriver."""
        if dest is None:
            dest = os.pwd
        latest = get("http://chromedriver.storage.googleapis.com/LATEST_RELEASE").text.strip()
        url = "http://chromedriver.storage.googleapis.com/%s/chromedriver_%s.zip" % (latest,
                                                                                     self.platform_string())
        unzip(get(url).raw, dest)

        path = find_executable("chromedriver", dest)
        st = os.stat(path)
        os.chmod(path, st.st_mode | stat.S_IEXEC)
        return path

    def version(self, root):
        """Retrieve the release version of the installed browser."""
        output = call(self.binary, "--version")
        return re.search(r"[0-9\.]+( [a-z]+)?$", output.strip()).group(0)

    def prepare_environment(self):
        # https://bugs.chromium.org/p/chromium/issues/detail?id=713947
        logger.debug("DBUS_SESSION_BUS_ADDRESS %s" % os.environ.get("DBUS_SESSION_BUS_ADDRESS"))
        if "DBUS_SESSION_BUS_ADDRESS" not in os.environ:
            if find_executable("dbus-launch"):
                logger.debug("Attempting to start dbus")
                dbus_conf = subprocess.check_output(["dbus-launch"])
                logger.debug(dbus_conf)

                # From dbus-launch(1):
                #
                # > When dbus-launch prints bus information to standard output,
                # > by default it is in a simple key-value pairs format.
                for line in dbus_conf.strip().split("\n"):
                    key, _, value = line.partition("=")
                    os.environ[key] = value
            else:
                logger.critical("dbus not running and can't be started")
                sys.exit(1)


class ChromeAndroid(Browser):
    """Chrome-specific interface for android.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "chrome_android"
    requirements = "requirements_chrome_android.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("chromedriver")

    def install_webdriver(self, dest=None):
        chrome = Chrome()
        return chrome.install_webdriver(dest)

    def version(self, root):
        raise NotImplementedError


class Opera(Browser):
    """Opera-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "opera"
    binary = "/usr/bin/opera"
    requirements = "requirements_opera.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def platform_string(self):
        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "mac"
        }.get(uname[0])

        if platform is None:
            raise ValueError("Unable to construct a valid Opera package name for current platform")

        if platform == "linux":
            bits = "64" if uname[4] == "x86_64" else "32"
        elif platform == "mac":
            bits = "64"
        elif platform == "win":
            bits = "32"

        return "%s%s" % (platform, bits)

    def find_webdriver(self):
        return find_executable("operadriver")

    def install_webdriver(self, dest=None):
        """Install latest Webdriver."""
        if dest is None:
            dest = os.pwd
        latest = get("https://api.github.com/repos/operasoftware/operachromiumdriver/releases/latest").json()["tag_name"]
        url = "https://github.com/operasoftware/operachromiumdriver/releases/download/%s/operadriver_%s.zip" % (latest,
                                                                                                                self.platform_string())
        unzip(get(url).raw, dest)

        operadriver_dir = os.path.join(dest, "operadriver_%s" % self.platform_string())
        shutil.move(os.path.join(operadriver_dir, "operadriver"), dest)
        shutil.rmtree(operadriver_dir)

        path = find_executable("operadriver")
        st = os.stat(path)
        os.chmod(path, st.st_mode | stat.S_IEXEC)
        return path

    def version(self, root):
        """Retrieve the release version of the installed browser."""
        output = call(self.binary, "--version")
        return re.search(r"[0-9\.]+( [a-z]+)?$", output.strip()).group(0)

    def prepare_environment(self):
        # https://bugs.chromium.org/p/chromium/issues/detail?id=713947
        logger.debug("DBUS_SESSION_BUS_ADDRESS %s" % os.environ.get("DBUS_SESSION_BUS_ADDRESS"))
        if "DBUS_SESSION_BUS_ADDRESS" not in os.environ:
            if find_executable("dbus-launch"):
                logger.debug("Attempting to start dbus")
                dbus_conf = subprocess.check_output(["dbus-launch"])
                logger.debug(dbus_conf)

                # From dbus-launch(1):
                #
                # > When dbus-launch prints bus information to standard output,
                # > by default it is in a simple key-value pairs format.
                for line in dbus_conf.strip().split("\n"):
                    key, _, value = line.partition("=")
                    os.environ[key] = value
            else:
                logger.critical("dbus not running and can't be started")
                sys.exit(1)


class Edge(Browser):
    """Edge-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "edge"
    requirements = "requirements_edge.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("MicrosoftWebDriver")

    def install_webdriver(self, dest=None):
        """Install latest Webdriver."""
        raise NotImplementedError

    def version(self):
        raise NotImplementedError


class InternetExplorer(Browser):
    """Internet Explorer-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "ie"
    requirements = "requirements_ie.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("IEDriverServer.exe")

    def install_webdriver(self, dest=None):
        """Install latest Webdriver."""
        raise NotImplementedError

    def version(self):
        raise NotImplementedError


class Servo(Browser):
    """Servo-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "servo"
    requirements = "requirements_servo.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_binary(self, path=None):
        return find_executable("servo")

    def find_webdriver(self):
        return None

    def install_webdriver(self):
        raise NotImplementedError

    def version(self, root):
        return None


class Sauce(Browser):
    """Sauce-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "sauce"
    requirements = "requirements_sauce.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_binary(self, path=None):
        return None

    def find_webdriver(self):
        return None

    def install_webdriver(self):
        raise NotImplementedError

    def version(self, root):
        return None
