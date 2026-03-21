import pytest

from tests.classic.perform_actions.support.refine import (
    get_events,
    wait_for_events
)
from tests.classic.perform_actions.support.mouse import (
    get_inview_center,
    get_viewport_rect,
)
from tests.support.keys import Keys
from . import assert_events


def parametrize_deltas(func):
    return pytest.mark.parametrize(
        "delta_x, delta_y",
        [
            (5, 0),
            (0, 10),
            (5, 10),
        ],
        ids=[
            "delta-x",
            "delta-y",
            "delta-x-and-y",
        ],
    )(func)


@parametrize_deltas
def test_scroll_on_not_scrollable_element(
    session, test_actions_scroll_page, wheel_chain, delta_x, delta_y
):
    target = session.find.css("#not-scrollable", all=False)

    wheel_chain.scroll(0, 0, delta_x, delta_y, origin=target).perform()

    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "not-scrollable-content"


@parametrize_deltas
def test_scroll_on_element_with_overflow_scroll(
    session, test_actions_scroll_page, wheel_chain, delta_x, delta_y
):
    target = session.find.css("#scrollable", all=False)

    wheel_chain.scroll(0, 0, delta_x, delta_y, origin=target).perform()

    events = get_events(session)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"


@parametrize_deltas
def test_scroll_on_iframe_with_overflow_scroll(
    session, test_actions_scroll_page, wheel_chain, delta_x, delta_y
):
    target = session.find.css("#iframe", all=False)

    wheel_chain.scroll(0, 0, delta_x, delta_y, origin=target).perform()

    # Chrome requires some time (~10-20ms) to process the event from the iframe,
    # so we wait for it.
    events = wait_for_events(session, 1, timeout=0.5, interval=0.02)

    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "iframeContent"


@parametrize_deltas
def test_scroll_element_in_iframe_with_overflow_scroll(
    session, test_actions_scroll_page, wheel_chain, delta_x, delta_y
):
    frame = session.find.css("#iframe", all=False)
    session.switch_to_frame(frame)

    target = session.find.css("div", all=False)
    wheel_chain.scroll(0, 0, delta_x, delta_y, origin=target).perform()

    session.switch_to_parent_frame()

    events = wait_for_events(session, 1)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "iframeContent"


@parametrize_deltas
@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
def test_scroll_element_in_shadow_tree(
    session, get_test_page, wheel_chain, mode, nested, delta_x, delta_y
):
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

    wheel_chain.scroll(0, 0, delta_x, delta_y, origin=scrollable).perform()

    expected_events = [
        {
            "type": "wheel",
            "target": "scrollableShadowTreeContent",
            "deltaX": delta_x,
            "deltaY": delta_y,
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
    assert events[0]["shiftKey"] is True


def test_scroll_more_than_a_page(session, test_actions_scroll_page, wheel_chain):
    delta_huge = 3000

    target = session.find.css("#scrollable", all=False)

    wheel_chain.scroll(0, 0, delta_huge, delta_huge, origin=target).perform()

    session.switch_to_parent_frame()

    events = wait_for_events(session, 1)
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_huge
    assert events[0]["deltaY"] == delta_huge
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"
