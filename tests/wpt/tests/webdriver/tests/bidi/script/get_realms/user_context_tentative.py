import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("user_context", ["default", "new"])
async def test_user_context(bidi_session, create_user_context, user_context):
    user_context_id = (
        await create_user_context() if user_context == "new" else user_context
    )

    context = await bidi_session.browsing_context.create(
        type_hint="tab", user_context=user_context_id
    )

    result = await bidi_session.script.get_realms(context=context["context"])

    window_realms = [r for r in result if r.get("type") == "window"]
    assert len(window_realms) > 0
    realm = window_realms[0]
    assert "userContext" in realm
    assert realm["userContext"] == user_context_id
