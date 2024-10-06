import pytest
import random

from tests.support.sync import AsyncPoll

from .. import (
    assert_response_event,
    get_cached_url,
    PAGE_EMPTY_TEXT,
    RESPONSE_COMPLETED_EVENT,
    SCRIPT_CONSOLE_LOG,
    SCRIPT_CONSOLE_LOG_IN_MODULE,
    STYLESHEET_GREY_BACKGROUND,
    STYLESHEET_RED_COLOR,
)


@pytest.mark.asyncio
async def test_cached(
    wait_for_event,
    wait_for_future_safe,
    url,
    fetch,
    setup_network_test,
):
    network_events = await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=200&nocache={random.random()}"
    )
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": cached_url}

    # The first request/response is used to fill the browser cache, so we expect
    # fromCache to be False here.
    expected_response = {
        "url": cached_url,
        "fromCache": False,
        "status": 200,
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(cached_url)
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 2

    # The second request for the same URL has to be read from the local cache.
    expected_response = {
        "url": cached_url,
        "fromCache": True,
        "status": 200,
    }
    assert_response_event(
        events[1],
        expected_request=expected_request,
        expected_response=expected_response,
    )


@pytest.mark.asyncio
async def test_cached_redirect(
    bidi_session,
    url,
    fetch,
    setup_network_test,
):
    network_events = await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

    text_url = url(PAGE_EMPTY_TEXT)
    cached_url = url(
        f"/webdriver/tests/support/http_handlers/cached.py?status=301&location={text_url}&nocache={random.random()}"
    )

    await fetch(cached_url)

    # Expect two events, one for the initial request and one for the redirect.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    # The first request/response is used to fill the cache, so we expect
    # fromCache to be False here.
    expected_request = {"method": "GET", "url": cached_url}
    expected_response = {
        "url": cached_url,
        "fromCache": False,
        "status": 301,
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    # The second request is the redirect
    redirected_request = {"method": "GET", "url": text_url}
    redirected_response = {"url": text_url, "status": 200}
    assert_response_event(
        events[1],
        expected_request=redirected_request,
        expected_response=redirected_response,
    )

    await fetch(cached_url)
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    # The third request hits cached_url again and has to be read from the local cache.
    expected_response = {
        "url": cached_url,
        "fromCache": True,
        "status": 301,
    }
    assert_response_event(
        events[2],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    # The fourth request is the redirect
    assert_response_event(
        events[3],
        expected_request=redirected_request,
        expected_response=redirected_response,
    )


@pytest.mark.asyncio
async def test_cached_revalidate(
    wait_for_event, wait_for_future_safe, url, fetch, setup_network_test
):
    network_events = await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

    # `nocache` is not used in cached.py, it is here to avoid the browser cache.
    revalidate_url = url(
        f"/webdriver/tests/support/http_handlers/must-revalidate.py?nocache={random.random()}"
    )
    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)
    await fetch(revalidate_url)
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 1
    expected_request = {"method": "GET", "url": revalidate_url}
    expected_response = {
        "url": revalidate_url,
        "fromCache": False,
        "status": 200,
    }
    assert_response_event(
        events[0],
        expected_request=expected_request,
        expected_response=expected_response,
    )

    on_response_completed = wait_for_event(RESPONSE_COMPLETED_EVENT)

    # Note that we pass a specific header so that the must-revalidate.py handler
    # can decide to return a 304 without having to use another URL.
    await fetch(revalidate_url, headers={"return-304": "true"})
    await wait_for_future_safe(on_response_completed)

    assert len(events) == 2

    # Here fromCache should still be false, because for a 304 response the response
    # cache state is "validated" and fromCache is only true if cache state is "local"
    expected_response = {
        "url": revalidate_url,
        "fromCache": False,
        "status": 304,
    }
    assert_response_event(
        events[1],
        expected_request=expected_request,
        expected_response=expected_response,
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
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

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

    assert_response_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )
    assert_response_event(
        events[1],
        expected_request={"method": "GET", "url": cached_link_css_url},
        expected_response={"url": cached_link_css_url, "fromCache": False},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events after reload, for the document and the stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_response_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )
    assert_response_event(
        events[3],
        expected_request={"method": "GET", "url": cached_link_css_url},
        expected_response={"url": cached_link_css_url, "fromCache": True},
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
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

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

    # Expect three events, one for the document, one for the linked stylesheet,
    # one for the imported stylesheet.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 2)
    assert len(events) == 2

    assert_response_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )
    assert_response_event(
        events[1],
        expected_request={"method": "GET", "url": cached_import_css_url},
        expected_response={"url": cached_import_css_url, "fromCache": False},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect three events after reload, for the document and the 2 stylesheets.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_response_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )
    assert_response_event(
        events[3],
        expected_request={"method": "GET", "url": cached_import_css_url},
        expected_response={"url": cached_import_css_url, "fromCache": True},
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
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

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

    assert_response_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )

    link_css_event = next(
        e for e in events if cached_link_css_url == e["request"]["url"]
    )
    assert_response_event(
        link_css_event,
        expected_request={"method": "GET", "url": cached_link_css_url},
        expected_response={"url": cached_link_css_url, "fromCache": False},
    )

    import_css_event = next(
        e for e in events if cached_import_css_url == e["request"]["url"]
    )
    assert_response_event(
        import_css_event,
        expected_request={"method": "GET", "url": cached_import_css_url},
        expected_response={"url": cached_import_css_url, "fromCache": False},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect three events after reload, for the document and the 2 stylesheets.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 6)
    assert len(events) == 6

    # Assert only cached events after reload.
    cached_events = events[3:]

    assert_response_event(
        cached_events[0],
        expected_request={"method": "GET", "url": page_with_cached_css},
        expected_response={"url": page_with_cached_css, "fromCache": False},
    )
    cached_link_css_event = next(
        e for e in cached_events if cached_link_css_url == e["request"]["url"]
    )
    assert_response_event(
        cached_link_css_event,
        expected_request={"method": "GET", "url": cached_link_css_url},
        expected_response={"url": cached_link_css_url, "fromCache": True},
    )
    cached_import_css_event = next(
        e for e in cached_events if cached_import_css_url == e["request"]["url"]
    )
    assert_response_event(
        cached_import_css_event,
        expected_request={"method": "GET", "url": cached_import_css_url},
        expected_response={"url": cached_import_css_url, "fromCache": True},
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
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

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

    assert_response_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_js},
        expected_response={"url": page_with_cached_js, "fromCache": False},
    )
    assert_response_event(
        events[1],
        expected_request={"method": "GET", "url": cached_script_js_url},
        expected_response={"url": cached_script_js_url, "fromCache": False},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events, one for the document and one for the javascript file.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_response_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_js},
        expected_response={"url": page_with_cached_js, "fromCache": False},
    )
    assert_response_event(
        events[3],
        expected_request={"method": "GET", "url": cached_script_js_url},
        expected_response={"url": cached_script_js_url, "fromCache": True},
    )

    page_with_2_cached_js = inline(
        f"""
        <head>
            <script src="{cached_script_js_url}"></script>
            <script src="{cached_script_js_url}"></script>
        </head>
        <body>test page with cached 2 javascript files</body>
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

    assert_response_event(
        cached_events[0],
        expected_request={"method": "GET", "url": page_with_2_cached_js},
        expected_response={"url": page_with_2_cached_js, "fromCache": False},
    )
    assert_response_event(
        cached_events[1],
        expected_request={"method": "GET", "url": cached_script_js_url},
        expected_response={"url": cached_script_js_url, "fromCache": True},
    )
    assert_response_event(
        cached_events[2],
        expected_request={"method": "GET", "url": cached_script_js_url},
        expected_response={"url": cached_script_js_url, "fromCache": True},
    )


@pytest.mark.asyncio
async def test_page_with_cached_javascript_module(
    bidi_session,
    url,
    inline,
    setup_network_test,
    top_context,
):
    network_events = await setup_network_test(
        events=[
            RESPONSE_COMPLETED_EVENT,
        ]
    )
    events = network_events[RESPONSE_COMPLETED_EVENT]

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

    assert_response_event(
        events[0],
        expected_request={"method": "GET", "url": page_with_cached_js_module},
        expected_response={"url": page_with_cached_js_module, "fromCache": False},
    )
    assert_response_event(
        events[1],
        expected_request={"method": "GET", "url": cached_js_module_url},
        expected_response={"url": cached_js_module_url, "fromCache": False},
    )

    # Reload the page.
    await bidi_session.browsing_context.reload(context=top_context["context"])

    # Expect two events, one for the document and one for the javascript module.
    wait = AsyncPoll(bidi_session, timeout=2)
    await wait.until(lambda _: len(events) >= 4)
    assert len(events) == 4

    assert_response_event(
        events[2],
        expected_request={"method": "GET", "url": page_with_cached_js_module},
        expected_response={"url": page_with_cached_js_module, "fromCache": False},
    )
    assert_response_event(
        events[3],
        expected_request={"method": "GET", "url": cached_js_module_url},
        expected_response={"url": cached_js_module_url, "fromCache": True},
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

    assert_response_event(
        cached_events[0],
        expected_request={"method": "GET", "url": page_with_2_cached_js_modules},
        expected_response={"url": page_with_2_cached_js_modules, "fromCache": False},
    )
    assert_response_event(
        cached_events[1],
        expected_request={"method": "GET", "url": cached_js_module_url},
        expected_response={"url": cached_js_module_url, "fromCache": True},
    )
