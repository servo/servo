import json
import uuid

import pytest_asyncio

from webdriver.bidi.error import UnknownErrorException
from webdriver.bidi.modules.script import ContextTarget, \
    ScriptEvaluateResultException


@pytest_asyncio.fixture
async def get_navigator_online(bidi_session):
    async def get_navigator_online(context):
        result = await bidi_session.script.evaluate(
            expression="navigator.onLine",
            target=ContextTarget(context["context"]),
            await_promise=True,
        )

        return result["value"]

    return get_navigator_online


@pytest_asyncio.fixture
async def get_can_fetch(bidi_session, url):
    async def get_can_fetch(context, fetch_url=None, fetch_options=None):
        if fetch_url is None:
            fetch_url = url(f"/common/blank.html?{uuid.uuid4()}")

        if fetch_options is None:
            function_declaration = "(url)=>fetch(url)"
        else:
            function_declaration = f"(url)=>fetch(url, {json.dumps(fetch_options)})"

        arguments = [{
            "type": "string",
            "value": fetch_url
        }]

        try:
            await bidi_session.script.call_function(
                function_declaration=function_declaration,
                arguments=arguments,
                target=ContextTarget(context["context"]),
                await_promise=True,
            )
            return True
        except ScriptEvaluateResultException:
            return False

    return get_can_fetch


@pytest_asyncio.fixture
async def get_can_navigate(bidi_session, url):
    async def get_can_navigate(context):
        try:
            await bidi_session.browsing_context.navigate(
                context=context["context"],
                url=url(f"/common/blank.html?{uuid.uuid4()}"),
                wait="complete")
            return True
        except UnknownErrorException:
            return False

    return get_can_navigate


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
