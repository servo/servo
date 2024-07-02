import asyncio
import pytest_asyncio

from webdriver.bidi.error import NoSuchInterceptException

from tests.support.sync import AsyncPoll
from . import PAGE_EMPTY_TEXT


@pytest_asyncio.fixture
async def add_intercept(bidi_session):
    """Add a network intercept for the provided phases and url patterns, and
    ensure the intercept is removed at the end of the test."""

    intercepts = []

    async def add_intercept(phases, url_patterns, contexts = None):
        nonlocal intercepts
        intercept = await bidi_session.network.add_intercept(
            phases=phases,
            url_patterns=url_patterns,
            contexts=contexts,
        )
        intercepts.append(intercept)

        return intercept

    yield add_intercept

    # Remove all added intercepts at the end of the test
    for intercept in intercepts:
        try:
            await bidi_session.network.remove_intercept(intercept=intercept)
        except NoSuchInterceptException:
            # Ignore exceptions in case a specific intercept was already removed
            # during the test.
            pass


@pytest_asyncio.fixture
async def setup_blocked_request(
    bidi_session,
    setup_network_test,
    url,
    add_intercept,
    fetch,
    wait_for_event,
    wait_for_future_safe,
    top_context,
):
    """Creates an intercept for the provided phase, sends a fetch request that
    should be blocked by this intercept and resolves when the corresponding
    event is received.

    Pass blocked_url to target a specific URL. Otherwise, the test will use
    PAGE_EMPTY_TEXT as default test url.

    Pass navigate=True in order to navigate instead of doing a fetch request.
    If the navigation url should be different from the blocked url, you can
    specify navigate_url.

    For the "authRequired" phase, the request will be sent to the authentication
    http handler. The optional arguments username, password and realm can be used
    to configure the handler.

    Returns the `request` id of the intercepted request.
    """

    async def setup_blocked_request(
        phase,
        context=top_context,
        username="user",
        password="password",
        realm="test",
        blocked_url=None,
        navigate=False,
        navigate_url=None,
        **kwargs,
    ):
        await setup_network_test(events=[f"network.{phase}"])

        if blocked_url is None:
            if phase == "authRequired":
                blocked_url = url(
                    "/webdriver/tests/support/http_handlers/authentication.py?"
                    f"username={username}&password={password}&realm={realm}"
                )
                if navigate:
                    # By default the authentication handler returns a text/plain
                    # content-type. Switch to text/html for a regular navigation.
                    blocked_url = f"{blocked_url}&contenttype=text/html"
            else:
                blocked_url = url(PAGE_EMPTY_TEXT)

        await add_intercept(
            phases=[phase],
            url_patterns=[
                {
                    "type": "string",
                    "pattern": blocked_url,
                }
            ],
        )

        events = []

        async def on_event(method, data):
            events.append(data)

        remove_listener = bidi_session.add_event_listener(
            f"network.{phase}", on_event
        )


        network_event = wait_for_event(f"network.{phase}")
        if navigate:
            if navigate_url is None:
                navigate_url = blocked_url

            asyncio.ensure_future(
                bidi_session.browsing_context.navigate(
                    context=context["context"], url=navigate_url, wait="complete"
                )
            )
        else:
            asyncio.ensure_future(fetch(blocked_url, context=context, **kwargs))


        # Wait for the first blocked request. When testing a navigation where
        # navigate_url is different from blocked_url, non-blocked events will
        # be received before the blocked request.
        wait = AsyncPoll(bidi_session, timeout=2)
        await wait.until(lambda _: any(e["isBlocked"] is True for e in events))

        [blocked_event] = [e for e in events if e["isBlocked"] is True]
        request = blocked_event["request"]["request"]

        return request

    return setup_blocked_request
