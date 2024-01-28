import pytest

import webdriver.bidi.error as error

from .. import get_user_context_ids


@pytest.mark.asyncio
async def test_remove_context(bidi_session, create_user_context):
    user_context = await create_user_context()
    assert user_context in await get_user_context_ids(bidi_session)

    await bidi_session.browser.remove_user_context(user_context=user_context)
    assert user_context not in await get_user_context_ids(bidi_session)
    assert "default" in await get_user_context_ids(bidi_session)
