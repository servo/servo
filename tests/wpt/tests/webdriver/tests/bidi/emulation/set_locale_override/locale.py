import pytest

pytestmark = pytest.mark.asyncio


async def test_locale_set_override_and_reset(bidi_session, top_context,
        get_current_locale, default_locale, some_locale, another_locale):
    assert await get_current_locale(top_context) == default_locale

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[top_context["context"]],
        locale=some_locale
    )

    assert await get_current_locale(top_context) == some_locale

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[top_context["context"]],
        locale=another_locale
    )

    assert await get_current_locale(top_context) == another_locale

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[top_context["context"]],
        locale=None
    )

    assert await get_current_locale(top_context) == default_locale


@pytest.mark.parametrize("value", [
    # Simple language code (2-letter).
    "en",
    # Language and region (both 2-letter).
    "en-US",
    # Language and script (4-letter).
    "sr-Latn",
    # Language, script, and region.
    "zh-Hans-CN",
])
async def test_locale_values(bidi_session, top_context, get_current_locale,
        default_locale, value):
    assert await get_current_locale(top_context) == default_locale

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[top_context["context"]],
        locale=value
    )

    assert await get_current_locale(top_context) == value


@pytest.mark.parametrize("locale,expected_locale", [
    # Locale with Unicode extension keyword for collation.
    ("de-DE-u-co-phonebk", "de-DE"),
    # Lowercase language and region.
    ("fr-ca", "fr-CA"),
    # Uppercase language and region (should be normalized by Intl.Locale).
    ("FR-CA", "fr-CA"),
    # Mixed case language and region (should be normalized by Intl.Locale).
    ("fR-cA", "fr-CA"),
    # Locale with transform extension (simple case).
    ("en-t-zh", "en"),
])
async def test_locale_values_normalized_by_intl(bidi_session, top_context,
        get_current_locale,
        default_locale, locale, expected_locale):
    assert await get_current_locale(top_context) == default_locale

    # Set locale override.
    await bidi_session.emulation.set_locale_override(
        contexts=[top_context["context"]],
        locale=locale
    )

    assert await get_current_locale(top_context) == expected_locale
