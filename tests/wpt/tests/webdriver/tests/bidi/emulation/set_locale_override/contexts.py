import pytest

pytestmark = pytest.mark.asyncio


async def test_contexts(bidi_session, new_tab, top_context, get_current_locale,
        default_locale, some_locale):
    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=some_locale
    )

    # Assert locale emulated only in the required context.
    assert await get_current_locale(new_tab) == some_locale
    assert await get_current_locale(top_context) == default_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"]],
        locale=None)

    # Assert the locale is restored to the initial one.
    assert await get_current_locale(new_tab) == default_locale
    assert await get_current_locale(top_context) == default_locale


async def test_multiple_contexts(bidi_session, new_tab, get_current_locale,
        default_locale, some_locale):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"], new_context["context"]],
        locale=some_locale
    )

    # Assert locale emulated in all the required contexts.
    assert await get_current_locale(new_tab) == some_locale
    assert await get_current_locale(new_context) == some_locale

    # Reset locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[new_tab["context"], new_context["context"]],
        locale=None)

    # Assert the locale is restored to the initial one.
    assert await get_current_locale(new_tab) == default_locale
    assert await get_current_locale(new_context) == default_locale
