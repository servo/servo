import pytest_asyncio
from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def get_max_touch_points(bidi_session):
    async def get_max_touch_points(context):
        result = await bidi_session.script.evaluate(
            expression="navigator.maxTouchPoints",
            target=ContextTarget(context["context"]),
            await_promise=True,
        )
        return result["value"]

    return get_max_touch_points


@pytest_asyncio.fixture
async def initial_max_touch_points(top_context, get_max_touch_points):
    """Return the initial value of maxTouchPoints."""
    return await get_max_touch_points(top_context)


@pytest_asyncio.fixture(params=['default', 'new'],
                        ids=["Default user context", "Custom user context"])
async def target_user_context(request):
    return request.param


@pytest_asyncio.fixture
async def affected_user_context(target_user_context, create_user_context):
    """ Returns either a new or default user context. """
    if target_user_context == 'default':
        return 'default'
    return await create_user_context()


@pytest_asyncio.fixture
async def not_affected_user_context(target_user_context, create_user_context):
    """ Returns opposite to `affected_user_context user context. """
    if target_user_context == 'new':
        return 'default'
    return await create_user_context()
