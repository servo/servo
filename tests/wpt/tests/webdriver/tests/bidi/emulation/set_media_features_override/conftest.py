import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

SOME_FEATURE_NAME = "prefers-contrast"
SOME_FEATURE_SOME_VALUE = "more"

ANOTHER_FEATURE_NAME = "prefers-reduced-motion"
ANOTHER_FEATURE_SOME_VALUE = "reduce"


@pytest_asyncio.fixture
async def is_feature_enabled(bidi_session):
    """Returns a helper that checks whether a CSS media feature value matches
    in a given browsing context."""

    async def _is_feature_enabled(context, feature, value):
        result = await bidi_session.script.call_function(
            function_declaration="(feature, value) => window.matchMedia(`(${feature}: ${value})`).matches",
            arguments=[
                {"type": "string", "value": feature},
                {"type": "string", "value": str(value)},
            ],
            target=ContextTarget(context["context"]),
            await_promise=False,
        )
        return result["value"]

    return _is_feature_enabled


@pytest.fixture(
    params=["default", "new"],
    ids=["Default user context", "Custom user context"],
)
def target_user_context(request):
    return request.param


@pytest_asyncio.fixture
async def affected_user_context(target_user_context, create_user_context):
    if target_user_context == "default":
        return "default"
    return await create_user_context()


@pytest_asyncio.fixture
async def not_affected_user_context(target_user_context, create_user_context):
    if target_user_context == "new":
        return "default"
    return await create_user_context()
