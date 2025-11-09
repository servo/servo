import pytest

from webdriver.bidi.modules.emulation import CoordinatesOptions

from . import TEST_COORDINATES


pytestmark = pytest.mark.asyncio


async def test_user_contexts(
    bidi_session,
    url,
    create_user_context,
    new_tab,
    get_current_geolocation,
    set_geolocation_permission,
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_1["context"],
        url=test_url,
        wait="complete",
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)
    await set_geolocation_permission(new_tab, user_context)

    default_coordinates = await get_current_geolocation(context_in_user_context_1)

    assert default_coordinates != TEST_COORDINATES
    assert await get_current_geolocation(new_tab) == default_coordinates

    # Set geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        user_contexts=[user_context],
        coordinates=CoordinatesOptions(
            latitude=TEST_COORDINATES["latitude"],
            longitude=TEST_COORDINATES["longitude"],
            accuracy=TEST_COORDINATES["accuracy"],
        ),
    )

    assert await get_current_geolocation(context_in_user_context_1) == TEST_COORDINATES
    assert await get_current_geolocation(new_tab) == default_coordinates

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_2["context"],
        url=test_url,
        wait="complete",
    )

    assert await get_current_geolocation(context_in_user_context_2) == TEST_COORDINATES


async def test_set_to_default_user_context(
    bidi_session,
    new_tab,
    create_user_context,
    url,
    get_current_geolocation,
    set_geolocation_permission,
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_1["context"],
        url=test_url,
        wait="complete",
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)
    await set_geolocation_permission(new_tab, user_context)

    default_coordinates = await get_current_geolocation(new_tab)
    assert default_coordinates != TEST_COORDINATES

    await bidi_session.emulation.set_geolocation_override(
        user_contexts=["default"],
        coordinates=CoordinatesOptions(
            latitude=TEST_COORDINATES["latitude"],
            longitude=TEST_COORDINATES["longitude"],
            accuracy=TEST_COORDINATES["accuracy"],
        ),
    )

    # Make sure that the geolocation changes are only applied to the context associated with default user context.
    assert (
        await get_current_geolocation(context_in_user_context_1) == default_coordinates
    )
    assert await get_current_geolocation(new_tab) == TEST_COORDINATES

    # Create a new context in the default context.
    context_in_default_context_2 = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    await bidi_session.browsing_context.navigate(
        context=context_in_default_context_2["context"],
        url=test_url,
        wait="complete",
    )

    assert (
        await get_current_geolocation(context_in_default_context_2) == TEST_COORDINATES
    )

    # Reset geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        user_contexts=["default"], coordinates=None
    )


async def test_set_to_multiple_user_contexts(
    bidi_session,
    create_user_context,
    url,
    get_current_geolocation,
    set_geolocation_permission,
):
    user_context_1 = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    user_context_2 = await create_user_context()
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_1["context"],
        url=test_url,
        wait="complete",
    )
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_2["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(context_in_user_context_1, user_context_1)
    await set_geolocation_permission(context_in_user_context_2, user_context_2)

    await bidi_session.emulation.set_geolocation_override(
        user_contexts=[user_context_1, user_context_2],
        coordinates=CoordinatesOptions(
            latitude=TEST_COORDINATES["latitude"],
            longitude=TEST_COORDINATES["longitude"],
            accuracy=TEST_COORDINATES["accuracy"],
        ),
    )

    assert await get_current_geolocation(context_in_user_context_1) == TEST_COORDINATES
    assert await get_current_geolocation(context_in_user_context_2) == TEST_COORDINATES


async def test_set_to_user_context_and_then_to_context(
    bidi_session,
    create_user_context,
    url,
    new_tab,
    get_current_geolocation,
    set_geolocation_permission,
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_1["context"],
        url=test_url,
        wait="complete",
    )
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)
    await set_geolocation_permission(new_tab, user_context)

    default_coordinates = await get_current_geolocation(context_in_user_context_1)

    assert default_coordinates != TEST_COORDINATES

    # Apply geolocation override to the user context.
    await bidi_session.emulation.set_geolocation_override(
        user_contexts=[user_context],
        coordinates=CoordinatesOptions(
            latitude=TEST_COORDINATES["latitude"],
            longitude=TEST_COORDINATES["longitude"],
            accuracy=TEST_COORDINATES["accuracy"],
        ),
    )

    new_geolocation_coordinates = {"latitude": 30, "longitude": 20, "accuracy": 3}
    # Apply geolocation override now only to the context.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[context_in_user_context_1["context"]],
        coordinates=CoordinatesOptions(
            latitude=new_geolocation_coordinates["latitude"],
            longitude=new_geolocation_coordinates["longitude"],
            accuracy=new_geolocation_coordinates["accuracy"],
        ),
    )
    assert (
        await get_current_geolocation(context_in_user_context_1)
        == new_geolocation_coordinates
    )

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the geolocation is still updated.
    assert (
        await get_current_geolocation(context_in_user_context_1)
        == new_geolocation_coordinates
    )

    # Create a new context in the user context.
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_2["context"],
        url=test_url,
        wait="complete",
    )
    # Make sure that the user context is applied for the new context.
    assert await get_current_geolocation(context_in_user_context_2) == TEST_COORDINATES

    # Remove browsing context geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[context_in_user_context_1["context"]],
        coordinates=None,
    )

    # Make sure that the user context override is applied.
    assert await get_current_geolocation(context_in_user_context_2) == TEST_COORDINATES

    # Remove user context override.
    await bidi_session.emulation.set_geolocation_override(
        user_contexts=[user_context],
        coordinates=None
    )

    # Make sure that the geolocation override was reset.
    assert (
        await get_current_geolocation(context_in_user_context_1) == default_coordinates
    )
