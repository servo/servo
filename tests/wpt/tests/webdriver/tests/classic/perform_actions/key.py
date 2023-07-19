import pytest

from webdriver.error import NoSuchWindowException

from tests.classic.perform_actions.support.refine import get_keys
from tests.support.keys import Keys


def test_null_response_value(session, key_chain):
    value = key_chain.key_up("a").perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, key_chain):
    with pytest.raises(NoSuchWindowException):
        key_chain.key_up("a").perform()


def test_no_browsing_context(session, closed_frame, key_chain):
    with pytest.raises(NoSuchWindowException):
        key_chain.key_up("a").perform()


def test_element_not_focused(session, test_actions_page, key_chain):
    key_reporter = session.find.css("#keys", all=False)

    key_chain.key_down("a").key_up("a").perform()

    assert get_keys(key_reporter) == ""


def test_backspace_erases_keys(session, key_reporter, key_chain):
    key_chain \
        .send_keys("efcd") \
        .send_keys([Keys.BACKSPACE, Keys.BACKSPACE]) \
        .perform()

    assert get_keys(key_reporter) == "ef"


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
def test_element_in_shadow_tree(session, get_test_page, key_chain, mode, nested):
    session.url = get_test_page(
        shadow_doc="<div><input type=text></div>",
        shadow_root_mode=mode,
        nested_shadow_dom=nested,
    )

    shadow_root = session.find.css("custom-element", all=False).shadow_root

    if nested:
        shadow_root = shadow_root.find_element(
            "css selector", "inner-custom-element"
        ).shadow_root

    input_el = shadow_root.find_element("css selector", "input")
    input_el.click()

    key_chain.key_down("a").key_up("a").perform()

    assert input_el.property("value") == "a"
