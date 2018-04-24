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


class Browser(object):
    __metaclass__ = ABCMeta

    @abstractmethod
    def install(self, dest=None):
        """Install the browser."""
        return NotImplemented

    @abstractmethod
    def install_webdriver(self, dest=None):
        """Install the WebDriver implementation for this browser."""
        return NotImplemented

    @abstractmethod
    def find_binary(self):
        """Find the binary of the browser.

        If the WebDriver for the browser is able to find the binary itself, this
        method doesn't need to be implemented, in which case NotImplementedError
        is suggested to be raised to prevent accidental use.
        """
        return NotImplemented

    @abstractmethod
    def find_webdriver(self):
        """Find the binary of the WebDriver."""
        return NotImplemented

    @abstractmethod
    def version(self, root):
        """Retrieve the release version of the installed browser."""
        return NotImplemented

    @abstractmethod
    def requirements(self):
        """Name of the browser-specific wptrunner requirements file"""
        return NotImplemented


class Firefox(Browser):
    """Firefox-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "firefox"
    binary = "browsers/firefox/firefox"
    platform_ini = "browsers/firefox/platform.ini"
    requirements = "requirements_firefox.txt"

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

    def install(self, dest=None):
        """Install Firefox."""

        from mozdownload import FactoryScraper
        import mozinstall

        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "mac"
        }.get(uname[0])

        if platform is None:
            raise ValueError("Unable to construct a valid Firefox package name for current platform")

        if dest is None:
            # os.getcwd() doesn't include the venv path
            dest = os.path.join(os.getcwd(), "_venv")

        dest = os.path.join(dest, "browsers")

        filename = FactoryScraper("daily", branch="mozilla-central", destination=dest).download()

        try:
            mozinstall.install(filename, dest)
        except mozinstall.mozinstall.InstallError as e:
            if platform == "mac" and os.path.exists(os.path.join(dest, "Firefox Nightly.app")):
                # mozinstall will fail if nightly is already installed in the venv because
                # mac installation uses shutil.copy_tree
                mozinstall.uninstall(os.path.join(dest, "Firefox Nightly.app"))
                mozinstall.install(filename, dest)
            else:
                raise

        os.remove(filename)
        return self.find_binary_path(dest)

    def find_binary_path(self, path=None):
        """Looks for the firefox binary in the virtual environment"""

        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "mac"
        }.get(uname[0])

        if path is None:
            #os.getcwd() doesn't include the venv path
            path = os.path.join(os.getcwd(), "_venv", "browsers")

        binary = None

        if platform == "linux":
            binary = find_executable("firefox", os.path.join(path, "firefox"))
        elif platform == "win":
            import mozinstall
            binary = mozinstall.get_binary(path, "firefox")
        elif platform == "mac":
            binary = find_executable("firefox", os.path.join(path, "Firefox Nightly.app", "Contents", "MacOS"))

        return binary

    def find_binary(self, venv_path=None):
        if venv_path is None:
            venv_path = os.path.join(os.getcwd(), venv_path)

        binary = self.find_binary_path(os.path.join(venv_path, "browsers"))

        if not binary and uname[0] == "Darwin":
            macpaths = ["/Applications/FirefoxNightly.app/Contents/MacOS",
                        os.path.expanduser("~/Applications/FirefoxNightly.app/Contents/MacOS"),
                        "/Applications/Firefox Developer Edition.app/Contents/MacOS",
                        os.path.expanduser("~/Applications/Firefox Developer Edition.app/Contents/MacOS"),
                        "/Applications/Firefox.app/Contents/MacOS",
                        os.path.expanduser("~/Applications/Firefox.app/Contents/MacOS")]
            return find_executable("firefox", os.pathsep.join(macpaths))

        if binary is None:
            return find_executable("firefox")

        return binary

    def find_certutil(self):
        path = find_executable("certutil")
        if path is None:
            return None
        if os.path.splitdrive(path)[1].split(os.path.sep) == ["", "Windows", "system32", "certutil.exe"]:
            return None
        return path

    def find_webdriver(self):
        return find_executable("geckodriver")

    def get_version_number(self, binary):
        version_re = re.compile("Mozilla Firefox (\d+\.\d+(?:\.\d+)?)(a|b)?")
        proc = subprocess.Popen([binary, "--version"], stdout=subprocess.PIPE)
        stdout, _ = proc.communicate()
        stdout.strip()
        m = version_re.match(stdout)
        if not m:
            return None, "nightly"
        version, status = m.groups()
        channel = {"a": "nightly", "b": "beta"}
        return version, channel.get(status, "stable")

    def get_prefs_url(self, version, channel):
        if channel == "stable":
            repo = "https://hg.mozilla.org/releases/mozilla-release"
            tag = "FIREFOX_%s_RELEASE" % version.replace(".", "_")
        else:
            repo = "https://hg.mozilla.org/mozilla-central"
            if channel == "beta":
                tag = "FIREFOX_%s_BETA" % version.split(".", 1)[0]
            else:
                # Always use tip as the tag for nightly; this isn't quite right
                # but to do better we need the actual build revision, which we
                # can get if we have an application.ini file
                tag = "tip"

        return "%s/raw-file/%s/testing/profiles/prefs_general.js" % (repo, tag)

    def install_prefs(self, binary, dest=None):
        version, channel = self.get_version_number(binary)

        if dest is None:
            dest = os.pwd

        dest = os.path.join(dest, "profiles")
        if not os.path.exists(dest):
            os.makedirs(dest)
        prefs_file = os.path.join(dest, "prefs_general.js")
        cache_file = os.path.join(dest,
                                  "%s-%s.cache" % (version, channel)
                                  if channel != "nightly"
                                  else "nightly.cache")

        have_cache = False
        if os.path.exists(cache_file):
            if channel != "nightly":
                have_cache = True
            else:
                now = datetime.now()
                have_cache = (datetime.fromtimestamp(os.stat(cache_file).st_mtime) >
                              now - timedelta(days=1))

        # If we don't have a recent download, grab the url
        if not have_cache:
            url = self.get_prefs_url(version, channel)

            with open(cache_file, "wb") as f:
                print("Installing test prefs from %s" % url)
                resp = get(url)
                f.write(resp.content)
        else:
            print("Using cached test prefs from %s" % cache_file)

        shutil.copyfile(cache_file, prefs_file)

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

    Includes webdriver installation, and wptrunner setup methods.
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

    def find_binary(self):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("chromedriver")

    def install_webdriver(self, dest=None):
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
        output = call(self.binary, "--version")
        return re.search(r"[0-9\.]+( [a-z]+)?$", output.strip()).group(0)


class ChromeAndroid(Browser):
    """Chrome-specific interface for Android.

    Includes webdriver installation.
    """

    product = "chrome_android"
    requirements = "requirements_chrome_android.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_binary(self):
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

    Includes webdriver installation, and wptrunner setup methods.
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

    def find_binary(self):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("operadriver")

    def install_webdriver(self, dest=None):
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


class Edge(Browser):
    """Edge-specific interface."""

    product = "edge"
    requirements = "requirements_edge.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_binary(self):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("MicrosoftWebDriver")

    def install_webdriver(self, dest=None):
        raise NotImplementedError

    def version(self, root):
        raise NotImplementedError


class InternetExplorer(Browser):
    """Internet Explorer-specific interface."""

    product = "ie"
    requirements = "requirements_ie.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_binary(self):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("IEDriverServer.exe")

    def install_webdriver(self, dest=None):
        raise NotImplementedError

    def version(self, root):
        raise NotImplementedError


class Safari(Browser):
    """Safari-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "safari"
    requirements = "requirements_safari.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_binary(self):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("safaridriver")

    def install_webdriver(self):
        raise NotImplementedError

    def version(self, root):
        raise NotImplementedError


class Servo(Browser):
    """Servo-specific interface."""

    product = "servo"
    requirements = "requirements_servo.txt"

    def platform_components(self):
        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "mac"
        }.get(uname[0])

        if platform is None:
            raise ValueError("Unable to construct a valid Servo package for current platform")

        if platform == "linux":
            extension = ".tar.gz"
            decompress = untar
        elif platform == "win" or platform == "mac":
            raise ValueError("Unable to construct a valid Servo package for current platform")

        return (platform, extension, decompress)

    def install(self, dest=None):
        """Install latest Browser Engine."""
        if dest is None:
            dest = os.pwd

        platform, extension, decompress = self.platform_components()
        url = "https://download.servo.org/nightly/%s/servo-latest%s" % (platform, extension)

        decompress(get(url).raw, dest=dest)
        path = find_executable("servo", os.path.join(dest, "servo"))
        st = os.stat(path)
        os.chmod(path, st.st_mode | stat.S_IEXEC)
        return path

    def find_binary(self):
        return find_executable("servo")

    def find_webdriver(self):
        return None

    def install_webdriver(self, dest=None):
        raise NotImplementedError

    def version(self, root):
        """Retrieve the release version of the installed browser."""
        output = call(self.binary, "--version")
        return re.search(r"[0-9\.]+( [a-z]+)?$", output.strip()).group(0)


class Sauce(Browser):
    """Sauce-specific interface."""

    product = "sauce"
    requirements = "requirements_sauce.txt"

    def install(self, dest=None):
        raise NotImplementedError

    def find_binary(self):
        raise NotImplementedError

    def find_webdriver(self):
        raise NotImplementedError

    def install_webdriver(self, dest=None):
        raise NotImplementedError

    def version(self, root):
        return None

class WebKit(Browser):
    """WebKit-specific interface."""

    product = "webkit"
    requirements = "requirements_webkit.txt"

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
