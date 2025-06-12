import pytest

from webdriver.bidi.modules.script import ContextTarget
from webdriver.error import TimeoutException
from tests.support.sync import AsyncPoll

from ... import any_int, int_interval, recursive_compare

pytestmark = pytest.mark.asyncio

EMPTY_PAGE = "/webdriver/tests/bidi/browsing_context/support/empty.html"
FRAGMENT_NAVIGATED_EVENT = "browsingContext.fragmentNavigated"
HISTORY_UPDATED_EVENT = "browsingContext.historyUpdated"
CREATED_EVENT = "browsingContext.contextCreated"


@pytest.mark.parametrize(
    "hash_before, hash_after, history_method",
    [
        ("", "#foo", "pushState"),
        ("#foo", "#bar", "pushState"),
        ("#foo", "#foo", "pushState"),
        ("#bar", "", "pushState"),
        ("", "#foo", "replaceState"),
        ("#foo", "#bar", "replaceState"),
        ("#foo", "#foo", "replaceState"),
        ("#bar", "", "replaceState"),
    ],
)
async def test_history_url_update(
    bidi_session,
    new_tab,
    url,
    subscribe_events,
    current_time,
    hash_before,
    hash_after,
    history_method,
):
    target_context = new_tab["context"]

    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE + hash_before), wait="complete"
    )

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT, HISTORY_UPDATED_EVENT])

    fragment_navigated_events = []
    history_updated_events = []

    async def on_event(method, data):
        if method == FRAGMENT_NAVIGATED_EVENT:
            fragment_navigated_events.append(data)
        if method == HISTORY_UPDATED_EVENT:
            history_updated_events.append(data)

    remove_fragment_navigated_listener = bidi_session.add_event_listener(
        FRAGMENT_NAVIGATED_EVENT, on_event
    )
    remove_history_updated_listener = bidi_session.add_event_listener(
        HISTORY_UPDATED_EVENT, on_event
    )

    try:
        target_url = url(EMPTY_PAGE + hash_after)

        await bidi_session.script.call_function(
            raw_result=True,
            function_declaration="""(method, url) => {
                history[method](null, null, url);
            }""",
            arguments=[
                {"type": "string", "value": history_method},
                {"type": "string", "value": target_url},
            ],
            await_promise=False,
            target=ContextTarget(target_context),
        )

        recursive_compare(
            [
                {
                    "context": target_context,
                    "timestamp": any_int,
                    "url": target_url,
                }
            ],
            history_updated_events,
        )

        # browsingContext.historyUpdated should not contain any navigation id.
        assert "navigation" not in history_updated_events[0]

        assert len(fragment_navigated_events) == 0
    finally:
        remove_fragment_navigated_listener()
        remove_history_updated_listener()


@pytest.mark.parametrize(
    "history_method",
    [
        ("pushState"),
        ("replaceState"),
    ],
)
async def test_history_state_update(
    bidi_session, new_tab, url, subscribe_events, history_method
):
    target_context = new_tab["context"]

    target_url = url(EMPTY_PAGE)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=target_url, wait="complete"
    )

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT, HISTORY_UPDATED_EVENT])

    fragment_navigated_events = []
    history_updated_events = []

    async def on_event(method, data):
        if method == FRAGMENT_NAVIGATED_EVENT:
            fragment_navigated_events.append(data)
        if method == HISTORY_UPDATED_EVENT:
            history_updated_events.append(data)

    remove_fragment_navigated_listener = bidi_session.add_event_listener(
        FRAGMENT_NAVIGATED_EVENT, on_event
    )
    remove_history_updated_listener = bidi_session.add_event_listener(
        HISTORY_UPDATED_EVENT, on_event
    )

    try:
        await bidi_session.script.call_function(
            raw_result=True,
            function_declaration="""(method) => {
                history[method]({}, null);
            }""",
            arguments=[
                {"type": "string", "value": history_method},
            ],
            await_promise=False,
            target=ContextTarget(target_context),
        )

        recursive_compare(
            [{"context": target_context, "url": target_url}], history_updated_events
        )

        assert len(fragment_navigated_events) == 0
    finally:
        remove_fragment_navigated_listener()
        remove_history_updated_listener()


async def test_history_document_open(bidi_session, new_tab, url, subscribe_events):
    target_context = new_tab["context"]

    target_url = url(EMPTY_PAGE)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=target_url, wait="complete"
    )

    await subscribe_events(
        [FRAGMENT_NAVIGATED_EVENT, HISTORY_UPDATED_EVENT, CREATED_EVENT]
    )

    fragment_navigated_events = []
    history_updated_events = []
    browsing_context_created_events = []

    async def on_event(method, data):
        if method == FRAGMENT_NAVIGATED_EVENT:
            fragment_navigated_events.append(data)
        if method == HISTORY_UPDATED_EVENT:
            history_updated_events.append(data)
        if method == CREATED_EVENT:
            browsing_context_created_events.append(data)

    remove_fragment_navigated_listener = bidi_session.add_event_listener(
        FRAGMENT_NAVIGATED_EVENT, on_event
    )
    remove_history_updated_listener = bidi_session.add_event_listener(
        HISTORY_UPDATED_EVENT, on_event
    )
    remove_created_listener = bidi_session.add_event_listener(CREATED_EVENT, on_event)

    try:
        await bidi_session.script.call_function(
            raw_result=True,
            function_declaration="""() => {
              const frame = document.createElement("iframe");
              document.body.append(frame);
              frame.contentDocument.open();
              return new Promise(resolve => {
                window.onhashchange = () => {
                  frame.contentDocument.open();
                  resolve();
                };
                window.location.hash = "heya";
              });
          }""",
            await_promise=True,
            target=ContextTarget(target_context),
        )

        recursive_compare([{"url": "about:blank"}], browsing_context_created_events)

        recursive_compare(
            [{"context": target_context, "url": target_url + "#heya"}],
            fragment_navigated_events,
        )

        # History updated URL should match the target_context's URL
        # without the fragment per
        # https://html.spec.whatwg.org/#document-open-steps step 12.2.
        recursive_compare(
            [
                {
                    # This event is for the first document.open before setting the
                    # location hash, per spec it should be set to the parent's document
                    # url.
                    "context": browsing_context_created_events[0]["context"],
                    "url": target_url,
                },
                {
                    # This is for the second document.open, after setting the hash.
                    # Again this should be set to target_url since the fragment should not
                    # be included.
                    "context": browsing_context_created_events[0]["context"],
                    "url": target_url,
                },
            ],
            history_updated_events,
        )

    finally:
        remove_fragment_navigated_listener()
        remove_history_updated_listener()
        remove_created_listener()


async def test_history_back_forward(
    bidi_session, new_tab, url, subscribe_events, wait_for_event, wait_for_future_safe
):
    target_context = new_tab["context"]

    target_url = url(EMPTY_PAGE)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=target_url, wait="complete"
    )

    await subscribe_events([FRAGMENT_NAVIGATED_EVENT, HISTORY_UPDATED_EVENT])

    fragment_navigated_events = []
    history_updated_events = []

    async def on_event(method, data):
        if method == FRAGMENT_NAVIGATED_EVENT:
            fragment_navigated_events.append(data)
        elif method == HISTORY_UPDATED_EVENT:
            history_updated_events.append(data)

    remove_fragment_navigated_listener = bidi_session.add_event_listener(
        FRAGMENT_NAVIGATED_EVENT, on_event
    )
    remove_history_updated_listener = bidi_session.add_event_listener(
        HISTORY_UPDATED_EVENT, on_event
    )

    try:
        await bidi_session.script.evaluate(
            expression="""
              history.pushState({}, "", "test1.html");
              history.pushState({}, "", "test2.html");
          """,
            await_promise=False,
            target=ContextTarget(target_context),
        )

        assert len(history_updated_events) == 2
        assert len(fragment_navigated_events) == 0

        on_entry = wait_for_event(HISTORY_UPDATED_EVENT)
        await bidi_session.script.evaluate(
            expression="history.back();",
            await_promise=False,
            target=ContextTarget(target_context),
        )

        await wait_for_future_safe(on_entry)
        assert len(history_updated_events) == 3
        assert len(fragment_navigated_events) == 0

        on_entry = wait_for_event(HISTORY_UPDATED_EVENT)
        await bidi_session.script.evaluate(
            expression="history.forward();",
            await_promise=False,
            target=ContextTarget(target_context),
        )

        await wait_for_future_safe(on_entry)
        assert len(history_updated_events) == 4
        assert len(fragment_navigated_events) == 0
    finally:
        remove_fragment_navigated_listener()
        remove_history_updated_listener()


async def test_timestamp(
    bidi_session,
    current_time,
    subscribe_events,
    url,
    new_tab,
    wait_for_event,
    wait_for_future_safe,
):
    target_context = new_tab["context"]
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url(EMPTY_PAGE), wait="complete"
    )

    await subscribe_events(events=[HISTORY_UPDATED_EVENT])

    on_entry = wait_for_event(HISTORY_UPDATED_EVENT)
    time_start = await current_time()

    await bidi_session.script.evaluate(
        expression="""history.pushState({}, "", "test1.html")""",
        await_promise=False,
        target=ContextTarget(target_context),
    )
    event = await wait_for_future_safe(on_entry)
    time_end = await current_time()

    recursive_compare(
        {
            "context": target_context,
            "timestamp": int_interval(time_start, time_end),
        },
        event,
    )
