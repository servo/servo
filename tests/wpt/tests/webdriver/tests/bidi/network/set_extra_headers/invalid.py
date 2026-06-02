import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases
from webdriver.bidi.undefined import UNDEFINED

pytestmark = pytest.mark.asyncio

VALID_HEADERS = [{
    "name": "some_name",
    "value": {
        "type": "string",
        "value": "come_value"
    }}]


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_contexts_invalid_type(bidi_session, value,
        set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            contexts=value
        )


async def test_params_contexts_empty_list(bidi_session, set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            contexts=[])


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_contexts_entry_invalid_type(bidi_session, value,
        set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            contexts=[value])


async def test_params_contexts_entry_invalid_value(bidi_session,
        set_extra_headers):
    with pytest.raises(error.NoSuchFrameException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            contexts=["_invalid_"],
        )


async def test_params_contexts_iframe(bidi_session, new_tab, get_test_page,
        set_extra_headers):
    url = get_test_page(as_frame=True)
    await bidi_session.browsing_context.navigate(
        context=new_tab["context"], url=url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(
        root=new_tab["context"])
    assert len(contexts) == 1
    frames = contexts[0]["children"]
    assert len(frames) == 1

    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            contexts=[frames[0]["context"]],
        )


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_user_contexts_invalid_type(bidi_session, value,
        set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            user_contexts=value,
        )


async def test_params_user_contexts_empty_list(bidi_session, set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            user_contexts=[],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_user_contexts_entry_invalid_type(bidi_session, value,
        set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            user_contexts=[value],
        )


@pytest.mark.parametrize("value", ["", "somestring"])
async def test_params_user_contexts_entry_invalid_value(bidi_session, value,
        set_extra_headers):
    with pytest.raises(error.NoSuchUserContextException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            user_contexts=[value],
        )


async def test_params_contexts_and_user_contexts(bidi_session, top_context,
        set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=VALID_HEADERS,
            contexts=[top_context["context"]],
            user_contexts=["default"],
        )


async def test_params_headers_missing(bidi_session, top_context,
        set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(headers=UNDEFINED)


@pytest.mark.parametrize("value", get_invalid_cases("list", nullable=False))
async def test_params_headers_invalid_type(bidi_session, top_context, value,
        set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(headers=value)


@pytest.mark.parametrize("value", get_invalid_cases("dict", nullable=False))
async def test_params_headers_header_invalid_type(bidi_session, top_context,
        value, set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(headers=[value])


@pytest.mark.parametrize("value", get_invalid_cases("string", nullable=False))
async def test_params_headers_header_name_invalid_type(bidi_session,
        top_context, value, set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=[{"name": value, "value": "some_value"}],
        )


# https://fetch.spec.whatwg.org/#header-name
@pytest.mark.parametrize("value",
                         ["", " ", "\t", "\n", '"', '(', ')', ',', '/', ':',
                          ';', '<', '=', '>', '?', '@', '[', '\\', ']', '{',
                          '}'])
async def test_params_headers_header_name_invalid_value(bidi_session,
        top_context, value, set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=[{"name": value, "value": "some_value"}],
        )


@pytest.mark.parametrize("value", get_invalid_cases("string", nullable=False))
async def test_params_headers_header_value_invalid_type(bidi_session,
        top_context, value, set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=[{"name": "some_name", "value": value}],
            contexts=[top_context["context"]],
        )


# https://fetch.spec.whatwg.org/#header-value
@pytest.mark.parametrize("value", [" value",  # leading space
                                   "value ",  # trailing space
                                   "\tvalue",  # leading tab
                                   "value\t",  # trailing tab
                                   "va\0lue",  # NUL
                                   "va\nlue",  # LF
                                   "va\rlue",  # CR
                                   ])
async def test_params_headers_header_value_invalid_value(bidi_session,
        top_context, value, set_extra_headers):
    with pytest.raises(error.InvalidArgumentException):
        await set_extra_headers(
            headers=[{"name": "some_name", "value": value}],
            contexts=[top_context["context"]],
        )
