# META: timeout=long

import pytest
from webdriver.bidi.modules.browser import (
    ClientWindowNamedState,
    ClientWindowRectState,
)


TRANSITION_STATES = [
    ("fullscreen", "normal"),
    ("fullscreen", "maximized"),
    ("fullscreen", "minimized"),
    ("maximized", "fullscreen"),
    ("maximized", "normal"),
    ("maximized", "minimized"),
    ("minimized", "fullscreen"),
    ("minimized", "maximized"),
    ("minimized", "normal"),
    ("normal", "fullscreen"),
    ("normal", "maximized"),
    ("normal", "minimized"),
]


@pytest.mark.parametrize(
    "initial_state,target_state",
    [
        pytest.param(initial, target, id=f"{initial}-to-{target}")
        for initial, target in TRANSITION_STATES
    ],
)
@pytest.mark.asyncio
async def test_set_client_window_state_transitions(
    bidi_session, top_context, initial_state, target_state
):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"], state=initial_state
    )
    assert result["state"] == initial_state

    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"], state=target_state
    )
    assert result["state"] == target_state


@pytest.mark.parametrize(
    "state",
    [
        ClientWindowRectState.NORMAL.value,
        ClientWindowNamedState.FULLSCREEN.value,
        ClientWindowNamedState.MAXIMIZED.value,
        ClientWindowNamedState.MINIMIZED.value,
    ],
)
@pytest.mark.asyncio
async def test_idempotent_state_change(bidi_session, top_context, state):
    result1 = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"], state=state
    )
    assert result1["state"] == state

    result2 = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"], state=state
    )
    assert result2["state"] == state


@pytest.mark.asyncio
async def test_set_client_window_state_maximized(
    bidi_session, top_context, initial_window_state
):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowNamedState.MAXIMIZED.value,
    )

    assert result["state"] == ClientWindowNamedState.MAXIMIZED.value
    assert result["width"] > initial_window_state["width"]
    assert result["height"] > initial_window_state["height"]
    assert result["x"] <= initial_window_state["x"]
    assert result["y"] <= initial_window_state["y"]


@pytest.mark.asyncio
async def test_set_client_window_state_minimized(bidi_session, top_context):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowNamedState.MINIMIZED.value,
    )

    assert result["state"] == ClientWindowNamedState.MINIMIZED.value


@pytest.mark.asyncio
async def test_set_client_window_state_fullscreen(bidi_session, top_context):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowNamedState.FULLSCREEN.value,
    )

    assert result["state"] == ClientWindowNamedState.FULLSCREEN.value
