import pytest

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_null(bidi_session, top_context, test_page, type_hint):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page, wait="complete"
    )

    current_top_level_context_id = top_context["context"]
    other_top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    other_top_level_context_id = other_top_level_context["context"]

    # Retrieve all top-level browsing contexts
    contexts = await bidi_session.browsing_context.get_tree(root=None)

    assert len(contexts) == 2
    if contexts[0]["context"] == current_top_level_context_id:
        current_info = contexts[0]
        other_info = contexts[1]
    else:
        current_info = contexts[1]
        other_info = contexts[0]

    assert_browsing_context(
        current_info,
        current_top_level_context_id,
        children=0,
        parent=None,
        url=test_page,
    )

    assert_browsing_context(
        other_info,
        other_top_level_context_id,
        children=0,
        parent=None,
        url="about:blank",
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_top_level_context(bidi_session, top_context, test_page, type_hint):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page, wait="complete"
    )

    other_top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    other_top_level_context_id = other_top_level_context["context"]
    # Retrieve all browsing contexts of the newly opened tab/window
    contexts = await bidi_session.browsing_context.get_tree(root=other_top_level_context_id)

    assert len(contexts) == 1
    assert_browsing_context(
        contexts[0],
        other_top_level_context_id,
        children=0,
        parent=None,
        url="about:blank",
    )


async def test_child_context(
    bidi_session,
    top_context,
    test_page_same_origin_frame,
    test_page_nested_frames,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=test_page_nested_frames, wait="complete"
    )

    # First retrieve all browsing contexts for the first tab
    top_level_context_id = top_context["context"]
    all_contexts = await bidi_session.browsing_context.get_tree(root=top_level_context_id)

    assert len(all_contexts) == 1
    root_info = all_contexts[0]
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
        is_root=False,
        parent=None,
        url=test_page_same_origin_frame,
    )

    # Now retrieve all browsing contexts for the first browsing context child
    child_contexts = await bidi_session.browsing_context.get_tree(root=child1_info["context"])

    assert len(child_contexts) == 1
    assert_browsing_context(
        child_contexts[0],
        root_info["children"][0]["context"],
        children=1,
        parent=root_info["context"],
        url=test_page_same_origin_frame,
    )

    assert child1_info["children"][0] == child_contexts[0]["children"][0]
