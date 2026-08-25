# META: timeout=long

import pytest
from webdriver.bidi.modules.browser import ClientWindowRectState, ClientWindowNamedState

pytestmark = pytest.mark.asyncio


async def test_maximized_state_transitions(bidi_session, top_context):
    for _ in range(5):
        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowRectState.NORMAL.value,
        )
        assert result["state"] == ClientWindowRectState.NORMAL.value

        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowNamedState.MAXIMIZED.value,
        )
        assert result["state"] == ClientWindowNamedState.MAXIMIZED.value

        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowRectState.NORMAL.value,
        )
        assert result["state"] == ClientWindowRectState.NORMAL.value


async def test_minimized_transitions(bidi_session, top_context):
    for _ in range(5):
        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowRectState.NORMAL.value,
        )
        assert result["state"] == ClientWindowRectState.NORMAL.value

        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowNamedState.MINIMIZED.value,
        )
        assert result["state"] == ClientWindowNamedState.MINIMIZED.value

        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowRectState.NORMAL.value,
        )
        assert result["state"] == ClientWindowRectState.NORMAL.value


async def test_fullscreen_transitions(bidi_session, top_context):
    for _ in range(5):
        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowRectState.NORMAL.value,
        )
        assert result["state"] == ClientWindowRectState.NORMAL.value

        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowNamedState.FULLSCREEN.value,
        )
        assert result["state"] == ClientWindowNamedState.FULLSCREEN.value

        result = await bidi_session.browser.set_client_window_state(
            client_window=top_context["clientWindow"],
            state=ClientWindowRectState.NORMAL.value,
        )
        assert result["state"] == ClientWindowRectState.NORMAL.value
