import pytest

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("type_hint", ["window", "tab"])
async def test_top_level_context(bidi_session, type_hint):
    top_level_context = await bidi_session.browsing_context.create(
        type_hint=type_hint
    )

    contexts = await bidi_session.browsing_context.get_tree()
    assert len(contexts) == 2

    await bidi_session.browsing_context.close(context=top_level_context["context"])

    contexts = await bidi_session.browsing_context.get_tree()
    assert len(contexts) == 1

    assert contexts[0]["context"] != top_level_context["context"]

    # TODO: Add a test for closing the last tab once the behavior has been specified
    # https://github.com/w3c/webdriver-bidi/issues/187
