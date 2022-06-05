import pytest

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", ["tab", "window"])
async def test_type(bidi_session, current_session, wait_for_event, value):
    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 1

    new_context_id = await bidi_session.browsing_context.create(type_hint=value)
    assert contexts[0]["context"] != new_context_id

    # Check there is an additional browsing context
    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 2

    # Retrieve the new context info
    contexts = await bidi_session.browsing_context.get_tree(
        max_depth=0, root=new_context_id
    )

    assert_browsing_context(
        contexts[0],
        new_context_id,
        children=None,
        is_root=True,
        parent=None,
        url="about:blank",
    )

    # Fallback to WebDriver HTTP to check that the new context has no opener.
    initial_window = current_session.window_handle
    current_session.window_handle = new_context_id

    try:
        opener = current_session.execute_script("return !!window.opener;")
        assert opener is False
    finally:
        # Restore the current window for WebDriver HTTP before closing the window.
        current_session.window_handle = initial_window

    await bidi_session.browsing_context.close(context=new_context_id)
