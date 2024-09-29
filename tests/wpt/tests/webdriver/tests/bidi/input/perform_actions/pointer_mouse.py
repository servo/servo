import pytest

from webdriver.bidi.error import MoveTargetOutOfBoundsException
from webdriver.bidi.modules.input import Actions, get_element_origin

from tests.support.asserts import assert_move_to_coordinates
from tests.support.helpers import filter_dict

from .. import get_events
from . import (
    assert_pointer_events,
    get_inview_center_bidi,
    get_shadow_root_from_test_page,
    record_pointer_events,
)

pytestmark = pytest.mark.asyncio


async def test_click_at_coordinates(bidi_session, top_context, load_static_test_page):
    await load_static_test_page(page="test_actions.html")

    div_point = {
        "x": 82,
        "y": 187,
    }
    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=div_point["x"], y=div_point["y"], duration=1000)
        .pointer_down(button=0)
        .pointer_up(button=0)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])

    assert len(events) == 4
    assert_move_to_coordinates(div_point, "outer", events)

    for e in events:
        if e["type"] != "mousedown":
            assert e["buttons"] == 0
        assert e["button"] == 0

    expected = [
        {"type": "mousedown", "buttons": 1},
        {"type": "mouseup", "buttons": 0},
        {"type": "click", "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    assert expected == filtered_events[1:]


@pytest.mark.parametrize("origin", ["element", "pointer", "viewport"])
async def test_params_actions_origin_outside_viewport(
    bidi_session, top_context, get_actions_origin_page, get_element, origin
):
    if origin == "element":
        url = get_actions_origin_page(
            """width: 100px; height: 50px; background: green;
            position: relative; left: -200px; top: -100px;"""
        )
        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=url,
            wait="complete",
        )

        element = await get_element("#inner")
        origin = get_element_origin(element)

    actions = Actions()
    actions.add_pointer().pointer_move(x=-100, y=-100, origin=origin)

    with pytest.raises(MoveTargetOutOfBoundsException):
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )


async def test_context_menu_at_coordinates(
    bidi_session, top_context, load_static_test_page
):
    await load_static_test_page(page="test_actions.html")

    div_point = {
        "x": 82,
        "y": 187,
    }

    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=div_point["x"], y=div_point["y"])
        .pointer_down(button=2)
        .pointer_up(button=2)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 4

    expected = [
        {"type": "mousedown", "button": 2, "buttons": 2},
        {"type": "contextmenu", "button": 2, "buttons": 2},
    ]
    # Some browsers in some platforms may dispatch `contextmenu` event as a
    # a default action of `mouseup`.  In the case, `.buttons` of the event
    # should be 0.
    anotherExpected = [
        {"type": "mousedown", "button": 2, "buttons": 2},
        {"type": "contextmenu", "button": 2, "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    mousedown_contextmenu_events = [
        x for x in filtered_events if x["type"] in ["mousedown", "contextmenu"]
    ]
    assert mousedown_contextmenu_events in [expected, anotherExpected]


async def test_middle_click(bidi_session, top_context, load_static_test_page):
    await load_static_test_page(page="test_actions.html")

    div_point = {
        "x": 82,
        "y": 187,
    }

    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=div_point["x"], y=div_point["y"])
        .pointer_down(button=1)
        .pointer_up(button=1)
    )
    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 3

    expected = [
        {"type": "mousedown", "button": 1, "buttons": 4},
        {"type": "mouseup", "button": 1, "buttons": 0},
    ]
    filtered_events = [filter_dict(e, expected[0]) for e in events]
    mousedown_mouseup_events = [
        x for x in filtered_events if x["type"] in ["mousedown", "mouseup"]
    ]
    assert expected == mousedown_mouseup_events


async def test_click_element_center(
    bidi_session, top_context, get_element, load_static_test_page
):
    await load_static_test_page(page="test_actions.html")

    outer = await get_element("#outer")
    center = await get_inview_center_bidi(
        bidi_session, context=top_context, element=outer
    )

    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(outer))
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 4

    event_types = [e["type"] for e in events]
    assert ["mousemove", "mousedown", "mouseup", "click"] == event_types
    for e in events:
        if e["type"] != "mousemove":
            assert e["pageX"] == pytest.approx(center["x"], abs=1.0)
            assert e["pageY"] == pytest.approx(center["y"], abs=1.0)
            assert e["target"] == "outer"


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
async def test_click_element_in_shadow_tree(
    bidi_session, top_context, get_test_page, mode, nested
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(
            shadow_doc="""
            <div id="pointer-target"
                 style="width: 10px; height: 10px; background-color:blue;">
            </div>""",
            shadow_root_mode=mode,
            nested_shadow_dom=nested,
        ),
        wait="complete",
    )

    shadow_root = await get_shadow_root_from_test_page(
        bidi_session, top_context, nested
    )

    target = await record_pointer_events(
        bidi_session, top_context, shadow_root, "#pointer-target"
    )

    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(target))
        .pointer_down(button=0)
        .pointer_up(button=0)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    await assert_pointer_events(
        bidi_session,
        top_context,
        expected_events=["pointerdown", "pointerup"],
        target="pointer-target",
        pointer_type="mouse",
    )


async def test_click_navigation(
    bidi_session,
    top_context,
    url,
    inline,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    get_element,
):
    await subscribe_events(events=["browsingContext.load"])

    destination = url("/webdriver/tests/support/html/test_actions.html")
    start = inline(f'<a href="{destination}" id="link">destination</a>')

    async def click_link():
        link = await get_element("#link")

        actions = Actions()
        (
            actions.add_pointer()
            .pointer_move(x=0, y=0, origin=get_element_origin(link))
            .pointer_down(button=0)
            .pointer_up(button=0)
        )
        await bidi_session.input.perform_actions(
            actions=actions, context=top_context["context"]
        )

    # repeat steps to check behaviour after document unload
    for _ in range(2):
        await bidi_session.browsing_context.navigate(
            context=top_context["context"], url=start, wait="complete"
        )

        on_entry = wait_for_event("browsingContext.load")
        await click_link()
        event = await wait_for_future_safe(on_entry)
        assert event["url"] == destination


@pytest.mark.parametrize("x, y, event_count", [
    (0, 0, 0),
    (1, 0, 1),
    (0, 1, 1),
], ids=["default value", "x", "y"])
async def test_move_to_position_in_viewport(
    bidi_session, load_static_test_page, top_context, x, y, event_count
):
    await load_static_test_page(page="test_actions.html")

    actions = Actions()
    actions.add_pointer().pointer_move(x=x, y=y)

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == event_count

    # Move again to check that no further mouse move event is emitted.
    actions = Actions()
    actions.add_pointer().pointer_move(x=x, y=y)

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == event_count


@pytest.mark.parametrize("origin", ["viewport", "pointer", "element"])
async def test_move_to_origin_position_within_frame(
    bidi_session, get_element, iframe, inline, top_context, origin
):
    url = inline(
        iframe(
            """
        <textarea style="width: 100px; height: 40px"></textarea>
        <script>
            "use strict;"

            var allEvents = { events: [] };
            window.addEventListener("mousemove", e => {
                allEvents.events.push([
                    e.clientX,
                    e.clientY,
                ]);
            });
        </script>
    """
        )
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    iframe = contexts[0]["children"][0]

    elem = await get_element("textarea", context=iframe)
    elem_center_point = await get_inview_center_bidi(
        bidi_session, context=iframe, element=elem
    )

    offset = [10, 5]

    if origin == "element":
        origin = get_element_origin(elem)
        target_point = [
            elem_center_point["x"] + offset[0],
            elem_center_point["y"] + offset[1],
        ]
    else:
        target_point = offset

    actions = Actions()
    actions.add_pointer().pointer_move(x=offset[0], y=offset[1], origin=origin)

    await bidi_session.input.perform_actions(actions=actions, context=iframe["context"])

    events = await get_events(bidi_session, iframe["context"])

    assert len(events) == 1
    assert events[0] == target_point
