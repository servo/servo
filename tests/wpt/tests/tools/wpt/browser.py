# mypy: allow-untyped-defs
import os
import platform
import re
import shutil
import stat
import subprocess
import tempfile
from abc import ABCMeta, abstractmethod
from datetime import datetime, timedelta, timezone
from shutil import which
from urllib.parse import urlsplit

import html5lib
import requests
from packaging.specifiers import SpecifierSet

from .utils import (
    call,
    get,
    get_download_to_descriptor,
    rmtree,
    sha256sum,
    untar,
    unzip,
)
from .wpt import venv_dir

uname = platform.uname()

# The root URL for Chrome for Testing API endpoints.
CHROME_FOR_TESTING_ROOT_URL = "https://googlechromelabs.github.io/chrome-for-testing/"
# File name containing a matching ChromeDriver download URL for a specific Chrome download.
CHROMEDRIVER_SAVED_DOWNLOAD_FILE = "matching_chromedriver_url.txt"

# the rootUrl for the firefox-ci deployment of Taskcluster
FIREFOX_CI_ROOT_URL = 'https://firefox-ci-tc.services.mozilla.com'


def _get_fileversion(binary, logger=None):
    command = "(Get-Item -ErrorAction Stop '%s').VersionInfo.FileVersion" % binary.replace("'", "''")
    try:
        return call("powershell.exe", command).strip()
    except (subprocess.CalledProcessError, OSError):
        if logger is not None:
            logger.warning("Failed to call %s in PowerShell" % command)
        return None


def get_ext(filename):
    """Get the extension from a filename with special handling for .tar.foo"""
    name, ext = os.path.splitext(filename)
    if name.endswith(".tar"):
        ext = ".tar%s" % ext
    return ext


def get_download_filename(resp, default=None):
    """Get the filename from a requests.Response, or default"""
    filename = None

    content_disposition = resp.headers.get("content-disposition")
    if content_disposition:
        filenames = re.findall("filename=(.+)", content_disposition)
        if filenames:
            filename = filenames[0]

    if not filename:
        filename = urlsplit(resp.url).path.rsplit("/", 1)[1]

    return filename or default


def get_taskcluster_artifact(index, path):
    TC_INDEX_BASE = FIREFOX_CI_ROOT_URL + "/api/index/v1/"

    resp = get(TC_INDEX_BASE + "task/%s/artifacts/%s" % (index, path))
    resp.raise_for_status()

    return resp


class Browser:
    __metaclass__ = ABCMeta

    def __init__(self, logger):
        self.logger = logger

    def _get_browser_download_dir(self, dest, channel):
        if dest is None:
            return self._get_browser_binary_dir(dest, channel)

        return dest

    def _get_browser_binary_dir(self, dest, channel):
        if dest is None:
            # os.getcwd() doesn't include the venv path
            dest = os.path.join(os.getcwd(), venv_dir())

        dest = os.path.join(dest, "browsers", channel)

        if not os.path.exists(dest):
            os.makedirs(dest)

        return dest

    def download_from_url(
        self, url, dest=None, channel=None, rename=None, default_name="download"
    ):
        """Download a URL into a dest/channel
        :param url: The URL to download
        :param dest: Directory in which to put the dowloaded
        :param channel: Browser channel to append to the dest
        :param rename: Optional name for the download; the original extension
                       is preserved
        :param default_name: The default name for the download if none is
                             provided and none can be found from the network
        :return: The path to the downloaded package/installer
        """
        self.logger.info("Downloading from %s" % url)

        dest = self._get_browser_download_dir(dest, channel)

        resp = get(url)
        filename = get_download_filename(resp, default_name)
        if rename:
            filename = "%s%s" % (rename, get_ext(filename))

        output_path = os.path.join(dest, filename)

        with open(output_path, "wb") as f:
            for chunk in resp.iter_content(chunk_size=64 * 1024):
                f.write(chunk)

        return output_path

    @abstractmethod
    def download(self, dest=None, channel=None, rename=None):
        """Download a package or installer for the browser
        :param dest: Directory in which to put the dowloaded package
        :param channel: Browser channel to download
        :param rename: Optional name for the downloaded package; the original
                       extension is preserved.
        :return: The path to the downloaded package/installer
        """
        return NotImplemented

    @abstractmethod
    def install(self, dest=None, channel=None):
        """Download and install the browser.

        This method usually calls download().

        :param dest: Directory in which to install the browser
        :param channel: Browser channel to install
        :return: The path to the installed browser
        """
        return NotImplemented

    @abstractmethod
    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        """Download and install the WebDriver implementation for this browser.

        :param dest: Directory in which to install the WebDriver
        :param channel: Browser channel to install
        :param browser_binary: The path to the browser binary
        :return: The path to the installed WebDriver
        """
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
    def find_webdriver(self, venv_path=None, channel=None):
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
        elif self.platform == "macos" and uname.machine == "arm64":
            bits = "-aarch64"
        else:
            bits = ""

        return "%s%s" % (self.platform, bits)

    def download(self, dest=None, channel="nightly", rename=None):
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
            ("macos", "arm64"): "osx",
        }
        os_key = (self.platform, uname[4])

        dest = self._get_browser_download_dir(dest, channel)

        if channel not in product:
            raise ValueError("Unrecognised release channel: %s" % channel)

        if os_key not in os_builds:
            raise ValueError("Unsupported platform: %s %s" % os_key)

        url = "https://download.mozilla.org/?product=%s&os=%s&lang=en-US" % (product[channel],
                                                                             os_builds[os_key])
        self.logger.info("Downloading Firefox from %s" % url)
        resp = get(url)

        filename = get_download_filename(resp, "firefox.tar.bz2")

        if rename:
            filename = "%s%s" % (rename, get_ext(filename))

        installer_path = os.path.join(dest, filename)

        with open(installer_path, "wb") as f:
            f.write(resp.content)

        return installer_path

    def install(self, dest=None, channel="nightly"):
        """Install Firefox."""
        import mozinstall

        dest = self._get_browser_binary_dir(dest, channel)

        filename = os.path.basename(dest)

        installer_path = self.download(dest, channel)

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
            path = self._get_browser_binary_dir(None, channel)

        binary = None

        if self.platform == "linux":
            binary = which("firefox", path=os.path.join(path, "firefox"))
        elif self.platform == "win":
            import mozinstall
            try:
                binary = mozinstall.get_binary(path, "firefox")
            except mozinstall.InvalidBinary:
                # ignore the case where we fail to get a binary
                pass
        elif self.platform == "macos":
            binary = which("firefox",
                           path=os.path.join(path,
                                             self.application_name.get(channel, "Firefox Nightly.app"),
                                             "Contents", "MacOS"))

        return binary

    def find_binary(self, venv_path=None, channel="nightly"):

        path = self._get_browser_binary_dir(venv_path, channel)
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
            return which("firefox", path=os.pathsep.join(macpaths))

        if binary is None:
            return which("firefox")

        return binary

    def find_certutil(self):
        path = which("certutil")
        if path is None:
            return None
        if os.path.splitdrive(os.path.normcase(path))[1].split(os.path.sep) == ["", "windows", "system32", "certutil.exe"]:
            return None
        return path

    def find_webdriver(self, venv_path=None, channel=None):
        return which("geckodriver")

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
        if binary and not binary.endswith(".apk"):
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
        if os.path.exists(dest) and os.path.exists(os.path.join(dest, "profiles.json")):
            if channel != "nightly":
                have_cache = True
            else:
                now = datetime.now()
                have_cache = (datetime.fromtimestamp(os.stat(dest).st_mtime) >
                              now - timedelta(days=1))

        # If we don't have a recent download, grab and extract the latest one
        if not have_cache:
            if os.path.exists(dest):
                rmtree(dest)
            os.makedirs(dest)

            url = self.get_profile_bundle_url(version, channel)

            self.logger.info(f"Downloading test prefs from {url}")
            try:
                extract_dir = tempfile.mkdtemp()
                unzip(get(url).raw, dest=extract_dir)

                profiles = os.path.join(extract_dir, os.listdir(extract_dir)[0], 'testing', 'profiles')
                for name in os.listdir(profiles):
                    path = os.path.join(profiles, name)
                    shutil.move(path, dest)
            finally:
                rmtree(extract_dir)
            self.logger.info(f"Test prefs downloaded to {dest}")
        else:
            self.logger.info(f"Using cached test prefs from {dest}")

        return dest

    def _latest_geckodriver_version(self):
        """Get and return latest version number for geckodriver."""
        # This is used rather than an API call to avoid rate limits
        tags = call("git", "ls-remote", "--tags", "--refs",
                    "https://github.com/mozilla/geckodriver.git")
        release_re = re.compile(r".*refs/tags/v(\d+)\.(\d+)\.(\d+)")
        latest_release = (0, 0, 0)
        for item in tags.split("\n"):
            m = release_re.match(item)
            if m:
                version = tuple(int(item) for item in m.groups())
                if version > latest_release:
                    latest_release = version
        assert latest_release != (0, 0, 0)
        return "v%s.%s.%s" % tuple(str(item) for item in latest_release)

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        """Install latest Geckodriver."""
        if dest is None:
            dest = os.getcwd()

        path = None
        if channel == "nightly":
            path = self.install_geckodriver_nightly(dest)
            if path is None:
                self.logger.warning("Nightly webdriver not found; falling back to release")

        if path is None:
            version = self._latest_geckodriver_version()
            format = "zip" if uname[0] == "Windows" else "tar.gz"
            self.logger.debug("Latest geckodriver release %s" % version)
            url = ("https://github.com/mozilla/geckodriver/releases/download/%s/geckodriver-%s-%s.%s" %
                   (version, version, self.platform_string_geckodriver(), format))
            if format == "zip":
                unzip(get(url).raw, dest=dest)
            else:
                untar(get(url).raw, dest=dest)
            path = which("geckodriver", path=dest)

        assert path is not None
        self.logger.info("Installed %s" %
                         subprocess.check_output([path, "--version"]).splitlines()[0])
        return path

    def install_geckodriver_nightly(self, dest):
        self.logger.info("Attempting to install webdriver from nightly")

        platform_bits = ("64" if uname[4] == "x86_64" else
                         ("32" if self.platform == "win" else ""))
        tc_platform = "%s%s" % (self.platform, platform_bits)

        archive_ext = ".zip" if uname[0] == "Windows" else ".tar.gz"
        archive_name = "public/build/geckodriver%s" % archive_ext

        try:
            resp = get_taskcluster_artifact(
                "gecko.v2.mozilla-central.latest.geckodriver.%s" % tc_platform,
                archive_name)
        except Exception:
            self.logger.info("Geckodriver download failed")
            return

        if archive_ext == ".zip":
            unzip(resp.raw, dest)
        else:
            untar(resp.raw, dest)

        exe_ext = ".exe" if uname[0] == "Windows" else ""
        path = os.path.join(dest, "geckodriver%s" % exe_ext)

        self.logger.info("Extracted geckodriver to %s" % path)

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

    def __init__(self, logger):
        super().__init__(logger)
        self.apk_path = None
        self._fx_browser = Firefox(self.logger)

    def download(self, dest=None, channel=None, rename=None):
        if dest is None:
            dest = os.pwd

        resp = get_taskcluster_artifact(
            "gecko.v2.mozilla-central.shippable.latest.mobile.android-x86_64-opt",
            "public/build/geckoview-test_runner.apk")

        filename = "geckoview-test_runner.apk"
        if rename:
            filename = "%s%s" % (rename, get_ext(filename)[1])
        self.apk_path = os.path.join(dest, filename)

        with open(self.apk_path, "wb") as f:
            f.write(resp.content)

        return self.apk_path

    def install(self, dest=None, channel=None):
        return self.download(dest, channel)

    def install_prefs(self, binary, dest=None, channel=None):
        return self._fx_browser.install_prefs(binary, dest, channel)

    def find_binary(self, venv_path=None, channel=None):
        return self.apk_path

    def find_webdriver(self, venv_path=None, channel=None):
        return self._fx_browser.find_webdriver(venv_path, channel)

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        return self._fx_browser.install_webdriver(dest, channel, None)

    def version(self, binary=None, webdriver_binary=None):
        return None


class ChromeChromiumBase(Browser):
    """
    Chrome/Chromium base Browser class for shared functionality between Chrome and Chromium

    For a detailed description on the installation and detection of these browser components,
    see https://web-platform-tests.org/running-tests/chrome-chromium-installation-detection.html
    """

    requirements = "requirements_chromium.txt"
    platform = {
        "Linux": "Linux",
        "Windows": "Win",
        "Darwin": "Mac",
    }.get(uname[0])

    def _build_snapshots_url(self, revision, filename):
        return ("https://storage.googleapis.com/chromium-browser-snapshots/"
                f"{self._chromium_platform_string}/{revision}/{filename}")

    def _get_latest_chromium_revision(self):
        """Returns latest Chromium revision available for download."""
        # This is only used if the user explicitly passes "latest" for the revision flag.
        # The pinned revision is used by default to avoid unexpected failures as versions update.
        revision_url = ("https://storage.googleapis.com/chromium-browser-snapshots/"
                        f"{self._chromium_platform_string}/LAST_CHANGE")
        return get(revision_url).text.strip()

    def _get_pinned_chromium_revision(self):
        """Returns the pinned Chromium revision number."""
        return get("https://storage.googleapis.com/wpt-versions/pinned_chromium_revision").text.strip()

    def _get_chromium_revision(self, filename=None, version=None):
        """Retrieve a valid Chromium revision to download a browser component."""

        # If a specific version is passed as an argument, we will use it.
        if version is not None:
            # Detect a revision number based on the version passed.
            revision = self._get_base_revision_from_version(version)
            if revision is not None:
                # File name is needed to test if request is valid.
                url = self._build_snapshots_url(revision, filename)
                try:
                    # Check the status without downloading the content (this is a streaming request).
                    get(url)
                    return revision
                except requests.RequestException:
                    self.logger.warning("404: Unsuccessful attempt to download file "
                                        f"based on version. {url}")
        # If no URL was used in a previous install
        # and no version was passed, use the pinned Chromium revision.
        revision = self._get_pinned_chromium_revision()

        # If the url is successfully used to download/install, it will be used again
        # if another component is also installed during this run (browser/webdriver).
        return revision

    def _get_base_revision_from_version(self, version):
        """Get a Chromium revision number that is associated with a given version."""
        # This is not the single revision associated with the version,
        #     but instead is where it branched from. Chromium revisions are just counting
        #     commits on the master branch, there are no Chromium revisions for branches.

        version = self._remove_version_suffix(version)

        # Try to find the Chromium build with the same revision.
        try:
            omaha = get(f"https://omahaproxy.appspot.com/deps.json?version={version}").json()
            detected_revision = omaha['chromium_base_position']
            return detected_revision
        except requests.RequestException:
            self.logger.debug("Unsuccessful attempt to detect revision based on version")
        return None

    def _remove_existing_chromedriver_binary(self, path):
        """Remove an existing ChromeDriver for this product if it exists
        in the virtual environment.
        """
        # There may be an existing chromedriver binary from a previous install.
        # To provide a clean install experience, remove the old binary - this
        # avoids tricky issues like unzipping over a read-only file.
        existing_chromedriver_path = which("chromedriver", path=path)
        if existing_chromedriver_path:
            self.logger.info(f"Removing existing ChromeDriver binary: {existing_chromedriver_path}")
            os.chmod(existing_chromedriver_path, stat.S_IWUSR)
            os.remove(existing_chromedriver_path)

    def _remove_version_suffix(self, version):
        """Removes channel suffixes from Chrome/Chromium version string (e.g. " dev")."""
        return version.split(' ')[0]

    @property
    def _chromedriver_platform_string(self):
        """Returns a string that represents the suffix of the ChromeDriver
        file name when downloaded from Chromium Snapshots.
        """
        if self.platform == "Linux":
            bits = "64" if uname[4] == "x86_64" else "32"
        elif self.platform == "Mac":
            bits = "64"
        elif self.platform == "Win":
            bits = "32"
        return f"{self.platform.lower()}{bits}"

    @property
    def _chromium_platform_string(self):
        """Returns a string that is used for the platform directory in Chromium Snapshots"""
        if (self.platform == "Linux" or self.platform == "Win") and uname[4] == "x86_64":
            return f"{self.platform}_x64"
        if self.platform == "Mac" and uname.machine == "arm64":
            return "Mac_Arm"
        return self.platform

    def find_webdriver(self, venv_path=None, channel=None, browser_binary=None):
        if venv_path:
            venv_path = os.path.join(venv_path, self.product)
        return which("chromedriver", path=venv_path)

    def install_mojojs(self, dest, browser_binary):
        """Install MojoJS web framework."""
        # MojoJS is platform agnostic, but the version number must be an
        # exact match of the Chrome/Chromium version to be compatible.
        chrome_version = self.version(binary=browser_binary)
        if not chrome_version:
            return None
        chrome_version = self._remove_version_suffix(chrome_version)

        try:
            # MojoJS version url must match the browser binary version exactly.
            url = ("https://storage.googleapis.com/chrome-wpt-mojom/"
                   f"{chrome_version}/linux64/mojojs.zip")
            # Check the status without downloading the content (this is a streaming request).
            get(url)
        except requests.RequestException:
            # If a valid matching version cannot be found in the wpt archive,
            # download from Chromium snapshots bucket. However,
            # MojoJS is only bundled with Linux from Chromium snapshots.
            if self.platform == "Linux":
                filename = "mojojs.zip"
                revision = self._get_chromium_revision(filename, chrome_version)
                url = self._build_snapshots_url(revision, filename)
            else:
                self.logger.error("A valid MojoJS version cannot be found "
                                  f"for browser binary version {chrome_version}.")
                return None

        extracted = os.path.join(dest, "mojojs", "gen")
        last_url_file = os.path.join(extracted, "DOWNLOADED_FROM")
        if os.path.exists(last_url_file):
            with open(last_url_file, "rt") as f:
                last_url = f.read().strip()
            if last_url == url:
                self.logger.info("Mojo bindings already up to date")
                return extracted
            rmtree(extracted)

        try:
            self.logger.info(f"Downloading Mojo bindings from {url}")
            unzip(get(url).raw, dest)
            with open(last_url_file, "wt") as f:
                f.write(url)
            return extracted
        except Exception as e:
            self.logger.error(f"Cannot enable MojoJS: {e}")
            return None

    def install_webdriver_by_version(self, version, dest, revision=None):
        dest = os.path.join(dest, self.product)
        self._remove_existing_chromedriver_binary(dest)
        # _get_webdriver_url is implemented differently for Chrome and Chromium because
        # they download their respective versions of ChromeDriver from different sources.
        url = self._get_webdriver_url(version, revision)
        self.logger.info(f"Downloading ChromeDriver from {url}")
        unzip(get(url).raw, dest)

        # The two sources of ChromeDriver have different zip structures:
        # * Chromium archives the binary inside a chromedriver_* directory;
        # * Chrome archives the binary directly.
        # We want to make sure the binary always ends up directly in bin/.
        chromedriver_dir = os.path.join(dest,
                                        f"chromedriver_{self._chromedriver_platform_string}")
        chromedriver_path = which("chromedriver", path=chromedriver_dir)
        if chromedriver_path is not None:
            shutil.move(chromedriver_path, dest)
            rmtree(chromedriver_dir)

        chromedriver_path = which("chromedriver", path=dest)
        assert chromedriver_path is not None
        return chromedriver_path

    def version(self, binary=None, webdriver_binary=None):
        if not binary:
            self.logger.warning("No browser binary provided.")
            return None

        if uname[0] == "Windows":
            return _get_fileversion(binary, self.logger)

        try:
            version_string = call(binary, "--version").strip()
        except (subprocess.CalledProcessError, OSError) as e:
            self.logger.warning(f"Failed to call {binary}: {e}")
            return None
        m = re.match(r"(?:Google Chrome|Chromium) (.*)", version_string)
        if not m:
            self.logger.warning(f"Failed to extract version from: {version_string}")
            return None
        return m.group(1)

    def webdriver_version(self, webdriver_binary):
        if webdriver_binary is None:
            self.logger.warning("No valid webdriver supplied to detect version.")
            return None

        try:
            version_string = call(webdriver_binary, "--version").strip()
        except (subprocess.CalledProcessError, OSError) as e:
            self.logger.warning(f"Failed to call {webdriver_binary}: {e}")
            return None
        m = re.match(r"ChromeDriver ([0-9][0-9.]*)", version_string)
        if not m:
            self.logger.warning(f"Failed to extract version from: {version_string}")
            return None
        return m.group(1)


class Chromium(ChromeChromiumBase):
    """Chromium-specific interface.

    Includes browser binary installation and detection.
    Webdriver installation and wptrunner setup shared in base class with Chrome

    For a detailed description on the installation and detection of these browser components,
    see https://web-platform-tests.org/running-tests/chrome-chromium-installation-detection.html
    """
    product = "chromium"

    @property
    def _chromium_package_name(self):
        return f"chrome-{self.platform.lower()}"

    def _get_existing_browser_revision(self, venv_path, channel):
        revision = None
        try:
            # A file referencing the revision number is saved with the binary.
            # Check if this revision number exists and use it if it does.
            path = os.path.join(self._get_browser_binary_dir(None, channel), "revision")
            with open(path) as f:
                revision = f.read().strip()
        except FileNotFoundError:
            # If there is no information about the revision downloaded,
            # use the pinned revision.
            revision = self._get_pinned_chromium_revision()
        return revision

    def _find_binary_in_directory(self, directory):
        """Search for Chromium browser binary in a given directory."""
        if uname[0] == "Darwin":
            return which("Chromium", path=os.path.join(directory,
                                                       self._chromium_package_name,
                                                       "Chromium.app",
                                                       "Contents",
                                                       "MacOS"))
        # which will add .exe on Windows automatically.
        return which("chrome", path=os.path.join(directory, self._chromium_package_name))

    def _get_webdriver_url(self, version, revision=None):
        """Get Chromium Snapshots url to download Chromium ChromeDriver."""
        filename = f"chromedriver_{self._chromedriver_platform_string}.zip"

        # Make sure we use the same revision in an invocation.
        # If we have a url that was last used successfully during this run,
        # that url takes priority over trying to form another.
        if hasattr(self, "last_revision_used") and self.last_revision_used is not None:
            return self._build_snapshots_url(self.last_revision_used, filename)
        if revision is None:
            revision = self._get_chromium_revision(filename, version)
        elif revision == "latest":
            revision = self._get_latest_chromium_revision()
        elif revision == "pinned":
            revision = self._get_pinned_chromium_revision()

        return self._build_snapshots_url(revision, filename)

    def download(self, dest=None, channel=None, rename=None, version=None, revision=None):
        dest = self._get_browser_download_dir(dest, channel)

        filename = f"{self._chromium_package_name}.zip"

        if revision is None:
            revision = self._get_chromium_revision(filename, version)
        elif revision == "latest":
            revision = self._get_latest_chromium_revision()
        elif revision == "pinned":
            revision = self._get_pinned_chromium_revision()

        url = self._build_snapshots_url(revision, filename)
        self.logger.info(f"Downloading Chromium from {url}")
        resp = get(url)
        installer_path = os.path.join(dest, filename)
        with open(installer_path, "wb") as f:
            f.write(resp.content)

        # Revision successfully used. Keep this revision if another component install is needed.
        self.last_revision_used = revision
        with open(os.path.join(dest, "revision"), "w") as f:
            f.write(revision)
        return installer_path

    def find_binary(self, venv_path=None, channel=None):
        return self._find_binary_in_directory(self._get_browser_binary_dir(venv_path, channel))

    def install(self, dest=None, channel=None, version=None, revision=None):
        dest = self._get_browser_binary_dir(dest, channel)
        installer_path = self.download(dest, channel, version=version, revision=revision)
        with open(installer_path, "rb") as f:
            unzip(f, dest)
        os.remove(installer_path)
        return self._find_binary_in_directory(dest)

    def install_webdriver(self, dest=None, channel=None, browser_binary=None, revision=None):
        if dest is None:
            dest = os.pwd

        if revision is None:
            # If a revision was not given, we will need to detect the browser version.
            # The ChromeDriver that is installed will match this version.
            revision = self._get_existing_browser_revision(dest, channel)

        chromedriver_path = self.install_webdriver_by_version(None, dest, revision)

        return chromedriver_path

    def webdriver_supports_browser(self, webdriver_binary, browser_binary, browser_channel=None):
        """Check that the browser binary and ChromeDriver versions are a valid match."""
        browser_version = self.version(browser_binary)
        chromedriver_version = self.webdriver_version(webdriver_binary)

        if not chromedriver_version:
            self.logger.warning("Unable to get version for ChromeDriver "
                                f"{webdriver_binary}, rejecting it")
            return False

        if not browser_version:
            # If we can't get the browser version,
            # we just have to assume the ChromeDriver is good.
            return True

        # Because Chromium and its ChromeDriver should be pulled from the
        # same revision number, their version numbers should match exactly.
        if browser_version == chromedriver_version:
            self.logger.debug("Browser and ChromeDriver versions match.")
            return True
        self.logger.warning(f"ChromeDriver version {chromedriver_version} does not match "
                            f"Chromium version {browser_version}.")
        return False


class DownloadNotFoundError(Exception):
    """Raised when a download is not found for browser/webdriver installation."""
    pass


class Chrome(ChromeChromiumBase):
    """Chrome-specific interface.

    Includes browser binary installation and detection.
    Webdriver installation and wptrunner setup shared in base class with Chromium.

    For a detailed description on the installation and detection of these browser components,
    see https://web-platform-tests.org/running-tests/chrome-chromium-installation-detection.html
    """

    product = "chrome"

    @property
    def _chrome_platform_string(self):
        """A string that represents the platform-based suffix
        of the Chrome for Testing downloads.
        """
        if self.platform in ("Linux", "Win"):
            bits = "64" if uname.machine == "x86_64" else "32"
        elif self.platform == "Mac":
            bits = "-arm64" if uname.machine == "arm64" else "-x64"
        else:
            bits = ""
        return f"{self.platform.lower()}{bits}"

    @property
    def _chrome_package_name(self):
        return f"chrome-{self._chrome_platform_string}"

    def _get_build_version(self, version):
        """Convert a Chrome/ChromeDriver version into MAJOR.MINOR.BUILD format."""
        version_parts = version.split(".")
        if len(version_parts) < 3:
            self.logger.info(f"Version {version} could not be formatted for build matching.")
            return None
        return ".".join(version_parts[0:3])

    def _get_webdriver_url(self, version, channel):
        """Get a ChromeDriver URL to download a version of ChromeDriver that matches
        the browser binary version.

        Raises: ValueError if the given version string could not be formatted.

        Returns: A ChromeDriver download URL that matches the given Chrome version.
        """
        # Remove version suffix if it exists.
        if version:
            version = self._remove_version_suffix(version)

        formatted_version = self._get_build_version(version)
        if formatted_version is None:
            raise ValueError(f"Unknown version format: {version}")
        major_version = version.split(".")[0]

        # Chrome for Testing only has ChromeDriver downloads available for Chrome 115+.
        # If we are matching an older version of Chrome, use the old ChromeDriver source.
        if int(major_version) < 115:
            return self._get_old_webdriver_url(formatted_version)

        # Check if a file exists containing the matching ChromeDriver version download URL.
        # This is generated when installing Chrome for Testing using the install command.
        download_url_reference_file = os.path.join(
            self._get_browser_binary_dir(None, channel), CHROMEDRIVER_SAVED_DOWNLOAD_FILE)
        if os.path.isfile(download_url_reference_file):
            self.logger.info("Download info for matching ChromeDriver version found.")
            with open(download_url_reference_file, "r") as f:
                return f.read()

        # If no ChromeDriver download URL reference file exists,
        # try to find a download URL based on the build version.
        self.logger.info(f"Searching for ChromeDriver downloads for version {version}.")
        download_url = self._get_webdriver_url_by_build(formatted_version)
        if download_url is None:
            milestone = version.split('.')[0]
            self.logger.info(f'No ChromeDriver download found for build {formatted_version}. '
                             f'Finding latest available download for milestone {milestone}')
            download_url = self._get_webdriver_url_by_milestone(milestone)
        return download_url

    def _get_old_webdriver_url(self, version):
        """Find a ChromeDriver download URL for Chrome version <= 114

        Raises: DownloadNotFoundError if no ChromeDriver download URL is found
                to match the given Chrome binary.

        Returns: A ChromeDriver download URL that matches the given Chrome version.
        """
        latest_url = ("https://chromedriver.storage.googleapis.com/LATEST_RELEASE_"
                      f"{version}")
        try:
            latest = get(latest_url).text.strip()
        except requests.RequestException as e:
            raise DownloadNotFoundError("No matching ChromeDriver download"
                                        f" found for version {version}.", e)

        filename = f"chromedriver_{self._chromedriver_platform_string}.zip"
        return f"https://chromedriver.storage.googleapis.com/{latest}/{filename}"

    def _get_webdriver_url_by_build(self, version):
        """Find a ChromeDriver download URL based on a MAJOR.MINOR.BUILD version.

        Raises: RequestException if a bad responses is received from
                Chrome for Testing sources.

        Returns: Download URL string or None if no matching build is found.
        """
        try:
            # Get a list of builds with download URLs from Chrome for Testing.
            resp = get(f"{CHROME_FOR_TESTING_ROOT_URL}"
                       "latest-patch-versions-per-build-with-downloads.json")
        except requests.RequestException as e:
            raise requests.RequestException(
                "Chrome for Testing versions not found", e)
        builds_json = resp.json()
        builds_dict = builds_json["builds"]
        if version not in builds_dict:
            self.logger.info(f"No builds found for version {version}.")
            return None
        download_info = builds_dict[version]["downloads"]
        if "chromedriver" not in download_info:
            self.logger.info(f"No ChromeDriver download found for build {version}")
            return None
        downloads_for_platform = [d for d in download_info["chromedriver"]
                                  if d["platform"] == self._chrome_platform_string]
        if len(downloads_for_platform) == 0:
            self.logger.info(f"No ChromeDriver download found for build {version}"
                             f"of platform {self.platform}")
            return None
        return downloads_for_platform[0]["url"]

    def _get_webdriver_url_by_milestone(self, milestone):
        """Find a ChromeDriver download URL that is the latest available
        for a Chrome milestone.

        Raises: RequestException if a bad responses is received from
                Chrome for Testing sources.

        Returns: Download URL string or None if no matching milestone is found.
        """

        try:
            # Get a list of builds with download URLs from Chrome for Testing.
            resp = get(f"{CHROME_FOR_TESTING_ROOT_URL}"
                       "latest-versions-per-milestone.json")
        except requests.RequestException as e:
            raise requests.RequestException(
                "Chrome for Testing versions not found", e)
        milestones_json = resp.json()
        milestones_dict = milestones_json["milestones"]
        if milestone not in milestones_dict:
            self.logger.info(f"No latest version found for milestone {milestone}.")
            return None
        version_available = self._get_build_version(
            milestones_dict[milestone]["version"])

        return self._get_webdriver_url_by_build(version_available)

    def _get_download_urls_by_version(self, version):
        """Find Chrome for Testing and ChromeDriver download URLs matching a specific version.

        Raises: DownloadNotFoundError if no download is found for the given version or platform.
                RequestException if a bad responses is received from
                Chrome for Testing sources.

        Returns: Both binary downloads for Chrome and ChromeDriver.
        """
        try:
            # Get a list of versions with download URLs from Chrome for Testing.
            resp = get(f"{CHROME_FOR_TESTING_ROOT_URL}"
                       "known-good-versions-with-downloads.json")
        except requests.RequestException as e:
            raise requests.RequestException(
                "Chrome for Testing versions not found", e)
        versions_json = resp.json()
        versions_list = versions_json["versions"]
        # Attempt to find a version match in the list of available downloads.
        matching_versions = [v for v in versions_list if v["version"] == version]
        if len(matching_versions) == 0:
            raise DownloadNotFoundError(f"No Chrome for Testing download found for {version}")

        download_info = matching_versions[0]["downloads"]
        # Find the download url that matches the current platform.
        browser_download_urls = [d for d in download_info["chrome"]
                                 if d["platform"] == self._chrome_platform_string]
        if len(browser_download_urls) == 0:
            raise DownloadNotFoundError(
                f"No Chrome for Testing download found for {self.platform} of version {version}")
        browser_download_url = browser_download_urls[0]["url"]

        # Get the corresponding ChromeDriver download URL for later use.
        chromedriver_download_urls = [d for d in download_info["chromedriver"]
                                      if d["platform"] == self._chrome_platform_string]
        if len(chromedriver_download_urls) == 0:
            # Some older versions of Chrome for Testing
            # do not have a matching ChromeDriver download.
            raise DownloadNotFoundError(
                f"ChromeDriver download does not exist for version {version}")
        chromedriver_url = chromedriver_download_urls[0]["url"]

        return browser_download_url, chromedriver_url

    def _get_download_urls_by_channel(self, channel):
        """Find Chrome for Testing and ChromeDriver download URLs matching the given channel.

        Raises: DownloadNotFoundError if no download is found for the given channel or platform.
                RequestException if a bad responses is received from Chrome for Testing sources.

        Returns: Both binary downloads for Chrome and ChromeDriver.
        """
        try:
            resp = get(f"{CHROME_FOR_TESTING_ROOT_URL}"
                       "last-known-good-versions-with-downloads.json")
        except requests.RequestException as e:
            raise requests.RequestException(
                "Chrome for Testing versions not found", e)
        channels_json = resp.json()
        download_info = channels_json["channels"][channel.capitalize()]["downloads"]["chrome"]

        # Find the download URL that matches the current platform.
        matching_download_urls = [d for d in download_info
                                  if d["platform"] == self._chrome_platform_string]
        if len(matching_download_urls) == 0:
            raise DownloadNotFoundError("No matching download for platform "
                                        f"{self.platform} of channel \"{channel}\".")

        browser_download_url = matching_download_urls[0]["url"]

        # Get the corresponding ChromeDriver download URL for later use.
        chromedriver_download_info = (
            channels_json["channels"][channel.capitalize()]["downloads"]["chromedriver"])

        matching_chromedriver_urls = [d for d in chromedriver_download_info
                                      if d["platform"] == self._chrome_platform_string]
        if len(matching_chromedriver_urls) == 0:
            raise DownloadNotFoundError(
                f"No ChromeDriver download found in Chrome for Testing {channel}.")
        chromedriver_url = matching_chromedriver_urls[0]["url"]

        return browser_download_url, chromedriver_url

    def _save_chromedriver_download_info(self, dest, url):
        """Save the download URL of a ChromeDriver binary that matches the browser.
        This will allow for easy version matching, even in separate CLI invocations.
        """
        with open(os.path.join(dest, CHROMEDRIVER_SAVED_DOWNLOAD_FILE), "w") as f:
            f.write(url)

    def download(self, dest=None, channel="canary", rename=None, version=None):
        """Download Chrome for Testing. For more information,
        see: https://github.com/GoogleChromeLabs/chrome-for-testing
        """
        dest = self._get_browser_binary_dir(None, channel)
        filename = f"{self._chrome_package_name}.zip"

        # If a version has been supplied, try to find a download to match that version.
        # Otherwise, find a download for the specified channel.
        if version is not None:
            download_url, chromedriver_url = self._get_download_urls_by_version(version)
        else:
            download_url, chromedriver_url = self._get_download_urls_by_channel(channel)

        self.logger.info(f"Downloading Chrome for Testing from {download_url}")
        resp = get(download_url)
        installer_path = os.path.join(dest, filename)
        with open(installer_path, "wb") as f:
            f.write(resp.content)

        # Save the ChromeDriver download URL for use if a matching ChromeDriver
        # needs to be downloaded in a separate install invocation.
        self._save_chromedriver_download_info(dest, chromedriver_url)

        return installer_path

    def _find_binary_in_directory(self, directory):
        """Search for Chrome for Testing browser binary in a given directory."""
        if uname[0] == "Darwin":
            return which(
                "Google Chrome for Testing",
                path=os.path.join(directory,
                                  self._chrome_package_name,
                                  "Google Chrome for Testing.app",
                                  "Contents",
                                  "MacOS"))
        # "which" will add .exe on Windows automatically.
        return which("chrome", path=os.path.join(directory, self._chrome_package_name))

    def find_binary(self, venv_path=None, channel=None):
        # Check for binary in venv first.
        path = self._find_binary_in_directory(self._get_browser_binary_dir(venv_path, channel))
        if path is not None:
            return path

        if uname[0] == "Linux":
            name = "google-chrome"
            if channel == "stable":
                name += "-stable"
            elif channel == "beta":
                name += "-beta"
            elif channel == "dev":
                name += "-unstable"
            # No Canary on Linux.
            return which(name)
        if uname[0] == "Darwin":
            suffix = ""
            if channel in ("beta", "dev", "canary"):
                suffix = " " + channel.capitalize()
            return f"/Applications/Google Chrome{suffix}.app/Contents/MacOS/Google Chrome{suffix}"
        if uname[0] == "Windows":
            name = "Chrome"
            if channel == "beta":
                name += " Beta"
            elif channel == "dev":
                name += " Dev"
            path = os.path.expandvars(fr"$PROGRAMFILES\Google\{name}\Application\chrome.exe")
            if channel == "canary":
                path = os.path.expandvars(r"$LOCALAPPDATA\Google\Chrome SxS\Application\chrome.exe")
            return path
        self.logger.warning("Unable to find the browser binary.")
        return None

    def install(self, dest=None, channel=None, version=None):
        dest = self._get_browser_binary_dir(dest, channel)
        installer_path = self.download(dest=dest, channel=channel, version=version)
        with open(installer_path, "rb") as f:
            unzip(f, dest)
        os.remove(installer_path)
        return self._find_binary_in_directory(dest)

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        if dest is None:
            dest = os.pwd

        # Detect the browser version.
        # The ChromeDriver that is installed will match this version.
        if browser_binary is None:
            # If a browser binary path was not given, detect a valid path.
            browser_binary = self.find_binary(channel=channel)
            # We need a browser to version match, so if a browser binary path
            # was not given and cannot be detected, raise an error.
            if browser_binary is None:
                raise FileNotFoundError("No browser binary detected. "
                                        "Cannot install ChromeDriver without a browser version.")

        version = self.version(browser_binary)
        if version is None:
            raise ValueError(f"Unable to detect browser version from binary at {browser_binary}. "
                             " Cannot install ChromeDriver without a valid version to match.")

        chromedriver_path = self.install_webdriver_by_version(version, dest, channel)

        return chromedriver_path

    def install_webdriver_by_version(self, version, dest, channel):
        # Set the destination to a specific "chrome" folder to not overwrite or remove
        # ChromeDriver versions used for Chromium.
        dest = os.path.join(dest, self.product)
        self._remove_existing_chromedriver_binary(dest)

        url = self._get_webdriver_url(version, channel)
        if url is None:
            raise DownloadNotFoundError(
                f"No ChromeDriver download found to match browser version {version}")
        self.logger.info(f"Downloading ChromeDriver from {url}")
        unzip(get(url).raw, dest)

        chromedriver_dir = os.path.join(
            dest, f"chromedriver-{self._chrome_platform_string}")
        chromedriver_path = which("chromedriver", path=chromedriver_dir)

        if chromedriver_path is not None:
            shutil.move(chromedriver_path, dest)
            rmtree(chromedriver_dir)

        chromedriver_path = which("chromedriver", path=dest)
        if chromedriver_path is None:
            raise FileNotFoundError("ChromeDriver could not be detected after installation.")
        return chromedriver_path

    def webdriver_supports_browser(self, webdriver_binary, browser_binary, browser_channel):
        """Check that the browser binary and ChromeDriver versions are a valid match."""
        browser_version = self.version(browser_binary)
        chromedriver_version = self.webdriver_version(webdriver_binary)

        if not chromedriver_version:
            self.logger.warning("Unable to get version for ChromeDriver "
                                f"{webdriver_binary}, rejecting it")
            return False

        # TODO(DanielRyanSmith): Determine if this version logic fail case is
        # still necessary and remove it if it isn't.
        if not browser_version:
            # If we can't get the browser version,
            # we just have to assume the ChromeDriver is good.
            return True

        # Format versions for comparison.
        browser_version = self._get_build_version(browser_version)
        chromedriver_version = self._get_build_version(chromedriver_version)

        # Chrome and ChromeDriver versions should match on the same MAJOR.MINOR.BUILD version.
        if browser_version is not None and browser_version != chromedriver_version:
            # Consider the same milestone as matching.
            # Workaround for https://github.com/web-platform-tests/wpt/issues/42545
            # TODO(DanielRyanSmith): Remove this logic when browser binary is
            # downloaded from Chrome for Testing in CI runs.
            browser_milestone = browser_version.split('.')[0]
            chromedriver_milestone = chromedriver_version.split('.')[0]
            if browser_milestone != chromedriver_milestone:
                self.logger.warning(
                    f"ChromeDriver {chromedriver_version} does not match Chrome {browser_version}")
                return False
        return True

    def version(self, binary=None, webdriver_binary=None):
        """Get version string from browser binary."""
        if not binary:
            self.logger.warning("No browser binary provided.")
            return None
        if uname[0] == "Windows":
            return _get_fileversion(binary, self.logger)

        try:
            version_string = call(binary, "--version").strip()
        except (subprocess.CalledProcessError, OSError) as e:
            self.logger.warning(f"Failed to call {binary}: {e}")
            return None
        m = re.match(r"(?:Google Chrome for Testing|Google Chrome) (.*)", version_string)
        if not m:
            self.logger.warning(f"Failed to extract version from: {version_string}")
            return None
        return m.group(1)

    def webdriver_version(self, webdriver_binary):
        """Get version string from ChromeDriver binary."""
        if webdriver_binary is None:
            self.logger.warning("No valid webdriver supplied to detect version.")
            return None

        try:
            version_string = call(webdriver_binary, "--version").strip()
        except (subprocess.CalledProcessError, OSError) as e:
            self.logger.warning(f"Failed to call {webdriver_binary}: {e}")
            return None
        m = re.match(r"ChromeDriver ([0-9][0-9.]*)", version_string)
        if not m:
            self.logger.warning(f"Failed to extract version from: {version_string}")
            return None
        return m.group(1)


class ContentShell(Browser):
    """Interface for the Chromium content shell.
    """

    product = "content_shell"
    requirements = None

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        if uname[0] == "Darwin":
            return which("Content Shell.app/Contents/MacOS/Content Shell")
        return which("content_shell")  # .exe is added automatically for Windows

    def find_webdriver(self, venv_path=None, channel=None):
        return None

    def version(self, binary=None, webdriver_binary=None):
        # content_shell does not return version information.
        return "N/A"

class ChromeAndroidBase(Browser):
    """A base class for ChromeAndroid and AndroidWebView.

    On Android, WebView is based on Chromium open source project, and on some
    versions of Android we share the library with Chrome. Therefore, we have
    a very similar WPT runner implementation.
    Includes webdriver installation.
    """
    __metaclass__ = ABCMeta  # This is an abstract class.

    def __init__(self, logger):
        super().__init__(logger)
        self.device_serial = None
        self.adb_binary = "adb"

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    @abstractmethod
    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, venv_path=None, channel=None):
        return which("chromedriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        if browser_binary is None:
            browser_binary = self.find_binary(channel)
        chrome = Chrome(self.logger)
        return chrome.install_webdriver_by_version(self.version(browser_binary), dest)

    def version(self, binary=None, webdriver_binary=None):
        if not binary:
            self.logger.warning("No package name provided.")
            return None

        command = [self.adb_binary]
        if self.device_serial:
            # Assume we have same version of browser on all devices
            command.extend(['-s', self.device_serial[0]])
        command.extend(['shell', 'dumpsys', 'package', binary])
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
    requirements = "requirements_chromium.txt"

    def find_binary(self, venv_path=None, channel=None):
        if channel in ("beta", "dev", "canary"):
            return "com.chrome." + channel
        return "com.android.chrome"


# TODO(aluo): This is largely copied from the AndroidWebView implementation.
# Tests are not running for weblayer yet (crbug/1019521), this initial
# implementation will help to reproduce and debug any issues.
class AndroidWeblayer(ChromeAndroidBase):
    """Weblayer-specific interface for Android."""

    product = "android_weblayer"
    # TODO(aluo): replace this with weblayer version after tests are working.
    requirements = "requirements_chromium.txt"

    def find_binary(self, venv_path=None, channel=None):
        return "org.chromium.weblayer.shell"


class AndroidWebview(ChromeAndroidBase):
    """Webview-specific interface for Android.

    Design doc:
    https://docs.google.com/document/d/19cGz31lzCBdpbtSC92svXlhlhn68hrsVwSB7cfZt54o/view
    """

    product = "android_webview"
    requirements = "requirements_chromium.txt"

    def find_binary(self, venv_path=None, channel=None):
        # Just get the current package name of the WebView provider.
        # For WebView, it is not trivial to change the WebView provider, so
        # we will just grab whatever is available.
        # https://chromium.googlesource.com/chromium/src/+/HEAD/android_webview/docs/channels.md
        command = [self.adb_binary]
        if self.device_serial:
            command.extend(['-s', self.device_serial[0]])
        command.extend(['shell', 'dumpsys', 'webviewupdate'])
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
    requirements = None

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, venv_path=None, channel=None):
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

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

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

    def find_webdriver(self, venv_path=None, channel=None):
        return which("operadriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        if dest is None:
            dest = os.pwd
        latest = get("https://api.github.com/repos/operasoftware/operachromiumdriver/releases/latest").json()["tag_name"]
        url = "https://github.com/operasoftware/operachromiumdriver/releases/download/%s/operadriver_%s.zip" % (latest,
                                                                                                                self.platform_string())
        unzip(get(url).raw, dest)

        operadriver_dir = os.path.join(dest, "operadriver_%s" % self.platform_string())
        shutil.move(os.path.join(operadriver_dir, "operadriver"), dest)
        rmtree(operadriver_dir)

        path = which("operadriver")
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
    """Microsoft Edge Chromium Browser class."""

    product = "edgechromium"
    requirements = "requirements_chromium.txt"
    platform = {
        "Linux": "linux",
        "Windows": "win",
        "Darwin": "macos"
    }.get(uname[0])

    def _get_build_version(self, version):
        """Convert a Edge/MSEdgeDriver version into MAJOR.MINOR.BUILD format."""
        version_parts = version.split(".")
        if len(version_parts) < 3:
            self.logger.info(f"Version {version} could not be formatted for build matching.")
            return None
        return ".".join(version_parts[0:3])

    def _remove_existing_edgedriver_binary(self, path):
        """Remove an existing MSEdgeDriver for this product if it exists
        in the virtual environment.
        """
        # There may be an existing MSEdgeDriver binary from a previous install.
        # To provide a clean install experience, remove the old binary - this
        # avoids tricky issues like unzipping over a read-only file.
        existing_edgedriver_path = which("MSEdgeDriver", path=path)
        if existing_edgedriver_path:
            self.logger.info(f"Removing existing MSEdgeDriver binary: {existing_edgedriver_path}")
            os.chmod(existing_edgedriver_path, stat.S_IWUSR)
            os.remove(existing_edgedriver_path)
        existing_driver_notes_path = os.path.join(path, "Driver_notes")
        if os.path.isdir(existing_driver_notes_path):
            self.logger.info(f"Removing existing MSEdgeDriver binary: {existing_driver_notes_path}")
            print(f"Delete {existing_driver_notes_path} folder")
            rmtree(existing_driver_notes_path)

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install_mojojs(self, dest, browser_binary):
        # TODO: Install MojoJS web framework.
        # MojoJS is platform agnostic, but the version number must be an
        # exact match of the Edge version to be compatible.
        return None

    def find_binary(self, venv_path=None, channel=None):
        # TODO: Check for binary in virtual environment first
        if self.platform == "linux":
            name = "microsoft-edge"
            if channel == "stable":
                name += "-stable"
            elif channel == "beta":
                name += "-beta"
            elif channel == "dev":
                name += "-dev"
            # No Canary on Linux.
            return which(name)
        if self.platform == "macos":
            suffix = ""
            if channel in ("beta", "dev", "canary"):
                suffix = " " + channel.capitalize()
            return f"/Applications/Microsoft Edge{suffix}.app/Contents/MacOS/Microsoft Edge{suffix}"
        if self.platform == "win":
            suffix = ""
            if channel in ("beta", "dev"):
                suffix = " " + channel.capitalize()
            winpaths = [os.path.expandvars(fr"%PROGRAMFILES%\Microsoft\Edge{suffix}\Application"),
                        os.path.expandvars(fr"%programfiles(x86)%\Microsoft\Edge{suffix}\Application")]
            path = which("msedge.exe", path=os.pathsep.join(winpaths))
            if channel == "canary":
                path = os.path.expandvars(r"%LOCALAPPDATA%\Microsoft\Edge SxS\Application\msedge.exe")
            return path
        self.logger.warning("Unable to find the browser binary.")
        return None

    def find_webdriver(self, venv_path=None, channel=None):
        return which("msedgedriver")

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        if dest is None:
            dest = os.pwd

        # Detect the browser version.
        # The MSEdgeDriver that is installed will match this version.
        if browser_binary is None:
            # If a browser binary path was not given, detect a valid path.
            browser_binary = self.find_binary(channel=channel)
            # We need a browser to version match, so if a browser binary path
            # was not given and cannot be detected, raise an error.
            if browser_binary is None:
                raise FileNotFoundError("No browser binary detected. "
                                        "Cannot install MSEdgeDriver without a browser version.")

        version = self.version(browser_binary)
        if version is None:
            raise ValueError(f"Unable to detect browser version from binary at {browser_binary}. "
                             "Cannot install MSEdgeDriver without a valid version to match.")

        edgedriver_path = self.install_webdriver_by_version(version, dest, channel)

        return edgedriver_path

    def install_webdriver_by_version(self, version, dest, channel):
        self._remove_existing_edgedriver_binary(dest)

        if self.platform == "linux":
            bits = "linux64"
        elif self.platform == "macos":
            bits = "mac64"
        elif self.platform == "win":
            bits = "win64" if uname[4] == "x86_64" else "win32"

        url = f"https://msedgedriver.azureedge.net/{version}/edgedriver_{bits}.zip"
        self.logger.info(f"Downloading MSEdgeDriver from {url}")
        unzip(get(url).raw, dest)
        edgedriver_path = which("msedgedriver", path=dest)
        assert edgedriver_path is not None
        return edgedriver_path

    def webdriver_supports_browser(self, webdriver_binary, browser_binary, browser_channel):
        """Check that the browser binary and MSEdgeDriver versions are a valid match."""
        browser_version = self.version(browser_binary)
        edgedriver_version = self.webdriver_version(webdriver_binary)

        if not edgedriver_version:
            self.logger.warning("Unable to get version for MSEdgeDriver "
                                f"{webdriver_binary}, rejecting it")
            return False

        if not browser_version:
            # If we can't get the browser version, we just have to assume the
            # MSEdgeDriver is good.
            self.logger.warning("Unable to get version for the browser "
                                f"{browser_binary}, assuming MSEdgeDriver is good.")
            return True

        # Check that the EdgeDriver version matches the Edge version.
        browser_version = self._get_build_version(browser_version)
        edgedriver_version = self._get_build_version(edgedriver_version)

        # Edge and MSEdgeDriver versions should match on the same MAJOR.MINOR.BUILD version.
        if browser_version is not None and browser_version != edgedriver_version:
            self.logger.warning(
                f"MSEdgeDriver {edgedriver_version} does not match Edge {browser_version}")
            return False
        return True

    def version(self, binary=None, webdriver_binary=None):
        """Get version string from browser binary."""
        if not binary:
            self.logger.warning("No browser binary provided.")
            return None

        if uname[0] == "Windows":
            return _get_fileversion(binary, self.logger)

        try:
            version_string = call(binary, "--version").strip()
        except (subprocess.CalledProcessError, OSError) as e:
            self.logger.warning(f"Failed to call {binary}: {e}")
            return None
        m = re.match(r"Microsoft Edge ([0-9][0-9.]*)", version_string)
        if not m:
            self.logger.warning(f"Failed to extract version from: {version_string}")
            return None
        return m.group(1)

    def webdriver_version(self, webdriver_binary):
        """Get version string from MSEdgeDriver binary."""
        if webdriver_binary is None:
            self.logger.warning("No valid webdriver supplied to detect version.")
            return None

        try:
            version_string = call(webdriver_binary, "--version").strip()
        except (subprocess.CalledProcessError, OSError) as e:
            self.logger.warning(f"Failed to call {webdriver_binary}: {e}")
            return None
        m = re.match(r"Microsoft Edge WebDriver ([0-9][0-9.]*)", version_string)
        if not m:
            self.logger.warning(f"Failed to extract version from: {version_string}")
            return None
        return m.group(1)


class Edge(Browser):
    """Edge-specific interface."""

    product = "edge"
    requirements = "requirements_edge.txt"

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, venv_path=None, channel=None):
        return which("MicrosoftWebDriver")

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

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, venv_path=None, channel=None):
        return which("IEDriverServer.exe")

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

    def _find_downloads(self):
        def text_content(e, __output=None):
            # this doesn't use etree.tostring so that we can add spaces for p and br
            if __output is None:
                __output = []

            if e.tag == "p":
                __output.append("\n\n")

            if e.tag == "br":
                __output.append("\n")

            if e.text is not None:
                __output.append(e.text)

            for child in e:
                text_content(child, __output)
                if child.tail is not None:
                    __output.append(child.tail)

            return "".join(__output)

        self.logger.info("Finding STP download URLs")
        resp = get("https://developer.apple.com/safari/download/")

        doc = html5lib.parse(
            resp.content,
            "etree",
            namespaceHTMLElements=False,
            transport_encoding=resp.encoding,
        )
        ascii_ws = re.compile(r"[\x09\x0A\x0C\x0D\x20]+")

        downloads = []
        for candidate in doc.iterfind(".//li[@class]"):
            class_names = set(ascii_ws.split(candidate.attrib["class"]))
            if {"download", "dmg", "zip"} & class_names:
                downloads.append(candidate)

        # Note we use \s throughout for space as we don't care what form the whitespace takes
        stp_link_text = re.compile(
            r"^\s*Safari\s+Technology\s+Preview\s+(?:[0-9]+\s+)?for\s+macOS"
        )
        requirement = re.compile(
            r"""(?x)  # (extended regexp syntax for comments)
            ^\s*Requires\s+macOS\s+  # Starting with the magic string
            ([0-9]+(?:\.[0-9]+)*)  # A macOS version number of numbers and dots
            (?:\s+beta(?:\s+[0-9]+)?)?  # Optionally a beta, itself optionally with a number (no dots!)
            (?:\s+or\s+later)?  # Optionally an 'or later'
            \.?\s*$  # Optionally ending with a literal dot
            """
        )

        stp_downloads = []
        for download in downloads:
            for link in download.iterfind(".//a[@href]"):
                if stp_link_text.search(text_content(link)):
                    break
                else:
                    self.logger.debug("non-matching anchor: " + text_content(link))
            else:
                continue

            for el in download.iter():
                # avoid assuming any given element here, just assume it is a single element
                m = requirement.search(text_content(el))
                if m:
                    version = m.group(1)

                    # This assumes the current macOS numbering, whereby X.Y is compatible
                    # with X.(Y+1), e.g. 12.4 is compatible with 12.3, but 13.0 isn't
                    # compatible with 12.3.
                    if version.count(".") >= (2 if version.startswith("10.") else 1):
                        spec = SpecifierSet(f"~={version}")
                    else:
                        spec = SpecifierSet(f"=={version}.*")

                    stp_downloads.append((spec, link.attrib["href"].strip()))
                    break
            else:
                self.logger.debug(
                    "Found a link but no requirement: " + text_content(download)
                )

        if stp_downloads:
            self.logger.info(
                "Found STP URLs for macOS " +
                ", ".join(str(dl[0]) for dl in stp_downloads)
            )
        else:
            self.logger.warning("Did not find any STP URLs")

        return stp_downloads

    def _download_image(self, downloads, dest, system_version=None):
        if system_version is None:
            system_version, _, _ = platform.mac_ver()

        chosen_url = None
        for version_spec, url in downloads:
            if system_version in version_spec:
                self.logger.debug(f"Will download Safari for {version_spec}")
                chosen_url = url
                break

        if chosen_url is None:
            raise ValueError(f"no download for {system_version}")

        self.logger.info(f"Downloading Safari from {chosen_url}")
        resp = get(chosen_url)

        filename = get_download_filename(resp, "SafariTechnologyPreview.dmg")
        installer_path = os.path.join(dest, filename)
        with open(installer_path, "wb") as f:
            f.write(resp.content)

        return installer_path

    def _download_extract(self, image_path, dest, rename=None):
        with tempfile.TemporaryDirectory() as tmpdir:
            self.logger.debug(f"Mounting {image_path}")
            r = subprocess.run(
                [
                    "hdiutil",
                    "attach",
                    "-readonly",
                    "-mountpoint",
                    tmpdir,
                    "-nobrowse",
                    "-verify",
                    "-noignorebadchecksums",
                    "-autofsck",
                    image_path,
                ],
                encoding="utf-8",
                capture_output=True,
                check=True,
            )

            mountpoint = None
            for line in r.stdout.splitlines():
                if not line.startswith("/dev/"):
                    continue

                _, _, mountpoint = line.split("\t", 2)
                if mountpoint:
                    break

            if mountpoint is None:
                raise ValueError("no volume mounted from image")

            pkgs = [p for p in os.listdir(mountpoint) if p.endswith((".pkg", ".mpkg"))]
            if len(pkgs) != 1:
                raise ValueError(
                    f"Expected a single .pkg/.mpkg, found {len(pkgs)}: {', '.join(pkgs)}"
                )

            source_path = os.path.join(mountpoint, pkgs[0])
            dest_path = os.path.join(
                dest, (rename + get_ext(pkgs[0])) if rename is not None else pkgs[0]
            )

            self.logger.debug(f"Copying {source_path} to {dest_path}")
            shutil.copy2(
                source_path,
                dest_path,
            )

            self.logger.debug(f"Unmounting {mountpoint}")
            subprocess.run(
                ["hdiutil", "detach", mountpoint],
                encoding="utf-8",
                capture_output=True,
                check=True,
            )

        return dest_path

    def download(self, dest=None, channel="preview", rename=None, system_version=None):
        if channel != "preview":
            raise ValueError(f"can only install 'preview', not '{channel}'")

        dest = self._get_browser_download_dir(dest, channel)

        stp_downloads = self._find_downloads()

        with tempfile.TemporaryDirectory() as tmpdir:
            image_path = self._download_image(stp_downloads, tmpdir, system_version)
            return self._download_extract(image_path, dest, rename)

    def install(self, dest=None, channel=None):
        # We can't do this because stable/beta releases are system components and STP
        # requires admin permissions to install.
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, venv_path=None, channel=None):
        path = None
        if channel == "preview":
            path = "/Applications/Safari Technology Preview.app/Contents/MacOS"
        return which("safaridriver", path=path)

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
    requirements = None

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

    def _get(self, channel="nightly"):
        if channel != "nightly":
            raise ValueError("Only nightly versions of Servo are available")

        platform, extension, _ = self.platform_components()
        url = "https://download.servo.org/nightly/%s/servo-latest%s" % (platform, extension)
        return get(url)

    def download(self, dest=None, channel="nightly", rename=None):
        if dest is None:
            dest = os.pwd

        resp = self._get(dest, channel)
        _, extension, _ = self.platform_components()

        filename = rename if rename is not None else "servo-latest"
        with open(os.path.join(dest, "%s%s" % (filename, extension,)), "w") as f:
            f.write(resp.content)

    def install(self, dest=None, channel="nightly"):
        """Install latest Browser Engine."""
        if dest is None:
            dest = os.pwd

        _, _, decompress = self.platform_components()

        resp = self._get(channel)
        decompress(resp.raw, dest=dest)
        path = which("servo", path=os.path.join(dest, "servo"))
        st = os.stat(path)
        os.chmod(path, st.st_mode | stat.S_IEXEC)
        return path

    def find_binary(self, venv_path=None, channel=None):
        path = which("servo", path=os.path.join(venv_path, "servo"))
        if path is None:
            path = which("servo")
        return path

    def find_webdriver(self, venv_path=None, channel=None):
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

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venev_path=None, channel=None):
        raise NotImplementedError

    def find_webdriver(self, venv_path=None, channel=None):
        raise NotImplementedError

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        return None


class WebKit(Browser):
    """WebKit-specific interface."""

    product = "webkit"
    requirements = None

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        return None

    def find_webdriver(self, venv_path=None, channel=None):
        return None

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        return None

class Ladybird(Browser):
    product = "ladybird"
    requirements = None

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        return which("ladybird")

    def find_webdriver(self, venv_path=None, channel=None):
        return which("WebDriver")

    def install_webdriver(self, dest=None, channel=None, browser_binary=None):
        raise NotImplementedError

    def version(self, binary=None, webdriver_binary=None):
        return None

class WebKitTestRunner(Browser):
    """Interface for WebKitTestRunner.
    """

    product = "wktr"
    requirements = None

    def _find_apple_port_builds(self, channel="main"):
        if channel != "main":
            raise ValueError(f"unable to get builds for branch {channel}")

        system_version, _, _ = platform.mac_ver()
        if system_version in SpecifierSet("==13.*"):
            platform_key = "mac-ventura-x86_64%20arm64"
        elif system_version in SpecifierSet("==12.*"):
            platform_key = "mac-monterey-x86_64%20arm64"
        else:
            raise ValueError(
                f"don't know what platform to use for macOS {system_version}"
            )

        # This should match http://github.com/WebKit/WebKit/blob/main/Websites/webkit.org/wp-content/themes/webkit/build-archives.php
        build_index = get(
            f"https://q1tzqfy48e.execute-api.us-west-2.amazonaws.com/v3/latest/{platform_key}-release"
        ).json()

        builds = []

        for entry in build_index["Items"]:
            creation_time = datetime.fromtimestamp(
                int(entry["creationTime"]["N"]), timezone.utc
            )
            identifier = entry["identifier"]["S"]
            s3_url = entry["s3_url"]["S"]

            builds.append((s3_url, identifier, creation_time))

        return builds

    def _download_metadata_apple_port(self, channel="main"):
        digit_re = re.compile("([0-9]+)")

        def natsort(string_to_split):
            split = digit_re.split(string_to_split)
            # this converts the split numbers into tuples so that "01" < "1"
            split[1::2] = [(int(i), i) for i in split[1::2]]
            return split

        builds = sorted(
            self._find_apple_port_builds(channel),
            key=lambda x: natsort(x[1]),
            reverse=True,
        )
        latest_build = builds[0]

        return {
            "url": latest_build[0],
            "identifier": latest_build[1],
            "creation_time": latest_build[2],
        }

    def download(
        self, dest=None, channel="main", rename=None, version=None, revision=None
    ):
        if platform.system() == "Darwin":
            meta = self._download_metadata_apple_port(channel)
        else:
            raise ValueError("Unsupported platform")

        output_path = self.download_from_url(
            meta["url"],
            dest=dest,
            channel=channel,
            rename=rename,
        )

        dest = os.path.dirname(output_path)  # This is the actual, used dest.

        self.last_revision_used = meta["identifier"]
        with open(os.path.join(dest, "identifier"), "w") as f:
            f.write(self.last_revision_used)

        return output_path

    def install(self, dest=None, channel="main"):
        dest = self._get_browser_binary_dir(dest, channel)
        installer_path = self.download(dest=dest, channel=channel)
        self.logger.info(f"Extracting to {dest}")
        with open(installer_path, "rb") as f:
            unzip(f, dest)

    def install_webdriver(self, dest=None, channel="main", browser_binary=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel="main"):
        path = self._get_browser_binary_dir(venv_path, channel)
        return which("WebKitTestRunner", path=os.path.join(path, "Release"))

    def find_webdriver(self, venv_path=None, channel="main"):
        return None

    def version(self, binary=None, webdriver_binary=None):
        dirname = os.path.dirname(binary)
        identifier = os.path.join(dirname, "..", "identifier")
        if not os.path.exists(identifier):
            return None

        with open(identifier, "r") as f:
            return f.read().strip()


class WebKitGTKMiniBrowser(WebKit):
    def _get_osidversion(self):
        with open('/etc/os-release') as osrelease_handle:
            for line in osrelease_handle.readlines():
                if line.startswith('ID='):
                    os_id = line.split('=')[1].strip().strip('"')
                if line.startswith('VERSION_ID='):
                    version_id = line.split('=')[1].strip().strip('"')
        assert os_id
        assert version_id
        osidversion = os_id + '-' + version_id
        assert ' ' not in osidversion
        assert len(osidversion) > 3
        return osidversion.capitalize()


    def download(self, dest=None, channel=None, rename=None):
        base_dowload_uri = "https://webkitgtk.org/built-products/"
        base_download_dir = base_dowload_uri + "x86_64/release/" + channel + "/" + self._get_osidversion() + "/MiniBrowser/"
        try:
            response = get(base_download_dir + "LAST-IS")
        except requests.exceptions.HTTPError as e:
            if e.response.status_code == 404:
                raise RuntimeError("Can't find a WebKitGTK MiniBrowser %s bundle for %s at %s"
                                   % (channel, self._get_osidversion(), base_dowload_uri))
            raise

        bundle_filename = response.text.strip()
        bundle_url = base_download_dir + bundle_filename

        dest = self._get_browser_download_dir(dest, channel)
        bundle_file_path = os.path.join(dest, bundle_filename)

        self.logger.info("Downloading WebKitGTK MiniBrowser bundle from %s" % bundle_url)
        with open(bundle_file_path, "w+b") as f:
            get_download_to_descriptor(f, bundle_url)

        bundle_filename_no_ext, _ = os.path.splitext(bundle_filename)
        bundle_hash_url = base_download_dir + bundle_filename_no_ext + ".sha256sum"
        bundle_expected_hash = get(bundle_hash_url).text.strip().split(" ")[0]
        bundle_computed_hash = sha256sum(bundle_file_path)

        if bundle_expected_hash != bundle_computed_hash:
            self.logger.error("Calculated SHA256 hash is %s but was expecting %s" % (bundle_computed_hash,bundle_expected_hash))
            raise RuntimeError("The WebKitGTK MiniBrowser bundle at %s has incorrect SHA256 hash." % bundle_file_path)
        return bundle_file_path

    def install(self, dest=None, channel=None, prompt=True):
        dest = self._get_browser_binary_dir(dest, channel)
        bundle_path = self.download(dest, channel)
        bundle_uncompress_directory = os.path.join(dest, "webkitgtk_minibrowser")

        # Clean it from previous runs
        if os.path.exists(bundle_uncompress_directory):
            rmtree(bundle_uncompress_directory)
        os.mkdir(bundle_uncompress_directory)

        with open(bundle_path, "rb") as f:
            unzip(f, bundle_uncompress_directory)

        install_dep_script = os.path.join(bundle_uncompress_directory, "install-dependencies.sh")
        if os.path.isfile(install_dep_script):
            self.logger.info("Executing install-dependencies.sh script from bundle.")
            install_dep_cmd = [install_dep_script]
            if not prompt:
                install_dep_cmd.append("--autoinstall")
            # use subprocess.check_call() directly to display unbuffered stdout/stderr in real-time.
            subprocess.check_call(install_dep_cmd)

        minibrowser_path = os.path.join(bundle_uncompress_directory, "MiniBrowser")
        if not os.path.isfile(minibrowser_path):
            raise RuntimeError("Can't find a MiniBrowser binary at %s" % minibrowser_path)

        os.remove(bundle_path)
        install_ok_file = os.path.join(bundle_uncompress_directory, ".installation-ok")
        open(install_ok_file, "w").close()  # touch
        self.logger.info("WebKitGTK MiniBrowser bundle for channel %s installed." % channel)
        return minibrowser_path

    def _find_executable_in_channel_bundle(self, binary, venv_path=None, channel=None):
        if venv_path:
            venv_base_path = self._get_browser_binary_dir(venv_path, channel)
            bundle_dir = os.path.join(venv_base_path, "webkitgtk_minibrowser")
            install_ok_file = os.path.join(bundle_dir, ".installation-ok")
            if os.path.isfile(install_ok_file):
                return which(binary, path=bundle_dir)
        return None

    def find_binary(self, venv_path=None, channel=None):
        minibrowser_path = self._find_executable_in_channel_bundle("MiniBrowser", venv_path, channel)
        if minibrowser_path:
            return minibrowser_path

        libexecpaths = ["/usr/libexec/webkit2gtk-4.0"]  # Fedora path
        triplet = "x86_64-linux-gnu"
        # Try to use GCC to detect this machine triplet
        gcc = which("gcc")
        if gcc:
            try:
                triplet = call(gcc, "-dumpmachine").strip()
            except subprocess.CalledProcessError:
                pass
        # Add Debian/Ubuntu path
        libexecpaths.append("/usr/lib/%s/webkit2gtk-4.0" % triplet)
        return which("MiniBrowser", path=os.pathsep.join(libexecpaths))

    def find_webdriver(self, venv_path=None, channel=None):
        webdriver_path = self._find_executable_in_channel_bundle("WebKitWebDriver", venv_path, channel)
        if not webdriver_path:
            webdriver_path = which("WebKitWebDriver")
        return webdriver_path

    def version(self, binary=None, webdriver_binary=None):
        if binary is None:
            return None
        try:  # WebKitGTK MiniBrowser before 2.26.0 doesn't support --version
            output = call(binary, "--version").strip()
        except subprocess.CalledProcessError:
            return None
        # Example output: "WebKitGTK 2.26.1"
        if output:
            m = re.match(r"WebKitGTK (.+)", output)
            if not m:
                self.logger.warning("Failed to extract version from: %s" % output)
                return None
            return m.group(1)
        return None


class Epiphany(Browser):
    """Epiphany-specific interface."""

    product = "epiphany"
    requirements = None

    def download(self, dest=None, channel=None, rename=None):
        raise NotImplementedError

    def install(self, dest=None, channel=None):
        raise NotImplementedError

    def find_binary(self, venv_path=None, channel=None):
        return which("epiphany")

    def find_webdriver(self, venv_path=None, channel=None):
        return which("WebKitWebDriver")

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
