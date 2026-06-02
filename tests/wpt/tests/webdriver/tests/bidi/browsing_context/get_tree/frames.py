import pytest

from tests.bidi import wait_for_bidi_events
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
        client_window=top_context["clientWindow"],
    )

    child1_info = root_info["children"][0]
    assert_browsing_context(
        child1_info,
        context=None,
        children=0,
        parent_expected=False,
        parent=None,
        url=test_page,
        client_window=top_context["clientWindow"],
    )
    assert child1_info["context"] != root_info["context"]

    child2_info = root_info["children"][1]
    assert_browsing_context(
        child2_info,
        context=None,
        children=0,
        parent_expected=False,
        parent=None,
        url=test_page2,
        client_window=top_context["clientWindow"],
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
        client_window=top_context["clientWindow"],
    )

    child1_info = root_info["children"][0]
    assert_browsing_context(
        child1_info,
        context=None,
        children=0,
        parent_expected=False,
        parent=None,
        url=test_page_cross_origin,
        client_window=top_context["clientWindow"],
    )
    assert child1_info["context"] != root_info["context"]


@pytest.mark.parametrize("user_context", ["default", "new"])
@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
async def test_user_context(
    bidi_session,
    create_user_context,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    inline,
    user_context,
    domain,
):
    await subscribe_events(["browsingContext.contextCreated", "browsingContext.load"])

    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )

    iframe_url_1 = inline("<div>foo</div>", domain=domain)
    iframe_url_2 = inline("<div>bar</div>", domain=domain)
    page_url = inline(
        f"<iframe src='{iframe_url_1}'></iframe><iframe src='{iframe_url_2}'></iframe>"
    )

    on_context_created = wait_for_event("browsingContext.contextCreated")
    context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )
    context_info = await wait_for_future_safe(on_context_created)

    # Record all load events.
    events = []
    async def on_event(method, data):
        events.append(data)
    remove_listener = bidi_session.add_event_listener("browsingContext.load", on_event)

    await bidi_session.browsing_context.navigate(
        context=context["context"], url=page_url, wait="complete"
    )

    # Wait until all iframes have been loaded.
    await wait_for_bidi_events(bidi_session, events, 3)

    top_level_context_id = context["context"]
    all_contexts = await bidi_session.browsing_context.get_tree(
        root=top_level_context_id
    )

    assert len(all_contexts) == 1
    root_info = all_contexts[0]
    assert_browsing_context(
        root_info,
        top_level_context_id,
        children=2,
        parent=None,
        url=page_url,
        user_context=user_context_id,
        client_window=context_info["clientWindow"],
    )

    # The contexts can be returned in any order, find the info matching iframe_url_1
    child1_info = next(
        filter(lambda x: x["url"] == iframe_url_1, root_info["children"]), None
    )
    assert child1_info is not None

    assert_browsing_context(
        child1_info,
        context=None,
        children=0,
        parent_expected=False,
        parent=None,
        url=iframe_url_1,
        user_context=user_context_id,
        client_window=context_info["clientWindow"],
    )
    assert child1_info["context"] != root_info["context"]

    child2_info = next(
        filter(lambda x: x["url"] == iframe_url_2, root_info["children"]), None
    )
    assert child2_info is not None

    assert_browsing_context(
        child2_info,
        context=None,
        children=0,
        parent_expected=False,
        parent=None,
        url=iframe_url_2,
        user_context=user_context_id,
        client_window=context_info["clientWindow"],
    )
    assert child2_info["context"] != root_info["context"]
    assert child2_info["context"] != child1_info["context"]
