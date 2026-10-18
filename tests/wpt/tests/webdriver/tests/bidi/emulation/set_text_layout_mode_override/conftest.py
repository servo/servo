import pytest
import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def is_text_autosizing_enabled(bidi_session):
    """Returns a helper that checks whether mobile text layout mode (CSS
    `text-size-adjust`) is active in a given browsing context."""

    async def _is_text_autosizing_enabled(context):
        result = await bidi_session.script.call_function(
            function_declaration="""() => {
                // Use a unique z-index per call so repeated checks on the same
                // document do not hit cached matched properties.
                window.__textLayoutCheckCount =
                    (window.__textLayoutCheckCount || 0) + 1;
                const unadjusted = document.createElement('div');
                unadjusted.style.fontSize = '16px';
                unadjusted.style.textSizeAdjust = 'none';
                unadjusted.style.zIndex = String(window.__textLayoutCheckCount);
                const adjusted = document.createElement('div');
                adjusted.style.fontSize = '16px';
                adjusted.style.textSizeAdjust = '200%';
                adjusted.style.zIndex = String(window.__textLayoutCheckCount);
                document.body.append(unadjusted, adjusted);
                const unadjustedSize = parseFloat(
                    getComputedStyle(unadjusted).fontSize
                );
                const adjustedSize = parseFloat(
                    getComputedStyle(adjusted).fontSize
                );
                unadjusted.remove();
                adjusted.remove();
                return adjustedSize === unadjustedSize * 2;
            }""",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )
        return result["value"]

    return _is_text_autosizing_enabled


@pytest_asyncio.fixture
async def default_text_autosizing_state(
    top_context,
    is_text_autosizing_enabled,
):
    """Returns whether text autosizing (`text-size-adjust`) is active by default."""
    return await is_text_autosizing_enabled(top_context)


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
