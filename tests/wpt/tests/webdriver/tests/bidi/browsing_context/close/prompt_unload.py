import pytest
import asyncio

from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

CONTEXT_DESTROYED_EVENT = "browsingContext.contextDestroyed"
USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest.mark.parametrize("type_hint", ["window", "tab"])
@pytest.mark.parametrize("prompt_unload", [None, False])
async def test_prompt_unload_not_triggering_dialog(
    bidi_session,
    subscribe_events,
    setup_beforeunload_page,
    wait_for_event,
    wait_for_future_safe,
    type_hint,
    prompt_unload,
):

    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    # Set up event listener to make sure the "browsingContext.userPromptOpened" event is not emitted
    await subscribe_events([USER_PROMPT_OPENED_EVENT, CONTEXT_DESTROYED_EVENT])
    # Track all received browsingContext.userPromptOpened events in the events array
    events = []

    async def on_event(method, data):
        if method == USER_PROMPT_OPENED_EVENT:
            events.append(data)

    remove_listener = bidi_session.add_event_listener(
        USER_PROMPT_OPENED_EVENT, on_event
    )

    await setup_beforeunload_page(new_context)

    on_context_destroyed = wait_for_event(CONTEXT_DESTROYED_EVENT)

    await bidi_session.browsing_context.close(
        context=new_context["context"], prompt_unload=prompt_unload
    )

    await wait_for_future_safe(on_context_destroyed)

    assert events == []

    remove_listener()


@pytest.mark.capabilities({"unhandledPromptBehavior": {'beforeUnload': 'ignore'}})
@pytest.mark.parametrize("type_hint", ["window", "tab"])
async def test_prompt_unload_triggering_dialog(
    bidi_session,
    setup_beforeunload_page,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    type_hint,
):

    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    # Set up event listener to make sure the "browsingContext.contextDestroyed" event is not emitted
    await subscribe_events([USER_PROMPT_OPENED_EVENT, CONTEXT_DESTROYED_EVENT])
    user_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    # Track all received browsingContext.contextDestroyed events in the events array
    events = []

    async def on_event(_, data):
        if data["type"] == CONTEXT_DESTROYED_EVENT:
            events.append(data)

    remove_listener = bidi_session.add_event_listener(
        CONTEXT_DESTROYED_EVENT, on_event)

    await setup_beforeunload_page(new_context)

    close_task = asyncio.create_task(
        bidi_session.browsing_context.close(
            context=new_context["context"], prompt_unload=True
        )
    )

    await wait_for_future_safe(user_prompt_opened)

    # Events that come after the handling are OK
    remove_listener()
    assert events == []

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_context["context"],
    )

    await close_task

    contexts = await bidi_session.browsing_context.get_tree()
    assert len(contexts) == 1

    assert contexts[0]["context"] != new_context["context"]
