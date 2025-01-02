import pytest
from tests.support.sync import AsyncPoll
from webdriver.error import TimeoutException


pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


async def test_unsubscribe(bidi_session, inline, new_tab):
    await bidi_session.session.subscribe(events=[USER_PROMPT_OPENED_EVENT])
    await bidi_session.session.unsubscribe(events=[USER_PROMPT_OPENED_EVENT])

    # Track all received browsingContext.userPromptOpened events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        USER_PROMPT_OPENED_EVENT, on_event
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline("<script>window.alert('test')</script>"),
    )

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


@pytest.mark.parametrize("prompt_type", ["alert", "confirm", "prompt"])
async def test_prompt_type(
    bidi_session,
    subscribe_events,
    inline,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    prompt_type,
):
    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    text = "test"

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(f"<script>window.{prompt_type}('{text}')</script>"),
    )

    event = await wait_for_future_safe(on_entry)

    assert event == {
        "context": new_tab["context"],
        "type": prompt_type,
        "message": text,
        "handler": "dismiss",
        **({"defaultValue": ""} if prompt_type == "prompt" else {}),
    }


@pytest.mark.parametrize(
    "default",
    [None, "", "default"],
    ids=["undefined", "empty string", "non empty string"],
)
async def test_prompt_default_value(
    bidi_session,
    inline,
    new_tab,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    default,
):
    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    text = "test"

    if default is None:
        script = f"<script>window.prompt('{text}')</script>"
    else:
        script = f"<script>window.prompt('{text}', '{default}')</script>"

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=inline(script),
    )

    event = await wait_for_future_safe(on_entry)

    assert event == {
        "context": new_tab["context"],
        "type": "prompt",
        "message": text,
        "handler": "dismiss",
        "defaultValue": default if default is not None else ""
    }


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_subscribe_to_one_context(
    bidi_session,
    subscribe_events,
    inline,
    wait_for_event,
    wait_for_future_safe,
    type_hint,
):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    await subscribe_events(
        events=[USER_PROMPT_OPENED_EVENT], contexts=[new_context["context"]]
    )
    # Track all received browsingContext.userPromptOpened events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        USER_PROMPT_OPENED_EVENT, on_event
    )

    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    another_new_context = await bidi_session.browsing_context.create(
        type_hint=type_hint
    )

    # Open a prompt in the different context.
    await bidi_session.browsing_context.navigate(
        context=another_new_context["context"],
        url=inline("<script>window.alert('second tab')</script>"),
    )

    # Make sure we don't receive this event.
    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    # Open a prompt in the subscribed context.
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline("<script>window.alert('first tab')</script>"),
    )

    event = await wait_for_future_safe(on_entry)

    assert event == {
        "context": new_context["context"],
        "type": "alert",
        "handler": "dismiss",
        "message": "first tab",
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
    wait_for_future_safe,
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

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

    event = await wait_for_future_safe(on_entry)

    assert event == {
        "context": new_tab["context"],
        "type": "alert",
        "handler": "dismiss",
        "message": "in iframe",
    }


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_two_prompts(
    bidi_session,
    subscribe_events,
    inline,
    wait_for_event,
    wait_for_future_safe,
    type_hint,
):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    await subscribe_events(
        events=[USER_PROMPT_OPENED_EVENT]
    )
    # Track all received browsingContext.userPromptOpened events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(
        USER_PROMPT_OPENED_EVENT, on_event
    )

    on_first_event = wait_for_event(USER_PROMPT_OPENED_EVENT)

    another_new_context = await bidi_session.browsing_context.create(
        type_hint=type_hint
    )

    # Open a prompt in the first context.
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline("<script>window.alert('first tab')</script>"),
    )

    await wait_for_future_safe(on_first_event)

    # Open a prompt in the second context.
    await bidi_session.browsing_context.navigate(
        context=another_new_context["context"],
        url=inline("<script>window.confirm('second tab')</script>"),
    )

    on_second_event = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await wait_for_future_safe(on_second_event)

    assert len(events) == 2

    assert events == [{
        "context": new_context["context"],
        "type": "alert",
        "handler": "dismiss",
        "message": "first tab",
    }, {
        "context": another_new_context["context"],
        "type": "confirm",
        "handler": "dismiss",
        "message": "second tab",
    }]

    remove_listener()
    await bidi_session.browsing_context.close(context=new_context["context"])
    await bidi_session.browsing_context.close(context=another_new_context["context"])
