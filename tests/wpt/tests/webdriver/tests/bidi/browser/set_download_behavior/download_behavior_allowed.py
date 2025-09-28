import pytest

pytestmark = pytest.mark.asyncio


async def test_allow_and_reset(bidi_session, new_tab, temp_dir,
        is_download_allowed, trigger_download, default_is_download_allowed):
    await bidi_session.browser.set_download_behavior(download_behavior={
        "type": "allowed",
        "destinationFolder": temp_dir
    })
    assert await is_download_allowed(new_tab) == True

    await bidi_session.browser.set_download_behavior(download_behavior=None)
    assert await is_download_allowed(new_tab) == default_is_download_allowed


async def test_destination_folder(bidi_session, new_tab, temp_dir,
        trigger_download):
    await bidi_session.browser.set_download_behavior(download_behavior={
        "type": "allowed",
        "destinationFolder": temp_dir
    })
    event = await trigger_download(new_tab)
    # Assert download is allowed.
    assert event["status"] == "complete"
    # Assert `destinationFolder` is respected.
    assert event["filepath"].startswith(temp_dir)
