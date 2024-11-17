import pytest
import webdriver.bidi.error as error
from webdriver.bidi.modules.input import Actions, get_element_origin

from . import get_element_rect
from .. import get_events

pytestmark = pytest.mark.asyncio

PAGE_CONTENT = """
    <input></input>
    <script>
        "use strict;"

        var allEvents = { events: [] };

        const input = document.querySelector("input");
        input.focus();

        window.addEventListener("keydown", e => allEvents.events.push([e.key]));
        window.addEventListener("mousemove", e => {
            allEvents.events.push([
                e.clientX,
                e.clientY,
            ]);
        });
    </script>
"""


async def test_key(bidi_session, inline, top_context, get_element):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(f"""
            <input onkeydown="window.location = '{inline(PAGE_CONTENT)}'"/>
            <script>
                const input = document.querySelector("input");
                input.focus();
            </script>
            """),
        wait="complete"
    )
    input = await get_element("input")

    actions = Actions()
    (
        actions.add_key()
        .key_down("1")
        .key_up("1")
        .pause(1000)
        .key_down("2")
        .key_up("2")
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    with pytest.raises(error.NoSuchNodeException):
        await get_element_rect(bidi_session, context=top_context, element=input)

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0] == ["2"]


async def test_pointer(bidi_session, inline, top_context, get_element):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=inline(
            f"""<input onmousedown="window.location = '{inline(PAGE_CONTENT)}'"/>"""),
        wait="complete"
    )
    input = await get_element("input")

    actions = Actions()
    (
        actions.add_pointer()
        .pointer_move(x=0, y=0, origin=get_element_origin(input))
        .pointer_down(button=0)
        .pointer_up(button=0)
        .pause(1000)
        .pointer_move(x=200, y=200)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    with pytest.raises(error.NoSuchNodeException):
        await get_element_rect(bidi_session, context=top_context, element=input)

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1
    assert events[0] == [200, 200]
