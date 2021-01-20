import pytest

from webdriver.error import NoSuchWindowException

from tests.perform_actions.support.refine import get_events
from tests.support.asserts import assert_move_to_coordinates
from tests.support.helpers import filter_dict


def test_null_response_value(session, wheel_chain):
    value = wheel_chain.scroll(0, 0, 0, 10).perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


def test_no_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


def test_wheel_scroll(session, test_actions_scroll_page, wheel_chain):
    session.execute_script("document.scrollingElement.scrollTop = 0")

    outer = session.find.css("#outer", all=False)
    wheel_chain.scroll(0, 0, 5, 10, origin=outer).perform()
    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] >= 5
    assert events[0]["deltaY"] >= 10
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "outer"


def test_wheel_scroll_overflow(session, test_actions_scroll_page, wheel_chain):
    session.execute_script("document.scrollingElement.scrollTop = 0")

    scrollable = session.find.css("#scrollable", all=False)
    wheel_chain.scroll(0, 0, 5, 10, origin=scrollable).perform()
    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] >= 5
    assert events[0]["deltaY"] >= 10
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollContent"


def test_wheel_scroll_iframe(session, test_actions_scroll_page, wheel_chain):
    session.execute_script("document.scrollingElement.scrollTop = 0")

    subframe = session.find.css("#subframe", all=False)
    wheel_chain.scroll(0, 0, 5, 10, origin=subframe).perform()
    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] >= 5
    assert events[0]["deltaY"] >= 10
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "iframeContent"
