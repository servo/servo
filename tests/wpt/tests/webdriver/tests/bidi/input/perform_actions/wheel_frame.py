import pytest

from webdriver.bidi.modules.input import Actions, get_element_origin
from webdriver.bidi.modules.script import ContextTarget

from .. import wait_for_events
from . import assert_scroll_position

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("iframe_domain", [None, "alt"], ids=["same-origin", "cross-origin"])
async def test_scroll_on_not_scrollable_element_in_iframe(
    bidi_session, new_tab, test_actions_wheel_page, get_element, iframe_domain
):
    page_kwargs = {"iframe_domain": iframe_domain} if iframe_domain else {}
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_actions_wheel_page(**page_kwargs),
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"]
    )
    frame_context = all_contexts[0]["children"][0]

    target = await get_element("#inner-not-scrollable", context=frame_context)

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=60, delta_y=85, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=frame_context["context"]
    )

    scroller = await bidi_session.script.evaluate(
        expression="document.scrollingElement",
        target=ContextTarget(frame_context["context"]),
        await_promise=False,
    )

    await assert_scroll_position(bidi_session, frame_context, scroller, 60, 85)


@pytest.mark.parametrize("iframe_domain", [None, "alt"], ids=["same-origin", "cross-origin"])
async def test_scroll_on_scrollable_element_in_iframe(
    bidi_session, new_tab, test_actions_wheel_page, get_element, iframe_domain
):
    page_kwargs = {"iframe_domain": iframe_domain} if iframe_domain else {}
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_actions_wheel_page(**page_kwargs),
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"]
    )
    frame_context = all_contexts[0]["children"][0]

    target = await get_element("#inner-scrollable", context=frame_context)

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=65, delta_y=90, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=frame_context["context"]
    )

    await assert_scroll_position(bidi_session, frame_context, target, 65, 90)


@pytest.mark.parametrize("iframe_domain", [None, "alt"], ids=["same-origin", "cross-origin"])
async def test_wheel_event_in_iframe(
    bidi_session, new_tab, test_actions_wheel_page, get_element, iframe_domain
):
    page_kwargs = {"iframe_domain": iframe_domain} if iframe_domain else {}
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_actions_wheel_page(events=["wheel", "scroll"], **page_kwargs),
        wait="complete",
    )

    all_contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"]
    )
    frame_context = all_contexts[0]["children"][0]

    target = await get_element("#inner-scrollable", context=frame_context)

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=55, delta_y=80, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=frame_context["context"]
    )

    events = await wait_for_events(bidi_session, new_tab["context"], 2)

    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 55
    assert events[0]["deltaY"] == 80
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "inner-scrollable-content"

    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "inner-scrollable"
