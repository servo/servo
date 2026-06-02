import pytest
import pytest_asyncio
import webdriver.bidi.error as error

from tests.bidi import wait_for_bidi_events
from .. import (
    IMAGE_RESPONSE_BODY,
    IMAGE_RESPONSE_DATA,
    RESPONSE_COMPLETED_EVENT,
    SCRIPT_CONSOLE_LOG,
    STYLESHEET_RED_COLOR,
    get_cached_url,
    get_next_event_for_url,
)


@pytest_asyncio.fixture
async def setup_cached_resource_test(bidi_session, top_context, setup_network_test, add_data_collector):
    async def _setup_cached_resource_test(page_url, resource_url):
        network_events = await setup_network_test(
            events=[
                RESPONSE_COMPLETED_EVENT,
            ]
        )
        events = network_events[RESPONSE_COMPLETED_EVENT]

        await bidi_session.browsing_context.navigate(
            context=top_context["context"],
            url=page_url,
            wait="complete",
        )

        # Expect two events, one for the document, one for the resource.
        await wait_for_bidi_events(bidi_session, events, 2, timeout=2)

        collector = await add_data_collector(
            collector_type="blob", data_types=["response"], max_encoded_data_size=1000
        )

        # Reload the page.
        await bidi_session.browsing_context.reload(
            context=top_context["context"], wait="complete"
        )

        # Expect two events after reload, for the document and the resource.
        await wait_for_bidi_events(bidi_session, events, 4, timeout=2)

        # Assert only cached events after reload.
        cached_events = events[2:]
        cached_resource_event = get_next_event_for_url(cached_events, resource_url)

        data = await bidi_session.network.get_data(
            request=cached_resource_event["request"]["request"],
            data_type="response",
            collector=collector,
        )

        return data

    return _setup_cached_resource_test


@pytest.mark.asyncio
async def test_cached_image(
    url,
    inline,
    setup_cached_resource_test,
):
    cached_image_url = url(get_cached_url("img/png", IMAGE_RESPONSE_BODY))
    page_with_cached_image = inline(
        f"""
        <body>
            test page with cached image
            <img src="{cached_image_url}">
        </body>
        """,
    )
    data = await setup_cached_resource_test(page_with_cached_image, cached_image_url)

    assert data["type"] == "base64"
    assert IMAGE_RESPONSE_DATA.decode("utf-8") == data["value"]


@pytest.mark.asyncio
async def test_cached_javascript(
    url,
    inline,
    setup_cached_resource_test,
):
    cached_script_js_url = url(get_cached_url("text/javascript", SCRIPT_CONSOLE_LOG))
    page_with_cached_js = inline(
        f"""
        <head><script src="{cached_script_js_url}"></script></head>
        <body>test page with cached js script file</body>
        """,
    )
    data = await setup_cached_resource_test(page_with_cached_js, cached_script_js_url)

    assert data["type"] == "string"
    assert isinstance(data["value"], str)


@pytest.mark.asyncio
async def test_cached_stylesheet(
    url,
    inline,
    setup_cached_resource_test,
):
    cached_link_css_url = url(get_cached_url("text/css", STYLESHEET_RED_COLOR))
    page_with_cached_css = inline(
        f"""
        <head><link rel="stylesheet" type="text/css" href="{cached_link_css_url}"></head>
        <body>test page with cached link stylesheet</body>
        """,
    )
    data = await setup_cached_resource_test(page_with_cached_css, cached_link_css_url)

    assert data["type"] == "string"
    assert data["value"] == "html, body { color: red; }"
