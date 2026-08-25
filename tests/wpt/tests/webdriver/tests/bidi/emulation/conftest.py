import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

LOCALES = ["de-DE", "es-ES", "fr-FR", "it-IT"]
TIMEZONES = ["Asia/Yekaterinburg", "Europe/Berlin", "America/New_York", "Asia/Tokyo"]


@pytest_asyncio.fixture
async def get_current_locale(bidi_session):
    async def get_current_locale(context, sandbox=None):
        result = await bidi_session.script.evaluate(
            expression="new Intl.DateTimeFormat().resolvedOptions().locale",
            target=ContextTarget(context["context"], sandbox=sandbox),
            await_promise=False,
        )

        return result["value"]

    return get_current_locale


@pytest_asyncio.fixture
async def default_locale(get_current_locale, top_context):
    """
    Returns default locale.
    """
    return await get_current_locale(top_context)


@pytest.fixture
def some_locale(default_locale):
    """
    Returns some locale which is not equal to `default_locale`.
    """
    for locale in LOCALES:
        if locale != default_locale:
            return locale

    raise Exception(
        f"Unexpectedly could not find locale different from the default {default_locale}"
    )


@pytest.fixture
def another_locale(default_locale, some_locale):
    """
    Returns some another locale which is not equal to `default_locale` nor to
    `some_locale`.
    """
    for locale in LOCALES:
        if locale != default_locale and locale != some_locale:
            return locale

    raise Exception(
        f"Unexpectedly could not find locale different from the default {default_locale} and {some_locale}"
    )


@pytest_asyncio.fixture
async def get_current_navigator_language(bidi_session):
    async def get_current_navigator_language(context, sandbox=None):
        result = await bidi_session.script.evaluate(
            expression="navigator.language",
            target=ContextTarget(context["context"], sandbox=sandbox),
            await_promise=False,
        )

        return result["value"]

    return get_current_navigator_language


@pytest_asyncio.fixture
async def default_navigator_language(get_current_navigator_language, top_context):
    """Returns default navigator.language value."""
    return await get_current_navigator_language(top_context)


@pytest_asyncio.fixture
async def get_current_navigator_languages(bidi_session):
    async def get_current_navigator_languages(context, sandbox=None):
        result = await bidi_session.script.evaluate(
            expression="navigator.languages",
            target=ContextTarget(context["context"], sandbox=sandbox),
            await_promise=False,
        )

        return [item["value"] for item in result["value"]]

    return get_current_navigator_languages


@pytest_asyncio.fixture
async def default_navigator_languages(get_current_navigator_languages, top_context):
    """Returns default navigator.languages value."""
    return await get_current_navigator_languages(top_context)


@pytest_asyncio.fixture
async def default_accept_language(bidi_session, get_fetch_headers, top_context, url):
    """
    Returns default value of `Accept-Language` header.
    """
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=url("/webdriver/tests/bidi/browsing_context/support/empty.html"),
        wait="complete"
    )
    headers = await get_fetch_headers(top_context)
    return headers["accept-language"][0] if "accept-language" in headers else None


@pytest_asyncio.fixture
async def assert_accept_language(bidi_session, assert_header_present, url):
    """
    Assert value of `Accept-Language` header.
    """
    async def assert_accept_language(context, value, sandbox_name=None):
        await bidi_session.browsing_context.navigate(
            context=context["context"],
            url=url("/webdriver/tests/bidi/browsing_context/support/empty.html"),
            wait="complete"
        )
        await assert_header_present(context, "accept-language", value, sandbox_name)

    return assert_accept_language


@pytest_asyncio.fixture
async def assert_locale_against_default(
    top_context,
    assert_accept_language,
    default_locale,
    default_navigator_language,
    default_navigator_languages,
    default_accept_language,
    get_current_locale,
    get_current_navigator_language,
    get_current_navigator_languages,
):
    """
    Assert JS locale and navigator.language/s against default values.

    Note: it's possible to have slightly different values between JS locale and
    navigator.language/s, that's why we have to retrieve the default values
    for each API.
    """

    async def assert_locale_against_default(context, sandbox_name=None):
        assert (
            context != top_context
        ), "Provided context should be different from top_context"

        assert await get_current_locale(context, sandbox_name) == default_locale
        assert (
            await get_current_navigator_language(context, sandbox_name)
            == default_navigator_language
        )
        assert (
            await get_current_navigator_languages(context, sandbox_name)
            == default_navigator_languages
        )
        await assert_accept_language(context, default_accept_language, sandbox_name)

    return assert_locale_against_default


@pytest_asyncio.fixture
async def assert_locale_against_value(
    top_context,
    assert_accept_language,
    get_current_locale,
    get_current_navigator_language,
    get_current_navigator_languages,
):
    """
    Assert JS locale and navigator.language/s against provided value
    """

    async def assert_locale_against_value(value, context, sandbox_name=None):
        assert (
            context != top_context
        ), "Provided context should be different from top_context"

        assert await get_current_locale(context, sandbox_name) == value
        assert await get_current_navigator_language(context, sandbox_name) == value
        assert await get_current_navigator_languages(context, sandbox_name) == [value]
        await assert_accept_language(context, value)

    return assert_locale_against_value


@pytest_asyncio.fixture
async def get_current_timezone(bidi_session):
    async def get_current_timezone(context, sandbox=None):
        result = await bidi_session.script.evaluate(
            expression="Intl.DateTimeFormat().resolvedOptions().timeZone",
            target=ContextTarget(context["context"], sandbox=sandbox),
            await_promise=False,
        )
        return result["value"]

    return get_current_timezone


@pytest_asyncio.fixture
async def default_timezone(get_current_timezone, top_context):
    """
    Returns default timezone.
    """
    return await get_current_timezone(top_context)


@pytest.fixture
def some_timezone(default_timezone):
    """
    Returns some timezone which is not equal to `default_timezone`.
    """
    for timezone in TIMEZONES:
        if timezone != default_timezone:
            return timezone

    raise Exception(
        f"Unexpectedly could not find timezone different from the default {default_timezone}"
    )


@pytest.fixture
def another_timezone(default_timezone, some_timezone):
    """
    Returns some another timezone which is not equal to `default_timezone` nor to
    `some_timezone`.
    """
    for timezone in TIMEZONES:
        if timezone != default_timezone and timezone != some_timezone:
            return timezone

    raise Exception(
        f"Unexpectedly could not find timezone different from the default {default_timezone} and {some_timezone}"
    )
