import pytest

from .... import get_context_origin
from . import get_permission_state

pytestmark = pytest.mark.asyncio


@pytest.mark.asyncio
async def test_set_permission(bidi_session, new_tab, url):
    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    origin = await get_context_origin(bidi_session, new_tab)

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="granted",
        origin=origin,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "granted"

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="denied",
        origin=origin,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "denied"

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="prompt",
        origin=origin,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"


@pytest.mark.asyncio
async def test_set_permission_new_context(bidi_session, new_tab, url):
    test_url = url("/common/blank.html", protocol="https")

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    origin = await get_context_origin(bidi_session, new_tab)

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="granted",
        origin=origin,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "granted"

    new_context = await bidi_session.browsing_context.create(type_hint="tab")
    assert new_tab["context"] != new_context["context"]
    await bidi_session.browsing_context.navigate(
        context=new_context["context"],
        url=test_url,
        wait="complete",
    )

    # See https://github.com/w3c/permissions/issues/437.
    assert await get_permission_state(bidi_session, new_context, "geolocation") == "granted"


@pytest.mark.parametrize("origin", ['UNKNOWN', ''])
async def test_set_permission_origin_unknown(bidi_session, new_tab, origin, url):
    test_url = url("/common/blank.html", protocol="https")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    # Ensure permission for the tab is prompt.
    tab_origin = await get_context_origin(bidi_session, new_tab)
    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="prompt",
        origin=tab_origin,
    )

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="granted",
        origin=origin,
    )
    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"


@pytest.mark.asyncio
async def test_set_permission_iframe(
    bidi_session, new_tab, test_page_cross_origin_frame
):
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_page_cross_origin_frame,
        wait="complete",
    )
    contexts = await bidi_session.browsing_context.get_tree(root=new_tab["context"])

    frames = contexts[0]["children"]
    assert len(frames) == 1
    iframe_context_id = frames[0]["context"]
    assert iframe_context_id
    iframe_context = {"context": iframe_context_id}
    iframe_orgin = await get_context_origin(bidi_session, iframe_context)

    # Ensure the initial permission for the frame is prompt.
    assert (
        await get_permission_state(bidi_session, iframe_context, "storage-access")
        == "prompt"
    )

    # Set permissions for the top-level context's origin.
    tab_origin = await get_context_origin(bidi_session, new_tab)
    await bidi_session.permissions.set_permission(
        descriptor={"name": "storage-access"},
        state="prompt",
        origin=tab_origin,
    )
    # Assert the frame's permission is still prompt.
    assert (
        await get_permission_state(bidi_session, iframe_context, "storage-access")
        == "prompt"
    )

    # Set permissions for the frame's origin.
    await bidi_session.permissions.set_permission(
        descriptor={"name": "storage-access"},
        state="granted",
        origin=tab_origin,
        embedded_origin=iframe_orgin,
    )
    assert (
        await get_permission_state(bidi_session, iframe_context, "storage-access")
        == "granted"
    )
