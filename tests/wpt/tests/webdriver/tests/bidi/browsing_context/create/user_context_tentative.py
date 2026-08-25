import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_create_result_custom_user_context(bidi_session, create_user_context, user_context):
    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )
    new_context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )
    assert "userContext" in new_context
    assert new_context["userContext"] == user_context_id
    await bidi_session.browsing_context.close(context=new_context["context"])
