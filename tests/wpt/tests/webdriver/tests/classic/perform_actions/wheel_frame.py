import pytest

from tests.classic.perform_actions.support.refine import wait_for_events
from . import assert_scroll_position


@pytest.mark.parametrize("iframe_domain", [None, "alt"], ids=["same-origin", "cross-origin"])
def test_scroll_on_not_scrollable_element_in_iframe(
    session, new_tab_classic, test_actions_wheel_page, wheel_chain, iframe_domain
):
    page_kwargs = {"iframe_domain": iframe_domain} if iframe_domain else {}
    session.url = test_actions_wheel_page(**page_kwargs)

    frame = session.find.css("#iframe", all=False)
    session.switch_to_frame(frame)

    target = session.find.css("#inner-not-scrollable", all=False)
    wheel_chain.scroll(0, 0, 60, 85, origin=target).perform()

    scroller = session.execute_script("return document.scrollingElement")
    assert_scroll_position(session, scroller, 60, 85)


@pytest.mark.parametrize("iframe_domain", [None, "alt"], ids=["same-origin", "cross-origin"])
def test_scroll_on_scrollable_element_in_iframe(
    session, new_tab_classic, test_actions_wheel_page, wheel_chain, iframe_domain
):
    page_kwargs = {"iframe_domain": iframe_domain} if iframe_domain else {}
    session.url = test_actions_wheel_page(**page_kwargs)

    frame = session.find.css("#iframe", all=False)
    session.switch_to_frame(frame)

    target = session.find.css("#inner-scrollable", all=False)
    wheel_chain.scroll(0, 0, 65, 90, origin=target).perform()

    assert_scroll_position(session, target, 65, 90)


@pytest.mark.parametrize("iframe_domain", [None, "alt"], ids=["same-origin", "cross-origin"])
def test_wheel_event_in_iframe(
    session, new_tab_classic, test_actions_wheel_page, wheel_chain, iframe_domain
):
    page_kwargs = {"iframe_domain": iframe_domain} if iframe_domain else {}
    session.url = test_actions_wheel_page(events=["wheel", "scroll"], **page_kwargs)

    frame = session.find.css("#iframe", all=False)
    session.switch_to_frame(frame)

    target = session.find.css("#inner-scrollable", all=False)
    wheel_chain.scroll(0, 0, 55, 80, origin=target).perform()

    session.switch_to_parent_frame()

    events = wait_for_events(session, 2)

    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 55
    assert events[0]["deltaY"] == 80
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "inner-scrollable-content"

    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "inner-scrollable"
