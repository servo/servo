import pytest

from .. import get_user_context_ids
from .. import get_local_storage, set_local_storage


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


@pytest.mark.asyncio
async def test_storage_isolation(bidi_session, create_user_context, inline):
    first_context = await create_user_context()
    other_context = await create_user_context()

    test_key = "test"

    tab_first_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context=first_context
    )

    await bidi_session.browsing_context.navigate(context=tab_first_context["context"],
                                              url=inline("test"),
                                              wait="complete")

    tab_other_context = await bidi_session.browsing_context.create(
        type_hint="tab",
        user_context=other_context
    )

    await bidi_session.browsing_context.navigate(context=tab_other_context["context"],
                                          url=inline("test"),
                                          wait="complete")

    assert await get_local_storage(bidi_session, tab_first_context, test_key) == None
    assert await get_local_storage(bidi_session, tab_other_context, test_key) == None

    await set_local_storage(bidi_session, tab_first_context, test_key, "value")

    assert await get_local_storage(bidi_session, tab_first_context, test_key) == "value"
    assert await get_local_storage(bidi_session, tab_other_context, test_key) == None
