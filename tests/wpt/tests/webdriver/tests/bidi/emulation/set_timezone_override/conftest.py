import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def get_timezone_offset(bidi_session):
    async def get_timezone_offset(timestamp, context):
        result = await bidi_session.script.evaluate(
            expression=f"(new Date({timestamp})).getTimezoneOffset()",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )
        return result["value"]

    return get_timezone_offset
