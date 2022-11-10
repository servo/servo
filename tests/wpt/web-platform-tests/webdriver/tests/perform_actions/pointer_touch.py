# META: timeout=long

import pytest

from webdriver.error import NoSuchWindowException, StaleElementReferenceException
from tests.perform_actions.support.mouse import get_inview_center, get_viewport_rect
from tests.perform_actions.support.refine import get_events


def test_null_response_value(session, touch_chain):
    value = touch_chain.click().perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, touch_chain):
    with pytest.raises(NoSuchWindowException):
        touch_chain.click().perform()


def test_no_browsing_context(session, closed_frame, touch_chain):
    with pytest.raises(NoSuchWindowException):
        touch_chain.click().perform()


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, touch_chain, as_frame):
    element = stale_element("<input>", "input", as_frame=as_frame)

    with pytest.raises(StaleElementReferenceException):
        touch_chain.click(element=element).perform()


def test_touch_pointer_properties(session, test_actions_pointer_page, touch_chain):
    pointerArea = session.find.css("#pointerArea", all=False)
    center = get_inview_center(pointerArea.rect, get_viewport_rect(session))
    touch_chain.pointer_move(0, 0, origin=pointerArea) \
        .pointer_down(width=23, height=31, pressure=0.78, tilt_x=21, tilt_y=-8, twist=355) \
        .pointer_move(10, 10, origin=pointerArea, width=39, height=35, pressure=0.91, tilt_x=-19, tilt_y=62, twist=345) \
        .pointer_up() \
        .pointer_move(80, 50, origin=pointerArea) \
        .perform()
    events = get_events(session)
    assert len(events) == 7
    event_types = [e["type"] for e in events]
    assert ["pointerover", "pointerenter", "pointerdown", "pointermove",
            "pointerup", "pointerout", "pointerleave"] == event_types
    assert events[2]["type"] == "pointerdown"
    assert events[2]["pageX"] == pytest.approx(center["x"], abs=1.0)
    assert events[2]["pageY"] == pytest.approx(center["y"], abs=1.0)
    assert events[2]["target"] == "pointerArea"
    assert events[2]["pointerType"] == "touch"
    assert round(events[2]["width"], 2) == 23
    assert round(events[2]["height"], 2) == 31
    assert round(events[2]["pressure"], 2) == 0.78
    assert events[3]["type"] == "pointermove"
    assert events[3]["pageX"] == pytest.approx(center["x"]+10, abs=1.0)
    assert events[3]["pageY"] == pytest.approx(center["y"]+10, abs=1.0)
    assert events[3]["target"] == "pointerArea"
    assert events[3]["pointerType"] == "touch"
    assert round(events[3]["width"], 2) == 39
    assert round(events[3]["height"], 2) == 35
    assert round(events[3]["pressure"], 2) == 0.91


def test_touch_pointer_properties_tilt_twist(session, test_actions_pointer_page, touch_chain):
    # This test only covers the tilt/twist properties which are
    # more specific to pen-type pointers, but which the spec allows
    # for generic touch pointers. Seperating this out gives better
    # coverage of the basic properties in test_touch_pointer_properties
    pointerArea = session.find.css("#pointerArea", all=False)
    center = get_inview_center(pointerArea.rect, get_viewport_rect(session))
    touch_chain.pointer_move(0, 0, origin=pointerArea) \
        .pointer_down(width=23, height=31, pressure=0.78, tilt_x=21, tilt_y=-8, twist=355) \
        .pointer_move(10, 10, origin=pointerArea, width=39, height=35, pressure=0.91, tilt_x=-19, tilt_y=62, twist=345) \
        .pointer_up() \
        .pointer_move(80, 50, origin=pointerArea) \
        .perform()
    events = get_events(session)
    assert len(events) == 7
    event_types = [e["type"] for e in events]
    assert ["pointerover", "pointerenter", "pointerdown", "pointermove",
            "pointerup", "pointerout", "pointerleave"] == event_types
    assert events[2]["type"] == "pointerdown"
    assert events[2]["tiltX"] == 21
    assert events[2]["tiltY"] == -8
    assert events[2]["twist"] == 355
    assert events[3]["type"] == "pointermove"
    assert events[3]["tiltX"] == -19
    assert events[3]["tiltY"] == 62
    assert events[3]["twist"] == 345
