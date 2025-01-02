import pytest

from webdriver.error import (
    MoveTargetOutOfBoundsException,
    NoSuchWindowException,
    StaleElementReferenceException,
)

from tests.classic.perform_actions.support.mouse import (
    get_inview_center,
    get_viewport_rect,
)
from tests.classic.perform_actions.support.refine import get_events

from . import assert_pointer_events, record_pointer_events


def test_null_response_value(session, pen_chain):
    value = pen_chain.click().perform()
    assert value is None


def test_no_top_browsing_context(session, closed_window, pen_chain):
    with pytest.raises(NoSuchWindowException):
        pen_chain.click().perform()


def test_no_browsing_context(session, closed_frame, pen_chain):
    with pytest.raises(NoSuchWindowException):
        pen_chain.click().perform()


def test_pointer_down_closes_browsing_context(
    session, configuration, http_new_tab, inline, pen_chain
):
    session.url = inline(
        """<input onpointerdown="window.close()">close</input>""")
    origin = session.find.css("input", all=False)

    with pytest.raises(NoSuchWindowException):
        pen_chain.pointer_move(0, 0, origin=origin) \
            .pointer_down(button=0) \
            .pause(100 * configuration["timeout_multiplier"]) \
            .pointer_up(button=0) \
            .perform()


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, pen_chain, as_frame):
    element = stale_element("input#text", as_frame=as_frame)

    with pytest.raises(StaleElementReferenceException):
        pen_chain.click(element=element).perform()


@pytest.mark.parametrize("origin", ["element", "pointer", "viewport"])
def test_params_actions_origin_outside_viewport(session, test_actions_page, pen_chain, origin):
    if origin == "element":
        origin = session.find.css("#outer", all=False)

    with pytest.raises(MoveTargetOutOfBoundsException):
        pen_chain.pointer_move(-100, -100, origin=origin).perform()


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
def test_pen_pointer_in_shadow_tree(
    session, get_test_page, pen_chain, mode, nested
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

    pen_chain.pointer_move(0, 0, origin=target) \
        .pointer_down() \
        .pointer_up() \
        .perform()

    assert_pointer_events(
        session,
        expected_events=["pointerdown", "pointerup"],
        target="pointer-target",
        pointer_type="pen",
    )


def test_pen_pointer_properties(session, test_actions_pointer_page, pen_chain):
    pointerArea = session.find.css("#pointerArea", all=False)
    center = get_inview_center(pointerArea.rect, get_viewport_rect(session))
    pen_chain.pointer_move(0, 0, origin=pointerArea) \
        .pointer_down(pressure=0.36, altitude_angle=0.3, azimuth_angle=0.2419, twist=86) \
        .pointer_move(10, 10, origin=pointerArea) \
        .pointer_up() \
        .pointer_move(80, 50, origin=pointerArea) \
        .perform()
    events = get_events(session)
    assert len(events) == 10
    event_types = [e["type"] for e in events]
    assert ["pointerover", "pointerenter", "pointermove", "pointerdown",
            "pointerover", "pointerenter", "pointermove", "pointerup",
            "pointerout", "pointerleave"] == event_types
    assert events[3]["type"] == "pointerdown"
    assert events[3]["pageX"] == pytest.approx(center["x"], abs=1.0)
    assert events[3]["pageY"] == pytest.approx(center["y"], abs=1.0)
    assert events[3]["target"] == "pointerArea"
    assert events[3]["pointerType"] == "pen"
    # The default value of width and height for mouse and pen inputs is 1
    assert round(events[3]["width"], 2) == 1
    assert round(events[3]["height"], 2) == 1
    assert round(events[3]["pressure"], 2) == 0.36
    assert events[3]["tiltX"] == 72
    assert events[3]["tiltY"] == 38
    assert events[3]["twist"] == 86
    assert events[6]["type"] == "pointermove"
    assert events[6]["pageX"] == pytest.approx(center["x"]+10, abs=1.0)
    assert events[6]["pageY"] == pytest.approx(center["y"]+10, abs=1.0)
    assert events[6]["target"] == "pointerArea"
    assert events[6]["pointerType"] == "pen"
    assert round(events[6]["width"], 2) == 1
    assert round(events[6]["height"], 2) == 1
    # The default value of pressure for all inputs is 0.5, other properties are 0
    assert round(events[6]["pressure"], 2) == 0.5
    assert events[6]["tiltX"] == 0
    assert events[6]["tiltY"] == 0
    assert events[6]["twist"] == 0
