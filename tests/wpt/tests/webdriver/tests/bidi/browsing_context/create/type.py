import pytest

from webdriver.bidi.modules.script import ContextTarget
from .. import assert_browsing_context, assert_document_status


pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_type(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, top_context, type_hint):
    await subscribe_events(["browsingContext.contextCreated"])
    is_window = type_hint == "window"

    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 1

    await assert_document_status(bidi_session, top_context, visible=True, focused=True)

    on_entry = wait_for_event("browsingContext.contextCreated")
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    context_info = await wait_for_future_safe(on_entry)
    assert contexts[0]["context"] != new_context["context"]

    await assert_document_status(bidi_session, new_context, visible=True, focused=True)
    await assert_document_status(bidi_session, top_context, visible=is_window, focused=False)

    # Check there is an additional browsing context
    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 2

    # Retrieve the new context info
    contexts = await bidi_session.browsing_context.get_tree(
        max_depth=0, root=new_context["context"]
    )

    assert_browsing_context(
        contexts[0],
        new_context["context"],
        children=None,
        parent_expected=True,
        parent=None,
        url="about:blank",
        client_window=context_info["clientWindow"],
    )

    opener_protocol_value = await bidi_session.script.evaluate(
        expression="!!window.opener",
        target=ContextTarget(new_context["context"]),
        await_promise=False)
    assert opener_protocol_value["value"] is False

    await bidi_session.browsing_context.close(context=new_context["context"])
