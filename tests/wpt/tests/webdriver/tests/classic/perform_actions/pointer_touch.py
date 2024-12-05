import pytest

from webdriver.error import (
    MoveTargetOutOfBoundsException,
    NoSuchWindowException,
    StaleElementReferenceException
)
from tests.classic.perform_actions.support.mouse import (
    get_inview_center,
    get_viewport_rect,
)
from tests.classic.perform_actions.support.refine import get_events

from . import assert_pointer_events, record_pointer_events


def test_null_response_value(session, touch_chain):
    value = touch_chain.click().perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, touch_chain):
    with pytest.raises(NoSuchWindowException):
        touch_chain.click().perform()


def test_no_browsing_context(session, closed_frame, touch_chain):
    with pytest.raises(NoSuchWindowException):
        touch_chain.click().perform()


def test_pointer_down_closes_browsing_context(
    session, configuration, http_new_tab, inline, touch_chain
):
    session.url = inline(
        """<input onpointerdown="window.close()">close</input>""")
    origin = session.find.css("input", all=False)

    with pytest.raises(NoSuchWindowException):
        touch_chain.pointer_move(0, 0, origin=origin) \
            .pointer_down(button=0) \
            .pause(100 * configuration["timeout_multiplier"]) \
            .pointer_up(button=0) \
            .perform()


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, touch_chain, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    with pytest.raises(StaleElementReferenceException):
        touch_chain.click(element=element).perform()


@pytest.mark.parametrize("origin", ["element", "pointer", "viewport"])
def test_params_actions_origin_outside_viewport(session, test_actions_page, touch_chain, origin):
    if origin == "element":
        origin = session.find.css("#outer", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        touch_chain.pointer_move(-100, -100, origin=origin).perform()


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
def test_touch_pointer_in_shadow_tree(
    session, get_test_page, touch_chain, mode, nested
):
    session.url = get_test_page(
        shadow_doc="""
        <div id="pointer-target"
             style="width: 10px; height: 10px; background-color:blue;">
        </div>""",
        shadow_root_mode=mode,
        nested_shadow_dom=nested,
    )

    shadow_root = session.find.css("custom-element", all=False).shadow_root

    if nested:
        shadow_root = shadow_root.find_element(
            "css selector", "inner-custom-element"
        ).shadow_root

    target = shadow_root.find_element("css selector", "#pointer-target")

    record_pointer_events(session, target)

    touch_chain.pointer_move(0, 0, origin=target).pointer_down().pointer_up().perform()

    assert_pointer_events(
        session,
        expected_events=["pointerdown", "pointerup"],
        target="pointer-target",
        pointer_type="touch",
    )


def test_touch_pointer_properties(session, test_actions_pointer_page, touch_chain):
    pointerArea = session.find.css("#pointerArea", all=False)
    center = get_inview_center(pointerArea.rect, get_viewport_rect(session))
    touch_chain.pointer_move(0, 0, origin=pointerArea) \
        .pointer_down(width=23, height=31, pressure=0.78, twist=355) \
        .pointer_move(10, 10, origin=pointerArea, width=39, height=35, pressure=0.91, twist=345) \
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


def test_touch_pointer_properties_angle_twist(session, test_actions_pointer_page, touch_chain):
    pointerArea = session.find.css("#pointerArea", all=False)
    touch_chain.pointer_move(0, 0, origin=pointerArea) \
        .pointer_down(width=23, height=31, pressure=0.78, altitude_angle=1.2, azimuth_angle=6, twist=355) \
        .pointer_move(10, 10, origin=pointerArea, width=39, height=35, pressure=0.91, altitude_angle=0.5, azimuth_angle=1.8, twist=345) \
        .pointer_up() \
        .pointer_move(80, 50, origin=pointerArea) \
        .perform()
    events = get_events(session)
    assert len(events) == 7
    event_types = [e["type"] for e in events]
    assert ["pointerover", "pointerenter", "pointerdown", "pointermove",
            "pointerup", "pointerout", "pointerleave"] == event_types
    assert events[2]["type"] == "pointerdown"
    assert events[2]["tiltX"] == 20
    assert events[2]["tiltY"] == -6
    assert events[2]["twist"] == 355
    assert events[3]["type"] == "pointermove"
    assert events[3]["tiltX"] == -23
    assert events[3]["tiltY"] == 61
    assert events[3]["twist"] == 345


def test_touch_pointer_properties_tilt_twist(session, test_actions_pointer_page, touch_chain):
    pointerArea = session.find.css("#pointerArea", all=False)
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
