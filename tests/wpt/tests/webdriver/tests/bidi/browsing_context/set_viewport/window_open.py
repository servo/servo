import pytest
from webdriver.bidi.modules.script import ContextTarget

from ... import get_viewport_dimensions, remote_mapping_to_dict

pytestmark = pytest.mark.asyncio

CONTEXT_LOAD_EVENT = "browsingContext.load"


async def test_set_to_user_context_window_open(
    bidi_session,
    create_user_context,
    inline,
    subscribe_events,
    wait_for_event,
    wait_for_future_safe,
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    test_viewport = {"width": 250, "height": 300}
    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    await subscribe_events(events=[CONTEXT_LOAD_EVENT])

    # Assert that tabs opened via window.open in the same user context
    # successfully load and have the right viewport set.
    on_load = wait_for_event(CONTEXT_LOAD_EVENT)
    result = await bidi_session.script.call_function(
        await_promise=False,
        function_declaration=f"""() => {{
            const win = window.open('{inline("popup")}');
            return {{ width: win.innerWidth, height: win.innerHeight }};
        }}""",
        target=ContextTarget(context_in_user_context_1["context"]),
    )

    assert remote_mapping_to_dict(result["value"]) == test_viewport

    event = await wait_for_future_safe(on_load)

    contexts = await bidi_session.browsing_context.get_tree(root=event["context"])
    assert len(contexts) == 1
    popup_context = contexts[0]

    assert await get_viewport_dimensions(bidi_session, popup_context) == test_viewport
