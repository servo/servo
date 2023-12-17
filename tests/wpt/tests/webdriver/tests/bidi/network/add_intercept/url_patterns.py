import asyncio

import pytest

from .. import assert_before_request_sent_event, BEFORE_REQUEST_SENT_EVENT


@pytest.fixture
def substitute_host(server_config):
    """This test will perform various requests which should not reach the
    external network. All strings refering to a domain will define it as a
    placeholder which needs to be dynamically replaced by a value from the
    current server configuration"""

    def substitute_host(str):
        wpt_host = server_config["browser_host"]
        return str.format(
            wpt_host=wpt_host,
            wpt_host_upper=wpt_host.upper(),
        )

    return substitute_host


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "patterns, url_template",
    [
        ([], "https://{wpt_host}"),
        ([], "https://{wpt_host}/"),
        ([], "https://{wpt_host}:1234/"),
        ([], "https://{wpt_host}/path"),
        ([], "https://{wpt_host}/?search"),
        ([{},], "https://{wpt_host}"),
        ([{},], "https://{wpt_host}/"),
        ([{},], "https://{wpt_host}:1234/"),
        ([{},], "https://{wpt_host}/path"),
        ([{},], "https://{wpt_host}/?search"),
        ([{"protocol": "https"},], "https://{wpt_host}/"),
        ([{"protocol": "https"},], "https://{wpt_host}:1234/"),
        ([{"protocol": "https"},], "https://{wpt_host}/path"),
        ([{"protocol": "https"},], "https://{wpt_host}/?search"),
        ([{"protocol": "HTTPS"},], "https://{wpt_host}/"),
        ([{"hostname": "{wpt_host}"},], "https://{wpt_host}/"),
        ([{"hostname": "{wpt_host}"},], "https://{wpt_host}:1234/"),
        ([{"hostname": "{wpt_host}"},], "https://{wpt_host}/path"),
        ([{"hostname": "{wpt_host}"},], "https://{wpt_host}/?search"),
        ([{"hostname": "{wpt_host}"},], "https://{wpt_host_upper}/"),
        ([{"hostname": "{wpt_host_upper}"},], "https://{wpt_host}/"),
        ([{"port": "1234"},], "https://{wpt_host}:1234/"),
        ([{"pathname": ""},], "https://{wpt_host}"),
        ([{"pathname": ""},], "https://{wpt_host}/"),
        ([{"pathname": "path"},], "https://{wpt_host}/path"),
        ([{"search": ""},], "https://{wpt_host}/"),
        ([{"search": ""},], "https://{wpt_host}/?"),
        ([{"search": "search"},], "https://{wpt_host}/?search"),
    ],
)
async def test_pattern_patterns_matching(
    wait_for_event,
    subscribe_events,
    top_context,
    add_intercept,
    fetch,
    substitute_host,
    wait_for_future_safe,
    patterns,
    url_template,
):
    await subscribe_events(events=[BEFORE_REQUEST_SENT_EVENT], contexts=[top_context["context"]])

    for pattern in patterns:
        for key in pattern:
            pattern[key] = substitute_host(pattern[key])

        pattern.update({"type": "pattern"})

    intercept = await add_intercept(phases=["beforeRequestSent"], url_patterns=patterns)

    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(substitute_host(url_template)))
    event = await wait_for_future_safe(on_network_event)

    assert_before_request_sent_event(event, is_blocked=True, intercepts=[intercept])


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "pattern, url_template",
    [
        ({"protocol": "http"}, "https://{wpt_host}/"),
        ({"hostname": "abc.{wpt_host}"}, "https://{wpt_host}/"),
        ({"hostname": "web-platform"}, "https://{wpt_host}/"),
        ({"hostname": "web-platform.com"}, "https://{wpt_host}/"),
        ({"port": "443"}, "https://{wpt_host}:1234/"),
        ({"port": "1234"}, "https://{wpt_host}/"),
        ({"pathname": ""}, "https://{wpt_host}/path"),
        ({"pathname": "path"}, "https://{wpt_host}/"),
        ({"pathname": "path"}, "https://{wpt_host}/path/"),
        ({"pathname": "path"}, "https://{wpt_host}/other/path"),
        ({"pathname": "path"}, "https://{wpt_host}/path/continued"),
        ({"search": ""}, "https://{wpt_host}/?search"),
        ({"search": "search"}, "https://{wpt_host}/?other"),
    ],
)
async def test_pattern_patterns_not_matching(
    wait_for_event,
    subscribe_events,
    top_context,
    add_intercept,
    fetch,
    substitute_host,
    wait_for_future_safe,
    pattern,
    url_template,
):
    await subscribe_events(events=[BEFORE_REQUEST_SENT_EVENT], contexts=[top_context["context"]])

    for key in pattern:
        pattern[key] = substitute_host(pattern[key])

    pattern.update({"type": "pattern"})

    await add_intercept(phases=["beforeRequestSent"], url_patterns=[pattern])

    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(substitute_host(url_template)))
    event = await wait_for_future_safe(on_network_event)

    assert_before_request_sent_event(event, is_blocked=False)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "pattern, url_template",
    [
        ("https://{wpt_host}/", "https://{wpt_host}/"),
        ("https://{wpt_host}", "https://{wpt_host}/"),
        ("https://{wpt_host}/", "https://{wpt_host}"),
        ("HTTPS://{wpt_host}/", "https://{wpt_host}/"),
        ("https://{wpt_host}/", "HTTPS://{wpt_host}/"),
        ("https://{wpt_host_upper}/", "https://{wpt_host}/"),
        ("https://{wpt_host}/", "https://{wpt_host_upper}/"),
        ("https://user:password@{wpt_host}/", "https://{wpt_host}/"),
        ("https://{wpt_host}/", "https://{wpt_host}:443/"),
        ("https://{wpt_host}:443/", "https://{wpt_host}/"),
        ("https://{wpt_host}:443/", "https://{wpt_host}:443/"),
        ("https://{wpt_host}:1234/", "https://{wpt_host}:1234/"),
        ("https://{wpt_host}/path", "https://{wpt_host}/path"),
        ("https://{wpt_host}/?search", "https://{wpt_host}/?search"),
        ("https://{wpt_host}/#ref", "https://{wpt_host}/"),
        ("https://{wpt_host}/", "https://{wpt_host}/#ref"),
        ("https://{wpt_host}/#ref1", "https://{wpt_host}/#ref2"),
    ],
)
async def test_string_patterns_matching(
    wait_for_event,
    subscribe_events,
    top_context,
    add_intercept,
    fetch,
    substitute_host,
    wait_for_future_safe,
    pattern,
    url_template,
):
    await subscribe_events(events=[BEFORE_REQUEST_SENT_EVENT], contexts=[top_context["context"]])

    intercept = await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": substitute_host(pattern)}],
    )

    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(substitute_host(url_template)))
    event = await wait_for_future_safe(on_network_event)

    assert_before_request_sent_event(event, is_blocked=True, intercepts=[intercept])


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "pattern, url_template",
    [
        ("https://{wpt_host}/", "https://some.other.host/"),
        ("https://{wpt_host}:1234/", "https://{wpt_host}:5678/"),
        ("https://{wpt_host}/", "https://{wpt_host}:5678/"),
        ("https://{wpt_host}/path", "https://{wpt_host}/other/path"),
        ("https://{wpt_host}/path", "https://{wpt_host}/path/continued"),
        ("https://{wpt_host}/pathcase", "https://{wpt_host}/PATHCASE"),
        ("https://{wpt_host}/?searchcase", "https://{wpt_host}/?SEARCHCASE"),
        ("https://{wpt_host}/?key", "https://{wpt_host}/?otherkey"),
        ("https://{wpt_host}/?key", "https://{wpt_host}/?key=value"),
        ("https://{wpt_host}/?a=b&c=d", "https://{wpt_host}/?c=d&a=b"),
        ("https://{wpt_host}/??", "https://{wpt_host}/?"),
    ],
)
async def test_string_patterns_not_matching(
    wait_for_event,
    subscribe_events,
    top_context,
    add_intercept,
    fetch,
    substitute_host,
    wait_for_future_safe,
    pattern,
    url_template,
):
    await subscribe_events(events=[BEFORE_REQUEST_SENT_EVENT], contexts=[top_context["context"]])

    await add_intercept(
        phases=["beforeRequestSent"],
        url_patterns=[{"type": "string", "pattern": substitute_host(pattern)}],
    )

    on_network_event = wait_for_event(BEFORE_REQUEST_SENT_EVENT)
    asyncio.ensure_future(fetch(substitute_host(url_template)))
    event = await wait_for_future_safe(on_network_event)

    assert_before_request_sent_event(event, is_blocked=False)
