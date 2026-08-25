import pytest

import webdriver.bidi.error as error
from tests.bidi import get_invalid_cases

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("value", get_invalid_cases("dict", nullable=True))
async def test_params_download_behavior_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_download_behavior(
            download_behavior=value)


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_download_behavior_type_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_download_behavior(
            download_behavior={"type": value})


async def test_params_download_behavior_type_invalid_value(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_download_behavior(
            download_behavior={"type": "SOME_INVALID_VALUE"})


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_download_behavior_allowed_destination_folder_invalid_value(
        bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_download_behavior(
            download_behavior={
                "type": "allowed",
                "destinationFolder": value
            })


@pytest.mark.parametrize("value", get_invalid_cases("list"))
async def test_params_user_contexts_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_download_behavior(
            download_behavior=None, user_contexts=value)


async def test_params_user_contexts_empty_list(bidi_session):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_download_behavior(
            download_behavior=None, user_contexts=[])


@pytest.mark.parametrize("value", get_invalid_cases("string"))
async def test_params_user_contexts_value_invalid_type(bidi_session, value):
    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browser.set_download_behavior(
            download_behavior=None, user_contexts=[value])


async def test_params_user_contexts_value_unknown_user_context(bidi_session):
    with pytest.raises(error.NoSuchUserContextException):
        await bidi_session.browser.set_download_behavior(
            download_behavior=None, user_contexts=["UNKNOWN_USER_CONTEXT"])
