import mock
import subprocess
import logging
import sys
import pytest

from tools.wpt import browser


logger = logging.getLogger()


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
