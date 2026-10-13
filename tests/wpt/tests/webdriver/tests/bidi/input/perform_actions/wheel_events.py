import pytest

from webdriver.bidi.modules.input import Actions, get_element_origin

from tests.support.keys import Keys
from .. import get_events, wait_for_events

pytestmark = pytest.mark.asyncio


async def test_scroll_on_not_scrollable_element(
    bidi_session, top_context, test_actions_wheel_page, get_element
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_actions_wheel_page(events=["wheel"]),
        wait="complete",
    )

    target = await get_element("#not-scrollable")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=55, delta_y=70, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1

    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 55
    assert events[0]["deltaY"] == 70
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "not-scrollable-content"


async def test_scroll_on_element_with_overflow_scroll(
    bidi_session, top_context, test_actions_wheel_page, get_element
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_actions_wheel_page(events=["wheel", "scroll"]),
        wait="complete",
    )

    target = await get_element("#scrollable")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=60, delta_y=80, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await wait_for_events(bidi_session, top_context["context"], 2)
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == 60
    assert events[0]["deltaY"] == 80
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"

    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "scrollable"


async def test_scroll_emits_scrollend_event(
    bidi_session, top_context, test_actions_wheel_page, get_element
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_actions_wheel_page(events=["wheel", "scroll", "scrollend"]),
        wait="complete",
    )

    target = await get_element("#scrollable")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=65, delta_y=85, origin=get_element_origin(target)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await wait_for_events(bidi_session, top_context["context"], 3)

    assert events[0]["type"] == "wheel"
    assert events[0]["target"] == "scrollable-content"

    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "scrollable"

    assert events[2]["type"] == "scrollend"
    assert events[2]["target"] == "scrollable"


async def test_scroll_with_key_pressed(
    bidi_session, top_context, test_actions_wheel_page, get_element
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_actions_wheel_page(events=["wheel"]),
        wait="complete",
    )

    scrollable = await get_element("#scrollable")

    actions = Actions()
    actions.add_key().key_down(Keys.R_SHIFT)
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=50, delta_y=75,
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
    bidi_session, top_context, test_actions_wheel_page, get_element
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_actions_wheel_page(events=["wheel", "scroll"]),
        wait="complete",
    )

    delta_huge = 3000

    target = await get_element("#scrollable")

    actions = Actions()
    actions.add_wheel().scroll(
        x=0, y=0, delta_x=delta_huge, delta_y=delta_huge,
        origin=get_element_origin(target),
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    events = await wait_for_events(bidi_session, top_context["context"], 2)
    assert events[0]["type"] == "wheel"
    assert events[0]["deltaX"] == delta_huge
    assert events[0]["deltaY"] == delta_huge
    assert events[0]["deltaZ"] == 0
    assert events[0]["target"] == "scrollable-content"
    assert events[1]["type"] == "scroll"
    assert events[1]["target"] == "scrollable"
