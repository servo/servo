import os
import platform
import re
import shutil
import stat
import errno
import subprocess
import tempfile
import urlparse
from abc import ABCMeta, abstractmethod
from datetime import datetime, timedelta
from distutils.spawn import find_executable

import requests

from utils import call, get, untar, unzip

uname = platform.uname()


def _get_fileversion(binary, logger=None):
    command = "(Get-Item '%s').VersionInfo.FileVersion" % binary.replace("'", "''")
    try:
        return call("powershell.exe", command).strip()
    except (subprocess.CalledProcessError, OSError):
        if logger is not None:
            logger.warning("Failed to call %s in PowerShell" % command)
        return None


def handle_remove_readonly(func, path, exc):
    excvalue = exc[1]
    if func in (os.rmdir, os.remove) and excvalue.errno == errno.EACCES:
        os.chmod(path, stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)  # 0777
        func(path)
    else:
        raise


class Browser(object):
    __metaclass__ = ABCMeta

    def __init__(self, logger):
        self.logger = logger

    @abstractmethod
    def install(self, dest=None):
        """Install the browser."""
        return NotImplemented

    @abstractmethod
    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
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
    def find_webdriver(self, channel=None):
        """Find the binary of the WebDriver."""
        return NotImplemented

    @abstractmethod
    def version(self, binary=None, webdriver_binary=None):
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

    platform = {
        "Linux": "linux",
        "Windows": "win",
        "Darwin": "macos"
    }.get(uname[0])

    application_name = {
        "stable": "Firefox.app",
        "beta": "Firefox.app",
        "nightly": "Firefox Nightly.app"
    }

    def platform_string_geckodriver(self):
        if self.platform is None:
            raise ValueError("Unable to construct a valid Geckodriver package name for current platform")

        if self.platform in ("linux", "win"):
            bits = "64" if uname[4] == "x86_64" else "32"
        else:
            bits = ""

        return "%s%s" % (self.platform, bits)

    def install(self, dest=None, channel="nightly"):
        """Install Firefox."""

        import mozinstall

        product = {
            "nightly": "firefox-nightly-latest-ssl",
            "beta": "firefox-beta-latest-ssl",
            "stable": "firefox-latest-ssl"
        }

        os_builds = {
            ("linux", "x86"): "linux",
            ("linux", "x86_64"): "linux64",
            ("win", "x86"): "win",
            ("win", "AMD64"): "win64",
            ("macos", "x86_64"): "osx",
        }
        os_key = (self.platform, uname[4])

        if channel not in product:
            raise ValueError("Unrecognised release channel: %s" % channel)

        if os_key not in os_builds:
            raise ValueError("Unsupported platform: %s %s" % os_key)

        if dest is None:
            # os.getcwd() doesn't include the venv path
            dest = os.path.join(os.getcwd(), "_venv")

        dest = os.path.join(dest, "browsers", channel)

        if not os.path.exists(dest):
            os.makedirs(dest)

        url = "https://download.mozilla.org/?product=%s&os=%s&lang=en-US" % (product[channel],
                                                                             os_builds[os_key])
        self.logger.info("Downloading Firefox from %s" % url)
        resp = requests.get(url)

        filename = None

        content_disposition = resp.headers.get('content-disposition')
        if content_disposition:
            filenames = re.findall("filename=(.+)", content_disposition)
            if filenames:
                filename = filenames[0]

        if not filename:
            filename = urlparse.urlsplit(resp.url).path.rsplit("/", 1)[1]

        if not filename:
            filename = "firefox.tar.bz2"

        installer_path = os.path.join(dest, filename)

        with open(installer_path, "wb") as f:
            f.write(resp.content)

        try:
            mozinstall.install(installer_path, dest)
        except mozinstall.mozinstall.InstallError:
            if self.platform == "macos" and os.path.exists(os.path.join(dest, self.application_name.get(channel, "Firefox Nightly.app"))):
                # mozinstall will fail if nightly is already installed in the venv because
                # mac installation uses shutil.copy_tree
                mozinstall.uninstall(os.path.join(dest, self.application_name.get(channel, "Firefox Nightly.app")))
                mozinstall.install(filename, dest)
            else:
                raise

        os.remove(installer_path)
        return self.find_binary_path(dest)

    def find_binary_path(self, path=None, channel="nightly"):
        """Looks for the firefox binary in the virtual environment"""

        if path is None:
            # os.getcwd() doesn't include the venv path
            path = os.path.join(os.getcwd(), "_venv", "browsers", channel)

        binary = None

        if self.platform == "linux":
            binary = find_executable("firefox", os.path.join(path, "firefox"))
        elif self.platform == "win":
            import mozinstall
            try:
                binary = mozinstall.get_binary(path, "firefox")
            except mozinstall.InvalidBinary:
                # ignore the case where we fail to get a binary
                pass
        elif self.platform == "macos":
            binary = find_executable("firefox", os.path.join(path, self.application_name.get(channel, "Firefox Nightly.app"),
                                                             "Contents", "MacOS"))

        return binary

    def find_binary(self, venv_path=None, channel="nightly"):
        if venv_path is None:
            venv_path = os.path.join(os.getcwd(), "_venv")

        path = os.path.join(venv_path, "browsers", channel)
        binary = self.find_binary_path(path, channel)

        if not binary and self.platform == "win":
            winpaths = [os.path.expandvars("$SYSTEMDRIVE\\Program Files\\Mozilla Firefox"),
                        os.path.expandvars("$SYSTEMDRIVE\\Program Files (x86)\\Mozilla Firefox")]
            for winpath in winpaths:
                binary = self.find_binary_path(winpath, channel)
                if binary is not None:
                    break

        if not binary and self.platform == "macos":
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
        if os.path.splitdrive(os.path.normcase(path))[1].split(os.path.sep) == ["", "windows", "system32", "certutil.exe"]:
            return None
        return path

    def find_webdriver(self, channel=None):
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
            # Always use tip as the tag for nightly; this isn't quite right
            # but to do better we need the actual build revision, which we
            # can get if we have an application.ini file
            tag = "tip"

        return "%s/archive/%s.zip/testing/profiles/" % (repo, tag)

    def install_prefs(self, binary, dest=None, channel=None):
        if binary:
            version, channel_ = self.get_version_and_channel(binary)
            if channel is not None and channel != channel_:
                # Beta doesn't always seem to have the b in the version string, so allow the
                # manually supplied value to override the one from the binary
                self.logger.warning("Supplied channel doesn't match binary, using supplied channel")
            elif channel is None:
                channel = channel_
        else:
            version = None

        if dest is None:
            dest = os.curdir

        dest = os.path.join(dest, "profiles", channel)
        if version:
            dest = os.path.join(dest, version)
        have_cache = False
        if os.path.exists(dest) and len(os.listdir(dest)) > 0:
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

            self.logger.info("Installing test prefs from %s" % url)
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
            self.logger.info("Using cached test prefs from %s" % dest)

        return dest

    def _latest_geckodriver_version(self):
        """Get and return latest version number for geckodriver."""
        # This is used rather than an API call to avoid rate limits
        tags = call("git", "ls-remote", "--tags", "--refs",
                    "https://github.com/mozilla/geckodriver.git")
        release_re = re.compile(r".*refs/tags/v(\d+)\.(\d+)\.(\d+)")
        latest_release = 0
        for item in tags.split("\n"):
            m = release_re.match(item)
            if m:
                version = [int(item) for item in m.groups()]
                if version > latest_release:
                    latest_release = version
        assert latest_release != 0
        return "v%s.%s.%s" % tuple(str(item) for item in latest_release)

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        """Install latest Geckodriver."""
        if dest is None:
            dest = os.getcwd()

        if channel == "nightly":
            path = self.install_geckodriver_nightly(dest)
            if path is not None:
                return path
            else:
                self.logger.warning("Nightly webdriver not found; falling back to release")

        version = self._latest_geckodriver_version()
        format = "zip" if uname[0] == "Windows" else "tar.gz"
        self.logger.debug("Latest geckodriver release %s" % version)
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
        self.logger.info("Attempting to install webdriver from nightly")
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
            self.logger.info("Extracted geckodriver to %s" % path)
        finally:
            os.unlink(package_path)

        return path

    def version(self, binary=None, webdriver_binary=None):
        """Retrieve the release version of the installed browser."""
        version_string = call(binary, "--version").strip()
        m = re.match(r"Mozilla Firefox (.*)", version_string)
        if not m:
            return None
        return m.group(1)


class FirefoxAndroid(Browser):
    """Android-specific Firefox interface."""

    product = "firefox_android"
    requirements = "requirements_firefox.txt"

    def install(self, dest=None, channel=None):
        if dest is None:
            dest = os.pwd

        TC_QUEUE_BASE = "https://queue.taskcluster.net/v1/"
        TC_INDEX_BASE = "https://index.taskcluster.net/v1/"


        resp = requests.get(TC_INDEX_BASE +
                            "task/gecko.v2.mozilla-central.latest.mobile.android-x86_64-opt")
        resp.raise_for_status()
        index = resp.json()
        task_id = index["taskId"]

        resp = requests.get(TC_QUEUE_BASE + "task/%s/artifacts/%s" %
                            (task_id, "public/build/geckoview-androidTest.apk"))
        resp.raise_for_status()

        apk_path = os.path.join(dest, "geckoview-androidTest.apk")

        with open(apk_path, "wb") as f:
            f.write(resp.content)

        return apk_path

    def install_prefs(self, binary, dest=None, channel=None):
        fx_browser = Firefox(self.logger)
        return fx_browser.install_prefs(binary, dest, channel)

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, channel=None):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        return None


class Chrome(Browser):
    """Chrome-specific interface.

    Includes webdriver installation, and wptrunner setup methods.
    """

    product = "chrome"
    requirements = "requirements_chrome.txt"

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

    def chromium_platform_string(self):
        platform = {
            "Linux": "Linux",
            "Windows": "Win",
            "Darwin": "Mac"
        }.get(uname[0])

        if platform is None:
            raise ValueError("Unable to construct a valid Chromium package name for current platform")

        if (platform == "Linux" or platform == "Win") and uname[4] == "x86_64":
            platform += "_x64"

        return platform

    def find_binary(self, venv_path=None, channel=None):
        if uname[0] == "Linux":
            name = "google-chrome"
            if channel == "stable":
                name += "-stable"
            elif channel == "beta":
                name += "-beta"
            elif channel == "dev":
                name += "-unstable"
            # No Canary on Linux.
            return find_executable(name)
        if uname[0] == "Darwin":
            if channel == "canary":
                return "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"
            # All other channels share the same path on macOS.
            return "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        if uname[0] == "Windows":
            return os.path.expandvars(r"$SYSTEMDRIVE\Program Files (x86)\Google\Chrome\Application\chrome.exe")
        self.logger.warning("Unable to find the browser binary.")
        return None

    def find_webdriver(self, channel=None):
        return find_executable("chromedriver")

    def _official_chromedriver_url(self, chrome_version):
        # http://chromedriver.chromium.org/downloads/version-selection
        parts = chrome_version.split(".")
        assert len(parts) == 4
        latest_url = "https://chromedriver.storage.googleapis.com/LATEST_RELEASE_%s.%s.%s" % tuple(parts[:-1])
        try:
            latest = get(latest_url).text.strip()
        except requests.RequestException:
            latest_url = "https://chromedriver.storage.googleapis.com/LATEST_RELEASE_%s" % parts[0]
            try:
                latest = get(latest_url).text.strip()
            except requests.RequestException:
                return None
        return "https://chromedriver.storage.googleapis.com/%s/chromedriver_%s.zip" % (
            latest, self.platform_string())

    def _chromium_chromedriver_url(self, chrome_version):
        try:
            # Try to find the Chromium build with the same revision.
            omaha = get("https://omahaproxy.appspot.com/deps.json?version=" + chrome_version).json()
            revision = omaha['chromium_base_position']
            url = "https://storage.googleapis.com/chromium-browser-snapshots/%s/%s/chromedriver_%s.zip" % (
                self.chromium_platform_string(), revision, self.platform_string())
            # Check the status without downloading the content (this is a streaming request).
            get(url)
        except requests.RequestException:
            # Fall back to the tip-of-tree Chromium build.
            revision_url = "https://storage.googleapis.com/chromium-browser-snapshots/%s/LAST_CHANGE" % (
                self.chromium_platform_string())
            revision = get(revision_url).text.strip()
            url = "https://storage.googleapis.com/chromium-browser-snapshots/%s/%s/chromedriver_%s.zip" % (
                self.chromium_platform_string(), revision, self.platform_string())
        return url

    def _latest_chromedriver_url(self, chrome_version):
        # Remove channel suffixes (e.g. " dev").
        chrome_version = chrome_version.split(' ')[0]
        return (self._official_chromedriver_url(chrome_version) or
                self._chromium_chromedriver_url(chrome_version))

    def install_webdriver_by_version(self, version, dest=None):
        assert version, "Cannot install ChromeDriver without Chrome version"
        if dest is None:
            dest = os.pwd
        url = self._latest_chromedriver_url(version)
        self.logger.info("Downloading ChromeDriver from %s" % url)
        unzip(get(url).raw, dest)
        chromedriver_dir = os.path.join(
            dest, 'chromedriver_%s' % self.platform_string())
        if os.path.isfile(os.path.join(chromedriver_dir, "chromedriver")):
            shutil.move(os.path.join(chromedriver_dir, "chromedriver"), dest)
            shutil.rmtree(chromedriver_dir)
        return find_executable("chromedriver", dest)

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        if browser_binary is None:
            browser_binary = self.find_binary(channel)
        return self.install_webdriver_by_version(
            self.version(browser_binary), dest)

    def version(self, binary=None, webdriver_binary=None):
        if not binary:
            self.logger.warning("No browser binary provided.")
            return None

        if uname[0] == "Windows":
            return _get_fileversion(binary, self.logger)

        try:
            version_string = call(binary, "--version").strip()
        except subprocess.CalledProcessError:
            self.logger.warning("Failed to call %s" % binary)
            return None
        m = re.match(r"(?:Google Chrome|Chromium) (.*)", version_string)
        if not m:
            self.logger.warning("Failed to extract version from: %s" % version_string)
            return None
        return m.group(1)


class ChromeAndroidBase(Browser):
    """A base class for ChromeAndroid and AndroidWebView.

    On Android, WebView is based on Chromium open source project, and on some
    versions of Android we share the library with Chrome. Therefore, we have
    a very similar WPT runner implementation.
    Includes webdriver installation.
    """
    __metaclass__ = ABCMeta  # This is an abstract class.

    def __init__(self, logger):
        super(ChromeAndroidBase, self).__init__(logger)

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    @abstractmethod
    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, channel=None):
        return find_executable("chromedriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        if browser_binary is None:
            browser_binary = self.find_binary(channel)
        chrome = Chrome(self.logger)
        return chrome.install_webdriver_by_version(
            self.version(browser_binary), dest)

    def version(self, binary=None, webdriver_binary=None):
        if not binary:
            self.logger.warning("No package name provided.")
            return None

        command = ['adb', 'shell', 'dumpsys', 'package', binary]
        try:
            output = call(*command)
        except (subprocess.CalledProcessError, OSError):
            self.logger.warning("Failed to call %s" % " ".join(command))
            return None
        match = re.search(r'versionName=(.*)', output)
        if not match:
            self.logger.warning("Failed to find versionName")
            return None
        return match.group(1)


class ChromeAndroid(ChromeAndroidBase):
    """Chrome-specific interface for Android.
    """

    product = "chrome_android"
    requirements = "requirements_chrome_android.txt"

    def find_binary(self, venv_path=None, channel=None):
        if channel in ("beta", "dev", "canary"):
            return "com.chrome." + channel
        return "com.android.chrome"


class AndroidWebview(ChromeAndroidBase):
    """Webview-specific interface for Android.

    Design doc:
    https://docs.google.com/document/d/19cGz31lzCBdpbtSC92svXlhlhn68hrsVwSB7cfZt54o/view
    """

    product = "android_webview"
    requirements = "requirements_android_webview.txt"

    def find_binary(self, venv_path=None, channel=None):
        # Just get the current package name of the WebView provider.
        # For WebView, it is not trivial to change the WebView provider, so
        # we will just grab whatever is available.
        # https://chromium.googlesource.com/chromium/src/+/HEAD/android_webview/docs/channels.md
        command = ['adb', 'shell', 'dumpsys', 'webviewupdate']
        try:
            output = call(*command)
        except (subprocess.CalledProcessError, OSError):
            self.logger.warning("Failed to call %s" % " ".join(command))
            return None
        m = re.search(r'^\s*Current WebView package \(name, version\): \((.*), ([0-9.]*)\)$',
                      output, re.M)
        if m is None:
            self.logger.warning("Unable to find current WebView package in dumpsys output")
            return None
        self.logger.warning("Final package name: " + m.group(1))
        return m.group(1)


class ChromeiOS(Browser):
    """Chrome-specific interface for iOS.
    """

    product = "chrome_ios"
    requirements = "requirements_chrome_ios.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, channel=None):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        return None


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
        self.logger.warning("Unable to find the browser binary.")
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

    def find_webdriver(self, channel=None):
        return find_executable("operadriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
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

    def version(self, binary=None, webdriver_binary=None):
        """Retrieve the release version of the installed browser."""
        binary = binary or self.binary
        try:
            output = call(binary, "--version")
        except subprocess.CalledProcessError:
            self.logger.warning("Failed to call %s" % binary)
            return None
        m = re.search(r"[0-9\.]+( [a-z]+)?$", output.strip())
        if m:
            return m.group(0)


class EdgeChromium(Browser):
    """MicrosoftEdge-specific interface."""
    platform = {
        "Linux": "linux",
        "Windows": "win",
        "Darwin": "macos"
    }.get(uname[0])
    product = "edgechromium"
    edgedriver_name = "msedgedriver"
    requirements = "requirements_edge_chromium.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        binary = None
        if self.platform == "win":
            binaryname = "msedge"
            binary = find_executable(binaryname)
            if not binary:
                # Use paths from different Edge channels starting with Release\Beta\Dev\Canary
                winpaths = [os.path.expanduser("~\\AppData\\Local\\Microsoft\\Edge\\Application"),
                            os.path.expandvars("$SYSTEMDRIVE\\Program Files\\Microsoft\\Edge Beta\\Application"),
                            os.path.expandvars("$SYSTEMDRIVE\\Program Files\\Microsoft\\Edge Dev\\Application"),
                            os.path.expandvars("$SYSTEMDRIVE\\Program Files (x86)\\Microsoft\\Edge Beta\\Application"),
                            os.path.expandvars("$SYSTEMDRIVE\\Program Files (x86)\\Microsoft\\Edge Dev\\Application"),
                            os.path.expanduser("~\\AppData\Local\\Microsoft\\Edge SxS\\Application")]
                return find_executable(binaryname, os.pathsep.join(winpaths))
        if self.platform == "macos":
            binaryname = "Microsoft Edge Canary"
            binary = find_executable(binaryname)
            if not binary:
                macpaths = ["/Applications/Microsoft Edge.app/Contents/MacOS",
                            os.path.expanduser("~/Applications/Microsoft Edge.app/Contents/MacOS"),
                            "/Applications/Microsoft Edge Canary.app/Contents/MacOS",
                            os.path.expanduser("~/Applications/Microsoft Edge Canary.app/Contents/MacOS")]
                return find_executable("Microsoft Edge Canary", os.pathsep.join(macpaths))
        return binary

    def find_webdriver(self, channel=None):
        return find_executable("msedgedriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        if self.platform != "win" and self.platform != "macos":
            raise ValueError("Only Windows and Mac platforms are currently supported")

        if dest is None:
            dest = os.pwd

        if channel is None:
            version_url = "https://msedgedriver.azureedge.net/LATEST_DEV"
        else:
            version_url = "https://msedgedriver.azureedge.net/LATEST_%s" % channel.upper()
        version = get(version_url).text.strip()

        if self.platform == "macos":
            bits = "mac64"
            edgedriver_path = os.path.join(dest, self.edgedriver_name)
        else:
            bits = "win64" if uname[4] == "x86_64" else "win32"
            edgedriver_path = os.path.join(dest, "%s.exe" % self.edgedriver_name)
        url = "https://msedgedriver.azureedge.net/%s/edgedriver_%s.zip" % (version, bits)

        # cleanup existing Edge driver files to avoid access_denied errors when unzipping
        if os.path.isfile(edgedriver_path):
            # remove read-only attribute
            os.chmod(edgedriver_path, stat.S_IRWXU | stat.S_IRWXG | stat.S_IRWXO)  # 0777
            os.remove(edgedriver_path)
            driver_notes_path = os.path.join(dest, "Driver_notes")
            if os.path.isdir(driver_notes_path):
                shutil.rmtree(driver_notes_path, ignore_errors=False, onerror=handle_remove_readonly)

        self.logger.info("Downloading MSEdgeDriver from %s" % url)
        unzip(get(url).raw, dest)
        if os.path.isfile(edgedriver_path):
            self.logger.info("Successfully downloaded MSEdgeDriver to %s" % edgedriver_path)
        return find_executable(self.edgedriver_name, dest)

    def version(self, binary=None, webdriver_binary=None):
        if binary is None:
            binary = self.find_binary()
        if self.platform != "win":
            try:
                version_string = call(binary, "--version").strip()
            except subprocess.CalledProcessError:
                self.logger.warning("Failed to call %s" % binary)
                return None
            m = re.match(r"Microsoft Edge (.*) ", version_string)
            if not m:
                self.logger.warning("Failed to extract version from: %s" % version_string)
                return None
            return m.group(1)
        else:
            if binary is not None:
                return _get_fileversion(binary, self.logger)
            self.logger.warning("Failed to find Edge binary.")
            return None


class Edge(Browser):
    """Edge-specific interface."""

    product = "edge"
    requirements = "requirements_edge.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, channel=None):
        return find_executable("MicrosoftWebDriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        command = "(Get-AppxPackage Microsoft.MicrosoftEdge).Version"
        try:
            return call("powershell.exe", command).strip()
        except (subprocess.CalledProcessError, OSError):
            self.logger.warning("Failed to call %s in PowerShell" % command)
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

    def find_webdriver(self, channel=None):
        return find_executable("IEDriverServer.exe")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
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

    def find_webdriver(self, channel=None):
        path = None
        if channel == "preview":
            path = "/Applications/Safari Technology Preview.app/Contents/MacOS"
        return find_executable("safaridriver", path)

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        if webdriver_binary is None:
            self.logger.warning("Cannot find Safari version without safaridriver")
            return None
        # Use `safaridriver --version` to get the version. Example output:
        # "Included with Safari 12.1 (14607.1.11)"
        # "Included with Safari Technology Preview (Release 67, 13607.1.9.0.1)"
        # The `--version` flag was added in STP 67, so allow the call to fail.
        try:
            version_string = call(webdriver_binary, "--version").strip()
        except subprocess.CalledProcessError:
            self.logger.warning("Failed to call %s --version" % webdriver_binary)
            return None
        m = re.match(r"Included with Safari (.*)", version_string)
        if not m:
            self.logger.warning("Failed to extract version from: %s" % version_string)
            return None
        return m.group(1)


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

    def find_webdriver(self, channel=None):
        return None

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        """Retrieve the release version of the installed browser."""
        output = call(binary, "--version")
        m = re.search(r"Servo ([0-9\.]+-[a-f0-9]+)?(-dirty)?$", output.strip())
        if m:
            return m.group(0)


class ServoWebDriver(Servo):
    product = "servodriver"


class Sauce(Browser):
    """Sauce-specific interface."""

    product = "sauce"
    requirements = "requirements_sauce.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venev_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, channel=None):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        return None


class WebKit(Browser):
    """WebKit-specific interface."""

    product = "webkit"
    requirements = "requirements_webkit.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        return None

    def find_webdriver(self, channel=None):
        return None

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        return None


class Epiphany(Browser):
    """Epiphany-specific interface."""

    product = "epiphany"
    requirements = "requirements_epiphany.txt"

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        return find_executable("epiphany")

    def find_webdriver(self, channel=None):
        return find_executable("WebKitWebDriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        if binary is None:
            return None
        output = call(binary, "--version")
        if output:
            # Stable release output looks like: "Web 3.30.2"
            # Tech Preview output looks like "Web 3.31.3-88-g97db4f40f"
            return output.split()[1]
        return None
