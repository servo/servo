import pytest

from . import get_angle
from ... import remote_mapping_to_dict

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("_type", ["portrait-primary", "portrait-secondary",
                                   "landscape-primary", "landscape-secondary"])
@pytest.mark.parametrize("natural", ["portrait", "landscape"])
async def test_screen_orientation(bidi_session, top_context,
        get_screen_orientation, _type, natural, default_screen_orientation):
    # Set screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[top_context["context"]],
        screen_orientation={
            "type": _type,
            "natural": natural
        })

    # Assert screen orientation is updated.
    assert (await get_screen_orientation(top_context)) == {
        "type": _type,
        "angle": get_angle(_type, natural)
    }

    # Reset screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[top_context["context"]], screen_orientation=None
    )

    # Assert screen orientation is the default.
    assert await get_screen_orientation(
        top_context) == default_screen_orientation


async def test_screen_orientation_change_event(
    bidi_session,
    new_tab,
    some_bidi_screen_orientation,
    some_web_screen_orientation,
    another_bidi_screen_orientation,
    another_web_screen_orientation,
    default_screen_orientation,
    subscribe_events,
    add_preload_script,
    wait_for_event,
    wait_for_future_safe,
    inline,
):
    await subscribe_events(["script.message"])
    await add_preload_script(
        function_declaration="""(channel) => {
            window.screen.orientation.addEventListener(
                "change",
                (e)=>channel({type: e.target.type, angle: e.target.angle})
            )
        }""",
        arguments=[{"type": "channel", "value": {"channel": "change_event"}}],
    )

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=inline("<div>foo</div>"), wait="complete"
    )

    on_script_message = wait_for_event("script.message")

    # Set screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[new_tab["context"]], screen_orientation=some_bidi_screen_orientation
    )

    event_data = await wait_for_future_safe(on_script_message)

    assert (
        remote_mapping_to_dict(event_data["data"]["value"])
        == some_web_screen_orientation
    )

    on_script_message = wait_for_event("script.message")

    # Set another screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[new_tab["context"]],
        screen_orientation=another_bidi_screen_orientation,
    )

    event_data = await wait_for_future_safe(on_script_message)

    assert (
        remote_mapping_to_dict(event_data["data"]["value"])
        == another_web_screen_orientation
    )

    on_script_message = wait_for_event("script.message")

    # Reset screen orientation override.
    await bidi_session.emulation.set_screen_orientation_override(
        contexts=[new_tab["context"]], screen_orientation=None
    )

    event_data = await wait_for_future_safe(on_script_message)

    assert (
        remote_mapping_to_dict(event_data["data"]["value"])
        == default_screen_orientation
    )
