import pytest

from webdriver.bidi.modules.script import ContextTarget
from . import is_selector_focused
from .. import get_document_focus, get_visibility_state

pytestmark = pytest.mark.asyncio


async def test_activate(bidi_session, new_tab, top_context):
    assert await get_document_focus(bidi_session, top_context) is False

    await bidi_session.browsing_context.activate(context=top_context["context"])

    assert await get_visibility_state(bidi_session, top_context) == 'visible'
    assert await get_document_focus(bidi_session, top_context) is True


async def test_deactivates_other_contexts(bidi_session, new_tab, top_context):
    await bidi_session.browsing_context.activate(context=top_context["context"])

    assert await get_visibility_state(bidi_session, top_context) == 'visible'
    assert await get_document_focus(bidi_session, top_context) is True

    assert await get_document_focus(bidi_session, new_tab) is False

    await bidi_session.browsing_context.activate(context=new_tab["context"])

    assert await get_document_focus(bidi_session, top_context) is False

    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await get_document_focus(bidi_session, new_tab) is True


async def test_keeps_focused_area(bidi_session, inline, new_tab, top_context):
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await get_document_focus(bidi_session, new_tab) is True

    await bidi_session.browsing_context.navigate(context=new_tab["context"],
                                                 url=inline("<textarea autofocus></textarea><input>"),
                                                 wait="complete")

    await bidi_session.script.evaluate(
        expression="""document.querySelector("input").focus()""",
        target=ContextTarget(new_tab["context"]),
        await_promise=False)

    assert await is_selector_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=top_context["context"])
    assert await get_document_focus(bidi_session, new_tab) is False
    assert await is_selector_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await get_document_focus(bidi_session, new_tab) is True
    assert await is_selector_focused(bidi_session, new_tab, "input")


async def test_double_activation(bidi_session, inline, new_tab):
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await get_document_focus(bidi_session, new_tab) is True

    await bidi_session.browsing_context.navigate(context=new_tab["context"],
                                                 url=inline("<input><script>document.querySelector('input').focus();</script>"),
                                                 wait="complete")
    assert await is_selector_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await get_document_focus(bidi_session, new_tab) is True
    assert await is_selector_focused(bidi_session, new_tab, "input")

    # Activate again.
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await get_visibility_state(bidi_session, new_tab) == 'visible'
    assert await get_document_focus(bidi_session, new_tab) is True
    assert await is_selector_focused(bidi_session, new_tab, "input")


async def test_activate_window(bidi_session):
    new_window_1 = await bidi_session.browsing_context.create(type_hint="window")
    new_window_2 = await bidi_session.browsing_context.create(type_hint="window")

    assert await get_visibility_state(bidi_session, new_window_2) == 'visible'
    assert await get_document_focus(bidi_session, new_window_2) is True

    assert await get_document_focus(bidi_session, new_window_1) is False

    await bidi_session.browsing_context.activate(context=new_window_1["context"])

    assert await get_visibility_state(bidi_session, new_window_1) == 'visible'
    assert await get_document_focus(bidi_session, new_window_1) is True
