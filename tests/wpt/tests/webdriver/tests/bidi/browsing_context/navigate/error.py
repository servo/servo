import asyncio
import pytest
from webdriver.bidi.error import UnknownErrorException

from . import navigate_and_assert

pytestmark = pytest.mark.asyncio

NAVIGATION_STARTED_EVENT = "browsingContext.navigationStarted"
USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest.mark.parametrize(
    "url",
    [
        "thisprotocoldoesnotexist://",
        "https://doesnotexist.localhost/",
        "https://localhost:0",
    ],
    ids=[
        "protocol",
        "host",
        "port",
    ],
)
async def test_invalid_address(bidi_session, new_tab, url):
    await navigate_and_assert(bidi_session, new_tab, url, expected_error=True)


async def test_with_csp_meta_tag(
    bidi_session,
    inline,
    new_tab,
):
    same_origin_url = inline("<div>foo</div>")
    cross_origin_url = inline("<div>bar</div>", domain="alt")
    page_url = inline(
        f"""
<!DOCTYPE html>
<html>
    <head>
        <meta
  http-equiv="Content-Security-Policy"
  content="default-src 'self'" />
    </head>
    <body><iframe src="{same_origin_url}"></iframe></body>
</html>
"""
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]["context"]

    # Make sure that cross-origin navigation in iframe failed.
    with pytest.raises(UnknownErrorException):
        await bidi_session.browsing_context.navigate(
            context=iframe_context, url=cross_origin_url, wait="complete"
        )


@pytest.mark.parametrize(
    "header",
    [
        "Content-Security-Policy, default-src 'self'",
        "Cross-Origin-Embedder-Policy, require-corp",
    ],
)
async def test_with_content_blocking_header_in_top_context(
    bidi_session,
    subscribe_events,
    inline,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    header,
):
    same_origin_url = inline("<div>foo</div>")
    cross_origin_url = inline("<div>bar</div>", domain="alt")
    page_url = inline(
        f"""<iframe src={same_origin_url}></iframe>""",
        parameters={"pipe": f"header({header})"},
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]["context"]

    # Make sure that cross-origin navigation in iframe failed.
    with pytest.raises(UnknownErrorException):
        await bidi_session.browsing_context.navigate(
            context=iframe_context, url=cross_origin_url, wait="complete"
        )


@pytest.mark.parametrize(
    "header_value",
    [
        "SAMEORIGIN",
        "DENY",
    ],
)
async def test_with_x_frame_options_header(
    bidi_session,
    subscribe_events,
    inline,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
    header_value,
):
    iframe_url_without_header = inline("<div>bar</div>")
    iframe_url_with_header = inline(
        "<div>foo</div>",
        parameters={"pipe": f"header(X-Frame-Options, {header_value})"},
    )
    page_url = inline(
        f"""<iframe src={iframe_url_without_header}></iframe>""", domain="alt"
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]["context"]

    # Make sure that cross-origin navigation in iframe failed.
    with pytest.raises(UnknownErrorException):
        await bidi_session.browsing_context.navigate(
            context=iframe_context, url=iframe_url_with_header, wait="complete"
        )


async def test_with_new_navigation(
    bidi_session,
    subscribe_events,
    inline,
    url,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
):
    slow_page_url = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.html?pipe=trickle(d10)"
    )
    await subscribe_events(events=[NAVIGATION_STARTED_EVENT])

    on_navigation_started = wait_for_event(NAVIGATION_STARTED_EVENT)
    task = asyncio.ensure_future(
        bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=slow_page_url, wait="complete"
        )
    )
    await wait_for_future_safe(on_navigation_started)
    second_url = inline("<div>foo</div>")

    # Trigger the second navigation which should fail the first one.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=second_url, wait="none"
    )

    # Make sure that the first navigation failed.
    with pytest.raises(UnknownErrorException):
        await task


async def test_with_new_navigation_inside_page(
    bidi_session,
    subscribe_events,
    inline,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
):
    second_url = inline("<div>foo</div>")
    slow_page_url = inline(
        f"""
<!DOCTYPE html>
<html>
    <body>
        <img src="/webdriver/tests/bidi/browsing_context/support/empty.svg?pipe=trickle(d10)" />
        <script>
            location.href = "{second_url}"
        </script>
        <img src="/webdriver/tests/bidi/browsing_context/support/empty.svg?pipe=trickle(d10)" />
    </body>
</html>
"""
    )

    # Make sure that the navigation failed.
    with pytest.raises(UnknownErrorException):
        await bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=slow_page_url, wait="complete"
        )


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_close_context(
    bidi_session,
    url,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    type_hint,
):
    await subscribe_events(events=[NAVIGATION_STARTED_EVENT])

    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)
    slow_page_url = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.html?pipe=trickle(d10)"
    )

    on_navigation_started = wait_for_event(NAVIGATION_STARTED_EVENT)
    task = asyncio.ensure_future(
        bidi_session.browsing_context.navigate(
            context=new_context["context"], url=slow_page_url, wait="complete"
        )
    )
    await wait_for_future_safe(on_navigation_started)

    await bidi_session.browsing_context.close(context=new_context["context"])

    # Make sure that the navigation failed.
    with pytest.raises(UnknownErrorException):
        await task


async def test_close_iframe(
    bidi_session,
    subscribe_events,
    inline,
    url,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
):
    iframe_url = inline("<div>foo</div>")
    page_url = inline(f"<iframe src={iframe_url}></iframe")

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])
    iframe_context = contexts[0]["children"][0]["context"]

    slow_page_url = url(
        "/webdriver/tests/bidi/browsing_context/support/empty.html?pipe=trickle(d10)"
    )
    await subscribe_events(events=[NAVIGATION_STARTED_EVENT])

    on_navigation_started = wait_for_event(NAVIGATION_STARTED_EVENT)
    # Navigate in the iframe.
    task = asyncio.ensure_future(
        bidi_session.browsing_context.navigate(
            context=iframe_context, url=slow_page_url, wait="complete"
        )
    )
    await wait_for_future_safe(on_navigation_started)

    # Reload the top context to destroy the iframe.
    await bidi_session.browsing_context.reload(context=new_tab["context"], wait="none")

    # Make sure that the iframe navigation failed.
    with pytest.raises(UnknownErrorException):
        await task


@pytest.mark.capabilities({"unhandledPromptBehavior": {"beforeUnload": "ignore"}})
async def test_beforeunload_rejected(
    bidi_session,
    new_tab,
    inline,
    setup_beforeunload_page,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
    await setup_beforeunload_page(new_tab)

    url_after = inline("<div>foo</div>")

    on_prompt_opened = wait_for_event(USER_PROMPT_OPENED_EVENT)
    task = asyncio.ensure_future(
        bidi_session.browsing_context.navigate(
            context=new_tab["context"], url=url_after, wait="complete"
        )
    )
    # Wait for the prompt to open.
    await wait_for_future_safe(on_prompt_opened)

    # Stay on the page to fail the started navigation.
    await bidi_session.browsing_context.handle_user_prompt(
        context=new_tab["context"], accept=False
    )

    with pytest.raises(UnknownErrorException):
        await task
