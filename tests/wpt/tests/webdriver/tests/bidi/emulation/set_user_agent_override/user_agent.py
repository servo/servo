import pytest

pytestmark = pytest.mark.asyncio

SOME_USER_AGENT = "SOME_USER_AGENT"


async def test_user_agent_set_override_and_reset(bidi_session, top_context,
        default_user_agent, assert_user_agent):
    await assert_user_agent(top_context, default_user_agent)

    await bidi_session.emulation.set_user_agent_override(
        user_agent=SOME_USER_AGENT
    )
    await assert_user_agent(top_context, SOME_USER_AGENT)

    await bidi_session.emulation.set_user_agent_override(
        user_agent=None
    )

    await assert_user_agent(top_context, default_user_agent)
