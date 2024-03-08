async def assert_cookies_are_not_present(bidi_session, filter=None, partition=None):
    result = await bidi_session.storage.get_cookies(filter=filter, partition=partition)
    assert result["cookies"] == []
