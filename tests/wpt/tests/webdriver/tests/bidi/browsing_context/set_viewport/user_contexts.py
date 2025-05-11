import pytest
from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.undefined import UNDEFINED

from ... import get_device_pixel_ratio, get_viewport_dimensions

pytestmark = pytest.mark.asyncio

CONTEXT_LOAD_EVENT = "browsingContext.load"


async def test_set_to_user_context(bidi_session, new_tab, create_user_context):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    test_viewport = {"width": 250, "height": 300}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )

    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )

    # Make sure that the viewport changes are only applied to the context associated with user context
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )
    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport

    # Create a new context in the user context
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Make sure that the viewport changes are also applied
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        == test_viewport
    )

    # Create a new context in the default context
    context_in_default_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    # Make sure that the viewport changes are not applied
    assert (
        await get_viewport_dimensions(bidi_session, context_in_default_context)
        != test_viewport
    )


async def test_set_to_user_context_window_open(
    bidi_session,
    new_tab,
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
    result = await bidi_session.script.evaluate(
        await_promise=False,
        expression=f"""window.open('{inline("popup")}')""",
        target=ContextTarget(context_in_user_context_1["context"]),
    )
    event = await wait_for_future_safe(on_load)

    contexts = await bidi_session.browsing_context.get_tree(root=event["context"])
    assert len(contexts) == 1
    popup_context = contexts[0]

    assert await get_viewport_dimensions(bidi_session, popup_context) == test_viewport


async def test_set_to_default_user_context(bidi_session, new_tab, create_user_context):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    test_viewport = {"width": 250, "height": 300}

    assert await get_viewport_dimensions(bidi_session, new_tab) != test_viewport
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )

    await bidi_session.browsing_context.set_viewport(
        user_contexts=["default"], viewport=test_viewport
    )

    # Make sure that the viewport changes are only applied to the context associated with default user context
    assert await get_viewport_dimensions(bidi_session, new_tab) == test_viewport
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )

    # Create a new context in the default context
    context_in_default_context = await bidi_session.browsing_context.create(
        type_hint="tab"
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_default_context)
        == test_viewport
    )

    # Reset viewport settings
    await bidi_session.browsing_context.set_viewport(
        user_contexts=["default"], viewport=None
    )


async def test_set_to_multiple_user_contexts(bidi_session, create_user_context):
    user_context_1 = await create_user_context()
    user_context_2 = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context_1, type_hint="tab"
    )
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context_2, type_hint="tab"
    )

    test_viewport = {"width": 250, "height": 300}

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        != test_viewport
    )

    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context_1, user_context_2], viewport=test_viewport
    )

    # Make sure that the viewport changes are applied to both user contexts
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        == test_viewport
    )


async def test_undefined_viewport(bidi_session, inline, create_user_context):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    test_viewport = {"width": 499, "height": 599}

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )

    # Load a page so that reflow is triggered when changing the viewport
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_1["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=UNDEFINED
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    # Create another context in updated user context
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        == test_viewport
    )


async def test_reset_to_default(bidi_session, inline, create_user_context):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    original_viewport = await get_viewport_dimensions(
        bidi_session, context_in_user_context_1
    )

    test_viewport = {"width": 666, "height": 333}

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )

    # Load a page so that reflow is triggered when changing the viewport
    url = inline("<div>foo</div>")
    await bidi_session.browsing_context.navigate(
        context=context_in_user_context_1["context"], url=url, wait="complete"
    )

    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=None
    )
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == original_viewport
    )

    # Create another context in updated user context
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        == original_viewport
    )


async def test_set_viewport_and_device_pixel_ratio(bidi_session, create_user_context):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    test_viewport = {"width": 250, "height": 300}

    # Set the viewport changes
    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    # Create a new context in the user context
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    # Make sure that the viewport changes are also applied
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        == test_viewport
    )

    # Set the device pixel ratio changes
    test_device_pixel_ratio = 2
    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], device_pixel_ratio=test_device_pixel_ratio
    )

    assert (
        await get_device_pixel_ratio(bidi_session, context_in_user_context_1)
        == test_device_pixel_ratio
    )
    assert (
        await get_device_pixel_ratio(bidi_session, context_in_user_context_2)
        == test_device_pixel_ratio
    )

    # Create a new context in the user context
    context_in_user_context_3 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    assert (
        await get_device_pixel_ratio(bidi_session, context_in_user_context_3)
        == test_device_pixel_ratio
    )
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_3)
        == test_viewport
    )


async def test_set_to_user_context_and_then_to_context(
    bidi_session, create_user_context
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    test_viewport = {"width": 250, "height": 300}

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )

    # Apply viewport dimensions to the user context.
    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    new_test_viewport = {"width": 100, "height": 100}
    # Apply viewport dimensions now only to the context
    await bidi_session.browsing_context.set_viewport(
        context=context_in_user_context_1["context"], viewport=new_test_viewport
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == new_test_viewport
    )

    await bidi_session.browsing_context.reload(
        context=context_in_user_context_1["context"], wait="complete"
    )

    # Make sure that after reload the viewport dimensions are still updated.
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == new_test_viewport
    )

    # Create a new context in the user context
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the viewport settings for the user context are applied
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        == test_viewport
    )


async def test_set_viewport_to_user_context_and_then_device_pixel_ratio_to_context(
    bidi_session, create_user_context
):
    user_context = await create_user_context()
    context_in_user_context_1 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )

    original_dpr = await get_device_pixel_ratio(bidi_session, context_in_user_context_1)
    test_dpr = original_dpr + 1

    test_viewport = {"width": 250, "height": 300}

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        != test_viewport
    )

    # Apply viewport dimensions to the user context.
    await bidi_session.browsing_context.set_viewport(
        user_contexts=[user_context], viewport=test_viewport
    )

    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    # Apply viewport device pixel ratio now only to the context.
    await bidi_session.browsing_context.set_viewport(
        context=context_in_user_context_1["context"], device_pixel_ratio=test_dpr
    )

    assert (
        await get_device_pixel_ratio(bidi_session, context_in_user_context_1)
        == test_dpr
    )
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_1)
        == test_viewport
    )

    # Create a new context in the user context
    context_in_user_context_2 = await bidi_session.browsing_context.create(
        user_context=user_context, type_hint="tab"
    )
    # Make sure that the viewport settings for the user context are applied
    assert (
        await get_viewport_dimensions(bidi_session, context_in_user_context_2)
        == test_viewport
    )
    assert (
        await get_device_pixel_ratio(bidi_session, context_in_user_context_2)
        == original_dpr
    )
