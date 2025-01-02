import pytest

from . import get_context_origin, get_permission_state

pytestmark = pytest.mark.asyncio


async def test_set_permission_user_context(
    bidi_session, new_tab, url, create_user_context
):
    test_url = url("/common/blank.html", protocol="https", domain="alt")

    user_context = await create_user_context()
    # new_tab is in the default user context. new_tab2 is in the non-default user context.
    new_tab2 = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )

    # Navigate a context in the default user context.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    # Navigate a context in the non-default user context.
    await bidi_session.browsing_context.navigate(
        context=new_tab2["context"],
        url=test_url,
        wait="complete",
    )

    origin = await get_context_origin(bidi_session, new_tab)

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"
    assert await get_permission_state(bidi_session, new_tab2, "geolocation") == "prompt"

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="granted",
        origin=origin,
        user_context=user_context,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"
    assert (
        await get_permission_state(bidi_session, new_tab2, "geolocation") == "granted"
    )

    # Create another tab in the non-default user context
    new_tab3 = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab3["context"],
        url=test_url,
        wait="complete",
    )

    # Make sure that the permission state is still present.
    assert (
        await get_permission_state(bidi_session, new_tab3, "geolocation") == "granted"
    )


async def test_set_permission_with_reload(bidi_session, url, create_user_context):
    user_context = await create_user_context()
    new_tab = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )

    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    origin = await get_context_origin(bidi_session, new_tab)

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="granted",
        origin=origin,
        user_context=user_context,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "granted"

    await bidi_session.browsing_context.reload(
        context=new_tab["context"],
        wait="complete",
    )

    # Make sure that permission state is still set.
    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "granted"


async def test_reset_permission(bidi_session, url, create_user_context):
    test_url = url("/common/blank.html", protocol="https")

    user_context = await create_user_context()
    new_tab = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    origin = await get_context_origin(bidi_session, new_tab)

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="granted",
        origin=origin,
        user_context=user_context,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "granted"

    # Reset the permission state
    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="prompt",
        origin=origin,
        user_context=user_context,
    )
    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"
