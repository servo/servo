import pytest

from webdriver.error import InvalidArgumentException, NoSuchWindowException

from tests.classic.perform_actions.support.refine import get_events
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


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
def test_wheel_scroll_shadow_tree(session, get_test_page, wheel_chain, mode, nested):
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
    assert events[0]["deltaX"] >= 5
    assert events[0]["deltaY"] >= 10
    assert events[0]["target"] == "scrollableShadowTreeContent"


@pytest.mark.parametrize("missing", ["x", "y", "deltaX", "deltaY"])
def test_wheel_missing_prop(session, test_actions_scroll_page, wheel_chain, missing):
    session.execute_script("document.scrollingElement.scrollTop = 0")

    outer = session.find.css("#outer", all=False)
    actions = wheel_chain.scroll(0, 0, 5, 10, origin=outer)
    del actions._actions[-1][missing]
    with pytest.raises(InvalidArgumentException):
        actions.perform()
