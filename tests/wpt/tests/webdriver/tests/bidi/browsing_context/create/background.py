import asyncio
import pytest

pytestmark = pytest.mark.asyncio

from .. import get_document_focus, get_visibility_state


@pytest.mark.parametrize("type_hint", ["tab", "window"])
async def test_background_default_false(bidi_session, type_hint):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint)

    try:
        assert await get_visibility_state(bidi_session, new_context) == "visible"
        assert await get_document_focus(bidi_session, new_context) is True
    finally:
        await bidi_session.browsing_context.close(context=new_context["context"])


@pytest.mark.parametrize("type_hint", ["tab", "window"])
@pytest.mark.parametrize("background", [True, False])
async def test_background(bidi_session, top_context, type_hint, background):
    new_context = await bidi_session.browsing_context.create(type_hint=type_hint, background=background)

    try:
        if background:
            assert await get_visibility_state(bidi_session, top_context) == "visible"
        else:
            assert await get_visibility_state(bidi_session, new_context) == "visible"

        assert await get_document_focus(bidi_session, new_context) != background
    finally:
        await bidi_session.browsing_context.close(context=new_context["context"])


@pytest.mark.parametrize("type_hint", ["tab", "window"])
@pytest.mark.parametrize("background", [True, False])
async def test_create_in_parallel(
    bidi_session, top_context, wait_for_future_safe, type_hint, background
):
    # Create 2 browsing contexts in quick succession, without waiting for
    # the individual commands to resolve.
    context_task_1 = asyncio.create_task(
        bidi_session.browsing_context.create(type_hint="tab", background=background)
    )
    context_task_2 = asyncio.create_task(
        bidi_session.browsing_context.create(type_hint="tab", background=background)
    )

    # Wait for both contexts to be created successfully
    context_1 = await wait_for_future_safe(context_task_1)
    context_2 = await wait_for_future_safe(context_task_2)

    try:
        if background:
            # if background was true, the initial tab should still be selected
            assert await get_visibility_state(bidi_session, top_context) == "visible"
            assert await get_document_focus(bidi_session, top_context)
        else:
            # otherwise either context 1 or 2 might end up with the visibility and focus.
            context_1_focus = await get_document_focus(bidi_session, context_1)
            context_2_focus = await get_document_focus(bidi_session, context_2)
            assert context_1_focus or context_2_focus

            context_1_visible = await get_visibility_state(bidi_session, context_1) == "visible"
            context_2_visible = await get_visibility_state(bidi_session, context_2) == "visible"
            assert context_1_visible or context_2_visible

    finally:
        await bidi_session.browsing_context.close(context=context_1["context"])
        await bidi_session.browsing_context.close(context=context_2["context"])