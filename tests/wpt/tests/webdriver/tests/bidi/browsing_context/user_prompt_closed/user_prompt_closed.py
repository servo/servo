import asyncio
import pytest
from tests.support.sync import AsyncPoll
from webdriver.error import TimeoutException

pytestmark = pytest.mark.asyncio

USER_PROMPT_CLOSED_EVENT = "browsingContext.userPromptClosed"
USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


async def test_unsubscribe(bidi_session, inline, new_tab, wait_for_event):
    await bidi_session.session.subscribe(
        events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT]
    )
    await bidi_session.session.unsubscribe(events=[USER_PROMPT_CLOSED_EVENT])

    on_entry = wait_for_event("browsingContext.userPromptOpened")

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<script>window.alert('test')</script>"),
    )

    # Wait for the alert to open
    await on_entry

    # Track all received browsingContext.userPromptClosed events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        USER_PROMPT_CLOSED_EVENT, on_event
    )

    await bidi_session.browsing_context.handle_user_prompt(context=new_tab["context"])

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


async def test_subscribe_with_alert(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event
):
    await subscribe_events(events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<script>window.alert('test')</script>"),
    )

    # Wait for the prompt to open.
    await on_prompt_opened

    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    await bidi_session.browsing_context.handle_user_prompt(context=new_tab["context"])

    event = await on_prompt_closed

    assert event == {"context": new_tab["context"], "accepted": True}


@pytest.mark.parametrize("accept", [True, False])
async def test_subscribe_with_confirm(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event, accept
):
    await subscribe_events(events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<script>window.confirm('test')</script>"),
    )

    # Wait for the prompt to open.
    await on_prompt_opened

    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept
    )

    event = await on_prompt_closed

    assert event == {"context": new_tab["context"], "accepted": accept}


@pytest.mark.parametrize("accept", [True, False])
async def test_subscribe_with_prompt(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event, accept
):
    await subscribe_events(events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<script>window.prompt('Enter Your Name: ')</script>"),
    )

    # Wait for the prompt to open.
    await on_prompt_opened

    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    test_user_text = "Test"
    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept, user_text=test_user_text
    )

    event = await on_prompt_closed

    if accept is True:
        assert event == {
            "context": new_tab["context"],
            "accepted": accept,
            "userText": test_user_text,
        }
    else:
        assert event == {"context": new_tab["context"], "accepted": accept}


async def test_subscribe_with_prompt_with_defaults(
    bidi_session, subscribe_events, inline, new_tab, wait_for_event
):
    await subscribe_events(events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<script>window.prompt('Enter Your Name: ')</script>"),
    )

    # Wait for the prompt to open.
    await on_prompt_opened

    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"]
    )

    event = await on_prompt_closed

    assert event == {"context": new_tab["context"], "accepted": True}


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_subscribe_to_one_context(
    bidi_session, subscribe_events, inline, wait_for_event, type_hint
):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    # Subscribe to open events for all contexts.
    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])

    # Subscribe to close events for only one context.
    await subscribe_events(
        events=[USER_PROMPT_CLOSED_EVENT],
        contexts=[new_context["context"]],
    )
    # Track all received browsingContext.userPromptClosed events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        USER_PROMPT_CLOSED_EVENT, on_event
    )

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    another_new_context = await bidi_session.browsing_context.create(
        type_hint=type_hint
    )

    # Open a prompt in the different context.
    await bidi_session.browsing_context.navigate(
        context=another_new_context["context"],
        url=inline(f"<script>window.alert('second tab')</script>"),
    )

    await on_prompt_opened

    await bidi_session.browsing_context.handle_user_prompt(
        context=another_new_context["context"]
    )

    # Make sure we don't receive this event.
    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)
    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    # Open a prompt in the subscribed context.
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline(f"<script>window.alert('first tab')</script>"),
    )

    await on_prompt_opened
    await bidi_session.browsing_context.handle_user_prompt(
        context=new_context["context"]
    )

    event = await on_prompt_closed

    assert event == {
        "context": new_context["context"],
        "accepted": True,
    }

    remove_listener()
    await bidi_session.browsing_context.close(context=new_context["context"])
    await bidi_session.browsing_context.close(context=another_new_context["context"])


async def test_iframe(
    bidi_session,
    new_tab,
    inline,
    test_origin,
    subscribe_events,
    wait_for_event,
):
    await subscribe_events(events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)
    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(f"<iframe src='{test_origin}'>"),
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    assert len(contexts) == 1

    assert len(contexts[0]["children"]) == 1
    frame = contexts[0]["children"][0]

    await bidi_session.browsing_context.navigate(
        context=frame["context"],
        url=inline("<script>window.alert('in iframe')</script>"),
    )

    await on_prompt_opened

    await bidi_session.browsing_context.handle_user_prompt(
        context=frame["context"]
    )

    event = await on_prompt_closed

    assert event == {"context": new_tab["context"], "accepted": True}
