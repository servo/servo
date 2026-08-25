import pytest

from .. import assert_browsing_context

pytestmark = pytest.mark.asyncio

CONTEXT_CREATED_EVENT = "browsingContext.contextCreated"

def find_event_by_context(context, events):
    return next(
        (data for method, data in events if data["context"] == context["context"]), None
    )

def find_client_window_by_context(context, contexts):
    return next(
        (found_context["clientWindow"] for found_context in contexts if found_context["context"] == context["context"]), None
    )

@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_existing_context(bidi_session, wait_for_event, wait_for_future_safe, subscribe_events, type_hint):
    # See https://w3c.github.io/webdriver-bidi/#ref-for-remote-end-subscribe-steps%E2%91%A1.
    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)
    await subscribe_events([CONTEXT_CREATED_EVENT], contexts=[top_level_context["context"]])
    context_info = await wait_for_future_safe(on_entry)
    contexts = await bidi_session.browsing_context.get_tree(root=top_level_context["context"])

    assert_browsing_context(
        context_info,
        top_level_context["context"],
        url="about:blank",
        user_context="default",
        client_window=contexts[0]["clientWindow"],
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_existing_context_via_user_context(bidi_session, create_user_context, wait_for_event, wait_for_future_safe, subscribe_events, type_hint):
    user_context = await create_user_context()
    # See https://w3c.github.io/webdriver-bidi/#ref-for-remote-end-subscribe-steps%E2%91%A1.
    top_level_context = await bidi_session.browsing_context.create(type_hint=type_hint, user_context=user_context)

    on_entry = wait_for_event(CONTEXT_CREATED_EVENT)
    await subscribe_events([CONTEXT_CREATED_EVENT], user_contexts=[user_context])
    context_info = await wait_for_future_safe(on_entry)
    contexts = await bidi_session.browsing_context.get_tree(root=top_level_context["context"])

    assert_browsing_context(
        context_info,
        top_level_context["context"],
        url="about:blank",
        user_context=user_context,
        client_window=contexts[0]["clientWindow"],
    )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_existing_nested_contexts(bidi_session, wait_for_events, test_page_nested_frames, subscribe_events, type_hint):
    second_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.navigate(
        context=second_context["context"], url=test_page_nested_frames, wait="complete"
    )

    with wait_for_events([CONTEXT_CREATED_EVENT]) as waiter:
        await subscribe_events([CONTEXT_CREATED_EVENT], contexts=[second_context["context"]])

        events = await waiter.get_events(lambda events : len(events) >= 1)
        assert len(events) == 3

        contexts = await bidi_session.browsing_context.get_tree()

        assert_browsing_context(
            find_event_by_context(second_context, events),
            second_context["context"],
            url=test_page_nested_frames,
            user_context="default",
            client_window=find_client_window_by_context(second_context, contexts),
        )

        assert_browsing_context(
            find_event_by_context(contexts[1]["children"][0], events),
            contexts[1]["children"][0]["context"],
            url=contexts[1]["children"][0]["url"],
            parent=second_context["context"],
            user_context="default",
            client_window=contexts[1]["children"][0]["clientWindow"],
        )

        assert_browsing_context(
            find_event_by_context(contexts[1]["children"][0]["children"][0], events),
            contexts[1]["children"][0]["children"][0]["context"],
            url=contexts[1]["children"][0]["children"][0]["url"],
            parent=contexts[1]["children"][0]["context"],
            user_context="default",
            client_window=contexts[1]["children"][0]["children"][0]["clientWindow"],
        )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context_event_not_subscribed(bidi_session, wait_for_events, test_page_nested_frames, subscribe_events, type_hint):
    top_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    second_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.navigate(
        context=second_context["context"], url=test_page_nested_frames, wait="complete"
    )

    with wait_for_events([CONTEXT_CREATED_EVENT]) as waiter:
        await subscribe_events(events=[CONTEXT_CREATED_EVENT], contexts=[top_context["context"]])

        events = await waiter.get_events(lambda events : len(events) >= 1)
        assert len(events) == 1

        first_context = await bidi_session.browsing_context.create(type_hint=type_hint)

        await subscribe_events(events=[CONTEXT_CREATED_EVENT], contexts=[first_context["context"]])
        events = await waiter.get_events(lambda events : len(events) >= 1)
        assert len(events) == 2

        contexts = await bidi_session.browsing_context.get_tree()

        assert_browsing_context(
            find_event_by_context(top_context, events),
            top_context["context"],
            url="about:blank",
            user_context="default",
            client_window=find_client_window_by_context(top_context, contexts),
        )

        assert_browsing_context(
            find_event_by_context(first_context, events),
            first_context["context"],
            url="about:blank",
            user_context="default",
            client_window=find_client_window_by_context(first_context, contexts),
        )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_new_context_event_cross_origin_frame(bidi_session, wait_for_events, test_page_nested_frames, test_page_cross_origin_frame, subscribe_events, type_hint):
    first_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await bidi_session.browsing_context.navigate(
      context=first_context["context"], url=test_page_cross_origin_frame, wait="complete"
    )

    with wait_for_events([CONTEXT_CREATED_EVENT]) as waiter:
        await subscribe_events(events=[CONTEXT_CREATED_EVENT], contexts=[first_context["context"]])

        events = await waiter.get_events(lambda events : len(events) >= 1)
        assert len(events) == 2

        contexts = await bidi_session.browsing_context.get_tree()

        assert_browsing_context(
            find_event_by_context(first_context, events),
            first_context["context"],
            url=test_page_cross_origin_frame,
            user_context="default",
            client_window=find_client_window_by_context(first_context, contexts),
        )

        assert_browsing_context(
            find_event_by_context(contexts[1]["children"][0], events),
            contexts[1]["children"][0]["context"],
            url=contexts[1]["children"][0]["url"],
            parent=first_context["context"],
            user_context="default",
            client_window=contexts[1]["children"][0]["clientWindow"],
        )
