import pytest
import pytest_asyncio

from ... import recursive_compare

pytestmark = pytest.mark.asyncio

USER_PROMPT_OPENED_EVENT = "browsingContext.userPromptOpened"


@pytest_asyncio.fixture
async def check_handler(
    bidi_session,
    subscribe_events,
    inline,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
):
    async def check_handler(expected_handler):
        await subscribe_events(events=[USER_PROMPT_OPENED_EVENT])
        on_entry = wait_for_event(USER_PROMPT_OPENED_EVENT)

        await bidi_session.browsing_context.navigate(
            context=new_tab["context"],
            url=inline(f"<script>window.alert('foo')</script>"),
        )

        event = await wait_for_future_safe(on_entry)

        expected = {
            "context": new_tab["context"],
            "type": "alert",
            "handler": expected_handler,
        }
        recursive_compare(expected, event)

    return check_handler


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept"})
async def test_accept(check_handler):
    await check_handler("accept")


@pytest.mark.capabilities({"unhandledPromptBehavior": "accept and notify"})
async def test_accept_and_notify(check_handler):
    await check_handler("accept")


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss"})
async def test_dismiss(check_handler):
    await check_handler("dismiss")


@pytest.mark.capabilities({"unhandledPromptBehavior": "dismiss and notify"})
async def test_dismiss_and_notify(check_handler):
    await check_handler("dismiss")


@pytest.mark.capabilities({"unhandledPromptBehavior": "ignore"})
async def test_ignore(check_handler):
    await check_handler("ignore")
