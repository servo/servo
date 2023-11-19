import pytest

from webdriver.bidi.error import NoSuchFrameException
from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from .. import get_events, get_object_from_context
from . import get_shadow_root_from_test_page

pytestmark = pytest.mark.asyncio


async def test_invalid_browsing_context(bidi_session):
    actions = Actions()
    actions.add_wheel()

    with pytest.raises(NoSuchFrameException):
        await bidi_session.input.perform_actions(actions=actions, context="foo")


@pytest.mark.parametrize("delta_x, delta_y", [(0, 10), (5, 0), (5, 10)])
async def test_scroll_not_scrollable(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    actions = Actions()

    target = await get_element("#not-scrollable")
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


@pytest.mark.parametrize("delta_x, delta_y", [(0, 10), (5, 0), (5, 10)])
async def test_scroll_scrollable_overflow(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    actions = Actions()

    scrollable = await get_element("#scrollable")

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


@pytest.mark.parametrize("delta_x, delta_y", [(0, 10), (5, 0), (5, 10)])
async def test_scroll_iframe(
    bidi_session, setup_wheel_test, top_context, get_element, delta_x, delta_y
):
    actions = Actions()

    target = await get_element("#iframe")
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
    assert events[0]["target"] == "iframeContent"


@pytest.mark.parametrize("mode", ["open", "closed"])
@pytest.mark.parametrize("nested", [False, True], ids=["outer", "inner"])
async def test_scroll_shadow_tree(
    bidi_session, top_context, get_test_page, mode, nested
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
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

    shadow_root = await get_shadow_root_from_test_page(bidi_session, top_context, nested)

    # Add a simplified event recorder to track events in the test ShadowRoot.
    scrollable = await bidi_session.script.call_function(
        function_declaration="""shadowRoot => {
            window.wheelEvents = [];
            const scrollable = shadowRoot.querySelector("#scrollableShadowTree");
            scrollable.addEventListener("wheel",
                function(event) {
                    window.wheelEvents.push({
                        "deltaX": event.deltaX,
                        "deltaY": event.deltaY,
                        "target": event.target.id
                    });
                }
            );
            return scrollable;
        }
        """,
        arguments=[shadow_root],
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    actions = Actions()
    actions.add_wheel().scroll(
        x=0,
        y=0,
        delta_x=5,
        delta_y=10,
        origin=get_element_origin(scrollable),
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_object_from_context(
        bidi_session, top_context["context"], "window.wheelEvents"
    )

    assert len(events) == 1
    assert events[0]["deltaX"] >= 5
    assert events[0]["deltaY"] >= 10
    assert events[0]["target"] == "scrollableShadowTreeContent"
