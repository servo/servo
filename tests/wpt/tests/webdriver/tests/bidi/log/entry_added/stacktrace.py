import pytest

from . import assert_console_entry, assert_javascript_entry


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "log_method, expect_stack",
    [
        ("assert", True),
        ("debug", False),
        ("error", True),
        ("info", False),
        ("log", False),
        ("table", False),
        ("trace", True),
        ("warn", True),
    ],
)
async def test_console_entry_sync_callstack(
    bidi_session, subscribe_events, inline, top_context, wait_for_event, log_method, expect_stack
):
    if log_method == "assert":
        # assert has to be called with a first falsy argument to trigger a log.
        url = inline(
            f"""
            <script>
                function foo() {{ console.{log_method}(false, "cheese"); }}
                function bar() {{ foo(); }}
                bar();
            </script>
            """
        )
    else:
        url = inline(
            f"""
            <script>
                function foo() {{ console.{log_method}("cheese"); }}
                function bar() {{ foo(); }}
                bar();
            </script>
            """
        )

    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    if expect_stack:
        expected_stack = [
            {"columnNumber": 41, "functionName": "foo", "lineNumber": 4, "url": url},
            {"columnNumber": 33, "functionName": "bar", "lineNumber": 5, "url": url},
            {"columnNumber": 16, "functionName": "", "lineNumber": 6, "url": url},
        ]
    else:
        expected_stack = None

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    event_data = await on_entry_added

    assert_console_entry(
        event_data,
        method=log_method,
        text="cheese",
        stacktrace=expected_stack,
        context=top_context["context"],
    )

    # Navigate to a page with no error to avoid polluting the next tests with
    # JavaScript errors.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<p>foo"), wait="complete"
    )


@pytest.mark.asyncio
async def test_javascript_entry_sync_callstack(
    bidi_session, subscribe_events, inline, top_context, wait_for_event
):
    url = inline(
        """
        <script>
            function foo() { throw new Error("cheese"); }
            function bar() { foo(); }
            bar();
        </script>
        """
    )

    await subscribe_events(events=["log.entryAdded"])

    on_entry_added = wait_for_event("log.entryAdded")

    expected_stack = [
        {"columnNumber": 35, "functionName": "foo", "lineNumber": 4, "url": url},
        {"columnNumber": 29, "functionName": "bar", "lineNumber": 5, "url": url},
        {"columnNumber": 12, "functionName": "", "lineNumber": 6, "url": url},
    ]

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    event_data = await on_entry_added

    assert_javascript_entry(
        event_data,
        level="error",
        text="Error: cheese",
        stacktrace=expected_stack,
        context=top_context["context"],
    )

    # Navigate to a page with no error to avoid polluting the next tests with
    # JavaScript errors.
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline("<p>foo"), wait="complete"
    )
