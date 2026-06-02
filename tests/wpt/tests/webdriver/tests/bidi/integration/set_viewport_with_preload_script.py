import pytest

from .. import remote_mapping_to_dict

pytestmark = pytest.mark.asyncio


async def test_order(
    bidi_session,
    add_preload_script,
    create_user_context,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    await subscribe_events(["script.message"])
    test_viewport = {"width": 250, "height": 300}

    user_context = await create_user_context()

    await add_preload_script(
        function_declaration="""(channel) => {
        channel({
            height: window.innerHeight,
            width: window.innerWidth,
        });
   }""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
    )

    on_message = wait_for_event("script.message")

    await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    message = await wait_for_future_safe(on_message)
    viewport = remote_mapping_to_dict(message["data"]["value"])

    assert test_viewport != viewport

    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )

    on_message = wait_for_event("script.message")
    await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    message = await wait_for_future_safe(on_message)
    viewport = remote_mapping_to_dict(message["data"]["value"])

    # Make sure that the preload script runs after viewport settings are updated.
    assert test_viewport == viewport
