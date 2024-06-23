import pytest

import webdriver.bidi.error as error
from webdriver.error import TimeoutException

from tests.support.sync import AsyncPoll
from .. import get_user_context_ids

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest.mark.asyncio
async def test_remove_context(bidi_session, create_user_context):
    user_context = await create_user_context()
    assert user_context in await get_user_context_ids(bidi_session)

    await bidi_session.browser.remove_user_context(user_context=user_context)
    assert user_context not in await get_user_context_ids(bidi_session)
    assert "default" in await get_user_context_ids(bidi_session)


@pytest.mark.parametrize("type_hint", ["tab", "window"])
@pytest.mark.asyncio
async def test_remove_context_closes_contexts(
    bidi_session, subscribe_events, create_user_context, type_hint
):
    await subscribe_events(events=["browsingContext.contextDestroyed"])

    user_context_1 = await create_user_context()
    user_context_2 = await create_user_context()

    # context 1 and 2 are owned by user context 1
    context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint=type_hint
    )
    context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint=type_hint
    )
    # context 3 and 4 are owned by user context 2
    context_3 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint=type_hint
    )
    context_4 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint=type_hint
    )

    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("browsingContext.contextDestroyed", on_event)

    # destroy user context 1 and wait for context 1 and 2 to be destroyed
    await bidi_session.browser.remove_user_context(user_context=user_context_1)

    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)

    assert len(events) == 2
    destroyed_contexts = [event["context"] for event in events]
    assert context_1["context"] in destroyed_contexts
    assert context_2["context"] in destroyed_contexts

    # destroy user context 1 and wait for context 3 and 4 to be destroyed
    await bidi_session.browser.remove_user_context(user_context=user_context_2)

    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)

    assert len(events) == 4
    destroyed_contexts = [event["context"] for event in events]
    assert context_3["context"] in destroyed_contexts
    assert context_4["context"] in destroyed_contexts

    remove_listener()


@pytest.mark.parametrize("type_hint", ["tab", "window"])
@pytest.mark.asyncio
async def test_remove_context_skips_beforeunload_prompt(
    bidi_session,
    subscribe_events,
    create_user_context,
    setup_beforeunload_page,
    type_hint,
):
    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])

    events = []

    async def on_event(method, data):
        if data["type"] == "beforeunload":
            events.append(method)

    remove_listener = bidi_session.add_event_listener(
        USER_PROMPT_OPENED_EVENT, on_event)

    user_context = await create_user_context()

    context = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint=type_hint
    )

    await setup_beforeunload_page(context)

    await bidi_session.browser.remove_user_context(user_context=user_context)

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()
