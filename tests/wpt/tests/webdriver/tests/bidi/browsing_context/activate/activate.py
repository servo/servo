import pytest

from webdriver.bidi.modules.script import ContextTarget
from . import is_element_focused
from .. import assert_document_status

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_switch_between_contexts(bidi_session, top_context, type_hint):
    is_window = type_hint == "window"

    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.activate(context=top_context["context"])
    await assert_document_status(bidi_session, top_context, visible=True, focused=True)
    await assert_document_status(bidi_session, new_context, visible=is_window, focused=False)

    await bidi_session.browsing_context.activate(context=new_context["context"])
    await assert_document_status(bidi_session, top_context, visible=is_window, focused=False)
    await assert_document_status(bidi_session, new_context, visible=True, focused=True)


async def test_keeps_element_focused(bidi_session, inline, new_tab, top_context):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<textarea autofocus></textarea><input>"),
        wait="complete")

    await bidi_session.script.evaluate(
        expression="""document.querySelector("input").focus()""",
        target=ContextTarget(new_tab["context"]),
        await_promise=False)

    assert await is_element_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=top_context["context"])
    assert await is_element_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=new_tab["context"])
    assert await is_element_focused(bidi_session, new_tab, "input")


async def test_multiple_activation(bidi_session, inline, new_tab):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(
            "<input><script>document.querySelector('input').focus();</script>"),
        wait="complete")

    await assert_document_status(bidi_session, new_tab, visible=True, focused=True)
    assert await is_element_focused(bidi_session, new_tab, "input")

    await bidi_session.browsing_context.activate(context=new_tab["context"])
    await assert_document_status(bidi_session, new_tab, visible=True, focused=True)
    assert await is_element_focused(bidi_session, new_tab, "input")

    # Activate again.
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    await assert_document_status(bidi_session, new_tab, visible=True, focused=True)
    assert await is_element_focused(bidi_session, new_tab, "input")
