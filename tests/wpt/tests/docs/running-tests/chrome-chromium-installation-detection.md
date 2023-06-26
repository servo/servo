# Detection and Installation of Browser and WebDriver Binaries and for Chrome and Chromium

This is a detailed description of the process in which WPT detects and installs the browser
components for Chrome and Chromium. This process can seem convoluted and difficult to
understand at first glance, but the reason for this process is to best ensure these components
are compatible with each other and are the intended items that the user is trying to test.

## Chrome

### Detection
**Browser**: Because WPT does not offer installation of Chrome browser binaries, it will
not attempt to detect a Chrome browser binary in the virtual environment directory.
Instead, commonly-used installation locations on various operating systems are checked to
detect a valid Chrome binary. This detection process is only used if the user has not passed
a binary path as an argument using the `--binary` flag.

**WebDriver**: ChromeDriver detection for Chrome will only occur if a valid browser binary
has been found. Once the browser binary version is detected, the virtual environment
directory will be checked to see if a matching ChromeDriver version is already installed.
If the browser and ChromeDriver versions do not match, the ChromeDriver binary will be
removed from the directory and the user will be prompted to begin the webdriver installation
process. A ChromeDriver version is considered matching the browser version if ChromeDriver shares
the same major version, or next major version when testing Chrome Dev. For example, Chrome 98.x.x.x
is considered to match ChromeDriver version 98.x.x.x, or also ChromeDriver 99.x.x.x if testing
Chrome Dev.

Note: Both Chrome and Chromium’s versions of ChromeDriver are stored in separate
directories in the virtual environment directory i.e
`_venv3/bin/{chrome|chromium}/{chromedriver}`. This safeguards from accidentally
using Chromium’s ChromeDriver for a Chrome run and vice versa. Additionally, there
is no need to reinstall ChromeDriver versions if switching between testing Chrome and Chromium.

### Installation
**Browser**: Browser binary installation is not provided through WPT and will throw a
`NotImplementedError` if attempted via `./wpt install`. The user will need to
have a browser binary on their system that can be detected or provide a path explicitly
using the `--binary` flag.

**WebDriver**: A version of ChromeDriver will only be installed once a Chrome browser binary
has been given or detected. A `FileNotFoundError` will be raised if the user tries to download
ChromeDriver via `./wpt install` and a browser binary is not located. After browser binary
detection, a version of ChromeDriver that matches the browser binary will be installed.
The download source for this ChromeDriver is
[described here](https://chromedriver.chromium.org/downloads/version-selection).
If a matching ChromeDriver version cannot be found using this process, it is assumed that
the Chrome browser binary is a dev version which does not have a ChromeDriver version available
through official releases. In this case, the Chromium revision associated with this version is
detected from [OmahaProxy](https://omahaproxy.appspot.com/) and used to download
Chromium's version of ChromeDriver for use from Chromium snapshots, as this is currently
the closest version we can match for Chrome Dev. Finally, if the revision number detected is
not available in Chromium snapshots, or if the version does not match any revision number,
the latest revision of Chromium's ChromeDriver is installed from Chromium snapshots.

## Chromium

### Detection
**Browser**: Chromium browser binary detection is only done in the virtual
environment directory `_venv3/browsers/{channel}/`, not on the user’s system
outside of this directory. This detection process is only used if the user has
not passed a binary path as an argument using the `--binary` flag.

**WebDriver**: ChromeDriver detection for Chromium will only occur if a valid browser binary has
been found. Once the browser binary version is detected, the virtual environment directory will
be checked to see if a matching ChromeDriver version is already installed. If the versions do not
match, the ChromeDriver binary will be removed from the directory and the user will be prompted to
begin the webdriver installation process. For Chromium, the ChromeDriver and browser versions must be
the same to be considered matching. For example, Chromium 99.0.4844.74 will only match ChromeDriver
99.0.4844.74.

### Installation
**Browser**: Chromium’s browser binary will be installed from
[Chromium snapshots storage](https://storage.googleapis.com/chromium-browser-snapshots/index.html).
The last revision associated with the user’s operating system will be downloaded
(this revision is obtained by the LAST_CHANGE designation from the snapshots bucket).
Chromium does not have varying channels, so the installation uses the default `nightly`
designation. The install path is `_venv3/browsers/nightly/{chromium_binary}`.

Note: If this download process is successful, the Chromium snapshot URL that the browser
binary was downloaded from will be kept during the current invocation. If a Chromium ChromeDriver
is also downloaded later to match this browser binary, the same URL is used for that download to
ensure both components are downloaded from the same source.

**WebDriver**: A version of ChromeDriver will only be installed once a Chromium browser binary
has been given or detected. A FileNotFoundError will be raised if the user tries to download
ChromeDriver via the install command and a browser binary is not located. A version of
ChromeDriver that matches the version of the browser binary will be installed. The download
source for this ChromeDriver will be the Chromium snapshots.  If a Chromium browser
binary and webdriver are installed in the same invocation of `./wpt run`
(for example, by passing both `--install-browser` and `--install-webdriver` flags), then the
browser binary and ChromeDriver will be pulled from the same Chromium snapshots URL (see Note
from browser installation). Although unusual, if a Chromium browser binary is detected and
it is not the tip-of-tree revision and the browser binary was not downloaded and installed
during this invocation of `./wpt run` and the currently installed ChromeDriver version does
not match the browser version, then an attempt will be made to detect the revision number from
the browser binary version using the [OmahaProxy](https://omahaproxy.appspot.com/)
and download the matching ChromeDriver using this revision number from Chromium snapshots.
