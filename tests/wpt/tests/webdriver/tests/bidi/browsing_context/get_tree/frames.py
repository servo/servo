import pytest

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio


async def test_multiple_frames(
    bidi_session,
    top_context,
    test_page,
    test_page2,
    test_page_multiple_frames,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_multiple_frames, wait="complete"
    )

    # First retrieve all browsing contexts of the first tab
    top_level_context_id = top_context["context"]
    all_contexts = await bidi_session.browsing_context.get_tree(root=top_level_context_id)

    assert len(all_contexts) == 1
    root_info = all_contexts[0]
    assert_browsing_context(
        root_info,
        top_level_context_id,
        children=2,
        parent=None,
        url=test_page_multiple_frames,
    )

    child1_info = root_info["children"][0]
    assert_browsing_context(
        child1_info,
        context=None,
        children=0,
        is_root=False,
        parent=None,
        url=test_page,
    )
    assert child1_info["context"] != root_info["context"]

    child2_info = root_info["children"][1]
    assert_browsing_context(
        child2_info,
        context=None,
        children=0,
        is_root=False,
        parent=None,
        url=test_page2,
    )
    assert child2_info["context"] != root_info["context"]
    assert child2_info["context"] != child1_info["context"]


async def test_cross_origin(
    bidi_session,
    top_context,
    test_page_cross_origin,
    test_page_cross_origin_frame,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_cross_origin_frame, wait="complete"
    )

    # First retrieve all browsing contexts of the first tab
    top_level_context_id = top_context["context"]
    all_contexts = await bidi_session.browsing_context.get_tree(root=top_level_context_id)

    assert len(all_contexts) == 1
    root_info = all_contexts[0]
    assert_browsing_context(
        root_info,
        top_level_context_id,
        children=1,
        parent=None,
        url=test_page_cross_origin_frame,
    )

    child1_info = root_info["children"][0]
    assert_browsing_context(
        child1_info,
        context=None,
        children=0,
        is_root=False,
        parent=None,
        url=test_page_cross_origin,
    )
    assert child1_info["context"] != root_info["context"]


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(
    bidi_session,
    create_user_context,
    test_page,
    test_page2,
    test_page_multiple_frames,
    user_context
):
    user_context_id = await create_user_context() if user_context == "new" else user_context

    context = await bidi_session.browsing_context.create(type_hint="tab", user_context=user_context_id)

    await bidi_session.browsing_context.navigate(
        context=context["context"], url=test_page_multiple_frames, wait="complete"
    )

    top_level_context_id = context["context"]
    all_contexts = await bidi_session.browsing_context.get_tree(root=top_level_context_id)

    assert len(all_contexts) == 1
    root_info = all_contexts[0]
    assert_browsing_context(
        root_info,
        top_level_context_id,
        children=2,
        parent=None,
        url=test_page_multiple_frames,
        user_context=user_context_id,
    )

    child1_info = root_info["children"][0]
    assert_browsing_context(
        child1_info,
        context=None,
        children=0,
        is_root=False,
        parent=None,
        url=test_page,
        user_context=user_context_id,
    )
    assert child1_info["context"] != root_info["context"]

    child2_info = root_info["children"][1]
    assert_browsing_context(
        child2_info,
        context=None,
        children=0,
        is_root=False,
        parent=None,
        url=test_page2,
        user_context=user_context_id,
    )
    assert child2_info["context"] != root_info["context"]
    assert child2_info["context"] != child1_info["context"]
