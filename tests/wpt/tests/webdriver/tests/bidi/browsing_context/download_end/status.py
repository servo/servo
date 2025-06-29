import pytest
import uuid

from webdriver.bidi.modules.script import ContextTarget

from ... import (any_int, any_string, recursive_compare)

pytestmark = pytest.mark.asyncio

DOWNLOAD_END = "browsingContext.downloadEnd"


@pytest.fixture
def filename():
    return str(uuid.uuid4()) + '.txt'


async def test_status_complete(bidi_session, subscribe_events, new_tab, inline,
        wait_for_event, wait_for_future_safe, filename):
    download_link = inline("SOME_CONTENT")
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
            'navigation': any_string,
            'status': 'complete',
            'timestamp': any_int,
            'url': download_link,
        }, event)


async def test_status_canceled(bidi_session, subscribe_events, new_tab, inline,
        wait_for_event, wait_for_future_safe, filename):
    url = inline(
        f"""<a id="download_link" href="some_non_existing_link" download="{filename}">download</a>""")

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
            'context': new_tab["context"],
            'navigation': any_string,
            'status': 'canceled',
            'timestamp': any_int,
            'url': any_string,
        }, event)
