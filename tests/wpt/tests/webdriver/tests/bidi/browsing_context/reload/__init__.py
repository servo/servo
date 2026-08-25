import pytest

from webdriver.bidi.error import UnknownErrorException

from ... import any_string


async def reload_and_assert(
    bidi_session,
    context,
    expected_error=False,
    last_navigation=None,
    url=None,
    wait="complete",
):
    if expected_error:
        with pytest.raises(UnknownErrorException):
            await bidi_session.browsing_context.reload(
                context=context['context'], wait=wait
            )

    else:
        result = await bidi_session.browsing_context.reload(
            context=context['context'], wait=wait
        )

        any_string(result["navigation"])
        any_string(result["url"])

        if last_navigation is not None:
            assert result["navigation"] != last_navigation

        if url is not None:
            assert result["url"] == url

            contexts = await bidi_session.browsing_context.get_tree(root=context['context'])
            assert len(contexts) == 1
            assert contexts[0]["url"] == url

        return result
