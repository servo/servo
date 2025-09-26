import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

LOCALES = ["de-DE", "es-ES", "fr-FR", "it-IT"]


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
