import pytest
from tests.bidi import wait_for_bidi_events
from tests.bidi.network import (
    BEFORE_REQUEST_SENT_EVENT,
    STYLESHEET_RED_COLOR,
    get_cached_url,
)
from webdriver.bidi import error


# This is a smoke test triggering various requests of different kinds: regular
# http requests, data channels and cached resources, and check that no id is
# duplicated amongst them.
@pytest.mark.asyncio
async def test_unique_request_ids(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
    fetch,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    cached_link_css_url = url(get_cached_url("text/css", STYLESHEET_RED_COLOR))
    page_with_cached_css = inline(
        f"""
        <head><link rel="stylesheet" type="text/css" href="{cached_link_css_url}"></head>
        <body>test page with cached link stylesheet</body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_cached_css,
        wait="complete",
    )

    # Expect two events, one for the document, one for the stylesheet.
    await wait_for_bidi_events(bidi_session, events, 2, timeout=2)

    # Reload the page.
    await bidi_session.browsing_context.reload(
        context=top_context["context"], wait="complete"
    )

    # Expect two events after reload, for the document and the stylesheet.
    await wait_for_bidi_events(bidi_session, events, 4, timeout=2)

    await fetch("data:text/plain,1")
    await fetch("data:text/plain,2")
    await fetch("data:text/plain,3")
    await fetch("data:text/plain,4")

    # Expect four events for data: scheme fetches.
    await wait_for_bidi_events(bidi_session, events, 8, timeout=2)

    ids = list(map(lambda event: event["request"]["request"], events))

    # Check that all ids are unique by turning the list in a set and checking
    # the length.
    assert len(ids) == len(set(ids))
