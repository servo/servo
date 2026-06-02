import pytest

from webdriver.bidi.error import MoveTargetOutOfBoundsException, NoSuchFrameException
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from tests.support.keys import Keys
from tests.support.sync import AsyncPoll
from .. import get_events, wait_for_events
from . import assert_events, get_inview_center_bidi, get_shadow_root_from_test_page

pytestmark = pytest.mark.asyncio


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
async def test_scroll_on_not_scrollable_element(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    target = await get_element("#not-scrollable")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=delta_x, delta_y=delta_y, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "not-scrollable-content"


@parametrize_deltas
async def test_scroll_on_element_with_overflow_scroll(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    scrollable = await get_element("#scrollable")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0,
        y=0,
        delta_x=delta_x,
        delta_y=delta_y,
        origin=get_element_origin(scrollable),
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"


@parametrize_deltas
async def test_scroll_on_iframe_with_overflow_scroll(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    target = await get_element("#iframe")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=delta_x, delta_y=delta_y, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    # Chrome requires some time (~10-20ms) to process the event from the iframe, so we wait for it.
    events = await wait_for_events(bidi_session, top_context["context"], 1, timeout=0.5, interval=0.02)

    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "iframeContent"


@parametrize_deltas
async def test_scroll_element_in_iframe_with_overflow_scroll(
    bidi_session, get_element, setup_wheel_test, top_context, delta_x, delta_y
):
    all_contexts = await bidi_session.browsing_context.get_tree(
        root=top_context["context"]
    )
    frame_context = all_contexts[0]["children"][0]

    target = await get_element("div", context=frame_context)

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=delta_x, delta_y=delta_y, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=frame_context["context"]
    )

    events = await wait_for_events(
        bidi_session, top_context["context"], 1, timeout=0.5, interval=0.02
    )
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_x
    assert events[0]["deltaY"] == delta_y
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "iframeContent"


@parametrize_deltas
@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
async def test_scroll_element_in_shadow_tree(
    bidi_session, new_tab, get_test_page, mode, nested, delta_x, delta_y
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=get_test_page(
            shadow_doc="""
            <div id="scrollableShadowTree"
                 style="width: 100px; height: 100px; overflow: auto;">
                <div
                    id="scrollableShadowTreeContent"
                    style="width: 600px; height: 1000px; background-color:blue"></div>
            </div>""",
            shadow_root_mode=mode,
            nested_shadow_dom=nested,
        ),
        wait="complete",
    )

    shadow_root = await get_shadow_root_from_test_page(bidi_session, new_tab, nested)

    # Add a simplified event recorder to track events in the test ShadowRoot.
    scrollable = await bidi_session.script.call_function(
        function_declaration="""shadowRoot => {
            window.allEvents = { events: [] };

            const scrollable = shadowRoot.querySelector("#scrollableShadowTree");
            scrollable.addEventListener("wheel", event => {
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

            scrollable.addEventListener("scroll", event => {
                window.allEvents.events.push({
                    type: event.type,
                    target: event.target.id || event.target.localName || event.target.documentElement?.localName,
                });
            });

            return scrollable;
        }
        """,
        arguments=[shadow_root],
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    center = await get_inview_center_bidi(
        bidi_session, context=new_tab, element=scrollable
    )

    actions = Actions()
    actions.add_wheel().scroll(
        x=0,
        y=0,
        delta_x=delta_x,
        delta_y=delta_y,
        origin=get_element_origin(scrollable),
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=new_tab["context"]
    )

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

    events = await wait_for_events(bidi_session, new_tab["context"], 2, timeout=0.5, interval=0.02)
    assert_events(events, expected_events)


async def test_scroll_with_key_pressed(
    bidi_session, setup_wheel_test, top_context, get_element
):
    scrollable = await get_element("#scrollable")

    actions = Actions()
    actions.add_key().key_down(Keys.R_SHIFT)
    actions.add_wheel().scroll(
        x=0,
        y=0,
        delta_x=5,
        delta_y=10,
        origin=get_element_origin(scrollable),
    )
    actions.add_key().key_up(Keys.R_SHIFT)

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["shiftKey"] is True


async def test_scroll_more_than_a_page(
    bidi_session, get_element, setup_wheel_test, top_context
):
    scrollable = await get_element("#scrollable")

    delta_huge = 3000

    actions = Actions()
    actions.add_wheel().scroll(
        x=0,
        y=0,
        delta_x=delta_huge,
        delta_y=delta_huge,
        origin=get_element_origin(scrollable),
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_huge
    assert events[0]["deltaY"] == delta_huge
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"
