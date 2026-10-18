import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def get_scrollbar_width(bidi_session):
    async def get_scrollbar_width(context):
        result = await bidi_session.script.call_function(
            function_declaration="""() => {
                const outer = document.createElement('div');
                outer.style.visibility = 'hidden';
                outer.style.overflow = 'scroll';
                document.body.appendChild(outer);

                const width = outer.offsetWidth - outer.clientWidth;
                outer.remove();

                return width;
            }""",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )
        return result["value"]

    return get_scrollbar_width


@pytest_asyncio.fixture
async def default_scrollbar_width(top_context, get_scrollbar_width):
    """Return the baseline scrollbar width in an unmodified context."""
    return await get_scrollbar_width(top_context)


@pytest_asyncio.fixture(
    params=["default", "new"], ids=["Default user context", "Custom user context"]
)
async def target_user_context(request):
    return request.param


@pytest_asyncio.fixture
async def affected_user_context(target_user_context, create_user_context):
    """Returns either a new or default user context."""
    if target_user_context == "default":
        return "default"
    return await create_user_context()


@pytest_asyncio.fixture
async def not_affected_user_context(target_user_context, create_user_context):
    """Returns opposite to affected_user_context."""
    if target_user_context == "new":
        return "default"
    return await create_user_context()
