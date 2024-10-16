import pytest
import random

from tests.support.sync import AsyncPoll

from .. import (
    assert_before_request_sent_event,
    get_cached_url,
    BEFORE_REQUEST_SENT_EVENT,
    SCRIPT_CONSOLE_LOG,
    SCRIPT_CONSOLE_LOG_IN_MODULE,
    STYLESHEET_GREY_BACKGROUND,
    STYLESHEET_RED_COLOR,
)

# Note: The cached status cannot be checked in the beforeRequestSent event, but
# the goal is to verify that the events are still emitted for cached requests.


@pytest.mark.asyncio
async def test_cached_document(
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
    setup_network_test,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    # `nocache` is not used in cached.py, it is here to avoid the browser cache.
    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=200&nocache={random.random()}"
    )
    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": cached_url}

    assert_before_request_sent_event(
        events[0],
        expected_request=expected_request,
    )

    on_before_request_sent = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_before_request_sent)

    assert len(events) == 2

    assert_before_request_sent_event(
        events[1],
        expected_request=expected_request,
    )


@pytest.mark.asyncio
async def test_page_with_cached_link_stylesheet(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    cached_link_css_url = url(get_cached_url("text/css", STYLESHEET_RED_COLOR))
    page_with_cached_css = inline(
        f"""
        <head><link rel="stylesheet" type="text/css" href="{cached_link_css_url}"></head>
        <body>test page with cached link stylesheet</body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_cached_css,
        wait="complete",
    )

    # Expect two events, one for the document, one for the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    assert_before_request_sent_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )
    assert_before_request_sent_event(
        events[1],
        expected_request={"method": "GET", "url": cached_link_css_url},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events after reload, for the document and the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_before_request_sent_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )
    assert_before_request_sent_event(
        events[3],
        expected_request={"method": "GET", "url": cached_link_css_url},
    )


@pytest.mark.asyncio
async def test_page_with_cached_import_stylesheet(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    # Prepare a cached CSS url that will be loaded via @import in a style tag.
    cached_import_css_url = url(get_cached_url("text/css", STYLESHEET_GREY_BACKGROUND))

    page_with_cached_css = inline(
        f"""
        <head>
            <style>
                @import url({cached_import_css_url});
            </style>
        </head>
        <body>test page with cached link and import stylesheet</body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_cached_css,
        wait="complete",
    )

    # Expect two events, one for the document, one for the imported stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    assert_before_request_sent_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )
    assert_before_request_sent_event(
        events[1],
        expected_request={"method": "GET", "url": cached_import_css_url},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events after reload, for the document and the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_before_request_sent_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )
    assert_before_request_sent_event(
        events[3],
        expected_request={"method": "GET", "url": cached_import_css_url},
    )


# Similar test to test_page_with_cached_import_stylesheet, but with 3 links
# loading the same stylesheet, and a style tag with 3 identical imports.
# The browser should not issue requests for the duplicated stylesheets.
@pytest.mark.asyncio
async def test_page_with_cached_duplicated_stylesheets(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    # Prepare a cached CSS url that will be loaded via @import in a style tag.
    cached_import_css_url = url(get_cached_url("text/css", STYLESHEET_GREY_BACKGROUND))

    # Prepare a second cached CSS url, that will be loaded via a <link> tag,
    # three times.
    cached_link_css_url = url(get_cached_url("text/css", STYLESHEET_RED_COLOR))

    page_with_cached_css = inline(
        f"""
        <head>
            <link rel="stylesheet" type="text/css" href="{cached_link_css_url}">
            <link rel="stylesheet" type="text/css" href="{cached_link_css_url}">
            <link rel="stylesheet" type="text/css" href="{cached_link_css_url}">
            <style>
                @import url({cached_import_css_url});
                @import url({cached_import_css_url});
                @import url({cached_import_css_url});
            </style>
        </head>
        <body>test page with cached link and import stylesheet</body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_cached_css,
        wait="complete",
    )

    # Expect three events, one for the document, one for the linked stylesheet,
    # one for the imported stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 3)
    assert len(events) == 3

    assert_before_request_sent_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )

    link_css_event = next(
        e for e in events if cached_link_css_url == e["request"]["url"]
    )
    assert_before_request_sent_event(
        link_css_event,
        expected_request={"method": "GET", "url": cached_link_css_url},
    )

    import_css_event = next(
        e for e in events if cached_import_css_url == e["request"]["url"]
    )
    assert_before_request_sent_event(
        import_css_event,
        expected_request={"method": "GET", "url": cached_import_css_url},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect three events after reload, for the document and the 2 stylesheets.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 6)
    assert len(events) == 6

    # Assert only cached events after reload.
    cached_events = events[3:]

    assert_before_request_sent_event(
        cached_events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
    )
    cached_link_css_event = next(
        e for e in cached_events if cached_link_css_url == e["request"]["url"]
    )
    assert_before_request_sent_event(
        cached_link_css_event,
        expected_request={"method": "GET", "url": cached_link_css_url},
    )
    cached_import_css_event = next(
        e for e in cached_events if cached_import_css_url == e["request"]["url"]
    )
    assert_before_request_sent_event(
        cached_import_css_event,
        expected_request={"method": "GET", "url": cached_import_css_url},
    )


@pytest.mark.asyncio
async def test_page_with_cached_script_javascript(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    cached_script_js_url = url(get_cached_url("text/javascript", SCRIPT_CONSOLE_LOG))
    page_with_cached_js = inline(
        f"""
        <head><script src="{cached_script_js_url}"></script></head>
        <body>test page with cached js script file</body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_cached_js,
        wait="complete",
    )

    # Expect two events, one for the document and one for the javascript file.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    assert_before_request_sent_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_js},
    )
    assert_before_request_sent_event(
        events[1],
        expected_request={"method": "GET", "url": cached_script_js_url},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events, one for the document and one for the javascript file.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_before_request_sent_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_js},
    )
    assert_before_request_sent_event(
        events[3],
        expected_request={"method": "GET", "url": cached_script_js_url},
    )

    page_with_2_cached_js = inline(
        f"""
        <head>
            <script src="{cached_script_js_url}"></script>
            <script src="{cached_script_js_url}"></script>
        </head>
        <body>test page with 2 cached javascript files</body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_2_cached_js,
        wait="complete",
    )

    # Expect three events, one for the document and two for script javascript files.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 7)
    assert len(events) == 7

    # Assert only cached events after reload.
    cached_events = events[4:]

    assert_before_request_sent_event(
        cached_events[0],
        expected_request={"method": "GET", "url": page_with_2_cached_js},
    )
    assert_before_request_sent_event(
        cached_events[1],
        expected_request={"method": "GET", "url": cached_script_js_url},
    )
    assert_before_request_sent_event(
        cached_events[2],
        expected_request={"method": "GET", "url": cached_script_js_url},
    )


@pytest.mark.asyncio
async def tst_page_with_cached_javascript_module(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
):
    network_events = await setup_network_test(
        events=[
            BEFORE_REQUEST_SENT_EVENT,
        ]
    )
    events = network_events[BEFORE_REQUEST_SENT_EVENT]

    cached_js_module_url = url(
        get_cached_url("text/javascript", SCRIPT_CONSOLE_LOG_IN_MODULE)
    )
    page_with_cached_js_module = inline(
        f"""
        <body>
            test page with cached js module
            <script type="module">
                import foo from "{cached_js_module_url}";
                foo();
            </script>
        </body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_cached_js_module,
        wait="complete",
    )

    # Expect two events, one for the document and one for the javascript module.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    assert_before_request_sent_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_js_module},
    )
    assert_before_request_sent_event(
        events[1],
        expected_request={"method": "GET", "url": cached_js_module_url},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events, one for the document and one for the javascript module.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_before_request_sent_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_js_module},
    )
    assert_before_request_sent_event(
        events[3],
        expected_request={"method": "GET", "url": cached_js_module_url},
    )

    page_with_2_cached_js_modules = inline(
        f"""
        <body>
            test page with 2 cached javascript modules
            <script type="module">
                import foo from "{cached_js_module_url}";
                foo();
            </script>
            <script type="module">
                import foo from "{cached_js_module_url}";
                foo();
            </script>
        </body>
        """,
    )

    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=page_with_2_cached_js_modules,
        wait="complete",
    )

    # Expect two events, one for the document and one for the javascript module.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 6)
    assert len(events) == 6

    # Assert only cached events after reload.
    cached_events = events[4:]

    assert_before_request_sent_event(
        cached_events[0],
        expected_request={"method": "GET", "url": page_with_2_cached_js_modules},
    )
    assert_before_request_sent_event(
        cached_events[1],
        expected_request={"method": "GET", "url": cached_js_module_url},
    )
