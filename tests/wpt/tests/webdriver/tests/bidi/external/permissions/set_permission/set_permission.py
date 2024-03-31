import pytest
import webdriver.bidi.error as error

from . import get_context_origin, get_permission_state

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
async def test_set_permission_insecure_context(bidi_session, new_tab, url):
    test_url = url("/common/blank.html", protocol="http")
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    origin = await get_context_origin(bidi_session, new_tab)

    with pytest.raises(error.InvalidArgumentException):
      await bidi_session.permissions.set_permission(
          descriptor={"name": "push"},
          state="granted",
          origin=origin,
      )

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
async def test_set_permission_user_context(bidi_session, new_tab, url, create_user_context):
    test_url = url("/common/blank.html", protocol="https")

    user_context = await create_user_context()
    # new_tab is in the default user context. new_tab2 is in the non-default user context.
    new_tab2 = await bidi_session.browsing_context.create(type_hint="tab", user_context=user_context)

    # Navigate a context in the default user context.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=test_url,
        wait="complete",
    )

    # Navigate a context in the non-default user context.
    await bidi_session.browsing_context.navigate(
        context=new_tab2["context"],
        url=test_url,
        wait="complete",
    )

    origin = await get_context_origin(bidi_session, new_tab)

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"
    assert await get_permission_state(bidi_session, new_tab2, "geolocation") == "prompt"

    await bidi_session.permissions.set_permission(
        descriptor={"name": "geolocation"},
        state="granted",
        origin=origin,
        user_context=user_context,
    )

    assert await get_permission_state(bidi_session, new_tab, "geolocation") == "prompt"
    assert await get_permission_state(bidi_session, new_tab2, "geolocation") == "granted"
