import pytest

pytestmark = pytest.mark.asyncio


async def test_timezone_set_override_and_reset(bidi_session, top_context,
        get_current_timezone, default_timezone, some_timezone,
        another_timezone):
    assert await get_current_timezone(top_context) == default_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[top_context["context"]],
        timezone=some_timezone
    )

    assert await get_current_timezone(top_context) == some_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[top_context["context"]],
        timezone=another_timezone
    )

    assert await get_current_timezone(top_context) == another_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[top_context["context"]],
        timezone=None
    )

    assert await get_current_timezone(top_context) == default_timezone


@pytest.mark.parametrize("timezone_offset", ["+10:00"])
async def test_timezone_offset(bidi_session, top_context, get_current_timezone,
        default_timezone, timezone_offset):
    assert await get_current_timezone(top_context) == default_timezone

    # Set timezone override.
    await bidi_session.emulation.set_timezone_override(
        contexts=[top_context["context"]],
        timezone=timezone_offset
    )

    assert await get_current_timezone(top_context) == timezone_offset
