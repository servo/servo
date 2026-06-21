import pytest_asyncio

DOWNLOAD_END = "browsingContext.downloadEnd"


# This fixture is a workaround until we can cancel downloads.
# https://github.com/w3c/webdriver-bidi/issues/1031
@pytest_asyncio.fixture
async def expect_download_end(bidi_session, subscribe_events, wait_for_bidi_events):
    await subscribe_events(events=[DOWNLOAD_END])

    download_end_events = []

    async def on_event(method, data):
        download_end_events.append(data)

    remove_listener = bidi_session.add_event_listener(DOWNLOAD_END, on_event)

    expected_events = 0

    def _expect_download_end(count):
        nonlocal expected_events
        expected_events = count

    yield _expect_download_end

    await wait_for_bidi_events(download_end_events, expected_events, timeout=2)
    remove_listener()
