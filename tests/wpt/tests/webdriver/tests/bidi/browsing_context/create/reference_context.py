import pytest

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", ["tab", "window"])
async def test_reference_context(bidi_session, value):
    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 1

    reference_context = await bidi_session.browsing_context.create(type_hint="tab")
    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 2

    new_context = await bidi_session.browsing_context.create(
        reference_context=reference_context["context"], type_hint=value
    )
    assert contexts[0]["context"] != new_context["context"]
    assert contexts[0]["context"] != new_context["context"]

    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 3

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
    )

    # We can not assert the specific behavior of reference_context here,
    # so we only verify that a new browsing context was successfully created
    # when a valid reference_context is provided.

    await bidi_session.browsing_context.close(context=reference_context["context"])
    await bidi_session.browsing_context.close(context=new_context["context"])


@pytest.mark.parametrize("value", ["tab", "window"])
async def test_reference_context_with_no_user_context_set(
    bidi_session, value, create_user_context
):
    user_context = await create_user_context()

    reference_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )
    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)

    new_context = await bidi_session.browsing_context.create(
        reference_context=reference_context["context"], type_hint=value
    )
    new_context_info = await bidi_session.browsing_context.get_tree(
        max_depth=0, root=new_context["context"]
    )

    assert new_context_info[0]["userContext"] == user_context
