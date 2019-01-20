import mock
import subprocess
import logging

from tools.wpt import browser


logger = logging.getLogger()


@mock.patch('subprocess.check_output')
def test_safari_version(mocked_check_output):
    safari = browser.Safari(logger)

    # Safari
    mocked_check_output.return_value = 'Included with Safari 12.1 (14607.1.11)'
    assert safari.version(webdriver_binary="safaridriver") == '12.1 (14607.1.11)'

    # Safari Technology Preview
    mocked_check_output.return_value = 'Included with Safari Technology Preview (Release 67, 13607.1.9.0.1)'
    assert safari.version(webdriver_binary="safaridriver") == 'Technology Preview (Release 67, 13607.1.9.0.1)'

@mock.patch('subprocess.check_output')
def test_safari_version_errors(mocked_check_output):
    safari = browser.Safari(logger)

    # No webdriver_binary
    assert safari.version() is None

    # `safaridriver --version` return gibberish
    mocked_check_output.return_value = 'gibberish'
    assert safari.version(webdriver_binary="safaridriver") is None

    # `safaridriver --version` fails (as it does for Safari <=12.0)
    mocked_check_output.return_value = 'dummy'
    mocked_check_output.side_effect = subprocess.CalledProcessError(1, 'cmd')
    assert safari.version(webdriver_binary="safaridriver") is None
