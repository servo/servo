import pytest

pytestmark = pytest.mark.asyncio


async def test_top_level_context_id_equals_window_handle(top_context, current_session):
    assert top_context["context"] == current_session.window_handle
