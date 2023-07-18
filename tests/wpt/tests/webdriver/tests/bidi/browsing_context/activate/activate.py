import pytest

from webdriver.bidi.modules.script import ContextTarget
from . import get_visibility_state, is_selector_focused

@pytest.mark.asyncio
async def test_activate(bidi_session, new_tab):
    assert await get_visibility_state(bidi_session, new_tab) == 'hidden'

    await bidi_session.browsing_context.activate(context=new_tab["context"])

    assert await get_visibility_state(bidi_session, new_tab) == 'visible'


@pytest.mark.asyncio
async def test_deactivates_other_contexts(bidi_session, new_tab, top_context):
    await bidi_session.browsing_context.activate(context=top_context["context"])

    assert await get_visibility_state(bidi_session, top_context) == 'visible'
    assert await get_visibility_state(bidi_session, new_tab) == 'hidden'

    await bidi_session.browsing_context.activate(context=new_tab["context"])

    assert await get_visibility_state(bidi_session, top_context) == 'hidden'
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'


@pytest.mark.asyncio
async def test_keeps_focused_area(bidi_session, inline, new_tab, top_context):
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'

    await bidi_session.browsing_context.navigate(context=new_tab["context"],
                                                 url=inline("<textarea autofocus></textarea><input>"),
                                                 wait="complete")

    await bidi_session.script.evaluate(
        expression="""document.querySelector("input").focus()""",
        target=ContextTarget(new_tab["context"]),
        await_promise=False)

    assert await is_selector_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=top_context["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'hidden'
    assert await is_selector_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await is_selector_focused(bidi_session, new_tab, "input")


@pytest.mark.asyncio
async def test_double_activation(bidi_session, inline, new_tab, top_context):
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'

    await bidi_session.browsing_context.navigate(context=new_tab["context"],
                                                 url=inline("<input><script>document.querySelector('input').focus();</script>"),
                                                 wait="complete")
    assert await is_selector_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await is_selector_focused(bidi_session, new_tab, "input")

    # Activate again.
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await is_selector_focused(bidi_session, new_tab, "input")
