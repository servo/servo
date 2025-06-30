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
    # Simple language code (3-letter ISO 639-2/3).
    "ast",
    # Language and region (both 2-letter).
    "en-US",
    # Language and script (4-letter).
    "sr-Latn",
    # Language, script, and region.
    "zh-Hans-CN",
    # Language and variant (longer variant).
    "de-DE-1996",
    # Language and multiple variants.
    "sl-Roza-biske",
    # Language, region, and variant.
    "ca-ES-valencia",
    # Language and variant (4-char variant starting with digit).
    "sl-1994",
    # Locale with Unicode extension keyword for numbering system.
    "th-TH-u-nu-thai",
    # Locale with Unicode extension for calendar.
    "en-US-u-ca-gregory",
    # Canonical extended language subtag (Yue Chinese).
    "yue",
    # Canonical extended language subtag (North Levantine Arabic).
    "apc",
    # Language with a less common but valid 3-letter code.
    "gsw",
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
    # A complex but valid tag with multiple subtags including extension and private use.
    ("zh-Latn-CN-variant1-a-extend1-u-co-pinyin-x-private",
     "zh-Latn-CN-variant1"),
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
    # Language (2-letter) and region (3-digit UN M49).
    ("es-419", "es-MX"),
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
