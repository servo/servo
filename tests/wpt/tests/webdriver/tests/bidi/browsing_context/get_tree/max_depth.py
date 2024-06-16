import pytest

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [0, 2**53 - 1])
async def test_params_boundaries(bidi_session, value):
    await bidi_session.browsing_context.get_tree(max_depth=value)


async def test_null(
    bidi_session,
    top_context,
    test_page,
    test_page_same_origin_frame,
    test_page_nested_frames,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_nested_frames, wait="complete"
    )

    # Retrieve browsing contexts for first tab only
    top_level_context_id = top_context["context"]
    contexts = await bidi_session.browsing_context.get_tree(root=top_level_context_id)

    assert len(contexts) == 1
    root_info = contexts[0]
    assert_browsing_context(
        root_info,
        top_level_context_id,
        children=1,
        parent=None,
        url=test_page_nested_frames,
    )

    child1_info = root_info["children"][0]
    assert_browsing_context(
        child1_info,
        context=None,
        children=1,
        parent_expected=False,
        parent=None,
        url=test_page_same_origin_frame,
    )
    assert child1_info["context"] != root_info["context"]

    child2_info = child1_info["children"][0]
    assert_browsing_context(
        child2_info,
        context=None,
        children=0,
        parent_expected=False,
        parent=None,
        url=test_page,
    )
    assert child2_info["context"] != root_info["context"]
    assert child2_info["context"] != child1_info["context"]


async def test_top_level_only(bidi_session, top_context, test_page_nested_frames):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_nested_frames, wait="complete"
    )

    # Retrieve browsing contexts for first tab only
    top_level_context_id = top_context["context"]
    contexts = await bidi_session.browsing_context.get_tree(
        max_depth=0,
        root=top_level_context_id
    )

    assert len(contexts) == 1
    root_info = contexts[0]
    assert_browsing_context(
        root_info,
        top_level_context_id,
        children=None,
        parent=None,
        url=test_page_nested_frames,
    )


async def test_top_level_and_one_child(
    bidi_session,
    top_context,
    test_page_nested_frames,
    test_page_same_origin_frame,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_nested_frames, wait="complete"
    )

    # Retrieve browsing contexts for first tab only
    top_level_context_id = top_context["context"]
    contexts = await bidi_session.browsing_context.get_tree(
        max_depth=1,
        root=top_level_context_id
    )

    assert len(contexts) == 1
    root_info = contexts[0]
    assert_browsing_context(
        root_info,
        top_level_context_id,
        children=1,
        parent=None,
        url=test_page_nested_frames,
    )

    child1_info = root_info["children"][0]
    assert_browsing_context(
        child1_info,
        context=None,
        children=None,
        parent_expected=False,
        parent=None,
        url=test_page_same_origin_frame,
    )
    assert child1_info["context"] != root_info["context"]
