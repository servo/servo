# META: timeout=long

import asyncio
import pytest

from ... import any_string

pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


async def wait_for_reload(bidi_session, context, wait, expect_timeout):
    # Ultimately, "interactive" and "complete" should support a timeout argument.
    # See https://github.com/w3c/webdriver-bidi/issues/188.
    if expect_timeout:
        with pytest.raises(asyncio.TimeoutError):
            await asyncio.wait_for(
                asyncio.shield(
                    bidi_session.browsing_context.reload(context=context,
                                                         wait=wait)),
                timeout=1,
            )
    else:
        await bidi_session.browsing_context.reload(context=context, wait=wait)


@pytest.mark.parametrize("wait", ["none", "interactive", "complete"])
async def test_expected_url(bidi_session, inline, new_tab, wait):
    url = inline("<div>foo</div>")

    navigate_result = await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url,
        wait="complete"
    )

    reload_result = await bidi_session.browsing_context.reload(
        context=new_tab["context"],
        wait=wait
    )

    assert reload_result["navigation"] != navigate_result["navigation"]
    assert reload_result["url"] == url

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"], max_depth=0)
    assert contexts[0]["url"] == url


@pytest.mark.parametrize(
    "wait, expect_timeout",
    [
        ("none", False),
        ("interactive", False),
        ("complete", True),
    ],
)
async def test_slow_image_blocks_load(bidi_session, inline, new_tab, wait,
                                      expect_timeout):

    image_url = "/webdriver/tests/bidi/browsing_context/support/empty.svg"
    url = inline(f"<img src='{image_url}?pipe=trickle(d3)'>")

    await bidi_session.browsing_context.navigate(context=new_tab["context"],
                                                 url=url,
                                                 wait="complete")

    await wait_for_reload(bidi_session, new_tab["context"], wait,
                          expect_timeout)

    # We cannot assert the URL for "complete", since we expect a timeout. For
    # this case, the wait_for_navigation helper will resume after 1 second,
    # there is no guarantee that the URL has been updated.
    if wait != "complete":
        contexts = await bidi_session.browsing_context.get_tree(
            root=new_tab["context"], max_depth=0)
        assert contexts[0]["url"] == url


@pytest.mark.parametrize(
    "wait, expect_timeout",
    [
        ("none", False),
        ("interactive", True),
        ("complete", True),
    ],
)
async def test_slow_page(
    bidi_session,
    new_tab,
    url,
    wait,
    expect_timeout,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    url = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.html?pipe=trickle(d3)"
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await subscribe_events(
        events=["browsingContext.domContentLoaded", "browsingContext.load"],
        contexts=[new_tab["context"]],
    )

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener_1 = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_2 = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    assert len(events) == 0

    on_dom_content_load = wait_for_event("browsingContext.domContentLoaded")
    on_load = wait_for_event("browsingContext.load")

    await wait_for_reload(bidi_session, new_tab["context"], wait, expect_timeout)
    # Note that we cannot assert the top context url here, because the navigation
    # is blocked on the initial url for this test case.

    await wait_for_future_safe(on_load, timeout=5)
    await wait_for_future_safe(on_dom_content_load, timeout=5)

    assert len(events) == 2

    remove_listener_2()
    remove_listener_1()


@pytest.mark.parametrize(
    "wait, expect_timeout",
    [
        ("none", False),
        ("interactive", True),
        ("complete", True),
    ],
)
async def test_slow_script_blocks_domContentLoaded(
    bidi_session,
    inline,
    new_tab,
    wait,
    expect_timeout,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    delay_in_seconds = 3
    script_url = "/webdriver/tests/bidi/browsing_context/support/empty.js"
    url = inline(f"<script src='{script_url}?pipe=trickle(d{delay_in_seconds})'></script>")

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await subscribe_events(
        events=["browsingContext.domContentLoaded", "browsingContext.load"],
        contexts=[new_tab["context"]],
    )

    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener_1 = bidi_session.add_event_listener(
        "browsingContext.domContentLoaded", on_event
    )
    remove_listener_2 = bidi_session.add_event_listener(
        "browsingContext.load", on_event
    )

    assert len(events) == 0

    on_dom_content_load = wait_for_event("browsingContext.domContentLoaded")
    on_load = wait_for_event("browsingContext.load")

    await wait_for_reload(bidi_session, new_tab["context"], wait, expect_timeout)

    # Use twice the delay_in_seconds for the page load to wait for the navigation
    # events.
    await wait_for_future_safe(on_dom_content_load, timeout=delay_in_seconds*2)
    await wait_for_future_safe(on_load, timeout=delay_in_seconds*2)

    assert len(events) == 2

    remove_listener_2()
    remove_listener_1()


@pytest.mark.parametrize(
    "wait",
    ["none", "interactive", "complete"],
)
@pytest.mark.capabilities({"unhandledPromptBehavior": {"beforeUnload": "ignore"}})
async def test_beforeunload_prompt(
    bidi_session,
    new_tab,
    setup_beforeunload_page,
    url,
    subscribe_events,
    wait,
    wait_for_event,
    wait_for_future_safe,
):
    page_url = url("/webdriver/tests/support/html/beforeunload.html")
    await setup_beforeunload_page(new_tab)

    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)

    reloaded_future = asyncio.create_task(
        bidi_session.browsing_context.reload(context=new_tab["context"], wait=wait)
    )

    await wait_for_future_safe(on_prompt_opened)
    # Make sure the navigation is not finished.
    assert (
        not reloaded_future.done()
    ), "Reload should not be finished before prompt is handled."

    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=True
    )

    reloaded_result = await reloaded_future

    assert reloaded_result["url"] == page_url
    any_string(reloaded_result["navigation"])
