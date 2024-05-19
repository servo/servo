import pytest

from webdriver.error import NoSuchWindowException

import time
from tests.classic.perform_actions.support.refine import get_events
from tests.support.keys import Keys
from tests.support.sync import Poll


def test_null_response_value(session, wheel_chain):
    value = wheel_chain.scroll(0, 0, 0, 10).perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


def test_no_browsing_context(session, closed_window, wheel_chain):
    with pytest.raises(NoSuchWindowException):
        wheel_chain.scroll(0, 0, 0, 10).perform()


def test_scroll_not_scrollable(session, test_actions_scroll_page, wheel_chain):
    target = session.find.css("#not-scrollable", all=False)

    wheel_chain.scroll(0, 0, 5, 10, origin=target).perform()

    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 5
    assert events[0]["deltaY"] == 10
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "not-scrollable-content"


def test_scroll_scrollable_overflow(session, test_actions_scroll_page, wheel_chain):
    target = session.find.css("#scrollable", all=False)

    wheel_chain.scroll(0, 0, 5, 10, origin=target).perform()

    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 5
    assert events[0]["deltaY"] == 10
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"


def test_scroll_iframe(session, test_actions_scroll_page, wheel_chain):
    target = session.find.css("#iframe", all=False)

    wheel_chain.scroll(0, 0, 5, 10, origin=target).perform()

    # Chrome requires some time (~10-20ms) to process the event from the iframe, so we wait for it.
    def wait_for_events(_):
        return len(get_events(session)) > 0

    Poll(session, timeout=0.5, interval=0.01, message='No wheel events found').until(wait_for_events)
    events = get_events(session)

    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 5
    assert events[0]["deltaY"] == 10
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "iframeContent"


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
def test_scroll_shadow_tree(session, get_test_page, wheel_chain, mode, nested):
    session.url = get_test_page(
        shadow_doc="""
        <div id="scrollableShadowTree"
             style="width: 100px; height: 100px; overflow: auto;">
            <div
                id="scrollableShadowTreeContent"
                style="width: 600px; height: 1000px; background-color:blue"></div>
        </div>""",
        shadow_root_mode=mode,
        nested_shadow_dom=nested,
    )

    shadow_root = session.find.css("custom-element", all=False).shadow_root

    if nested:
        shadow_root = shadow_root.find_element(
            "css selector", "inner-custom-element"
        ).shadow_root

    scrollable = shadow_root.find_element("css selector", "#scrollableShadowTree")

    # Add a simplified event recorder to track events in the test ShadowRoot.
    session.execute_script(
        """
        window.wheelEvents = [];
        arguments[0].addEventListener("wheel",
            function(event) {
                window.wheelEvents.push({
                    "deltaX": event.deltaX,
                    "deltaY": event.deltaY,
                    "target": event.target.id
                });
            }
        );
    """,
        args=(scrollable,),
    )

    wheel_chain.scroll(0, 0, 5, 10, origin=scrollable).perform()

    events = session.execute_script("return window.wheelEvents;") or []
    assert len(events) == 1
    assert events[0]["deltaX"] == 5
    assert events[0]["deltaY"] == 10
    assert events[0]["target"] == "scrollableShadowTreeContent"


def test_scroll_with_key_pressed(
    session, test_actions_scroll_page, key_chain, wheel_chain
):
    scrollable = session.find.css("#scrollable", all=False)

    key_chain.key_down(Keys.R_SHIFT).perform()
    wheel_chain.scroll(0, 0, 5, 10, origin=scrollable).perform()
    key_chain.key_up(Keys.R_SHIFT).perform()

    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["shiftKey"] == True
