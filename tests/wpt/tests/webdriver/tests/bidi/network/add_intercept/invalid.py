import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", [None, "foo", False, 42, {}])
async def test_params_phases_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(phases=value)


async def test_params_phases_invalid_value_empty_array(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(phases=[])


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_phases_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(phases=[value])


@pytest.mark.parametrize("value", ["foo", "responseCompleted"])
async def test_params_phases_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(phases=[value])


@pytest.mark.parametrize("value", ["foo", False, 42, {}])
async def test_params_url_patterns_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"], url_patterns=value
        )


@pytest.mark.parametrize("value", [None, "foo", False, 42, []])
async def test_params_url_patterns_entry_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"], url_patterns=[value]
        )


@pytest.mark.parametrize("value", [{}, {"type": "foo"}])
async def test_params_url_patterns_entry_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"], url_patterns=[value]
        )


@pytest.mark.parametrize("value", [None, False, 42, [], {}])
async def test_params_url_patterns_string_pattern_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "string", "pattern": value}],
        )


@pytest.mark.parametrize(
    "value",
    [
        "foo",
        "*",
        "(",
        ")",
        "{",
        "}",
        "http\\{s\\}://example.com",
        "https://example.com:port/",
    ],
)
async def test_params_url_patterns_string_pattern_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "string", "pattern": value}],
        )


@pytest.mark.parametrize(
    "property", ["protocol", "hostname", "port", "pathname", "search"]
)
@pytest.mark.parametrize("value", [False, 42, [], {}])
async def test_params_url_patterns_pattern_property_invalid_type(
    bidi_session, property, value
):
    with pytest.raises(error.InvalidArgumentException):
        url_pattern = {"type": "pattern"}
        url_pattern[property] = value
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[url_pattern],
        )


@pytest.mark.parametrize(
    "property", ["protocol", "hostname", "port", "pathname", "search"]
)
@pytest.mark.parametrize("value", ["*", "(", ")", "{", "}"])
async def test_params_url_patterns_pattern_property_unescaped_character(
    bidi_session, property, value
):
    with pytest.raises(error.InvalidArgumentException):
        url_pattern = {"type": "pattern"}
        url_pattern[property] = value
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[url_pattern],
        )


@pytest.mark.parametrize(
    "value",
    [
        "",
        "http/",
        "http\\*",
        "http\\(",
        "http\\)",
        "http\\{",
        "http\\}",
        "http#",
        "http@",
        "http%",
    ],
)
async def test_params_url_patterns_pattern_protocol_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "pattern", "protocol": value}],
        )


@pytest.mark.parametrize(
    "value",
    [
        "file",
        "file:",
    ],
)
async def test_params_url_patterns_pattern_protocol_file_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "pattern", "protocol": value, "hostname": "example.com"}],
        )


@pytest.mark.parametrize("value", ["", "abc/com/", "abc?com", "abc#com", "abc:com", "abc::com", "::1"])
async def test_params_url_patterns_pattern_hostname_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "pattern", "hostname": value}],
        )


@pytest.mark.parametrize("value", ["", "abcd", "-1", "80 ", "1.3", ":80", "80:", "65536"])
async def test_params_url_patterns_pattern_port_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "pattern", "port": value}],
        )


@pytest.mark.parametrize("value", ["path?", "path#"])
async def test_params_url_patterns_pattern_pathname_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "pattern", "pathname": value}],
        )


@pytest.mark.parametrize("value", ["search#"])
async def test_params_url_patterns_pattern_search_invalid_value(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.network.add_intercept(
            phases=["beforeRequestSent"],
            url_patterns=[{"type": "pattern", "search": value}],
        )
