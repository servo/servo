import asyncio
import pytest

import webdriver.bidi.error as error
from webdriver.bidi.modules.script import ContextTarget


pytestmark = pytest.mark.asyncio

USER_PROMPT_CLOSED_EVENT = "browsingContext.userPromptClosed"
USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'ignore'}})
async def test_alert(
    bidi_session, wait_for_event, wait_for_future_safe, new_tab, subscribe_events
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    # Save as the task to await for it later.
    task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="window.alert('test')",
            target=ContextTarget(new_tab["context"]),
            await_promise=False,
        )
    )

    # Wait for prompt to appear.
    await wait_for_future_safe(on_entry)

    await bidi_session.browsing_context.handle_user_prompt(context=new_tab["context"])

    # Make sure that script returned.
    result = await task

    assert result == {"type": "undefined"}


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'ignore'}})
@pytest.mark.parametrize("accept", [True, False])
async def test_confirm(
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
    new_tab,
    subscribe_events,
    accept,
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    # Save as the task to await for it later.
    task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="window.confirm('test')",
            target=ContextTarget(new_tab["context"]),
            await_promise=False,
        )
    )

    # Wait for prompt to appear.
    await wait_for_future_safe(on_entry)

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept
    )

    # Check that return result of confirm is correct.
    result = await task

    assert result == {"type": "boolean", "value": accept}


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'ignore'}})
@pytest.mark.parametrize("accept", [True, False])
async def test_prompt(
    bidi_session,
    wait_for_event,
    wait_for_future_safe,
    new_tab,
    subscribe_events,
    accept,
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    # Save as the task to await for it later.
    task = asyncio.create_task(
        bidi_session.script.evaluate(
            expression="window.prompt('Enter Your Name: ')",
            target=ContextTarget(new_tab["context"]),
            await_promise=False,
        )
    )

    # Wait for prompt to appear.
    await wait_for_future_safe(on_entry)

    test_user_text = "Test"
    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept, user_text=test_user_text
    )

    # Check that return result of prompt is correct.
    result = await task

    if accept is True:
        assert result == {"type": "string", "value": test_user_text}
    else:
        assert result == {"type": "null"}


@pytest.mark.capabilities({"unhandledPromptBehavior": {'beforeUnload': 'ignore'}})
@pytest.mark.parametrize("accept", [True, False])
async def test_beforeunload(
    bidi_session,
    subscribe_events,
    url,
    new_tab,
    setup_beforeunload_page,
    wait_for_event,
    wait_for_future_safe,
    accept,
):
    await subscribe_events(events=[USER_PROMPT_CLOSED_EVENT, USER_PROMPT_OPENED_EVENT])

    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await setup_beforeunload_page(new_tab)

    page_target = url("/webdriver/tests/support/html/default.html")
    navigated_future = await bidi_session.send_command(
        "browsingContext.navigate",
        {
            "context": new_tab["context"],
            "url": page_target,
            "wait": "complete",
        },
    )

    # Wait for the prompt to open.
    await wait_for_future_safe(on_entry)

    on_prompt_closed = wait_for_event(USER_PROMPT_CLOSED_EVENT)

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept
    )

    # Wait for the prompt to be closed.
    await wait_for_future_safe(on_prompt_closed)

    if accept:
        # Check navigation to the target page.
        navigated = await wait_for_future_safe(navigated_future)
        assert navigated["url"] == page_target
    else:
        # If the beforeunload prompt was dismissed, the navigation is canceled.
        # Step 22.2 of the html spec 7.4.2.2 Beginning navigation.
        # https://html.spec.whatwg.org/multipage/browsing-the-web.html#beginning-navigation
        with pytest.raises(error.UnknownErrorException):
            await wait_for_future_safe(navigated_future)

        contexts = await bidi_session.browsing_context.get_tree(
            root=new_tab["context"], max_depth=0
        )
        assert contexts[0]["url"] != page_target


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'ignore'}})
@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_two_top_level_contexts(
    bidi_session,
    new_tab,
    inline,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    type_hint,
):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=inline("<script>window.alert('test')</script>"),
    )

    # Wait for prompt to appear.
    await wait_for_future_safe(on_entry)

    # Try to close the prompt in another context.
    with pytest.raises(error.NoSuchAlertException):
        await bidi_session.browsing_context.handle_user_prompt(
            context=new_tab["context"]
        )

    # Close the prompt in the correct context
    await bidi_session.browsing_context.handle_user_prompt(
        context=new_context["context"]
    )

    await bidi_session.browsing_context.close(context=new_context["context"])


@pytest.mark.capabilities({"unhandledPromptBehavior": {'default': 'ignore'}})
async def test_multiple_frames(
    bidi_session,
    new_tab,
    inline,
    test_page_multiple_frames,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    await subscribe_events([USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_page_multiple_frames,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
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
    await wait_for_future_safe(on_entry)

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
