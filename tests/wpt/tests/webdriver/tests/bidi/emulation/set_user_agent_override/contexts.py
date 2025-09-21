import pytest

pytestmark = pytest.mark.asyncio

SOME_USER_AGENT = "SOME_USER_AGENT"


async def test_contexts(bidi_session, new_tab, top_context,
        default_user_agent, assert_user_agent):
    # Set user-agent override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[new_tab["context"]],
        user_agent=SOME_USER_AGENT
    )

    # Assert user-agent override is set only in the required context.
    await assert_user_agent(new_tab, SOME_USER_AGENT)
    await assert_user_agent(top_context, default_user_agent)

    # Reset user-agent override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[new_tab["context"]],
        user_agent=None
    )

    # Assert user-agent override is reset.
    await assert_user_agent(new_tab, default_user_agent)
    await assert_user_agent(top_context, default_user_agent)


async def test_multiple_contexts(bidi_session, new_tab, default_user_agent,
        assert_user_agent):
    new_context = await bidi_session.browsing_context.create(type_hint="tab")

    # Set user-agent override
    await bidi_session.emulation.set_user_agent_override(
        contexts=[new_tab["context"], new_context["context"]],
        user_agent=SOME_USER_AGENT
    )

    # Assert user-agent override is set in all the required contexts.
    await assert_user_agent(new_tab, SOME_USER_AGENT)
    await assert_user_agent(new_context, SOME_USER_AGENT)

    # Reset user-agent override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[new_tab["context"], new_context["context"]],
        user_agent=None
    )

    # Assert user-agent override is reset.
    await assert_user_agent(new_tab, default_user_agent)
    await assert_user_agent(new_context, default_user_agent)


@pytest.mark.parametrize("domain", ["", "alt"],
                         ids=["same_origin", "cross_origin"])
async def test_iframe(bidi_session, new_tab, default_user_agent,
        assert_user_agent, domain, inline):
    # Set user-agent override
    await bidi_session.emulation.set_user_agent_override(
        contexts=[new_tab["context"]],
        user_agent=SOME_USER_AGENT
    )

    # Assert user-agent override is set in the required context.
    await assert_user_agent(new_tab, SOME_USER_AGENT)

    iframe_url = inline("<div id='in-iframe'>foo</div>", domain=domain)
    page_url = inline(f"<iframe src='{iframe_url}'></iframe>")

    # Load the page with iframes.
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"],
        url=page_url,
        wait="complete",
    )

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"])
    iframe = contexts[0]["children"][0]

    # Assert user-agent override is set in the iframe context.
    await assert_user_agent(iframe, SOME_USER_AGENT)

    # Reset user-agent override.
    await bidi_session.emulation.set_user_agent_override(
        contexts=[new_tab["context"]],
        user_agent=None
    )

    # Assert user-agent override is reset in the iframe context.
    await assert_user_agent(iframe, default_user_agent)
