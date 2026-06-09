import pytest
from webdriver.bidi.modules.browser import (
    ClientWindowNamedState,
    ClientWindowRectState,
)

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "rect",
    [
        {"width": 650},
        {"height": 550},
        {"x": 250},
        {"y": 150},
        {"width": 650, "x": 250},
        {"height": 550, "x": 250},
        {"width": 650, "y": 150},
        {"height": 550, "y": 150},
    ],
    ids=["width", "height", "x", "y", "width_x", "height_x", "width_y", "height_y"],
)
async def test_partial_input(bidi_session, is_wayland_headful, top_context, initial_window_state, rect
):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowRectState.NORMAL.value,
        **rect,
    )

    assert result["state"] == ClientWindowRectState.NORMAL.value
    assert result["width"] == rect.get("width", initial_window_state["width"])
    assert result["height"] == rect.get("height", initial_window_state["height"])

    # Wayland forbids programmatic window movement in headful mode.
    if is_wayland_headful:
        assert result["x"] == initial_window_state["x"]
        assert result["y"] == initial_window_state["y"]
    else:
        assert result["x"] == rect.get("x", initial_window_state["x"])
        assert result["y"] == rect.get("y", initial_window_state["y"])


async def test_set_client_window_state_normal(
    bidi_session, is_wayland_headful, top_context, initial_window_state
):
    new_width = initial_window_state["width"] + 200
    new_height = initial_window_state["height"] + 150
    new_x = initial_window_state["x"] + 50
    new_y = initial_window_state["y"] + 50

    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowRectState.NORMAL.value,
        width=new_width,
        height=new_height,
        x=new_x,
        y=new_y,
    )

    assert result["state"] == ClientWindowRectState.NORMAL.value
    assert result["width"] == new_width
    assert result["height"] == new_height

    # Wayland forbids programmatic window movement in headful mode.
    if not is_wayland_headful:
        assert result["x"] == new_x
        assert result["y"] == new_y


async def test_move_xy(
    bidi_session, is_wayland_headful, top_context, initial_window_state
):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowRectState.NORMAL.value,
        x=250,
        y=150,
    )
    assert result["state"] == ClientWindowRectState.NORMAL.value

    # Wayland forbids programmatic window movement in headful mode.
    if not is_wayland_headful:
        assert result["x"] == 250
        assert result["y"] == 150

    # Dimensions should not have changed.
    assert result["width"] == initial_window_state["width"]
    assert result["height"] == initial_window_state["height"]


async def test_resize_width_height(bidi_session, top_context, initial_window_state):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowRectState.NORMAL.value,
        width=750,
        height=550,
    )
    assert result["state"] == ClientWindowRectState.NORMAL.value
    assert result["width"] == 750
    assert result["height"] == 550

    # Position should not have changed
    assert result["x"] == initial_window_state["x"]
    assert result["y"] == initial_window_state["y"]


async def test_no_position_resize(bidi_session, top_context, initial_window_state):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowRectState.NORMAL.value,
    )
    assert result["state"] == ClientWindowRectState.NORMAL.value
    assert result["x"] == initial_window_state["x"]
    assert result["y"] == initial_window_state["y"]
    assert result["width"] == initial_window_state["width"]
    assert result["height"] == initial_window_state["height"]


async def test_same_position_resize(bidi_session, top_context, initial_window_state):
    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=ClientWindowRectState.NORMAL.value,
        x=initial_window_state["x"],
        y=initial_window_state["y"],
        width=initial_window_state["width"],
        height=initial_window_state["height"],
    )

    assert result["state"] == ClientWindowRectState.NORMAL.value
    assert result["x"] == initial_window_state["x"]
    assert result["y"] == initial_window_state["y"]
    assert result["width"] == initial_window_state["width"]
    assert result["height"] == initial_window_state["height"]


@pytest.mark.parametrize(
    "state",
    [
        ClientWindowNamedState.FULLSCREEN.value,
        ClientWindowNamedState.MAXIMIZED.value,
        ClientWindowNamedState.MINIMIZED.value,
    ],
)
async def test_move_in_special_state(bidi_session, top_context, state):
    initial_result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"], state=state
    )
    assert initial_result["state"] == state

    result = await bidi_session.browser.set_client_window_state(
        client_window=top_context["clientWindow"],
        state=state,
        x=150,
        y=250,
    )
    assert result["state"] == state
    assert result["x"] == initial_result["x"]
    assert result["y"] == initial_result["y"]
