import asyncio

import pytest
import webdriver.bidi.error as error
from webdriver.bidi.modules.script import ContextTarget

from . import navigate_and_assert
from ... import any_string

pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


async def test_payload(bidi_session, inline, new_tab):
    url = inline("<div>foo</div>")
    result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url
    )

    any_string(result["navigation"])
    assert result["url"] == url


async def test_interactive_simultaneous_navigation(bidi_session, wait_for_future_safe, inline, new_tab):
    frame1_start_url = inline("frame1")
    frame2_start_url = inline("frame2")

    url = inline(
        f"<iframe src='{frame1_start_url}'></iframe><iframe src='{frame2_start_url}'></iframe>"
    )

    contexts = await navigate_and_assert(bidi_session, new_tab, url)
    assert len(contexts[0]["children"]) == 2

    frame1_context_id = contexts[0]["children"][0]["context"]
    frame2_context_id = contexts[0]["children"][1]["context"]

    # The goal here is to navigate both iframes in parallel, and to use the
    # interactive wait condition for both.
    # Make sure that monitoring the DOMContentLoaded event for one frame does
    # prevent monitoring it for the other frame.
    img_url = "/webdriver/tests/bidi/browsing_context/support/empty.svg"
    script_url = "/webdriver/tests/bidi/browsing_context/support/empty.js"
    # frame1 also has a slow loading image so that it won't reach a complete
    # navigation, and we can make sure we resolved with the interactive state.
    frame1_url = inline(
        f"""frame1_new<script src='{script_url}?pipe=trickle(d2)'></script>
        <img src='{img_url}?pipe=trickle(d100)'>
        """
    )
    frame2_url = inline(
        f"frame2_new<script src='{script_url}?pipe=trickle(d0.5)'></script>"
    )

    frame1_task = asyncio.ensure_future(
        bidi_session.browsing_context.navigate(
            context=frame1_context_id, url=frame1_url, wait="interactive"
        )
    )

    frame2_result = await bidi_session.browsing_context.navigate(
        context=frame2_context_id, url=frame2_url, wait="interactive"
    )
    assert frame2_result["url"] == frame2_url

    # The "interactive" navigation should resolve before the 5 seconds timeout.
    await wait_for_future_safe(frame1_task, timeout=5)

    frame1_result = frame1_task.result()
    assert frame1_result["url"] == frame1_url

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    assert contexts[0]["children"][0]["url"] == frame1_url
    assert contexts[0]["children"][1]["url"] == frame2_url

    any_string(frame1_result["navigation"])
    any_string(frame2_result["navigation"])
    assert frame1_result["navigation"] != frame2_result["navigation"]


async def test_relative_url(bidi_session, new_tab, url):
    url_before = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.html"
    )

    await navigate_and_assert(bidi_session, new_tab, url_before, "none")

    url_after = url_before.replace("empty.html", "other.html")
    await navigate_and_assert(bidi_session, new_tab, url_after, "none")


async def test_same_document_navigation_in_before_unload(bidi_session, new_tab, url):
    url_before = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.html"
    )

    await navigate_and_assert(bidi_session, new_tab, url_before, "complete")

    await bidi_session.script.evaluate(
        expression="""window.addEventListener(
          'beforeunload',
          () => history.replaceState(null, 'initial', window.location.href),
          false
        );""",
        target=ContextTarget(new_tab["context"]),
        await_promise=False)

    url_after = url_before.replace("empty.html", "other.html")
    await navigate_and_assert(bidi_session, new_tab, url_after, "complete")


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
