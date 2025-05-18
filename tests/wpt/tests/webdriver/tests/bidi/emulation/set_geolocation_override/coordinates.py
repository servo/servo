import pytest

from webdriver.bidi.modules.emulation import CoordinatesOptions
from webdriver.bidi.modules.script import ContextTarget

from ... import remote_mapping_to_dict


pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "test_coordinates",
    [
        ({"latitude": 4, "longitude": 2}),
        ({"latitude": 4, "longitude": 2, "accuracy": 0.5}),
        ({"latitude": 4, "longitude": 2, "altitude": 2}),
        ({"latitude": 4, "longitude": 2, "altitude": 2, "altitudeAccuracy": 3}),
        ({"latitude": 4, "longitude": 2, "speed": 7}),
        ({"latitude": 4, "longitude": 2, "heading": 8}),
    ],
)
async def test_get_current_position(
    bidi_session,
    new_tab,
    url,
    get_current_geolocation,
    set_geolocation_permission,
    test_coordinates,
):
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)

    default_coordinates = await get_current_geolocation(new_tab)

    # Set default accuracy value.
    if "accuracy" not in test_coordinates:
        test_coordinates["accuracy"] = 1

    assert default_coordinates != test_coordinates

    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]], coordinates=test_coordinates
    )

    assert await get_current_geolocation(new_tab) == test_coordinates


async def test_watch_position(
    bidi_session,
    new_tab,
    url,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
    set_geolocation_permission,
):
    await subscribe_events(["script.message"])

    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)

    test_override_1 = {"latitude": 0, "longitude": 0, "accuracy": 1}
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        coordinates=CoordinatesOptions(
            latitude=test_override_1["latitude"],
            longitude=test_override_1["longitude"],
            accuracy=test_override_1["accuracy"],
        ),
    )

    on_script_message = wait_for_event("script.message")
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    watch_id = await bidi_session.script.call_function(
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        function_declaration="""(channel) =>
            window.navigator.geolocation.watchPosition(
                (result) => channel(result.coords.toJSON())
            )
        """,
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )
    event_data = await wait_for_future_safe(on_script_message)

    assert remote_mapping_to_dict(event_data["data"]["value"]) == test_override_1

    test_override_2 = {"latitude": 10, "longitude": 10, "accuracy": 3}
    on_script_message = wait_for_event("script.message")
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        coordinates=CoordinatesOptions(
            latitude=test_override_2["latitude"],
            longitude=test_override_2["longitude"],
            accuracy=test_override_2["accuracy"],
        ),
    )
    event_data = await wait_for_future_safe(on_script_message)

    assert remote_mapping_to_dict(event_data["data"]["value"]) == test_override_2

    test_override_3 = {"latitude": 20, "longitude": 10, "accuracy": 1}
    on_script_message = wait_for_event("script.message")
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        coordinates=CoordinatesOptions(
            latitude=test_override_3["latitude"],
            longitude=test_override_3["longitude"],
            accuracy=test_override_3["accuracy"],
        ),
    )
    event_data = await wait_for_future_safe(on_script_message)

    assert remote_mapping_to_dict(event_data["data"]["value"]) == test_override_3

    # Clear the geolocation watcher.
    await bidi_session.script.call_function(
        arguments=[{"type": "number", "value": watch_id["value"]}],
        function_declaration="""(watchId) =>
            window.navigator.geolocation.clearWatch(watchId)
        """,
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )


async def test_persists_on_reload(
    bidi_session, url, new_tab, get_current_geolocation, set_geolocation_permission
):
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)
    test_coordinates = {"latitude": 4, "longitude": 2, "accuracy": 3}

    # Set geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        coordinates=CoordinatesOptions(
            latitude=test_coordinates["latitude"],
            longitude=test_coordinates["longitude"],
            accuracy=test_coordinates["accuracy"],
        ),
    )

    assert await get_current_geolocation(new_tab) == test_coordinates

    await bidi_session.browsing_context.reload(
        context=new_tab["context"], wait="complete"
    )

    assert await get_current_geolocation(new_tab) == test_coordinates


async def test_persists_on_navigation(
    bidi_session, url, new_tab, get_current_geolocation, set_geolocation_permission
):
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)
    test_coordinates = {"latitude": 4, "longitude": 2, "accuracy": 3}

    # Set geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        coordinates=CoordinatesOptions(
            latitude=test_coordinates["latitude"],
            longitude=test_coordinates["longitude"],
            accuracy=test_coordinates["accuracy"],
        ),
    )

    assert await get_current_geolocation(new_tab) == test_coordinates

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url("/webdriver/tests/support/html/default.html"),
        wait="complete",
    )

    assert await get_current_geolocation(new_tab) == test_coordinates


async def test_reset_without_override(
    bidi_session, new_tab, url, get_current_geolocation, set_geolocation_permission
):
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)

    default_coordinates = await get_current_geolocation(new_tab)

    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]], coordinates=None
    )

    assert await get_current_geolocation(new_tab) == default_coordinates
