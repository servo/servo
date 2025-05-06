import pytest_asyncio

from ... import get_context_origin


@pytest_asyncio.fixture
async def set_geolocation_permission(bidi_session):
    data_to_cleanup = {}

    async def set_geolocation_permission(context, user_context="default"):
        nonlocal data_to_cleanup

        origin = await get_context_origin(bidi_session, context)

        data_to_cleanup = {
            "origin": origin,
            "user_context": user_context,
        }

        await bidi_session.permissions.set_permission(
            descriptor={"name": "geolocation"},
            state="granted",
            origin=origin,
            user_context=user_context,
        )

    yield set_geolocation_permission

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="prompt",
        origin=data_to_cleanup["origin"],
        user_context=data_to_cleanup["user_context"],
    )
