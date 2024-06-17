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
        user_context=user_context, type_hint=type_hint
    )

    contexts = await bidi_session.browsing_context.get_tree(max_depth=0)
    assert len(contexts) == 2

    assert_browsing_context(
        contexts[1],
        new_context["context"],
        children=None,
        parent_expected=True,
        parent=None,
        url="about:blank",
        user_context=user_context,
    )


async def test_user_context_default(bidi_session, create_user_context):
    user_context = await create_user_context()

    # Create a browsing context with userContext set to "default"
    context_1 = await bidi_session.browsing_context.create(
        type_hint="tab", user_context="default"
    )
    context_tree_1 = await bidi_session.browsing_context.get_tree(
        max_depth=0, root=context_1["context"]
    )
    assert_browsing_context(
        context_tree_1[0],
        context_1["context"],
        url="about:blank",
        user_context="default",
    )

    # Create a browsing context with no userContext parameter
    context_2 = await bidi_session.browsing_context.create(
        type_hint="tab",
    )
    context_tree_2 = await bidi_session.browsing_context.get_tree(
        max_depth=0, root=context_2["context"]
    )
    assert_browsing_context(
        context_tree_2[0],
        context_2["context"],
        url="about:blank",
        user_context="default",
    )


async def test_overrides_user_context_from_reference_context(
    bidi_session, create_user_context
):
    user_context_1 = await create_user_context()
    user_context_2 = await create_user_context()

    reference_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_1
    )
    reference_context_info = await bidi_session.browsing_context.get_tree(
        max_depth=0, root=reference_context["context"]
    )
    assert reference_context_info[0]["userContext"] == user_context_1

    new_context = await bidi_session.browsing_context.create(
        reference_context=reference_context["context"],
        type_hint="tab",
        user_context=user_context_2,
    )
    new_context_info = await bidi_session.browsing_context.get_tree(
        max_depth=0, root=new_context["context"]
    )
    assert new_context_info[0]["userContext"] == user_context_2


async def test_user_context_nested_iframes(
    bidi_session, create_user_context, new_tab, test_page_nested_frames
):
    user_context = await create_user_context()

    new_context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Navigate the user context tab to a page with iframes.
    await bidi_session.browsing_context.navigate(
        context=new_context["context"], url=test_page_nested_frames, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_context["context"])

    assert len(contexts) == 1
    root_info = contexts[0]

    # Check that iframes have the same user context as the parent.
    assert len(root_info["children"]) == 1
    child1_info = root_info["children"][0]
    assert child1_info["userContext"] == user_context
    assert len(child1_info["children"]) == 1
    child2_info = child1_info["children"][0]
    assert child2_info["userContext"] == user_context
