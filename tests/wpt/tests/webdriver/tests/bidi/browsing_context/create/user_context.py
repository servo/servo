import pytest

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_user_context(bidi_session, type_hint, create_user_context):
    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 1

    user_context = await create_user_context()

    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 1

    new_context = await bidi_session.browsing_context.create(
        user_context=user_context,
        type_hint=type_hint
    )

    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 2

    assert_browsing_context(
        contexts[1],
        new_context["context"],
        children=None,
        is_root=True,
        parent=None,
        url="about:blank",
        user_context=user_context
    )
