import pytest_asyncio


@pytest_asyncio.fixture
async def initial_window_state(bidi_session, top_context):
    windows = await bidi_session.browser.get_client_windows()

    return next(
        (
            window
            for window in windows
            if window["clientWindow"] == top_context["clientWindow"]
        ),
        None,
    )
