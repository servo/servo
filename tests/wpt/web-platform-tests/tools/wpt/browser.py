import logging
import os
import platform
import re
import shutil
import stat
import subprocess
import tempfile
from abc import ABCMeta, abstractmethod
from datetime import datetime, timedelta
from distutils.spawn import find_executable

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
    def install_webdriver(self, dest=None, channel=None):
        """Install the WebDriver implementation for this browser."""
        return NotImplemented

    @abstractmethod
    def find_binary(self, venv_path=None, channel=None):
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
    def version(self, binary=None):
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

    def install(self, dest=None, channel="nightly"):
        """Install Firefox."""

        branch = {
            "nightly": "mozilla-central",
            "beta": "mozilla-beta",
            "stable": "mozilla-stable"
        }
        scraper = {
            "nightly": "daily",
            "beta": "release",
            "stable": "release"
        }
        version = {
            "stable": "latest",
            "beta": "latest-beta",
            "nightly": "latest"
        }
        application_name = {
            "stable": "Firefox.app",
            "beta": "Firefox.app",
            "nightly": "Firefox Nightly.app"
        }
        if channel not in branch:
            raise ValueError("Unrecognised release channel: %s" % channel)

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

        dest = os.path.join(dest, "browsers", channel)

        filename = FactoryScraper(scraper[channel],
                                  branch=branch[channel],
                                  version=version[channel],
                                  destination=dest).download()

        try:
            mozinstall.install(filename, dest)
        except mozinstall.mozinstall.InstallError:
            if platform == "mac" and os.path.exists(os.path.join(dest, application_name[channel])):
                # mozinstall will fail if nightly is already installed in the venv because
                # mac installation uses shutil.copy_tree
                mozinstall.uninstall(os.path.join(dest, application_name[channel]))
                mozinstall.install(filename, dest)
            else:
                raise

        os.remove(filename)
        return self.find_binary_path(dest)

    def find_binary_path(self,path=None, channel="nightly"):
        """Looks for the firefox binary in the virtual environment"""

        platform = {
            "Linux": "linux",
            "Windows": "win",
            "Darwin": "mac"
        }.get(uname[0])

        application_name = {
            "stable": "Firefox.app",
            "beta": "Firefox.app",
            "nightly": "Firefox Nightly.app"
        }.get(channel)

        if path is None:
            #os.getcwd() doesn't include the venv path
            path = os.path.join(os.getcwd(), "_venv", "browsers", channel)

        binary = None

        if platform == "linux":
            binary = find_executable("firefox", os.path.join(path, "firefox"))
        elif platform == "win":
            import mozinstall
            binary = mozinstall.get_binary(path, "firefox")
        elif platform == "mac":
            binary = find_executable("firefox", os.path.join(path, application_name,
                                                             "Contents", "MacOS"))

        return binary

    def find_binary(self, venv_path=None, channel=None):
        if venv_path is None:
            venv_path = os.path.join(os.getcwd(), "_venv")

        if channel is None:
            channel = "nightly"

        path = os.path.join(venv_path, "browsers", channel)
        binary = self.find_binary_path(path, channel)

        if not binary and uname[0] == "Darwin":
            macpaths = ["/Applications/Firefox Nightly.app/Contents/MacOS",
                        os.path.expanduser("~/Applications/Firefox Nightly.app/Contents/MacOS"),
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

    def get_version_and_channel(self, binary):
        version_string = call(binary, "--version").strip()
        m = re.match(r"Mozilla Firefox (\d+\.\d+(?:\.\d+)?)(a|b)?", version_string)
        if not m:
            return None, "nightly"
        version, status = m.groups()
        channel = {"a": "nightly", "b": "beta"}
        return version, channel.get(status, "stable")

    def get_profile_bundle_url(self, version, channel):
        if channel == "stable":
            repo = "https://hg.mozilla.org/releases/mozilla-release"
            tag = "FIREFOX_%s_RELEASE" % version.replace(".", "_")
        elif channel == "beta":
            repo = "https://hg.mozilla.org/releases/mozilla-beta"
            major_version = version.split(".", 1)[0]
            # For beta we have a different format for betas that are now in stable releases
            # vs those that are not
            tags = get("https://hg.mozilla.org/releases/mozilla-beta/json-tags").json()["tags"]
            tags = {item["tag"] for item in tags}
            end_tag = "FIREFOX_BETA_%s_END" % major_version
            if end_tag in tags:
                tag = end_tag
            else:
                tag = "tip"
        else:
            repo = "https://hg.mozilla.org/mozilla-central"
            if channel == "beta":
                tag = "FIREFOX_%s_BETA" % version.split(".", 1)[0]
            else:
                # Always use tip as the tag for nightly; this isn't quite right
                # but to do better we need the actual build revision, which we
                # can get if we have an application.ini file
                tag = "tip"

        return "%s/archive/%s.zip/testing/profiles/" % (repo, tag)

    def install_prefs(self, binary, dest=None, channel=None):
        version, channel_ = self.get_version_and_channel(binary)
        if channel is not None and channel != channel_:
            # Beta doesn't always seem to have the b in the version string, so allow the
            # manually supplied value to override the one from the binary
            logger.warning("Supplied channel doesn't match binary, using supplied channel")
        elif channel is None:
            channel = channel_
        if dest is None:
            dest = os.pwd

        dest = os.path.join(dest, "profiles", channel, version)
        have_cache = False
        if os.path.exists(dest):
            if channel != "nightly":
                have_cache = True
            else:
                now = datetime.now()
                have_cache = (datetime.fromtimestamp(os.stat(dest).st_mtime) >
                              now - timedelta(days=1))

        # If we don't have a recent download, grab and extract the latest one
        if not have_cache:
            if os.path.exists(dest):
                shutil.rmtree(dest)
            os.makedirs(dest)

            url = self.get_profile_bundle_url(version, channel)

            print("Installing test prefs from %s" % url)
            try:
                extract_dir = tempfile.mkdtemp()
                unzip(get(url).raw, dest=extract_dir)

                profiles = os.path.join(extract_dir, os.listdir(extract_dir)[0], 'testing', 'profiles')
                for name in os.listdir(profiles):
                    path = os.path.join(profiles, name)
                    shutil.move(path, dest)
            finally:
                shutil.rmtree(extract_dir)
        else:
            print("Using cached test prefs from %s" % dest)

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

    def install_webdriver(self, dest=None, channel=None):
        """Install latest Geckodriver."""
        if dest is None:
            dest = os.getcwd()

        if channel == "nightly":
            path = self.install_geckodriver_nightly(dest)
            if path is not None:
                return path
            else:
                logger.warning("Nightly webdriver not found; falling back to release")

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

    def install_geckodriver_nightly(self, dest):
        import tarfile
        import mozdownload
        logger.info("Attempting to install webdriver from nightly")
        try:
            s = mozdownload.DailyScraper(branch="mozilla-central",
                                         extension="common.tests.tar.gz",
                                         destination=dest)
            package_path = s.download()
        except mozdownload.errors.NotFoundError:
            return

        try:
            exe_suffix = ".exe" if uname[0] == "Windows" else ""
            with tarfile.open(package_path, "r") as f:
                try:
                    member = f.getmember("bin%sgeckodriver%s" % (os.path.sep,
                                                                 exe_suffix))
                except KeyError:
                    return
                # Remove bin/ from the path.
                member.name = os.path.basename(member.name)
                f.extractall(members=[member], path=dest)
                path = os.path.join(dest, member.name)
            logger.info("Extracted geckodriver to %s" % path)
        finally:
            os.unlink(package_path)

        return path

    def version(self, binary=None, channel=None):
        """Retrieve the release version of the installed browser."""
        binary = binary or self.find_binary(channel)
        version_string = call(binary, "--version").strip()
        m = re.match(r"Mozilla Firefox (.*)", version_string)
        if not m:
            return None
        return m.group(1)


class Fennec(Browser):
    """Fennec-specific interface."""

    product = "fennec"
    requirements = "requirements_firefox.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None):
        raise NotImplementedError

    def version(self, binary=None):
        return None


class Chrome(Browser):
    """Chrome-specific interface.

    Includes webdriver installation, and wptrunner setup methods.
    """

    product = "chrome"
    requirements = "requirements_chrome.txt"

    @property
    def binary(self):
        if uname[0] == "Linux":
            return "/usr/bin/google-chrome"
        if uname[0] == "Darwin":
            return "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        # TODO Windows?
        logger.warn("Unable to find the browser binary.")
        return None

    def install(self, dest=None, channel=None):
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

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("chromedriver")

    def install_webdriver(self, dest=None, channel=None):
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

    def version(self, binary=None):
        binary = binary or self.binary
        if uname[0] != "Windows":
            try:
                version_string = call(binary, "--version").strip()
            except subprocess.CalledProcessError:
                logger.warn("Failed to call %s", binary)
                return None
            m = re.match(r"Google Chrome (.*)", version_string)
            if not m:
                logger.warn("Failed to extract version from: s%", version_string)
                return None
            return m.group(1)
        logger.warn("Unable to extract version from binary on Windows.")
        return None


class ChromeAndroid(Browser):
    """Chrome-specific interface for Android.

    Includes webdriver installation.
    """

    product = "chrome_android"
    requirements = "requirements_chrome_android.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("chromedriver")

    def install_webdriver(self, dest=None, channel=None):
        chrome = Chrome()
        return chrome.install_webdriver(dest, channel)

    def version(self, binary):
        return None

class ChromeWebDriver(Chrome):
    """Chrome-specific interface for chrome without using selenium.

    Includes webdriver installation.
    """
    product = "chrome_webdriver"

class Opera(Browser):
    """Opera-specific interface.

    Includes webdriver installation, and wptrunner setup methods.
    """

    product = "opera"
    requirements = "requirements_opera.txt"

    @property
    def binary(self):
        if uname[0] == "Linux":
            return "/usr/bin/opera"
        # TODO Windows, Mac?
        logger.warn("Unable to find the browser binary.")
        return None

    def install(self, dest=None, channel=None):
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

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("operadriver")

    def install_webdriver(self, dest=None, channel=None):
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

    def version(self, binary):
        """Retrieve the release version of the installed browser."""
        binary = binary or self.binary
        try:
            output = call(binary, "--version")
        except subprocess.CalledProcessError:
            logger.warn("Failed to call %s", binary)
            return None
        return re.search(r"[0-9\.]+( [a-z]+)?$", output.strip()).group(0)


class Edge(Browser):
    """Edge-specific interface."""

    product = "edge"
    requirements = "requirements_edge.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("MicrosoftWebDriver")

    def install_webdriver(self, dest=None, channel=None):
        raise NotImplementedError

    def version(self, binary):
        return None


class EdgeWebDriver(Edge):
    product = "edge_webdriver"


class InternetExplorer(Browser):
    """Internet Explorer-specific interface."""

    product = "ie"
    requirements = "requirements_ie.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("IEDriverServer.exe")

    def install_webdriver(self, dest=None, channel=None):
        raise NotImplementedError

    def version(self, binary):
        return None


class Safari(Browser):
    """Safari-specific interface.

    Includes installation, webdriver installation, and wptrunner setup methods.
    """

    product = "safari"
    requirements = "requirements_safari.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        return find_executable("safaridriver")

    def install_webdriver(self, dest=None, channel=None):
        raise NotImplementedError

    def version(self, binary):
        return None


class SafariWebDriver(Safari):
    product = "safari_webdriver"


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

    def install(self, dest=None, channel="nightly"):
        """Install latest Browser Engine."""
        if channel != "nightly":
            raise ValueError("Only nightly versions of Servo are available")
        if dest is None:
            dest = os.pwd

        platform, extension, decompress = self.platform_components()
        url = "https://download.servo.org/nightly/%s/servo-latest%s" % (platform, extension)

        decompress(get(url).raw, dest=dest)
        path = find_executable("servo", os.path.join(dest, "servo"))
        st = os.stat(path)
        os.chmod(path, st.st_mode | stat.S_IEXEC)
        return path

    def find_binary(self, venv_path=None, channel=None):
        path = find_executable("servo", os.path.join(venv_path, "servo"))
        if path is None:
            path = find_executable("servo")
        return path

    def find_webdriver(self):
        return None

    def install_webdriver(self, dest=None, channel=None):
        raise NotImplementedError

    def version(self, binary):
        """Retrieve the release version of the installed browser."""
        output = call(binary, "--version")
        return re.search(r"[0-9\.]+( [a-z]+)?$", output.strip()).group(0)


class Sauce(Browser):
    """Sauce-specific interface."""

    product = "sauce"
    requirements = "requirements_sauce.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venev_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None):
        raise NotImplementedError

    def version(self, binary):
        return None


class WebKit(Browser):
    """WebKit-specific interface."""

    product = "webkit"
    requirements = "requirements_webkit.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        return None

    def find_webdriver(self):
        return None

    def install_webdriver(self, dest=None, channel=None):
        raise NotImplementedError

    def version(self, binary):
        return None
