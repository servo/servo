import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

TIMEZONES = [
    "Asia/Yekaterinburg", "Europe/Berlin", "America/New_York", "Asia/Tokyo"
]


@pytest_asyncio.fixture
async def get_current_timezone(bidi_session):
    async def get_current_timezone(context):
        result = await bidi_session.script.evaluate(
            expression="Intl.DateTimeFormat().resolvedOptions().timeZone",
            target=ContextTarget(context["context"]),
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
    Returns some timezone which is not equal to `default_timezone` nor to
    `another_timezone`.
    """
    for timezone in TIMEZONES:
        if timezone != default_timezone:
            return timezone

    raise Exception(
        f"Unexpectedly could not find timezone different from the default {default_timezone}")


@pytest.fixture
def another_timezone(default_timezone, some_timezone):
    """
    Returns some another timezone which is not equal to `default_timezone` nor to
    `some_timezone`.
    """
    for timezone in TIMEZONES:
        if timezone != default_timezone and timezone != another_timezone:
            return timezone

    raise Exception(
        f"Unexpectedly could not find timezone different from the default {default_timezone}")
