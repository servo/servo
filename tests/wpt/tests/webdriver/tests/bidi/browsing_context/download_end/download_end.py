import pytest
import uuid
from tests.support.sync import AsyncPoll
from webdriver.error import TimeoutException

from webdriver.bidi.modules.script import ContextTarget

from ... import (any_int, any_string, recursive_compare)

pytestmark = pytest.mark.asyncio

CONTENT = "SOME_FILE_CONTENT"
DOWNLOAD_END = "browsingContext.downloadEnd"
NAVIGATION_STARTED = "browsingContext.navigationStarted"


@pytest.fixture
def filename():
    return str(uuid.uuid4()) + '.txt'


@pytest.fixture(params=['data', 'http'])
def download_link(request, filename, inline):
    if request.param == 'data':
        return f"data:text/plain;charset=utf-8,{CONTENT}"
    return inline(CONTENT,
                  # Doctype `html_quirks` is required to avoid wrapping content.
                  doctype="html_quirks")


async def test_unsubscribe(bidi_session, inline, new_tab, wait_for_event,
        wait_for_future_safe, download_link, filename):
    url = inline(
        f"""<a id="download_link" href="{download_link}" download="{filename}">download</a>""")

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await bidi_session.session.subscribe(events=[DOWNLOAD_END])
    await bidi_session.session.unsubscribe(events=[DOWNLOAD_END])

    # Track all received events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener(DOWNLOAD_END,
                                                      on_event)

    await bidi_session.script.evaluate(
        expression=
        "download_link.click()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
        user_activation=True)

    wait = AsyncPoll(bidi_session, timeout=0.5)
    with pytest.raises(TimeoutException):
        await wait.until(lambda _: len(events) > 0)

    remove_listener()


async def test_download_attribute(bidi_session, subscribe_events, new_tab, inline,
        wait_for_event, wait_for_future_safe, download_link, filename):
    url = inline(
        f"""<a id="download_link" href="{download_link}" download="{filename}">download</a>""")

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    await subscribe_events(events=[DOWNLOAD_END])
    on_entry = wait_for_event(DOWNLOAD_END)

    await bidi_session.script.evaluate(
        expression=
        "download_link.click()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
        user_activation=True)

    event = await wait_for_future_safe(on_entry)
    recursive_compare(
        {
            'filepath': any_string,
            'context': new_tab["context"],
            'navigation': None,
            'status': 'complete',
            'timestamp': any_int,
            'url': download_link,
        }, event)

    # Assert file content is available.
    with open(event['filepath'], mode='r', encoding='utf-8') as file:
        file_content = file.read()
    assert file_content == CONTENT


async def test_content_disposition_header(
    bidi_session,
    subscribe_events,
    new_tab,
    url,
    inline,
    wait_for_event,
    wait_for_future_safe,
    filename,
):
    content_disposition_link = url(
        "/webdriver/tests/support/http_handlers/headers.py?"
        + f"content={CONTENT}"
        + f"&header=Content-Disposition:attachment;%20filename={filename}"
    )
    page_url = inline(
        f"""<a id="content_disposition_link" href="{content_disposition_link}">contentdisposition</a>"""
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=page_url, wait="complete"
    )

    await subscribe_events(events=[DOWNLOAD_END, NAVIGATION_STARTED])
    on_navigation_started = wait_for_event(NAVIGATION_STARTED)
    on_download_end = wait_for_event(DOWNLOAD_END)

    await bidi_session.script.evaluate(
        expression="content_disposition_link.click()",
        target=ContextTarget(new_tab["context"]),
        await_promise=True,
        user_activation=True,
    )

    download_event = await wait_for_future_safe(on_download_end)
    recursive_compare(
        {
            "filepath": any_string,
            "context": new_tab["context"],
            "navigation": any_string,
            "status": "complete",
            "timestamp": any_int,
            "url": content_disposition_link,
        },
        download_event,
    )

    navigation_event = await wait_for_future_safe(on_navigation_started)

    # Check that the navigation id and url are identical for navigationStarted
    # and downloadWillBegin.
    assert download_event["navigation"] == navigation_event["navigation"]
    assert download_event["url"] == navigation_event["url"]

    # Assert file content is available.
    with open(download_event["filepath"], mode="r", encoding="utf-8") as file:
        file_content = file.read()
    assert file_content == CONTENT
