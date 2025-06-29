import pytest

from webdriver.error import MoveTargetOutOfBoundsException, NoSuchWindowException

from tests.classic.perform_actions.support.refine import get_events, wait_for_events
from tests.classic.perform_actions.support.mouse import (
    get_inview_center,
    get_viewport_rect,
)
from tests.support.keys import Keys
from . import assert_events


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
        origin = session.find.css("#scrollable", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        wheel_chain.scroll(-100, -100, 10, 20, origin="viewport").perform()


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

    # Chrome requires some time (~10-20ms) to process the event from the iframe,
    # so we wait for it.
    events = wait_for_events(session, 1, timeout=0.5, interval=0.02)

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
    center = get_inview_center(scrollable.rect, get_viewport_rect(session))

    # Add a simplified event recorder to track events in the test ShadowRoot.
    session.execute_script(
        """
        window.allEvents = { events: [] };

        arguments[0].addEventListener("wheel", event => {
            const data = {
                type: event.type,
                pageX: event.pageX,
                pageY: event.pageY,
                deltaX: event.deltaX,
                deltaY: event.deltaY,
                deltaZ: event.deltaZ,
                target: event.target.id || event.target.localName || event.target.documentElement?.localName,
            };

            window.allEvents.events.push(data);
        });
        arguments[0].addEventListener("scroll", event => {
            window.allEvents.events.push({
                type: event.type,
                target: event.target.id || event.target.localName || event.target.documentElement?.localName,
            });
        });
        """,
        args=(scrollable,),
    )

    wheel_chain.scroll(0, 0, 5, 10, origin=scrollable).perform()

    expected_events = [
        {
            "type": "wheel",
            "target": "scrollableShadowTreeContent",
            "deltaX": 5,
            "deltaY": 10,
            "deltaZ": 0,
            "pageX": pytest.approx(center["x"], abs=1.0),
            "pageY": pytest.approx(center["y"], abs=1.0),
        },
        {
            "type": "scroll",
            "target": "scrollableShadowTree",
        },
    ]

    events = wait_for_events(session, min_count=len(expected_events))
    assert_events(events, expected_events)


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
