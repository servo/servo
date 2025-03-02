import asyncio

import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest.mark.capabilities({"unhandledPromptBehavior": {'beforeUnload': 'ignore'}})
@pytest.mark.parametrize("value", ["none", "interactive", "complete"])
@pytest.mark.parametrize("accept", [True, False])
async def test_navigate_with_beforeunload_prompt(bidi_session, new_tab,
        setup_beforeunload_page, inline, subscribe_events, wait_for_event,
        wait_for_future_safe, value, accept):
    await setup_beforeunload_page(new_tab)

    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    url_after = inline("<div>foo</div>")

    navigated_future = asyncio.create_task(
        bidi_session.browsing_context.navigate(context=new_tab["context"],
                                               url=url_after, wait=value))

    # Wait for the prompt to open.
    await wait_for_future_safe(on_prompt_opened)
    # Make sure the navigation is not finished.
    assert not navigated_future.done(), "Navigation should not be finished before prompt is handled."

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept
    )

    if accept:
        await navigated_future
    else:
        with pytest.raises(error.UnknownErrorException):
            await wait_for_future_safe(navigated_future)


@pytest.mark.capabilities({"unhandledPromptBehavior": {'beforeUnload': 'ignore'}})
@pytest.mark.parametrize("value", ["none", "interactive", "complete"])
@pytest.mark.parametrize("accept", [True, False])
async def test_navigate_with_beforeunload_prompt_in_iframe(bidi_session,
        new_tab, setup_beforeunload_page, inline, subscribe_events,
        wait_for_event, wait_for_future_safe, value, accept):
    page = inline(f"""<iframe src={inline("foo")}></iframe>""")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]

    await setup_beforeunload_page(iframe_context)

    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    url_after = inline("<div>foo</div>")

    navigated_future = asyncio.create_task(
        bidi_session.browsing_context.navigate(
            context=iframe_context["context"], url=url_after, wait=value))

    # Wait for the prompt to open.
    await wait_for_future_safe(on_prompt_opened)
    # Make sure the navigation is not finished.
    assert not navigated_future.done(), "Navigation should not be finished before prompt is handled."

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept
    )

    if accept:
        await navigated_future
    else:
        with pytest.raises(error.UnknownErrorException):
            await wait_for_future_safe(navigated_future)


@pytest.mark.capabilities({"unhandledPromptBehavior": {'beforeUnload': 'ignore'}})
@pytest.mark.parametrize("value", ["none", "interactive", "complete"])
@pytest.mark.parametrize("accept", [True, False])
async def test_navigate_with_beforeunload_prompt_in_iframe_navigate_in_top_context(
        bidi_session, new_tab, setup_beforeunload_page, inline,
        subscribe_events, wait_for_event, wait_for_future_safe, value, accept):
    page = inline(f"""<iframe src={inline("foo")}></iframe>""")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]

    await setup_beforeunload_page(iframe_context)

    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    url_after = inline("<div>foo</div>")

    navigated_future = asyncio.create_task(
        bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=url_after, wait=value
        ))

    # Wait for the prompt to open.
    await wait_for_future_safe(on_prompt_opened)
    # Make sure the navigation is not finished.
    assert not navigated_future.done(), "Navigation should not be finished before prompt is handled."

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=accept
    )

    if accept:
        await navigated_future
    else:
        with pytest.raises(error.UnknownErrorException):
            await wait_for_future_safe(navigated_future)
