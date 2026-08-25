import pytest

from webdriver.bidi.modules.emulation import CoordinatesOptions

from . import TEST_COORDINATES


pytestmark = pytest.mark.asyncio


async def test_contexts(
    bidi_session, new_tab, top_context, url, get_current_geolocation, set_geolocation_permission
):
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)

    default_coordinates = await get_current_geolocation(new_tab)

    assert default_coordinates != TEST_COORDINATES
    assert await get_current_geolocation(top_context) == default_coordinates

    # Set geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        coordinates=CoordinatesOptions(
            latitude=TEST_COORDINATES["latitude"],
            longitude=TEST_COORDINATES["longitude"],
            accuracy=TEST_COORDINATES["accuracy"],
        ),
    )

    assert await get_current_geolocation(new_tab) == TEST_COORDINATES
    assert await get_current_geolocation(top_context) == default_coordinates

    # Reset geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]], coordinates=None
    )

    assert await get_current_geolocation(new_tab) == default_coordinates
    assert await get_current_geolocation(top_context) == default_coordinates


async def test_multiple_contexts(
    bidi_session, new_tab, url, get_current_geolocation, set_geolocation_permission
):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)

    default_coordinates = await get_current_geolocation(new_tab)

    assert default_coordinates != TEST_COORDINATES
    assert await get_current_geolocation(new_context) == default_coordinates

    # Set geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"], new_context["context"]],
        coordinates=CoordinatesOptions(
            latitude=TEST_COORDINATES["latitude"],
            longitude=TEST_COORDINATES["longitude"],
            accuracy=TEST_COORDINATES["accuracy"],
        ),
    )

    assert await get_current_geolocation(new_tab) == TEST_COORDINATES
    assert await get_current_geolocation(new_context) == TEST_COORDINATES

    # Reset geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"], new_context["context"]], coordinates=None
    )

    # The new coordinates can be different from the initial ones if the position
    # was not available at the beginning.
    assert await get_current_geolocation(new_tab) != TEST_COORDINATES
    assert await get_current_geolocation(new_context) != TEST_COORDINATES
