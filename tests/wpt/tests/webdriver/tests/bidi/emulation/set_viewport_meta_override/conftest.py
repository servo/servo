import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget

CUSTOM_VIEWPORT_WIDTH = 345


@pytest_asyncio.fixture
async def is_viewport_meta_active(bidi_session):
    """Returns a helper that checks whether `<meta name="viewport">` parsing
    is active in a given browsing context."""

    async def _is_viewport_meta_active(context):
        # Dynamically insert `<meta name="viewport" content="width=345">` into
        # the document if not present. When viewport meta tag parsing is active,
        # the layout viewport resizes immediately to match the specified width
        # (345px), which is reflected by `document.documentElement.clientWidth`.
        result = await bidi_session.script.call_function(
            function_declaration=f"""() => {{
                let meta = document.querySelector('meta[name="viewport"]');
                if (!meta) {{
                    meta = document.createElement('meta');
                    meta.name = 'viewport';
                    meta.content = 'width={CUSTOM_VIEWPORT_WIDTH}';
                    document.head.appendChild(meta);
                }}
                return document.documentElement.clientWidth === {CUSTOM_VIEWPORT_WIDTH};
            }}""",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )
        return result["value"]

    return _is_viewport_meta_active


@pytest_asyncio.fixture
async def default_viewport_meta_state(top_context, is_viewport_meta_active):
    """Returns whether `<meta name="viewport">` parsing is active by default."""
    return await is_viewport_meta_active(top_context)


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
