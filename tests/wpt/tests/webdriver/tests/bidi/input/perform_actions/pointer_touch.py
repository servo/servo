import pytest

from webdriver.bidi.modules.input import Actions, get_element_origin

from .. import get_events
from . import get_inview_center_bidi

pytestmark = pytest.mark.asyncio


async def test_touch_pointer_properties(
    bidi_session, top_context, get_element, load_static_test_page
):
    await load_static_test_page(page="test_actions_pointer.html")

    pointerArea = await get_element("#pointerArea")
    center = await get_inview_center_bidi(
        bidi_session, context=top_context, element=pointerArea
    )

    actions = Actions()
    (
        actions.add_pointer(pointer_type="touch")
        .pointer_move(x=0, y=0, origin=get_element_origin(pointerArea))
        .pointer_down(
            button=0,
            width=23,
            height=31,
            pressure=0.78,
            tilt_x=21,
            tilt_y=-8,
            twist=355,
        )
        .pointer_move(
            x=10,
            y=10,
            origin=get_element_origin(pointerArea),
            width=39,
            height=35,
            pressure=0.91,
            tilt_x=-19,
            tilt_y=62,
            twist=345,
        )
        .pointer_up(button=0)
        .pointer_move(x=80, y=50, origin=get_element_origin(pointerArea))
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])

    assert len(events) == 7
    event_types = [e["type"] for e in events]
    assert [
        "pointerover",
        "pointerenter",
        "pointerdown",
        "pointermove",
        "pointerup",
        "pointerout",
        "pointerleave",
    ] == event_types
    assert events[2]["type"] == "pointerdown"
    assert events[2]["pageX"] == pytest.approx(center["x"], abs=1.0)
    assert events[2]["pageY"] == pytest.approx(center["y"], abs=1.0)
    assert events[2]["target"] == "pointerArea"
    assert events[2]["pointerType"] == "touch"
    assert round(events[2]["width"], 2) == 23
    assert round(events[2]["height"], 2) == 31
    assert round(events[2]["pressure"], 2) == 0.78
    assert events[3]["type"] == "pointermove"
    assert events[3]["pageX"] == pytest.approx(center["x"] + 10, abs=1.0)
    assert events[3]["pageY"] == pytest.approx(center["y"] + 10, abs=1.0)
    assert events[3]["target"] == "pointerArea"
    assert events[3]["pointerType"] == "touch"
    assert round(events[3]["width"], 2) == 39
    assert round(events[3]["height"], 2) == 35
    assert round(events[3]["pressure"], 2) == 0.91


async def test_touch_pointer_properties_tilt_twist(
    bidi_session, top_context, get_element, load_static_test_page
):
    # This test only covers the tilt/twist properties which are
    # more specific to pen-type pointers, but which the spec allows
    # for generic touch pointers. Seperating this out gives better
    # coverage of the basic properties in test_touch_pointer_properties
    await load_static_test_page(page="test_actions_pointer.html")

    pointerArea = await get_element("#pointerArea")
    center = await get_inview_center_bidi(
        bidi_session, context=top_context, element=pointerArea
    )

    actions = Actions()
    (
        actions.add_pointer(pointer_type="touch")
        .pointer_move(x=0, y=0, origin=get_element_origin(pointerArea))
        .pointer_down(
            button=0,
            width=23,
            height=31,
            pressure=0.78,
            tilt_x=21,
            tilt_y=-8,
            twist=355,
        )
        .pointer_move(
            x=10,
            y=10,
            origin=get_element_origin(pointerArea),
            width=39,
            height=35,
            pressure=0.91,
            tilt_x=-19,
            tilt_y=62,
            twist=345,
        )
        .pointer_up(button=0)
        .pointer_move(x=80, y=50, origin=get_element_origin(pointerArea))
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])

    assert len(events) == 7
    event_types = [e["type"] for e in events]
    assert [
        "pointerover",
        "pointerenter",
        "pointerdown",
        "pointermove",
        "pointerup",
        "pointerout",
        "pointerleave",
    ] == event_types
    assert events[2]["type"] == "pointerdown"
    assert events[2]["tiltX"] == 21
    assert events[2]["tiltY"] == -8
    assert events[2]["twist"] == 355
    assert events[3]["type"] == "pointermove"
    assert events[3]["tiltX"] == -19
    assert events[3]["tiltY"] == 62
    assert events[3]["twist"] == 345
