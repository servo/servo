import pytest
from webdriver.bidi.modules.input import Actions, get_element_origin

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


async def test_key(bidi_session, inline, top_context):
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

    # Check that the page was navigated
    info = await bidi_session.browsing_context.get_tree(max_depth=1)
    assert info[0]["url"] == inline(PAGE_CONTENT)

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
        .pointer_move(x=300, y=200)
    )

    await bidi_session.input.perform_actions(
        actions=actions, context=top_context["context"]
    )

    # Check that the page was navigated
    info = await bidi_session.browsing_context.get_tree(max_depth=1)
    assert info[0]["url"] == inline(PAGE_CONTENT)

    events = await get_events(bidi_session, top_context["context"])
    assert len(events) == 1

    assert events[0] == [
        pytest.approx(300, abs=1.0),
        pytest.approx(200, abs=1.0)
    ]
