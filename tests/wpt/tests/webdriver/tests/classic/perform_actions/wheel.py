import pytest

from webdriver.error import MoveTargetOutOfBoundsException, NoSuchWindowException


def test_null_response_value(session, wheel_chain):
    value = wheel_chain.scroll(0, 0, 0, 10).perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


def test_no_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


@pytest.mark.parametrize("origin", ["element", "viewport"])
def test_params_actions_origin_outside_viewport(
    session, test_actions_scroll_page, wheel_chain, origin
):
    if origin == "element":
        origin = session.find.css("#not-scrollable", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        wheel_chain.scroll(-100, -100, 10, 20, origin="viewport").perform()
