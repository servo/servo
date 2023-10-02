import asyncio
import pytest

import webdriver.bidi.error as error
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


async def test_alert(bidi_session, wait_for_event, top_context, subscribe_events):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    # Save as the task to await for it later.
    task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="window.alert('test')",
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )
    )

    # Wait for prompt to appear.
    await on_entry

    await bidi_session.browsing_context.handle_user_prompt(
        context=top_context["context"]
    )

    # Make sure that script returned.
    result = await task

    assert result == {"type": "undefined"}


@pytest.mark.parametrize("accept", [True, False])
async def test_confirm(
    bidi_session, wait_for_event, top_context, subscribe_events, accept
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    # Save as the task to await for it later.
    task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="window.confirm('test')",
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )
    )

    # Wait for prompt to appear.
    await on_entry

    await bidi_session.browsing_context.handle_user_prompt(
        context=top_context["context"], accept=accept
    )

    # Check that return result of confirm is correct.
    result = await task

    assert result == {"type": "boolean", "value": accept}


@pytest.mark.parametrize("accept", [True, False])
async def test_prompt(
    bidi_session, wait_for_event, top_context, subscribe_events, accept
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    # Save as the task to await for it later.
    task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="window.prompt('Enter Your Name: ')",
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )
    )

    # Wait for prompt to appear.
    await on_entry

    test_user_text = "Test"
    await bidi_session.browsing_context.handle_user_prompt(
        context=top_context["context"], accept=accept, user_text=test_user_text
    )

    # Check that return result of prompt is correct.
    result = await task

    if accept is True:
        assert result == {"type": "string", "value": test_user_text}
    else:
        assert result == {"type": "null"}


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_two_top_level_contexts(
    bidi_session, top_context, inline, subscribe_events, wait_for_event, type_hint
):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline("<script>window.alert('test')</script>"),
    )

    # Wait for prompt to appear.
    await on_entry

    # Try to close the prompt in another context.
    with pytest.raises(error.NoSuchAlertException):
        await bidi_session.browsing_context.handle_user_prompt(
            context=top_context["context"]
        )

    # Close the prompt in the correct context
    await bidi_session.browsing_context.handle_user_prompt(
        context=new_context["context"]
    )

    await bidi_session.browsing_context.close(context=new_context["context"])


async def test_multiple_frames(
    bidi_session,
    top_context,
    inline,
    test_page_multiple_frames,
    subscribe_events,
    wait_for_event,
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_page_multiple_frames,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    assert len(contexts) == 1

    assert len(contexts[0]["children"]) == 2
    frame_1 = contexts[0]["children"][0]
    frame_2 = contexts[0]["children"][1]

    # Open a prompt in the first frame
    await bidi_session.browsing_context.navigate(
        context=frame_1["context"],
        url=inline("<script>window.response = window.confirm('test')</script>"),
    )

    # Wait for prompt to appear.
    await on_entry

    # Close prompt from the second frame.
    await bidi_session.browsing_context.handle_user_prompt(
        context=frame_2["context"], accept=True
    )

    # Check that return result of confirm is correct.
    result = await bidi_session.script.evaluate(
        expression="window.response",
        target=ContextTarget(frame_1["context"]),
        await_promise=False,
    )

    assert result == {"type": "boolean", "value": True}
