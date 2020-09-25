import mock
import subprocess
import logging
import sys
import pytest
import inspect

from tools.wpt import browser


logger = logging.getLogger()


def test_all_browser_abc():
    # Make sure all subclasses of Browser implement all abstract methods
    # (except some known base classes). This is a basic sanity test in case
    # we change the ABC interface of Browser as we only instantiate some
    # products in unit tests.
    classes = inspect.getmembers(browser)
    for name, cls in classes:
        if cls in (browser.Browser, browser.ChromeAndroidBase):
            continue
        if inspect.isclass(cls) and issubclass(cls, browser.Browser):
            assert not inspect.isabstract(cls), "%s is abstract" % name


@mock.patch('tools.wpt.browser.find_executable')
def test_chrome_find_webdriver(mocked_find_executable):
    # Cannot find ChromeDriver
    chrome = browser.Chrome(logger)
    mocked_find_executable.return_value = None
    assert chrome.find_webdriver() is None

    # ChromeDriver binary cannot be called.
    chrome = browser.Chrome(logger)
    mocked_find_executable.return_value = '/usr/bin/chromedriver'
    chrome.webdriver_version = mock.MagicMock(return_value=None)
    assert chrome.find_webdriver() is None

    # Browser binary cannot be called.
    chrome = browser.Chrome(logger)
    mocked_find_executable.return_value = '/usr/bin/chromedriver'
    chrome.webdriver_version = mock.MagicMock(return_value='70.0.1')
    chrome.version = mock.MagicMock(return_value=None)
    assert chrome.find_webdriver(browser_binary='/usr/bin/chrome') == '/usr/bin/chromedriver'

    # Browser version matches.
    chrome = browser.Chrome(logger)
    mocked_find_executable.return_value = '/usr/bin/chromedriver'
    chrome.webdriver_version = mock.MagicMock(return_value='70.0.1')
    chrome.version = mock.MagicMock(return_value='70.1.5')
    assert chrome.find_webdriver(browser_binary='/usr/bin/chrome') == '/usr/bin/chromedriver'

    # Browser version doesn't match.
    chrome = browser.Chrome(logger)
    mocked_find_executable.return_value = '/usr/bin/chromedriver'
    chrome.webdriver_version = mock.MagicMock(return_value='70.0.1')
    chrome.version = mock.MagicMock(return_value='69.0.1')
    assert chrome.find_webdriver(browser_binary='/usr/bin/chrome') is None


# On Windows, webdriver_version directly calls _get_fileversion, so there is no
# logic to test there.
@pytest.mark.skipif(sys.platform.startswith('win'), reason='just uses _get_fileversion on Windows')
@mock.patch('tools.wpt.browser.call')
def test_chrome_webdriver_version(mocked_call):
    chrome = browser.Chrome(logger)
    webdriver_binary = '/usr/bin/chromedriver'

    # Working cases.
    mocked_call.return_value = 'ChromeDriver 84.0.4147.30'
    assert chrome.webdriver_version(webdriver_binary) == '84.0.4147.30'
    mocked_call.return_value = 'ChromeDriver 87.0.1 (abcd1234-refs/branch-heads/4147@{#310})'
    assert chrome.webdriver_version(webdriver_binary) == '87.0.1'

    # Various invalid version strings
    mocked_call.return_value = 'Chrome 84.0.4147.30 (dev)'
    assert chrome.webdriver_version(webdriver_binary) is None
    mocked_call.return_value = 'ChromeDriver New 84.0.4147.30'
    assert chrome.webdriver_version(webdriver_binary) is None
    mocked_call.return_value = ''
    assert chrome.webdriver_version(webdriver_binary) is None

    # The underlying subprocess call throws.
    mocked_call.side_effect = subprocess.CalledProcessError(5, 'cmd', output='Call failed')
    assert chrome.webdriver_version(webdriver_binary) is None


@mock.patch('subprocess.check_output')
def test_safari_version(mocked_check_output):
    safari = browser.Safari(logger)

    # Safari
    mocked_check_output.return_value = b'Included with Safari 12.1 (14607.1.11)'
    assert safari.version(webdriver_binary="safaridriver") == '12.1 (14607.1.11)'

    # Safari Technology Preview
    mocked_check_output.return_value = b'Included with Safari Technology Preview (Release 67, 13607.1.9.0.1)'
    assert safari.version(webdriver_binary="safaridriver") == 'Technology Preview (Release 67, 13607.1.9.0.1)'

@mock.patch('subprocess.check_output')
def test_safari_version_errors(mocked_check_output):
    safari = browser.Safari(logger)

    # No webdriver_binary
    assert safari.version() is None

    # `safaridriver --version` return gibberish
    mocked_check_output.return_value = b'gibberish'
    assert safari.version(webdriver_binary="safaridriver") is None

    # `safaridriver --version` fails (as it does for Safari <=12.0)
    mocked_check_output.return_value = b'dummy'
    mocked_check_output.side_effect = subprocess.CalledProcessError(1, 'cmd')
    assert safari.version(webdriver_binary="safaridriver") is None


@mock.patch('subprocess.check_output')
def test_webkitgtk_minibrowser_version(mocked_check_output):
    webkitgtk_minibrowser = browser.WebKitGTKMiniBrowser(logger)

    # stable version
    mocked_check_output.return_value = b'WebKitGTK 2.26.1\n'
    assert webkitgtk_minibrowser.version(binary='MiniBrowser') == '2.26.1'

    # nightly version
    mocked_check_output.return_value = b'WebKitGTK 2.27.1 (r250823)\n'
    assert webkitgtk_minibrowser.version(binary='MiniBrowser') == '2.27.1 (r250823)'

@mock.patch('subprocess.check_output')
def test_webkitgtk_minibrowser_version_errors(mocked_check_output):
    webkitgtk_minibrowser = browser.WebKitGTKMiniBrowser(logger)

    # No binary
    assert webkitgtk_minibrowser.version() is None

    # `MiniBrowser --version` return gibberish
    mocked_check_output.return_value = b'gibberish'
    assert webkitgtk_minibrowser.version(binary='MiniBrowser') is None

    # `MiniBrowser --version` fails (as it does for MiniBrowser <= 2.26.0)
    mocked_check_output.return_value = b'dummy'
    mocked_check_output.side_effect = subprocess.CalledProcessError(1, 'cmd')
    assert webkitgtk_minibrowser.version(binary='MiniBrowser') is None


# The test below doesn't work on Windows because distutils find_binary()
# on Windows only works if the binary name ends with a ".exe" suffix.
# But, WebKitGTK itself doesn't support Windows, so lets skip the test.
@pytest.mark.skipif(sys.platform.startswith('win'), reason='test not needed on Windows')
@mock.patch('os.path.isfile')
def test_webkitgtk_minibrowser_find_binary(mocked_os_path_isfile):
    webkitgtk_minibrowser = browser.WebKitGTKMiniBrowser(logger)

    # No MiniBrowser found
    mocked_os_path_isfile.side_effect = lambda path: path == '/etc/passwd'
    assert webkitgtk_minibrowser.find_binary() is None

    # Found on the default Fedora path
    fedora_minibrowser_path = '/usr/libexec/webkit2gtk-4.0/MiniBrowser'
    mocked_os_path_isfile.side_effect = lambda path: path == fedora_minibrowser_path
    assert webkitgtk_minibrowser.find_binary() == fedora_minibrowser_path

    # Found on the default Debian path for AMD64 (gcc not available)
    debian_minibrowser_path_amd64 = '/usr/lib/x86_64-linux-gnu/webkit2gtk-4.0/MiniBrowser'
    mocked_os_path_isfile.side_effect = lambda path: path == debian_minibrowser_path_amd64
    assert webkitgtk_minibrowser.find_binary() == debian_minibrowser_path_amd64

    # Found on the default Debian path for AMD64 (gcc available but gives an error)
    debian_minibrowser_path_amd64 = '/usr/lib/x86_64-linux-gnu/webkit2gtk-4.0/MiniBrowser'
    mocked_os_path_isfile.side_effect = lambda path: path in [debian_minibrowser_path_amd64, '/usr/bin/gcc']
    with mock.patch('subprocess.check_output', return_value = b'error', side_effect = subprocess.CalledProcessError(1, 'cmd')):
        assert webkitgtk_minibrowser.find_binary() == debian_minibrowser_path_amd64

        # Found on the default Debian path for ARM64 (gcc available)
        debian_minibrowser_path_arm64 = '/usr/lib/aarch64-linux-gnu/webkit2gtk-4.0/MiniBrowser'
        mocked_os_path_isfile.side_effect = lambda path: path in [debian_minibrowser_path_arm64, '/usr/bin/gcc']
        with mock.patch('subprocess.check_output', return_value = b'aarch64-linux-gnu'):
            assert webkitgtk_minibrowser.find_binary() == debian_minibrowser_path_arm64
