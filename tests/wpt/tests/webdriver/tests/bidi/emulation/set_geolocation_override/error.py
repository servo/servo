import pytest

from webdriver.bidi.modules.script import ContextTarget

from ... import remote_mapping_to_dict

pytestmark = pytest.mark.asyncio

ERROR = {"type": "positionUnavailable"}
EXPECTED_ERROR = {"code": 2}


async def test_get_current_position(bidi_session, new_tab, url,
        get_current_geolocation, set_geolocation_permission):
    test_url = url("/common/blank.html")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )
    await set_geolocation_permission(new_tab)

    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]], error=ERROR
    )

    assert await get_current_geolocation(new_tab) == EXPECTED_ERROR


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

    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        error=ERROR
    )

    on_script_message = wait_for_event("script.message")
    await bidi_session.browsing_context.activate(context=new_tab["context"])
    await bidi_session.script.call_function(
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        function_declaration="""(channel) =>
            window.navigator.geolocation.watchPosition(
                (result) => channel("unexpected result"),
                (error) => channel({code: error.code})
            )
        """,
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )
    event_data = await wait_for_future_safe(on_script_message)

    assert remote_mapping_to_dict(event_data["data"]["value"]) == EXPECTED_ERROR


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

    # Set geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        error=ERROR,
    )

    assert await get_current_geolocation(new_tab) == EXPECTED_ERROR

    await bidi_session.browsing_context.reload(
        context=new_tab["context"], wait="complete"
    )

    assert await get_current_geolocation(new_tab) == EXPECTED_ERROR


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

    # Set geolocation override.
    await bidi_session.emulation.set_geolocation_override(
        contexts=[new_tab["context"]],
        error=ERROR,
    )

    assert await get_current_geolocation(new_tab) == EXPECTED_ERROR

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=url("/webdriver/tests/support/html/default.html"),
        wait="complete",
    )

    assert await get_current_geolocation(new_tab) == EXPECTED_ERROR
