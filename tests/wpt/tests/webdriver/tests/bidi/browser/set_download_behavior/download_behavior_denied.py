import pytest

pytestmark = pytest.mark.asyncio


async def test_deny_and_reset(bidi_session, new_tab, temp_dir,
        is_download_allowed,
        default_is_download_allowed):
    await bidi_session.browser.set_download_behavior(download_behavior={
        "type": "denied"
    })
    assert await is_download_allowed(new_tab) == False

    await bidi_session.browser.set_download_behavior(download_behavior=None)
    assert await is_download_allowed(new_tab) == default_is_download_allowed
