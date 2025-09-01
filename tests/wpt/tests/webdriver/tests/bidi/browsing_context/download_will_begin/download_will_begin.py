import random

import pytest
from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException

from tests.bidi import wait_for_bidi_events
from ... import any_int, any_string, recursive_compare

pytestmark = pytest.mark.asyncio

DOWNLOAD_WILL_BEGIN = "browsingContext.downloadWillBegin"


async def test_unsubscribe(bidi_session, inline, new_tab):
    filename = f"some_file_name{random.random()}.txt"
    download_link = "data:text/plain;charset=utf-8,"
    url = inline(
        f"""<a id="download_link" href="{download_link}" download="{filename}">download</a>"""
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.session.subscribe(events=[DOWNLOAD_WILL_BEGIN])
    await bidi_session.session.unsubscribe(events=[DOWNLOAD_WILL_BEGIN])

    # Track all received events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(DOWNLOAD_WILL_BEGIN, on_event)

    await bidi_session.script.evaluate(
        expression="download_link.click()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
        user_activation=True,
    )

    with pytest.raises(TimeoutException):
        await wait_for_bidi_events(bidi_session, events, 1, timeout=0.5)

    remove_listener()


async def test_subscribe(
    bidi_session, new_tab, inline, wait_for_event, wait_for_future_safe
):
    download_filename = f"download_filename{random.random()}.txt"
    download_link = "data:text/plain;charset=utf-8,"
    page_url = inline(
        f"""<a id="download_link" href="{download_link}" download="{download_filename}">download</a>"""
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    await bidi_session.session.subscribe(events=[DOWNLOAD_WILL_BEGIN])
    on_entry = wait_for_event(DOWNLOAD_WILL_BEGIN)

    # Test clicking on a link with a "download" attribute.
    await bidi_session.script.evaluate(
        expression="download_link.click()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
        user_activation=True,
    )

    event = await wait_for_future_safe(on_entry)
    recursive_compare(
        {
            "context": new_tab["context"],
            "navigation": any_string,
            "suggestedFilename": download_filename,
            "timestamp": any_int,
            "url": download_link,
        },
        event,
    )


async def test_content_disposition_header(
    bidi_session, new_tab, inline, wait_for_event, wait_for_future_safe, url
):
    content_disposition_filename = f"content_disposition_filename{random.random()}.txt"
    content_disposition_link = url(
        "/webdriver/tests/support/http_handlers/headers.py?"
        + f"header=Content-Disposition:attachment;%20filename={content_disposition_filename}"
    )
    page_url = inline(
        f"""<a id="content_disposition_link" href="{content_disposition_link}">contentdisposition</a>"""
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    # Test clicking on a link which returns a response with a
    # Content-Disposition header.
    on_entry = wait_for_event(DOWNLOAD_WILL_BEGIN)
    await bidi_session.script.evaluate(
        expression="content_disposition_link.click()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
        user_activation=True,
    )

    event = await wait_for_future_safe(on_entry)
    recursive_compare(
        {
            "context": new_tab["context"],
            "navigation": any_string,
            "suggestedFilename": content_disposition_filename,
            "timestamp": any_int,
            "url": content_disposition_link,
        },
        event,
    )

async def test_redirect_to_content_disposition_header(
    bidi_session, new_tab, inline, wait_for_event, wait_for_future_safe, url
):
    redirect_filename = f"redirect_filename{random.random()}.txt"
    content_disposition_link = url(
        "/webdriver/tests/support/http_handlers/headers.py?"
        + f"header=Content-Disposition:attachment;%20filename={redirect_filename}"
    )
    redirect_link = url(
        f"/webdriver/tests/support/http_handlers/redirect.py?location={content_disposition_link}"
    )

    page_url = inline(
        f"""<a id="redirect_link" href="{redirect_link}">redirect to download</a>"""
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    # Test clicking on a link which redirects to a response with a
    # Content-Disposition header.
    on_entry = wait_for_event(DOWNLOAD_WILL_BEGIN)
    await bidi_session.script.evaluate(
        expression="redirect_link.click()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
        user_activation=True,
    )

    event = await wait_for_future_safe(on_entry)
    recursive_compare(
        {
            "context": new_tab["context"],
            "navigation": any_string,
            "suggestedFilename": redirect_filename,
            "timestamp": any_int,
            "url": content_disposition_link,
        },
        event,
    )
