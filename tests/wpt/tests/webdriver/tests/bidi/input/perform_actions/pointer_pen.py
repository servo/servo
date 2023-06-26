import pytest

from webdriver.bidi.modules.input import Actions, get_element_origin

from .. import get_events
from . import get_inview_center_bidi

pytestmark = pytest.mark.asyncio


async def test_pen_pointer_properties(
    bidi_session, top_context, get_element, load_static_test_page
):
    await load_static_test_page(page="test_actions_pointer.html")

    pointerArea = await get_element("#pointerArea")
    center = await get_inview_center_bidi(
        bidi_session, context=top_context, element=pointerArea
    )

    actions = Actions()
    (
        actions.add_pointer(pointer_type="pen")
        .pointer_move(x=0, y=0, origin=get_element_origin(pointerArea))
        .pointer_down(button=0, pressure=0.36, tilt_x=-72, tilt_y=9, twist=86)
        .pointer_move(x=10, y=10, origin=get_element_origin(pointerArea))
        .pointer_up(button=0)
        .pointer_move(x=80, y=50, origin=get_element_origin(pointerArea))
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 10
    event_types = [e["type"] for e in events]
    assert [
        "pointerover",
        "pointerenter",
        "pointermove",
        "pointerdown",
        "pointerover",
        "pointerenter",
        "pointermove",
        "pointerup",
        "pointerout",
        "pointerleave",
    ] == event_types
    assert events[3]["type"] == "pointerdown"
    assert events[3]["pageX"] == pytest.approx(center["x"], abs=1.0)
    assert events[3]["pageY"] == pytest.approx(center["y"], abs=1.0)
    assert events[3]["target"] == "pointerArea"
    assert events[3]["pointerType"] == "pen"
    # The default value of width and height for mouse and pen inputs is 1
    assert round(events[3]["width"], 2) == 1
    assert round(events[3]["height"], 2) == 1
    assert round(events[3]["pressure"], 2) == 0.36
    assert events[3]["tiltX"] == -72
    assert events[3]["tiltY"] == 9
    assert events[3]["twist"] == 86
    assert events[6]["type"] == "pointermove"
    assert events[6]["pageX"] == pytest.approx(center["x"] + 10, abs=1.0)
    assert events[6]["pageY"] == pytest.approx(center["y"] + 10, abs=1.0)
    assert events[6]["target"] == "pointerArea"
    assert events[6]["pointerType"] == "pen"
    assert round(events[6]["width"], 2) == 1
    assert round(events[6]["height"], 2) == 1
    # The default value of pressure for all inputs is 0.5, other properties are 0
    assert round(events[6]["pressure"], 2) == 0.5
    assert events[6]["tiltX"] == 0
    assert events[6]["tiltY"] == 0
    assert events[6]["twist"] == 0
