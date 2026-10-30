import asyncio
import pytest
import pytest_asyncio

from webdriver.bidi.error import NoSuchAlertException, UnknownErrorException
from webdriver.bidi.modules.script import ContextTarget
from ... import recursive_compare

pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest_asyncio.fixture
async def cleanup_open_prompts(
    bidi_session, wait_for_future_safe, create_user_context
):
    """Close ignored prompts and await pending prompt tasks before teardown.

    Tests in this file spawn unawaited background tasks (`script.evaluate` and
    `browsingContext.navigate`) that block while a prompt is open. We clean
    them up here rather than in shared fixtures because:
    - `bidi_session` teardown (`_dismiss_user_prompts`) runs after the BiDi
      WebSocket closes and only sees the default user context via WebDriver
      Classic.
    - `create_user_context` teardown calls `browser.removeUserContext`, which
      closes custom user context tabs (and should not dismiss prompts first, as
      that would mask `removeUserContext` bugs), but does not hold the tests'
      background `asyncio.Task` handles. If those tasks are left unawaited when
      a `beforeunload` is dismissed or the tab is detached, they leak or raise
      unretrieved `UnknownErrorException`s.
    - Depending on `create_user_context` ensures this teardown runs before the
      user context is removed.
    """
    prompts = []

    def register_prompt(context, handler, task):
        prompts.append((context, handler, task))

    yield register_prompt

    for context, handler, task in prompts:
        if handler == "ignore":
            try:
                await bidi_session.browsing_context.handle_user_prompt(
                    context=context, accept=True
                )
            except NoSuchAlertException:
                pass
        if not task.done():
            try:
                await wait_for_future_safe(task)
            except UnknownErrorException:
                pass


@pytest.fixture
def assert_simple_prompt(bidi_session, subscribe_events, wait_for_event,
        wait_for_future_safe, create_user_context, cleanup_open_prompts):
    async def assert_simple_prompt(unhandled_prompt_behavior, prompt, handler):
        user_context = await create_user_context(
            unhandled_prompt_behavior=unhandled_prompt_behavior)
        new_tab = await bidi_session.browsing_context.create(
            type_hint="tab",
            user_context=user_context
        )

        await subscribe_events([USER_PROMPT_OPENED_EVENT],
                               contexts=[new_tab["context"]])
        on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

        # Schedule script opening prompt without awaiting so we can wait for
        # browsingContext.userPromptOpened while the script is blocked.
        prompt_task = asyncio.create_task(
            bidi_session.script.evaluate(
                expression=f"window.{prompt}('some message')",
                target=ContextTarget(new_tab["context"]),
                await_promise=False,
            )
        )
        # Register for teardown cleanup immediately so the prompt and task are
        # cleaned up even if waiting for the event or asserting below fails.
        cleanup_open_prompts(new_tab["context"], handler, prompt_task)

        # Wait for prompt to appear.
        resp = await wait_for_future_safe(on_entry)
        recursive_compare({
            'context': new_tab["context"],
            'handler': handler,
            'type': prompt,
        }, resp)

    return assert_simple_prompt


@pytest.mark.parametrize("handler", ["accept", "dismiss", "ignore"])
@pytest.mark.parametrize("prompt", ["alert", "confirm", "prompt"])
async def test_simple_prompts(assert_simple_prompt, prompt, handler):
    await assert_simple_prompt({prompt: handler}, prompt, handler)


@pytest.mark.parametrize("handler", ["accept", "dismiss", "ignore"])
@pytest.mark.parametrize("prompt", ["alert", "confirm", "prompt"])
async def test_default_handler(assert_simple_prompt, prompt, handler):
    await assert_simple_prompt({"default": handler}, prompt, handler)


@pytest.mark.parametrize("handler", ["accept", "dismiss", "ignore"])
async def test_beforeunload(bidi_session, subscribe_events, wait_for_event,
        wait_for_future_safe, create_user_context, cleanup_open_prompts,
        handler, setup_beforeunload_page, inline):
    user_context = await create_user_context(
        unhandled_prompt_behavior={"beforeUnload": handler})
    new_tab = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context=user_context
    )

    await setup_beforeunload_page(new_tab)

    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

    url_after = inline("<div>foo</div>")

    # Schedule navigation without awaiting so we can wait for
    # browsingContext.userPromptOpened while beforeunload blocks the navigation.
    nav_task = asyncio.create_task(
        bidi_session.browsing_context.navigate(context=new_tab["context"],
                                               url=url_after, wait="none"))
    # Register for teardown cleanup immediately so the prompt and task are
    # cleaned up even if waiting for the event or asserting below fails.
    cleanup_open_prompts(new_tab["context"], handler, nav_task)

    # Wait for prompt to appear.
    resp = await wait_for_future_safe(on_entry)
    recursive_compare({
        'context': new_tab["context"],
        'handler': handler,
        'type': "beforeunload",
    }, resp)

    # Dismissing beforeunload cancels the navigation and rejects
    # browsingContext.navigate with unknown error.
    if handler == "dismiss":
        with pytest.raises(UnknownErrorException):
            await wait_for_future_safe(nav_task)


@pytest.mark.parametrize("handler", ["accept", "dismiss", "ignore"])
async def test_file(bidi_session, create_user_context, handler,
        assert_file_dialog_not_canceled, assert_file_dialog_canceled):
    user_context = await create_user_context(
        unhandled_prompt_behavior={"file": handler})

    new_tab = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context=user_context
    )

    # Unless explicitly set to `ignore`, the file behavior is `dismiss`.
    if handler == 'ignore':
        await assert_file_dialog_not_canceled(new_tab)
    else:
        await assert_file_dialog_canceled(new_tab)
