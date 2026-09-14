import pytest

from tests.classic.perform_actions.support.refine import (
    get_events,
    wait_for_events
)
from tests.support.keys import Keys


def test_scroll_on_not_scrollable_element(
    session, test_actions_wheel_page, wheel_chain
):
    session.url = test_actions_wheel_page(events=["wheel"])

    target = session.find.css("#not-scrollable", all=False)

    wheel_chain.scroll(0, 0, 55, 70, origin=target).perform()

    # Wheel events are dispatched synchronously during action processing,
    # so they are available immediately after perform() returns.
    events = get_events(session)
    assert len(events) == 1

    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 55
    assert events[0]["deltaY"] == 70
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "not-scrollable-content"


def test_scroll_on_element_with_overflow_scroll(
    session, test_actions_wheel_page, wheel_chain
):
    session.url = test_actions_wheel_page(events=["wheel", "scroll"])

    target = session.find.css("#scrollable", all=False)

    wheel_chain.scroll(0, 0, 60, 80, origin=target).perform()

    events = wait_for_events(session, 2)
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 60
    assert events[0]["deltaY"] == 80
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"

    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "scrollable"


def test_scroll_emits_scrollend_event(session, test_actions_wheel_page, wheel_chain):
    session.url = test_actions_wheel_page(events=["wheel", "scroll", "scrollend"])

    target = session.find.css("#scrollable", all=False)

    wheel_chain.scroll(0, 0, 65, 85, origin=target).perform()

    events = wait_for_events(session, 3)

    assert events[0]["type"] == "wheel"
    assert events[0]["target"] == "scrollable-content"

    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "scrollable"

    assert events[2]["type"] == "scrollend"
    assert events[2]["target"] == "scrollable"


def test_scroll_with_key_pressed(
    session, test_actions_wheel_page, key_chain, wheel_chain
):
    session.url = test_actions_wheel_page(events=["wheel"])

    scrollable = session.find.css("#scrollable", all=False)

    key_chain.key_down(Keys.R_SHIFT).perform()
    wheel_chain.scroll(0, 0, 50, 75, origin=scrollable).perform()
    key_chain.key_up(Keys.R_SHIFT).perform()

    # Wheel events are dispatched synchronously during action processing,
    # so they are available immediately after perform() returns.
    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["shiftKey"] is True


def test_scroll_more_than_a_page(session, test_actions_wheel_page, wheel_chain):
    session.url = test_actions_wheel_page(events=["wheel", "scroll"])

    delta_huge = 3000

    target = session.find.css("#scrollable", all=False)

    wheel_chain.scroll(0, 0, delta_huge, delta_huge, origin=target).perform()

    events = wait_for_events(session, 2)
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_huge
    assert events[0]["deltaY"] == delta_huge
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"
    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "scrollable"
