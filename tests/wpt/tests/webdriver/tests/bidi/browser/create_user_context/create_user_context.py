import pytest

from .. import get_user_context_ids


@pytest.mark.asyncio
async def test_create_context(bidi_session, create_user_context):
    user_context = await create_user_context()
    assert user_context in await get_user_context_ids(bidi_session)


@pytest.mark.asyncio
async def test_unique_id(bidi_session, create_user_context):
    first_context = await create_user_context()
    assert isinstance(first_context, str)

    assert first_context in await get_user_context_ids(bidi_session)

    other_context = await create_user_context()
    assert isinstance(other_context, str)

    assert first_context in await get_user_context_ids(bidi_session)
    assert other_context in await get_user_context_ids(bidi_session)

    assert first_context != other_context
