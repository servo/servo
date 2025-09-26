import json

import pytest_asyncio

from webdriver.bidi.modules.script import ContextTarget


@pytest_asyncio.fixture
async def default_user_agent(top_context, get_navigator_user_agent):
    return await get_navigator_user_agent(top_context)


@pytest_asyncio.fixture
async def get_navigator_user_agent(bidi_session):
    async def get_navigator_user_agent(context):
        result = await bidi_session.script.evaluate(
            expression="window.navigator.userAgent",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

        return result["value"]

    return get_navigator_user_agent


@pytest_asyncio.fixture
async def assert_navigation_user_agent(bidi_session, url):
    """
    Helper to assert the right `user-agent` header sent in navigation request.
    """

    async def assert_navigation_user_agent(context, expected_user_agent):
        echo_link = url("webdriver/tests/support/http_handlers/headers_echo.py")
        await bidi_session.browsing_context.navigate(context=context["context"],
                                                     url=echo_link,
                                                     wait="complete")

        result = await bidi_session.script.evaluate(
            expression="window.document.body.textContent",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

        headers = (json.JSONDecoder().decode(result["value"]))["headers"]
        user_agent = headers["user-agent"][0]

        assert user_agent == expected_user_agent, \
            f"Navigation expected to send user agent '{expected_user_agent}' but sent '{user_agent}'"

    return assert_navigation_user_agent


@pytest_asyncio.fixture
async def assert_fetch_user_agent(bidi_session, url):
    """
    Helper to assert the right `user-agent` header sent in fetch.
    """

    async def assert_navigation_user_agent(context, expected_user_agent):
        echo_link = url("webdriver/tests/support/http_handlers/headers_echo.py")
        await bidi_session.browsing_context.navigate(context=context["context"],
                                                     url=echo_link,
                                                     wait="complete")

        result = await bidi_session.script.evaluate(
            expression=f"fetch('{echo_link}').then(r => r.text())",
            target=ContextTarget(context["context"]),
            await_promise=True,
        )

        headers = (json.JSONDecoder().decode(result["value"]))["headers"]
        user_agent = headers["user-agent"][0]

        assert user_agent == expected_user_agent, \
            f"Fetch expected to send user agent '{expected_user_agent}' but sent '{user_agent}'"

    return assert_navigation_user_agent


@pytest_asyncio.fixture
async def assert_user_agent(get_navigator_user_agent,
        assert_navigation_user_agent, assert_fetch_user_agent):
    """
    Helper to assert the right `user-agent` returned by navigator and sent in
    navigation and fetch.
    """

    async def assert_user_agent(context, expected_user_agent):
        assert await get_navigator_user_agent(context) == expected_user_agent, \
            "window.navigator.userAgent should be expected"
        await assert_navigation_user_agent(context, expected_user_agent)
        await assert_fetch_user_agent(context, expected_user_agent)

    return assert_user_agent
