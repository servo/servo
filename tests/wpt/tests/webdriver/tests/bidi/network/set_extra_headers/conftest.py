import json

import pytest_asyncio

import webdriver.bidi.error as error
from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.undefined import UNDEFINED


@pytest_asyncio.fixture
async def set_extra_headers(bidi_session):
    extra_headers = []

    async def set_extra_headers(headers, contexts=UNDEFINED,
            user_contexts=UNDEFINED):
        extra_headers.append((contexts, user_contexts))
        await bidi_session.network.set_extra_headers(
            headers=headers, contexts=contexts, user_contexts=user_contexts)

    yield set_extra_headers

    for (contexts, user_contexts) in extra_headers:
        try:
            await bidi_session.network.set_extra_headers(
                headers=[], contexts=contexts, user_contexts=user_contexts)
        except (
                error.InvalidArgumentException,
                error.NoSuchUserContextException,
                error.NoSuchFrameException):
            pass


@pytest_asyncio.fixture
async def prepare_context(bidi_session, url):
    async def prepare_context(context, domain=""):
        await bidi_session.browsing_context.navigate(
            context=context["context"],
            url=url(
                "/webdriver/tests/bidi/browsing_context/support/empty.html",
                domain=domain),
            wait="complete")

    return prepare_context


@pytest_asyncio.fixture
async def get_navigation_headers(bidi_session, url):
    async def get_navigation_headers(context):
        echo_link = url("webdriver/tests/support/http_handlers/headers_echo.py")
        await bidi_session.browsing_context.navigate(context=context["context"],
                                                     url=echo_link,
                                                     wait="complete")

        result = await bidi_session.script.evaluate(
            expression="window.document.body.textContent",
            target=ContextTarget(context["context"]),
            await_promise=False,
        )

        return (json.JSONDecoder().decode(result["value"]))["headers"]

    return get_navigation_headers


@pytest_asyncio.fixture(params=["fetch", "navigation"])
def get_headers_methods_invariant(request, get_fetch_headers,
        get_navigation_headers):
    if request.param == "fetch":
        return get_fetch_headers
    if request.param == "navigation":
        return get_navigation_headers
    raise Exception(f"Unsupported getter {request.param}")


@pytest_asyncio.fixture
def assert_header_not_present(get_fetch_headers):
    async def assert_header_not_present(context, header_name):
        actual_headers = await get_fetch_headers(context)
        assert header_name not in actual_headers, f"header '{header_name}' should not be present"

    return assert_header_not_present


@pytest_asyncio.fixture(params=['default', 'new'],
                        ids=["Default user context", "Custom user context"])
async def target_user_context(request, create_user_context):
    return request.param


@pytest_asyncio.fixture
async def affected_user_context(target_user_context, create_user_context):
    """ Returns either a new or default user context. """
    if target_user_context == 'default':
        return 'default'
    return await create_user_context()


@pytest_asyncio.fixture
async def not_affected_user_context(target_user_context, create_user_context):
    """ Returns opposite to `affected_user_context user context. """
    if target_user_context == 'new':
        return 'default'
    return await create_user_context()
