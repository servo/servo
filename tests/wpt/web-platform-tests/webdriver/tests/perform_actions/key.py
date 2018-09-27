import pytest

from webdriver.error import NoSuchWindowException

from tests.perform_actions.support.keys import Keys
from tests.perform_actions.support.refine import get_keys


def test_null_response_value(session, key_chain):
    value = key_chain.key_up("a").perform()
    assert value is None


def test_no_browsing_context(session, closed_window, key_chain):
    with pytest.raises(NoSuchWindowException):
        key_chain.key_up("a").perform()


def test_backspace_erases_keys(session, key_reporter, key_chain):
    key_chain \
        .send_keys("efcd") \
        .send_keys([Keys.BACKSPACE, Keys.BACKSPACE]) \
        .perform()

    assert get_keys(key_reporter) == "ef"
